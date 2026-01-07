use color_eyre::eyre::Result;
use std::process::Command;
use std::time::Duration;

use crate::daemon::{BatterySnapshot, BatteryState as ProtocolBatteryState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeState {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

pub struct BatteryData {
    current_charge: f32,
    max_capacity: f32,
    design_capacity: f32,
    voltage_mv: u32,
    amperage_ma: i32,
    state: ChargeState,
    time_to_full: Option<Duration>,
    time_to_empty: Option<Duration>,
    cycle_count: Option<u32>,
    health_percent: f32,
    charger_watts: Option<u32>,
    external_connected: bool,
}

impl BatteryData {
    pub fn new() -> Result<Self> {
        let mut data = Self {
            current_charge: 100.0,
            max_capacity: 100.0,
            design_capacity: 100.0,
            voltage_mv: 11500,
            amperage_ma: 0,
            state: ChargeState::Unknown,
            time_to_full: None,
            time_to_empty: None,
            cycle_count: None,
            health_percent: 100.0,
            charger_watts: None,
            external_connected: false,
        };

        data.refresh()?;
        Ok(data)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_from_pmset();
        self.refresh_from_ioreg();
        Ok(())
    }

    fn refresh_from_pmset(&mut self) {
        if let Ok(output) = Command::new("pmset").args(["-g", "batt"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_pmset_output(&stdout);
        }
    }

    fn parse_pmset_output(&mut self, output: &str) {
        for line in output.lines() {
            if line.contains("InternalBattery") || line.contains('%') {
                if let Some(percent_pos) = line.find('%') {
                    let start = line[..percent_pos]
                        .rfind(char::is_whitespace)
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    if let Ok(percent) = line[start..percent_pos].trim().parse::<f32>() {
                        self.current_charge = percent;
                    }
                }

                // Order matters: check more specific patterns first
                if line.contains("discharging") {
                    self.state = ChargeState::Discharging;
                } else if line.contains("not charging") {
                    self.state = ChargeState::NotCharging;
                } else if line.contains("charged") {
                    self.state = ChargeState::Full;
                } else if line.contains("charging") {
                    self.state = ChargeState::Charging;
                }

                if let Some(time) = parse_time_remaining(line) {
                    if self.state == ChargeState::Charging {
                        self.time_to_full = Some(time);
                    } else {
                        self.time_to_empty = Some(time);
                    }
                }
            }
        }
    }

    fn refresh_from_ioreg(&mut self) {
        if let Ok(output) = Command::new("ioreg")
            .args(["-rn", "AppleSmartBattery"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_ioreg_output(&stdout);
        }
    }

    fn parse_ioreg_output(&mut self, output: &str) {
        let mut nominal_capacity: Option<f32> = None;
        let mut design_capacity: Option<f32> = None;
        let mut is_charging = false;
        let mut avg_time_to_full: Option<u64> = None;
        let mut avg_time_to_empty: Option<u64> = None;

        for line in output.lines() {
            let line = line.trim();

            if line.contains("\"NominalChargeCapacity\"") {
                if let Some(val) = extract_number(line) {
                    nominal_capacity = Some(val as f32);
                }
            } else if line.contains("\"DesignCapacity\"") && !line.contains("FedDesignCapacity") {
                if let Some(val) = extract_number(line) {
                    design_capacity = Some(val as f32);
                }
            } else if line.contains("\"CycleCount\"")
                && !line.contains("DesignCycleCount")
                && !line.contains("Lifetime")
            {
                if let Some(val) = extract_number(line) {
                    self.cycle_count = Some(val as u32);
                }
            } else if line.starts_with("\"Voltage\"") {
                if let Some(val) = extract_number(line) {
                    self.voltage_mv = val as u32;
                }
            } else if line.contains("\"Amperage\"") || line.contains("\"InstantAmperage\"") {
                if let Some(val) = extract_number(line) {
                    self.amperage_ma = val as i32;
                }
            } else if line.contains("\"ExternalConnected\"") {
                self.external_connected = line.contains("Yes");
            } else if line.contains("\"IsCharging\"") {
                is_charging = line.contains("Yes");
            } else if line.contains("\"AvgTimeToFull\"") {
                if let Some(val) = extract_number(line) {
                    if val > 0 && val < 65535 {
                        avg_time_to_full = Some(val as u64);
                    }
                }
            } else if line.contains("\"AvgTimeToEmpty\"") {
                if let Some(val) = extract_number(line) {
                    if val > 0 && val < 65535 {
                        avg_time_to_empty = Some(val as u64);
                    }
                }
            }
        }

        if let Some(mins) = avg_time_to_full {
            self.time_to_full = Some(Duration::from_secs(mins * 60));
        }
        if let Some(mins) = avg_time_to_empty {
            self.time_to_empty = Some(Duration::from_secs(mins * 60));
        }

        if self.external_connected {
            if is_charging {
                self.state = ChargeState::Charging;
            } else if self.current_charge >= 99.0 {
                self.state = ChargeState::Full;
            } else {
                self.state = ChargeState::NotCharging;
            }
        } else {
            self.state = ChargeState::Discharging;
        }

        if self.external_connected {
            self.charger_watts = parse_charger_watts(output);
        } else {
            self.charger_watts = None;
        }

        if let (Some(nominal), Some(design)) = (nominal_capacity, design_capacity) {
            if design > 0.0 {
                self.max_capacity = nominal;
                self.design_capacity = design;
                self.health_percent = (nominal / design) * 100.0;
            }
        }
    }

    pub fn charge_percent(&self) -> f32 {
        self.current_charge
    }

    pub fn max_capacity_wh(&self) -> f32 {
        self.max_capacity * (self.voltage_mv as f32 / 1000.0) / 1000.0
    }

    pub fn design_capacity_wh(&self) -> f32 {
        self.design_capacity * (self.voltage_mv as f32 / 1000.0) / 1000.0
    }

    pub fn state(&self) -> ChargeState {
        self.state
    }

    pub fn state_label(&self) -> &'static str {
        match self.state {
            ChargeState::Charging => "Charging",
            ChargeState::Discharging => "On Battery",
            ChargeState::Full => "Full",
            ChargeState::NotCharging => "Not Charging",
            ChargeState::Unknown => "Unknown",
        }
    }

    pub fn time_remaining(&self) -> Option<Duration> {
        let system_estimate = match self.state {
            ChargeState::Charging => self.time_to_full,
            ChargeState::Discharging => self.time_to_empty,
            _ => None,
        };

        if system_estimate.is_some() {
            return system_estimate;
        }

        if self.state == ChargeState::Discharging {
            if let Some(watts) = self.discharge_watts() {
                if watts > 0.1 {
                    let capacity_wh = self.max_capacity_wh();
                    let current_wh = capacity_wh * (self.current_charge / 100.0);
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
        self.cycle_count
    }

    pub fn health_percent(&self) -> f32 {
        self.health_percent
    }

    pub fn is_charging(&self) -> bool {
        matches!(self.state, ChargeState::Charging)
    }

    pub fn charging_watts(&self) -> Option<f32> {
        if self.state == ChargeState::Charging && self.amperage_ma > 0 {
            let watts = (self.amperage_ma as f32 / 1000.0) * (self.voltage_mv as f32 / 1000.0);
            Some(watts)
        } else {
            None
        }
    }

    pub fn charger_watts(&self) -> Option<u32> {
        self.charger_watts
    }

    pub fn voltage_mv(&self) -> u32 {
        self.voltage_mv
    }

    pub fn amperage_ma(&self) -> i32 {
        self.amperage_ma
    }

    pub fn external_connected(&self) -> bool {
        self.external_connected
    }

    pub fn discharge_watts(&self) -> Option<f32> {
        if self.state == ChargeState::Discharging && self.amperage_ma < 0 {
            let watts =
                (self.amperage_ma.abs() as f32 / 1000.0) * (self.voltage_mv as f32 / 1000.0);
            Some(watts)
        } else {
            None
        }
    }

    pub fn update_from_snapshot(&mut self, snapshot: &BatterySnapshot) {
        self.current_charge = snapshot.charge_percent;
        self.health_percent = snapshot.health_percent;
        self.max_capacity =
            snapshot.max_capacity_wh * 1000.0 / (snapshot.voltage_mv as f32 / 1000.0);
        self.design_capacity =
            snapshot.design_capacity_wh * 1000.0 / (snapshot.voltage_mv as f32 / 1000.0);
        self.voltage_mv = snapshot.voltage_mv;
        self.amperage_ma = snapshot.amperage_ma;
        self.cycle_count = snapshot.cycle_count;
        self.external_connected = snapshot.external_connected;
        self.charger_watts = snapshot.charger_watts;

        self.state = match snapshot.state {
            ProtocolBatteryState::Charging => ChargeState::Charging,
            ProtocolBatteryState::Discharging => ChargeState::Discharging,
            ProtocolBatteryState::Full => ChargeState::Full,
            ProtocolBatteryState::NotCharging => ChargeState::NotCharging,
            ProtocolBatteryState::Unknown => ChargeState::Unknown,
        };

        self.time_to_full = if self.state == ChargeState::Charging {
            snapshot
                .time_remaining_mins
                .map(|m| Duration::from_secs(m * 60))
        } else {
            None
        };

        self.time_to_empty = if self.state == ChargeState::Discharging {
            snapshot
                .time_remaining_mins
                .map(|m| Duration::from_secs(m * 60))
        } else {
            None
        };
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

fn parse_time_remaining(line: &str) -> Option<Duration> {
    let remaining_idx = line.find("remaining")?;
    let before_remaining = line[..remaining_idx].trim_end();

    let time_end = before_remaining.len();
    let time_start = before_remaining.rfind(|c: char| !c.is_ascii_digit() && c != ':')?;
    let time_str = &before_remaining[time_start + 1..time_end];

    let colon_pos = time_str.find(':')?;
    let hours: u64 = time_str[..colon_pos].parse().ok()?;
    let mins: u64 = time_str[colon_pos + 1..].parse().ok()?;

    Some(Duration::from_secs(hours * 3600 + mins * 60))
}
