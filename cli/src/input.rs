use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{Action, App, AppView};

pub mod keys {
    pub const HELP: &str = "?";
    pub const HISTORY: &str = "h";
    pub const QUIT: &str = "q";
    pub const THEME: &str = "t";
    pub const APPEARANCE: &str = "a";
    pub const ABOUT: &str = "A";
    pub const GRAPH: &str = "g";
    pub const MERGE: &str = "m";
    pub const SORT: &str = "o";
    pub const SORT_DIR: &str = "O";
    pub const KILL: &str = "K";
    pub const PERIOD_PREV: &str = "\u{2190}";
    pub const PERIOD_NEXT: &str = "\u{2192}";
    pub const ESC: &str = "Esc";
    pub const SETTINGS: &str = "s";
}

pub fn handle_key(app: &App, key: KeyEvent) -> Action {
    match app.view {
        AppView::Main => handle_main_keys(key, app.selection_mode),
        AppView::Help => handle_help_keys(key),
        AppView::About => handle_about_keys(key),
        AppView::KillConfirm => handle_kill_confirm_keys(key),
        AppView::ThemePicker => handle_theme_picker_keys(key),
        AppView::ThemeImporter => {
            if app.importer_search_focused {
                handle_theme_importer_keys_search(key)
            } else {
                handle_theme_importer_keys_normal(key)
            }
        }
        AppView::History => handle_history_keys(key),
        AppView::Settings => handle_settings_keys(key),
    }
}

fn handle_main_keys(key: KeyEvent, selection_mode: bool) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Esc => {
            if selection_mode {
                Action::ExitSelectionMode
            } else {
                Action::Quit
            }
        }
        KeyCode::Char('?') | KeyCode::Char('/') => Action::ToggleHelp,
        KeyCode::Char('A') => Action::ToggleAbout,
        KeyCode::Char('a') => Action::CycleAppearance,
        KeyCode::Up | KeyCode::Char('k') => Action::SelectPrevious,
        KeyCode::Down | KeyCode::Char('j') => Action::SelectNext,
        KeyCode::Enter | KeyCode::Char(' ') => Action::ToggleExpand,
        KeyCode::Char('K') => Action::KillProcess,
        KeyCode::Char('t') => Action::OpenThemePicker,
        KeyCode::Char('g') => Action::ToggleGraphView,
        KeyCode::Char('m') => Action::ToggleMerge,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        KeyCode::Char('s') => Action::ToggleSettings,
        KeyCode::Char('o') => Action::CycleSortColumn,
        KeyCode::Char('O') => Action::ToggleSortDirection,
        KeyCode::Char('=') | KeyCode::Char('+') => Action::IncreaseRefreshRate,
        KeyCode::Char('-') => Action::DecreaseRefreshRate,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char('h') => Action::ToggleHistory,
        _ => Action::None,
    }
}

fn handle_theme_picker_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('t') | KeyCode::Char('q') => Action::CloseThemePicker,
        KeyCode::Up | KeyCode::Char('k') => Action::SelectPrevious,
        KeyCode::Down | KeyCode::Char('j') => Action::SelectNext,
        KeyCode::Enter | KeyCode::Char(' ') => Action::SelectTheme,
        KeyCode::Char('a') | KeyCode::Left | KeyCode::Right => Action::TogglePreviewAppearance,
        KeyCode::Char('i') => Action::OpenThemeImporter,
        _ => Action::None,
    }
}

fn handle_theme_importer_keys_search(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::ImporterToggleSearch,
        KeyCode::Enter => Action::ImporterToggleSearch,
        KeyCode::Backspace => Action::ImporterFilterBackspace,
        KeyCode::Char(c) => Action::ImporterFilterChar(c),
        _ => Action::None,
    }
}

fn handle_theme_importer_keys_normal(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::CloseThemeImporter,
        KeyCode::Up | KeyCode::Char('k') => Action::SelectPrevious,
        KeyCode::Down | KeyCode::Char('j') => Action::SelectNext,
        KeyCode::Char(' ') => Action::ImporterToggleSelect,
        KeyCode::Enter | KeyCode::Char('p') => Action::ImporterPreview,
        KeyCode::Char('i') => Action::ImporterImport,
        KeyCode::Char('r') => Action::ImporterRefresh,
        KeyCode::Char('/') | KeyCode::Char('s') => Action::ImporterToggleSearch,
        KeyCode::Backspace => Action::ImporterClearFilter,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        _ => Action::None,
    }
}

fn handle_help_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('/') | KeyCode::Char('q') => {
            Action::ToggleHelp
        }
        _ => Action::None,
    }
}

fn handle_about_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('A') | KeyCode::Char('q') => Action::ToggleAbout,
        _ => Action::None,
    }
}

fn handle_kill_confirm_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Action::ConfirmKill,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::CancelKill,
        _ => Action::None,
    }
}

fn handle_history_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Action::ToggleHistory,
        KeyCode::Left | KeyCode::Char('[') => Action::HistoryPrevPeriod,
        KeyCode::Right | KeyCode::Char(']') => Action::HistoryNextPeriod,
        KeyCode::Tab => Action::HistoryNextPeriod,
        KeyCode::Char('s') => Action::ToggleSettings,
        _ => Action::None,
    }
}

fn handle_settings_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('q') => Action::ToggleSettings,
        KeyCode::Up | KeyCode::Char('k') => Action::SelectPrevious,
        KeyCode::Down | KeyCode::Char('j') => Action::SelectNext,
        KeyCode::Enter | KeyCode::Char(' ') => Action::SettingsToggleValue,
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('=') => Action::SettingsIncrement,
        KeyCode::Left | KeyCode::Char('-') => Action::SettingsDecrement,
        _ => Action::None,
    }
}

pub struct KeyBinding {
    pub key: &'static str,
    pub description: &'static str,
}

pub const KEY_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: "↑/k",
        description: "Move selection up (enters selection mode)",
    },
    KeyBinding {
        key: "↓/j",
        description: "Move selection down (enters selection mode)",
    },
    KeyBinding {
        key: keys::ESC,
        description: "Exit selection mode / Quit",
    },
    KeyBinding {
        key: "Enter/Space",
        description: "Expand/collapse process group",
    },
    KeyBinding {
        key: keys::KILL,
        description: "Kill selected process",
    },
    KeyBinding {
        key: keys::GRAPH,
        description: "Toggle graph metric",
    },
    KeyBinding {
        key: keys::MERGE,
        description: "Toggle merge mode (group similar processes)",
    },
    KeyBinding {
        key: keys::THEME,
        description: "Open theme picker",
    },
    KeyBinding {
        key: keys::APPEARANCE,
        description: "Cycle appearance (Auto/Dark/Light)",
    },
    KeyBinding {
        key: "PgUp/PgDn",
        description: "Page up/down",
    },
    KeyBinding {
        key: "Home/End",
        description: "Jump to start/end",
    },
    KeyBinding {
        key: keys::SORT,
        description: "Cycle sort column",
    },
    KeyBinding {
        key: keys::SORT_DIR,
        description: "Toggle sort direction",
    },
    KeyBinding {
        key: "-/+",
        description: "Decrease/increase refresh rate",
    },
    KeyBinding {
        key: keys::SETTINGS,
        description: "Open settings",
    },
    KeyBinding {
        key: keys::HELP,
        description: "Toggle help",
    },
    KeyBinding {
        key: keys::ABOUT,
        description: "About jolt",
    },
    KeyBinding {
        key: keys::HISTORY,
        description: "View history",
    },
    KeyBinding {
        key: keys::QUIT,
        description: "Quit",
    },
];
