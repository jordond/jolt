use std::collections::HashMap;
use std::fs;
use std::sync::mpsc as std_mpsc;
use std::thread;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

use crate::config::{runtime_dir, HistoryConfig, UserConfig};
use crate::daemon::protocol::{
    BatterySnapshot, BatteryState, ChargeSession, DaemonRequest, DaemonResponse, DaemonStatus,
    DailyCycle, DailyStat, DailyTopProcess, DataSnapshot, HourlyStat, KillProcessResult, PowerMode,
    PowerSnapshot, ProcessSnapshot, ProcessState, Sample, MAX_SUBSCRIBERS, MIN_SUPPORTED_VERSION,
    PROTOCOL_VERSION,
};
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

type ClientId = u64;

enum ClientMessage {
    Request { request: DaemonRequest },
    Disconnect,
}

struct ClientHandle {
    response_tx: mpsc::Sender<DaemonResponse>,
    is_subscriber: bool,
}

const PROCESS_REFRESH_INTERVAL: Duration = Duration::from_secs(3);

enum RefreshRequest {
    Full,
    MetricsOnly,
    Shutdown,
}

struct RefreshWorker {
    request_tx: std_mpsc::Sender<RefreshRequest>,
    response_rx: std_mpsc::Receiver<DataSnapshot>,
    _handle: thread::JoinHandle<()>,
}

impl RefreshWorker {
    fn new(user_config: &UserConfig) -> Result<Self> {
        let (request_tx, request_rx) = std_mpsc::channel::<RefreshRequest>();
        let (response_tx, response_rx) = std_mpsc::channel::<DataSnapshot>();

        let excluded: Vec<String> = user_config
            .effective_excluded_processes()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let config = user_config.history.clone();
        let excluded_clone = excluded.clone();

        let handle = thread::spawn(move || {
            Self::worker_loop(request_rx, response_tx, config, excluded_clone);
        });

        Ok(Self {
            request_tx,
            response_rx,
            _handle: handle,
        })
    }

    fn worker_loop(
        request_rx: std_mpsc::Receiver<RefreshRequest>,
        response_tx: std_mpsc::Sender<DataSnapshot>,
        config: HistoryConfig,
        excluded: Vec<String>,
    ) {
        let mut battery = match BatteryData::new() {
            Ok(b) => b,
            Err(e) => {
                error!(error = %e, "Failed to initialize battery data in worker");
                return;
            }
        };
        let mut power = match PowerData::new() {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "Failed to initialize power data in worker");
                return;
            }
        };
        let mut processes = match ProcessData::with_exclusions(excluded.clone()) {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "Failed to initialize process data in worker");
                return;
            }
        };

        let mut recorder = match Recorder::new(config, excluded) {
            Ok(r) => Some(r),
            Err(e) => {
                warn!(error = %e, "Failed to initialize recorder, history disabled");
                None
            }
        };

        let mut last_process_refresh = Instant::now();

        while let Ok(request) = request_rx.recv() {
            let refresh_start = Instant::now();
            let request_type = match &request {
                RefreshRequest::Shutdown => "shutdown",
                RefreshRequest::Full => "full",
                RefreshRequest::MetricsOnly => "metrics_only",
            };

            match request {
                RefreshRequest::Shutdown => break,
                RefreshRequest::Full => {
                    let _ = battery.refresh();
                    let _ = power.refresh();
                    let _ = processes.refresh();
                    last_process_refresh = Instant::now();

                    if let Some(ref mut rec) = recorder {
                        if let Err(e) = rec.record_all(&battery, &power, &processes) {
                            warn!(error = %e, "Failed to record data");
                        }
                    }
                }
                RefreshRequest::MetricsOnly => {
                    let _ = battery.refresh();
                    let _ = power.refresh();
                    let process_refresh_due =
                        last_process_refresh.elapsed() >= PROCESS_REFRESH_INTERVAL;
                    if process_refresh_due {
                        let _ = processes.refresh();
                        last_process_refresh = Instant::now();
                    }
                    trace!(
                        process_refresh_due,
                        since_last_process_ms = last_process_refresh.elapsed().as_millis() as u64,
                        "MetricsOnly refresh"
                    );
                }
            }

            let snapshot = create_snapshot(&battery, &power, &processes);
            let refresh_duration = refresh_start.elapsed();
            debug!(
                request_type,
                refresh_ms = refresh_duration.as_millis() as u64,
                battery_percent = snapshot.battery.charge_percent,
                battery_state = %snapshot.battery.state_label,
                external_connected = snapshot.battery.external_connected,
                power_watts = snapshot.power.total_power_watts,
                "Worker completed refresh"
            );
            if response_tx.send(snapshot).is_err() {
                break;
            }
        }
    }

    fn request_refresh(&self, full: bool) {
        let req = if full {
            RefreshRequest::Full
        } else {
            RefreshRequest::MetricsOnly
        };
        let _ = self.request_tx.send(req);
    }

    fn try_recv_snapshot(&self) -> Option<DataSnapshot> {
        self.response_rx.try_recv().ok()
    }

    fn shutdown(&self) {
        let _ = self.request_tx.send(RefreshRequest::Shutdown);
    }
}

