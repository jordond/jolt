// Linux: systemd user service

use std::path::PathBuf;
use color_eyre::eyre::{eyre, Result};
use crate::daemon::service::ServiceStatus;

const SYSTEMD_SERVICE_NAME: &str = "jolt-daemon.service";


fn linux_service_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join(".config")
    });

    config_dir.join("systemd/user").join(SYSTEMD_SERVICE_NAME)
}


pub fn disable_linux_service() -> Result<()> {
    let status = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", SYSTEMD_SERVICE_NAME])
        .status()?;

    if !status.success() {
        return Err(eyre!("Failed to disable service"));
    }

    Ok(())
}


fn is_systemd_available() -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "status"])
        .output()
        .map(|o| o.status.code() != Some(127))
        .unwrap_or(false)
}


pub fn is_linux_service_enabled() -> bool {
    let output = std::process::Command::new("systemctl")
        .args(["--user", "is-enabled", SYSTEMD_SERVICE_NAME])
        .output();

    matches!(output, Ok(o) if o.status.success())
}


pub fn is_linux_service_active() -> bool {
    let output = std::process::Command::new("systemctl")
        .args(["--user", "is-active", SYSTEMD_SERVICE_NAME])
        .output();

    matches!(output, Ok(o) if o.status.success())
}


pub fn get_linux_service_status() -> ServiceStatus {
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
    use std::path::PathBuf;
    use super::*;

    // #[test]
    // fn test_linux_service_path() {
    //     let path = linux_service_path();
    //     assert!(path.to_string_lossy().contains("systemd/user"));
    //     assert!(path.to_string_lossy().contains("jolt-daemon.service"));
    // }

    #[test]
    fn test_extract_exe_from_systemd_service() {
        let service = "ExecStart=/usr/local/bin/jolt-daemon start --foreground";
        let exe = extract_exe_from_systemd_service(service);
        assert_eq!(exe, Some(PathBuf::from("/usr/local/bin/jolt-daemon")));
    }
}