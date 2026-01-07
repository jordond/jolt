use color_eyre::eyre::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use sysinfo::{ProcessStatus, ProcessesToUpdate, System};

use crate::config::cache_dir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessState {
    Running,
    Sleeping,
    Idle,
    Stopped,
    Zombie,
    #[default]
    Unknown,
}

impl From<ProcessStatus> for ProcessState {
    fn from(status: ProcessStatus) -> Self {
        match status {
            ProcessStatus::Run => ProcessState::Running,
            ProcessStatus::Sleep => ProcessState::Sleeping,
            ProcessStatus::Idle => ProcessState::Idle,
            ProcessStatus::Stop => ProcessState::Stopped,
            ProcessStatus::Zombie => ProcessState::Zombie,
            _ => ProcessState::Unknown,
        }
    }
}

impl ProcessState {
    pub fn as_char(&self) -> char {
        match self {
            ProcessState::Running => 'R',
            ProcessState::Sleeping => 'S',
            ProcessState::Idle => 'I',
            ProcessState::Stopped => 'T',
            ProcessState::Zombie => 'Z',
            ProcessState::Unknown => '?',
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub energy_impact: f32,
    pub parent_pid: Option<u32>,
    pub children: Option<Vec<ProcessInfo>>,
    pub is_killable: bool,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub status: ProcessState,
    pub run_time_secs: u64,
    pub total_cpu_time_secs: u64,
}

pub struct ProcessData {
    system: System,
    pub processes: Vec<ProcessInfo>,
    display_name_cache: HashMap<String, String>,
    excluded_processes: Vec<String>,
}

impl ProcessData {
    pub fn new() -> Result<Self> {
        Self::with_exclusions(Vec::new())
    }

    pub fn with_exclusions(excluded: Vec<String>) -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        let display_name_cache = load_display_name_cache();

        let mut data = Self {
            system,
            processes: Vec::new(),
            display_name_cache,
            excluded_processes: excluded,
        };

        data.refresh()?;
        Ok(data)
    }

    fn is_excluded(&self, name: &str, pid: u32) -> bool {
        if pid == 1 {
            return true;
        }

        let name_lower = name.to_lowercase();
        if name_lower == "launchd" {
            return true;
        }

        for excluded in &self.excluded_processes {
            if name_lower == excluded.to_lowercase() {
                return true;
            }
        }

        false
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        let mut process_map: HashMap<u32, ProcessInfo> = HashMap::new();
        let mut children_map: HashMap<u32, Vec<ProcessInfo>> = HashMap::new();

        for (pid, process) in self.system.processes() {
            let pid_u32 = pid.as_u32();
            let binary_name = process.name().to_string_lossy().to_string();

            if self.is_excluded(&binary_name, pid_u32) {
                continue;
            }

            let parent_pid = process.parent().map(|p| p.as_u32());

            let cpu = process.cpu_usage();
            let memory_mb = process.memory() as f64 / (1024.0 * 1024.0);

            let energy_impact = calculate_energy_impact(cpu, memory_mb as f32);
            let exe_path = process.exe().map(|p| p.to_path_buf());

            let (display_name, cache_updated) = if let Some(ref path) = exe_path {
                let path_str = path.to_string_lossy().to_string();
                if let Some(cached) = self.display_name_cache.get(&path_str) {
                    (cached.clone(), false)
                } else {
                    let name = get_app_display_name(path).unwrap_or_else(|| binary_name.clone());
                    self.display_name_cache.insert(path_str, name.clone());
                    (name, true)
                }
            } else {
                (binary_name.clone(), false)
            };

            if cache_updated {
                save_display_name_cache(&self.display_name_cache);
            }

            let is_killable = is_process_killable(pid_u32, &binary_name);

            let disk_usage = process.disk_usage();
            let status = ProcessState::from(process.status());
            let run_time_secs = process.run_time();
            let total_cpu_time_secs = process.accumulated_cpu_time();

            let info = ProcessInfo {
                pid: pid_u32,
                name: display_name,
                command: binary_name.clone(),
                cpu_usage: cpu,
                memory_mb,
                energy_impact,
                parent_pid,
                children: None,
                is_killable,
                disk_read_bytes: disk_usage.read_bytes,
                disk_write_bytes: disk_usage.written_bytes,
                status,
                run_time_secs,
                total_cpu_time_secs,
            };

            process_map.insert(pid_u32, info.clone());

            if let Some(parent) = parent_pid {
                children_map.entry(parent).or_default().push(info);
            }
        }

        let mut top_processes: Vec<ProcessInfo> = Vec::new();

        for (pid, mut process) in process_map {
            if let Some(children) = children_map.remove(&pid) {
                let total_energy: f32 = children.iter().map(|c| c.energy_impact).sum();
                process.energy_impact += total_energy * 0.3;

                let mut sorted_children = children;
                sorted_children.sort_by(|a, b| {
                    b.energy_impact
                        .partial_cmp(&a.energy_impact)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                if !sorted_children.is_empty() {
                    process.children = Some(sorted_children);
                }
            }

            if process.energy_impact >= 0.5 {
                top_processes.push(process);
            }
        }

        top_processes.sort_by(|a, b| {
            b.energy_impact
                .partial_cmp(&a.energy_impact)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.processes = top_processes.into_iter().take(50).collect();

        Ok(())
    }

    pub fn kill_process(&self, pid: u32) -> Result<()> {
        use std::process::Command;

        Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()?;

        Ok(())
    }
}

fn calculate_energy_impact(cpu_usage: f32, memory_mb: f32) -> f32 {
    let cpu_factor = cpu_usage * 0.8;
    let memory_factor = (memory_mb / 100.0).min(20.0) * 0.2;

    cpu_factor + memory_factor
}

const SYSTEM_PROCESSES: &[&str] = &[
    "kernel_task",
    "launchd",
    "WindowServer",
    "loginwindow",
    "SystemUIServer",
    "Dock",
    "Finder",
    "coreaudiod",
    "configd",
    "mds",
    "mds_stores",
    "mdworker",
    "diskarbitrationd",
    "notifyd",
    "powerd",
    "opendirectoryd",
    "securityd",
    "trustd",
    "cfprefsd",
    "coreservicesd",
    "distnoted",
    "usbd",
    "hidd",
    "bluetoothd",
    "airportd",
    "watchdogd",
    "fseventsd",
    "kextd",
    "UserEventAgent",
    "sandboxd",
    "syslogd",
    "logd",
    "spindump",
    "init",
];

fn is_process_killable(pid: u32, name: &str) -> bool {
    if pid == 0 || pid == 1 {
        return false;
    }

    let name_lower = name.to_lowercase();
    for sys_proc in SYSTEM_PROCESSES {
        if name_lower == sys_proc.to_lowercase() {
            return false;
        }
    }

    true
}

fn get_app_display_name(exe_path: &Path) -> Option<String> {
    use std::process::Command;

    let path_str = exe_path.to_string_lossy();
    let binary_name = exe_path.file_name()?.to_string_lossy().to_string();

    let app_idx = path_str.find(".app/")?;
    let app_bundle_path = &path_str[..app_idx + 4];

    let info_plist_path = format!("{}/Contents/Info.plist", app_bundle_path);

    if !Path::new(&info_plist_path).exists() {
        return None;
    }

    let mut app_name: Option<String> = None;
    for key in ["CFBundleDisplayName", "CFBundleName"] {
        if let Ok(output) = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", &format!("Print :{}", key), &info_plist_path])
            .output()
        {
            if output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() {
                    app_name = Some(name);
                    break;
                }
            }
        }
    }

    let base_name = app_name?;

    let suffix = get_process_suffix(&binary_name, &path_str);
    if let Some(s) = suffix {
        Some(format!("{} {}", base_name, s))
    } else {
        Some(base_name)
    }
}

fn get_process_suffix(binary_name: &str, path: &str) -> Option<&'static str> {
    let binary_lower = binary_name.to_lowercase();
    let path_lower = path.to_lowercase();

    if binary_lower.contains("helper") {
        if path_lower.contains("renderer") || binary_lower.contains("renderer") {
            return Some("Helper (Renderer)");
        } else if path_lower.contains("gpu") || binary_lower.contains("gpu") {
            return Some("Helper (GPU)");
        } else if path_lower.contains("plugin") || binary_lower.contains("plugin") {
            return Some("Helper (Plugin)");
        } else if path_lower.contains("network") || binary_lower.contains("network") {
            return Some("Helper (Network)");
        } else {
            return Some("Helper");
        }
    }

    if binary_lower.contains("renderer") {
        return Some("Renderer");
    }

    if binary_lower.contains("web content") || path_lower.contains("webcontent") {
        return Some("Web Content");
    }

    if binary_lower.contains("gpu") && !binary_lower.contains("helper") {
        return Some("GPU Process");
    }

    if binary_lower.contains("extension") {
        return Some("Extension");
    }

    None
}

fn display_name_cache_path() -> std::path::PathBuf {
    cache_dir().join("display_names.json")
}

fn load_display_name_cache() -> HashMap<String, String> {
    let path = display_name_cache_path();
    if !path.exists() {
        return HashMap::new();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn save_display_name_cache(cache: &HashMap<String, String>) {
    let path = display_name_cache_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(content) = serde_json::to_string(cache) {
        let _ = fs::write(path, content);
    }
}
