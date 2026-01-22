//! Service installation and management for auto-start on login.
//!
//! Supports:
//! - macOS: LaunchAgent via launchd

use std::path::PathBuf;

const SERVICE_LABEL: &str = "sh.getjolt.daemon";

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
    install_macos_service(force)
}

pub fn uninstall_service() -> Result<()> {
    uninstall_macos_service()
}

pub fn get_service_status() -> ServiceStatus {
    get_macos_service_status()
}

// macOS: launchd LaunchAgent

fn macos_plist_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| {
        PathBuf::from(std::env::var("HOME").expect("HOME environment variable not set"))
    });

    home.join("Library/LaunchAgents")
        .join(format!("{}.plist", SERVICE_LABEL))
}

fn macos_log_dir() -> PathBuf {
    crate::config::runtime_dir()
}

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

fn is_macos_service_loaded() -> bool {
    let output = std::process::Command::new("launchctl")
        .args(["list", SERVICE_LABEL])
        .output();

    matches!(output, Ok(o) if o.status.success())
}

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
        assert!(path.to_string_lossy().contains("sh.getjolt.daemon.plist"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_generate_macos_plist() {
        let exe = PathBuf::from("/usr/local/bin/jolt");
        let plist = generate_macos_plist(&exe);
        assert!(plist.contains("sh.getjolt.daemon"));
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

}
