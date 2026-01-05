use std::collections::VecDeque;
use chrono::{DateTime, Local};

const MAX_HISTORY_POINTS: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryMetric {
    Battery,
    Power,
}

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub timestamp: DateTime<Local>,
    pub battery_percent: f32,
    pub power_watts: f32,
}

pub struct HistoryData {
    pub points: VecDeque<DataPoint>,
    pub current_metric: HistoryMetric,
}

impl HistoryData {
    pub fn new() -> Self {
        Self {
            points: VecDeque::with_capacity(MAX_HISTORY_POINTS),
            current_metric: HistoryMetric::Battery,
        }
    }

    pub fn record(&mut self, battery_percent: f32, power_watts: f32) {
        let point = DataPoint {
            timestamp: Local::now(),
            battery_percent,
            power_watts,
        };

        if self.points.len() >= MAX_HISTORY_POINTS {
            self.points.pop_front();
        }

        self.points.push_back(point);
    }

    pub fn toggle_metric(&mut self) {
        self.current_metric = match self.current_metric {
            HistoryMetric::Battery => HistoryMetric::Power,
            HistoryMetric::Power => HistoryMetric::Battery,
        };
    }

    pub fn metric_label(&self) -> &'static str {
        match self.current_metric {
            HistoryMetric::Battery => "Battery %",
            HistoryMetric::Power => "Power (W)",
        }
    }

    pub fn current_values(&self) -> Vec<(f64, f64)> {
        let values: Vec<f32> = match self.current_metric {
            HistoryMetric::Battery => self.points.iter().map(|p| p.battery_percent).collect(),
            HistoryMetric::Power => self.points.iter().map(|p| p.power_watts).collect(),
        };

        values
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v as f64))
            .collect()
    }

    pub fn value_range(&self) -> (f64, f64) {
        match self.current_metric {
            HistoryMetric::Battery => (0.0, 100.0),
            HistoryMetric::Power => {
                let max = self
                    .points
                    .iter()
                    .map(|p| p.power_watts)
                    .fold(20.0_f32, f32::max);
                (0.0, (max * 1.2) as f64)
            }
        }
    }

    pub fn latest(&self) -> Option<&DataPoint> {
        self.points.back()
    }

    pub fn average_power(&self) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.points.iter().map(|p| p.power_watts).sum();
        sum / self.points.len() as f32
    }

    pub fn battery_trend(&self) -> Option<f32> {
        if self.points.len() < 10 {
            return None;
        }

        let recent: Vec<f32> = self
            .points
            .iter()
            .rev()
            .take(10)
            .map(|p| p.battery_percent)
            .collect();

        let first = recent.last()?;
        let last = recent.first()?;

        Some(last - first)
    }
}
