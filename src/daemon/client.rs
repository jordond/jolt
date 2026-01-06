use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use crate::daemon::protocol::{DaemonRequest, DaemonResponse, DaemonStatus};
use crate::daemon::socket_path;
use crate::data::{DailyStat, DailyTopProcess, HourlyStat};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection failed: {0}")]
    Connection(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Daemon error: {0}")]
    Daemon(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;

pub struct DaemonClient {
    stream: UnixStream,
}

impl DaemonClient {
    pub fn connect() -> Result<Self> {
        let path = socket_path();
        let stream = UnixStream::connect(&path)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self { stream })
    }

    fn send_request(&mut self, request: DaemonRequest) -> Result<DaemonResponse> {
        let json = request
            .to_json()
            .map_err(|e| ClientError::Protocol(e.to_string()))?;

        writeln!(self.stream, "{}", json)?;
        self.stream.flush()?;

        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;

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
}
