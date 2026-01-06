mod client;
mod protocol;
mod server;

pub use client::DaemonClient;
#[allow(unused_imports)]
pub use protocol::{DaemonRequest, DaemonResponse, DaemonStatus};
pub use server::run_daemon;
#[allow(unused_imports)]
pub use server::DaemonError;

use std::path::PathBuf;

use crate::config::runtime_dir;

const SOCKET_NAME: &str = "jolt.sock";
const LOG_NAME: &str = "daemon.log";

pub fn socket_path() -> PathBuf {
    runtime_dir().join(SOCKET_NAME)
}

pub fn log_path() -> PathBuf {
    runtime_dir().join(LOG_NAME)
}

pub fn is_daemon_running() -> bool {
    DaemonClient::connect().is_ok()
}
