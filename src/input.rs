use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{Action, App, AppView};

pub fn handle_key(app: &App, key: KeyEvent) -> Action {
    match app.view {
        AppView::Main => handle_main_keys(key, app.selection_mode),
        AppView::Help => handle_help_keys(key),
        AppView::About => handle_about_keys(key),
        AppView::KillConfirm => handle_kill_confirm_keys(key),
        AppView::Config => handle_config_keys(key),
        AppView::ThemePicker => handle_theme_picker_keys(key),
        AppView::ThemeImporter => handle_theme_importer_keys(key),
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
        KeyCode::Char('h') | KeyCode::Char('?') => Action::ToggleHelp,
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
        KeyCode::Char('s') => Action::CycleSortColumn,
        KeyCode::Char('S') => Action::ToggleSortDirection,
        KeyCode::Char('=') | KeyCode::Char('+') => Action::IncreaseRefreshRate,
        KeyCode::Char('-') => Action::DecreaseRefreshRate,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char('C') => Action::ToggleConfig,
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

fn handle_theme_importer_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::CloseThemeImporter,
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => Action::SelectPrevious,
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => Action::SelectNext,
        KeyCode::Char(' ') => Action::ImporterToggleSelect,
        KeyCode::Enter => Action::ImporterPreview,
        KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ImporterImport
        }
        KeyCode::Char('r') if key.modifiers.is_empty() => Action::ImporterRefresh,
        KeyCode::Backspace => Action::ImporterFilterBackspace,
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ImporterFilterChar(c)
        }
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        _ => Action::None,
    }
}

fn handle_config_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('C') | KeyCode::Char('q') => Action::ToggleConfig,
        KeyCode::Up | KeyCode::Char('k') => Action::SelectPrevious,
        KeyCode::Down | KeyCode::Char('j') => Action::SelectNext,
        KeyCode::Enter | KeyCode::Char(' ') => Action::ConfigToggleValue,
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('=') => Action::ConfigIncrement,
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('-') => Action::ConfigDecrement,
        KeyCode::Char('r') => Action::ConfigRevert,
        KeyCode::Char('D') => Action::ConfigDefaults,
        _ => Action::None,
    }
}

fn handle_help_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('?') | KeyCode::Char('q') => {
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
        key: "Esc",
        description: "Exit selection mode / Quit",
    },
    KeyBinding {
        key: "Enter/Space",
        description: "Expand/collapse process group",
    },
    KeyBinding {
        key: "K",
        description: "Kill selected process",
    },
    KeyBinding {
        key: "g",
        description: "Toggle graph metric",
    },
    KeyBinding {
        key: "m",
        description: "Toggle merge mode (group similar processes)",
    },
    KeyBinding {
        key: "t",
        description: "Open theme picker",
    },
    KeyBinding {
        key: "a",
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
        key: "s",
        description: "Cycle sort column",
    },
    KeyBinding {
        key: "S",
        description: "Toggle sort direction",
    },
    KeyBinding {
        key: "-/+",
        description: "Decrease/increase refresh rate",
    },
    KeyBinding {
        key: "C",
        description: "Open config editor",
    },
    KeyBinding {
        key: "h/?",
        description: "Toggle help",
    },
    KeyBinding {
        key: "A",
        description: "About jolt",
    },
    KeyBinding {
        key: "q",
        description: "Quit",
    },
];
