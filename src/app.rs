use color_eyre::eyre::Result;

use crate::config::{Config, ThemeMode};
use crate::data::{BatteryData, HistoryData, PowerData, ProcessData, ProcessInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    ToggleHelp,
    SelectNext,
    SelectPrevious,
    ToggleExpand,
    KillProcess,
    ConfirmKill,
    CancelKill,
    CycleTheme,
    ToggleGraphView,
    PageUp,
    PageDown,
    Home,
    End,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Main,
    Help,
    KillConfirm,
}

pub struct App {
    pub config: Config,
    pub view: AppView,
    pub battery: BatteryData,
    pub power: PowerData,
    pub processes: ProcessData,
    pub history: HistoryData,
    pub selected_process_index: usize,
    pub process_scroll_offset: usize,
    pub expanded_groups: std::collections::HashSet<u32>,
    process_to_kill: Option<ProcessInfo>,
}

impl App {
    pub fn new(theme_mode: ThemeMode) -> Result<Self> {
        let config = Config::new(theme_mode);

        Ok(Self {
            config,
            view: AppView::Main,
            battery: BatteryData::new()?,
            power: PowerData::new()?,
            processes: ProcessData::new()?,
            history: HistoryData::new(),
            selected_process_index: 0,
            process_scroll_offset: 0,
            expanded_groups: std::collections::HashSet::new(),
            process_to_kill: None,
        })
    }

    pub fn tick(&mut self) -> Result<()> {
        self.battery.refresh()?;
        self.power.refresh()?;
        self.processes.refresh()?;

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
            Action::SelectPrevious => {
                if self.selected_process_index > 0 {
                    self.selected_process_index -= 1;
                    self.adjust_scroll();
                }
            }
            Action::SelectNext => {
                let visible_count = self.visible_process_count();
                if visible_count > 0 && self.selected_process_index < visible_count - 1 {
                    self.selected_process_index += 1;
                    self.adjust_scroll();
                }
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
                    self.process_to_kill = Some(process.clone());
                    self.view = AppView::KillConfirm;
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
            Action::CycleTheme => {
                self.config.cycle_theme();
            }
            Action::ToggleGraphView => {
                self.history.toggle_metric();
            }
            Action::PageUp => {
                self.selected_process_index = self.selected_process_index.saturating_sub(10);
                self.adjust_scroll();
            }
            Action::PageDown => {
                let visible_count = self.visible_process_count();
                if visible_count > 0 {
                    self.selected_process_index =
                        (self.selected_process_index + 10).min(visible_count - 1);
                    self.adjust_scroll();
                }
            }
            Action::Home => {
                self.selected_process_index = 0;
                self.process_scroll_offset = 0;
            }
            Action::End => {
                let visible_count = self.visible_process_count();
                if visible_count > 0 {
                    self.selected_process_index = visible_count - 1;
                    self.adjust_scroll();
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
        let mut visible = Vec::new();

        for process in &self.processes.processes {
            visible.push((process.clone(), 0));

            if self.expanded_groups.contains(&process.pid) {
                if let Some(children) = &process.children {
                    for child in children {
                        visible.push((child.clone(), 1));
                    }
                }
            }
        }

        visible
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

    fn adjust_scroll(&mut self) {
        const VISIBLE_ROWS: usize = 15;

        if self.selected_process_index < self.process_scroll_offset {
            self.process_scroll_offset = self.selected_process_index;
        } else if self.selected_process_index >= self.process_scroll_offset + VISIBLE_ROWS {
            self.process_scroll_offset = self.selected_process_index - VISIBLE_ROWS + 1;
        }
    }
}
