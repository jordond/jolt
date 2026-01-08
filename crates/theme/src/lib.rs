mod builtin;
pub mod cache;
pub mod contrast;
pub mod iterm2;
mod loader;
mod types;
pub mod validation;

pub use builtin::get_builtin_themes;
pub use loader::{load_themes_from_dir, parse_theme_toml};
pub use types::{Color, NamedTheme, ThemeColors, ThemeVariants};

use std::path::Path;

pub fn get_all_themes(user_themes_dir: Option<&Path>) -> Vec<NamedTheme> {
    let mut themes = get_builtin_themes();
    if let Some(dir) = user_themes_dir {
        themes.extend(load_themes_from_dir(dir, false));
    }
    themes
}

pub fn get_theme_by_id(id: &str, user_themes_dir: Option<&Path>) -> Option<NamedTheme> {
    get_all_themes(user_themes_dir)
        .into_iter()
        .find(|t| t.id == id)
}

pub fn color_to_hex(color: &Color) -> String {
    color.to_hex()
}

pub fn generate_theme_toml(name: &str, base: &NamedTheme) -> String {
    let mut content = format!("name = \"{}\"\n\n", name);

    if let Some(ref dark) = base.variants.dark {
        content.push_str("[dark]\n");
        content.push_str(&colors_to_toml(dark, ""));
        content.push_str("\n\n");
    }

    if let Some(ref light) = base.variants.light {
        content.push_str("[light]\n");
        content.push_str(&colors_to_toml(light, ""));
        content.push('\n');
    }

    content
}

pub fn generate_blank_theme_toml(name: &str) -> String {
    format!(
        r##"name = "{}"

[dark]
bg = "#1e1e2e"
dialog_bg = "#313244"
fg = "#cdd6f4"
accent = "#89b4fa"
accent_secondary = "#cba6f7"
highlight = "#f9e2af"
muted = "#6c7086"
success = "#a6e3a1"
warning = "#fab387"
danger = "#f38ba8"
border = "#45475a"
selection_bg = "#585b70"
selection_fg = "#cdd6f4"
graph_line = "#89b4fa"

[light]
bg = "#eff1f5"
dialog_bg = "#e6e9ef"
fg = "#4c4f69"
accent = "#1e66f5"
accent_secondary = "#8839ef"
highlight = "#df8e1d"
muted = "#6c6f85"
success = "#40a02b"
warning = "#fe640b"
danger = "#d20f39"
border = "#bcc0cc"
selection_bg = "#acb0be"
selection_fg = "#4c4f69"
graph_line = "#1e66f5"
"##,
        name
    )
}

fn colors_to_toml(colors: &ThemeColors, indent: &str) -> String {
    format!(
        r#"{i}bg = "{}"
{i}dialog_bg = "{}"
{i}fg = "{}"
{i}accent = "{}"
{i}accent_secondary = "{}"
{i}highlight = "{}"
{i}muted = "{}"
{i}success = "{}"
{i}warning = "{}"
{i}danger = "{}"
{i}border = "{}"
{i}selection_bg = "{}"
{i}selection_fg = "{}"
{i}graph_line = "{}""#,
        colors.bg.to_hex(),
        colors.dialog_bg.to_hex(),
        colors.fg.to_hex(),
        colors.accent.to_hex(),
        colors.accent_secondary.to_hex(),
        colors.highlight.to_hex(),
        colors.muted.to_hex(),
        colors.success.to_hex(),
        colors.warning.to_hex(),
        colors.danger.to_hex(),
        colors.border.to_hex(),
        colors.selection_bg.to_hex(),
        colors.selection_fg.to_hex(),
        colors.graph_line.to_hex(),
        i = indent,
    )
}
