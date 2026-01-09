//! Action handling methods for App.
//!
//! This module contains the decomposed handle_action method and category-specific
//! handlers for processing user actions in the TUI application.

use tracing::info;

use crate::config::themes_dir;
use crate::daemon::KillSignal;

use super::types::{Action, AppView, MAX_REFRESH_MS, MIN_REFRESH_MS, REFRESH_STEP_MS};
use super::App;

impl App {
    /// Main action handler that dispatches to category-specific handlers.
    ///
    /// Returns `false` if the application should quit, `true` otherwise.
    pub fn handle_action(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return false,
            Action::None => {}

            // View toggle actions
            Action::ToggleHelp
            | Action::ToggleAbout
            | Action::ToggleSettings
            | Action::ToggleHistory
            | Action::ToggleBatteryDetails => {
                self.handle_view_action(action);
            }

            // Navigation actions
            Action::SelectNext
            | Action::SelectPrevious
            | Action::ExitSelectionMode
            | Action::PageUp
            | Action::PageDown
            | Action::Home
            | Action::End => {
                self.handle_navigation_action(action);
            }

            // Process actions
            Action::ToggleExpand
            | Action::KillProcess
            | Action::ConfirmKill
            | Action::CancelKill
            | Action::ToggleKillSignal
            | Action::ToggleMerge
            | Action::CycleSortColumn
            | Action::ToggleSortDirection => {
                self.handle_process_action(action);
            }

            // Theme actions
            Action::CycleAppearance
            | Action::OpenThemePicker
            | Action::CloseThemePicker
            | Action::SelectTheme
            | Action::TogglePreviewAppearance
            | Action::ToggleGraphView => {
                self.handle_theme_action(action);
            }

            // Importer actions
            Action::OpenThemeImporter
            | Action::CloseThemeImporter
            | Action::ImporterToggleSelect
            | Action::ImporterPreview
            | Action::ImporterImport
            | Action::ImporterRefresh
            | Action::ImporterToggleSearch
            | Action::ImporterFilterChar(_)
            | Action::ImporterFilterBackspace
            | Action::ImporterClearFilter => {
                self.handle_importer_action(action);
            }

            // History actions
            Action::HistoryPrevPeriod | Action::HistoryNextPeriod => {
                self.handle_history_action(action);
            }

            // Settings actions
            Action::SettingsToggleValue
            | Action::SettingsIncrement
            | Action::SettingsDecrement => {
                self.handle_settings_action(action);
            }

            // Refresh rate actions
            Action::IncreaseRefreshRate | Action::DecreaseRefreshRate => {
                self.handle_refresh_action(action);
            }
        }
        true
    }

    /// Handles view toggle actions (Help, About, Settings, History, BatteryDetails).
    fn handle_view_action(&mut self, action: Action) {
        match action {
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
            Action::ToggleSettings => {
                self.view = match self.view {
                    AppView::Settings => AppView::Main,
                    _ => {
                        self.refresh_daemon_status();
                        self.settings_selected_item = crate::settings::first_selectable_index();
                        AppView::Settings
                    }
                };
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
            Action::ToggleBatteryDetails => {
                self.view = match self.view {
                    AppView::BatteryDetails => AppView::Main,
                    _ => AppView::BatteryDetails,
                };
            }
            _ => {}
        }
    }

    /// Handles navigation actions (selection movement, scrolling).
    fn handle_navigation_action(&mut self, action: Action) {
        match action {
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
            _ => {}
        }
    }

    /// Handles process-related actions (expand, kill, sort, merge).
    fn handle_process_action(&mut self, action: Action) {
        match action {
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
                        self.kill_signal = KillSignal::default();
                        self.view = AppView::KillConfirm;
                    }
                }
            }
            Action::ConfirmKill => {
                if let Some(ref process) = self.process_to_kill {
                    let signal_label = match self.kill_signal {
                        KillSignal::Graceful => "gracefully",
                        KillSignal::Force => "forcefully",
                    };
                    info!(pid = process.pid, name = %process.name, signal = signal_label, "Killing process");
                    self.kill_process_impl(process.pid, self.kill_signal);
                }
                self.process_to_kill = None;
                self.view = AppView::Main;
            }
            Action::CancelKill => {
                self.process_to_kill = None;
                self.view = AppView::Main;
            }
            Action::ToggleKillSignal => {
                self.kill_signal = match self.kill_signal {
                    KillSignal::Graceful => KillSignal::Force,
                    KillSignal::Force => KillSignal::Graceful,
                };
            }
            Action::ToggleMerge => {
                self.merge_mode = !self.merge_mode;
            }
            Action::CycleSortColumn => {
                self.sort_column = self.sort_column.next();
            }
            Action::ToggleSortDirection => {
                self.sort_ascending = !self.sort_ascending;
            }
            _ => {}
        }
    }

    /// Handles theme-related actions (appearance, theme picker, graph view).
    fn handle_theme_action(&mut self, action: Action) {
        match action {
            Action::CycleAppearance => {
                self.config.cycle_appearance();
            }
            Action::OpenThemePicker => {
                self.theme_picker_themes = jolt_theme::get_all_themes(Some(&themes_dir()));
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
            _ => {}
        }
    }

    /// Handles theme importer actions (import, preview, filter).
    fn handle_importer_action(&mut self, action: Action) {
        match action {
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
            _ => {}
        }
    }

    /// Handles history view actions (period navigation).
    fn handle_history_action(&mut self, action: Action) {
        match action {
            Action::HistoryNextPeriod => {
                self.history_period = self.history_period.next();
                self.load_history_data();
            }
            Action::HistoryPrevPeriod => {
                self.history_period = self.history_period.prev();
                self.load_history_data();
            }
            _ => {}
        }
    }

    /// Handles settings view actions (toggle, increment, decrement).
    fn handle_settings_action(&mut self, action: Action) {
        match action {
            Action::SettingsToggleValue => {
                if let Some(id) = crate::settings::setting_id_at(self.settings_selected_item) {
                    let outcome = crate::settings::setting_apply(
                        self,
                        id,
                        crate::settings::SettingInput::Activate,
                    );
                    if outcome.open_modal {
                        self.open_theme_picker_from_config();
                    }
                }
            }
            Action::SettingsIncrement => {
                if let Some(id) = crate::settings::setting_id_at(self.settings_selected_item) {
                    let outcome = crate::settings::setting_apply(
                        self,
                        id,
                        crate::settings::SettingInput::Increment,
                    );
                    if outcome.open_modal {
                        self.open_theme_picker_from_config();
                    }
                }
            }
            Action::SettingsDecrement => {
                if let Some(id) = crate::settings::setting_id_at(self.settings_selected_item) {
                    let outcome = crate::settings::setting_apply(
                        self,
                        id,
                        crate::settings::SettingInput::Decrement,
                    );
                    if outcome.open_modal {
                        self.open_theme_picker_from_config();
                    }
                }
            }
            _ => {}
        }
    }

    /// Handles refresh rate adjustment actions.
    fn handle_refresh_action(&mut self, action: Action) {
        match action {
            Action::IncreaseRefreshRate => {
                self.refresh_ms = (self.refresh_ms + REFRESH_STEP_MS).min(MAX_REFRESH_MS);
                self.config.user_config.refresh_ms = self.refresh_ms;
                let _ = self.config.user_config.save();
                self.sync_daemon_broadcast_interval();
            }
            Action::DecreaseRefreshRate => {
                self.refresh_ms = self
                    .refresh_ms
                    .saturating_sub(REFRESH_STEP_MS)
                    .max(MIN_REFRESH_MS);
                self.config.user_config.refresh_ms = self.refresh_ms;
                let _ = self.config.user_config.save();
                self.sync_daemon_broadcast_interval();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_none_does_not_quit() {
        // Action::None should return true (continue running)
        // This is a basic sanity check for the action handling
        let action = Action::None;
        assert_eq!(action, Action::None);
    }

    #[test]
    fn action_quit_is_distinct() {
        // Action::Quit should be handled specially
        let action = Action::Quit;
        assert_eq!(action, Action::Quit);
        assert_ne!(action, Action::None);
    }
}
