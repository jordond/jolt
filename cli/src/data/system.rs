#[cfg(target_os = "macos")]
use std::process::Command;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::daemon::SystemSnapshot;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub chip: String,
    pub os_name: String,
    pub os_version: String,
    pub p_cores: u32,
    pub e_cores: u32,
}

impl SystemInfo {
    pub fn new() -> Self {
        let (chip, p_cores, e_cores) = get_chip_info();
        let (os_name, os_version) = get_os_info();

        Self {
            chip,
            os_name,
            os_version,
            p_cores,
            e_cores,
        }
    }

    pub fn cores_display(&self) -> String {
        if self.p_cores > 0 && self.e_cores > 0 {
            format!("{}P+{}E", self.p_cores, self.e_cores)
        } else {
            format!("{}", self.p_cores + self.e_cores)
        }
    }

    pub fn update_from_snapshot(&mut self, snapshot: &SystemSnapshot) {
        self.chip = snapshot.chip.clone();
        self.os_name = snapshot.os_name.clone();
        self.os_version = snapshot.os_version.clone();
        self.p_cores = snapshot.p_cores;
        self.e_cores = snapshot.e_cores;
    }
}

fn get_chip_info() -> (String, u32, u32) {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
    );
    system.refresh_cpu_all();

    let cpus = system.cpus();
    let chip = if let Some(first_cpu) = cpus.first() {
        clean_chip_name(first_cpu.brand())
    } else {
        "Unknown".to_string()
    };

    // Fallback to sysctl on macOS if sysinfo returns generic name or for P/E cores
    #[cfg(target_os = "macos")]
    {
        let p_cores = get_sysctl_int("hw.perflevel0.physicalcpu").unwrap_or(0);
        let e_cores = get_sysctl_int("hw.perflevel1.physicalcpu").unwrap_or(0);
        if p_cores > 0 || e_cores > 0 {
             // If we have P/E cores, we trust sysctl more for the count
             return (chip, p_cores, e_cores);
        }
    }

    let physical_cores = System::physical_core_count().unwrap_or(cpus.len());
    
    // On non-macOS or if sysctl failed, we treat all as P-cores for now
    (chip, physical_cores as u32, 0)
}

fn clean_chip_name(name: &str) -> String {
    name.replace("Apple ", "").trim().to_string()
}

fn get_os_info() -> (String, String) {
    let name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    
    // Clean up macOS name
    let name = if name == "Darwin" {
        "macOS".to_string()
    } else {
        name
    };

    (name, version)
}

#[cfg(target_os = "macos")]
fn get_sysctl_int(key: &str) -> Option<u32> {
    let output = Command::new("sysctl").arg("-n").arg(key).output().ok()?;
    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().parse().ok()
    } else {
        None
    }
}

