use color_eyre::eyre::Result;

use crate::config::{GraphMetric, RuntimeConfig, UserConfig};
use crate::daemon::{DaemonClient, DaemonStatus};
use crate::data::{
    BatteryData, DailyStat, DailyTopProcess, HistoryData, HistoryMetric, HourlyStat, PowerData,
    ProcessData, ProcessInfo, SystemInfo,
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
    ToggleConfig,
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
    ConfigToggleValue,
    ConfigIncrement,
    ConfigDecrement,
    ConfigRevert,
    ConfigDefaults,
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
    ToggleDaemonInfo,
    DaemonStart,
    DaemonStop,
    ToggleHistoryConfig,
    HistoryConfigToggleValue,
    HistoryConfigIncrement,
    HistoryConfigDecrement,
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
    Config,
    ThemePicker,
    ThemeImporter,
    History,
    DaemonInfo,
    HistoryConfig,
}

pub struct App {
    pub config: RuntimeConfig,
    pub view: AppView,
    pub system_info: SystemInfo,
    pub battery: BatteryData,
    pub power: PowerData,
    pub processes: ProcessData,
    pub history: HistoryData,
    pub selected_process_index: usize,
    pub process_scroll_offset: usize,
    pub expanded_groups: std::collections::HashSet<u32>,
    pub selection_mode: bool,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub merge_mode: bool,
    pub refresh_ms: u64,
    pub config_selected_item: usize,
    frozen_processes: Option<Vec<ProcessInfo>>,
    process_to_kill: Option<ProcessInfo>,
    tick_count: u32,
    config_snapshot: Option<(UserConfig, u64, bool)>,
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
    pub history_config_selected_item: usize,
}

impl App {
    pub fn new(user_config: UserConfig, refresh_from_cli: bool) -> Result<Self> {
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

        Ok(Self {
            config,
            view: AppView::Main,
            system_info: SystemInfo::new(),
            battery: BatteryData::new()?,
            power: PowerData::new()?,
            processes: ProcessData::with_exclusions(excluded)?,
            history: HistoryData::with_metric(graph_metric),
            selected_process_index: 0,
            process_scroll_offset: 0,
            expanded_groups: std::collections::HashSet::new(),
            selection_mode: false,
            sort_column: SortColumn::default(),
            sort_ascending: false,
            merge_mode,
            refresh_ms,
            config_selected_item: 0,
            frozen_processes: None,
            process_to_kill: None,
            tick_count: 0,
            config_snapshot: None,
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
            history_config_selected_item: 0,
        })
    }

    pub fn low_power_mode(&self) -> bool {
        self.config.user_config.low_power_mode
    }

