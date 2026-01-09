//! Theme picker and importer methods for App.
//!
//! This module contains methods for the theme picker dialog,
//! theme preview functionality, and the theme importer interface.

use crate::config::{cache_dir, themes_dir};
use crate::theme::ThemeColors;
use jolt_theme::cache::ThemeGroup;

use super::types::AppView;
use super::App;

impl App {
    /// Sets the preview theme to the currently selected theme in the picker.
    pub(crate) fn set_theme_preview(&mut self) {
        if let Some(theme) = self.theme_picker_themes.get(self.theme_picker_index) {
            self.preview_theme_id = Some(theme.id.clone());
        }
    }

    /// Opens the theme picker from the settings view.
    /// Loads all available themes and positions the selector on the current theme.
    pub(crate) fn open_theme_picker_from_config(&mut self) {
        self.theme_picker_themes = jolt_theme::get_all_themes(Some(&themes_dir()));
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

    /// Toggles between light and dark preview appearance.
    pub(crate) fn toggle_preview_appearance(&mut self) {
        let current = self
            .preview_appearance
            .unwrap_or_else(|| self.config.is_dark_mode());
        self.preview_appearance = Some(!current);
    }

    /// Returns whether the current preview is in dark mode.
    pub fn preview_is_dark(&self) -> bool {
        self.preview_appearance
            .unwrap_or_else(|| self.config.is_dark_mode())
    }

    /// Returns the current theme colors, accounting for any active preview.
    pub fn current_theme(&self) -> ThemeColors {
        let is_dark = self.preview_is_dark();
        if let Some(ref preview_id) = self.preview_theme_id {
            if let Some(theme) = self
                .theme_picker_themes
                .iter()
                .find(|t| &t.id == preview_id)
            {
                return theme.get_colors(is_dark).into();
            }
        }
        if self.preview_appearance.is_some() {
            return self.config.theme_with_mode(is_dark);
        }
        self.config.theme()
    }

    /// Opens the theme importer view.
    /// Loads cached themes or fetches fresh data if cache is empty/expired.
    pub(crate) fn open_theme_importer(&mut self) {
        let cached = jolt_theme::cache::get_cached_or_empty(&cache_dir());
        if cached.groups.is_empty() || cached.is_expired() {
            self.importer_loading = true;
            if let Ok(fresh) = jolt_theme::cache::fetch_and_cache_schemes(&cache_dir(), false) {
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

    /// Refreshes the importer cache by fetching fresh data from the source.
    pub(crate) fn refresh_importer_cache(&mut self) {
        self.importer_loading = true;
        if let Ok(fresh) = jolt_theme::cache::fetch_and_cache_schemes(&cache_dir(), true) {
            let age = fresh.age_description();
            self.importer_groups = fresh.groups;
            self.importer_cache_age = Some(age);
        }
        self.importer_loading = false;
        self.importer_index = 0;
    }

    /// Returns the filtered list of theme groups based on the current search filter.
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

    /// Toggles the selection state of the currently highlighted theme group.
    pub(crate) fn toggle_importer_selection(&mut self) {
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

    /// Previews the currently highlighted theme from the importer.
    /// Imports the theme temporarily and sets it as the preview.
    pub(crate) fn preview_selected_importer_theme(&mut self) {
        let group_info: Option<(Option<String>, String)> = {
            let groups = self.get_filtered_importer_groups();
            groups
                .get(self.importer_index)
                .map(|g| (g.dark.clone().or_else(|| g.light.clone()), g.name.clone()))
        };

        if let Some((Some(scheme_name), group_name)) = group_info {
            if let Ok(result) =
                jolt_theme::iterm2::import_scheme(&scheme_name, Some(&group_name), &themes_dir())
            {
                self.theme_picker_themes = jolt_theme::get_all_themes(Some(&themes_dir()));
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

    /// Imports the selected themes from the importer.
    /// If no themes are selected, previews the currently highlighted theme.
    pub(crate) fn import_selected_themes(&mut self) {
        if self.importer_selected.is_empty() {
            self.preview_selected_importer_theme();
            return;
        }

        for group_name in self.importer_selected.clone() {
            if let Some(group) = self.importer_groups.iter().find(|g| g.name == group_name) {
                let scheme_name = group.dark.as_ref().or(group.light.as_ref());
                if let Some(name) = scheme_name {
                    let _ =
                        jolt_theme::iterm2::import_scheme(name, Some(&group.name), &themes_dir());
                }
            }
        }

        self.theme_picker_themes = jolt_theme::get_all_themes(Some(&themes_dir()));
        self.importer_selected.clear();
        self.view = AppView::ThemePicker;
    }
}
