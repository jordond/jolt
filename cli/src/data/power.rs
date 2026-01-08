use std::collections::VecDeque;

use color_eyre::eyre::Result;
use jolt_platform::PowerProvider;

use crate::daemon::PowerSnapshot;

pub use jolt_platform::PowerMode;

#[cfg(target_os = "macos")]
type PlatformPower = jolt_platform::macos::MacOSPower;

#[cfg(target_os = "linux")]
type PlatformPower = jolt_platform::linux::LinuxPower;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("PlatformPower is only defined for macOS and Linux targets.");

const SMOOTHING_SAMPLE_COUNT: usize = 5;
const MIN_WARMUP_SAMPLES: usize = 3;

#[derive(Debug, Clone, Copy)]
struct PowerSample {
    cpu_power: f32,
    gpu_power: f32,
    system_power: f32,
}

pub struct PowerData {
    provider: PlatformPower,
    samples: VecDeque<PowerSample>,
    cached_snapshot: Option<PowerSnapshot>,
}

impl PowerData {
    pub fn new() -> Result<Self> {
        let provider = PlatformPower::new()?;
        let info = provider.info();
        let mut samples = VecDeque::with_capacity(SMOOTHING_SAMPLE_COUNT);

        let sample = PowerSample {
            cpu_power: info.cpu_power_watts,
            gpu_power: info.gpu_power_watts,
            system_power: info.system_power_watts,
        };
        samples.push_back(sample);

        Ok(Self {
            provider,
            samples,
            cached_snapshot: None,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.provider.refresh()?;
        self.record_sample();
        self.cached_snapshot = None;
        Ok(())
    }

    fn record_sample(&mut self) {
        let info = self.provider.info();
        let sample = PowerSample {
            cpu_power: info.cpu_power_watts,
            gpu_power: info.gpu_power_watts,
            system_power: info.system_power_watts,
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

    pub fn cpu_power_watts(&self) -> f32 {
        self.smoothed_value(|s| s.cpu_power)
    }

    pub fn gpu_power_watts(&self) -> f32 {
        self.smoothed_value(|s| s.gpu_power)
    }

    pub fn total_power_watts(&self) -> f32 {
        self.smoothed_value(|s| s.system_power)
    }

    pub fn is_warmed_up(&self) -> bool {
        self.samples.len() >= MIN_WARMUP_SAMPLES
    }

    pub fn power_mode(&self) -> PowerMode {
        if let Some(ref snapshot) = self.cached_snapshot {
            return match snapshot.power_mode {
                crate::daemon::PowerMode::LowPower => PowerMode::LowPower,
                crate::daemon::PowerMode::Automatic => PowerMode::Automatic,
                crate::daemon::PowerMode::HighPerformance => PowerMode::HighPerformance,
                crate::daemon::PowerMode::Unknown => PowerMode::Unknown,
            };
        }
        self.provider.info().power_mode
    }

    pub fn power_mode_label(&self) -> &'static str {
        self.power_mode().label()
    }

    pub fn update_from_snapshot(&mut self, snapshot: &PowerSnapshot) {
        let sample = PowerSample {
            cpu_power: snapshot.cpu_power_watts,
            gpu_power: snapshot.gpu_power_watts,
            system_power: snapshot.total_power_watts,
        };

        if self.samples.is_empty() {
            for _ in 0..MIN_WARMUP_SAMPLES {
                self.samples.push_back(sample);
            }
        } else {
            if self.samples.len() >= SMOOTHING_SAMPLE_COUNT {
                self.samples.pop_front();
            }
            self.samples.push_back(sample);
        }

        self.cached_snapshot = Some(snapshot.clone());
    }
}
