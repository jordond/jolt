mod client;
mod protocol;
mod server;

#[allow(unused_imports)]
pub use client::async_client::{AsyncClientError, AsyncDaemonClient};
pub use client::DaemonClient;
#[allow(unused_imports)]
pub use protocol::{
    BatterySnapshot, BatteryState, CycleSummary, DaemonRequest, DaemonResponse, DaemonStatus,
    DataSnapshot, KillProcessResult, KillSignal, PowerMode, PowerSnapshot, ProcessSnapshot,
    ProcessState, MAX_SUBSCRIBERS,
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
