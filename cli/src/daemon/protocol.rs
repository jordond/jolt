use serde::{Deserialize, Serialize};

use crate::data::{DailyStat, DailyTopProcess, HourlyStat, Sample};

#[allow(dead_code)]
pub const MAX_SUBSCRIBERS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    NotCharging,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PowerMode {
    LowPower,
    #[default]
    Automatic,
    HighPerformance,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    Running,
    Sleeping,
    Idle,
    Stopped,
    Zombie,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KillSignal {
    Graceful,
    #[default]
    Force,
}

impl KillSignal {
    pub fn as_arg(&self) -> &'static str {
        match self {
            KillSignal::Graceful => "-15",
            KillSignal::Force => "-9",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatterySnapshot {
    pub charge_percent: f32,
    pub state: BatteryState,
    pub state_label: String,
    pub health_percent: f32,
    pub max_capacity_wh: f32,
    pub design_capacity_wh: f32,
    pub cycle_count: Option<u32>,
    pub time_remaining_mins: Option<u64>,
    pub time_remaining_formatted: Option<String>,
    pub charging_watts: Option<f32>,
    pub charger_watts: Option<u32>,
    pub discharge_watts: Option<f32>,
    pub voltage_mv: u32,
    pub amperage_ma: i32,
    pub external_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PowerSnapshot {
    pub cpu_power_watts: f32,
    pub gpu_power_watts: f32,
    pub total_power_watts: f32,
    pub power_mode: PowerMode,
    pub power_mode_label: String,
    pub is_warmed_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub energy_impact: f32,
    pub parent_pid: Option<u32>,
    pub children: Option<Vec<ProcessSnapshot>>,
    pub is_killable: bool,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub status: ProcessState,
    pub run_time_secs: u64,
    pub total_cpu_time_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSnapshot {
    pub timestamp: i64,
    pub battery: BatterySnapshot,
    pub power: PowerSnapshot,
    pub processes: Vec<ProcessSnapshot>,
}

impl Default for DataSnapshot {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp(),
            battery: BatterySnapshot::default(),
            power: PowerSnapshot::default(),
            processes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonRequest {
    GetStatus,
    GetHourlyStats {
        from: i64,
        to: i64,
    },
    GetDailyStats {
        from: String,
        to: String,
    },
    GetTopProcessesRange {
        from: String,
        to: String,
        limit: usize,
    },
    GetRecentSamples {
        window_secs: u64,
    },
    Shutdown,
    Subscribe,
    Unsubscribe,
    GetCurrentData,
    KillProcess {
        pid: u32,
        signal: KillSignal,
    },
    SetBroadcastInterval {
        interval_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub uptime_secs: u64,
    pub sample_count: i64,
    pub last_sample_time: Option<i64>,
    pub database_size_bytes: u64,
    pub version: String,
    pub subscriber_count: usize,
    pub history_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillProcessResult {
    pub pid: u32,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonResponse {
    Status(DaemonStatus),
    HourlyStats(Vec<HourlyStat>),
    DailyStats(Vec<DailyStat>),
    TopProcesses(Vec<DailyTopProcess>),
    RecentSamples(Vec<Sample>),
    Ok,
    Error(String),
    Subscribed,
    Unsubscribed,
    DataUpdate(DataSnapshot),
    CurrentData(DataSnapshot),
    KillResult(KillProcessResult),
    SubscriptionRejected { reason: String },
}

impl DaemonRequest {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl DaemonResponse {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_snapshot_serialization() {
        let snapshot = BatterySnapshot {
            charge_percent: 85.5,
            state: BatteryState::Discharging,
            state_label: "On Battery".to_string(),
            health_percent: 92.0,
            max_capacity_wh: 52.6,
            design_capacity_wh: 58.0,
            cycle_count: Some(245),
            time_remaining_mins: Some(180),
            time_remaining_formatted: Some("3h 0m".to_string()),
            charging_watts: None,
            charger_watts: None,
            discharge_watts: Some(12.5),
            voltage_mv: 11500,
            amperage_ma: -1087,
            external_connected: false,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: BatterySnapshot = serde_json::from_str(&json).unwrap();

        assert!((deserialized.charge_percent - 85.5).abs() < 0.01);
        assert_eq!(deserialized.state, BatteryState::Discharging);
        assert_eq!(deserialized.cycle_count, Some(245));
    }

    #[test]
    fn test_power_snapshot_serialization() {
        let snapshot = PowerSnapshot {
            cpu_power_watts: 8.5,
            gpu_power_watts: 2.3,
            total_power_watts: 12.8,
            power_mode: PowerMode::Automatic,
            power_mode_label: "Automatic".to_string(),
            is_warmed_up: true,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: PowerSnapshot = serde_json::from_str(&json).unwrap();

        assert!((deserialized.total_power_watts - 12.8).abs() < 0.01);
        assert_eq!(deserialized.power_mode, PowerMode::Automatic);
        assert!(deserialized.is_warmed_up);
    }

    #[test]
    fn test_process_snapshot_serialization() {
        let child = ProcessSnapshot {
            pid: 1002,
            name: "Helper".to_string(),
            command: "helper".to_string(),
            cpu_usage: 1.5,
            memory_mb: 50.0,
            energy_impact: 2.0,
            parent_pid: Some(1001),
            children: None,
            is_killable: true,
            disk_read_bytes: 1000,
            disk_write_bytes: 500,
            status: ProcessState::Running,
            run_time_secs: 3600,
            total_cpu_time_secs: 120,
        };

        let process = ProcessSnapshot {
            pid: 1001,
            name: "MyApp".to_string(),
            command: "myapp".to_string(),
            cpu_usage: 15.5,
            memory_mb: 256.0,
            energy_impact: 25.0,
            parent_pid: None,
            children: Some(vec![child]),
            is_killable: true,
            disk_read_bytes: 10000,
            disk_write_bytes: 5000,
            status: ProcessState::Running,
            run_time_secs: 7200,
            total_cpu_time_secs: 600,
        };

        let json = serde_json::to_string(&process).unwrap();
        let deserialized: ProcessSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.pid, 1001);
        assert_eq!(deserialized.name, "MyApp");
        assert!(deserialized.children.is_some());
        assert_eq!(deserialized.children.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_data_snapshot_serialization() {
        let snapshot = DataSnapshot {
            timestamp: 1704067200,
            battery: BatterySnapshot {
                charge_percent: 75.0,
                state: BatteryState::Charging,
                ..Default::default()
            },
            power: PowerSnapshot {
                total_power_watts: 15.0,
                ..Default::default()
            },
            processes: vec![ProcessSnapshot {
                pid: 100,
                name: "Test".to_string(),
                command: "test".to_string(),
                cpu_usage: 5.0,
                memory_mb: 100.0,
                energy_impact: 10.0,
                parent_pid: None,
                children: None,
                is_killable: true,
                disk_read_bytes: 0,
                disk_write_bytes: 0,
                status: ProcessState::Running,
                run_time_secs: 100,
                total_cpu_time_secs: 10,
            }],
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: DataSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.timestamp, 1704067200);
        assert!((deserialized.battery.charge_percent - 75.0).abs() < 0.01);
        assert_eq!(deserialized.processes.len(), 1);
    }

    #[test]
    fn test_daemon_request_subscribe() {
        let req = DaemonRequest::Subscribe;
        let json = req.to_json().unwrap();
        let deserialized = DaemonRequest::from_json(&json).unwrap();

        assert!(matches!(deserialized, DaemonRequest::Subscribe));
    }

    #[test]
    fn test_daemon_request_unsubscribe() {
        let req = DaemonRequest::Unsubscribe;
        let json = req.to_json().unwrap();
        let deserialized = DaemonRequest::from_json(&json).unwrap();

        assert!(matches!(deserialized, DaemonRequest::Unsubscribe));
    }

    #[test]
    fn test_daemon_request_kill_process() {
        let req = DaemonRequest::KillProcess {
            pid: 12345,
            signal: KillSignal::Force,
        };
        let json = req.to_json().unwrap();
        let deserialized = DaemonRequest::from_json(&json).unwrap();

        match deserialized {
            DaemonRequest::KillProcess { pid, signal } => {
                assert_eq!(pid, 12345);
                assert_eq!(signal, KillSignal::Force);
            }
            _ => panic!("Expected KillProcess"),
        }
    }

    #[test]
    fn test_daemon_request_kill_process_graceful() {
        let req = DaemonRequest::KillProcess {
            pid: 99999,
            signal: KillSignal::Graceful,
        };
        let json = req.to_json().unwrap();
        let deserialized = DaemonRequest::from_json(&json).unwrap();

        match deserialized {
            DaemonRequest::KillProcess { pid, signal } => {
                assert_eq!(pid, 99999);
                assert_eq!(signal, KillSignal::Graceful);
            }
            _ => panic!("Expected KillProcess"),
        }
    }

    #[test]
    fn test_kill_signal_variants() {
        for signal in [KillSignal::Graceful, KillSignal::Force] {
            let json = serde_json::to_string(&signal).unwrap();
            let deserialized: KillSignal = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, signal);
        }
    }

    #[test]
    fn test_daemon_request_get_current_data() {
        let req = DaemonRequest::GetCurrentData;
        let json = req.to_json().unwrap();
        let deserialized = DaemonRequest::from_json(&json).unwrap();

        assert!(matches!(deserialized, DaemonRequest::GetCurrentData));
    }

    #[test]
    fn test_daemon_response_subscribed() {
        let resp = DaemonResponse::Subscribed;
        let json = resp.to_json().unwrap();
        let deserialized = DaemonResponse::from_json(&json).unwrap();

        assert!(matches!(deserialized, DaemonResponse::Subscribed));
    }

    #[test]
    fn test_daemon_response_unsubscribed() {
        let resp = DaemonResponse::Unsubscribed;
        let json = resp.to_json().unwrap();
        let deserialized = DaemonResponse::from_json(&json).unwrap();

        assert!(matches!(deserialized, DaemonResponse::Unsubscribed));
    }

    #[test]
    fn test_daemon_response_data_update() {
        let snapshot = DataSnapshot::default();
        let resp = DaemonResponse::DataUpdate(snapshot);
        let json = resp.to_json().unwrap();
        let deserialized = DaemonResponse::from_json(&json).unwrap();

        assert!(matches!(deserialized, DaemonResponse::DataUpdate(_)));
    }

    #[test]
    fn test_daemon_response_kill_result() {
        let result = KillProcessResult {
            pid: 12345,
            success: true,
            error: None,
        };
        let resp = DaemonResponse::KillResult(result);
        let json = resp.to_json().unwrap();
        let deserialized = DaemonResponse::from_json(&json).unwrap();

        match deserialized {
            DaemonResponse::KillResult(r) => {
                assert_eq!(r.pid, 12345);
                assert!(r.success);
                assert!(r.error.is_none());
            }
            _ => panic!("Expected KillResult"),
        }
    }

    #[test]
    fn test_daemon_response_subscription_rejected() {
        let resp = DaemonResponse::SubscriptionRejected {
            reason: "Max subscribers reached".to_string(),
        };
        let json = resp.to_json().unwrap();
        let deserialized = DaemonResponse::from_json(&json).unwrap();

        match deserialized {
            DaemonResponse::SubscriptionRejected { reason } => {
                assert_eq!(reason, "Max subscribers reached");
            }
            _ => panic!("Expected SubscriptionRejected"),
        }
    }

    #[test]
    fn test_daemon_status_with_new_fields() {
        let status = DaemonStatus {
            running: true,
            uptime_secs: 3600,
            sample_count: 1000,
            last_sample_time: Some(1704067200),
            database_size_bytes: 1024000,
            version: "0.1.0".to_string(),
            subscriber_count: 2,
            history_enabled: true,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: DaemonStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.subscriber_count, 2);
        assert!(deserialized.history_enabled);
    }

    #[test]
    fn test_battery_state_variants() {
        for state in [
            BatteryState::Charging,
            BatteryState::Discharging,
            BatteryState::Full,
            BatteryState::NotCharging,
            BatteryState::Unknown,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: BatteryState = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, state);
        }
    }

    #[test]
    fn test_power_mode_variants() {
        for mode in [
            PowerMode::LowPower,
            PowerMode::Automatic,
            PowerMode::HighPerformance,
            PowerMode::Unknown,
        ] {
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: PowerMode = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, mode);
        }
    }

    #[test]
    fn test_process_state_variants() {
        for state in [
            ProcessState::Running,
            ProcessState::Sleeping,
            ProcessState::Idle,
            ProcessState::Stopped,
            ProcessState::Zombie,
            ProcessState::Unknown,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: ProcessState = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, state);
        }
    }

    #[test]
    fn test_existing_requests_still_work() {
        let req = DaemonRequest::GetStatus;
        let json = req.to_json().unwrap();
        assert!(DaemonRequest::from_json(&json).is_ok());

        let req = DaemonRequest::GetHourlyStats {
            from: 1000,
            to: 2000,
        };
        let json = req.to_json().unwrap();
        assert!(DaemonRequest::from_json(&json).is_ok());

        let req = DaemonRequest::GetDailyStats {
            from: "2024-01-01".to_string(),
            to: "2024-01-31".to_string(),
        };
        let json = req.to_json().unwrap();
        assert!(DaemonRequest::from_json(&json).is_ok());

        let req = DaemonRequest::Shutdown;
        let json = req.to_json().unwrap();
        assert!(DaemonRequest::from_json(&json).is_ok());
    }

    #[test]
    fn test_existing_responses_still_work() {
        let status = DaemonStatus {
            running: true,
            uptime_secs: 100,
            sample_count: 50,
            last_sample_time: None,
            database_size_bytes: 1000,
            version: "1.0.0".to_string(),
            subscriber_count: 0,
            history_enabled: true,
        };
        let resp = DaemonResponse::Status(status);
        let json = resp.to_json().unwrap();
        assert!(DaemonResponse::from_json(&json).is_ok());

        let resp = DaemonResponse::Ok;
        let json = resp.to_json().unwrap();
        assert!(DaemonResponse::from_json(&json).is_ok());

        let resp = DaemonResponse::Error("test error".to_string());
        let json = resp.to_json().unwrap();
        assert!(DaemonResponse::from_json(&json).is_ok());
    }
}
