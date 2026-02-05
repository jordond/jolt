use chrono::{DateTime, Duration, Timelike, Utc};
use tracing::debug;

use crate::config::HistoryConfig;
use crate::data::{
    DailyCycle, DailyStat, HistoryStore, HistoryStoreError, HourlyStat, SessionType,
};

pub struct Aggregator<'a> {
    store: &'a HistoryStore,
    config: &'a HistoryConfig,
}

impl<'a> Aggregator<'a> {
    pub fn new(store: &'a HistoryStore, config: &'a HistoryConfig) -> Self {
        Self { store, config }
    }

    pub fn compute_hourly_stats(
        &self,
        hour_start: i64,
    ) -> Result<Option<HourlyStat>, HistoryStoreError> {
        let hour_end = hour_start + 3600;
        let samples = self.store.get_samples(hour_start, hour_end - 1)?;

        if samples.is_empty() {
            return Ok(None);
        }

        let total_samples = samples.len() as i32;
        let sum_power: f32 = samples.iter().map(|s| s.power_watts).sum();
        let sum_battery: f32 = samples.iter().map(|s| s.battery_percent).sum();
        let max_power = samples
            .iter()
            .map(|s| s.power_watts)
            .fold(0.0_f32, f32::max);
        let min_power = samples
            .iter()
            .map(|s| s.power_watts)
            .fold(f32::MAX, f32::min);

        let first_battery = samples.first().map(|s| s.battery_percent).unwrap_or(0.0);
        let last_battery = samples.last().map(|s| s.battery_percent).unwrap_or(0.0);
        let battery_delta = last_battery - first_battery;

        Ok(Some(HourlyStat {
            id: None,
            hour_start,
            avg_power: sum_power / total_samples as f32,
            max_power,
            min_power,
            avg_battery: sum_battery / total_samples as f32,
            battery_delta,
            total_samples,
        }))
    }

    pub fn compute_daily_stats(&self, date: &str) -> Result<Option<DailyStat>, HistoryStoreError> {
        let day_start = date_to_timestamp(date)?;
        let day_end = day_start + 86400;

        let samples = self.store.get_samples(day_start, day_end - 1)?;
        if samples.is_empty() {
            return Ok(None);
        }

        let total_samples = samples.len() as f32;
        let sum_power: f32 = samples.iter().map(|s| s.power_watts).sum();
        let max_power = samples
            .iter()
            .map(|s| s.power_watts)
            .fold(0.0_f32, f32::max);

        let interval_hours = self.config.sample_interval_secs as f32 / 3600.0;
        let total_energy_wh = sum_power * interval_hours;

        let charging_samples = samples
            .iter()
            .filter(|s| matches!(s.charging_state, crate::data::ChargingState::Charging))
            .count() as f32;
        let charging_hours = charging_samples * interval_hours;

        let first_battery = samples.first().map(|s| s.battery_percent).unwrap_or(0.0);
        let last_battery = samples.last().map(|s| s.battery_percent).unwrap_or(0.0);
        let battery_delta = (first_battery - last_battery).abs();
        let battery_cycles = battery_delta / 100.0;

        let screen_on_hours = total_samples * interval_hours;

        Ok(Some(DailyStat {
            id: None,
            date: date.to_string(),
            avg_power: sum_power / total_samples,
            max_power,
            total_energy_wh,
            screen_on_hours,
            charging_hours,
            battery_cycles,
        }))
    }

