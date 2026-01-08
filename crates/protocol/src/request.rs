use serde::{Deserialize, Serialize};

use crate::types::KillSignal;

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
    Subscribe,
    Unsubscribe,
    GetCurrentData,
    KillProcess {
        pid: u32,
        signal: KillSignal,
    },
    SetBroadcastInterval {
        interval_ms: u64,
    },
    GetCycleSummary {
        days: u32,
    },
    GetChargeSessions {
        from: i64,
        to: i64,
    },
    GetDailyCycles {
        from: String,
        to: String,
    },
}

impl DaemonRequest {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
