use color_eyre::eyre::Result;
use tracing::{debug, info};

use crate::config::{GraphMetric, RuntimeConfig, UserConfig};

const FORECAST_REFRESH_TICKS: u32 = 10;
use crate::daemon::{DaemonClient, DaemonStatus, DataSnapshot};
use crate::data::{
    BatteryData, DailyStat, DailyTopProcess, ForecastData, HistoryData, HistoryMetric, HourlyStat,
    PowerData, ProcessData, ProcessInfo, SystemInfo,
};
use crate::theme::cache::ThemeGroup;
use crate::theme::{get_all_themes, NamedTheme, ThemeColors};

fn get_base_process_name(name: &str) -> String {
    let name = name
        .trim_end_matches(" Helper")
        .trim_end_matches(" Helper (Renderer)")
        .trim_end_matches(" Helper (GPU)")
        .trim_end_matches(" Helper (Plugin)")
        .trim_end_matches(" Renderer")
        .trim_end_matches(" (GPU Process)")
        .trim_end_matches(" Web Content");

    if let Some(pos) = name.rfind(" (") {
        if name.ends_with(')') {
            return name[..pos].to_string();
        }
    }

    name.to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    ToggleHelp,
    ToggleAbout,
    ToggleSettings,
    SelectNext,
    SelectPrevious,
    ToggleExpand,
    KillProcess,
    ConfirmKill,
    CancelKill,
    CycleAppearance,
    OpenThemePicker,
    CloseThemePicker,
    SelectTheme,
    TogglePreviewAppearance,
    ToggleGraphView,
    ToggleMerge,
    PageUp,
    PageDown,
    Home,
    End,
    ExitSelectionMode,
    CycleSortColumn,
    ToggleSortDirection,
    IncreaseRefreshRate,
    DecreaseRefreshRate,
    OpenThemeImporter,
    CloseThemeImporter,
    ImporterToggleSelect,
    ImporterPreview,
    ImporterImport,
    ImporterRefresh,
    ImporterToggleSearch,
    ImporterFilterChar(char),
    ImporterFilterBackspace,
    ImporterClearFilter,
    ToggleHistory,
    HistoryPrevPeriod,
    HistoryNextPeriod,
    SettingsToggleValue,
    SettingsIncrement,
    SettingsDecrement,
    None,
}

const MIN_REFRESH_MS: u64 = 500;
const MAX_REFRESH_MS: u64 = 10000;
const REFRESH_STEP_MS: u64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoryPeriod {
    #[default]
    Today,
    Week,
    Month,
    All,
}

