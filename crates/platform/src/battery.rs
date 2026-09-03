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
    ///
    /// A power source only counts as a battery when its readings are usable.
    /// Machines with no real battery (desktop Macs, pseudo power sources) can
    /// still expose a power source whose maximum capacity is zero, which makes
    /// the `energy / energy_full` state-of-charge division produce NaN. Those
    /// are reported as "no battery" so callers take their existing no-battery
    /// path instead of rendering a degraded UI.
    fn is_available() -> bool
    where
        Self: Sized,
    {
        use starship_battery::units::energy::watt_hour;
        use starship_battery::units::ratio::percent;
        use starship_battery::Manager;

        Manager::new()
            .ok()
            .and_then(|m| m.batteries().ok())
            .and_then(|mut b| b.next())
            .and_then(|b| b.ok())
            .is_some_and(|battery| {
                has_usable_readings(
                    battery.state_of_charge().get::<percent>(),
                    battery.energy_full().get::<watt_hour>(),
                )
            })
    }
}

/// Whether a power source's readings describe a battery jolt can display.
///
/// A charge percentage is only meaningful when it is finite and non-negative.
/// A maximum capacity of zero (or a non-finite one) means the source is not a
/// real battery, and it is also what makes `state_of_charge()` return NaN,
/// which downstream gauge widgets refuse to render.
fn has_usable_readings(charge_percent: f32, max_capacity_wh: f32) -> bool {
    charge_percent.is_finite()
        && charge_percent >= 0.0
        && max_capacity_wh.is_finite()
        && max_capacity_wh > 0.0
}

#[cfg(test)]
mod tests {
    use super::has_usable_readings;

    #[test]
    fn accepts_a_real_battery() {
        assert!(has_usable_readings(0.0, 52.6));
        assert!(has_usable_readings(63.5, 52.6));
        assert!(has_usable_readings(100.0, 52.6));
    }

    #[test]
    fn rejects_non_finite_charge_percent() {
        // 0.0 / 0.0 on a machine whose power source reports no capacity.
        assert!(!has_usable_readings(f32::NAN, 52.6));
        assert!(!has_usable_readings(f32::INFINITY, 52.6));
        assert!(!has_usable_readings(f32::NEG_INFINITY, 52.6));
    }

    #[test]
    fn rejects_out_of_range_charge_percent() {
        assert!(!has_usable_readings(-0.1, 52.6));
        assert!(!has_usable_readings(-100.0, 52.6));
    }

    #[test]
    fn rejects_unusable_max_capacity() {
        assert!(!has_usable_readings(63.5, 0.0));
        assert!(!has_usable_readings(63.5, -1.0));
        assert!(!has_usable_readings(63.5, f32::NAN));
        assert!(!has_usable_readings(63.5, f32::INFINITY));
    }

    #[test]
    fn rejects_a_pseudo_power_source() {
        // Desktop Macs: zero energy and zero max capacity, so the ratio is NaN.
        assert!(!has_usable_readings(f32::NAN, 0.0));
    }
}
