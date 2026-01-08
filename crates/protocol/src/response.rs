use serde::{Deserialize, Serialize};

use crate::types::{
    ChargeSession, CycleSummary, DaemonStatus, DailyCycle, DailyStat, DailyTopProcess,
    DataSnapshot, HourlyStat, KillProcessResult, Sample,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonResponse {
    Status(DaemonStatus),
    HourlyStats(Vec<HourlyStat>),
    DailyStats(Vec<DailyStat>),
    TopProcesses(Vec<DailyTopProcess>),
    RecentSamples(Vec<Sample>),
    Ok,
    Error(String),
    Subscribed,
    Unsubscribed,
    DataUpdate(DataSnapshot),
    CurrentData(DataSnapshot),
    KillResult(KillProcessResult),
    SubscriptionRejected { reason: String },
    CycleSummary(CycleSummary),
    ChargeSessions(Vec<ChargeSession>),
    DailyCycles(Vec<DailyCycle>),
}

impl DaemonResponse {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
