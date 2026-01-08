//! Settings system with type-safe identifiers and unified behavior definitions.
//!
//! This module replaces string-based settings matching with enum-based identifiers,
//! making settings type-safe, DRY, and localization-friendly.

use crate::app::App;

/// Unique identifier for each setting. Used for type-safe matching instead of strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingId {
    // General
    Theme,
    Appearance,
    RefreshMs,
    // Display
    ShowGraph,
    ShowBrightness,
    MergeMode,
    ProcessCount,
    EnergyThreshold,
    // Recording
    BackgroundRecording,
    SampleInterval,
    RawRetention,
    HourlyRetention,
    DailyRetention,
    SessionRetention,
    MaxDatabase,
}

/// A row in the settings UI - either a section header or a selectable item.
#[derive(Debug, Clone, Copy)]
pub enum SettingsRow {
    /// Non-selectable section header
    Section(&'static str),
    /// Selectable setting with its identifier and display label
    Item { id: SettingId, label: &'static str },
}

impl SettingsRow {
    /// Returns true if this row is a section header (non-selectable).
    pub const fn is_section(&self) -> bool {
        matches!(self, SettingsRow::Section(_))
    }

    /// Returns the setting ID if this is an item, None if section header.
    pub const fn setting_id(&self) -> Option<SettingId> {
        match self {
            SettingsRow::Section(_) => None,
            SettingsRow::Item { id, .. } => Some(*id),
        }
    }
}

/// Input action for a setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingInput {
    /// Toggle/activate (Enter key)
    Activate,
    /// Increment value (Right arrow, +)
    Increment,
    /// Decrement value (Left arrow, -)
    Decrement,
}

/// Result of applying a setting action.
#[derive(Debug, Clone, Copy, Default)]
pub struct SettingOutcome {
    pub open_modal: bool,
}

/// The complete settings layout for the UI.
/// Order here determines display order in the settings panel.
pub const SETTINGS_LAYOUT: &[SettingsRow] = &[
    // General section
    SettingsRow::Section("General"),
    SettingsRow::Item {
        id: SettingId::Theme,
        label: "Theme",
    },
    SettingsRow::Item {
        id: SettingId::Appearance,
        label: "Appearance",
    },
    SettingsRow::Item {
        id: SettingId::RefreshMs,
        label: "Refresh Rate (ms)",
    },
    // Display section
    SettingsRow::Section("Display"),
    SettingsRow::Item {
        id: SettingId::ShowGraph,
        label: "Show Graph",
    },
    SettingsRow::Item {
        id: SettingId::ShowBrightness,
        label: "Show Brightness",
    },
    SettingsRow::Item {
        id: SettingId::MergeMode,
        label: "Merge Mode",
    },
    SettingsRow::Item {
        id: SettingId::ProcessCount,
        label: "Process Count",
    },
    SettingsRow::Item {
        id: SettingId::EnergyThreshold,
        label: "Energy Threshold",
    },
    // Recording section
    SettingsRow::Section("Recording"),
    SettingsRow::Item {
        id: SettingId::BackgroundRecording,
        label: "Background Recording",
    },
    SettingsRow::Item {
        id: SettingId::SampleInterval,
        label: "Sample Interval (s)",
    },
    SettingsRow::Item {
        id: SettingId::RawRetention,
        label: "Raw Retention (days)",
    },
    SettingsRow::Item {
        id: SettingId::HourlyRetention,
        label: "Hourly Retention (days)",
    },
    SettingsRow::Item {
        id: SettingId::DailyRetention,
        label: "Daily Retention (days)",
    },
    SettingsRow::Item {
        id: SettingId::SessionRetention,
        label: "Session Retention (days)",
    },
    SettingsRow::Item {
        id: SettingId::MaxDatabase,
        label: "Max Database (MB)",
    },
];

/// Get the current display value for a setting.
pub fn setting_value(app: &App, id: SettingId) -> String {
    match id {
        SettingId::Theme => format!("{} \u{2192}", app.config.theme_name()),
        SettingId::Appearance => app.config.appearance_label().to_string(),
        SettingId::RefreshMs => app.refresh_ms.to_string(),
        SettingId::ShowGraph => bool_label(app.config.user_config.show_graph),
        SettingId::ShowBrightness => bool_label(app.config.user_config.show_brightness),
        SettingId::MergeMode => bool_label(app.merge_mode),
        SettingId::ProcessCount => app.config.user_config.process_count.to_string(),
        SettingId::EnergyThreshold => format!("{:.1}", app.config.user_config.energy_threshold),
        SettingId::BackgroundRecording => {
            bool_label(app.config.user_config.history.background_recording)
        }
        SettingId::SampleInterval => app
            .config
            .user_config
            .history
            .sample_interval_secs
            .to_string(),
        SettingId::RawRetention => app
            .config
            .user_config
            .history
            .retention_raw_days
            .to_string(),
        SettingId::HourlyRetention => app
            .config
            .user_config
            .history
            .retention_hourly_days
            .to_string(),
        SettingId::DailyRetention => {
            let days = app.config.user_config.history.retention_daily_days;
            if days == 0 {
                "Forever".to_string()
            } else {
                days.to_string()
            }
        }
        SettingId::SessionRetention => app
            .config
            .user_config
            .history
            .retention_sessions_days
            .to_string(),
        SettingId::MaxDatabase => app.config.user_config.history.max_database_mb.to_string(),
    }
}

/// Apply an input action to a setting. Returns the outcome.
pub fn setting_apply(app: &mut App, id: SettingId, input: SettingInput) -> SettingOutcome {
    match id {
        SettingId::Theme => SettingOutcome { open_modal: true },
        SettingId::Appearance => {
            app.config.cycle_appearance();
            SettingOutcome { open_modal: false }
        }
        SettingId::RefreshMs => apply_int(
            app,
            input,
            |a| a.refresh_ms as i64,
            |a, v| {
                a.refresh_ms = v as u64;
                a.config.user_config.refresh_ms = v as u64;
                a.sync_daemon_broadcast_interval();
            },
            500,
            10000,
            500,
        ),
        SettingId::ShowGraph => apply_bool(
            app,
            input,
            |a| a.config.user_config.show_graph,
            |a, v| a.config.user_config.show_graph = v,
        ),
        SettingId::ShowBrightness => apply_bool(
            app,
            input,
            |a| a.config.user_config.show_brightness,
            |a, v| a.config.user_config.show_brightness = v,
        ),
        SettingId::MergeMode => apply_bool(
            app,
            input,
            |a| a.merge_mode,
            |a, v| {
                a.merge_mode = v;
                a.config.user_config.merge_mode = v;
            },
        ),
        SettingId::ProcessCount => apply_int(
            app,
            input,
            |a| a.config.user_config.process_count as i64,
            |a, v| a.config.user_config.process_count = v as usize,
            10,
            200,
            10,
        ),
        SettingId::EnergyThreshold => apply_float(
            app,
            input,
            |a| a.config.user_config.energy_threshold as f64,
            |a, v| a.config.user_config.energy_threshold = v as f32,
            0.0,
            10.0,
            0.5,
        ),
        SettingId::BackgroundRecording => apply_bool(
            app,
            input,
            |a| a.config.user_config.history.background_recording,
            |a, v| a.config.user_config.history.background_recording = v,
        ),
        SettingId::SampleInterval => apply_int(
            app,
            input,
            |a| a.config.user_config.history.sample_interval_secs as i64,
            |a, v| a.config.user_config.history.sample_interval_secs = v as u64,
            10,
            600,
            10,
        ),
        SettingId::RawRetention => apply_int(
            app,
            input,
            |a| a.config.user_config.history.retention_raw_days as i64,
            |a, v| a.config.user_config.history.retention_raw_days = v as u32,
            1,
            365,
            5,
        ),
        SettingId::HourlyRetention => apply_int(
            app,
            input,
            |a| a.config.user_config.history.retention_hourly_days as i64,
            |a, v| a.config.user_config.history.retention_hourly_days = v as u32,
            0,
            730,
            30,
        ),
        SettingId::DailyRetention => apply_int(
            app,
            input,
            |a| a.config.user_config.history.retention_daily_days as i64,
            |a, v| a.config.user_config.history.retention_daily_days = v as u32,
            0,
            3650,
            30,
        ),
        SettingId::SessionRetention => apply_int(
            app,
            input,
            |a| a.config.user_config.history.retention_sessions_days as i64,
            |a, v| a.config.user_config.history.retention_sessions_days = v as u32,
            7,
            365,
            30,
        ),
        SettingId::MaxDatabase => apply_int(
            app,
            input,
            |a| a.config.user_config.history.max_database_mb as i64,
            |a, v| a.config.user_config.history.max_database_mb = v as u32,
            50,
            10000,
            100,
        ),
    }
}

/// Returns the index of the first selectable (non-section) row.
pub fn first_selectable_index() -> usize {
    SETTINGS_LAYOUT
        .iter()
        .position(|row| !row.is_section())
        .unwrap_or(0)
}

/// Check if a row at the given index is a section header.
pub fn is_section_header(index: usize) -> bool {
    SETTINGS_LAYOUT
        .get(index)
        .map(|row| row.is_section())
        .unwrap_or(false)
}

/// Get the setting ID for a row at the given index, if it's a selectable item.
pub fn setting_id_at(index: usize) -> Option<SettingId> {
    SETTINGS_LAYOUT.get(index).and_then(|row| row.setting_id())
}

/// Total number of rows in the settings layout.
pub fn row_count() -> usize {
    SETTINGS_LAYOUT.len()
}

fn bool_label(value: bool) -> String {
    if value { "On" } else { "Off" }.to_string()
}

fn apply_bool<G, S>(app: &mut App, input: SettingInput, get: G, set: S) -> SettingOutcome
where
    G: Fn(&App) -> bool,
    S: Fn(&mut App, bool),
{
    if input == SettingInput::Activate {
        let new_val = !get(app);
        set(app, new_val);
        let _ = app.config.user_config.save();
        SettingOutcome { open_modal: false }
    } else {
        SettingOutcome::default()
    }
}

fn apply_int<G, S>(
    app: &mut App,
    input: SettingInput,
    get: G,
    set: S,
    min: i64,
    max: i64,
    step: i64,
) -> SettingOutcome
where
    G: Fn(&App) -> i64,
    S: Fn(&mut App, i64),
{
    let current = get(app);
    let new_val = match input {
        SettingInput::Increment => (current + step).min(max),
        SettingInput::Decrement => (current - step).max(min),
        SettingInput::Activate => return SettingOutcome::default(),
    };

    if new_val != current {
        set(app, new_val);
        let _ = app.config.user_config.save();
        SettingOutcome { open_modal: false }
    } else {
        SettingOutcome::default()
    }
}

fn apply_float<G, S>(
    app: &mut App,
    input: SettingInput,
    get: G,
    set: S,
    min: f64,
    max: f64,
    step: f64,
) -> SettingOutcome
where
    G: Fn(&App) -> f64,
    S: Fn(&mut App, f64),
{
    let current = get(app);
    let new_val = match input {
        SettingInput::Increment => (current + step).min(max),
        SettingInput::Decrement => (current - step).max(min),
        SettingInput::Activate => return SettingOutcome::default(),
    };

    // Use epsilon comparison for floats
    if (new_val - current).abs() > f64::EPSILON {
        set(app, new_val);
        let _ = app.config.user_config.save();
        SettingOutcome { open_modal: false }
    } else {
        SettingOutcome::default()
    }
}
