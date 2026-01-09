//! Application core module.
//!
//! This module contains the main App struct and related types for the TUI application.

mod actions;
mod daemon;
mod history;
mod process;
mod theme;
mod tui;
pub mod types;

pub use tui::run_tui;

use std::time::Duration;

use color_eyre::eyre::Result;
use tracing::{debug, info};

use crate::config::{GraphMetric, RuntimeConfig, UserConfig};
use crate::daemon::CycleSummary;
use crate::daemon::{DaemonClient, DaemonStatus, DataSnapshot, KillSignal};
use crate::data::{
    BatteryData, ChargeSession, DailyCycle, DailyStat, DailyTopProcess, ForecastData, HistoryData,
    HistoryMetric, HourlyStat, PowerData, ProcessData, ProcessInfo, SystemInfo,
};
use jolt_theme::cache::ThemeGroup;
use jolt_theme::NamedTheme;

pub use types::{Action, AppView, HistoryPeriod, SortColumn};
pub use types::{MAX_REFRESH_MS, MIN_REFRESH_MS, REFRESH_STEP_MS};

/// How many ticks between forecast refreshes.
const FORECAST_REFRESH_TICKS: u32 = 10;

/// Interval for checking system theme changes.
const THEME_CHECK_INTERVAL: Duration = Duration::from_secs(2);

/// Main application state for the TUI.
///
/// This struct holds all the runtime state needed to render the TUI and
/// respond to user input. It manages data sources (battery, power, processes),
/// view state, and daemon connectivity.
pub struct App {
    pub config: RuntimeConfig,
    pub view: AppView,
    pub system_info: SystemInfo,
    pub battery: BatteryData,
    pub power: PowerData,
    pub processes: ProcessData,
    pub history: HistoryData,
    pub forecast: ForecastData,
    pub selected_process_index: usize,
    pub process_scroll_offset: usize,
    pub expanded_groups: std::collections::HashSet<u32>,
    pub selection_mode: bool,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub merge_mode: bool,
    pub refresh_ms: u64,
    pub(crate) frozen_processes: Option<Vec<ProcessInfo>>,
    pub(crate) process_to_kill: Option<ProcessInfo>,
    pub kill_signal: KillSignal,
    tick_count: u32,
    pub theme_picker_themes: Vec<NamedTheme>,
    pub theme_picker_index: usize,
    pub(crate) preview_theme_id: Option<String>,
    pub(crate) preview_appearance: Option<bool>,
    pub(crate) theme_picker_from_config: bool,
    pub importer_groups: Vec<ThemeGroup>,
    pub importer_index: usize,
    pub importer_selected: std::collections::HashSet<String>,
    pub importer_filter: String,
    pub importer_loading: bool,
    pub importer_cache_age: Option<String>,
    pub importer_search_focused: bool,
    pub history_period: HistoryPeriod,
    pub history_daily_stats: Vec<DailyStat>,
    pub history_hourly_stats: Vec<HourlyStat>,
    pub history_top_processes: Vec<DailyTopProcess>,
    pub history_loading: bool,
    pub cycle_summary: Option<CycleSummary>,
    pub recent_charge_sessions: Vec<ChargeSession>,
    pub daily_cycles: Vec<DailyCycle>,
    pub daemon_status: Option<DaemonStatus>,
    pub daemon_connected: bool,
    pub settings_selected_item: usize,
    pub(crate) daemon_subscription: Option<DaemonClient>,
    pub(crate) last_snapshot: Option<DataSnapshot>,
    pub using_daemon_data: bool,
    pub(crate) last_daemon_update: Option<std::time::Instant>,
    pub(crate) reconnect_attempts: u32,
    pub(crate) last_reconnect_attempt: Option<std::time::Instant>,
    last_theme_check: std::time::Instant,
}

