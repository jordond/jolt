use std::process::Command;
use std::time::Duration;

use color_eyre::eyre::{eyre, Result};
use starship_battery::units::electric_potential::millivolt;
use starship_battery::units::energy::watt_hour;
use starship_battery::units::power::watt;
use starship_battery::units::ratio::percent;
use starship_battery::units::thermodynamic_temperature::degree_celsius;
use starship_battery::units::time::second;
use starship_battery::Manager;

use crate::battery::{BatteryInfo, BatteryProvider};
use crate::types::{BatteryTechnology, ChargeState};

pub struct MacOSBattery {
    info: BatteryInfo,
    manager: Manager,
}

impl BatteryProvider for MacOSBattery {
    fn new() -> Result<Self> {
        let manager = Manager::new()?;
        let mut provider = Self {
            info: BatteryInfo::default(),
            manager,
        };
        provider.refresh()?;
        Ok(provider)
    }

    fn refresh(&mut self) -> Result<()> {
        self.refresh_from_battery_crate()?;
        self.refresh_ioreg_extras();
        Ok(())
    }

    fn info(&self) -> &BatteryInfo {
        &self.info
    }
}

impl MacOSBattery {
    fn refresh_from_battery_crate(&mut self) -> Result<()> {
        let mut battery = self
            .manager
            .batteries()?
            .next()
            .ok_or_else(|| eyre!("No battery found"))??;

        self.manager.refresh(&mut battery)?;

        self.info.charge_percent = battery.state_of_charge().get::<percent>();
        self.info.max_capacity_wh = battery.energy_full().get::<watt_hour>();
        self.info.design_capacity_wh = battery.energy_full_design().get::<watt_hour>();
        self.info.voltage_mv = battery.voltage().get::<millivolt>() as u32;
        self.info.health_percent = battery.state_of_health().get::<percent>();
        self.info.cycle_count = battery.cycle_count();
        self.info.temperature_c = battery.temperature().map(|t| t.get::<degree_celsius>());
        self.info.time_to_full = battery
            .time_to_full()
            .map(|t| Duration::from_secs(t.get::<second>() as u64));
        self.info.time_to_empty = battery
            .time_to_empty()
            .map(|t| Duration::from_secs(t.get::<second>() as u64));

        self.info.state = ChargeState::from(battery.state());

        self.info.vendor = battery.vendor().map(|s| s.to_string());
        self.info.model = battery.model().map(|s| s.to_string());
        self.info.serial_number = battery.serial_number().map(|s| s.to_string());
        self.info.technology = BatteryTechnology::from(battery.technology());
        self.info.energy_wh = battery.energy().get::<watt_hour>();
        self.info.energy_rate_watts = battery.energy_rate().get::<watt>();

        Ok(())
    }

