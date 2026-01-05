use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Auto,
    Dark,
    Light,
}

impl ThemeMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dark" => ThemeMode::Dark,
            "light" => ThemeMode::Light,
            _ => ThemeMode::Auto,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ThemeMode::Auto => "Auto",
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ThemeMode::Auto => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Auto,
        }
    }
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub theme: ThemeMode,
    pub refresh_ms: u64,
    pub low_power_mode: bool,
    pub show_graph: bool,
    pub graph_metric: GraphMetric,
    pub process_count: usize,
    pub energy_threshold: f32,
    pub merge_mode: bool,
    #[serde(default)]
    pub excluded_processes: Vec<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Auto,
            refresh_ms: 2000,
            low_power_mode: false,
            show_graph: true,
            graph_metric: GraphMetric::Power,
            process_count: 50,
            energy_threshold: 0.5,
            merge_mode: true,
            excluded_processes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GraphMetric {
    #[default]
    Power,
    Battery,
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("jolt")
}

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("jolt")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    fs::create_dir_all(config_dir())?;
    fs::create_dir_all(cache_dir())?;
    Ok(())
}

impl UserConfig {
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let _ = ensure_dirs();
        let path = config_path();
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        fs::write(path, content)
    }

    pub fn merge_with_args(
        &mut self,
        theme: Option<&str>,
        refresh_ms: Option<u64>,
        low_power: bool,
    ) -> bool {
        if let Some(t) = theme {
            self.theme = ThemeMode::from_str(t);
        }
        let refresh_from_cli = refresh_ms.is_some();
        if let Some(ms) = refresh_ms {
            self.refresh_ms = ms;
        }
        if low_power {
            self.low_power_mode = true;
        }
        refresh_from_cli
    }

    pub fn effective_excluded_processes(&self) -> Vec<&str> {
        let mut excluded: Vec<&str> = vec!["launchd"];
        excluded.extend(self.excluded_processes.iter().map(|s| s.as_str()));
        excluded
    }
}

pub struct RuntimeConfig {
    pub user_config: UserConfig,
    system_is_dark: bool,
    pub refresh_from_cli: bool,
}

impl RuntimeConfig {
    pub fn new(user_config: UserConfig, refresh_from_cli: bool) -> Self {
        let system_is_dark = detect_system_dark_mode();
        Self {
            user_config,
            system_is_dark,
            refresh_from_cli,
        }
    }

    pub fn theme(&self) -> Theme {
        let is_dark = match self.user_config.theme {
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
        self.user_config.theme = self.user_config.theme.next();
        let _ = self.user_config.save();
    }

    pub fn theme_mode_label(&self) -> &'static str {
        self.user_config.theme.label()
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