fn create_snapshot(
    battery: &BatteryData,
    power: &PowerData,
    processes: &ProcessData,
) -> DataSnapshot {
    let battery_state = match battery.state_label() {
        "Charging" => BatteryState::Charging,
        "On Battery" => BatteryState::Discharging,
        "Full" => BatteryState::Full,
        "Not Charging" => BatteryState::NotCharging,
        _ => BatteryState::Unknown,
    };

    let power_mode = match power.power_mode() {
        crate::data::power::PowerMode::LowPower => PowerMode::LowPower,
        crate::data::power::PowerMode::Automatic => PowerMode::Automatic,
        crate::data::power::PowerMode::HighPerformance => PowerMode::HighPerformance,
        crate::data::power::PowerMode::Unknown => PowerMode::Unknown,
    };

    let battery_snapshot = BatterySnapshot {
        charge_percent: battery.charge_percent(),
        state: battery_state,
        state_label: battery.state_label().to_string(),
        health_percent: battery.health_percent(),
        max_capacity_wh: battery.max_capacity_wh(),
        design_capacity_wh: battery.design_capacity_wh(),
        cycle_count: battery.cycle_count(),
        time_remaining_mins: battery.time_remaining_minutes(),
        time_remaining_formatted: battery.time_remaining_formatted(),
        charging_watts: battery.charging_watts(),
        charger_watts: battery.charger_watts(),
        discharge_watts: battery.discharge_watts(),
        voltage_mv: battery.voltage_mv(),
        amperage_ma: battery.amperage_ma(),
        external_connected: battery.external_connected(),
        temperature_c: battery.temperature_c(),
        daily_min_soc: battery.daily_min_soc(),
        daily_max_soc: battery.daily_max_soc(),
    };

    let power_snapshot = PowerSnapshot {
        cpu_power_watts: power.cpu_power_watts(),
        gpu_power_watts: power.gpu_power_watts(),
        total_power_watts: power.total_power_watts(),
        power_mode,
        power_mode_label: power.power_mode_label().to_string(),
        is_warmed_up: power.is_warmed_up(),
    };

    let process_snapshots: Vec<ProcessSnapshot> = processes
        .processes
        .iter()
        .map(process_to_snapshot)
        .collect();

    DataSnapshot {
        timestamp: chrono::Utc::now().timestamp(),
        battery: battery_snapshot,
        power: power_snapshot,
        processes: process_snapshots,
    }
}

fn process_to_snapshot(p: &crate::data::ProcessInfo) -> ProcessSnapshot {
    let status = match p.status {
        crate::data::ProcessState::Running => ProcessState::Running,
        crate::data::ProcessState::Sleeping => ProcessState::Sleeping,
        crate::data::ProcessState::Idle => ProcessState::Idle,
        crate::data::ProcessState::Stopped => ProcessState::Stopped,
        crate::data::ProcessState::Zombie => ProcessState::Zombie,
        crate::data::ProcessState::Unknown => ProcessState::Unknown,
    };

    ProcessSnapshot {
        pid: p.pid,
        name: p.name.clone(),
        command: p.command.clone(),
        cpu_usage: p.cpu_usage,
        memory_mb: p.memory_mb,
        energy_impact: p.energy_impact,
        parent_pid: p.parent_pid,
        children: p
            .children
            .as_ref()
            .map(|children| children.iter().map(process_to_snapshot).collect()),
        is_killable: p.is_killable,
        disk_read_bytes: p.disk_read_bytes,
        disk_write_bytes: p.disk_write_bytes,
        status,
        run_time_secs: p.run_time_secs,
        total_cpu_time_secs: p.total_cpu_time_secs,
    }
}

