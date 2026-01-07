use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tracing::{debug, error, info, warn};

use crate::config::{runtime_dir, HistoryConfig, UserConfig};
use crate::daemon::protocol::{DaemonRequest, DaemonResponse, DaemonStatus};
use crate::daemon::socket_path;
use crate::data::aggregator::Aggregator;
use crate::data::{BatteryData, PowerData, ProcessData, Recorder};

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] crate::data::HistoryStoreError),

    #[error("Already running")]
    AlreadyRunning,

    #[error("Failed to daemonize: {0}")]
    Daemonize(String),
}

pub type Result<T> = std::result::Result<T, DaemonError>;

struct DaemonState {
    battery: BatteryData,
    power: PowerData,
    processes: ProcessData,
    recorder: Recorder,
    start_time: Instant,
    config: HistoryConfig,
    last_aggregation: Instant,
    last_prune: Instant,
}

impl DaemonState {
    fn new(user_config: &UserConfig) -> Result<Self> {
        let excluded: Vec<String> = user_config
            .effective_excluded_processes()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let now = Instant::now();
        Ok(Self {
            battery: BatteryData::new()
                .map_err(|e| DaemonError::Io(std::io::Error::other(e.to_string())))?,
            power: PowerData::new()
                .map_err(|e| DaemonError::Io(std::io::Error::other(e.to_string())))?,
            processes: ProcessData::with_exclusions(excluded.clone())
                .map_err(|e| DaemonError::Io(std::io::Error::other(e.to_string())))?,
            recorder: Recorder::new(user_config.history.clone(), excluded)?,
            start_time: now,
            config: user_config.history.clone(),
            last_aggregation: now,
            last_prune: now,
        })
    }

    fn refresh(&mut self) -> Result<()> {
        let _ = self.battery.refresh();
        let _ = self.power.refresh();
        let _ = self.processes.refresh();

        self.recorder
            .record_all(&self.battery, &self.power, &self.processes)?;

        Ok(())
    }

    fn run_aggregation(&mut self) {
        let aggregator = Aggregator::new(self.recorder.store(), &self.config);

        match aggregator.aggregate_completed_hours() {
            Ok(count) if count > 0 => {
                info!(count, "Aggregated hourly stats");
            }
            Err(e) => {
                error!(error = %e, "Error aggregating hourly stats");
            }
            _ => {}
        }

        match aggregator.aggregate_completed_days() {
            Ok(count) if count > 0 => {
                info!(count, "Aggregated daily stats");
            }
            Err(e) => {
                error!(error = %e, "Error aggregating daily stats");
            }
            _ => {}
        }

        self.last_aggregation = Instant::now();
    }

    fn run_prune(&mut self) {
        let aggregator = Aggregator::new(self.recorder.store(), &self.config);

        match aggregator.prune_old_data() {
            Ok(result) => {
                let total = result.samples_deleted
                    + result.hourly_deleted
                    + result.daily_deleted
                    + result.processes_deleted;
                if total > 0 {
                    info!(
                        total,
                        samples = result.samples_deleted,
                        hourly = result.hourly_deleted,
                        daily = result.daily_deleted,
                        processes = result.processes_deleted,
                        "Pruned old records"
                    );
                }
            }
            Err(e) => {
                error!(error = %e, "Error pruning data");
            }
        }

        self.last_prune = Instant::now();
    }

    fn get_status(&self) -> DaemonStatus {
        let stats = self.recorder.store().get_stats().ok();

        DaemonStatus {
            running: true,
            uptime_secs: self.start_time.elapsed().as_secs(),
            sample_count: stats.as_ref().map(|s| s.sample_count).unwrap_or(0),
            last_sample_time: stats.and_then(|s| s.newest_sample),
            database_size_bytes: self.recorder.store().size_bytes().unwrap_or(0),
            version: env!("CARGO_PKG_VERSION").to_string(),
            subscriber_count: 0,
            history_enabled: self.config.enabled,
        }
    }

