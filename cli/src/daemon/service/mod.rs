//! Service installation and management for auto-start on login.
//!
//! Supports:
//! - macOS: LaunchAgent via launchd
//! - Linux: systemd user service

#[cfg(target_os = "linux")]
pub(super) mod linux;
#[cfg(target_os = "macos")]
pub(super) mod macos;

use color_eyre::eyre::Result;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use crate::daemon::service::linux::get_linux_service_status as _get_service_status;

#[cfg(target_os = "macos")]
use crate::daemon::service::macos::{
    get_mocos_service_status as _get_service_status, install_macos_service, uninstall_macos_service,
};

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

#[cfg(target_os = "macos")]
pub fn install_service(force: bool) -> Result<()> {
    install_macos_service(force)
}

#[cfg(target_os = "macos")]
pub fn uninstall_service() -> Result<()> {
    uninstall_macos_service()
}

pub fn get_service_status() -> ServiceStatus {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        _get_service_status()
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

pub fn disable_service() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        linux::disable_linux_service()
    }
    #[cfg(target_os = "macos")]
    {
        macos::unload_macos_service()
    }
}

#[cfg(test)]
mod tests {
    use crate::daemon::service::ServiceStatus;
    use std::path::PathBuf;

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
}
