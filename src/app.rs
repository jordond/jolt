use color_eyre::eyre::Result;

use crate::config::{GraphMetric, RuntimeConfig, UserConfig};
use crate::data::{
    BatteryData, HistoryData, HistoryMetric, PowerData, ProcessData, ProcessInfo, SystemInfo,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    None,
}

const MIN_REFRESH_MS: u64 = 500;
const MAX_REFRESH_MS: u64 = 10000;
const REFRESH_STEP_MS: u64 = 500;

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
                self.enter_selection_mode();
                self.selected_process_index = self.selected_process_index.saturating_sub(10);
                self.adjust_scroll();
            }
            Action::PageDown => {
                self.enter_selection_mode();
                let visible_count = self.visible_process_count();
                if visible_count > 0 {
                    self.selected_process_index =
                        (self.selected_process_index + 10).min(visible_count - 1);
                    self.adjust_scroll();
                }
            }
            Action::Home => {
                self.enter_selection_mode();
                self.selected_process_index = 0;
                self.process_scroll_offset = 0;
            }
            Action::End => {
                self.enter_selection_mode();
                let visible_count = self.visible_process_count();
                if visible_count > 0 {
                    self.selected_process_index = visible_count - 1;
                    self.adjust_scroll();
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
}
