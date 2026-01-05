use color_eyre::eyre::Result;
use std::process::Command;
use std::time::Duration;

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
    state: ChargeState,
    time_to_full: Option<Duration>,
    time_to_empty: Option<Duration>,
    cycle_count: Option<u32>,
    health_percent: f32,
}

impl BatteryData {
    pub fn new() -> Result<Self> {
        let mut data = Self {
            current_charge: 100.0,
            max_capacity: 100.0,
            design_capacity: 100.0,
            state: ChargeState::Unknown,
            time_to_full: None,
            time_to_empty: None,
            cycle_count: None,
            health_percent: 100.0,
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

                if line.contains("charging") && !line.contains("not charging") {
                    self.state = ChargeState::Charging;
                } else if line.contains("discharging") {
                    self.state = ChargeState::Discharging;
                } else if line.contains("charged") {
                    self.state = ChargeState::Full;
                } else if line.contains("not charging") {
                    self.state = ChargeState::NotCharging;
                }

                if let Some(time_start) =
                    line.find(|c: char| c.is_ascii_digit() && line.contains(':'))
                {
                    let remaining = &line[time_start..];
                    if let Some(colon_pos) = remaining.find(':') {
                        let hours_str = &remaining[..colon_pos];
                        let mins_start = colon_pos + 1;
                        let mins_end = remaining[mins_start..]
                            .find(|c: char| !c.is_ascii_digit())
                            .map(|p| mins_start + p)
                            .unwrap_or(remaining.len());
                        let mins_str = &remaining[mins_start..mins_end];

                        if let (Ok(hours), Ok(mins)) =
                            (hours_str.parse::<u64>(), mins_str.parse::<u64>())
                        {
                            let duration = Duration::from_secs(hours * 3600 + mins * 60);
                            if self.state == ChargeState::Charging {
                                self.time_to_full = Some(duration);
                            } else {
                                self.time_to_empty = Some(duration);
                            }
                        }
                    }
                }
            }
        }
    }

    fn refresh_from_ioreg(&mut self) {
        if let Ok(output) = Command::new("ioreg")
            .args(["-r", "-c", "AppleSmartBattery", "-d", "1"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.parse_ioreg_output(&stdout);
        }
    }

    fn parse_ioreg_output(&mut self, output: &str) {
        for line in output.lines() {
            let line = line.trim();

            if line.contains("\"MaxCapacity\"") {
                if let Some(val) = extract_number(line) {
                    self.max_capacity = val as f32;
                }
            } else if line.contains("\"DesignCapacity\"") {
                if let Some(val) = extract_number(line) {
                    self.design_capacity = val as f32;
                }
            } else if line.contains("\"CycleCount\"") {
                if let Some(val) = extract_number(line) {
                    self.cycle_count = Some(val as u32);
                }
            }
        }

        if self.design_capacity > 0.0 {
            self.health_percent = (self.max_capacity / self.design_capacity) * 100.0;
        }
    }

    pub fn charge_percent(&self) -> f32 {
        self.current_charge
    }

    #[allow(dead_code)]
    pub fn max_capacity_wh(&self) -> f32 {
        self.max_capacity
    }

    #[allow(dead_code)]
    pub fn design_capacity_wh(&self) -> f32 {
        self.design_capacity
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
        match self.state {
            ChargeState::Charging => self.time_to_full,
            ChargeState::Discharging => self.time_to_empty,
            _ => None,
        }
    }

    pub fn time_remaining_formatted(&self) -> Option<String> {
        self.time_remaining().map(|d| {
            let total_mins = d.as_secs() / 60;
            let hours = total_mins / 60;
            let mins = total_mins % 60;

            if hours > 0 {
                format!("{}h {}m", hours, mins)
            } else {
                format!("{}m", mins)
            }
        })
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

    #[allow(dead_code)]
    pub fn is_on_ac(&self) -> bool {
        matches!(
            self.state,
            ChargeState::Charging | ChargeState::Full | ChargeState::NotCharging
        )
    }
}

fn extract_number(line: &str) -> Option<i64> {
    line.split('=').nth(1)?.trim().parse::<i64>().ok()
}