impl App {
    /// Creates a new App instance with the given user configuration.
    ///
    /// This initializes all data sources, sets up the initial state,
    /// and attempts to connect to the daemon for real-time updates.
    pub fn new(user_config: UserConfig) -> Result<Self> {
        info!(refresh_ms = user_config.refresh_ms, "Initializing app");

        let refresh_ms = user_config.refresh_ms;
        let merge_mode = user_config.merge_mode;
        let graph_metric = match user_config.graph_metric {
            GraphMetric::Power => HistoryMetric::Power,
            GraphMetric::Battery => HistoryMetric::Battery,
            GraphMetric::Split => HistoryMetric::Split,
            GraphMetric::Merged => HistoryMetric::Merged,
        };
        let excluded = user_config
            .effective_excluded_processes()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let config = RuntimeConfig::new(user_config);

        debug!("Data sources initialized");

        let mut app = Self {
            config,
            view: AppView::Main,
            system_info: SystemInfo::new(),
            battery: BatteryData::new()?,
            power: PowerData::new()?,
            processes: ProcessData::with_exclusions(excluded)?,
            history: HistoryData::with_metric(graph_metric),
            forecast: ForecastData::new(),
            selected_process_index: 0,
            process_scroll_offset: 0,
            expanded_groups: std::collections::HashSet::new(),
            selection_mode: false,
            sort_column: SortColumn::default(),
            sort_ascending: false,
            merge_mode,
            refresh_ms,
            frozen_processes: None,
            process_to_kill: None,
            kill_signal: KillSignal::default(),
            tick_count: 0,
            theme_picker_themes: Vec::new(),
            theme_picker_index: 0,
            preview_theme_id: None,
            preview_appearance: None,
            theme_picker_from_config: false,
            importer_groups: Vec::new(),
            importer_index: 0,
            importer_selected: std::collections::HashSet::new(),
            importer_filter: String::new(),
            importer_loading: false,
            importer_cache_age: None,
            importer_search_focused: false,
            history_period: HistoryPeriod::default(),
            history_daily_stats: Vec::new(),
            history_hourly_stats: Vec::new(),
            history_top_processes: Vec::new(),
            history_loading: false,
            cycle_summary: None,
            recent_charge_sessions: Vec::new(),
            daily_cycles: Vec::new(),
            daemon_status: None,
            daemon_connected: false,
            settings_selected_item: crate::settings::first_selectable_index(),
            daemon_subscription: None,
            last_snapshot: None,
            using_daemon_data: false,
            last_daemon_update: None,
            reconnect_attempts: 0,
            last_reconnect_attempt: None,
            last_theme_check: std::time::Instant::now(),
        };

        app.try_connect_daemon();

        Ok(app)
    }

    /// Performs a single tick of the application loop.
    ///
    /// This method:
    /// - Checks for system theme changes
    /// - Updates data from the daemon or local sources
    /// - Records history data points
    /// - Refreshes the forecast periodically
    ///
    /// Returns `Ok(true)` if the UI should be redrawn, `Ok(false)` otherwise.
    pub fn tick(&mut self) -> Result<bool> {
        let theme_changed = if self.last_theme_check.elapsed() >= THEME_CHECK_INTERVAL {
            let was_dark = self.config.is_dark_mode();
            self.config.refresh_system_theme();
            self.last_theme_check = std::time::Instant::now();
            let changed = was_dark != self.config.is_dark_mode();
            if changed {
                debug!(
                    was_dark,
                    now_dark = self.config.is_dark_mode(),
                    "Theme mode changed"
                );
            }
            changed
        } else {
            false
        };

        let data_updated = if self.using_daemon_data {
            self.tick_from_daemon()?
        } else {
            self.tick_from_local()?;
            true
        };

        if data_updated {
            self.tick_count = self.tick_count.wrapping_add(1);

            self.history.record(
                self.battery.charge_percent(),
                self.power.total_power_watts(),
                self.battery.temperature_c(),
            );

            if self.tick_count.is_multiple_of(FORECAST_REFRESH_TICKS) {
                self.refresh_forecast();
            }
        }

        debug!(
            data_updated,
            theme_changed,
            using_daemon = self.using_daemon_data,
            tick_count = self.tick_count,
            "App::tick result"
        );

        Ok(data_updated || theme_changed)
    }

