use std::fs;
use std::path::{Path, PathBuf};

use protocol::*;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures")
}

fn requests_dir() -> PathBuf {
    fixtures_dir().join("requests")
}

fn responses_dir() -> PathBuf {
    fixtures_dir().join("responses")
}

fn write_fixture(dir: &Path, name: &str, json: &str) {
    let path = dir.join(format!("{}.json", name));
    fs::write(&path, json).unwrap_or_else(|_| panic!("Failed to write fixture: {:?}", path));
}

fn sample_battery_snapshot() -> BatterySnapshot {
    BatterySnapshot {
        charge_percent: 85.5,
        state: BatteryState::Discharging,
        state_label: "On Battery".to_string(),
        health_percent: 92.0,
        max_capacity_wh: 52.6,
        design_capacity_wh: 58.0,
        cycle_count: Some(245),
        time_remaining_mins: Some(180),
        time_remaining_formatted: Some("3:00".to_string()),
        charging_watts: None,
        charger_watts: None,
        discharge_watts: Some(12.5),
        voltage_mv: 11500,
        amperage_ma: -1087,
        external_connected: false,
        temperature_c: Some(32.5),
        daily_min_soc: Some(25.0),
        daily_max_soc: Some(95.0),
    }
}

fn sample_power_snapshot() -> PowerSnapshot {
    PowerSnapshot {
        cpu_power_watts: 8.5,
        gpu_power_watts: 2.3,
        total_power_watts: 12.8,
        power_mode: PowerMode::Automatic,
        power_mode_label: "Automatic".to_string(),
        is_warmed_up: true,
    }
}

fn sample_process_snapshot() -> ProcessSnapshot {
    ProcessSnapshot {
        pid: 1234,
        name: "Safari".to_string(),
        command: "/Applications/Safari.app/Contents/MacOS/Safari".to_string(),
        cpu_usage: 15.5,
        memory_mb: 256.0,
        energy_impact: 25.0,
        parent_pid: Some(1),
        children: Some(vec![ProcessSnapshot {
            pid: 1235,
            name: "Safari Web Content".to_string(),
            command: "Safari Web Content".to_string(),
            cpu_usage: 5.0,
            memory_mb: 128.0,
            energy_impact: 10.0,
            parent_pid: Some(1234),
            children: None,
            is_killable: false,
            disk_read_bytes: 1000,
            disk_write_bytes: 500,
            status: ProcessState::Running,
            run_time_secs: 3600,
            total_cpu_time_secs: 120,
        }]),
        is_killable: true,
        disk_read_bytes: 10000,
        disk_write_bytes: 5000,
        status: ProcessState::Running,
        run_time_secs: 7200,
        total_cpu_time_secs: 600,
    }
}

fn sample_data_snapshot() -> DataSnapshot {
    DataSnapshot {
        timestamp: 1704067200,
        battery: sample_battery_snapshot(),
        power: sample_power_snapshot(),
        processes: vec![sample_process_snapshot()],
    }
}

fn sample_daemon_status() -> DaemonStatus {
    DaemonStatus {
        running: true,
        uptime_secs: 3600,
        sample_count: 1000,
        last_sample_time: Some(1704067200),
        database_size_bytes: 1024000,
        version: "0.1.0".to_string(),
        subscriber_count: 2,
        history_enabled: true,
        protocol_version: PROTOCOL_VERSION,
        min_supported_version: MIN_SUPPORTED_VERSION,
    }
}

fn sample_hourly_stat() -> HourlyStat {
    HourlyStat {
        hour_start: 1704067200,
        avg_power: 12.5,
        max_power: 25.0,
        min_power: 5.0,
        avg_battery: 75.0,
        battery_delta: -5.0,
        total_samples: 60,
    }
}

fn sample_daily_stat() -> DailyStat {
    DailyStat {
        date: "2024-01-01".to_string(),
        avg_power: 15.0,
        max_power: 35.0,
        total_energy_wh: 180.0,
        screen_on_hours: 8.5,
        charging_hours: 2.0,
        battery_cycles: 0.5,
    }
}

fn sample_daily_top_process() -> DailyTopProcess {
    DailyTopProcess {
        date: "2024-01-01".to_string(),
        process_name: "Safari".to_string(),
        total_impact: 500.0,
        avg_cpu: 15.0,
        avg_memory_mb: 256.0,
        sample_count: 100,
        avg_power: 12.5,
        total_energy_wh: 15.0,
    }
}

fn sample_sample() -> Sample {
    Sample {
        timestamp: 1704067200,
        battery_percent: 85.0,
        power_watts: 12.5,
        cpu_power: 8.5,
        gpu_power: 2.3,
        charging_state: ChargingState::Discharging,
    }
}

fn sample_charge_session() -> ChargeSession {
    ChargeSession {
        start_time: 1704060000,
        end_time: Some(1704067200),
        start_percent: 20.0,
        end_percent: Some(80.0),
        energy_wh: Some(35.0),
        charger_watts: Some(67),
        avg_power_watts: Some(17.5),
        session_type: SessionType::Charge,
        is_complete: true,
    }
}

fn sample_daily_cycle() -> DailyCycle {
    DailyCycle {
        date: "2024-01-01".to_string(),
        charge_sessions: 2,
        discharge_sessions: 3,
        total_charging_mins: 120,
        total_discharge_mins: 360,
        deepest_discharge_percent: Some(20.0),
        energy_charged_wh: 50.0,
        energy_discharged_wh: 45.0,
        partial_cycles: 0.75,
        macos_cycle_count: Some(245),
        avg_temperature_c: Some(32.5),
        time_at_high_soc_mins: 60,
    }
}