struct DaemonState {
    worker: RefreshWorker,
    recorder: Recorder,
    start_time: Instant,
    config: HistoryConfig,
    last_snapshot: Option<DataSnapshot>,
}

impl DaemonState {
    fn new(user_config: &UserConfig) -> Result<Self> {
        let excluded: Vec<String> = user_config
            .effective_excluded_processes()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let worker = RefreshWorker::new(user_config)?;

        Ok(Self {
            worker,
            recorder: Recorder::new(user_config.history.clone(), excluded)?,
            start_time: Instant::now(),
            config: user_config.history.clone(),
            last_snapshot: None,
        })
    }

    fn request_refresh(&self, full: bool) {
        self.worker.request_refresh(full);
    }

    fn poll_snapshot(&mut self) -> Option<DataSnapshot> {
        if let Some(snapshot) = self.worker.try_recv_snapshot() {
            self.last_snapshot = Some(snapshot.clone());
            Some(snapshot)
        } else {
            None
        }
    }

    fn current_snapshot(&self) -> Option<&DataSnapshot> {
        self.last_snapshot.as_ref()
    }

    fn shutdown_worker(&self) {
        self.worker.shutdown();
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
    }

    fn get_status(&self, subscriber_count: usize) -> DaemonStatus {
        let stats = self.recorder.store().get_stats().ok();

        DaemonStatus {
            running: true,
            uptime_secs: self.start_time.elapsed().as_secs(),
            sample_count: stats.as_ref().map(|s| s.sample_count).unwrap_or(0),
            last_sample_time: stats.and_then(|s| s.newest_sample),
            database_size_bytes: self.recorder.store().size_bytes().unwrap_or(0),
            version: env!("CARGO_PKG_VERSION").to_string(),
            subscriber_count,
            history_enabled: self.config.background_recording,
            protocol_version: PROTOCOL_VERSION,
            min_supported_version: MIN_SUPPORTED_VERSION,
        }
    }

