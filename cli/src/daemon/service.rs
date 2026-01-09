//! Service installation and management for auto-start on login.
//!
//! Supports:
//! - macOS: LaunchAgent via launchd
//! - Linux: systemd user service

use std::path::PathBuf;

use color_eyre::eyre::{eyre, Result};

const SERVICE_LABEL: &str = "com.jolt.daemon";

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub installed: bool,
    pub enabled: bool,
    pub running: bool,
    pub config_path: PathBuf,
    pub configured_exe: Option<PathBuf>,
    pub warnings: Vec<String>,
}

impl ServiceStatus {
    pub fn display(&self) -> String {
        let mut lines = vec![
            format!(
                "Service installed: {}",
                if self.installed { "yes" } else { "no" }
            ),
            format!(
                "Service enabled:   {}",
                if self.enabled { "yes" } else { "no" }
            ),
            format!(
                "Service running:   {}",
                if self.running { "yes" } else { "no" }
            ),
            format!("Config path:       {}", self.config_path.display()),
        ];

        if let Some(exe) = &self.configured_exe {
            lines.push(format!("Configured exe:    {}", exe.display()));
        }

        for warning in &self.warnings {
            lines.push(format!("Warning:           {}", warning));
        }

        lines.join("\n")
    }
}

fn warn_if_dev_binary(exe_path: &std::path::Path) {
    let exe_str = exe_path.to_string_lossy();
    if exe_str.contains("/target/debug/") || exe_str.contains("/target/release/") {
        eprintln!(
            "Warning: Using development binary at {}\n\
             Consider installing jolt to a stable location (e.g., /usr/local/bin/jolt)",
            exe_path.display()
        );
    }
}

pub fn install_service(force: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        install_macos_service(force)
    }
    #[cfg(target_os = "linux")]
    {
        install_linux_service(force)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(eyre!("Service installation not supported on this platform"))
    }
}

pub fn uninstall_service() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        uninstall_macos_service()
    }
    #[cfg(target_os = "linux")]
    {
        uninstall_linux_service()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(eyre!(
            "Service uninstallation not supported on this platform"
        ))
    }
}

pub fn get_service_status() -> ServiceStatus {
    #[cfg(target_os = "macos")]
    {
        get_macos_service_status()
    }
    #[cfg(target_os = "linux")]
    {
        get_linux_service_status()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        ServiceStatus {
            installed: false,
            enabled: false,
            running: false,
            config_path: PathBuf::new(),
            configured_exe: None,
            warnings: vec!["Service management not supported on this platform".to_string()],
        }
    }
}

// macOS: launchd LaunchAgent

#[cfg(target_os = "macos")]
fn macos_plist_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| {
        PathBuf::from(std::env::var("HOME").expect("HOME environment variable not set"))
    });

    home.join("Library/LaunchAgents")
        .join(format!("{}.plist", SERVICE_LABEL))
}

#[cfg(target_os = "macos")]
fn macos_log_dir() -> PathBuf {
    crate::config::runtime_dir()
}

#[cfg(target_os = "macos")]
fn generate_macos_plist(exe_path: &std::path::Path) -> String {
    let log_dir = macos_log_dir();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>daemon</string>
        <string>start</string>
        <string>--foreground</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>10</integer>
    <key>StandardErrorPath</key>
    <string>{log_dir}/jolt-daemon-stderr.log</string>
    <key>StandardOutPath</key>
    <string>{log_dir}/jolt-daemon-stdout.log</string>
</dict>
</plist>"#,
        label = SERVICE_LABEL,
        exe = exe_path.display(),
        log_dir = log_dir.display()
    )
}

#[cfg(target_os = "macos")]
fn install_macos_service(force: bool) -> Result<()> {
    let plist_path = macos_plist_path();
    let exe_path = std::env::current_exe()?;

    if plist_path.exists() && !force {
        return Err(eyre!(
            "Service already installed at: {}\nUse --force to overwrite",
            plist_path.display()
        ));
    }

    warn_if_dev_binary(&exe_path);

    if is_macos_service_loaded() {
        let _ = unload_macos_service();
    }

    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let log_dir = macos_log_dir();
    std::fs::create_dir_all(&log_dir)?;

    let plist_content = generate_macos_plist(&exe_path);
    std::fs::write(&plist_path, plist_content)?;

    let uid = get_uid();
    let bootstrap_result = std::process::Command::new("launchctl")
        .args(["bootstrap", &format!("gui/{}", uid)])
        .arg(&plist_path)
        .output();

    let (loaded, error_msg) = match bootstrap_result {
        Ok(output) if output.status.success() => (true, None),
        _ => {
            let legacy_result = std::process::Command::new("launchctl")
                .args(["load", "-w"])
                .arg(&plist_path)
                .output();

            match legacy_result {
                Ok(output) if output.status.success() => (true, None),
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    (
                        false,
                        Some(format!("Legacy 'launchctl load' failed: {}", stderr.trim())),
                    )
                }
                Err(e) => (
                    false,
                    Some(format!("Failed to execute 'launchctl load': {}", e)),
                ),
            }
        }
    };

    if !loaded {
        let msg = error_msg.unwrap_or_else(|| "Failed to load service with launchctl".to_string());
        return Err(eyre!(msg));
    }

    println!("Daemon installed and started.");
    println!("Plist: {}", plist_path.display());
    println!("\nThe daemon will now start automatically on login.");
    println!("To uninstall: jolt daemon uninstall");

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_macos_service() -> Result<()> {
    let plist_path = macos_plist_path();

    if !plist_path.exists() {
        println!("Service is not installed.");
        return Ok(());
    }

    if is_macos_service_loaded() {
        unload_macos_service()?;
    }

    std::fs::remove_file(&plist_path)?;

    println!("Daemon uninstalled.");
    println!("The daemon will no longer start automatically on login.");

    Ok(())
}

