use std::process::Command;

use crate::daemon::SystemSnapshot;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub chip: String,
    pub os_version: String,
    pub p_cores: u32,
    pub e_cores: u32,
}

impl SystemInfo {
    pub fn new() -> Self {
        let (chip, p_cores, e_cores) = get_chip_info();
        let os_version = get_os_version();

        Self {
            chip,
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
        self.os_version = snapshot.os_version.clone();
        self.p_cores = snapshot.p_cores;
        self.e_cores = snapshot.e_cores;
    }
}

fn get_chip_info() -> (String, u32, u32) {
    let chip = get_sysctl_string("machdep.cpu.brand_string")
        .map(|s| clean_chip_name(&s))
        .unwrap_or_else(|| "Unknown".to_string());

    let p_cores = get_sysctl_int("hw.perflevel0.physicalcpu").unwrap_or(0);
    let e_cores = get_sysctl_int("hw.perflevel1.physicalcpu").unwrap_or(0);

    (chip, p_cores, e_cores)
}

fn clean_chip_name(name: &str) -> String {
    name.replace("Apple ", "").trim().to_string()
}

fn get_os_version() -> String {
    if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    "Unknown".to_string()
}

fn get_sysctl_string(key: &str) -> Option<String> {
    let output = Command::new("sysctl").arg("-n").arg(key).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn get_sysctl_int(key: &str) -> Option<u32> {
    get_sysctl_string(key)?.parse().ok()
}