fn sample_cycle_summary() -> CycleSummary {
    CycleSummary {
        total_cycles_macos: 245,
        partial_cycles_calculated: 0.75,
        avg_daily_cycles: 0.25,
        avg_depth_of_discharge: 60.0,
        avg_charge_sessions_per_day: 2.0,
        time_at_high_soc_percent: 10.0,
        estimated_cycles_remaining: Some(755),
        days_analyzed: 7,
    }
}

fn sample_kill_result() -> KillProcessResult {
    KillProcessResult {
        pid: 1234,
        success: true,
        error: None,
    }
}

#[test]
fn generate_request_fixtures() {
    let dir = requests_dir();
    fs::create_dir_all(&dir).unwrap();

    let requests: Vec<(&str, DaemonRequest)> = vec![
        ("get_status", DaemonRequest::GetStatus),
        (
            "get_hourly_stats",
            DaemonRequest::GetHourlyStats {
                from: 1704060000,
                to: 1704067200,
            },
        ),
        (
            "get_daily_stats",
            DaemonRequest::GetDailyStats {
                from: "2024-01-01".to_string(),
                to: "2024-01-07".to_string(),
            },
        ),
        (
            "get_top_processes_range",
            DaemonRequest::GetTopProcessesRange {
                from: "2024-01-01".to_string(),
                to: "2024-01-07".to_string(),
                limit: 10,
            },
        ),
        (
            "get_recent_samples",
            DaemonRequest::GetRecentSamples { window_secs: 300 },
        ),
        ("shutdown", DaemonRequest::Shutdown),
        ("subscribe", DaemonRequest::Subscribe),
        ("unsubscribe", DaemonRequest::Unsubscribe),
        ("get_current_data", DaemonRequest::GetCurrentData),
        (
            "kill_process_force",
            DaemonRequest::KillProcess {
                pid: 1234,
                signal: KillSignal::Force,
            },
        ),
        (
            "kill_process_graceful",
            DaemonRequest::KillProcess {
                pid: 1234,
                signal: KillSignal::Graceful,
            },
        ),
        (
            "set_broadcast_interval",
            DaemonRequest::SetBroadcastInterval { interval_ms: 1000 },
        ),
        (
            "get_cycle_summary",
            DaemonRequest::GetCycleSummary { days: 7 },
        ),
        (
            "get_charge_sessions",
            DaemonRequest::GetChargeSessions {
                from: 1704060000,
                to: 1704067200,
            },
        ),
        (
            "get_daily_cycles",
            DaemonRequest::GetDailyCycles {
                from: "2024-01-01".to_string(),
                to: "2024-01-07".to_string(),
            },
        ),
    ];

    for (name, request) in requests {
        let json = serde_json::to_string_pretty(&request).unwrap();
        write_fixture(&dir, name, &json);
    }
}

#[test]
fn generate_response_fixtures() {
    let dir = responses_dir();
    fs::create_dir_all(&dir).unwrap();

    let responses: Vec<(&str, DaemonResponse)> = vec![
        ("status", DaemonResponse::Status(sample_daemon_status())),
        (
            "hourly_stats",
            DaemonResponse::HourlyStats(vec![sample_hourly_stat()]),
        ),
        (
            "daily_stats",
            DaemonResponse::DailyStats(vec![sample_daily_stat()]),
        ),
        (
            "top_processes",
            DaemonResponse::TopProcesses(vec![sample_daily_top_process()]),
        ),
        (
            "recent_samples",
            DaemonResponse::RecentSamples(vec![sample_sample()]),
        ),
        ("ok", DaemonResponse::Ok),
        (
            "error",
            DaemonResponse::Error("Something went wrong".to_string()),
        ),
        ("subscribed", DaemonResponse::Subscribed),
        ("unsubscribed", DaemonResponse::Unsubscribed),
        (
            "data_update",
            DaemonResponse::DataUpdate(sample_data_snapshot()),
        ),
        (
            "current_data",
            DaemonResponse::CurrentData(sample_data_snapshot()),
        ),
        (
            "kill_result",
            DaemonResponse::KillResult(sample_kill_result()),
        ),
        (
            "subscription_rejected",
            DaemonResponse::SubscriptionRejected {
                reason: "Maximum subscribers (10) reached".to_string(),
            },
        ),
        (
            "cycle_summary",
            DaemonResponse::CycleSummary(sample_cycle_summary()),
        ),
        (
            "charge_sessions",
            DaemonResponse::ChargeSessions(vec![sample_charge_session()]),
        ),
        (
            "daily_cycles",
            DaemonResponse::DailyCycles(vec![sample_daily_cycle()]),
        ),
    ];

    for (name, response) in responses {
        let json = serde_json::to_string_pretty(&response).unwrap();
        write_fixture(&dir, name, &json);
    }
}

#[test]
fn verify_request_fixtures_deserialize() {
    let dir = requests_dir();

    for entry in fs::read_dir(&dir).expect("Failed to read requests directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let content = fs::read_to_string(&path).unwrap();
            let result: Result<DaemonRequest, _> = serde_json::from_str(&content);
            assert!(
                result.is_ok(),
                "Failed to deserialize {:?}: {:?}",
                path,
                result.err()
            );
        }
    }
}

#[test]
fn verify_response_fixtures_deserialize() {
    let dir = responses_dir();

    for entry in fs::read_dir(&dir).expect("Failed to read responses directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let content = fs::read_to_string(&path).unwrap();
            let result: Result<DaemonResponse, _> = serde_json::from_str(&content);
            assert!(
                result.is_ok(),
                "Failed to deserialize {:?}: {:?}",
                path,
                result.err()
            );
        }
    }
}
