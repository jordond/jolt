use super::theme::ThemeColors;
pub(super) use crate::config::{cache_dir, config_dir};
use crate::config::{AppearanceMode, UserConfig};
use jolt_theme::NamedTheme;
use std::path::PathBuf;
use std::process::Command;

pub struct RuntimeConfig {
    pub user_config: UserConfig,
    pub system_is_dark: bool,
    current_theme: NamedTheme,
}

impl RuntimeConfig {
    pub fn new(user_config: UserConfig) -> Self {
        let system_is_dark = detect_system_dark_mode();
        let current_theme = jolt_theme::get_theme_by_id(&user_config.theme, Some(&themes_dir()))
            .unwrap_or_else(|| {
                jolt_theme::get_theme_by_id("default", Some(&themes_dir()))
                    .expect("Default theme must exist")
            });

        Self {
            user_config,
            system_is_dark,
            current_theme,
        }
    }

    pub fn is_dark_mode(&self) -> bool {
        match self.user_config.appearance {
            AppearanceMode::Auto => self.system_is_dark,
            AppearanceMode::Dark => true,
            AppearanceMode::Light => false,
        }
    }

    pub fn theme(&self) -> ThemeColors {
        self.current_theme.get_colors(self.is_dark_mode()).into()
    }

    pub fn theme_with_mode(&self, is_dark: bool) -> ThemeColors {
        self.current_theme.get_colors(is_dark).into()
    }

    pub fn set_theme(&mut self, theme_id: &str) {
        if let Some(theme) = jolt_theme::get_theme_by_id(theme_id, Some(&themes_dir())) {
            self.current_theme = theme;
            self.user_config.theme = theme_id.to_string();
            let _ = self.user_config.save();
        }
    }

    pub fn cycle_appearance(&mut self) {
        self.user_config.appearance = self.user_config.appearance.next();
        let _ = self.user_config.save();
    }

    pub fn appearance_label(&self) -> &'static str {
        self.user_config.appearance.label()
    }

    pub fn theme_name(&self) -> &str {
        &self.current_theme.name
    }

    pub fn theme_id(&self) -> &str {
        &self.current_theme.id
    }

    pub fn refresh_system_theme(&mut self) -> bool {
        if self.user_config.appearance != AppearanceMode::Auto {
            return false;
        }
        let new_is_dark = detect_system_dark_mode();
        if new_is_dark != self.system_is_dark {
            self.system_is_dark = new_is_dark;
            true
        } else {
            false
        }
    }
}

pub fn detect_system_dark_mode() -> bool {
    Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .eq_ignore_ascii_case("dark")
        })
        .unwrap_or(false)
}

pub fn themes_dir() -> PathBuf {
    config_dir().join("themes")
}