    pub fn compute_daily_cycles(
        &self,
        date: &str,
    ) -> Result<Option<DailyCycle>, HistoryStoreError> {
        let day_start = date_to_timestamp(date)?;
        let day_end = day_start + 86400;

        let sessions = self
            .store
            .get_charge_sessions(day_start, day_end - 1, None)?;
        if sessions.is_empty() {
            return Ok(None);
        }

        let charge_sessions = sessions
            .iter()
            .filter(|s| matches!(s.session_type, SessionType::Charge))
            .count() as i32;

        let discharge_sessions = sessions
            .iter()
            .filter(|s| matches!(s.session_type, SessionType::Discharge))
            .count() as i32;

        let mut total_charging_secs: i64 = 0;
        let mut total_discharge_secs: i64 = 0;
        let mut energy_charged_wh: f32 = 0.0;
        let mut energy_discharged_wh: f32 = 0.0;
        let mut deepest_discharge: Option<f32> = None;
        let mut total_discharge_percent: f32 = 0.0;

        for session in &sessions {
            let duration = session.duration_secs().unwrap_or(0);

            match session.session_type {
                SessionType::Charge => {
                    total_charging_secs += duration;
                    if let Some(energy) = session.energy_wh {
                        energy_charged_wh += energy;
                    }
                }
                SessionType::Discharge => {
                    total_discharge_secs += duration;
                    if let Some(energy) = session.energy_wh {
                        energy_discharged_wh += energy;
                    }
                    if let Some(end_percent) = session.end_percent {
                        match deepest_discharge {
                            Some(current) if end_percent < current => {
                                deepest_discharge = Some(end_percent);
                            }
                            None => {
                                deepest_discharge = Some(end_percent);
                            }
                            _ => {}
                        }
                    }
                    if let Some(delta) = session.percent_delta() {
                        total_discharge_percent += delta.abs();
                    }
                }
            }
        }

        let partial_cycles = total_discharge_percent / 100.0;
        let time_at_high_soc_mins = self.compute_time_at_high_soc(day_start, day_end)?;

        Ok(Some(DailyCycle {
            id: None,
            date: date.to_string(),
            charge_sessions,
            discharge_sessions,
            total_charging_mins: (total_charging_secs / 60) as i32,
            total_discharge_mins: (total_discharge_secs / 60) as i32,
            deepest_discharge_percent: deepest_discharge,
            energy_charged_wh,
            energy_discharged_wh,
            partial_cycles,
            macos_cycle_count: None,
            avg_temperature_c: None,
            time_at_high_soc_mins,
        }))
    }

    fn compute_time_at_high_soc(&self, from_ts: i64, to_ts: i64) -> Result<i32, HistoryStoreError> {
        const HIGH_SOC_THRESHOLD: f32 = 80.0;

        let samples = self.store.get_samples(from_ts, to_ts)?;
        if samples.len() < 2 {
            return Ok(0);
        }

        let mut total_secs: i64 = 0;
        for window in samples.windows(2) {
            if window[0].battery_percent >= HIGH_SOC_THRESHOLD {
                let elapsed = window[1].timestamp - window[0].timestamp;
                total_secs += elapsed;
            }
        }

        Ok((total_secs / 60) as i32)
    }

    pub fn aggregate_completed_hours(&self) -> Result<usize, HistoryStoreError> {
        let now = Utc::now();
        let current_hour = now.date_naive().and_hms_opt(now.hour(), 0, 0).unwrap();
        let current_hour_ts = current_hour.and_utc().timestamp();

        let stats = self.store.get_stats()?;
        let oldest_sample = match stats.oldest_sample {
            Some(ts) => ts,
            None => return Ok(0),
        };

        let oldest_hour = DateTime::from_timestamp(oldest_sample, 0)
            .map(|dt| {
                dt.date_naive()
                    .and_hms_opt(dt.hour(), 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp()
            })
            .unwrap_or(oldest_sample);

        let mut aggregated = 0;
        let mut hour_ts = oldest_hour;

        while hour_ts < current_hour_ts {
            let existing = self.store.get_hourly_stats(hour_ts, hour_ts)?;
            if existing.is_empty() {
                if let Some(stat) = self.compute_hourly_stats(hour_ts)? {
                    self.store.upsert_hourly_stat(&stat)?;
                    aggregated += 1;
                }
            }
            hour_ts += 3600;
        }

        Ok(aggregated)
    }

    pub fn aggregate_completed_days(&self) -> Result<usize, HistoryStoreError> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        let stats = self.store.get_stats()?;
        let oldest_sample = match stats.oldest_sample {
            Some(ts) => ts,
            None => return Ok(0),
        };

        let oldest_date = DateTime::from_timestamp(oldest_sample, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| today.clone());

        let mut aggregated = 0;
        let mut current_date = oldest_date;

        while current_date < today {
            let existing = self.store.get_daily_stat(&current_date)?;
            if existing.is_none() {
                if let Some(stat) = self.compute_daily_stats(&current_date)? {
                    self.store.upsert_daily_stat(&stat)?;
                    aggregated += 1;
                }
            }

            let existing_cycles = self.store.get_daily_cycle(&current_date)?;
            if existing_cycles.is_none() {
                if let Some(cycle) = self.compute_daily_cycles(&current_date)? {
                    self.store.upsert_daily_cycle(&cycle)?;
                }
            }

            current_date = next_date(&current_date);
        }

        Ok(aggregated)
    }