    /// Updates data from local sources (battery, power, processes).
    ///
    /// This is used when not connected to the daemon, or as a fallback
    /// when daemon data is unavailable.
    fn tick_from_local(&mut self) -> Result<()> {
        let start = std::time::Instant::now();

        self.battery.refresh()?;
        let battery_time = start.elapsed();

        self.power.refresh()?;
        let power_time = start.elapsed() - battery_time;

        if !self.selection_mode {
            self.processes.refresh()?;
        }
        let process_time = start.elapsed() - battery_time - power_time;

        debug!(
            battery_ms = battery_time.as_millis() as u64,
            power_ms = power_time.as_millis() as u64,
            process_ms = process_time.as_millis() as u64,
            total_ms = start.elapsed().as_millis() as u64,
            battery_percent = self.battery.charge_percent(),
            battery_state = self.battery.state_label(),
            external_connected = self.battery.external_connected(),
            "Local data refresh completed"
        );

        Ok(())
    }

    /// Refreshes the battery forecast based on recent usage patterns.
    ///
    /// This uses daemon samples if available, falling back to session
    /// history data if not.
    fn refresh_forecast(&mut self) {
        use crate::data::battery::ChargeState;
        use crate::data::history::DataPoint;

        if self.battery.state() != ChargeState::Discharging {
            return;
        }

        let battery_percent = self.battery.charge_percent();
        let battery_capacity_wh = self.battery.max_capacity_wh();

        let forecast_window = self.config.user_config.forecast_window_secs;
        if let Ok(mut client) = DaemonClient::connect() {
            if let Ok(samples) = client.get_recent_samples(forecast_window) {
                let converted: Vec<crate::data::Sample> =
                    samples.into_iter().map(Into::into).collect();
                if self.forecast.calculate_from_daemon_samples(
                    &converted,
                    battery_percent,
                    battery_capacity_wh,
                    forecast_window as i64,
                ) {
                    return;
                }
            }
        }

        let points: Vec<DataPoint> = self.history.points.iter().copied().collect();
        self.forecast
            .calculate_from_session_data(&points, battery_percent, battery_capacity_wh);
    }

    /// Moves the settings selection up, skipping section headers.
    pub fn move_settings_selection_up(&mut self) {
        if self.settings_selected_item == 0 {
            return;
        }
        let mut new_index = self.settings_selected_item - 1;
        while new_index > 0 && crate::settings::is_section_header(new_index) {
            new_index -= 1;
        }
        if !crate::settings::is_section_header(new_index) {
            self.settings_selected_item = new_index;
        }
    }

    /// Moves the settings selection down, skipping section headers.
    pub fn move_settings_selection_down(&mut self) {
        let max_index = crate::settings::row_count().saturating_sub(1);
        if self.settings_selected_item >= max_index {
            return;
        }
        let mut new_index = self.settings_selected_item + 1;
        while new_index < max_index && crate::settings::is_section_header(new_index) {
            new_index += 1;
        }
        if !crate::settings::is_section_header(new_index) {
            self.settings_selected_item = new_index;
        }
    }

    /// Performs cleanup before the application exits.
    ///
    /// This optionally shuts down the daemon if background recording is disabled,
    /// and unsubscribes from daemon updates.
    pub fn cleanup(&mut self) {
        if !self.config.user_config.history.background_recording {
            if let Some(ref mut client) = self.daemon_subscription {
                let _ = client.shutdown();
            } else if let Ok(mut client) = DaemonClient::connect() {
                let _ = client.shutdown();
            }
        }

        if let Some(ref mut client) = self.daemon_subscription {
            let _ = client.unsubscribe();
        }
        self.daemon_subscription = None;
    }
}
