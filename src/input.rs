use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{Action, App, AppView};

pub fn handle_key(app: &App, key: KeyEvent) -> Action {
    match app.view {
        AppView::Main => handle_main_keys(key),
        AppView::Help => handle_help_keys(key),
        AppView::KillConfirm => handle_kill_confirm_keys(key),
    }
}

fn handle_main_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('h') | KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Up | KeyCode::Char('k') => Action::SelectPrevious,
        KeyCode::Down | KeyCode::Char('j') => Action::SelectNext,
        KeyCode::Enter | KeyCode::Char(' ') => Action::ToggleExpand,
        KeyCode::Char('K') => Action::KillProcess,
        KeyCode::Char('t') => Action::CycleTheme,
        KeyCode::Char('g') => Action::ToggleGraphView,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
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
        description: "Move selection up",
    },
    KeyBinding {
        key: "↓/j",
        description: "Move selection down",
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
        key: "t",
        description: "Cycle theme (Auto/Dark/Light)",
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
        key: "h/?",
        description: "Toggle help",
    },
    KeyBinding {
        key: "q/Esc",
        description: "Quit",
    },
];
