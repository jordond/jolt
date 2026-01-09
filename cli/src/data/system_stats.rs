use std::time::Duration;

use color_eyre::eyre::Result;
use systemstat::{Platform, System};

const BYTES_PER_GB: f64 = 1_073_741_824.0;

pub struct SystemStatsData {
    system: System,
    load_one: f32,
    memory_used_bytes: u64,
    memory_total_bytes: u64,
    uptime: Duration,
    warmed_up: bool,
}

impl SystemStatsData {
    pub fn new() -> Result<Self> {
        let system = System::new();

        let mut stats = Self {
            system,
            load_one: 0.0,
            memory_used_bytes: 0,
            memory_total_bytes: 0,
            uptime: Duration::ZERO,
            warmed_up: false,
        };

        stats.refresh_load_average();
        stats.refresh_memory();
        stats.refresh_uptime();

        Ok(stats)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_load_average();
        self.refresh_memory();
        self.refresh_uptime();
        self.warmed_up = true;
        Ok(())
    }

    fn refresh_load_average(&mut self) {
        if let Ok(load) = self.system.load_average() {
            self.load_one = load.one;
        }
    }

    fn refresh_memory(&mut self) {
        if let Ok(mem) = self.system.memory() {
            self.memory_total_bytes = mem.total.as_u64();
            let free = mem.free.as_u64();
            self.memory_used_bytes = self.memory_total_bytes.saturating_sub(free);
        }
    }

    fn refresh_uptime(&mut self) {
        if let Ok(uptime) = self.system.uptime() {
            self.uptime = uptime;
        }
    }

    pub fn load_one(&self) -> f32 {
        self.load_one
    }

    pub fn memory_used_gb(&self) -> f64 {
        self.memory_used_bytes as f64 / BYTES_PER_GB
    }

    pub fn memory_total_gb(&self) -> f64 {
        self.memory_total_bytes as f64 / BYTES_PER_GB
    }

    pub fn memory_formatted(&self) -> String {
        format!(
            "{:.1}/{:.0} GB",
            self.memory_used_gb(),
            self.memory_total_gb()
        )
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
