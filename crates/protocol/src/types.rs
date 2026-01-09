use serde::{Deserialize, Serialize};

use crate::version::{MIN_SUPPORTED_VERSION, PROTOCOL_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    NotCharging,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PowerMode {
    LowPower,
    #[default]
    Automatic,
    HighPerformance,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    Running,
    Sleeping,
    Idle,
    Stopped,
    Zombie,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KillSignal {
    Graceful,
    #[default]
    Force,
}

impl KillSignal {
    pub fn as_arg(&self) -> &'static str {
        match self {
            KillSignal::Graceful => "-15",
            KillSignal::Force => "-9",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum ChargingState {
    Discharging = 0,
    Charging = 1,
    Full = 2,
    #[default]
    Unknown = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum SessionType {
    Charge = 0,
    #[default]
    Discharge = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatterySnapshot {
    pub charge_percent: f32,
    pub state: BatteryState,
    pub state_label: String,
    pub health_percent: f32,
    pub max_capacity_wh: f32,
    pub design_capacity_wh: f32,
    pub cycle_count: Option<u32>,
    pub time_remaining_mins: Option<u64>,
    pub time_remaining_formatted: Option<String>,
    pub charging_watts: Option<f32>,
    pub charger_watts: Option<u32>,
    pub discharge_watts: Option<f32>,
    pub voltage_mv: u32,
    pub amperage_ma: i32,
    pub external_connected: bool,
    pub temperature_c: Option<f32>,
    pub daily_min_soc: Option<f32>,
    pub daily_max_soc: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PowerSnapshot {
    pub cpu_power_watts: f32,
    pub gpu_power_watts: f32,
    pub total_power_watts: f32,
    pub power_mode: PowerMode,
    pub power_mode_label: String,
    pub is_warmed_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemSnapshot {
    pub chip: String,
    pub os_version: String,
    pub p_cores: u32,
    pub e_cores: u32,
}

impl SystemSnapshot {
    pub fn cores_display(&self) -> String {
        if self.p_cores > 0 && self.e_cores > 0 {
            format!("{}P+{}E", self.p_cores, self.e_cores)
        } else {
            format!("{}", self.p_cores + self.e_cores)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ForecastSource {
    Daemon,
    Session,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForecastSnapshot {
    pub duration_secs: Option<u64>,
    pub avg_power_watts: Option<f32>,
    pub sample_count: usize,
    pub source: ForecastSource,
}

impl ForecastSnapshot {
    pub fn formatted(&self) -> Option<String> {
        self.duration_secs.map(|secs| {
            let total_mins = secs / 60;
            if total_mins == 0 {
                return "< 1m".to_string();
            }
            let hours = total_mins / 60;
            let mins = total_mins % 60;

            if hours > 0 {
                format!("{}h {}m", hours, mins)
            } else {
                format!("{}m", mins)
            }
        })
    }

    pub fn has_forecast(&self) -> bool {
        self.duration_secs.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub energy_impact: f32,
    pub parent_pid: Option<u32>,
    pub children: Option<Vec<ProcessSnapshot>>,
    pub is_killable: bool,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub status: ProcessState,
    pub run_time_secs: u64,
    pub total_cpu_time_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DataSnapshot {
    pub timestamp: i64,
    pub battery: BatterySnapshot,
    pub power: PowerSnapshot,
    pub processes: Vec<ProcessSnapshot>,
    pub system: SystemSnapshot,
    pub forecast: ForecastSnapshot,
}

impl Default for DataSnapshot {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp(),
            battery: BatterySnapshot::default(),
            power: PowerSnapshot::default(),
            processes: Vec::new(),
            system: SystemSnapshot::default(),
            forecast: ForecastSnapshot::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub uptime_secs: u64,
    pub sample_count: i64,
    pub last_sample_time: Option<i64>,
    pub database_size_bytes: u64,
    pub version: String,
    pub subscriber_count: usize,
    pub history_enabled: bool,
    pub protocol_version: u32,
    pub min_supported_version: u32,
}

impl Default for DaemonStatus {
    fn default() -> Self {
        Self {
            running: false,
            uptime_secs: 0,
            sample_count: 0,
            last_sample_time: None,
            database_size_bytes: 0,
            version: String::new(),
            subscriber_count: 0,
            history_enabled: false,
            protocol_version: PROTOCOL_VERSION,
            min_supported_version: MIN_SUPPORTED_VERSION,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillProcessResult {
    pub pid: u32,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CycleSummary {
    pub total_cycles_macos: u32,
    pub partial_cycles_calculated: f32,
    pub avg_daily_cycles: f32,
    pub avg_depth_of_discharge: f32,
    pub avg_charge_sessions_per_day: f32,
    pub time_at_high_soc_percent: f32,
    pub estimated_cycles_remaining: Option<u32>,
    pub days_analyzed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub timestamp: i64,
    pub battery_percent: f32,
    pub power_watts: f32,
    pub cpu_power: f32,
    pub gpu_power: f32,
    pub charging_state: ChargingState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStat {
    pub hour_start: i64,
    pub avg_power: f32,
    pub max_power: f32,
    pub min_power: f32,
    pub avg_battery: f32,
    pub battery_delta: f32,
    pub total_samples: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStat {
    pub date: String,
    pub avg_power: f32,
    pub max_power: f32,
    pub total_energy_wh: f32,
    pub screen_on_hours: f32,
    pub charging_hours: f32,
    pub battery_cycles: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTopProcess {
    pub date: String,
    pub process_name: String,
    pub total_impact: f32,
    pub avg_cpu: f32,
    pub avg_memory_mb: f32,
    pub sample_count: i32,
    pub avg_power: f32,
    pub total_energy_wh: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargeSession {
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub start_percent: f32,
    pub end_percent: Option<f32>,
    pub energy_wh: Option<f32>,
    pub charger_watts: Option<u32>,
    pub avg_power_watts: Option<f32>,
    pub session_type: SessionType,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyCycle {
    pub date: String,
    pub charge_sessions: i32,
    pub discharge_sessions: i32,
    pub total_charging_mins: i32,
    pub total_discharge_mins: i32,
    pub deepest_discharge_percent: Option<f32>,
    pub energy_charged_wh: f32,
    pub energy_discharged_wh: f32,
    pub partial_cycles: f32,
    pub macos_cycle_count: Option<i32>,
    pub avg_temperature_c: Option<f32>,
    pub time_at_high_soc_mins: i32,
}

pub const MAX_SUBSCRIBERS: usize = 10;
