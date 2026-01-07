use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::{runtime_dir, HistoryConfig, UserConfig};
use crate::daemon::protocol::{
    BatterySnapshot, BatteryState, DaemonRequest, DaemonResponse, DaemonStatus, DataSnapshot,
    KillProcessResult, PowerMode, PowerSnapshot, ProcessSnapshot, ProcessState, MAX_SUBSCRIBERS,
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
        }
    }

    fn create_snapshot(&self) -> DataSnapshot {
        DataSnapshot {
            timestamp: chrono::Utc::now().timestamp(),
            battery: self.create_battery_snapshot(),
            power: self.create_power_snapshot(),
            processes: self.create_process_snapshots(),
        }
    }

    fn create_battery_snapshot(&self) -> BatterySnapshot {
        let state = match self.battery.state_label() {
            "Charging" => BatteryState::Charging,
            "On Battery" => BatteryState::Discharging,
            "Full" => BatteryState::Full,
            "Not Charging" => BatteryState::NotCharging,
            _ => BatteryState::Unknown,
        };

        BatterySnapshot {
            charge_percent: self.battery.charge_percent(),
            state,
            state_label: self.battery.state_label().to_string(),
            health_percent: self.battery.health_percent(),
            max_capacity_wh: self.battery.max_capacity_wh(),
            design_capacity_wh: self.battery.design_capacity_wh(),
            cycle_count: self.battery.cycle_count(),
            time_remaining_mins: self.battery.time_remaining_minutes(),
            time_remaining_formatted: self.battery.time_remaining_formatted(),
            charging_watts: self.battery.charging_watts(),
            charger_watts: self.battery.charger_watts(),
            discharge_watts: self.battery.discharge_watts(),
            voltage_mv: 0,
            amperage_ma: 0,
            external_connected: false,
        }
    }

    fn create_power_snapshot(&self) -> PowerSnapshot {
        let mode = match self.power.power_mode() {
            crate::data::power::PowerMode::LowPower => PowerMode::LowPower,
            crate::data::power::PowerMode::Automatic => PowerMode::Automatic,
            crate::data::power::PowerMode::HighPerformance => PowerMode::HighPerformance,
            crate::data::power::PowerMode::Unknown => PowerMode::Unknown,
        };

        PowerSnapshot {
            cpu_power_watts: self.power.cpu_power_watts(),
            gpu_power_watts: self.power.gpu_power_watts(),
            total_power_watts: self.power.total_power_watts(),
            power_mode: mode,
            power_mode_label: self.power.power_mode_label().to_string(),
            is_warmed_up: self.power.is_warmed_up(),
        }
    }

    fn create_process_snapshots(&self) -> Vec<ProcessSnapshot> {
        self.processes
            .processes
            .iter()
            .map(|p| self.process_info_to_snapshot(p))
            .collect()
    }

    fn process_info_to_snapshot(&self, p: &crate::data::ProcessInfo) -> ProcessSnapshot {
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
            children: p.children.as_ref().map(|children| {
                children
                    .iter()
                    .map(|c| self.process_info_to_snapshot(c))
                    .collect()
            }),
            is_killable: p.is_killable,
            disk_read_bytes: p.disk_read_bytes,
            disk_write_bytes: p.disk_write_bytes,
            status,
            run_time_secs: p.run_time_secs,
            total_cpu_time_secs: p.total_cpu_time_secs,
        }
    }

    fn handle_request(&self, request: &DaemonRequest, subscriber_count: usize) -> DaemonResponse {
        match request {
            DaemonRequest::GetStatus => DaemonResponse::Status(self.get_status(subscriber_count)),
            DaemonRequest::GetHourlyStats { from, to } => {
                match self.recorder.store().get_hourly_stats(*from, *to) {
                    Ok(stats) => DaemonResponse::HourlyStats(stats),
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetDailyStats { from, to } => {
                match self.recorder.store().get_daily_stats(from, to) {
                    Ok(stats) => DaemonResponse::DailyStats(stats),
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetTopProcessesRange { from, to, limit } => {
                match self
                    .recorder
                    .store()
                    .get_top_processes_range(from, to, *limit)
                {
                    Ok(processes) => DaemonResponse::TopProcesses(processes),
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetRecentSamples { window_secs } => {
                let now = chrono::Utc::now().timestamp();
                let from = now - *window_secs as i64;
                match self.recorder.store().get_samples(from, now) {
                    Ok(samples) => DaemonResponse::RecentSamples(samples),
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::GetCurrentData => DaemonResponse::CurrentData(self.create_snapshot()),
            DaemonRequest::KillProcess { pid } => {
                match std::process::Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output()
                {
                    Ok(output) => DaemonResponse::KillResult(KillProcessResult {
                        pid: *pid,
                        success: output.status.success(),
                        error: if output.status.success() {
                            None
                        } else {
                            Some("kill command failed".to_string())
                        },
                    }),
                    Err(e) => DaemonResponse::KillResult(KillProcessResult {
                        pid: *pid,
                        success: false,
                        error: Some(e.to_string()),
                    }),
                }
            }
            DaemonRequest::Shutdown => DaemonResponse::Ok,
            DaemonRequest::Subscribe | DaemonRequest::Unsubscribe => {
                DaemonResponse::Error("Handled separately".to_string())
            }
        }
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
        let json = match response.to_json() {
            Ok(j) => j,
            Err(_) => continue,
        };
        if writer
            .write_all(format!("{}\n", json).as_bytes())
            .await
            .is_err()
        {
            break;
        }
    }
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
        let _guard =
            crate::logging::init(log_level, crate::logging::LogMode::File, log_level_override);
        std::mem::forget(_guard);
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
    let broadcast_interval = Duration::from_secs(1);

    debug!(
        sample_interval_secs = state.config.sample_interval_secs,
        "Running initial aggregation"
    );
    state.run_aggregation();

    let mut sample_tick = tokio::time::interval(sample_interval);
    let mut aggregation_tick = tokio::time::interval(aggregation_interval);
    let mut prune_tick = tokio::time::interval(prune_interval);
    let mut broadcast_tick = tokio::time::interval(broadcast_interval);

    sample_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    aggregation_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    prune_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    broadcast_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let (msg_tx, mut msg_rx) = mpsc::channel::<(ClientId, ClientMessage)>(256);
    let mut clients: HashMap<ClientId, ClientHandle> = HashMap::new();
    let mut next_client_id: ClientId = 1;
    let mut shutdown_requested = false;

    loop {
        tokio::select! {
            _ = sample_tick.tick() => {
                if let Err(e) = state.refresh() {
                    error!(error = %e, "Error refreshing data");
                }
            }
            _ = aggregation_tick.tick() => {
                if state.last_aggregation.elapsed() >= aggregation_interval {
                    state.run_aggregation();
                }
            }
            _ = prune_tick.tick() => {
                if state.last_prune.elapsed() >= prune_interval {
                    state.run_prune();
                }
            }
            _ = broadcast_tick.tick() => {
                let subscriber_count: usize = clients.values().filter(|c| c.is_subscriber).count();
                if subscriber_count > 0 {
                    let snapshot = state.create_snapshot();
                    let update = DaemonResponse::DataUpdate(snapshot);

                    let mut disconnected = Vec::new();
                    for (id, client) in &clients {
                        if client.is_subscriber
                            && client.response_tx.send(update.clone()).await.is_err()
                        {
                            disconnected.push(*id);
                        }
                    }
                    for id in disconnected {
                        clients.remove(&id);
                        debug!(client_id = id, "Removed disconnected subscriber");
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
    fs::remove_file(&socket).ok();

    Ok(())
}
