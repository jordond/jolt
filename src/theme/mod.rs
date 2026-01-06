mod builtin;
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


