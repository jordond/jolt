mod builtin;
pub mod contrast;
mod loader;

pub use builtin::get_builtin_themes;
pub use loader::load_user_themes;

use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub bg: Color,
    pub dialog_bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub highlight: Color,
    pub muted: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub border: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub graph_line: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeVariants {
    pub dark: Option<ThemeColors>,
    pub light: Option<ThemeColors>,
}

#[derive(Debug, Clone)]
pub struct NamedTheme {
    pub id: String,
    pub name: String,
    pub variants: ThemeVariants,
    pub is_builtin: bool,
}

impl NamedTheme {
    pub fn get_colors(&self, is_dark: bool) -> ThemeColors {
        if is_dark {
            self.variants
                .dark
                .or(self.variants.light)
                .expect("Theme must have at least one variant")
        } else {
            self.variants
                .light
                .or(self.variants.dark)
                .expect("Theme must have at least one variant")
        }
    }

    pub fn has_dark(&self) -> bool {
        self.variants.dark.is_some()
    }

    pub fn has_light(&self) -> bool {
        self.variants.light.is_some()
    }

    pub fn variants_label(&self) -> &'static str {
        match (self.has_dark(), self.has_light()) {
            (true, true) => "dark + light",
            (true, false) => "dark only",
            (false, true) => "light only",
            _ => "unknown",
        }
    }
}

pub fn get_all_themes() -> Vec<NamedTheme> {
    let mut themes = get_builtin_themes();
    themes.extend(load_user_themes());
    themes
}

pub fn get_theme_by_id(id: &str) -> Option<NamedTheme> {
    get_all_themes().into_iter().find(|t| t.id == id)
}

fn color_to_hex(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
        _ => "#808080".to_string(),
    }
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
        color_to_hex(colors.bg),
        color_to_hex(colors.dialog_bg),
        color_to_hex(colors.fg),
        color_to_hex(colors.accent),
        color_to_hex(colors.accent_secondary),
        color_to_hex(colors.highlight),
        color_to_hex(colors.muted),
        color_to_hex(colors.success),
        color_to_hex(colors.warning),
        color_to_hex(colors.danger),
        color_to_hex(colors.border),
        color_to_hex(colors.selection_bg),
        color_to_hex(colors.selection_fg),
        color_to_hex(colors.graph_line),
        i = indent,
    )
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
