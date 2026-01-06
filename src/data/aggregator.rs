use chrono::{DateTime, Duration, Timelike, Utc};

use crate::config::HistoryConfig;
use crate::data::{DailyStat, HistoryStore, HistoryStoreError, HourlyStat};

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
        }

        if self.config.max_database_mb > 0 {
            let max_bytes = (self.config.max_database_mb as u64) * 1024 * 1024;
            let current_size = self.store.size_bytes()?;

            if current_size > max_bytes {
                let stats = self.store.get_stats()?;
                if stats.oldest_sample.is_some() {
                    let target_size = max_bytes * 80 / 100;
                    let ratio = target_size as f64 / current_size as f64;
                    let days_to_keep = (stats.sample_count as f64 * ratio) as i64
                        / (86400 / self.config.sample_interval_secs as i64);

                    let cutoff = now - Duration::days(days_to_keep.max(1));
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
    pub daily_deleted: usize,
    pub processes_deleted: usize,
}

fn date_to_timestamp(date: &str) -> Result<i64, HistoryStoreError> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
        .map_err(|e| {
            HistoryStoreError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
        })
}

fn next_date(date: &str) -> String {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(1)).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|_| date.to_string())
}
