use crate::{Color, NamedTheme, ThemeColors, ThemeVariants};
use serde::Deserialize;
use std::fs;
use std::path::Path;

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

fn convert_colors(colors: &ThemeColorsToml) -> Option<ThemeColors> {
    Some(ThemeColors {
        bg: Color::from_hex(&colors.bg)?,
        dialog_bg: Color::from_hex(&colors.dialog_bg)?,
        fg: Color::from_hex(&colors.fg)?,
        accent: Color::from_hex(&colors.accent)?,
        accent_secondary: Color::from_hex(&colors.accent_secondary)?,
        highlight: Color::from_hex(&colors.highlight)?,
        muted: Color::from_hex(&colors.muted)?,
        success: Color::from_hex(&colors.success)?,
        warning: Color::from_hex(&colors.warning)?,
        danger: Color::from_hex(&colors.danger)?,
        border: Color::from_hex(&colors.border)?,
        selection_bg: Color::from_hex(&colors.selection_bg)?,
        selection_fg: Color::from_hex(&colors.selection_fg)?,
        graph_line: Color::from_hex(&colors.graph_line)?,
    })
}

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

pub fn load_themes_from_dir(dir: &Path, is_builtin: bool) -> Vec<NamedTheme> {
    if !dir.exists() {
        return Vec::new();
    }

    let mut themes = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "toml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    if let Some(mut theme) = parse_theme_toml(&id, &content, is_builtin) {
                        if !is_builtin {
                            theme.name = format!("{} (user)", theme.name);
                        }
                        themes.push(theme);
                    }
                }
            }
        }
    }

    themes
}
