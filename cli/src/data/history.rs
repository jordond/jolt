use std::collections::VecDeque;

const MAX_HISTORY_POINTS: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryMetric {
    Power,
    Battery,
    Split,
    Merged,
}

#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub battery_percent: f32,
    pub power_watts: f32,
}

#[derive(Debug, Clone)]
pub struct BatteryChange {
    pub index: usize,
    pub value: f32,
}

pub struct HistoryData {
    pub points: VecDeque<DataPoint>,
    pub current_metric: HistoryMetric,
    pub battery_changes: Vec<BatteryChange>,
    last_battery_percent: Option<f32>,
}

impl HistoryData {
    pub fn with_metric(metric: HistoryMetric) -> Self {
        Self {
            points: VecDeque::with_capacity(MAX_HISTORY_POINTS),
            current_metric: metric,
            battery_changes: Vec::new(),
            last_battery_percent: None,
        }
    }

    pub fn record(&mut self, battery_percent: f32, power_watts: f32) {
        let point = DataPoint {
            battery_percent,
            power_watts,
        };

        if self.points.len() >= MAX_HISTORY_POINTS {
            self.points.pop_front();
            for change in &mut self.battery_changes {
                if change.index > 0 {
                    change.index -= 1;
                }
            }
            self.battery_changes.retain(|c| c.index > 0);
        }

        let current_index = self.points.len();

        if let Some(last) = self.last_battery_percent {
            let diff = (battery_percent - last).abs();
            if diff >= 1.0 {
                self.battery_changes.push(BatteryChange {
                    index: current_index,
                    value: battery_percent,
                });
            }
        }
        self.last_battery_percent = Some(battery_percent);

        self.points.push_back(point);
    }

    pub fn toggle_metric(&mut self) {
        self.current_metric = match self.current_metric {
            HistoryMetric::Power => HistoryMetric::Battery,
            HistoryMetric::Battery => HistoryMetric::Merged,
            HistoryMetric::Merged => HistoryMetric::Split,
            HistoryMetric::Split => HistoryMetric::Power,
        };
    }

    pub fn metric_label(&self) -> &'static str {
        match self.current_metric {
            HistoryMetric::Battery => "Battery %",
            HistoryMetric::Power => "Power (W)",
            HistoryMetric::Split => "Split View",
            HistoryMetric::Merged => "Combined",
        }
    }

    pub fn current_values(&self) -> Vec<(f64, f64)> {
        let values: Vec<f32> = match self.current_metric {
            HistoryMetric::Battery => self.points.iter().map(|p| p.battery_percent).collect(),
            HistoryMetric::Power | HistoryMetric::Split | HistoryMetric::Merged => {
                self.points.iter().map(|p| p.power_watts).collect()
            }
        };

        values
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v as f64))
            .collect()
    }

    pub fn battery_values(&self) -> Vec<(f64, f64)> {
        self.points
            .iter()
            .enumerate()
            .map(|(i, p)| (i as f64, p.battery_percent as f64))
            .collect()
    }

    pub fn power_values(&self) -> Vec<(f64, f64)> {
        self.points
            .iter()
            .enumerate()
            .map(|(i, p)| (i as f64, p.power_watts as f64))
            .collect()
    }

    pub fn value_range(&self) -> (f64, f64) {
        match self.current_metric {
            HistoryMetric::Battery => (0.0, 100.0),
            HistoryMetric::Power | HistoryMetric::Split | HistoryMetric::Merged => {
                let max = self
                    .points
                    .iter()
                    .map(|p| p.power_watts)
                    .fold(20.0_f32, f32::max);
                (0.0, (max * 1.2) as f64)
            }
        }
    }

    pub fn power_range(&self) -> (f64, f64) {
        let max = self
            .points
            .iter()
            .map(|p| p.power_watts)
            .fold(20.0_f32, f32::max);
        (0.0, (max * 1.2) as f64)
    }
}
