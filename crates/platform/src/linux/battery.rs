use std::fs;
use std::path::{Path, PathBuf};
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

const POWER_SUPPLY_PATH: &str = "/sys/class/power_supply";

pub struct LinuxBattery {
    info: BatteryInfo,
    manager: Manager,
    battery_path: Option<PathBuf>,
}

impl BatteryProvider for LinuxBattery {
    fn new() -> Result<Self> {
        let manager = Manager::new()?;
        let battery_path = find_battery_path();
        let mut provider = Self {
            info: BatteryInfo::default(),
            manager,
            battery_path,
        };
        provider.refresh()?;
        Ok(provider)
    }

    fn refresh(&mut self) -> Result<()> {
        self.refresh_from_battery_crate()?;
        self.refresh_linux_extras();
        Ok(())
    }

    fn info(&self) -> &BatteryInfo {
        &self.info
    }

    fn is_supported() -> bool {
        Path::new(POWER_SUPPLY_PATH).exists()
    }
}

impl LinuxBattery {
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
        self.info.temperature_c = battery
            .temperature()
            .map(|t| t.get::<degree_celsius>() as f32);
        self.info.time_to_full = battery
            .time_to_full()
            .map(|t| Duration::from_secs(t.get::<second>() as u64));
        self.info.time_to_empty = battery
            .time_to_empty()
            .map(|t| Duration::from_secs(t.get::<second>() as u64));

        let battery_state = battery.state();
        self.info.state = ChargeState::from(battery_state);

        Ok(())
    }

    fn refresh_linux_extras(&mut self) {
        self.info.external_connected = is_ac_connected();

        if let Some(ref path) = self.battery_path {
            self.detect_not_charging_state(path);
            self.read_amperage(path);
        }
    }

    fn detect_not_charging_state(&mut self, battery_path: &Path) {
        if self.info.state != ChargeState::Unknown {
            if self.info.external_connected
                && self.info.state != ChargeState::Charging
                && self.info.state != ChargeState::Full
            {
                self.info.state = ChargeState::NotCharging;
            }
            return;
        }

        let status_path = battery_path.join("status");
        if let Ok(status) = fs::read_to_string(status_path) {
            let status = status.trim();
            if status.eq_ignore_ascii_case("Not charging") {
                self.info.state = ChargeState::NotCharging;
            } else if self.info.external_connected {
                self.info.state = ChargeState::NotCharging;
            }
        }
    }

    fn read_amperage(&mut self, battery_path: &Path) {
        let current_path = battery_path.join("current_now");
        if let Ok(content) = fs::read_to_string(current_path) {
            if let Ok(microamps) = content.trim().parse::<i64>() {
                let milliamps = (microamps / 1000) as i32;
                self.info.amperage_ma = if self.info.state == ChargeState::Discharging {
                    -milliamps.abs()
                } else {
                    milliamps.abs()
                };
            }
        }
    }
}

fn find_battery_path() -> Option<PathBuf> {
    let power_supply = Path::new(POWER_SUPPLY_PATH);
    if !power_supply.exists() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(power_supply) {
        for entry in entries.flatten() {
            let path = entry.path();
            let type_path = path.join("type");
            if let Ok(type_content) = fs::read_to_string(type_path) {
                if type_content.trim() == "Battery" {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn is_ac_connected() -> bool {
    let power_supply = Path::new(POWER_SUPPLY_PATH);
    if !power_supply.exists() {
        return false;
    }

    if let Ok(entries) = fs::read_dir(power_supply) {
        for entry in entries.flatten() {
            let path = entry.path();
            let type_path = path.join("type");
            if let Ok(type_content) = fs::read_to_string(&type_path) {
                if type_content.trim() == "Mains" {
                    let online_path = path.join("online");
                    if let Ok(online) = fs::read_to_string(online_path) {
                        if online.trim() == "1" {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}
