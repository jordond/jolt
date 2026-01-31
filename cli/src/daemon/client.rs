use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use crate::daemon::protocol::{
    ChargeSession, CycleSummary, DaemonRequest, DaemonResponse, DaemonStatus, DailyCycle,
    DailyStat, DailyTopProcess, DataSnapshot, HourlyStat, KillProcessResult, KillSignal, Sample,
    MIN_SUPPORTED_VERSION, PROTOCOL_VERSION,
};
use crate::daemon::socket_path;

#[derive(Debug, Clone)]
pub struct VersionMismatchError {
    pub tui_protocol_version: u32,
    pub tui_min_supported: u32,
    pub daemon_protocol_version: u32,
    pub daemon_min_supported: u32,
    pub daemon_binary_version: String,
    pub kind: VersionMismatchKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionMismatchKind {
    TuiTooOld,
    DaemonTooOld,
}

impl std::fmt::Display for VersionMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            VersionMismatchKind::TuiTooOld => {
                write!(
                    f,
                    "Protocol version mismatch: TUI uses protocol v{}, but daemon (v{}) requires v{}+.\n\n\
                    Please update jolt:\n  \
                    brew upgrade jolt\n  \
                    # or: cargo install jolt-tui",
                    self.tui_protocol_version,
                    self.daemon_binary_version,
                    self.daemon_min_supported
                )
            }
            VersionMismatchKind::DaemonTooOld => {
                write!(
                    f,
                    "Protocol version mismatch: daemon (v{}) uses protocol v{}, but this TUI requires v{}+.\n\n\
                    Please restart the daemon:\n  \
                    jolt daemon restart",
                    self.daemon_binary_version,
                    self.daemon_protocol_version,
                    self.tui_min_supported
                )
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection failed: {0}")]
    Connection(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Daemon error: {0}")]
    Daemon(String),

    #[error("Subscription rejected: {0}")]
    SubscriptionRejected(String),

    #[error("{0}")]
    VersionMismatch(VersionMismatchError),
}

pub type Result<T> = std::result::Result<T, ClientError>;

/// Checks if the TUI and daemon protocol versions are compatible.
/// Returns Ok(()) if compatible, or Err with detailed mismatch info.
pub fn check_version_compatibility(status: &DaemonStatus) -> Result<()> {
    // Check 1: Can daemon understand TUI's messages?
    if PROTOCOL_VERSION < status.min_supported_version {
        return Err(ClientError::VersionMismatch(VersionMismatchError {
            tui_protocol_version: PROTOCOL_VERSION,
            tui_min_supported: MIN_SUPPORTED_VERSION,
            daemon_protocol_version: status.protocol_version,
            daemon_min_supported: status.min_supported_version,
            daemon_binary_version: status.version.clone(),
            kind: VersionMismatchKind::TuiTooOld,
        }));
    }

    // Check 2: Can TUI understand daemon's messages?
    if status.protocol_version < MIN_SUPPORTED_VERSION {
        return Err(ClientError::VersionMismatch(VersionMismatchError {
            tui_protocol_version: PROTOCOL_VERSION,
            tui_min_supported: MIN_SUPPORTED_VERSION,
            daemon_protocol_version: status.protocol_version,
            daemon_min_supported: status.min_supported_version,
            daemon_binary_version: status.version.clone(),
            kind: VersionMismatchKind::DaemonTooOld,
        }));
    }

    Ok(())
}

pub struct DaemonClient {
    stream: UnixStream,
    read_buffer: Vec<u8>,
}

impl DaemonClient {
    pub fn connect() -> Result<Self> {
        let path = socket_path();
        let stream = UnixStream::connect(&path)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self {
            stream,
            read_buffer: Vec::with_capacity(64 * 1024),
        })
    }

    /// Connects to the daemon and validates protocol version compatibility.
    /// This is the preferred connection method for the TUI.
    pub fn connect_with_version_check() -> Result<Self> {
        let mut client = Self::connect()?;
        let status = client.get_status()?;
        check_version_compatibility(&status)?;
        Ok(client)
    }

    fn read_line_blocking(&mut self) -> Result<String> {
        let mut temp_buf = [0u8; 8192];
        loop {
            if let Some(pos) = self.read_buffer.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = self.read_buffer.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line_bytes).to_string();
                return Ok(line);
            }
            let n = self.stream.read(&mut temp_buf)?;
            if n == 0 {
                return Err(ClientError::Protocol("Connection closed".into()));
            }
            self.read_buffer.extend_from_slice(&temp_buf[..n]);
        }
    }

    fn read_line_nonblocking(&mut self) -> Result<Option<String>> {
        let mut temp_buf = [0u8; 8192];
        loop {
            if let Some(pos) = self.read_buffer.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = self.read_buffer.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line_bytes).to_string();
                tracing::trace!(
                    line_len = line.len(),
                    buffer_remaining = self.read_buffer.len(),
                    "read_line_nonblocking: found complete line"
                );
                return Ok(Some(line));
            }
            match self.stream.read(&mut temp_buf) {
                Ok(0) => {
                    tracing::debug!("read_line_nonblocking: connection closed (read 0 bytes)");
                    return Err(ClientError::Protocol("Connection closed".into()));
                }
                Ok(n) => {
                    tracing::trace!(bytes_read = n, "read_line_nonblocking: read data");
                    self.read_buffer.extend_from_slice(&temp_buf[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tracing::trace!(
                        buffer_len = self.read_buffer.len(),
                        "read_line_nonblocking: would block"
                    );
                    return Ok(None);
                }
                Err(e) => {
                    tracing::debug!(error = %e, "read_line_nonblocking: read error");
                    return Err(ClientError::Connection(e));
                }
            }
        }
    }

    fn send_request(&mut self, request: DaemonRequest) -> Result<DaemonResponse> {
        let json = request
            .to_json()
            .map_err(|e| ClientError::Protocol(e.to_string()))?;

        writeln!(self.stream, "{}", json)?;
        self.stream.flush()?;

        let line = self.read_line_blocking()?;
        tracing::debug!(line_len = line.len(), "send_request read response");

        DaemonResponse::from_json(&line).map_err(|e| ClientError::Protocol(e.to_string()))
    }

    pub fn get_status(&mut self) -> Result<DaemonStatus> {
        match self.send_request(DaemonRequest::GetStatus)? {
            DaemonResponse::Status(status) => Ok(status),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn get_hourly_stats(&mut self, from: i64, to: i64) -> Result<Vec<HourlyStat>> {
        match self.send_request(DaemonRequest::GetHourlyStats { from, to })? {
            DaemonResponse::HourlyStats(stats) => Ok(stats),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn get_daily_stats(&mut self, from: &str, to: &str) -> Result<Vec<DailyStat>> {
        match self.send_request(DaemonRequest::GetDailyStats {
            from: from.to_string(),
            to: to.to_string(),
        })? {
            DaemonResponse::DailyStats(stats) => Ok(stats),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn get_top_processes_range(
        &mut self,
        from: &str,
        to: &str,
        limit: usize,
    ) -> Result<Vec<DailyTopProcess>> {
        match self.send_request(DaemonRequest::GetTopProcessesRange {
            from: from.to_string(),
            to: to.to_string(),
            limit,
        })? {
            DaemonResponse::TopProcesses(processes) => Ok(processes),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn shutdown(&mut self) -> Result<()> {
        match self.send_request(DaemonRequest::Shutdown)? {
            DaemonResponse::Ok => Ok(()),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn get_recent_samples(&mut self, window_secs: u64) -> Result<Vec<Sample>> {
        match self.send_request(DaemonRequest::GetRecentSamples { window_secs })? {
            DaemonResponse::RecentSamples(samples) => Ok(samples),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn kill_process(&mut self, pid: u32, signal: KillSignal) -> Result<KillProcessResult> {
        match self.send_request(DaemonRequest::KillProcess { pid, signal })? {
            DaemonResponse::KillResult(result) => Ok(result),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn subscribe(&mut self) -> Result<()> {
        match self.send_request(DaemonRequest::Subscribe)? {
            DaemonResponse::Subscribed => Ok(()),
            DaemonResponse::SubscriptionRejected { reason } => {
                Err(ClientError::SubscriptionRejected(reason))
            }
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn set_broadcast_interval(&mut self, interval_ms: u64) -> Result<()> {
        match self.send_request(DaemonRequest::SetBroadcastInterval { interval_ms })? {
            DaemonResponse::Ok => Ok(()),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn unsubscribe(&mut self) -> Result<()> {
        match self.send_request(DaemonRequest::Unsubscribe)? {
            DaemonResponse::Unsubscribed => Ok(()),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn get_cycle_summary(&mut self, days: u32) -> Result<CycleSummary> {
        match self.send_request(DaemonRequest::GetCycleSummary { days })? {
            DaemonResponse::CycleSummary(summary) => Ok(summary),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn get_charge_sessions(&mut self, from: i64, to: i64) -> Result<Vec<ChargeSession>> {
        match self.send_request(DaemonRequest::GetChargeSessions { from, to })? {
            DaemonResponse::ChargeSessions(sessions) => Ok(sessions),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn get_daily_cycles(&mut self, from: &str, to: &str) -> Result<Vec<DailyCycle>> {
        match self.send_request(DaemonRequest::GetDailyCycles {
            from: from.to_string(),
            to: to.to_string(),
        })? {
            DaemonResponse::DailyCycles(cycles) => Ok(cycles),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    pub fn read_update(&mut self) -> Result<Option<DataSnapshot>> {
        let mut latest_snapshot: Option<DataSnapshot> = None;
        let mut messages_read = 0;

        loop {
            match self.read_line_nonblocking()? {
                None => break,
                Some(line) => {
                    messages_read += 1;
                    let response = match DaemonResponse::from_json(&line) {
                        Ok(r) => r,
                        Err(e) => {
                            let start: String = line.chars().take(50).collect();
                            let end: String = line
                                .chars()
                                .rev()
                                .take(50)
                                .collect::<String>()
                                .chars()
                                .rev()
                                .collect();
                            tracing::error!(
                                error = %e,
                                line_len = line.len(),
                                start = %start,
                                end = %end,
                                "JSON parse failed"
                            );
                            return Err(ClientError::Protocol(e.to_string()));
                        }
                    };
                    match response {
                        DaemonResponse::DataUpdate(snapshot) => {
                            latest_snapshot = Some(snapshot);
                        }
                        DaemonResponse::Error(e) => return Err(ClientError::Daemon(e)),
                        _ => {}
                    }
                }
            }
        }

        if messages_read > 0 {
            tracing::debug!(
                messages_read,
                has_snapshot = latest_snapshot.is_some(),
                "read_update: drained buffer"
            );
        }

        Ok(latest_snapshot)
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> Result<()> {
        self.stream.set_nonblocking(nonblocking)?;
        if nonblocking {
            self.stream.set_read_timeout(None)?;
            self.stream.set_write_timeout(None)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_status(
        protocol_version: u32,
        min_supported_version: u32,
        version: &str,
    ) -> DaemonStatus {
        DaemonStatus {
            running: true,
            uptime_secs: 0,
            sample_count: 0,
            last_sample_time: None,
            database_size_bytes: 0,
            version: version.to_string(),
            subscriber_count: 0,
            history_enabled: false,
            protocol_version,
            min_supported_version,
        }
    }

    #[test]
    fn test_version_compatible_same_version() {
        let status = make_status(PROTOCOL_VERSION, MIN_SUPPORTED_VERSION, "1.0.0");
        assert!(check_version_compatibility(&status).is_ok());
    }

    #[test]
    fn test_version_compatible_daemon_newer() {
        let status = make_status(PROTOCOL_VERSION + 1, MIN_SUPPORTED_VERSION, "2.0.0");
        assert!(check_version_compatibility(&status).is_ok());
    }

    #[test]
    fn test_version_compatible_at_min_boundary() {
        let status = make_status(MIN_SUPPORTED_VERSION, MIN_SUPPORTED_VERSION, "0.5.0");
        assert!(check_version_compatibility(&status).is_ok());
    }

    #[test]
    fn test_version_tui_too_old() {
        let status = make_status(10, PROTOCOL_VERSION + 1, "3.0.0");
        let result = check_version_compatibility(&status);
        assert!(result.is_err());
        if let Err(ClientError::VersionMismatch(e)) = result {
            assert_eq!(e.kind, VersionMismatchKind::TuiTooOld);
            assert_eq!(e.tui_protocol_version, PROTOCOL_VERSION);
            assert_eq!(e.daemon_min_supported, PROTOCOL_VERSION + 1);
            assert!(e.to_string().contains("update jolt"));
        } else {
            panic!("Expected VersionMismatch error");
        }
    }

    #[test]
    fn test_version_daemon_too_old() {
        let status = make_status(0, 0, "0.1.0");
        let result = check_version_compatibility(&status);
        assert!(result.is_err());
        if let Err(ClientError::VersionMismatch(e)) = result {
            assert_eq!(e.kind, VersionMismatchKind::DaemonTooOld);
            assert_eq!(e.daemon_protocol_version, 0);
            assert_eq!(e.tui_min_supported, MIN_SUPPORTED_VERSION);
            assert!(e.to_string().contains("restart the daemon"));
        } else {
            panic!("Expected VersionMismatch error");
        }
    }

    #[test]
    fn test_version_mismatch_error_display_tui_too_old() {
        let error = VersionMismatchError {
            tui_protocol_version: 1,
            tui_min_supported: 1,
            daemon_protocol_version: 3,
            daemon_min_supported: 2,
            daemon_binary_version: "2.0.0".to_string(),
            kind: VersionMismatchKind::TuiTooOld,
        };
        let msg = error.to_string();
        assert!(msg.contains("TUI uses protocol v1"));
        assert!(msg.contains("daemon (v2.0.0) requires v2+"));
        assert!(msg.contains("brew upgrade jolt"));
    }

    #[test]
    fn test_version_mismatch_error_display_daemon_too_old() {
        let error = VersionMismatchError {
            tui_protocol_version: 3,
            tui_min_supported: 2,
            daemon_protocol_version: 1,
            daemon_min_supported: 1,
            daemon_binary_version: "0.5.0".to_string(),
            kind: VersionMismatchKind::DaemonTooOld,
        };
        let msg = error.to_string();
        assert!(msg.contains("daemon (v0.5.0) uses protocol v1"));
        assert!(msg.contains("TUI requires v2+"));
        assert!(msg.contains("jolt daemon restart"));
    }
}
