use std::process::Command;
use std::time::Duration;

use battery::units::electric_potential::millivolt;
use battery::units::energy::watt_hour;
use battery::units::ratio::percent;
use battery::units::thermodynamic_temperature::degree_celsius;
use battery::units::time::second;
use battery::Manager;
use color_eyre::eyre::{eyre, Result};

use crate::battery::{BatteryInfo, BatteryProvider};
use crate::types::ChargeState;

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