#[cfg(target_os = "macos")]
fn unload_macos_service() -> Result<()> {
    let plist_path = macos_plist_path();
    let uid = get_uid();

    let bootout_result = std::process::Command::new("launchctl")
        .args(["bootout", &format!("gui/{}/{}", uid, SERVICE_LABEL)])
        .output();

    if let Ok(output) = bootout_result {
        if output.status.success() {
            return Ok(());
        }
    }

    let status = std::process::Command::new("launchctl")
        .arg("unload")
        .arg(&plist_path)
        .status()?;

    if !status.success() {
        return Err(eyre!("Failed to unload service"));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn is_macos_service_loaded() -> bool {
    let output = std::process::Command::new("launchctl")
        .args(["list", SERVICE_LABEL])
        .output();

    matches!(output, Ok(o) if o.status.success())
}

#[cfg(target_os = "macos")]
fn get_macos_service_status() -> ServiceStatus {
    let plist_path = macos_plist_path();
    let installed = plist_path.exists();
    let enabled = is_macos_service_loaded();
    let running = crate::daemon::is_daemon_running();

    let mut warnings = Vec::new();
    let mut configured_exe = None;

    if installed {
        if let Ok(content) = std::fs::read_to_string(&plist_path) {
            configured_exe = extract_exe_from_plist(&content);
            if let Some(ref exe) = configured_exe {
                if !exe.exists() {
                    warnings.push(format!(
                        "Configured executable not found: {}",
                        exe.display()
                    ));
                }
            }
        }
    }

    ServiceStatus {
        installed,
        enabled,
        running,
        config_path: plist_path,
        configured_exe,
        warnings,
    }
}

#[cfg(target_os = "macos")]
fn extract_exe_from_plist(content: &str) -> Option<PathBuf> {
    let start = content.find("<array>")?;
    let end = content[start..].find("</array>")?;
    let array_content = &content[start..start + end];

    let exe_start = array_content.find("<string>")?;
    let exe_end = array_content[exe_start + 8..].find("</string>")?;
    let exe_str = &array_content[exe_start + 8..exe_start + 8 + exe_end];

    Some(PathBuf::from(exe_str))
}

#[cfg(target_os = "macos")]
fn get_uid() -> u32 {
    // SAFETY: `libc::getuid` is a simple read-only syscall that returns the
    // calling process's user ID. It does not dereference pointers or rely on
    // any Rust-side invariants, so it cannot cause undefined behavior.
    unsafe { libc::getuid() }
}

// Linux: systemd user service

#[cfg(target_os = "linux")]
const SYSTEMD_SERVICE_NAME: &str = "jolt-daemon.service";

#[cfg(target_os = "linux")]
fn linux_service_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join(".config")
    });

    config_dir.join("systemd/user").join(SYSTEMD_SERVICE_NAME)
}

#[cfg(target_os = "linux")]
fn generate_linux_service(exe_path: &std::path::Path) -> String {
    format!(
        r#"[Unit]
Description=Jolt Battery Monitor Daemon
Documentation=https://github.com/jordond/jolt
After=graphical-session.target

[Service]
Type=simple
ExecStart={exe} daemon start --foreground
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
"#,
        exe = exe_path.display()
    )
}

