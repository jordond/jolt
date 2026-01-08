use chrono::Utc;

use crate::data::history_store::{ChargeSession, SessionType};
use crate::data::BatteryData;

const HIGH_SOC_THRESHOLD: f32 = 80.0;
const MIN_SESSION_DURATION_SECS: i64 = 60;

/// Events emitted when charge/discharge state changes.
#[derive(Debug, Clone)]
pub enum SessionEvent {
    Started(ChargeSession),
    Ended(ChargeSession),
}

/// Tracks battery charge/discharge sessions by detecting state transitions.
///
/// Monitors charging state changes and emits `SessionEvent`s when sessions
/// start or end. Also tracks partial discharge cycles and time at high SOC.
pub struct SessionTracker {
    current_session: Option<ChargeSession>,
    last_is_charging: Option<bool>,
    last_external_connected: Option<bool>,
    last_battery_percent: Option<f32>,
    accumulated_discharge_percent: f32,
    session_start_capacity_wh: Option<f32>,
    power_samples: Vec<f32>,
    time_at_high_soc_secs: i64,
    last_sample_time: Option<i64>,
}

impl SessionTracker {
    pub fn new() -> Self {
        Self {
            current_session: None,
            last_is_charging: None,
            last_external_connected: None,
            last_battery_percent: None,
            accumulated_discharge_percent: 0.0,
            session_start_capacity_wh: None,
            power_samples: Vec::new(),
            time_at_high_soc_secs: 0,
            last_sample_time: None,
        }
    }

    /// Restores tracker state from an incomplete session (e.g., after app restart).
    pub fn with_incomplete_session(session: ChargeSession) -> Self {
        let mut tracker = Self::new();
        tracker.last_is_charging = Some(matches!(session.session_type, SessionType::Charge));
        tracker.last_battery_percent = Some(session.start_percent);
        tracker.current_session = Some(session);
        tracker
    }

    /// Processes a battery sample and returns an event if a session state change occurred.
    pub fn process_sample(&mut self, battery: &BatteryData) -> Option<SessionEvent> {
        let now = Utc::now().timestamp();
        let is_charging = battery.is_charging();
        let external_connected = battery.external_connected();
        let battery_percent = battery.charge_percent();
        let capacity_wh = battery.max_capacity_wh();

        if let Some(last_time) = self.last_sample_time {
            let elapsed = now - last_time;
            if battery_percent >= HIGH_SOC_THRESHOLD {
                self.time_at_high_soc_secs += elapsed;
            }
        }
        self.last_sample_time = Some(now);

        let event = self.detect_state_change(
            now,
            is_charging,
            external_connected,
            battery_percent,
            capacity_wh,
            battery.charger_watts(),
        );

        if !is_charging {
            if let Some(last_percent) = self.last_battery_percent {
                if battery_percent < last_percent {
                    let delta = last_percent - battery_percent;
                    self.accumulated_discharge_percent += delta;
                }
            }
        }

        self.last_is_charging = Some(is_charging);
        self.last_external_connected = Some(external_connected);
        self.last_battery_percent = Some(battery_percent);

        event
    }

    fn detect_state_change(
        &mut self,
        now: i64,
        is_charging: bool,
        external_connected: bool,
        battery_percent: f32,
        capacity_wh: f32,
        charger_watts: Option<u32>,
    ) -> Option<SessionEvent> {
        let was_charging = self.last_is_charging.unwrap_or(false);
        let was_external = self.last_external_connected.unwrap_or(false);

        if is_charging && !was_charging {
            if let Some(session) = self.end_current_session(now, battery_percent, capacity_wh) {
                self.start_charge_session(now, battery_percent, capacity_wh, charger_watts);
                return Some(SessionEvent::Ended(session));
            }
            self.start_charge_session(now, battery_percent, capacity_wh, charger_watts);
            return self.current_session.clone().map(SessionEvent::Started);
        }

        if !is_charging && was_charging {
            if let Some(session) = self.end_current_session(now, battery_percent, capacity_wh) {
                self.start_discharge_session(now, battery_percent, capacity_wh);
                return Some(SessionEvent::Ended(session));
            }
        }

        if !external_connected && was_external && !was_charging && self.current_session.is_none() {
            self.start_discharge_session(now, battery_percent, capacity_wh);
            return self.current_session.clone().map(SessionEvent::Started);
        }

        None
    }

