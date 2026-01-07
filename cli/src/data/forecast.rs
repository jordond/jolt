//! Battery runtime forecast based on historical power consumption.
//!
//! Calculates a more accurate battery runtime estimate by averaging power draw
//! over a configurable time window, rather than using just the instantaneous value.

use std::time::Duration;

use crate::data::history::DataPoint;
use crate::data::history_store::Sample;

const MIN_SAMPLES_FOR_FORECAST: usize = 3;
const MAX_STALENESS_SECS: i64 = 300;

/// Source of data used for forecast calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForecastSource {
    /// Using daemon historical data (more accurate, longer history)
    Daemon,
    /// Using TUI session data (since app started)
    Session,
    /// No data available for forecast
    None,
}

#[derive(Debug, Clone)]
pub struct ForecastData {
    forecast_duration: Option<Duration>,
    avg_power_watts: Option<f32>,
    sample_count: usize,
    source: ForecastSource,
    last_sample_timestamp: Option<i64>,
}

impl Default for ForecastData {
    fn default() -> Self {
        Self::new()
    }
}

impl ForecastData {
    pub fn new() -> Self {
        Self {
            forecast_duration: None,
            avg_power_watts: None,
            sample_count: 0,
            source: ForecastSource::None,
            last_sample_timestamp: None,
        }
    }

    /// Calculate forecast from daemon samples
    ///
    /// Returns true if forecast was successfully calculated
    pub fn calculate_from_daemon_samples(
        &mut self,
        samples: &[Sample],
        current_battery_percent: f32,
        battery_capacity_wh: f32,
    ) -> bool {
        self.source = ForecastSource::Daemon;

        if samples.len() < MIN_SAMPLES_FOR_FORECAST {
            self.clear_forecast();
            return false;
        }

        // Check staleness - most recent sample should be within threshold
        let now = chrono::Utc::now().timestamp();
        let most_recent = samples.iter().map(|s| s.timestamp).max().unwrap_or(0);

        if now - most_recent > MAX_STALENESS_SECS {
            self.clear_forecast();
            self.source = ForecastSource::None;
            return false;
        }

        self.last_sample_timestamp = Some(most_recent);

        // Filter to only discharging samples (charging_state == 0)
        let discharging_samples: Vec<_> = samples
            .iter()
            .filter(|s| s.charging_state as i32 == 0)
            .collect();

        if discharging_samples.len() < MIN_SAMPLES_FOR_FORECAST {
            self.clear_forecast();
            return false;
        }

        // Calculate average power consumption
        let total_power: f32 = discharging_samples.iter().map(|s| s.power_watts).sum();
        let avg_power = total_power / discharging_samples.len() as f32;

        self.calculate_forecast(
            avg_power,
            discharging_samples.len(),
            current_battery_percent,
            battery_capacity_wh,
        )
    }

    /// Calculate forecast from in-memory session data points
    ///
    /// Returns true if forecast was successfully calculated
    pub fn calculate_from_session_data(
        &mut self,
        points: &[DataPoint],
        current_battery_percent: f32,
        battery_capacity_wh: f32,
    ) -> bool {
        self.source = ForecastSource::Session;
        self.last_sample_timestamp = None;

        if points.len() < MIN_SAMPLES_FOR_FORECAST {
            self.clear_forecast();
            return false;
        }

        // Use all available points (already filtered by time window in caller if needed)
        // Filter to points with positive power draw (discharging)
        let discharging_points: Vec<_> = points.iter().filter(|p| p.power_watts > 0.1).collect();

        if discharging_points.len() < MIN_SAMPLES_FOR_FORECAST {
            self.clear_forecast();
            return false;
        }

        // Calculate average power consumption
        let total_power: f32 = discharging_points.iter().map(|p| p.power_watts).sum();
        let avg_power = total_power / discharging_points.len() as f32;

        self.calculate_forecast(
            avg_power,
            discharging_points.len(),
            current_battery_percent,
            battery_capacity_wh,
        )
    }

    /// Internal forecast calculation
    fn calculate_forecast(
        &mut self,
        avg_power: f32,
        sample_count: usize,
        current_battery_percent: f32,
        battery_capacity_wh: f32,
    ) -> bool {
        self.sample_count = sample_count;
        self.avg_power_watts = Some(avg_power);

        if avg_power < 0.1 {
            // Power too low to calculate meaningful forecast
            self.forecast_duration = None;
            return false;
        }

        // Calculate remaining energy
        let remaining_wh = battery_capacity_wh * (current_battery_percent / 100.0);

        // Calculate time remaining: hours = Wh / W
        let hours_remaining = remaining_wh / avg_power;

        // Sanity check: cap at 24 hours
        if hours_remaining > 0.0 && hours_remaining < 24.0 {
            let secs = (hours_remaining * 3600.0) as u64;
            self.forecast_duration = Some(Duration::from_secs(secs));
            true
        } else {
            self.forecast_duration = None;
            false
        }
    }

    fn clear_forecast(&mut self) {
        self.forecast_duration = None;
        self.avg_power_watts = None;
        self.sample_count = 0;
        self.last_sample_timestamp = None;
    }

    pub fn formatted(&self) -> Option<String> {
        self.forecast_duration.map(|d| {
            let total_mins = d.as_secs() / 60;
            if total_mins == 0 {
                return "< 1m".to_string();
            }
            let hours = total_mins / 60;
            let mins = total_mins % 60;

            if hours > 0 {
                format!("{}h {}m", hours, mins)
            } else {
                format!("{}m", mins)
            }
        })
    }

    pub fn source(&self) -> ForecastSource {
        self.source
    }

    #[cfg(test)]
    pub fn has_forecast(&self) -> bool {
        self.forecast_duration.is_some()
    }

    #[cfg(test)]
    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    #[cfg(test)]
    pub fn duration(&self) -> Option<Duration> {
        self.forecast_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forecast_calculation() {
        let mut forecast = ForecastData::new();

        // Create test data points
        let points = vec![
            DataPoint {
                battery_percent: 80.0,
                power_watts: 10.0,
            },
            DataPoint {
                battery_percent: 79.0,
                power_watts: 12.0,
            },
            DataPoint {
                battery_percent: 78.0,
                power_watts: 11.0,
            },
        ];

        // 100 Wh battery at 50%
        let result = forecast.calculate_from_session_data(&points, 50.0, 100.0);

        assert!(result);
        assert!(forecast.has_forecast());
        assert_eq!(forecast.sample_count(), 3);

        // Average power: 11W, Remaining: 50Wh
        // Expected: 50/11 = ~4.5 hours
        let duration = forecast.duration().unwrap();
        let hours = duration.as_secs() as f32 / 3600.0;
        assert!(hours > 4.0 && hours < 5.0);
    }

    #[test]
    fn test_insufficient_samples() {
        let mut forecast = ForecastData::new();

        let points = vec![DataPoint {
            battery_percent: 80.0,
            power_watts: 10.0,
        }];

        let result = forecast.calculate_from_session_data(&points, 50.0, 100.0);

        assert!(!result);
        assert!(!forecast.has_forecast());
    }
}
