use std::time::Duration;

use color_eyre::eyre::Result;
use jolt_platform::BatteryProvider;

use crate::daemon::{BatterySnapshot, BatteryState as ProtocolBatteryState};

pub use jolt_platform::{BatteryTechnology, ChargeState};

#[cfg(target_os = "macos")]
type PlatformBattery = jolt_platform::macos::MacOSBattery;

#[cfg(target_os = "linux")]
type PlatformBattery = jolt_platform::linux::LinuxBattery;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("BatteryData (PlatformBattery) is only supported on macOS and Linux targets.");

pub struct BatteryData {
    provider: PlatformBattery,
    time_to_full: Option<Duration>,
    time_to_empty: Option<Duration>,
}

impl BatteryData {
    pub fn new() -> Result<Self> {
        let provider = PlatformBattery::new()?;
        let info = provider.info();
        Ok(Self {
            time_to_full: info.time_to_full,
            time_to_empty: info.time_to_empty,
            provider,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.provider.refresh()?;
        let info = self.provider.info();
        self.time_to_full = info.time_to_full;
        self.time_to_empty = info.time_to_empty;
        Ok(())
    }

    pub fn charge_percent(&self) -> f32 {
        self.provider.info().charge_percent
    }

    pub fn max_capacity_wh(&self) -> f32 {
        self.provider.info().max_capacity_wh
    }

    pub fn design_capacity_wh(&self) -> f32 {
        self.provider.info().design_capacity_wh
    }

    pub fn state(&self) -> ChargeState {
        self.provider.info().state
    }

    pub fn state_label(&self) -> &'static str {
        self.provider.info().state.label()
    }

    pub fn time_remaining(&self) -> Option<Duration> {
        let info = self.provider.info();
        let system_estimate = match info.state {
            ChargeState::Charging => self.time_to_full,
            ChargeState::Discharging => self.time_to_empty,
            _ => None,
        };

        if system_estimate.is_some() {
            return system_estimate;
        }

        if info.state == ChargeState::Discharging {
            if let Some(watts) = self.discharge_watts() {
                if watts > 0.1 {
                    let current_wh = info.max_capacity_wh * (info.charge_percent / 100.0);
                    let hours_remaining = current_wh / watts;
                    let secs = (hours_remaining * 3600.0) as u64;
                    if secs > 0 && secs < 86400 {
                        return Some(Duration::from_secs(secs));
                    }
                }
            }
        }

        None
    }

    pub fn time_remaining_formatted(&self) -> Option<String> {
        self.time_remaining().and_then(|d| {
            let total_mins = d.as_secs() / 60;
            if total_mins == 0 {
                return None;
            }
            let hours = total_mins / 60;
            let mins = total_mins % 60;

            if hours > 0 {
                Some(format!("{}h {}m", hours, mins))
            } else {
                Some(format!("{}m", mins))
            }
        })
    }

    pub fn time_remaining_minutes(&self) -> Option<u64> {
        self.time_remaining().map(|d| d.as_secs() / 60)
    }

    pub fn cycle_count(&self) -> Option<u32> {
        self.provider.info().cycle_count
    }

    pub fn health_percent(&self) -> f32 {
        self.provider.info().health_percent
    }

    pub fn is_charging(&self) -> bool {
        self.provider.info().state.is_charging()
    }

    pub fn charging_watts(&self) -> Option<f32> {
        self.provider.info().charging_watts()
    }

    pub fn charger_watts(&self) -> Option<u32> {
        self.provider.info().charger_watts
    }

    pub fn voltage_mv(&self) -> u32 {
        self.provider.info().voltage_mv
    }

    pub fn amperage_ma(&self) -> i32 {
        self.provider.info().amperage_ma
    }

    pub fn external_connected(&self) -> bool {
        self.provider.info().external_connected
    }

    pub fn temperature_c(&self) -> Option<f32> {
        self.provider.info().temperature_c
    }

    pub fn daily_min_soc(&self) -> Option<f32> {
        self.provider.info().daily_min_soc
    }

    pub fn daily_max_soc(&self) -> Option<f32> {
        self.provider.info().daily_max_soc
    }

    pub fn discharge_watts(&self) -> Option<f32> {
        self.provider.info().discharge_watts()
    }

    pub fn vendor(&self) -> Option<&str> {
        self.provider.info().vendor.as_deref()
    }

    pub fn model(&self) -> Option<&str> {
        self.provider.info().model.as_deref()
    }

    pub fn serial_number(&self) -> Option<&str> {
        self.provider.info().serial_number.as_deref()
    }

    pub fn technology(&self) -> BatteryTechnology {
        self.provider.info().technology
    }

    pub fn energy_wh(&self) -> f32 {
        self.provider.info().energy_wh
    }

    pub fn energy_rate_watts(&self) -> f32 {
        self.provider.info().energy_rate_watts
    }

    pub fn update_from_snapshot(&mut self, snapshot: &BatterySnapshot) {
        self.time_to_full = if matches!(snapshot.state, ProtocolBatteryState::Charging) {
            snapshot
                .time_remaining_mins
                .map(|m| Duration::from_secs(m * 60))
        } else {
            None
        };

        self.time_to_empty = if matches!(snapshot.state, ProtocolBatteryState::Discharging) {
            snapshot
                .time_remaining_mins
                .map(|m| Duration::from_secs(m * 60))
        } else {
            None
        };
    }
}