    fn refresh_ioreg_extras(&mut self) {
        if let Ok(output) = Command::new("ioreg")
            .args(["-rn", "AppleSmartBattery"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_ioreg_output(&stdout);
        }
    }

    fn parse_ioreg_output(&mut self, output: &str) {
        let mut is_charging = false;

        for line in output.lines() {
            let line = line.trim();

            if line.contains("\"Amperage\"") || line.contains("\"InstantAmperage\"") {
                if let Some(val) = extract_number(line) {
                    self.info.amperage_ma = val as i32;
                }
            } else if line.contains("\"ExternalConnected\"") {
                self.info.external_connected = line.contains("Yes");
            } else if line.contains("\"IsCharging\"") {
                is_charging = line.contains("Yes");
            } else if line.contains("\"BatteryData\"") {
                apply_battery_data_capacities(line, &mut self.info);

                if let Some(pos) = line.find("\"DailyMinSoc\"=") {
                    let after = &line[pos + 14..];
                    if let Some(end) = after.find(|c: char| !c.is_ascii_digit() && c != '.') {
                        if let Ok(val) = after[..end].parse::<f32>() {
                            self.info.daily_min_soc = Some(val);
                        }
                    }
                }
                if let Some(pos) = line.find("\"DailyMaxSoc\"=") {
                    let after = &line[pos + 14..];
                    if let Some(end) = after.find(|c: char| !c.is_ascii_digit() && c != '.') {
                        if let Ok(val) = after[..end].parse::<f32>() {
                            self.info.daily_max_soc = Some(val);
                        }
                    }
                }
            }
        }

        if self.info.external_connected {
            self.info.charger_watts = parse_charger_watts(output);

            if is_charging {
                self.info.state = ChargeState::Charging;
            } else if self.info.charge_percent >= 99.0 {
                self.info.state = ChargeState::Full;
            } else {
                self.info.state = ChargeState::NotCharging;
            }
        } else {
            self.info.state = ChargeState::Discharging;
            self.info.charger_watts = None;
        }
    }
}

fn extract_number(line: &str) -> Option<i64> {
    line.split('=').nth(1)?.trim().parse::<i64>().ok()
}

/// Apply capacity values from the nested `BatteryData` dictionary.
///
/// macOS 27 moved `AppleRawMaxCapacity`, `AppleRawCurrentCapacity`, and
/// `DesignCapacity` from the top-level `AppleSmartBattery` properties into
/// `BatteryData`. `starship-battery` still reads the top-level keys, so it falls
/// back to the percentage-based `MaxCapacity` and reports a near-zero capacity.
fn apply_battery_data_capacities(line: &str, info: &mut BatteryInfo) {
    let Some(battery_data_pos) = line.find("\"BatteryData\"") else {
        return;
    };
    let battery_data = &line[battery_data_pos..];

    let Some(max_mah) = extract_battery_data_number(battery_data, "AppleRawMaxCapacity")
        .or_else(|| extract_battery_data_number(battery_data, "FullChargeCapacity"))
    else {
        return;
    };

    if max_mah <= 0 || info.voltage_mv == 0 {
        return;
    }

    let voltage = info.voltage_mv as f32 / 1000.0;
    info.max_capacity_wh = max_mah as f32 * voltage / 1000.0;

    if let Some(design_mah) = extract_battery_data_number(battery_data, "DesignCapacity") {
        if design_mah > 0 {
            info.design_capacity_wh = design_mah as f32 * voltage / 1000.0;
            info.health_percent = ((max_mah as f32 / design_mah as f32) * 100.0).clamp(0.0, 100.0);
        }
    }

    if info.charge_percent > 0.0 {
        info.energy_wh = info.max_capacity_wh * (info.charge_percent / 100.0);
    }
}

fn extract_battery_data_number(battery_data: &str, key: &str) -> Option<i64> {
    let key_pos = battery_data.find(&format!("\"{key}\""))?;
    let after_key = &battery_data[key_pos..];
    let value_start = after_key.find('=')? + 1;
    let value = after_key[value_start..].trim_start();
    let value_end = value
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(value.len());
    value[..value_end].parse::<i64>().ok()
}

fn parse_charger_watts(output: &str) -> Option<u32> {
    for line in output.lines() {
        if line.contains("\"AdapterDetails\"") || line.contains("\"AppleRawAdapterDetails\"") {
            if let Some(watts_pos) = line.find("\"Watts\"=") {
                let after_watts = &line[watts_pos + 8..];
                let end = after_watts
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(after_watts.len());
                if let Ok(watts) = after_watts[..end].parse::<u32>() {
                    return Some(watts);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_info() -> BatteryInfo {
        BatteryInfo {
            charge_percent: 88.0,
            voltage_mv: 12_550,
            max_capacity_wh: 1.2573,
            design_capacity_wh: 1.2573,
            health_percent: 100.0,
            energy_wh: 1.106,
            ..BatteryInfo::default()
        }
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.01,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn applies_nested_battery_data_capacities() {
        let line = r#"|   "BatteryData" = {"CurrentCapacity"=88,"AppleRawMaxCapacity"=4475,"AppleRawCurrentCapacity"=3780,"DesignCapacity"=4629,"MaxCapacity"=100}"#;
        let mut info = test_info();

        apply_battery_data_capacities(line, &mut info);

        assert_close(info.max_capacity_wh, 56.16);
        assert_close(info.design_capacity_wh, 58.09);
        assert_close(info.energy_wh, 49.42);
        assert_close(info.health_percent, 96.67);
    }

    #[test]
    fn leaves_capacities_unchanged_without_nested_keys() {
        let line = r#"|   "BatteryData" = {"CurrentCapacity"=88,"MaxCapacity"=100}"#;
        let mut info = test_info();

        apply_battery_data_capacities(line, &mut info);

        assert_close(info.max_capacity_wh, 1.2573);
        assert_close(info.design_capacity_wh, 1.2573);
        assert_close(info.energy_wh, 1.106);
        assert_close(info.health_percent, 100.0);
    }

    #[test]
    fn updates_capacity_without_design_capacity() {
        let line = r#"|   "BatteryData" = {"AppleRawMaxCapacity"=4475}"#;
        let mut info = test_info();

        apply_battery_data_capacities(line, &mut info);

        assert_close(info.max_capacity_wh, 56.16);
        assert_close(info.design_capacity_wh, 1.2573);
        assert_close(info.energy_wh, 49.42);
        assert_close(info.health_percent, 100.0);
    }

    #[test]
    fn falls_back_to_full_charge_capacity() {
        let line = r#"|   "BatteryData" = {"FullChargeCapacity" = 4475,"DesignCapacity" = 4629}"#;
        let mut info = test_info();

        apply_battery_data_capacities(line, &mut info);

        assert_close(info.max_capacity_wh, 56.16);
        assert_close(info.design_capacity_wh, 58.09);
    }

    #[test]
    fn ignores_zero_capacity() {
        let line = r#"|   "BatteryData" = {"AppleRawMaxCapacity"=0,"DesignCapacity"=4629}"#;
        let mut info = test_info();

        apply_battery_data_capacities(line, &mut info);

        assert_close(info.max_capacity_wh, 1.2573);
        assert_close(info.design_capacity_wh, 1.2573);
    }
}
