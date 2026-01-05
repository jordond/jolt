use ratatui::style::Color;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Auto,
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg: Color,
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
    pub graph_fill: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb(22, 22, 30),
            fg: Color::Rgb(230, 230, 240),
            accent: Color::Rgb(138, 180, 248),
            accent_secondary: Color::Rgb(187, 134, 252),
            highlight: Color::Rgb(255, 203, 107),
            muted: Color::Rgb(128, 128, 140),
            success: Color::Rgb(129, 199, 132),
            warning: Color::Rgb(255, 183, 77),
            danger: Color::Rgb(239, 83, 80),
            border: Color::Rgb(60, 60, 80),
            selection_bg: Color::Rgb(50, 50, 70),
            selection_fg: Color::Rgb(255, 255, 255),
            graph_line: Color::Rgb(138, 180, 248),
            graph_fill: Color::Rgb(60, 90, 140),
        }
    }

    pub fn light() -> Self {
        Self {
            bg: Color::Rgb(250, 250, 252),
            fg: Color::Rgb(30, 30, 40),
            accent: Color::Rgb(25, 118, 210),
            accent_secondary: Color::Rgb(123, 31, 162),
            highlight: Color::Rgb(255, 160, 0),
            muted: Color::Rgb(140, 140, 150),
            success: Color::Rgb(46, 125, 50),
            warning: Color::Rgb(239, 108, 0),
            danger: Color::Rgb(211, 47, 47),
            border: Color::Rgb(200, 200, 210),
            selection_bg: Color::Rgb(220, 230, 245),
            selection_fg: Color::Rgb(0, 0, 0),
            graph_line: Color::Rgb(25, 118, 210),
            graph_fill: Color::Rgb(180, 210, 240),
        }
    }
}

pub struct Config {
    pub theme_mode: ThemeMode,
    system_is_dark: bool,
}

impl Config {
    pub fn new(theme_mode: ThemeMode) -> Self {
        let system_is_dark = detect_system_dark_mode();

        Self {
            theme_mode,
            system_is_dark,
        }
    }

    pub fn theme(&self) -> Theme {
        let is_dark = match self.theme_mode {
            ThemeMode::Auto => self.system_is_dark,
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
        };

        if is_dark {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    pub fn cycle_theme(&mut self) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::Auto => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Auto,
        };
    }

    pub fn theme_mode_label(&self) -> &'static str {
        match self.theme_mode {
            ThemeMode::Auto => "Auto",
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
        }
    }
}

fn detect_system_dark_mode() -> bool {
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
