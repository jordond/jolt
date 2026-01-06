use super::{NamedTheme, ThemeColors, ThemeVariants};
use crate::config::config_dir;
use ratatui::style::Color;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct ThemeFile {
    name: String,
    dark: Option<ThemeColorsToml>,
    light: Option<ThemeColorsToml>,
}

#[derive(Debug, Deserialize)]
struct ThemeColorsToml {
    bg: String,
    dialog_bg: String,
    fg: String,
    accent: String,
    accent_secondary: String,
    highlight: String,
    muted: String,
    success: String,
    warning: String,
    danger: String,
    border: String,
    selection_bg: String,
    selection_fg: String,
    graph_line: String,
}

fn themes_dir() -> PathBuf {
    config_dir().join("themes")
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

fn convert_colors(colors: &ThemeColorsToml) -> Option<ThemeColors> {
    Some(ThemeColors {
        bg: parse_hex_color(&colors.bg)?,
        dialog_bg: parse_hex_color(&colors.dialog_bg)?,
        fg: parse_hex_color(&colors.fg)?,
        accent: parse_hex_color(&colors.accent)?,
        accent_secondary: parse_hex_color(&colors.accent_secondary)?,
        highlight: parse_hex_color(&colors.highlight)?,
        muted: parse_hex_color(&colors.muted)?,
        success: parse_hex_color(&colors.success)?,
        warning: parse_hex_color(&colors.warning)?,
        danger: parse_hex_color(&colors.danger)?,
        border: parse_hex_color(&colors.border)?,
        selection_bg: parse_hex_color(&colors.selection_bg)?,
        selection_fg: parse_hex_color(&colors.selection_fg)?,
        graph_line: parse_hex_color(&colors.graph_line)?,
    })
}

/// Parse a theme from TOML content. Used by both builtin and user theme loading.
pub fn parse_theme_toml(id: &str, content: &str, is_builtin: bool) -> Option<NamedTheme> {
    let theme_file: ThemeFile = toml::from_str(content).ok()?;

    let dark = theme_file.dark.as_ref().and_then(convert_colors);
    let light = theme_file.light.as_ref().and_then(convert_colors);

    if dark.is_none() && light.is_none() {
        return None;
    }

    Some(NamedTheme {
        id: id.to_string(),
        name: theme_file.name,
        is_builtin,
        variants: ThemeVariants { dark, light },
    })
}

pub fn load_user_themes() -> Vec<NamedTheme> {
    let themes_path = themes_dir();
    if !themes_path.exists() {
        return Vec::new();
    }

    let mut themes = Vec::new();

    if let Ok(entries) = fs::read_dir(&themes_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    if let Some(mut theme) = parse_theme_toml(&id, &content, false) {
                        theme.name = format!("{} (user)", theme.name);
                        themes.push(theme);
                    }
                }
            }
        }
    }

    themes
}
