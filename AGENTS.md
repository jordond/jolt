# AGENTS.md - Coding Agent Guidelines for jolt

A terminal-based battery and energy monitor TUI for macOS Apple Silicon.

## Build Commands

```bash
cargo build                    # Development build
cargo build --release          # Release build (optimized, stripped)
cargo run                      # Run TUI
cargo run -- debug             # Print system/battery info
cargo run -- pipe --samples 2  # JSON output
cargo run -- daemon start      # Start background recorder
```

## Lint & Check Commands

```bash
cargo fmt --all --check                                      # Format check (CI enforces)
cargo fmt --all                                              # Format code
cargo clippy --all-targets --all-features -- -D warnings     # Clippy (CI enforces)
cargo check --all-targets --all-features                     # Type check
```

## Test Commands

```bash
cargo test                     # Run all tests
cargo test test_name           # Run single test by name
cargo test module_name::       # Run tests in module
cargo test -- --nocapture      # Run with output
```

## Project Structure

```
src/
├── main.rs              # CLI entry (clap), subcommands
├── app.rs               # App state, Action enum, event handling
├── config.rs            # UserConfig, HistoryConfig, persistence
├── input.rs             # Key bindings -> Action mapping
├── daemon/
│   ├── mod.rs           # Re-exports
│   ├── client.rs        # IPC client for TUI
│   ├── server.rs        # Background daemon process
│   └── protocol.rs      # Message types
├── data/
│   ├── mod.rs           # Re-exports
│   ├── battery.rs       # Battery from pmset/ioreg
│   ├── power.rs         # Power from IOReport framework
│   ├── processes.rs     # Process data from sysinfo
│   ├── system.rs        # System info (model, chip)
│   ├── history.rs       # In-memory time-series
│   ├── history_store.rs # SQLite persistence
│   ├── recorder.rs      # Background sample collection
│   └── aggregator.rs    # Hourly/daily aggregation
├── theme/
│   ├── mod.rs           # Theme types, get_all_themes()
│   ├── builtin.rs       # Embedded .toml themes
│   ├── loader.rs        # User theme loading
│   ├── cache.rs         # iTerm2 theme cache
│   ├── iterm2.rs        # iTerm2 color scheme import
│   ├── contrast.rs      # WCAG contrast checking
│   └── validation.rs    # Theme validation
└── ui/
    ├── mod.rs           # Main render(), layout
    ├── battery.rs       # Battery gauge widget
    ├── power.rs         # Power metrics widget
    ├── processes.rs     # Process table
    ├── graphs.rs        # Sparkline charts
    ├── history.rs       # Historical data view
    ├── help.rs          # Help/About dialogs
    ├── config_editor.rs # Settings modal
    ├── theme_picker.rs  # Theme selection
    ├── theme_importer.rs# iTerm2 import UI
    ├── daemon_info.rs   # Daemon status modal
    ├── history_config.rs# History settings
    └── status_bar.rs    # Bottom status bar
```

## Code Style

### Imports (three groups, blank line separated)

```rust
use std::collections::HashMap;

use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::config::UserConfig;
```

### Error Handling

- Use `color_eyre::eyre::Result` as default Result type
- Propagate with `?`, fallback to defaults for non-critical failures
- Prefer `unwrap_or_default()` over `unwrap()`

### Naming

- **Types/Enums**: `PascalCase` - `BatteryData`, `AppView`
- **Functions**: `snake_case` - `get_visible_processes`
- **Constants**: `SCREAMING_SNAKE_CASE` - `MAX_REFRESH_MS`

### Struct Definitions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppearanceMode {
    #[default]
    Auto,
    Dark,
    Light,
}
```

### Module Re-exports

```rust
// data/mod.rs
pub use battery::BatteryData;
pub use history::{HistoryData, HistoryMetric};
```

### UI Rendering Pattern

```rust
pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Title ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    // Render content in `inner`
}
```

### Action Pattern

Actions in `app.rs` are enums handled by `handle_action()`:

```rust
pub enum Action {
    Quit,
    ToggleHelp,
    // ... etc
    None,
}
```

## Platform Notes

- **macOS only**: Uses `ioreg`, `pmset`, IOReport APIs
- **Apple Silicon**: Power metrics require M-series chips
- **Rust 2021 edition**: Uses tokio async runtime

## Common Tasks

### Adding a Config Option

1. Add field to `UserConfig` in `config.rs` with serde default
2. Add to config editor in `ui/config_editor.rs`
3. Add handler in `App::toggle_config_value` or similar

### Adding a View/Modal

1. Add variant to `AppView` enum in `app.rs`
2. Add `Action::Toggle*` variant
3. Add key handler in `input.rs`
4. Add render function in `ui/`, match arm in `ui/mod.rs`

### Adding a Theme

1. Create `.toml` in `src/theme/themes/` with `[dark]` and/or `[light]` sections
2. Theme is auto-loaded by `builtin.rs`

### Adding a Data Source

1. Create struct in `data/` with `new()` and `refresh()` methods
2. Re-export in `data/mod.rs`
3. Add to `App` struct, init in `App::new()`, refresh in `App::tick()`

## File Organization

- **Scratch files**: Store plans and temp files in `./scratchpad/` (gitignored)
- **Themes**: Builtin themes in `src/theme/themes/*.toml`
- **User data**: `~/.config/jolt/` for config, `~/.local/share/jolt/` for data

## Task & Issue Workflow

### Scratchpad vs GitHub Issues

| Phase                | Location        | Purpose                              |
| -------------------- | --------------- | ------------------------------------ |
| **Drafting**         | `./scratchpad/` | Brainstorming, research, rough notes |
| **Final Plans**      | GitHub Issues   | Finalized plans, specifications      |
| **Progress Updates** | GitHub Issues   | All status updates as issue comments |

**IMPORTANT**: Scratchpad is for temporary drafts only. Once a plan is finalized:

1. Create a GitHub issue with `gh issue create`
2. Post ALL progress updates as comments on the issue
3. Never track final plans or progress in scratchpad

### Issue Labels

| Label         | Use For                         |
| ------------- | ------------------------------- |
| `bug`         | Something broken                |
| `enhancement` | Improvement to existing feature |
| `feature`     | New functionality               |
| `in-progress` | Currently being worked on       |

### Branch Workflow

```bash
git checkout -b fix/issue-description   # For bugs
git checkout -b feat/issue-description  # For features
```

Use `Fixes #N` or `Closes #N` in PR descriptions to auto-close issues.

## Agent Commands

See `.opencode/commands/` for available commands:

- `/plan <description>` - Create a new plan (drafts in scratchpad, finalizes to GitHub issue)
- `/workon <issue-number | search-query>` - Begin working on a plan issue
- `/update-plan <issue-number>` - Update progress with continuation prompt
- `/close-plan <issue-number>` - Close completed plan with summary
