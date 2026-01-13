// UI adapter: converts jolt-theme colors to ratatui colors
// The actual theme logic lives in the jolt-theme crate

use ratatui::style::{Color as RatatuiColor, Style};

pub use jolt_theme::Color;

/// ThemeColors with ratatui Color types for direct use in UI rendering
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

impl From<jolt_theme::ThemeColors> for ThemeColors {
    fn from(colors: jolt_theme::ThemeColors) -> Self {
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

impl ThemeColors {
    #[inline]
    #[allow(dead_code)]
    pub fn style(&self, color: RatatuiColor) -> Style {
        Style::default().fg(color)
    }

    #[inline]
    pub fn fg_style(&self) -> Style {
        Style::default().fg(self.fg)
    }

    #[inline]
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    #[inline]
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    #[inline]
    pub fn accent_secondary_style(&self) -> Style {
        Style::default().fg(self.accent_secondary)
    }

    #[inline]
    pub fn highlight_style(&self) -> Style {
        Style::default().fg(self.highlight)
    }

    #[inline]
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    #[inline]
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    #[inline]
    pub fn danger_style(&self) -> Style {
        Style::default().fg(self.danger)
    }

    #[inline]
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    #[inline]
    pub fn graph_style(&self) -> Style {
        Style::default().fg(self.graph_line)
    }
}
