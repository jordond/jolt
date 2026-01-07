use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use crate::daemon::protocol::{
    DaemonRequest, DaemonResponse, DaemonStatus, DataSnapshot, KillProcessResult,
};
use crate::daemon::socket_path;
use crate::data::{DailyStat, DailyTopProcess, HourlyStat, Sample};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection failed: {0}")]
    Connection(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Daemon error: {0}")]
    Daemon(String),

    #[allow(dead_code)]
    #[error("Subscription rejected: {0}")]
    SubscriptionRejected(String),
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

    pub fn get_recent_samples(&mut self, window_secs: u64) -> Result<Vec<Sample>> {
        match self.send_request(DaemonRequest::GetRecentSamples { window_secs })? {
            DaemonResponse::RecentSamples(samples) => Ok(samples),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    #[allow(dead_code)]
    pub fn get_current_data(&mut self) -> Result<DataSnapshot> {
        match self.send_request(DaemonRequest::GetCurrentData)? {
            DaemonResponse::CurrentData(snapshot) => Ok(snapshot),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    #[allow(dead_code)]
    pub fn kill_process(&mut self, pid: u32) -> Result<KillProcessResult> {
        match self.send_request(DaemonRequest::KillProcess { pid })? {
            DaemonResponse::KillResult(result) => Ok(result),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn unsubscribe(&mut self) -> Result<()> {
        match self.send_request(DaemonRequest::Unsubscribe)? {
            DaemonResponse::Unsubscribed => Ok(()),
            DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
            _ => Err(ClientError::Protocol("Unexpected response".into())),
        }
    }

    #[allow(dead_code)]
    pub fn read_update(&mut self) -> Result<Option<DataSnapshot>> {
        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => Ok(None),
            Ok(_) => {
                let response = DaemonResponse::from_json(&line)
                    .map_err(|e| ClientError::Protocol(e.to_string()))?;
                match response {
                    DaemonResponse::DataUpdate(snapshot) => Ok(Some(snapshot)),
                    DaemonResponse::Error(e) => Err(ClientError::Daemon(e)),
                    _ => Ok(None),
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
            Err(e) => Err(ClientError::Connection(e)),
        }
    }

    #[allow(dead_code)]
    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<()> {
        self.stream.set_nonblocking(nonblocking)?;
        Ok(())
    }
}

#[allow(dead_code)]
pub mod async_client {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    use crate::daemon::protocol::{
        DaemonRequest, DaemonResponse, DaemonStatus, DataSnapshot, KillProcessResult,
    };
    use crate::daemon::socket_path;
    use crate::data::{DailyStat, DailyTopProcess, HourlyStat, Sample};

    #[derive(Debug, thiserror::Error)]
    pub enum AsyncClientError {
        #[error("Connection failed: {0}")]
        Connection(#[from] std::io::Error),

        #[error("Protocol error: {0}")]
        Protocol(String),

        #[error("Daemon error: {0}")]
        Daemon(String),

        #[error("Subscription rejected: {0}")]
        SubscriptionRejected(String),
    }

    pub type Result<T> = std::result::Result<T, AsyncClientError>;

    pub struct AsyncDaemonClient {
        reader: BufReader<tokio::net::unix::OwnedReadHalf>,
        writer: tokio::net::unix::OwnedWriteHalf,
    }

    impl AsyncDaemonClient {
        pub async fn connect() -> Result<Self> {
            let path = socket_path();
            let stream = UnixStream::connect(&path).await?;
            let (reader, writer) = stream.into_split();
            Ok(Self {
                reader: BufReader::new(reader),
                writer,
            })
        }

        async fn send_request(&mut self, request: DaemonRequest) -> Result<DaemonResponse> {
            let json = request
                .to_json()
                .map_err(|e| AsyncClientError::Protocol(e.to_string()))?;

            self.writer
                .write_all(format!("{}\n", json).as_bytes())
                .await?;

            let mut line = String::new();
            self.reader.read_line(&mut line).await?;

            DaemonResponse::from_json(&line).map_err(|e| AsyncClientError::Protocol(e.to_string()))
        }

        pub async fn get_status(&mut self) -> Result<DaemonStatus> {
            match self.send_request(DaemonRequest::GetStatus).await? {
                DaemonResponse::Status(status) => Ok(status),
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn get_hourly_stats(&mut self, from: i64, to: i64) -> Result<Vec<HourlyStat>> {
            match self
                .send_request(DaemonRequest::GetHourlyStats { from, to })
                .await?
            {
                DaemonResponse::HourlyStats(stats) => Ok(stats),
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn get_daily_stats(&mut self, from: &str, to: &str) -> Result<Vec<DailyStat>> {
            match self
                .send_request(DaemonRequest::GetDailyStats {
                    from: from.to_string(),
                    to: to.to_string(),
                })
                .await?
            {
                DaemonResponse::DailyStats(stats) => Ok(stats),
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn get_top_processes_range(
            &mut self,
            from: &str,
            to: &str,
            limit: usize,
        ) -> Result<Vec<DailyTopProcess>> {
            match self
                .send_request(DaemonRequest::GetTopProcessesRange {
                    from: from.to_string(),
                    to: to.to_string(),
                    limit,
                })
                .await?
            {
                DaemonResponse::TopProcesses(processes) => Ok(processes),
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn shutdown(&mut self) -> Result<()> {
            match self.send_request(DaemonRequest::Shutdown).await? {
                DaemonResponse::Ok => Ok(()),
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn get_recent_samples(&mut self, window_secs: u64) -> Result<Vec<Sample>> {
            match self
                .send_request(DaemonRequest::GetRecentSamples { window_secs })
                .await?
            {
                DaemonResponse::RecentSamples(samples) => Ok(samples),
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn get_current_data(&mut self) -> Result<DataSnapshot> {
            match self.send_request(DaemonRequest::GetCurrentData).await? {
                DaemonResponse::CurrentData(snapshot) => Ok(snapshot),
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn kill_process(&mut self, pid: u32) -> Result<KillProcessResult> {
            match self
                .send_request(DaemonRequest::KillProcess { pid })
                .await?
            {
                DaemonResponse::KillResult(result) => Ok(result),
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn subscribe(&mut self) -> Result<()> {
            match self.send_request(DaemonRequest::Subscribe).await? {
                DaemonResponse::Subscribed => Ok(()),
                DaemonResponse::SubscriptionRejected { reason } => {
                    Err(AsyncClientError::SubscriptionRejected(reason))
                }
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn unsubscribe(&mut self) -> Result<()> {
            match self.send_request(DaemonRequest::Unsubscribe).await? {
                DaemonResponse::Unsubscribed => Ok(()),
                DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                _ => Err(AsyncClientError::Protocol("Unexpected response".into())),
            }
        }

        pub async fn read_update(&mut self) -> Result<Option<DataSnapshot>> {
            let mut line = String::new();
            match self.reader.read_line(&mut line).await {
                Ok(0) => Ok(None),
                Ok(_) => {
                    let response = DaemonResponse::from_json(&line)
                        .map_err(|e| AsyncClientError::Protocol(e.to_string()))?;
                    match response {
                        DaemonResponse::DataUpdate(snapshot) => Ok(Some(snapshot)),
                        DaemonResponse::Error(e) => Err(AsyncClientError::Daemon(e)),
                        _ => Ok(None),
                    }
                }
                Err(e) => Err(AsyncClientError::Connection(e)),
            }
        }
    }
}