    pub fn prune_old_data(&self) -> Result<PruneResult, HistoryStoreError> {
        let now = Utc::now();
        let mut result = PruneResult::default();

        if self.config.retention_raw_days > 0 {
            let cutoff = now - Duration::days(self.config.retention_raw_days as i64);
            let cutoff_ts = cutoff.timestamp();
            result.samples_deleted = self.store.delete_samples_before(cutoff_ts)?;
        }

        if self.config.retention_hourly_days > 0 {
            let cutoff = now - Duration::days(self.config.retention_hourly_days as i64);
            let cutoff_ts = cutoff.timestamp();
            result.hourly_deleted = self.store.delete_hourly_stats_before(cutoff_ts)?;
        }

        if self.config.retention_daily_days > 0 {
            let cutoff = now - Duration::days(self.config.retention_daily_days as i64);
            let cutoff_date = cutoff.format("%Y-%m-%d").to_string();
            result.daily_deleted = self.store.delete_daily_stats_before(&cutoff_date)?;
            result.processes_deleted = self.store.delete_daily_processes_before(&cutoff_date)?;
            result.daily_cycles_deleted = self.store.delete_daily_cycles_before(&cutoff_date)?;
        }

        let session_cutoff = now - Duration::days(self.config.retention_sessions_days as i64);
        let session_cutoff_ts = session_cutoff.timestamp();
        result.sessions_deleted = self
            .store
            .delete_charge_sessions_before(session_cutoff_ts)?;

        if self.config.max_database_mb > 0 {
            let max_bytes = (self.config.max_database_mb as u64) * 1024 * 1024;
            let current_size = self.store.size_bytes()?;

            if current_size > max_bytes {
                debug!(
                    current_mb = current_size / (1024 * 1024),
                    max_mb = self.config.max_database_mb,
                    "Database size exceeded limit, pruning"
                );
                let stats = self.store.get_stats()?;
                if stats.oldest_sample.is_some() {
                    let target_size = max_bytes * 80 / 100;
                    let ratio = target_size as f64 / current_size as f64;
                    let samples_per_day =
                        (86400_f64 / self.config.sample_interval_secs as f64).max(1.0);
                    let estimated_days = (stats.sample_count as f64 * ratio) / samples_per_day;
                    let min_retention_days: i64 = 7;
                    let days_to_keep = (estimated_days.floor() as i64).max(min_retention_days);

                    let cutoff = now - Duration::days(days_to_keep);
                    let cutoff_ts = cutoff.timestamp();
                    result.samples_deleted += self.store.delete_samples_before(cutoff_ts)?;
                }

                self.store.vacuum()?;
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Default)]
pub struct PruneResult {
    pub samples_deleted: usize,
    pub hourly_deleted: usize,
    pub sessions_deleted: usize,
    pub daily_cycles_deleted: usize,
    pub daily_deleted: usize,
    pub processes_deleted: usize,
}

fn date_to_timestamp(date: &str) -> Result<i64, HistoryStoreError> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| {
            let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
            d.and_time(time).and_utc().timestamp()
        })
        .map_err(|e| {
            HistoryStoreError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })
}

fn next_date(date: &str) -> String {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(1)).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|_| date.to_string())
}
