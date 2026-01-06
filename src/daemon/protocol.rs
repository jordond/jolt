use serde::{Deserialize, Serialize};

use crate::data::{DailyStat, DailyTopProcess, DatabaseStats, HourlyStat, Sample};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonRequest {
    Ping,
    GetStatus,
    GetCurrentData,
    GetSamples {
        from: i64,
        to: i64,
    },
    GetHourlyStats {
        from: i64,
        to: i64,
    },
    GetDailyStats {
        from: String,
        to: String,
    },
    GetTopProcesses {
        date: String,
        limit: usize,
    },
    GetTopProcessesRange {
        from: String,
        to: String,
        limit: usize,
    },
    GetDatabaseStats,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentData {
    pub battery_percent: f32,
    pub power_watts: f32,
    pub cpu_power: f32,
    pub gpu_power: f32,
    pub charging: bool,
    pub health_percent: f32,
    pub time_remaining_mins: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub uptime_secs: u64,
    pub sample_count: i64,
    pub last_sample_time: Option<i64>,
    pub database_size_bytes: u64,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonResponse {
    Pong,
    Status(DaemonStatus),
    CurrentData(CurrentData),
    Samples(Vec<Sample>),
    HourlyStats(Vec<HourlyStat>),
    DailyStats(Vec<DailyStat>),
    TopProcesses(Vec<DailyTopProcess>),
    DatabaseStats(DatabaseStats),
    Ok,
    Error(String),
}

impl DaemonRequest {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl DaemonResponse {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
