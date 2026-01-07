use serde::{Deserialize, Serialize};

use crate::data::{DailyStat, DailyTopProcess, HourlyStat, Sample};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonRequest {
    GetStatus,
    GetHourlyStats {
        from: i64,
        to: i64,
    },
    GetDailyStats {
        from: String,
        to: String,
    },
    GetTopProcessesRange {
        from: String,
        to: String,
        limit: usize,
    },
    GetRecentSamples {
        window_secs: u64,
    },
    Shutdown,
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
    Status(DaemonStatus),
    HourlyStats(Vec<HourlyStat>),
    DailyStats(Vec<DailyStat>),
    TopProcesses(Vec<DailyTopProcess>),
    RecentSamples(Vec<Sample>),
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
