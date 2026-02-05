use std::time::{Duration, Instant};

use chrono::Utc;
use tracing::{debug, trace};

use crate::config::HistoryConfig;
use crate::data::{
    BatteryData, BatteryHealthSnapshot, ChargingState, CycleSnapshot, DailyTopProcess,
    HistoryStore, HistoryStoreError, PowerData, ProcessData, Sample, SessionEvent, SessionTracker,
};

pub struct Recorder {
    store: HistoryStore,
    config: HistoryConfig,
    last_sample_time: Option<Instant>,
    last_health_date: Option<String>,
    last_cycle_snapshot_date: Option<String>,
    excluded_processes: Vec<String>,
    session_tracker: SessionTracker,
}

impl Recorder {
    pub fn new(
        config: HistoryConfig,
        excluded_processes: Vec<String>,
    ) -> Result<Self, HistoryStoreError> {
        let store = HistoryStore::open()?;

        let session_tracker = match store.get_incomplete_session()? {
            Some(session) => {
                debug!(session_id = session.id, "Recovered incomplete session");
                SessionTracker::with_incomplete_session(session)
            }
            None => SessionTracker::new(),
        };

        debug!(
            sample_interval_secs = config.sample_interval_secs,
            retention_raw_days = config.retention_raw_days,
            "Recorder initialized"
        );
        Ok(Self {
            store,
            config,
            last_sample_time: None,
            last_health_date: None,
            last_cycle_snapshot_date: None,
            excluded_processes,
            session_tracker,
        })
    }

    pub fn store(&self) -> &HistoryStore {
        &self.store
    }

    pub fn should_record(&self) -> bool {
        if !self.config.background_recording {
            return false;
        }

        match self.last_sample_time {
            Some(last) => last.elapsed() >= Duration::from_secs(self.config.sample_interval_secs),
            None => true,
        }
    }

    pub fn record_sample(
        &mut self,
        battery: &BatteryData,
        power: &PowerData,
    ) -> Result<(), HistoryStoreError> {
        if !self.should_record() {
            return Ok(());
        }

        let charging_state = match battery.state_label() {
            "Charging" => ChargingState::Charging,
            "Full" => ChargingState::Full,
            "Discharging" | "Not Charging" => ChargingState::Discharging,
            _ => ChargingState::Unknown,
        };

        let sample = Sample {
            id: None,
            timestamp: Utc::now().timestamp(),
            battery_percent: battery.charge_percent(),
            power_watts: power.total_power_watts(),
            cpu_power: power.cpu_power_watts(),
            gpu_power: power.gpu_power_watts(),
            charging_state,
        };

        self.store.insert_sample(&sample)?;
        self.last_sample_time = Some(Instant::now());

        self.session_tracker
            .record_power_sample(power.total_power_watts());
        if let Some(event) = self.session_tracker.process_sample(battery) {
            self.handle_session_event(event)?;
        }

        Ok(())
    }

    fn handle_session_event(&mut self, event: SessionEvent) -> Result<(), HistoryStoreError> {
        match event {
            SessionEvent::Started(session) => {
                let id = self.store.insert_charge_session(&session)?;
                trace!(session_id = id, session_type = ?session.session_type, "Session started");
            }
            SessionEvent::Ended(session) => {
                if session.id.is_some() {
                    self.store.update_charge_session(&session)?;
                    trace!(session_id = session.id, "Session ended and updated");
                } else {
                    let id = self.store.insert_charge_session(&session)?;
                    trace!(session_id = id, "Session ended and inserted");
                }
            }
        }
        Ok(())
    }

    pub fn record_processes(
        &mut self,
        processes: &ProcessData,
        system_cpu_power: f32,
    ) -> Result<(), HistoryStoreError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        let top_processes: Vec<_> = processes
            .processes
            .iter()
            .filter(|p| !self.excluded_processes.iter().any(|ex| p.name.contains(ex)))
            .take(10)
            .collect();

        let total_cpu: f32 = top_processes.iter().map(|p| p.cpu_usage).sum();
        let sample_hours = self.config.sample_interval_secs as f32 / 3600.0;

        for process in top_processes {
            let process_power = if total_cpu > 0.0 {
                (process.cpu_usage / total_cpu) * system_cpu_power
            } else {
                0.0
            };
            let sample_energy_wh = process_power * sample_hours;

            let entry = DailyTopProcess {
                id: None,
                date: today.clone(),
                process_name: process.name.clone(),
                total_impact: process.energy_impact,
                avg_cpu: process.cpu_usage,
                avg_memory_mb: process.memory_mb as f32,
                sample_count: 1,
                avg_power: process_power,
                total_energy_wh: sample_energy_wh,
            };
            self.store.upsert_daily_process(&entry)?;
        }

        Ok(())
    }

    pub fn record_battery_health(
        &mut self,
        battery: &BatteryData,
    ) -> Result<(), HistoryStoreError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        if self.last_health_date.as_ref() == Some(&today) {
            return Ok(());
        }

        let snapshot = BatteryHealthSnapshot {
            id: None,
            date: today.clone(),
            health_percent: battery.health_percent(),
            cycle_count: battery.cycle_count().map(|c| c as i32),
            max_capacity_wh: battery.max_capacity_wh(),
            design_capacity_wh: battery.design_capacity_wh(),
        };

        self.store.upsert_battery_health(&snapshot)?;
        self.last_health_date = Some(today);

        Ok(())
    }

    pub fn record_cycle_snapshot(
        &mut self,
        battery: &BatteryData,
    ) -> Result<(), HistoryStoreError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        if self.last_cycle_snapshot_date.as_ref() == Some(&today) {
            return Ok(());
        }

        let macos_cycle_count = battery.cycle_count().unwrap_or(0) as i32;
        let calculated_partial_cycles = self.session_tracker.get_partial_cycles();

        let snapshot = CycleSnapshot {
            id: None,
            date: today.clone(),
            macos_cycle_count,
            calculated_partial_cycles,
            battery_health_percent: battery.health_percent(),
        };

        self.store.upsert_cycle_snapshot(&snapshot)?;
        self.last_cycle_snapshot_date = Some(today);
        self.session_tracker.reset_partial_cycles();
        self.session_tracker.reset_time_at_high_soc();

        debug!(
            macos_cycle_count,
            calculated_partial_cycles, "Recorded daily cycle snapshot"
        );

        Ok(())
    }

    pub fn record_all(
        &mut self,
        battery: &BatteryData,
        power: &PowerData,
        processes: &ProcessData,
    ) -> Result<(), HistoryStoreError> {
        if self.should_record() {
            self.record_sample(battery, power)?;
            self.record_processes(processes, power.cpu_power_watts())?;
            self.record_battery_health(battery)?;
            self.record_cycle_snapshot(battery)?;
        }
        Ok(())
    }
}
