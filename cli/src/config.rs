use crate::theme::{get_theme_by_id, NamedTheme, ThemeColors};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppearanceMode {
    #[default]
    Auto,
    Dark,
    Light,
}

impl AppearanceMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dark" => AppearanceMode::Dark,
            "light" => AppearanceMode::Light,
            _ => AppearanceMode::Auto,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AppearanceMode::Auto => "Auto",
            AppearanceMode::Dark => "Dark",
            AppearanceMode::Light => "Light",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            AppearanceMode::Auto => AppearanceMode::Dark,
            AppearanceMode::Dark => AppearanceMode::Light,
            AppearanceMode::Light => AppearanceMode::Auto,
        }
    }
}

fn default_theme_name() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryConfig {
    pub enabled: bool,
    pub sample_interval_secs: u64,
    pub retention_raw_days: u32,
    pub retention_hourly_days: u32,
    pub retention_daily_days: u32,
    pub max_database_mb: u32,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_interval_secs: 60,
            retention_raw_days: 30,
            retention_hourly_days: 180,
            retention_daily_days: 0,
            max_database_mb: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub appearance: AppearanceMode,
    #[serde(default = "default_theme_name")]
    pub theme: String,
    pub refresh_ms: u64,
    pub low_power_mode: bool,
    pub show_graph: bool,
    pub graph_metric: GraphMetric,
    pub process_count: usize,
    pub energy_threshold: f32,
    pub merge_mode: bool,
    pub forecast_window_secs: u64,
    #[serde(default)]
    pub excluded_processes: Vec<String>,
    #[serde(default)]
    pub history: HistoryConfig,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            appearance: AppearanceMode::Auto,
            theme: "default".to_string(),
            refresh_ms: 2000,
            low_power_mode: false,
            show_graph: true,
            graph_metric: GraphMetric::Power,
            process_count: 50,
            energy_threshold: 0.5,
            merge_mode: true,
            forecast_window_secs: 300,
            excluded_processes: Vec::new(),
            history: HistoryConfig::default(),
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

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("jolt")
}

pub fn runtime_dir() -> PathBuf {
    dirs::runtime_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("jolt")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn themes_dir() -> PathBuf {
    config_dir().join("themes")
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
        appearance: Option<&str>,
        refresh_ms: Option<u64>,
        low_power: bool,
    ) -> bool {
        if let Some(a) = appearance {
            self.appearance = AppearanceMode::from_str(a);
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
    pub system_is_dark: bool,
    pub refresh_from_cli: bool,
    current_theme: NamedTheme,
}

impl RuntimeConfig {
    pub fn new(user_config: UserConfig, refresh_from_cli: bool) -> Self {
        let system_is_dark = detect_system_dark_mode();
        let current_theme = get_theme_by_id(&user_config.theme)
            .unwrap_or_else(|| get_theme_by_id("default").expect("Default theme must exist"));

        Self {
            user_config,
            system_is_dark,
            refresh_from_cli,
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
        self.current_theme.get_colors(self.is_dark_mode())
    }

    pub fn theme_with_mode(&self, is_dark: bool) -> ThemeColors {
        self.current_theme.get_colors(is_dark)
    }

    pub fn set_theme(&mut self, theme_id: &str) {
        if let Some(theme) = get_theme_by_id(theme_id) {
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
