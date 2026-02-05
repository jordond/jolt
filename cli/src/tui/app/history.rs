//! History view methods for App.
//!
//! This module contains methods for loading and managing history data,
//! including daily/hourly stats, cycle summaries, and charge sessions.

use crate::daemon::DaemonClient;

use super::types::HistoryPeriod;
use super::App;

impl App {
    /// Loads history data from the daemon for the current period.
    ///
    /// This method fetches daily stats, top processes, cycle summary,
    /// daily cycles, charge sessions, and hourly stats (for Today period).
    /// If the daemon is not connected, it clears all history data.
    pub(crate) fn load_history_data(&mut self) {
        self.history_loading = true;

        if let Ok(mut client) = DaemonClient::connect() {
            self.daemon_connected = true;

            let (from_date, to_date) = self.get_period_dates();

            if let Ok(daily) = client.get_daily_stats(&from_date, &to_date) {
                self.history_daily_stats = daily.into_iter().map(Into::into).collect();
            }

            if let Ok(top) = client.get_top_processes_range(&from_date, &to_date, 10) {
                self.history_top_processes = top.into_iter().map(Into::into).collect();
            }

            let cycle_days = self.history_period.days();
            if let Ok(summary) = client.get_cycle_summary(cycle_days) {
                self.cycle_summary = Some(summary);
            }

            if let Ok(cycles) = client.get_daily_cycles(&from_date, &to_date) {
                self.daily_cycles = cycles.into_iter().map(Into::into).collect();
            }

            let now = chrono::Utc::now();
            let session_window_days = self.history_period.days() as i64;
            let session_from = (now - chrono::Duration::days(session_window_days)).timestamp();
            if let Ok(sessions) = client.get_charge_sessions(session_from, now.timestamp()) {
                self.recent_charge_sessions = sessions.into_iter().map(Into::into).collect();
            }

            if self.history_period == HistoryPeriod::Today {
                let start_of_day = now
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp();
                let end_ts = now.timestamp();
                if let Ok(hourly) = client.get_hourly_stats(start_of_day, end_ts) {
                    self.history_hourly_stats = hourly.into_iter().map(Into::into).collect();
                }
            }
        } else {
            self.daemon_connected = false;
            self.history_daily_stats.clear();
            self.history_hourly_stats.clear();
            self.history_top_processes.clear();
            self.cycle_summary = None;
            self.recent_charge_sessions.clear();
            self.daily_cycles.clear();
        }

        self.history_loading = false;
    }

    /// Calculates the date range for the current history period.
    ///
    /// Returns a tuple of (from_date, to_date) as ISO date strings (YYYY-MM-DD).
    /// The to_date is always today, and from_date depends on the selected period:
    /// - Today: same as today
    /// - Week: 7 days ago
    /// - Month: 30 days ago
    /// - All: 1970-01-01 (beginning of time)
    pub(crate) fn get_period_dates(&self) -> (String, String) {
        use chrono::{Duration, Utc};

        let today = Utc::now().format("%Y-%m-%d").to_string();
        let from = match self.history_period {
            HistoryPeriod::Today => today.clone(),
            HistoryPeriod::Week => (Utc::now() - Duration::days(7))
                .format("%Y-%m-%d")
                .to_string(),
            HistoryPeriod::Month => (Utc::now() - Duration::days(30))
                .format("%Y-%m-%d")
                .to_string(),
            HistoryPeriod::All => "1970-01-01".to_string(),
        };
        (from, today)
    }
}