impl HistoryPeriod {
    pub fn next(self) -> Self {
        match self {
            HistoryPeriod::Today => HistoryPeriod::Week,
            HistoryPeriod::Week => HistoryPeriod::Month,
            HistoryPeriod::Month => HistoryPeriod::All,
            HistoryPeriod::All => HistoryPeriod::Today,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            HistoryPeriod::Today => HistoryPeriod::All,
            HistoryPeriod::Week => HistoryPeriod::Today,
            HistoryPeriod::Month => HistoryPeriod::Week,
            HistoryPeriod::All => HistoryPeriod::Month,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            HistoryPeriod::Today => "Today",
            HistoryPeriod::Week => "Week",
            HistoryPeriod::Month => "Month",
            HistoryPeriod::All => "All",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    Pid,
    Name,
    Cpu,
    Memory,
    #[default]
    Energy,
}

impl SortColumn {
    pub fn next(self) -> Self {
        match self {
            SortColumn::Pid => SortColumn::Name,
            SortColumn::Name => SortColumn::Cpu,
            SortColumn::Cpu => SortColumn::Memory,
            SortColumn::Memory => SortColumn::Energy,
            SortColumn::Energy => SortColumn::Pid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Main,
    Help,
    About,
    KillConfirm,
    ThemePicker,
    ThemeImporter,
    History,
    Settings,
}

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
    frozen_processes: Option<Vec<ProcessInfo>>,
    process_to_kill: Option<ProcessInfo>,
    tick_count: u32,
    pub theme_picker_themes: Vec<NamedTheme>,
    pub theme_picker_index: usize,
    preview_theme_id: Option<String>,
    preview_appearance: Option<bool>,
    theme_picker_from_config: bool,
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
    pub daemon_status: Option<DaemonStatus>,
    pub daemon_connected: bool,
    pub settings_selected_item: usize,
    daemon_subscription: Option<DaemonClient>,
    last_snapshot: Option<DataSnapshot>,
    pub using_daemon_data: bool,
    last_daemon_update: Option<std::time::Instant>,
    reconnect_attempts: u32,
    last_reconnect_attempt: Option<std::time::Instant>,
}

impl App {
    pub fn new(user_config: UserConfig, refresh_from_cli: bool) -> Result<Self> {
        info!(refresh_ms = user_config.refresh_ms, "Initializing app");

        let refresh_ms = user_config.refresh_ms;
        let merge_mode = user_config.merge_mode;
        let graph_metric = match user_config.graph_metric {
            GraphMetric::Power => HistoryMetric::Power,
            GraphMetric::Battery => HistoryMetric::Battery,
        };
        let excluded = user_config
            .effective_excluded_processes()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let config = RuntimeConfig::new(user_config, refresh_from_cli);

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
            daemon_status: None,
            daemon_connected: false,
            settings_selected_item: 0,
            daemon_subscription: None,
            last_snapshot: None,
            using_daemon_data: false,
            last_daemon_update: None,
            reconnect_attempts: 0,
            last_reconnect_attempt: None,
        };

        app.try_connect_daemon();

        Ok(app)
    }

    fn try_connect_daemon(&mut self) {
        if self.try_subscribe_to_daemon() {
            return;
        }

        if !crate::daemon::is_daemon_running() {
            debug!("Daemon not running, attempting auto-start");
            if self.auto_start_daemon() {
                std::thread::sleep(std::time::Duration::from_millis(500));
                for _ in 0..5 {
                    if self.try_subscribe_to_daemon() {
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
                debug!("Failed to subscribe after auto-start");
            }
        }
    }

    fn try_subscribe_to_daemon(&mut self) -> bool {
        if let Ok(mut client) = DaemonClient::connect() {
            if client.subscribe().is_ok() && client.set_nonblocking(true).is_ok() {
                info!("Subscribed to daemon for real-time data");
                self.daemon_subscription = Some(client);
                self.using_daemon_data = true;
                self.daemon_connected = true;
                return true;
            }
        }
        false
    }

    fn auto_start_daemon(&self) -> bool {
        let Ok(exe) = std::env::current_exe() else {
            debug!("Failed to get current exe path");
            return false;
        };

        match std::process::Command::new(&exe)
            .args(["daemon", "start"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(_) => {
                debug!("Daemon spawn initiated");
                true
            }
            Err(e) => {
                debug!("Failed to spawn daemon: {}", e);
                false
            }
        }
    }

    pub fn low_power_mode(&self) -> bool {
        self.config.user_config.low_power_mode
    }

    pub fn is_data_stale(&self) -> bool {
        if !self.using_daemon_data {
            return false;
        }
        if let Some(last_update) = self.last_daemon_update {
            last_update.elapsed() > std::time::Duration::from_secs(2)
        } else {
            false
        }
    }

    pub fn is_reconnecting(&self) -> bool {
        self.using_daemon_data && self.daemon_subscription.is_none() && self.reconnect_attempts > 0
    }

    pub fn tick(&mut self) -> Result<()> {
        self.tick_count = self.tick_count.wrapping_add(1);

        if self.using_daemon_data {
            self.tick_from_daemon()?;
        } else {
            self.tick_from_local()?;
        }

        self.history.record(
            self.battery.charge_percent(),
            self.power.total_power_watts(),
        );

        if self.tick_count.is_multiple_of(FORECAST_REFRESH_TICKS) {
            self.refresh_forecast();
        }

        Ok(())
    }

    fn tick_from_daemon(&mut self) -> Result<()> {
        let mut received_data = false;

        if let Some(ref mut client) = self.daemon_subscription {
            match client.read_update() {
                Ok(Some(snapshot)) => {
                    self.apply_snapshot(&snapshot);
                    self.last_snapshot = Some(snapshot);
                    self.last_daemon_update = Some(std::time::Instant::now());
                    self.reconnect_attempts = 0;
                    received_data = true;
                }
                Ok(None) => {}
                Err(_) => {
                    debug!("Daemon connection lost");
                    self.daemon_subscription = None;
                    self.daemon_connected = false;
                    self.attempt_reconnect();
                }
            }
        }

        if !received_data {
            if let Some(last_update) = self.last_daemon_update {
                if last_update.elapsed() > std::time::Duration::from_secs(5) {
                    debug!("No daemon data for 5s, attempting reconnect");
                    self.daemon_subscription = None;
                    self.attempt_reconnect();
                }
            }
        }

        if !self.using_daemon_data {
            self.tick_from_local()?;
        }

        Ok(())
    }

    fn attempt_reconnect(&mut self) {
        const MAX_RECONNECT_ATTEMPTS: u32 = 3;
        const RECONNECT_BACKOFF_MS: u64 = 1000;

        if self.reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
            debug!("Max reconnect attempts reached, falling back to local data");
            self.using_daemon_data = false;
            self.daemon_connected = false;
            return;
        }

        let backoff_duration = std::time::Duration::from_millis(
            RECONNECT_BACKOFF_MS * (self.reconnect_attempts + 1) as u64,
        );
        if let Some(last_attempt) = self.last_reconnect_attempt {
            if last_attempt.elapsed() < backoff_duration {
                return;
            }
        }

        self.reconnect_attempts += 1;
        self.last_reconnect_attempt = Some(std::time::Instant::now());
        debug!(
            attempt = self.reconnect_attempts,
            "Attempting daemon reconnect"
        );

        if self.try_subscribe_to_daemon() {
            self.reconnect_attempts = 0;
        }
    }

    fn tick_from_local(&mut self) -> Result<()> {
        self.battery.refresh()?;
        self.power.refresh()?;

        let should_refresh_processes = if self.selection_mode {
            false
        } else if self.low_power_mode() {
            self.tick_count.is_multiple_of(3)
        } else {
            true
        };

        if should_refresh_processes {
            self.processes.refresh()?;
        }

        Ok(())
    }

    fn apply_snapshot(&mut self, snapshot: &DataSnapshot) {
        self.battery.update_from_snapshot(&snapshot.battery);
        self.power.update_from_snapshot(&snapshot.power);

        if !self.selection_mode {
            self.processes
                .update_from_snapshots(snapshot.processes.clone());
        }
    }

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
                if self.forecast.calculate_from_daemon_samples(
                    &samples,
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

    pub fn handle_action(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return false,
            Action::ToggleHelp => {
                self.view = match self.view {
                    AppView::Help => AppView::Main,
                    _ => AppView::Help,
                };
            }
            Action::ToggleAbout => {
                self.view = match self.view {
                    AppView::About => AppView::Main,
                    _ => AppView::About,
                };
            }
            Action::SelectPrevious => {
                if self.view == AppView::ThemePicker {
                    if self.theme_picker_index > 0 {
                        self.theme_picker_index -= 1;
                        self.set_theme_preview();
                    }
                } else if self.view == AppView::ThemeImporter {
                    if self.importer_index > 0 {
                        self.importer_index -= 1;
                    }
                } else if self.view == AppView::Settings {
                    self.move_settings_selection_up();
                } else {
                    self.enter_selection_mode();
                    if self.selected_process_index > 0 {
                        self.selected_process_index -= 1;
                        self.adjust_scroll();
                    }
                }
            }
            Action::SelectNext => {
                if self.view == AppView::ThemePicker {
                    if self.theme_picker_index < self.theme_picker_themes.len().saturating_sub(1) {
                        self.theme_picker_index += 1;
                        self.set_theme_preview();
                    }
                } else if self.view == AppView::ThemeImporter {
                    let filtered_count = self.get_filtered_importer_groups().len();
                    if filtered_count > 0 && self.importer_index < filtered_count - 1 {
                        self.importer_index += 1;
                    }
                } else if self.view == AppView::Settings {
                    self.move_settings_selection_down();
                } else {
                    self.enter_selection_mode();
                    let visible_count = self.visible_process_count();
                    if visible_count > 0 && self.selected_process_index < visible_count - 1 {
                        self.selected_process_index += 1;
                        self.adjust_scroll();
                    }
                }
            }
            Action::ExitSelectionMode => {
                self.exit_selection_mode();
            }
            Action::ToggleExpand => {
                if let Some(process) = self.get_selected_process() {
                    if process.children.is_some() {
                        let pid = process.pid;
                        if self.expanded_groups.contains(&pid) {
                            self.expanded_groups.remove(&pid);
                        } else {
                            self.expanded_groups.insert(pid);
                        }
                    }
                }
            }
            Action::KillProcess => {
                if let Some(process) = self.get_selected_process() {
                    if process.is_killable {
                        self.process_to_kill = Some(process.clone());
                        self.view = AppView::KillConfirm;
                    }
                }
            }
            Action::ConfirmKill => {
                if let Some(ref process) = self.process_to_kill {
                    info!(pid = process.pid, name = %process.name, "Killing process");
                    self.kill_process_impl(process.pid);
                }
                self.process_to_kill = None;
                self.view = AppView::Main;
            }
            Action::CancelKill => {
                self.process_to_kill = None;
                self.view = AppView::Main;
            }
            Action::CycleAppearance => {
                self.config.cycle_appearance();
            }
            Action::OpenThemePicker => {
                self.theme_picker_themes = get_all_themes();
                self.theme_picker_index = self
                    .theme_picker_themes
                    .iter()
                    .position(|t| t.id == self.config.theme_id())
                    .unwrap_or(0);
                self.preview_theme_id = None;
                self.preview_appearance = None;
                self.theme_picker_from_config = false;
                self.view = AppView::ThemePicker;
            }
            Action::CloseThemePicker => {
                let return_to_settings = self.theme_picker_from_config;
                self.preview_theme_id = None;
                self.preview_appearance = None;
                self.theme_picker_from_config = false;
                self.view = if return_to_settings {
                    AppView::Settings
                } else {
                    AppView::Main
                };
            }
            Action::SelectTheme => {
                if let Some(theme) = self.theme_picker_themes.get(self.theme_picker_index) {
                    self.config.set_theme(&theme.id);
                }
                let return_to_settings = self.theme_picker_from_config;
                self.preview_theme_id = None;
                self.preview_appearance = None;
                self.theme_picker_from_config = false;
                self.view = if return_to_settings {
                    AppView::Settings
                } else {
                    AppView::Main
                };
            }
            Action::TogglePreviewAppearance => {
                self.toggle_preview_appearance();
            }
            Action::ToggleGraphView => {
                self.history.toggle_metric();
            }
            Action::ToggleMerge => {
                self.merge_mode = !self.merge_mode;
            }
            Action::PageUp => {
                if self.view == AppView::ThemeImporter {
                    self.importer_index = self.importer_index.saturating_sub(10);
                } else {
                    self.enter_selection_mode();
                    self.selected_process_index = self.selected_process_index.saturating_sub(10);
                    self.adjust_scroll();
                }
            }
            Action::PageDown => {
                if self.view == AppView::ThemeImporter {
                    let filtered_count = self.get_filtered_importer_groups().len();
                    if filtered_count > 0 {
                        self.importer_index = (self.importer_index + 10).min(filtered_count - 1);
                    }
                } else {
                    self.enter_selection_mode();
                    let visible_count = self.visible_process_count();
                    if visible_count > 0 {
                        self.selected_process_index =
                            (self.selected_process_index + 10).min(visible_count - 1);
                        self.adjust_scroll();
                    }
                }
            }
            Action::Home => {
                if self.view == AppView::ThemeImporter {
                    self.importer_index = 0;
                } else {
                    self.enter_selection_mode();
                    self.selected_process_index = 0;
                    self.process_scroll_offset = 0;
                }
            }
            Action::End => {
                if self.view == AppView::ThemeImporter {
                    let filtered_count = self.get_filtered_importer_groups().len();
                    if filtered_count > 0 {
                        self.importer_index = filtered_count - 1;
                    }
                } else {
                    self.enter_selection_mode();
                    let visible_count = self.visible_process_count();
                    if visible_count > 0 {
                        self.selected_process_index = visible_count - 1;
                        self.adjust_scroll();
                    }
                }
            }
            Action::CycleSortColumn => {
                self.sort_column = self.sort_column.next();
            }
            Action::ToggleSortDirection => {
                self.sort_ascending = !self.sort_ascending;
            }
            Action::IncreaseRefreshRate => {
                self.refresh_ms = (self.refresh_ms + REFRESH_STEP_MS).min(MAX_REFRESH_MS);
                if !self.config.refresh_from_cli {
                    self.config.user_config.refresh_ms = self.refresh_ms;
                    let _ = self.config.user_config.save();
                }
            }
            Action::DecreaseRefreshRate => {
                self.refresh_ms = self
                    .refresh_ms
                    .saturating_sub(REFRESH_STEP_MS)
                    .max(MIN_REFRESH_MS);
                if !self.config.refresh_from_cli {
                    self.config.user_config.refresh_ms = self.refresh_ms;
                    let _ = self.config.user_config.save();
                }
            }
            Action::OpenThemeImporter => {
                self.open_theme_importer();
            }
            Action::CloseThemeImporter => {
                self.view = AppView::ThemePicker;
                self.importer_filter.clear();
                self.importer_selected.clear();
                self.importer_search_focused = false;
            }
            Action::ImporterToggleSelect => {
                self.toggle_importer_selection();
            }
            Action::ImporterPreview => {
                self.importer_loading = true;
                self.preview_selected_importer_theme();
                self.importer_loading = false;
            }
            Action::ImporterImport => {
                self.importer_loading = true;
                self.import_selected_themes();
                self.importer_loading = false;
            }
            Action::ImporterRefresh => {
                self.refresh_importer_cache();
            }
            Action::ImporterToggleSearch => {
                self.importer_search_focused = !self.importer_search_focused;
            }
            Action::ImporterFilterChar(c) => {
                if self.importer_search_focused {
                    self.importer_filter.push(c);
                    self.importer_index = 0;
                }
            }
            Action::ImporterFilterBackspace => {
                if self.importer_search_focused {
                    self.importer_filter.pop();
                    self.importer_index = 0;
                }
            }
            Action::ImporterClearFilter => {
                self.importer_filter.clear();
                self.importer_index = 0;
                self.importer_search_focused = false;
            }
            Action::ToggleHistory => {
                self.view = match self.view {
                    AppView::History => AppView::Main,
                    _ => {
                        self.load_history_data();
                        AppView::History
                    }
                };
            }
            Action::HistoryNextPeriod => {
                self.history_period = self.history_period.next();
                self.load_history_data();
            }
            Action::HistoryPrevPeriod => {
                self.history_period = self.history_period.prev();
                self.load_history_data();
            }
            Action::ToggleSettings => {
                self.view = match self.view {
                    AppView::Settings => AppView::Main,
                    _ => {
                        self.refresh_daemon_status();
                        self.settings_selected_item = 1;
                        AppView::Settings
                    }
                };
            }
            Action::SettingsToggleValue => {
                if self.toggle_settings_value() {
                    self.open_theme_picker_from_config();
                }
            }
            Action::SettingsIncrement => {
                if self.increment_settings_value() {
                    self.open_theme_picker_from_config();
                }
            }
            Action::SettingsDecrement => {
                if self.decrement_settings_value() {
                    self.open_theme_picker_from_config();
                }
            }
            Action::None => {}
        }
        true
    }

    pub fn visible_process_count(&self) -> usize {
        self.get_visible_processes().len()
    }

    pub fn get_visible_processes(&self) -> Vec<(ProcessInfo, u8)> {
        let processes = if let Some(ref frozen) = self.frozen_processes {
            frozen.clone()
        } else {
            self.processes.processes.clone()
        };

        let sorted = if self.merge_mode {
            self.merge_processes(processes)
        } else {
            processes
        };

        let mut sorted = sorted;
        let asc = self.sort_ascending;
        match self.sort_column {
            SortColumn::Pid => sorted.sort_by(|a, b| {
                if asc {
                    a.pid.cmp(&b.pid)
                } else {
                    b.pid.cmp(&a.pid)
                }
            }),
            SortColumn::Name => sorted.sort_by(|a, b| {
                let cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                if asc {
                    cmp
                } else {
                    cmp.reverse()
                }
            }),
            SortColumn::Cpu => sorted.sort_by(|a, b| {
                let cmp = a
                    .cpu_usage
                    .partial_cmp(&b.cpu_usage)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if asc {
                    cmp
                } else {
                    cmp.reverse()
                }
            }),
            SortColumn::Memory => sorted.sort_by(|a, b| {
                let cmp = a
                    .memory_mb
                    .partial_cmp(&b.memory_mb)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if asc {
                    cmp
                } else {
                    cmp.reverse()
                }
            }),
            SortColumn::Energy => sorted.sort_by(|a, b| {
                let cmp = a
                    .energy_impact
                    .partial_cmp(&b.energy_impact)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if asc {
                    cmp
                } else {
                    cmp.reverse()
                }
            }),
        }

        let mut visible = Vec::new();
        for process in sorted {
            let pid = process.pid;
            visible.push((process.clone(), 0));

            if self.expanded_groups.contains(&pid) {
                if let Some(children) = &process.children {
                    for child in children {
                        visible.push((child.clone(), 1));
                    }
                }
            }
        }

        visible
    }

    fn merge_processes(&self, processes: Vec<ProcessInfo>) -> Vec<ProcessInfo> {
        use std::collections::HashMap;

        let mut merged: HashMap<String, ProcessInfo> = HashMap::new();

        for mut process in processes {
            let original_name = process.name.clone();
            let base_name = get_base_process_name(&original_name);

            process.children = None;

            if let Some(existing) = merged.get_mut(&base_name) {
                existing.cpu_usage += process.cpu_usage;
                existing.memory_mb += process.memory_mb;
                existing.energy_impact += process.energy_impact;
                existing.disk_read_bytes += process.disk_read_bytes;
                existing.disk_write_bytes += process.disk_write_bytes;
                existing.total_cpu_time_secs += process.total_cpu_time_secs;
                existing.run_time_secs = existing.run_time_secs.max(process.run_time_secs);
                if let Some(ref mut children) = existing.children {
                    children.push(process);
                    existing.name = format!("{} ({})", base_name, children.len());
                }
            } else {
                let group = ProcessInfo {
                    pid: process.pid,
                    parent_pid: process.parent_pid,
                    name: base_name.clone(),
                    command: process.command.clone(),
                    cpu_usage: process.cpu_usage,
                    memory_mb: process.memory_mb,
                    energy_impact: process.energy_impact,
                    is_killable: process.is_killable,
                    children: Some(vec![process.clone()]),
                    disk_read_bytes: process.disk_read_bytes,
                    disk_write_bytes: process.disk_write_bytes,
                    status: process.status,
                    run_time_secs: process.run_time_secs,
                    total_cpu_time_secs: process.total_cpu_time_secs,
                };
                merged.insert(base_name, group);
            }
        }

        merged.into_values().collect()
    }

    pub fn get_selected_process(&self) -> Option<ProcessInfo> {
        let visible = self.get_visible_processes();
        visible
            .get(self.selected_process_index)
            .map(|(p, _)| p.clone())
    }

    pub fn process_to_kill(&self) -> Option<&ProcessInfo> {
        self.process_to_kill.as_ref()
    }

    fn enter_selection_mode(&mut self) {
        if !self.selection_mode {
            self.selection_mode = true;
            self.frozen_processes = Some(self.processes.processes.clone());
        }
    }

    fn exit_selection_mode(&mut self) {
        self.selection_mode = false;
        self.frozen_processes = None;
        self.selected_process_index = 0;
        self.process_scroll_offset = 0;
    }

    fn adjust_scroll(&mut self) {
        const VISIBLE_ROWS: usize = 15;

        if self.selected_process_index < self.process_scroll_offset {
            self.process_scroll_offset = self.selected_process_index;
        } else if self.selected_process_index >= self.process_scroll_offset + VISIBLE_ROWS {
            self.process_scroll_offset = self.selected_process_index - VISIBLE_ROWS + 1;
        }
    }

    pub const SETTINGS_ITEMS: &'static [(&'static str, bool)] = &[
        ("General", true),
        ("Theme", false),
        ("Appearance", false),
        ("Refresh Rate (ms)", false),
        ("Low Power Mode", false),
        ("Display", true),
        ("Show Graph", false),
        ("Merge Mode", false),
        ("Process Count", false),
        ("Energy Threshold", false),
        ("Recording", true),
        ("Background Recording", false),
        ("Sample Interval (s)", false),
        ("Raw Retention (days)", false),
        ("Hourly Retention (days)", false),
        ("Daily Retention (days)", false),
        ("Max Database (MB)", false),
    ];

    pub fn settings_is_section_header(&self, index: usize) -> bool {
        Self::SETTINGS_ITEMS
            .get(index)
            .map(|(_, is_header)| *is_header)
            .unwrap_or(false)
    }

    pub fn settings_item_value(&self, index: usize) -> String {
        let (name, is_header) = match Self::SETTINGS_ITEMS.get(index) {
            Some(item) => item,
            None => return String::new(),
        };

        if *is_header {
            return String::new();
        }

        match *name {
            "Theme" => format!("{} \u{2192}", self.config.theme_name()),
            "Appearance" => self.config.appearance_label().to_string(),
            "Refresh Rate (ms)" => self.refresh_ms.to_string(),
            "Low Power Mode" => if self.config.user_config.low_power_mode {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            "Show Graph" => if self.config.user_config.show_graph {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            "Merge Mode" => if self.merge_mode { "On" } else { "Off" }.to_string(),
            "Process Count" => self.config.user_config.process_count.to_string(),
            "Energy Threshold" => format!("{:.1}", self.config.user_config.energy_threshold),
            "Background Recording" => if self.config.user_config.history.background_recording {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            "Sample Interval (s)" => self
                .config
                .user_config
                .history
                .sample_interval_secs
                .to_string(),
            "Raw Retention (days)" => self
                .config
                .user_config
                .history
                .retention_raw_days
                .to_string(),
            "Hourly Retention (days)" => self
                .config
                .user_config
                .history
                .retention_hourly_days
                .to_string(),
            "Daily Retention (days)" => {
                let days = self.config.user_config.history.retention_daily_days;
                if days == 0 {
                    "Forever".to_string()
                } else {
                    days.to_string()
                }
            }
            "Max Database (MB)" => self.config.user_config.history.max_database_mb.to_string(),
            _ => String::new(),
        }
    }

    pub fn toggle_settings_value(&mut self) -> bool {
        let (name, is_header) = match Self::SETTINGS_ITEMS.get(self.settings_selected_item) {
            Some(item) => item,
            None => return false,
        };

        if *is_header {
            return false;
        }

        match *name {
            "Theme" => return true,
            "Appearance" => self.config.cycle_appearance(),
            "Low Power Mode" => {
                self.config.user_config.low_power_mode = !self.config.user_config.low_power_mode;
                let _ = self.config.user_config.save();
            }
            "Show Graph" => {
                self.config.user_config.show_graph = !self.config.user_config.show_graph;
                let _ = self.config.user_config.save();
            }
            "Merge Mode" => {
                self.merge_mode = !self.merge_mode;
                self.config.user_config.merge_mode = self.merge_mode;
                let _ = self.config.user_config.save();
            }
            "Background Recording" => {
                self.config.user_config.history.background_recording =
                    !self.config.user_config.history.background_recording;
                let _ = self.config.user_config.save();
            }
            _ => {}
        }
        false
    }

    pub fn increment_settings_value(&mut self) -> bool {
        let (name, is_header) = match Self::SETTINGS_ITEMS.get(self.settings_selected_item) {
            Some(item) => item,
            None => return false,
        };

        if *is_header {
            return false;
        }

        match *name {
            "Theme" => return true,
            "Appearance" => self.config.cycle_appearance(),
            "Refresh Rate (ms)" => {
                self.refresh_ms = (self.refresh_ms + REFRESH_STEP_MS).min(MAX_REFRESH_MS);
                if !self.config.refresh_from_cli {
                    self.config.user_config.refresh_ms = self.refresh_ms;
                    let _ = self.config.user_config.save();
                }
            }
            "Process Count" => {
                self.config.user_config.process_count =
                    (self.config.user_config.process_count + 10).min(200);
                let _ = self.config.user_config.save();
            }
            "Energy Threshold" => {
                self.config.user_config.energy_threshold =
                    (self.config.user_config.energy_threshold + 0.5).min(10.0);
                let _ = self.config.user_config.save();
            }
            "Sample Interval (s)" => {
                self.config.user_config.history.sample_interval_secs =
                    (self.config.user_config.history.sample_interval_secs + 10).min(600);
                let _ = self.config.user_config.save();
            }
            "Raw Retention (days)" => {
                self.config.user_config.history.retention_raw_days =
                    (self.config.user_config.history.retention_raw_days + 5).min(365);
                let _ = self.config.user_config.save();
            }
            "Hourly Retention (days)" => {
                self.config.user_config.history.retention_hourly_days =
                    (self.config.user_config.history.retention_hourly_days + 30).min(730);
                let _ = self.config.user_config.save();
            }
            "Daily Retention (days)" => {
                self.config.user_config.history.retention_daily_days =
                    (self.config.user_config.history.retention_daily_days + 30).min(3650);
                let _ = self.config.user_config.save();
            }
            "Max Database (MB)" => {
                self.config.user_config.history.max_database_mb =
                    (self.config.user_config.history.max_database_mb + 100).min(10000);
                let _ = self.config.user_config.save();
            }
            _ => {}
        }
        false
    }

    pub fn decrement_settings_value(&mut self) -> bool {
        let (name, is_header) = match Self::SETTINGS_ITEMS.get(self.settings_selected_item) {
            Some(item) => item,
            None => return false,
        };

        if *is_header {
            return false;
        }

        match *name {
            "Theme" => return true,
            "Appearance" => self.config.cycle_appearance(),
            "Refresh Rate (ms)" => {
                self.refresh_ms = self
                    .refresh_ms
                    .saturating_sub(REFRESH_STEP_MS)
                    .max(MIN_REFRESH_MS);
                if !self.config.refresh_from_cli {
                    self.config.user_config.refresh_ms = self.refresh_ms;
                    let _ = self.config.user_config.save();
                }
            }
            "Process Count" => {
                self.config.user_config.process_count = self
                    .config
                    .user_config
                    .process_count
                    .saturating_sub(10)
                    .max(10);
                let _ = self.config.user_config.save();
            }
            "Energy Threshold" => {
                self.config.user_config.energy_threshold =
                    (self.config.user_config.energy_threshold - 0.5).max(0.0);
                let _ = self.config.user_config.save();
            }
            "Sample Interval (s)" => {
                self.config.user_config.history.sample_interval_secs = self
                    .config
                    .user_config
                    .history
                    .sample_interval_secs
                    .saturating_sub(10)
                    .max(10);
                let _ = self.config.user_config.save();
            }
            "Raw Retention (days)" => {
                self.config.user_config.history.retention_raw_days = self
                    .config
                    .user_config
                    .history
                    .retention_raw_days
                    .saturating_sub(5)
                    .max(1);
                let _ = self.config.user_config.save();
            }
            "Hourly Retention (days)" => {
                self.config.user_config.history.retention_hourly_days = self
                    .config
                    .user_config
                    .history
                    .retention_hourly_days
                    .saturating_sub(30);
                let _ = self.config.user_config.save();
            }
            "Daily Retention (days)" => {
                self.config.user_config.history.retention_daily_days = self
                    .config
                    .user_config
                    .history
                    .retention_daily_days
                    .saturating_sub(30);
                let _ = self.config.user_config.save();
            }
            "Max Database (MB)" => {
                self.config.user_config.history.max_database_mb = self
                    .config
                    .user_config
                    .history
                    .max_database_mb
                    .saturating_sub(100)
                    .max(50);
                let _ = self.config.user_config.save();
            }
            _ => {}
        }
        false
    }

    pub fn move_settings_selection_up(&mut self) {
        if self.settings_selected_item == 0 {
            return;
        }
        let mut new_index = self.settings_selected_item - 1;
        while new_index > 0 && self.settings_is_section_header(new_index) {
            new_index -= 1;
        }
        if !self.settings_is_section_header(new_index) {
            self.settings_selected_item = new_index;
        }
    }

    pub fn move_settings_selection_down(&mut self) {
        let max_index = Self::SETTINGS_ITEMS.len().saturating_sub(1);
        if self.settings_selected_item >= max_index {
            return;
        }
        let mut new_index = self.settings_selected_item + 1;
        while new_index < max_index && self.settings_is_section_header(new_index) {
            new_index += 1;
        }
        if !self.settings_is_section_header(new_index) {
            self.settings_selected_item = new_index;
        }
    }

    fn set_theme_preview(&mut self) {
        if let Some(theme) = self.theme_picker_themes.get(self.theme_picker_index) {
            self.preview_theme_id = Some(theme.id.clone());
        }
    }

    fn open_theme_picker_from_config(&mut self) {
        self.theme_picker_themes = get_all_themes();
        self.theme_picker_index = self
            .theme_picker_themes
            .iter()
            .position(|t| t.id == self.config.theme_id())
            .unwrap_or(0);
        self.preview_theme_id = None;
        self.preview_appearance = None;
        self.theme_picker_from_config = true;
        self.view = AppView::ThemePicker;
    }

    fn toggle_preview_appearance(&mut self) {
        let current = self
            .preview_appearance
            .unwrap_or_else(|| self.config.is_dark_mode());
        self.preview_appearance = Some(!current);
    }

    pub fn preview_is_dark(&self) -> bool {
        self.preview_appearance
            .unwrap_or_else(|| self.config.is_dark_mode())
    }

    pub fn current_theme(&self) -> ThemeColors {
        let is_dark = self.preview_is_dark();
        if let Some(ref preview_id) = self.preview_theme_id {
            if let Some(theme) = self
                .theme_picker_themes
                .iter()
                .find(|t| &t.id == preview_id)
            {
                return theme.get_colors(is_dark);
            }
        }
        if self.preview_appearance.is_some() {
            return self.config.theme_with_mode(is_dark);
        }
        self.config.theme()
    }

    fn open_theme_importer(&mut self) {
        use crate::theme::cache::{fetch_and_cache_schemes, get_cached_or_empty};

        let cached = get_cached_or_empty();
        if cached.groups.is_empty() || cached.is_expired() {
            self.importer_loading = true;
            if let Ok(fresh) = fetch_and_cache_schemes(false) {
                let age = fresh.age_description();
                self.importer_groups = fresh.groups;
                self.importer_cache_age = Some(age);
            } else {
                let age = cached.age_description();
                self.importer_groups = cached.groups;
                self.importer_cache_age = Some(age);
            }
            self.importer_loading = false;
        } else {
            let age = cached.age_description();
            self.importer_groups = cached.groups;
            self.importer_cache_age = Some(age);
        }

        self.importer_index = 0;
        self.importer_filter.clear();
        self.importer_selected.clear();
        self.importer_search_focused = false;
        self.view = AppView::ThemeImporter;
    }

    fn refresh_importer_cache(&mut self) {
        use crate::theme::cache::fetch_and_cache_schemes;

        self.importer_loading = true;
        if let Ok(fresh) = fetch_and_cache_schemes(true) {
            let age = fresh.age_description();
            self.importer_groups = fresh.groups;
            self.importer_cache_age = Some(age);
        }
        self.importer_loading = false;
        self.importer_index = 0;
    }

    pub fn get_filtered_importer_groups(&self) -> Vec<&ThemeGroup> {
        if self.importer_filter.is_empty() {
            self.importer_groups.iter().collect()
        } else {
            let filter_lower = self.importer_filter.to_lowercase();
            self.importer_groups
                .iter()
                .filter(|g| g.name.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }

    fn toggle_importer_selection(&mut self) {
        let groups = self.get_filtered_importer_groups();
        if let Some(group) = groups.get(self.importer_index) {
            let name = group.name.clone();
            if self.importer_selected.contains(&name) {
                self.importer_selected.remove(&name);
            } else {
                self.importer_selected.insert(name);
            }
        }
    }

    fn preview_selected_importer_theme(&mut self) {
        use crate::theme::iterm2::import_scheme;

        let group_info: Option<(Option<String>, String)> = {
            let groups = self.get_filtered_importer_groups();
            groups
                .get(self.importer_index)
                .map(|g| (g.dark.clone().or_else(|| g.light.clone()), g.name.clone()))
        };

        if let Some((Some(scheme_name), group_name)) = group_info {
            if let Ok(result) = import_scheme(&scheme_name, Some(&group_name)) {
                self.theme_picker_themes = get_all_themes();
                let new_id = result
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&group_name)
                    .to_string();

                self.theme_picker_index = self
                    .theme_picker_themes
                    .iter()
                    .position(|t| t.id == new_id)
                    .unwrap_or(0);

                self.preview_theme_id = Some(new_id);
            }
        }
    }

    fn import_selected_themes(&mut self) {
        use crate::theme::iterm2::import_scheme;

        if self.importer_selected.is_empty() {
            self.preview_selected_importer_theme();
            return;
        }

        for group_name in self.importer_selected.clone() {
            if let Some(group) = self.importer_groups.iter().find(|g| g.name == group_name) {
                let scheme_name = group.dark.as_ref().or(group.light.as_ref());
                if let Some(name) = scheme_name {
                    let _ = import_scheme(name, Some(&group.name));
                }
            }
        }

        self.theme_picker_themes = get_all_themes();
        self.importer_selected.clear();
        self.view = AppView::ThemePicker;
    }

    fn load_history_data(&mut self) {
        self.history_loading = true;

        if let Ok(mut client) = DaemonClient::connect() {
            self.daemon_connected = true;

            let (from_date, to_date) = self.get_period_dates();

            if let Ok(daily) = client.get_daily_stats(&from_date, &to_date) {
                self.history_daily_stats = daily;
            }

            if let Ok(top) = client.get_top_processes_range(&from_date, &to_date, 10) {
                self.history_top_processes = top;
            }

            if self.history_period == HistoryPeriod::Today {
                let now = chrono::Utc::now();
                let start_of_day = now
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp();
                let end_ts = now.timestamp();
                if let Ok(hourly) = client.get_hourly_stats(start_of_day, end_ts) {
                    self.history_hourly_stats = hourly;
                }
            }
        } else {
            self.daemon_connected = false;
            self.history_daily_stats.clear();
            self.history_hourly_stats.clear();
            self.history_top_processes.clear();
        }

        self.history_loading = false;
    }

    fn get_period_dates(&self) -> (String, String) {
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

    fn refresh_daemon_status(&mut self) {
        if let Ok(mut client) = DaemonClient::connect() {
            self.daemon_connected = true;
            if let Ok(status) = client.get_status() {
                self.daemon_status = Some(status);
            }
        } else {
            self.daemon_connected = false;
            self.daemon_status = None;
        }
    }

    fn kill_process_impl(&self, pid: u32) {
        if self.using_daemon_data {
            if let Ok(mut client) = DaemonClient::connect() {
                let _ = client.kill_process(pid);
                return;
            }
        }
        let _ = self.processes.kill_process(pid);
    }

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