    fn handle_request(&self, request: DaemonRequest) -> DaemonResponse {
        match request {
            DaemonRequest::GetStatus => DaemonResponse::Status(self.get_status()),
            DaemonRequest::GetHourlyStats { from, to } => {
                match self.recorder.store().get_hourly_stats(from, to) {
                    Ok(stats) => DaemonResponse::HourlyStats(stats),
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetDailyStats { from, to } => {
                match self.recorder.store().get_daily_stats(&from, &to) {
                    Ok(stats) => DaemonResponse::DailyStats(stats),
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetTopProcessesRange { from, to, limit } => {
                match self
                    .recorder
                    .store()
                    .get_top_processes_range(&from, &to, limit)
                {
                    Ok(processes) => DaemonResponse::TopProcesses(processes),
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetRecentSamples { window_secs } => {
                let now = chrono::Utc::now().timestamp();
                let from = now - window_secs as i64;
                match self.recorder.store().get_samples(from, now) {
                    Ok(samples) => DaemonResponse::RecentSamples(samples),
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::Shutdown => DaemonResponse::Ok,
            DaemonRequest::Subscribe => {
                DaemonResponse::Error("Subscribe not implemented in sync server".to_string())
            }
            DaemonRequest::Unsubscribe => {
                DaemonResponse::Error("Unsubscribe not implemented in sync server".to_string())
            }
            DaemonRequest::GetCurrentData => {
                DaemonResponse::Error("GetCurrentData not implemented in sync server".to_string())
            }
            DaemonRequest::KillProcess { pid } => {
                match std::process::Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            DaemonResponse::KillResult(crate::daemon::protocol::KillProcessResult {
                                pid,
                                success: true,
                                error: None,
                            })
                        } else {
                            DaemonResponse::KillResult(crate::daemon::protocol::KillProcessResult {
                                pid,
                                success: false,
                                error: Some("kill command failed".to_string()),
                            })
                        }
                    }
                    Err(e) => {
                        DaemonResponse::KillResult(crate::daemon::protocol::KillProcessResult {
                            pid,
                            success: false,
                            error: Some(e.to_string()),
                        })
                    }
                }
            }
        }
    }
}

fn handle_client(stream: UnixStream, state: &DaemonState, shutdown: &AtomicBool) -> bool {
    let reader = BufReader::new(&stream);
    let mut writer = &stream;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                debug!(error = %e, "Client read error");
                break;
            }
        };

        let request = match DaemonRequest::from_json(&line) {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, "Invalid client request");
                let response = DaemonResponse::Error(format!("Invalid request: {}", e));
                let _ = writeln!(writer, "{}", response.to_json().unwrap_or_default());
                continue;
            }
        };

        debug!(request = ?request, "Handling client request");
        let is_shutdown = matches!(request, DaemonRequest::Shutdown);
        let response = state.handle_request(request);

        if writeln!(writer, "{}", response.to_json().unwrap_or_default()).is_err() {
            debug!("Client write error");
            break;
        }

        if is_shutdown {
            info!("Shutdown requested by client");
            shutdown.store(true, Ordering::SeqCst);
            return true;
        }
    }

    false
}

pub fn run_daemon(foreground: bool) -> Result<()> {
    let socket = socket_path();

    if socket.exists() {
        if crate::daemon::is_daemon_running() {
            return Err(DaemonError::AlreadyRunning);
        }
        fs::remove_file(&socket)?;
    }

    fs::create_dir_all(runtime_dir())?;

    if !foreground {
        match daemonize::Daemonize::new()
            .working_directory(runtime_dir())
            .start()
        {
            Ok(_) => {}
            Err(e) => return Err(DaemonError::Daemonize(e.to_string())),
        }
    }

    info!(version = env!("CARGO_PKG_VERSION"), "Daemon starting");

    let user_config = UserConfig::load();
    let mut state = DaemonState::new(&user_config)?;
    let shutdown = Arc::new(AtomicBool::new(false));

    let listener = UnixListener::bind(&socket)?;
    listener.set_nonblocking(true)?;

    info!(socket = ?socket, "Listening for connections");

    let sample_interval = Duration::from_secs(state.config.sample_interval_secs);
    let aggregation_interval = Duration::from_secs(3600);
    let prune_interval = Duration::from_secs(86400);
    let mut last_sample = Instant::now();

    debug!(
        sample_interval_secs = state.config.sample_interval_secs,
        "Running initial aggregation"
    );
    state.run_aggregation();

    while !shutdown.load(Ordering::SeqCst) {
        if last_sample.elapsed() >= sample_interval {
            if let Err(e) = state.refresh() {
                error!(error = %e, "Error refreshing data");
            }
            last_sample = Instant::now();
        }

        if state.last_aggregation.elapsed() >= aggregation_interval {
            state.run_aggregation();
        }

        if state.last_prune.elapsed() >= prune_interval {
            state.run_prune();
        }

        match listener.accept() {
            Ok((stream, _)) => {
                debug!("Client connected");
                stream.set_nonblocking(false).ok();
                stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
                stream.set_write_timeout(Some(Duration::from_secs(30))).ok();

                if handle_client(stream, &state, &shutdown) {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                error!(error = %e, "Socket accept error");
            }
        }
    }

    info!("Daemon shutting down");
    fs::remove_file(&socket).ok();

    Ok(())
}
