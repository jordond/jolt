//! Battery monitoring traits and types.

use std::time::Duration;

use color_eyre::eyre::Result;

use crate::types::{BatteryTechnology, ChargeState};

/// Battery information snapshot.
///
/// All values represent the current state at the time of the last refresh.
#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
    /// Current charge level as a percentage (0-100).
    pub charge_percent: f32,

    /// Current charging state.
    pub state: ChargeState,

    /// Maximum capacity in watt-hours (current full charge capacity).
    pub max_capacity_wh: f32,

    /// Design capacity in watt-hours (original factory capacity).
    pub design_capacity_wh: f32,

    /// Current voltage in millivolts.
    pub voltage_mv: u32,

    /// Current amperage in milliamps. Negative when discharging.
    /// May be 0 on platforms that don't report this.
    pub amperage_ma: i32,

    /// Battery health as a percentage (0-100).
    /// Calculated as max_capacity / design_capacity * 100.
    pub health_percent: f32,

    /// Number of charge cycles, if available.
    pub cycle_count: Option<u32>,

    /// Estimated time until fully charged, if charging.
    pub time_to_full: Option<Duration>,

    /// Estimated time until empty, if discharging.
    pub time_to_empty: Option<Duration>,

    /// Battery temperature in Celsius, if available.
    pub temperature_c: Option<f32>,

    /// Whether external power is connected.
    pub external_connected: bool,

    /// Battery vendor/manufacturer name (e.g., "Apple", "Samsung SDI").
    pub vendor: Option<String>,

    /// Battery model identifier (e.g., "bq20z451").
    pub model: Option<String>,

    /// Battery serial number.
    pub serial_number: Option<String>,

    /// Battery technology/chemistry type.
    pub technology: BatteryTechnology,

    /// Current energy remaining in watt-hours.
    pub energy_wh: f32,

    /// Instantaneous power rate in watts (positive = charging, negative = discharging).
    pub energy_rate_watts: f32,

    // === macOS-specific fields (None on other platforms) ===
    /// Charger wattage rating (e.g., 96W), macOS only.
    pub charger_watts: Option<u32>,

    /// Minimum state of charge today (0-100), macOS only.
    pub daily_min_soc: Option<f32>,

    /// Maximum state of charge today (0-100), macOS only.
    pub daily_max_soc: Option<f32>,
}

impl BatteryInfo {
    /// Calculate the current charging power in watts.
    ///
    /// Returns Some if charging and amperage is available.
    pub fn charging_watts(&self) -> Option<f32> {
        if self.state == ChargeState::Charging && self.amperage_ma > 0 {
            let watts = (self.amperage_ma as f32 / 1000.0) * (self.voltage_mv as f32 / 1000.0);
            Some(watts)
        } else {
            None
        }
    }

    /// Calculate the current discharge power in watts.
    ///
    /// Returns Some if discharging and amperage is available.
    pub fn discharge_watts(&self) -> Option<f32> {
        if self.state == ChargeState::Discharging && self.amperage_ma < 0 {
            let watts =
                (self.amperage_ma.abs() as f32 / 1000.0) * (self.voltage_mv as f32 / 1000.0);
            Some(watts)
        } else {
            None
        }
    }

    /// Get the time remaining (to full or empty depending on state).
    pub fn time_remaining(&self) -> Option<Duration> {
        match self.state {
            ChargeState::Charging => self.time_to_full,
            ChargeState::Discharging => self.time_to_empty,
            _ => None,
        }
    }

    /// Format time remaining as a human-readable string.
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
}

/// Trait for platform-specific battery providers.
pub trait BatteryProvider {
    /// Create a new battery provider instance.
    fn new() -> Result<Self>
    where
        Self: Sized;

    /// Refresh battery information from the system.
    fn refresh(&mut self) -> Result<()>;

    /// Get the current battery information.
    fn info(&self) -> &BatteryInfo;

    /// Check if battery monitoring is supported on this system.
    fn is_supported() -> bool
    where
        Self: Sized,
    {
        true
    }

    /// Check if a battery is available on this system.
    fn is_available() -> bool
    where
        Self: Sized,
    {
        use starship_battery::Manager;
        Manager::new()
            .ok()
            .and_then(|m| m.batteries().ok())
            .and_then(|mut b| b.next())
            .and_then(|b| b.ok())
            .is_some()
    }
}