#[cfg(target_os = "linux")]
fn install_linux_service(force: bool) -> Result<()> {
    if !is_systemd_available() {
        return Err(eyre!(
            "systemd not available. Service installation requires systemd.\n\
             For other init systems, please configure auto-start manually."
        ));
    }

    let service_path = linux_service_path();
    let exe_path = std::env::current_exe()?;

    if service_path.exists() && !force {
        return Err(eyre!(
            "Service already installed at: {}\nUse --force to overwrite",
            service_path.display()
        ));
    }

    warn_if_dev_binary(&exe_path);

    if is_linux_service_enabled() {
        let _ = disable_linux_service();
    }

    if let Some(parent) = service_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let service_content = generate_linux_service(&exe_path);
    std::fs::write(&service_path, service_content)?;

    let reload = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;

    if !reload.success() {
        return Err(eyre!("Failed to reload systemd daemon"));
    }

    let enable = std::process::Command::new("systemctl")
        .args(["--user", "enable", "--now", SYSTEMD_SERVICE_NAME])
        .status()?;

    if !enable.success() {
        return Err(eyre!("Failed to enable and start service"));
    }

    println!("Daemon installed and started.");
    println!("Service file: {}", service_path.display());
    println!("\nThe daemon will now start automatically on login.");
    println!("To uninstall: jolt daemon uninstall");

    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_linux_service() -> Result<()> {
    let service_path = linux_service_path();

    if !service_path.exists() {
        println!("Service is not installed.");
        return Ok(());
    }

    if is_linux_service_enabled() {
        disable_linux_service()?;
    }

    std::fs::remove_file(&service_path)?;

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    println!("Daemon uninstalled.");
    println!("The daemon will no longer start automatically on login.");

    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_linux_service() -> Result<()> {
    let status = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", SYSTEMD_SERVICE_NAME])
        .status()?;

    if !status.success() {
        return Err(eyre!("Failed to disable service"));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn is_systemd_available() -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "status"])
        .output()
        .map(|o| o.status.code() != Some(127))
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn is_linux_service_enabled() -> bool {
    let output = std::process::Command::new("systemctl")
        .args(["--user", "is-enabled", SYSTEMD_SERVICE_NAME])
        .output();

    matches!(output, Ok(o) if o.status.success())
}

#[cfg(target_os = "linux")]
fn is_linux_service_active() -> bool {
    let output = std::process::Command::new("systemctl")
        .args(["--user", "is-active", SYSTEMD_SERVICE_NAME])
        .output();

    matches!(output, Ok(o) if o.status.success())
}

#[cfg(target_os = "linux")]
fn get_linux_service_status() -> ServiceStatus {
    let service_path = linux_service_path();
    let installed = service_path.exists();

    let (enabled, running) = if is_systemd_available() {
        (is_linux_service_enabled(), is_linux_service_active())
    } else {
        (false, crate::daemon::is_daemon_running())
    };

    let mut warnings = Vec::new();
    let mut configured_exe = None;

    if installed {
        if let Ok(content) = std::fs::read_to_string(&service_path) {
            configured_exe = extract_exe_from_systemd_service(&content);
            if let Some(ref exe) = configured_exe {
                if !exe.exists() {
                    warnings.push(format!(
                        "Configured executable not found: {}",
                        exe.display()
                    ));
                }
            }
        }
    }

    if !is_systemd_available() {
        warnings.push("systemd not available on this system".to_string());
    }

    ServiceStatus {
        installed,
        enabled,
        running,
        config_path: service_path,
        configured_exe,
        warnings,
    }
}

#[cfg(target_os = "linux")]
fn extract_exe_from_systemd_service(content: &str) -> Option<PathBuf> {
    for line in content.lines() {
        if let Some(exec_start) = line.strip_prefix("ExecStart=") {
            if let Some(exe_str) = exec_start.split_whitespace().next() {
                return Some(PathBuf::from(exe_str));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_display() {
        let status = ServiceStatus {
            installed: true,
            enabled: true,
            running: false,
            config_path: PathBuf::from("/test/path"),
            configured_exe: Some(PathBuf::from("/usr/local/bin/jolt")),
            warnings: vec!["Test warning".to_string()],
        };

        let display = status.display();
        assert!(display.contains("Service installed: yes"));
        assert!(display.contains("Service enabled:   yes"));
        assert!(display.contains("Service running:   no"));
        assert!(display.contains("Test warning"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_plist_path() {
        let path = macos_plist_path();
        assert!(path.to_string_lossy().contains("LaunchAgents"));
        assert!(path.to_string_lossy().contains("com.jolt.daemon.plist"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_generate_macos_plist() {
        let exe = PathBuf::from("/usr/local/bin/jolt");
        let plist = generate_macos_plist(&exe);
        assert!(plist.contains("com.jolt.daemon"));
        assert!(plist.contains("/usr/local/bin/jolt"));
        assert!(plist.contains("RunAtLoad"));
        assert!(plist.contains("KeepAlive"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_extract_exe_from_plist() {
        let plist = r#"<array>
            <string>/usr/local/bin/jolt</string>
            <string>daemon</string>
        </array>"#;
        let exe = extract_exe_from_plist(plist);
        assert_eq!(exe, Some(PathBuf::from("/usr/local/bin/jolt")));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_service_path() {
        let path = linux_service_path();
        assert!(path.to_string_lossy().contains("systemd/user"));
        assert!(path.to_string_lossy().contains("jolt-daemon.service"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_generate_linux_service() {
        let exe = PathBuf::from("/usr/local/bin/jolt");
        let service = generate_linux_service(&exe);
        assert!(service.contains("Jolt Battery Monitor Daemon"));
        assert!(service.contains("/usr/local/bin/jolt"));
        assert!(service.contains("Restart=on-failure"));
        assert!(service.contains("WantedBy=default.target"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_extract_exe_from_systemd_service() {
        let service = "ExecStart=/usr/local/bin/jolt daemon start --foreground";
        let exe = extract_exe_from_systemd_service(service);
        assert_eq!(exe, Some(PathBuf::from("/usr/local/bin/jolt")));
    }
}