    pub fn tick(&mut self) -> Result<()> {
        self.tick_count = self.tick_count.wrapping_add(1);

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

        self.history.record(
            self.battery.charge_percent(),
            self.power.total_power_watts(),
        );

        Ok(())
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
            Action::ToggleConfig => {
                self.view = match self.view {
                    AppView::Config => {
                        self.config_snapshot = None;
                        AppView::Main
                    }
                    _ => {
                        self.config_snapshot = Some((
                            self.config.user_config.clone(),
                            self.refresh_ms,
                            self.merge_mode,
                        ));
                        AppView::Config
                    }
                };
            }
            Action::SelectPrevious => {
                if self.view == AppView::Config {
                    if self.config_selected_item > 0 {
                        self.config_selected_item -= 1;
                    }
                } else if self.view == AppView::ThemePicker {
                    if self.theme_picker_index > 0 {
                        self.theme_picker_index -= 1;
                        self.set_theme_preview();
                    }
                } else if self.view == AppView::ThemeImporter {
                    if self.importer_index > 0 {
                        self.importer_index -= 1;
                    }
                } else if self.view == AppView::HistoryConfig {
                    if self.history_config_selected_item > 0 {
                        self.history_config_selected_item -= 1;
                    }
                } else {
                    self.enter_selection_mode();
                    if self.selected_process_index > 0 {
                        self.selected_process_index -= 1;
                        self.adjust_scroll();
                    }
                }
            }
            Action::SelectNext => {
                if self.view == AppView::Config {
                    if self.config_selected_item < self.config_item_count() - 1 {
                        self.config_selected_item += 1;
                    }
                } else if self.view == AppView::ThemePicker {
                    if self.theme_picker_index < self.theme_picker_themes.len().saturating_sub(1) {
                        self.theme_picker_index += 1;
                        self.set_theme_preview();
                    }
                } else if self.view == AppView::ThemeImporter {
                    let filtered_count = self.get_filtered_importer_groups().len();
                    if filtered_count > 0 && self.importer_index < filtered_count - 1 {
                        self.importer_index += 1;
                    }
                } else if self.view == AppView::HistoryConfig {
                    if self.history_config_selected_item < Self::HISTORY_CONFIG_ITEMS.len() - 1 {
                        self.history_config_selected_item += 1;
                    }
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
                    let _ = self.processes.kill_process(process.pid);
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
                let return_to_config = self.theme_picker_from_config;
                self.preview_theme_id = None;
                self.preview_appearance = None;
                self.theme_picker_from_config = false;
                self.view = if return_to_config {
                    AppView::Config
                } else {
                    AppView::Main
                };
            }
            Action::SelectTheme => {
                if let Some(theme) = self.theme_picker_themes.get(self.theme_picker_index) {
                    self.config.set_theme(&theme.id);
                }
                let return_to_config = self.theme_picker_from_config;
                self.preview_theme_id = None;
                self.preview_appearance = None;
                self.theme_picker_from_config = false;
                self.view = if return_to_config {
                    AppView::Config
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
            Action::ConfigToggleValue => {
                if self.toggle_config_value() {
                    self.open_theme_picker_from_config();
                }
            }
            Action::ConfigIncrement => {
                if self.increment_config_value() {
                    self.open_theme_picker_from_config();
                }
            }
            Action::ConfigDecrement => {
                if self.decrement_config_value() {
                    self.open_theme_picker_from_config();
                }
            }
            Action::ConfigRevert => {
                if let Some((snapshot, refresh, merge)) = self.config_snapshot.take() {
                    self.config.user_config = snapshot.clone();
                    self.refresh_ms = refresh;
                    self.merge_mode = merge;
                    let _ = self.config.user_config.save();
                    self.config_snapshot = Some((snapshot, refresh, merge));
                }
            }
            Action::ConfigDefaults => {
                self.config.user_config = UserConfig::default();
                self.refresh_ms = self.config.user_config.refresh_ms;
                self.merge_mode = self.config.user_config.merge_mode;
                let _ = self.config.user_config.save();
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
            Action::ToggleDaemonInfo => {
                self.view = match self.view {
                    AppView::DaemonInfo => AppView::Main,
                    _ => {
                        self.refresh_daemon_status();
                        AppView::DaemonInfo
                    }
                };
            }
            Action::DaemonStart => {
                self.start_daemon();
            }
            Action::DaemonStop => {
                self.stop_daemon();
            }
            Action::ToggleHistoryConfig => {
                self.view = match self.view {
                    AppView::HistoryConfig => AppView::Main,
                    _ => {
                        self.history_config_selected_item = 0;
                        AppView::HistoryConfig
                    }
                };
            }
            Action::HistoryConfigToggleValue => {
                self.toggle_history_config_value();
            }
            Action::HistoryConfigIncrement => {
                self.increment_history_config_value();
            }
            Action::HistoryConfigDecrement => {
                self.decrement_history_config_value();
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
                    children: Some(vec![process]),
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

    pub const CONFIG_ITEMS: &'static [&'static str] = &[
        "Theme",
        "Appearance",
        "Refresh Rate (ms)",
        "Low Power Mode",
        "Show Graph",
        "Merge Mode",
        "Process Count",
        "Energy Threshold",
    ];

    pub fn config_item_count(&self) -> usize {
        Self::CONFIG_ITEMS.len()
    }

    pub fn config_item_value(&self, index: usize) -> String {
        match index {
            0 => format!("{} →", self.config.theme_name()),
            1 => self.config.appearance_label().to_string(),
            2 => self.refresh_ms.to_string(),
            3 => if self.config.user_config.low_power_mode {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            4 => if self.config.user_config.show_graph {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            5 => if self.merge_mode { "On" } else { "Off" }.to_string(),
            6 => self.config.user_config.process_count.to_string(),
            7 => format!("{:.1}", self.config.user_config.energy_threshold),
            _ => String::new(),
        }
    }

    fn toggle_config_value(&mut self) -> bool {
        match self.config_selected_item {
            0 => return true,
            1 => self.config.cycle_appearance(),
            3 => {
                self.config.user_config.low_power_mode = !self.config.user_config.low_power_mode;
                let _ = self.config.user_config.save();
            }
            4 => {
                self.config.user_config.show_graph = !self.config.user_config.show_graph;
                let _ = self.config.user_config.save();
            }
            5 => {
                self.merge_mode = !self.merge_mode;
                self.config.user_config.merge_mode = self.merge_mode;
                let _ = self.config.user_config.save();
            }
            _ => {}
        }
        false
    }

    fn increment_config_value(&mut self) -> bool {
        match self.config_selected_item {
            0 => return true,
            1 => self.config.cycle_appearance(),
            2 => {
                self.refresh_ms = (self.refresh_ms + REFRESH_STEP_MS).min(MAX_REFRESH_MS);
                if !self.config.refresh_from_cli {
                    self.config.user_config.refresh_ms = self.refresh_ms;
                    let _ = self.config.user_config.save();
                }
            }
            6 => {
                self.config.user_config.process_count =
                    (self.config.user_config.process_count + 10).min(200);
                let _ = self.config.user_config.save();
            }
            7 => {
                self.config.user_config.energy_threshold =
                    (self.config.user_config.energy_threshold + 0.5).min(10.0);
                let _ = self.config.user_config.save();
            }
            _ => {}
        }
        false
    }

    fn decrement_config_value(&mut self) -> bool {
        match self.config_selected_item {
            0 => return true,
            1 => self.config.cycle_appearance(),
            2 => {
                self.refresh_ms = self
                    .refresh_ms
                    .saturating_sub(REFRESH_STEP_MS)
                    .max(MIN_REFRESH_MS);
                if !self.config.refresh_from_cli {
                    self.config.user_config.refresh_ms = self.refresh_ms;
                    let _ = self.config.user_config.save();
                }
            }
            6 => {
                self.config.user_config.process_count = self
                    .config
                    .user_config
                    .process_count
                    .saturating_sub(10)
                    .max(10);
                let _ = self.config.user_config.save();
            }
            7 => {
                self.config.user_config.energy_threshold =
                    (self.config.user_config.energy_threshold - 0.5).max(0.0);
                let _ = self.config.user_config.save();
            }
            _ => {}
        }
        false
    }

    pub const HISTORY_CONFIG_ITEMS: &'static [&'static str] = &[
        "Recording Enabled",
        "Sample Interval (s)",
        "Raw Retention (days)",
        "Hourly Retention (days)",
        "Daily Retention (days)",
        "Max Database (MB)",
    ];

    pub fn history_config_item_value(&self, index: usize) -> String {
        let history = &self.config.user_config.history;
        match index {
            0 => if history.enabled { "On" } else { "Off" }.to_string(),
            1 => history.sample_interval_secs.to_string(),
            2 => history.retention_raw_days.to_string(),
            3 => history.retention_hourly_days.to_string(),
            4 => {
                if history.retention_daily_days == 0 {
                    "Forever".to_string()
                } else {
                    history.retention_daily_days.to_string()
                }
            }
            5 => history.max_database_mb.to_string(),
            _ => String::new(),
        }
    }

    fn toggle_history_config_value(&mut self) {
        if self.history_config_selected_item == 0 {
            self.config.user_config.history.enabled = !self.config.user_config.history.enabled;
            let _ = self.config.user_config.save();
        }
    }

    fn increment_history_config_value(&mut self) {
        let history = &mut self.config.user_config.history;
        match self.history_config_selected_item {
            1 => {
                history.sample_interval_secs = (history.sample_interval_secs + 10).min(600);
            }
            2 => {
                history.retention_raw_days = (history.retention_raw_days + 5).min(365);
            }
            3 => {
                history.retention_hourly_days = (history.retention_hourly_days + 30).min(730);
            }
            4 => {
                history.retention_daily_days = (history.retention_daily_days + 30).min(3650);
            }
            5 => {
                history.max_database_mb = (history.max_database_mb + 100).min(10000);
            }
            _ => return,
        }
        let _ = self.config.user_config.save();
    }

    fn decrement_history_config_value(&mut self) {
        let history = &mut self.config.user_config.history;
        match self.history_config_selected_item {
            1 => {
                history.sample_interval_secs =
                    history.sample_interval_secs.saturating_sub(10).max(10);
            }
            2 => {
                history.retention_raw_days = history.retention_raw_days.saturating_sub(5).max(1);
            }
            3 => {
                history.retention_hourly_days = history.retention_hourly_days.saturating_sub(30);
            }
            4 => {
                history.retention_daily_days = history.retention_daily_days.saturating_sub(30);
            }
            5 => {
                history.max_database_mb = history.max_database_mb.saturating_sub(100).max(50);
            }
            _ => return,
        }
        let _ = self.config.user_config.save();
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

    fn start_daemon(&mut self) {
        if crate::daemon::is_daemon_running() {
            return;
        }

        // Spawn a separate process to start the daemon
        // We can't call run_daemon directly because daemonize causes the parent to exit
        if let Ok(exe) = std::env::current_exe() {
            match std::process::Command::new(exe)
                .args(["daemon", "start"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(_child) => {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    for _ in 0..3 {
                        self.refresh_daemon_status();
                        if self.daemon_connected {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                }
                Err(_) => {
                    self.refresh_daemon_status();
                }
            }
        } else {
            self.refresh_daemon_status();
        }
    }

    fn stop_daemon(&mut self) {
        if let Ok(mut client) = DaemonClient::connect() {
            let _ = client.shutdown();
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        self.daemon_connected = false;
        self.daemon_status = None;
    }
}
