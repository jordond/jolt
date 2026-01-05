use color_eyre::eyre::Result;
use std::process::Command;

#[allow(dead_code)]
mod ioreport_ffi {
    use core_foundation::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};
    use std::ffi::c_void;

    pub type IOReportSubscriptionRef = *const c_void;
    pub type CFTypeRef = *const c_void;

    #[link(name = "IOReport", kind = "dylib")]
    extern "C" {
        pub fn IOReportCopyChannelsInGroup(
            group: core_foundation_sys::string::CFStringRef,
            subgroup: core_foundation_sys::string::CFStringRef,
            a: u64,
            b: u64,
            c: u64,
        ) -> CFDictionaryRef;

        pub fn IOReportMergeChannels(a: CFDictionaryRef, b: CFDictionaryRef, nil: CFTypeRef);

        pub fn IOReportCreateSubscription(
            a: CFTypeRef,
            b: CFMutableDictionaryRef,
            c: *mut CFMutableDictionaryRef,
            d: u64,
            e: CFTypeRef,
        ) -> IOReportSubscriptionRef;

        pub fn IOReportCreateSamples(
            subscription: IOReportSubscriptionRef,
            channels: CFMutableDictionaryRef,
            nil: CFTypeRef,
        ) -> CFDictionaryRef;

        pub fn IOReportCreateSamplesDelta(
            prev: CFDictionaryRef,
            current: CFDictionaryRef,
            nil: CFTypeRef,
        ) -> CFDictionaryRef;

        pub fn IOReportSimpleGetIntegerValue(sample: CFDictionaryRef, index: i32) -> i64;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    LowPower,
    Automatic,
    HighPerformance,
    Unknown,
}

pub struct PowerData {
    cpu_power: f32,
    gpu_power: f32,
    total_system_power: f32,
    power_mode: PowerMode,
    cpu_usage_percent: f32,
    gpu_usage_percent: f32,
}

impl PowerData {
    pub fn new() -> Result<Self> {
        let mut data = Self {
            cpu_power: 0.0,
            gpu_power: 0.0,
            total_system_power: 0.0,
            power_mode: PowerMode::Unknown,
            cpu_usage_percent: 0.0,
            gpu_usage_percent: 0.0,
        };

        data.refresh()?;
        Ok(data)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_power_metrics();
        self.refresh_power_mode();
        Ok(())
    }

    fn refresh_power_metrics(&mut self) {
        if let Ok(output) = Command::new("powermetrics")
            .args([
                "-n",
                "1",
                "-i",
                "100",
                "--samplers",
                "cpu_power,gpu_power",
                "-f",
                "plist",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_powermetrics_output(&stdout);
        } else {
            self.estimate_power_from_cpu_usage();
        }
    }

    fn parse_powermetrics_output(&mut self, _output: &str) {
        self.estimate_power_from_cpu_usage();
    }

    fn estimate_power_from_cpu_usage(&mut self) {
        use sysinfo::System;

        let mut sys = System::new_all();
        sys.refresh_all();
        std::thread::sleep(std::time::Duration::from_millis(100));
        sys.refresh_all();

        let cpu_usage: f32 =
            sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;

        self.cpu_usage_percent = cpu_usage;

        let base_power = 3.0;
        let max_cpu_power = 20.0;
        self.cpu_power = base_power + (cpu_usage / 100.0) * max_cpu_power;

        let gpu_base = 1.0;
        let max_gpu = 15.0;
        self.gpu_power = gpu_base + (self.gpu_usage_percent / 100.0) * max_gpu;

        self.total_system_power = self.cpu_power + self.gpu_power + 2.0;
    }

    fn refresh_power_mode(&mut self) {
        if let Ok(output) = Command::new("pmset").args(["-g"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            if stdout.contains("lowpowermode 1") {
                self.power_mode = PowerMode::LowPower;
            } else if stdout.contains("highpowermode 1") {
                self.power_mode = PowerMode::HighPerformance;
            } else {
                self.power_mode = PowerMode::Automatic;
            }
        }
    }

    pub fn cpu_power_watts(&self) -> f32 {
        self.cpu_power
    }

    pub fn gpu_power_watts(&self) -> f32 {
        self.gpu_power
    }

    pub fn total_power_watts(&self) -> f32 {
        self.total_system_power
    }

    pub fn power_mode(&self) -> PowerMode {
        self.power_mode
    }

    pub fn power_mode_label(&self) -> &'static str {
        match self.power_mode {
            PowerMode::LowPower => "Low Power",
            PowerMode::Automatic => "Automatic",
            PowerMode::HighPerformance => "High Performance",
            PowerMode::Unknown => "Unknown",
        }
    }

    #[allow(dead_code)]
    pub fn cpu_usage(&self) -> f32 {
        self.cpu_usage_percent
    }

    #[allow(dead_code)]
    pub fn gpu_usage(&self) -> f32 {
        self.gpu_usage_percent
    }
}
