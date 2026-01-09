mod client;
mod protocol;
mod server;
pub mod service;

pub use client::DaemonClient;
#[allow(unused_imports)]
pub use protocol::{
    BatterySnapshot, BatteryState, ChargeSession, ChargingState, CycleSummary, DaemonRequest,
    DaemonResponse, DaemonStatus, DailyCycle, DailyStat, DailyTopProcess, DataSnapshot,
    ForecastSnapshot, ForecastSource, HourlyStat, KillProcessResult, KillSignal, PowerMode,
    PowerSnapshot, ProcessSnapshot, ProcessState, Sample, SessionType, SystemSnapshot,
    SystemStatsSnapshot, MAX_SUBSCRIBERS, MIN_SUPPORTED_VERSION, PROTOCOL_VERSION,
};
pub use server::run_daemon;
#[allow(unused_imports)]
pub use server::DaemonError;

use std::path::PathBuf;

use crate::config::runtime_dir;

const SOCKET_NAME: &str = "jolt.sock";

pub fn socket_path() -> PathBuf {
    runtime_dir().join(SOCKET_NAME)
}

pub fn is_daemon_running() -> bool {
    DaemonClient::connect().is_ok()
}