    fn start_charge_session(
        &mut self,
        now: i64,
        battery_percent: f32,
        capacity_wh: f32,
        charger_watts: Option<u32>,
    ) {
        self.current_session = Some(ChargeSession::new_charge(
            now,
            battery_percent,
            charger_watts,
        ));
        self.session_start_capacity_wh = Some(capacity_wh);
        self.power_samples.clear();
    }

    fn start_discharge_session(&mut self, now: i64, battery_percent: f32, capacity_wh: f32) {
        self.current_session = Some(ChargeSession::new_discharge(now, battery_percent));
        self.session_start_capacity_wh = Some(capacity_wh);
        self.power_samples.clear();
    }

    fn end_current_session(
        &mut self,
        now: i64,
        battery_percent: f32,
        capacity_wh: f32,
    ) -> Option<ChargeSession> {
        let session = self.current_session.take()?;

        let duration = now - session.start_time;
        if duration < MIN_SESSION_DURATION_SECS {
            return None;
        }

        let energy_wh = self.calculate_energy_wh(
            session.start_percent,
            battery_percent,
            self.session_start_capacity_wh.unwrap_or(capacity_wh),
            session.session_type,
        );

        let avg_power = if !self.power_samples.is_empty() {
            let sum: f32 = self.power_samples.iter().sum();
            Some(sum / self.power_samples.len() as f32)
        } else {
            None
        };

        let completed = ChargeSession {
            id: session.id,
            start_time: session.start_time,
            end_time: Some(now),
            start_percent: session.start_percent,
            end_percent: Some(battery_percent),
            energy_wh,
            charger_watts: session.charger_watts,
            avg_power_watts: avg_power,
            session_type: session.session_type,
            is_complete: true,
        };

        self.session_start_capacity_wh = None;
        self.power_samples.clear();

        Some(completed)
    }

    fn calculate_energy_wh(
        &self,
        start_percent: f32,
        end_percent: f32,
        capacity_wh: f32,
        session_type: SessionType,
    ) -> Option<f32> {
        let delta = match session_type {
            SessionType::Charge => end_percent - start_percent,
            SessionType::Discharge => start_percent - end_percent,
        };

        if delta <= 0.0 {
            return None;
        }

        Some((delta / 100.0) * capacity_wh)
    }

    pub fn record_power_sample(&mut self, power_watts: f32) {
        if self.current_session.is_some() {
            self.power_samples.push(power_watts);
        }
    }

    pub fn get_partial_cycles(&self) -> f32 {
        self.accumulated_discharge_percent / 100.0
    }

    pub fn reset_partial_cycles(&mut self) {
        self.accumulated_discharge_percent = 0.0;
    }

    pub fn reset_time_at_high_soc(&mut self) {
        self.time_at_high_soc_secs = 0;
    }

    #[allow(dead_code)]
    pub fn get_time_at_high_soc_secs(&self) -> i64 {
        self.time_at_high_soc_secs
    }
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_cycle_accumulation() {
        let mut tracker = SessionTracker::new();

        tracker.last_is_charging = Some(false);
        tracker.last_external_connected = Some(false);
        tracker.last_battery_percent = Some(100.0);
        tracker.last_sample_time = Some(Utc::now().timestamp() - 60);

        tracker.accumulated_discharge_percent = 0.0;

        tracker.last_battery_percent = Some(50.0);
        let delta = 100.0 - 50.0;
        tracker.accumulated_discharge_percent += delta;

        assert!((tracker.get_partial_cycles() - 0.5).abs() < 0.01);

        let delta2 = 50.0 - 0.0;
        tracker.accumulated_discharge_percent += delta2;

        assert!((tracker.get_partial_cycles() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_partial_cycle_reset() {
        let mut tracker = SessionTracker::new();
        tracker.accumulated_discharge_percent = 75.0;

        assert!((tracker.get_partial_cycles() - 0.75).abs() < 0.01);

        tracker.reset_partial_cycles();
        assert!((tracker.get_partial_cycles()).abs() < 0.01);
    }

    #[test]
    fn test_energy_calculation() {
        let tracker = SessionTracker::new();

        let energy = tracker.calculate_energy_wh(20.0, 80.0, 50.0, SessionType::Charge);
        assert!((energy.unwrap() - 30.0).abs() < 0.1);

        let energy = tracker.calculate_energy_wh(80.0, 20.0, 50.0, SessionType::Discharge);
        assert!((energy.unwrap() - 30.0).abs() < 0.1);

        let energy = tracker.calculate_energy_wh(80.0, 20.0, 50.0, SessionType::Charge);
        assert!(energy.is_none());
    }
}