    fn handle_request(&self, request: &DaemonRequest, subscriber_count: usize) -> DaemonResponse {
        match request {
            DaemonRequest::GetStatus => DaemonResponse::Status(self.get_status(subscriber_count)),
            DaemonRequest::GetHourlyStats { from, to } => {
                match self.recorder.store().get_hourly_stats(*from, *to) {
                    Ok(stats) => {
                        let converted: Vec<HourlyStat> = stats.iter().map(Into::into).collect();
                        DaemonResponse::HourlyStats(converted)
                    }
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetDailyStats { from, to } => {
                match self.recorder.store().get_daily_stats(from, to) {
                    Ok(stats) => {
                        let converted: Vec<DailyStat> = stats.iter().map(Into::into).collect();
                        DaemonResponse::DailyStats(converted)
                    }
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetTopProcessesRange { from, to, limit } => {
                match self
                    .recorder
                    .store()
                    .get_top_processes_range(from, to, *limit)
                {
                    Ok(processes) => {
                        let converted: Vec<DailyTopProcess> =
                            processes.iter().map(Into::into).collect();
                        DaemonResponse::TopProcesses(converted)
                    }
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetRecentSamples { window_secs } => {
                let now = chrono::Utc::now().timestamp();
                let from = now - *window_secs as i64;
                match self.recorder.store().get_samples(from, now) {
                    Ok(samples) => {
                        let converted: Vec<Sample> = samples.iter().map(Into::into).collect();
                        DaemonResponse::RecentSamples(converted)
                    }
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetCurrentData => match self.current_snapshot() {
                Some(snapshot) => DaemonResponse::CurrentData(snapshot.clone()),
                None => DaemonResponse::Error("No data available yet".to_string()),
            },
            DaemonRequest::KillProcess { pid, signal } => {
                match std::process::Command::new("kill")
                    .args([signal.as_arg(), &pid.to_string()])
                    .output()
                {
                    Ok(output) => {
                        let success = output.status.success();
                        let error = if success {
                            None
                        } else {
                            let mut msg = String::from("kill command failed");
                            if let Some(code) = output.status.code() {
                                msg = format!("{msg} with exit code {code}");
                            }
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stderr = stderr.trim();
                            if !stderr.is_empty() {
                                msg = format!("{msg}: {stderr}");
                            }
                            Some(msg)
                        };
                        DaemonResponse::KillResult(KillProcessResult {
                            pid: *pid,
                            success,
                            error,
                        })
                    }
                    Err(e) => DaemonResponse::KillResult(KillProcessResult {
                        pid: *pid,
                        success: false,
                        error: Some(e.to_string()),
                    }),
                }
            }
            DaemonRequest::Shutdown => DaemonResponse::Ok,
            DaemonRequest::Subscribe
            | DaemonRequest::Unsubscribe
            | DaemonRequest::SetBroadcastInterval { .. } => {
                DaemonResponse::Error("Handled separately".to_string())
            }
            DaemonRequest::GetCycleSummary { days } => match self.compute_cycle_summary(*days) {
                Ok(summary) => DaemonResponse::CycleSummary(summary),
                Err(e) => DaemonResponse::Error(e.to_string()),
            },
            DaemonRequest::GetChargeSessions { from, to } => {
                match self.recorder.store().get_charge_sessions(*from, *to, None) {
                    Ok(sessions) => {
                        let converted: Vec<ChargeSession> =
                            sessions.iter().map(Into::into).collect();
                        DaemonResponse::ChargeSessions(converted)
                    }
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetDailyCycles { from, to } => {
                match self.recorder.store().get_daily_cycles(from, to) {
                    Ok(cycles) => {
                        let converted: Vec<DailyCycle> = cycles.iter().map(Into::into).collect();
                        DaemonResponse::DailyCycles(converted)
                    }
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
        }
    }

    fn compute_cycle_summary(
        &self,
        days: u32,
    ) -> std::result::Result<crate::daemon::protocol::CycleSummary, crate::data::HistoryStoreError>
    {
        use crate::daemon::protocol::CycleSummary;

        let now = chrono::Utc::now();
        let from_date = (now - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string();
        let to_date = now.format("%Y-%m-%d").to_string();

        let daily_cycles = self
            .recorder
            .store()
            .get_daily_cycles(&from_date, &to_date)?;

        if daily_cycles.is_empty() {
            return Ok(CycleSummary::default());
        }

        let days_count = daily_cycles.len() as f32;

        let total_charge_sessions: i32 = daily_cycles.iter().map(|c| c.charge_sessions).sum();
        let total_partial_cycles: f32 = daily_cycles.iter().map(|c| c.partial_cycles).sum();

        let deepest_discharges: Vec<f32> = daily_cycles
            .iter()
            .filter_map(|c| c.deepest_discharge_percent)
            .collect();

        let avg_depth_of_discharge = if !deepest_discharges.is_empty() {
            100.0 - (deepest_discharges.iter().sum::<f32>() / deepest_discharges.len() as f32)
        } else {
            0.0
        };

        let total_high_soc_mins: i32 = daily_cycles.iter().map(|c| c.time_at_high_soc_mins).sum();
        let total_active_mins = days_count * 24.0 * 60.0;
        let time_at_high_soc_percent = if total_active_mins > 0.0 {
            (total_high_soc_mins as f32 / total_active_mins) * 100.0
        } else {
            0.0
        };

        let macos_cycle_count = self
            .current_snapshot()
            .and_then(|s| s.battery.cycle_count)
            .unwrap_or(0);

        let estimated_remaining = if macos_cycle_count > 0 && macos_cycle_count < 1000 {
            Some(1000 - macos_cycle_count)
        } else {
            None
        };

        Ok(CycleSummary {
            total_cycles_macos: macos_cycle_count,
            partial_cycles_calculated: total_partial_cycles,
            avg_daily_cycles: total_partial_cycles / days_count,
            avg_depth_of_discharge,
            avg_charge_sessions_per_day: total_charge_sessions as f32 / days_count,
            time_at_high_soc_percent,
            estimated_cycles_remaining: estimated_remaining,
            days_analyzed: days_count as u32,
        })
    }
}

async fn client_reader_task(
    mut reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    msg_tx: mpsc::Sender<(ClientId, ClientMessage)>,
    client_id: ClientId,
) {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                let _ = msg_tx.send((client_id, ClientMessage::Disconnect)).await;
                break;
            }
            Ok(_) => match DaemonRequest::from_json(line.trim()) {
                Ok(request) => {
                    if msg_tx
                        .send((client_id, ClientMessage::Request { request }))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    warn!(client_id, error = %e, "Invalid request from client");
                }
            },
            Err(e) => {
                debug!(client_id, error = %e, "Client read error");
                let _ = msg_tx.send((client_id, ClientMessage::Disconnect)).await;
                break;
            }
        }
    }
}

async fn client_writer_task(
    mut writer: tokio::net::unix::OwnedWriteHalf,
    mut response_rx: mpsc::Receiver<DaemonResponse>,
) {
    while let Some(response) = response_rx.recv().await {
        let is_data_update = matches!(response, DaemonResponse::DataUpdate(_));
        let json = match response.to_json() {
            Ok(j) => j,
            Err(e) => {
                warn!(error = %e, "Failed to serialize response");
                continue;
            }
        };
        let json_len = json.len();
        if let Err(e) = writer.write_all(format!("{}\n", json).as_bytes()).await {
            debug!(error = %e, "Write failed, closing connection");
            break;
        }
        if let Err(e) = writer.flush().await {
            debug!(error = %e, "Flush failed, closing connection");
            break;
        }
        if is_data_update {
            trace!(json_len, "Sent DataUpdate to client");
        }
    }
    debug!("Client writer task ending");
}

pub fn run_daemon(
    foreground: bool,
    log_level: crate::config::LogLevel,
    log_level_override: Option<crate::config::LogLevel>,
) -> Result<()> {
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
        // Keep the logging guard alive for the daemon's entire lifetime.
        // mem::forget is intentional - the guard must not drop or logging stops.
        // The OS reclaims all resources when the daemon process exits.
        let guard =
            crate::logging::init(log_level, crate::logging::LogMode::File, log_level_override);
        std::mem::forget(guard);
    }

    info!(version = env!("CARGO_PKG_VERSION"), "Daemon starting");

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let local = tokio::task::LocalSet::new();
    local.block_on(&runtime, run_daemon_async(socket))
}

async fn run_daemon_async(socket: std::path::PathBuf) -> Result<()> {
    let user_config = UserConfig::load();
    let mut state = DaemonState::new(&user_config)?;

    let listener = UnixListener::bind(&socket)?;
    info!(socket = ?socket, "Listening for connections");

    let sample_interval = Duration::from_secs(state.config.sample_interval_secs);
    let aggregation_interval = Duration::from_secs(3600);
    let prune_interval = Duration::from_secs(86400);
    let mut broadcast_interval_ms = user_config.refresh_ms;

    debug!(
        sample_interval_secs = state.config.sample_interval_secs,
        broadcast_interval_ms, "Running initial aggregation"
    );
    state.run_aggregation();

    let mut sample_tick = tokio::time::interval(sample_interval);
    let mut aggregation_tick = tokio::time::interval(aggregation_interval);
    let mut prune_tick = tokio::time::interval(prune_interval);
    let mut broadcast_tick = tokio::time::interval(Duration::from_millis(broadcast_interval_ms));
    let mut poll_tick = tokio::time::interval(Duration::from_millis(50));

    sample_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    aggregation_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    prune_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    broadcast_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    poll_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let (msg_tx, mut msg_rx) = mpsc::channel::<(ClientId, ClientMessage)>(256);
    let mut clients: HashMap<ClientId, ClientHandle> = HashMap::new();
    let mut next_client_id: ClientId = 1;
    let mut shutdown_requested = false;
    let mut pending_broadcast = false;

    state.request_refresh(true);

    loop {
        tokio::select! {
            _ = sample_tick.tick() => {
                state.request_refresh(true);
            }
            _ = aggregation_tick.tick() => {
                state.run_aggregation();
            }
            _ = prune_tick.tick() => {
                state.run_prune();
            }
            _ = broadcast_tick.tick() => {
                let subscriber_count: usize = clients.values().filter(|c| c.is_subscriber).count();
                debug!(
                    subscriber_count,
                    broadcast_interval_ms,
                    "Broadcast tick fired"
                );
                if subscriber_count > 0 {
                    state.request_refresh(false);
                    pending_broadcast = true;
                }
            }
            _ = poll_tick.tick() => {
                if let Some(snapshot) = state.poll_snapshot() {
                    debug!(
                        battery_percent = snapshot.battery.charge_percent,
                        battery_state = %snapshot.battery.state_label,
                        external_connected = snapshot.battery.external_connected,
                        power_watts = snapshot.power.total_power_watts,
                        process_count = snapshot.processes.len(),
                        pending_broadcast,
                        "Daemon polled new snapshot"
                    );
                    if pending_broadcast {
                        pending_broadcast = false;
                        let update = DaemonResponse::DataUpdate(snapshot);

                        let mut sent_count = 0;
                        let mut disconnected = Vec::new();
                        for (id, client) in &clients {
                            if client.is_subscriber {
                                if client.response_tx.send(update.clone()).await.is_err() {
                                    disconnected.push(*id);
                                } else {
                                    sent_count += 1;
                                }
                            }
                        }
                        if sent_count > 0 {
                            debug!(sent_count, "Broadcast DataUpdate to subscribers");
                        }
                        for id in disconnected {
                            clients.remove(&id);
                            debug!(client_id = id, "Removed disconnected subscriber");
                        }
                    }
                }
            }
            result = listener.accept() => {
                match result {
                    Ok((stream, _)) => {
                        let client_id = next_client_id;
                        next_client_id += 1;
                        debug!(client_id, "Client connected");

                        let (reader, writer) = stream.into_split();
                        let (response_tx, response_rx) = mpsc::channel::<DaemonResponse>(64);

                        clients.insert(client_id, ClientHandle {
                            response_tx,
                            is_subscriber: false,
                        });

                        let msg_tx_clone = msg_tx.clone();
                        tokio::task::spawn_local(client_reader_task(
                            BufReader::new(reader),
                            msg_tx_clone,
                            client_id,
                        ));
                        tokio::task::spawn_local(client_writer_task(writer, response_rx));
                    }
                    Err(e) => {
                        error!(error = %e, "Socket accept error");
                    }
                }
            }
            Some((client_id, msg)) = msg_rx.recv() => {
                match msg {
                    ClientMessage::Disconnect => {
                        if clients.remove(&client_id).is_some() {
                            debug!(client_id, count = clients.len(), "Client disconnected");
                        }
                    }
                    ClientMessage::Request { request } => {
                        debug!(client_id, request = ?request, "Handling request");

                        let response = match &request {
                            DaemonRequest::Subscribe => {
                                let subscriber_count = clients.values().filter(|c| c.is_subscriber).count();
                                if subscriber_count >= MAX_SUBSCRIBERS {
                                    DaemonResponse::SubscriptionRejected {
                                        reason: format!("Maximum subscribers ({}) reached", MAX_SUBSCRIBERS),
                                    }
                                } else if let Some(client) = clients.get_mut(&client_id) {
                                    client.is_subscriber = true;
                                    info!(client_id, count = subscriber_count + 1, "Subscriber added");
                                    DaemonResponse::Subscribed
                                } else {
                                    DaemonResponse::Error("Client not found".to_string())
                                }
                            }
                            DaemonRequest::Unsubscribe => {
                                if let Some(client) = clients.get_mut(&client_id) {
                                    if client.is_subscriber {
                                        client.is_subscriber = false;
                                        let subscriber_count = clients.values().filter(|c| c.is_subscriber).count();
                                        info!(client_id, count = subscriber_count, "Subscriber removed");
                                    }
                                }
                                DaemonResponse::Unsubscribed
                            }
                            DaemonRequest::Shutdown => {
                                info!("Shutdown requested by client");
                                shutdown_requested = true;
                                DaemonResponse::Ok
                            }
                            DaemonRequest::SetBroadcastInterval { interval_ms } => {
                                let new_interval = (*interval_ms).max(100);
                                if new_interval != broadcast_interval_ms {
                                    broadcast_interval_ms = new_interval;
                                    broadcast_tick = tokio::time::interval(Duration::from_millis(broadcast_interval_ms));
                                    broadcast_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                                    info!(broadcast_interval_ms, "Broadcast interval updated");
                                }
                                DaemonResponse::Ok
                            }
                            _ => {
                                let subscriber_count = clients.values().filter(|c| c.is_subscriber).count();
                                state.handle_request(&request, subscriber_count)
                            }
                        };

                        if let Some(client) = clients.get(&client_id) {
                            let _ = client.response_tx.send(response).await;
                        }

                        if shutdown_requested {
                            break;
                        }
                    }
                }
            }
        }
    }

    info!("Daemon shutting down");
    state.shutdown_worker();
    fs::remove_file(&socket).ok();

    Ok(())
}
