pub mod cache;
pub mod contrast;
pub mod iterm2;
pub mod validation;

use crate::config::config_dir;
use ratatui::style::Color as RatatuiColor;
use std::path::PathBuf;

pub use jolt_theme::{
    generate_blank_theme_toml, generate_theme_toml, get_builtin_themes, Color, NamedTheme,
};

pub use jolt_theme::ThemeColors as JoltThemeColors;

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub bg: RatatuiColor,
    pub dialog_bg: RatatuiColor,
    pub fg: RatatuiColor,
    pub accent: RatatuiColor,
    pub accent_secondary: RatatuiColor,
    pub highlight: RatatuiColor,
    pub muted: RatatuiColor,
    pub success: RatatuiColor,
    pub warning: RatatuiColor,
    pub danger: RatatuiColor,
    pub border: RatatuiColor,
    pub selection_bg: RatatuiColor,
    pub selection_fg: RatatuiColor,
    pub graph_line: RatatuiColor,
}

impl From<JoltThemeColors> for ThemeColors {
    fn from(colors: JoltThemeColors) -> Self {
        Self {
            bg: to_ratatui_color(colors.bg),
            dialog_bg: to_ratatui_color(colors.dialog_bg),
            fg: to_ratatui_color(colors.fg),
            accent: to_ratatui_color(colors.accent),
            accent_secondary: to_ratatui_color(colors.accent_secondary),
            highlight: to_ratatui_color(colors.highlight),
            muted: to_ratatui_color(colors.muted),
            success: to_ratatui_color(colors.success),
            warning: to_ratatui_color(colors.warning),
            danger: to_ratatui_color(colors.danger),
            border: to_ratatui_color(colors.border),
            selection_bg: to_ratatui_color(colors.selection_bg),
            selection_fg: to_ratatui_color(colors.selection_fg),
            graph_line: to_ratatui_color(colors.graph_line),
        }
    }
}

fn to_ratatui_color(color: Color) -> RatatuiColor {
    RatatuiColor::Rgb(color.r, color.g, color.b)
}

fn themes_dir() -> PathBuf {
    config_dir().join("themes")
}

pub fn load_user_themes() -> Vec<NamedTheme> {
    jolt_theme::load_themes_from_dir(&themes_dir(), false)
}

pub fn get_all_themes() -> Vec<NamedTheme> {
    jolt_theme::get_all_themes(Some(&themes_dir()))
}

pub fn get_theme_by_id(id: &str) -> Option<NamedTheme> {
    jolt_theme::get_theme_by_id(id, Some(&themes_dir()))
}
