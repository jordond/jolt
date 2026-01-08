use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use color_eyre::eyre::Result;

use crate::power::{PowerInfo, PowerProvider};
use crate::types::PowerMode;

const RAPL_PATH: &str = "/sys/class/powercap/intel-rapl";
const HWMON_PATH: &str = "/sys/class/hwmon";
const SMOOTHING_SAMPLE_COUNT: usize = 5;
const MIN_WARMUP_SAMPLES: usize = 3;

#[derive(Debug, Clone, Copy)]
struct PowerSample {
    cpu_power: f32,
    gpu_power: f32,
    system_power: f32,
}

#[derive(Debug)]
struct RaplDomain {
    path: PathBuf,
    name: String,
    last_energy_uj: u64,
    last_time: Instant,
}

pub struct LinuxPower {
    info: PowerInfo,
    rapl_domains: Vec<RaplDomain>,
    gpu_hwmon_path: Option<PathBuf>,
    samples: VecDeque<PowerSample>,
    cpu_power: f32,
    gpu_power: f32,
    system_power: f32,
}

impl PowerProvider for LinuxPower {
    fn new() -> Result<Self> {
        let rapl_domains = discover_rapl_domains();
        let gpu_hwmon_path = discover_gpu_hwmon();

        let mut provider = Self {
            info: PowerInfo::default(),
            rapl_domains,
            gpu_hwmon_path,
            samples: VecDeque::with_capacity(SMOOTHING_SAMPLE_COUNT),
            cpu_power: 0.0,
            gpu_power: 0.0,
            system_power: 0.0,
        };

        std::thread::sleep(Duration::from_millis(100));
        provider.refresh()?;

        Ok(provider)
    }

    fn refresh(&mut self) -> Result<()> {
        self.refresh_rapl_power();
        self.refresh_gpu_power();
        self.system_power = self.cpu_power + self.gpu_power;
        self.record_sample();
        self.update_info();
        Ok(())
    }

    fn info(&self) -> &PowerInfo {
        &self.info
    }

    fn is_supported() -> bool {
        Path::new(RAPL_PATH).exists()
    }
}

impl LinuxPower {
    fn update_info(&mut self) {
        self.info.cpu_power_watts = self.smoothed_value(|s| s.cpu_power);
        self.info.gpu_power_watts = self.smoothed_value(|s| s.gpu_power);
        self.info.system_power_watts = self.smoothed_value(|s| s.system_power);
        self.info.is_warmed_up = self.samples.len() >= MIN_WARMUP_SAMPLES;
        self.info.power_mode = PowerMode::Unknown;
    }

    fn record_sample(&mut self) {
        let sample = PowerSample {
            cpu_power: self.cpu_power,
            gpu_power: self.gpu_power,
            system_power: self.system_power,
        };

        if self.samples.len() >= SMOOTHING_SAMPLE_COUNT {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    fn smoothed_value<F>(&self, extractor: F) -> f32
    where
        F: Fn(&PowerSample) -> f32,
    {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.samples.iter().map(extractor).sum();
        sum / self.samples.len() as f32
    }

    fn refresh_rapl_power(&mut self) {
        let mut total_cpu_power = 0.0f32;
        let now = Instant::now();

        for domain in &mut self.rapl_domains {
            let energy_path = domain.path.join("energy_uj");
            if let Ok(content) = fs::read_to_string(&energy_path) {
                if let Ok(energy_uj) = content.trim().parse::<u64>() {
                    let elapsed = now.duration_since(domain.last_time);
                    let elapsed_us = elapsed.as_micros() as u64;

                    if elapsed_us > 0 && domain.last_energy_uj > 0 {
                        let energy_delta = if energy_uj >= domain.last_energy_uj {
                            energy_uj - domain.last_energy_uj
                        } else {
                            energy_uj
                        };

                        let watts = energy_delta as f32 / elapsed_us as f32;
                        total_cpu_power += watts;
                    }

                    domain.last_energy_uj = energy_uj;
                    domain.last_time = now;
                }
            }
        }

        self.cpu_power = total_cpu_power;
    }

    fn refresh_gpu_power(&mut self) {
        if let Some(ref path) = self.gpu_hwmon_path {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(microwatts) = content.trim().parse::<u64>() {
                    self.gpu_power = microwatts as f32 / 1_000_000.0;
                    return;
                }
            }
        }
        self.gpu_power = 0.0;
    }
}

fn discover_rapl_domains() -> Vec<RaplDomain> {
    let mut domains = Vec::new();
    let rapl_path = Path::new(RAPL_PATH);

    if !rapl_path.exists() {
        return domains;
    }

    if let Ok(entries) = fs::read_dir(rapl_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name_path = path.join("name");
            let energy_path = path.join("energy_uj");

            if energy_path.exists() {
                let name = fs::read_to_string(&name_path)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());

                if name.contains("package") || name.contains("psys") {
                    let last_energy_uj = fs::read_to_string(&energy_path)
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(0);

                    domains.push(RaplDomain {
                        path,
                        name,
                        last_energy_uj,
                        last_time: Instant::now(),
                    });
                }
            }
        }
    }

    domains
}

fn discover_gpu_hwmon() -> Option<PathBuf> {
    let hwmon_path = Path::new(HWMON_PATH);
    if !hwmon_path.exists() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            let name_path = path.join("name");
            if let Ok(name) = fs::read_to_string(&name_path) {
                let name = name.trim().to_lowercase();
                if name.contains("amdgpu") || name.contains("i915") || name.contains("nouveau") {
                    let power_path = path.join("power1_input");
                    if power_path.exists() {
                        return Some(power_path);
                    }
                }
            }
        }
    }

    None
}
