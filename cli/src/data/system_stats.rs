use std::time::Duration;

use color_eyre::eyre::Result;
use systemstat::{Platform, System};

const CPU_SAMPLE_DELAY_MS: u64 = 100;

pub struct SystemStatsData {
    system: System,
    cpu_load_percent: Option<f32>,
    memory_used_percent: f32,
    disk_used_percent: f32,
    uptime: Duration,
    warmed_up: bool,
}

impl SystemStatsData {
    pub fn new() -> Result<Self> {
        let system = System::new();

        let mut stats = Self {
            system,
            cpu_load_percent: None,
            memory_used_percent: 0.0,
            disk_used_percent: 0.0,
            uptime: Duration::ZERO,
            warmed_up: false,
        };

        stats.refresh_memory();
        stats.refresh_disk();
        stats.refresh_uptime();

        Ok(stats)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_memory();
        self.refresh_disk();
        self.refresh_uptime();
        self.refresh_cpu();
        self.warmed_up = true;
        Ok(())
    }

    fn refresh_memory(&mut self) {
        if let Ok(mem) = self.system.memory() {
            let total = mem.total.as_u64();
            let free = mem.free.as_u64();
            if total > 0 {
                let used = total.saturating_sub(free);
                self.memory_used_percent = (used as f64 / total as f64 * 100.0) as f32;
            }
        }
    }

    fn refresh_disk(&mut self) {
        if let Ok(mount) = self.system.mount_at("/") {
            let total = mount.total.as_u64();
            let avail = mount.avail.as_u64();
            if total > 0 {
                let used = total.saturating_sub(avail);
                self.disk_used_percent = (used as f64 / total as f64 * 100.0) as f32;
            }
        }
    }

    fn refresh_uptime(&mut self) {
        if let Ok(uptime) = self.system.uptime() {
            self.uptime = uptime;
        }
    }

    fn refresh_cpu(&mut self) {
        if let Ok(cpu) = self.system.cpu_load_aggregate() {
            std::thread::sleep(Duration::from_millis(CPU_SAMPLE_DELAY_MS));
            if let Ok(cpu_load) = cpu.done() {
                let used = 1.0 - cpu_load.idle;
                self.cpu_load_percent = Some(used * 100.0);
            }
        }
    }

    pub fn cpu_load_percent(&self) -> Option<f32> {
        self.cpu_load_percent
    }

    pub fn memory_used_percent(&self) -> f32 {
        self.memory_used_percent
    }

    pub fn disk_used_percent(&self) -> f32 {
        self.disk_used_percent
    }

    pub fn uptime_formatted(&self) -> String {
        let total_secs = self.uptime.as_secs();
        let days = total_secs / 86400;
        let hours = (total_secs % 86400) / 3600;
        let minutes = (total_secs % 3600) / 60;

        if days > 0 {
            format!("{}d {}h", days, hours)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    pub fn is_warmed_up(&self) -> bool {
        self.warmed_up
    }
}
