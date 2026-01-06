# AGENTS.md - jolt

Monorepo: macOS battery/energy TUI (Rust) + documentation website (Astro).

## Structure

```
jolt/
├── cli/                   # Rust TUI application
│   └── src/
│       ├── main.rs        # CLI entry (clap subcommands)
│       ├── app.rs         # App state + Action dispatch
│       ├── config.rs      # UserConfig + RuntimeConfig
│       ├── input.rs       # KeyEvent -> Action mapping
│       ├── data/          # Data collection (battery, power, processes)
│       ├── daemon/        # Background recorder + IPC
│       ├── theme/         # Theme system + iTerm2 import
│       └── ui/            # Ratatui widgets
├── website/               # Astro/Starlight docs
└── scripts/check          # Pre-commit checks (fmt, clippy, build)
```

## Commands

### CLI (Rust) - run from cli/ directory

| Task | Command |
|------|---------|
| Build | `cargo build` |
| Build release | `cargo build --release` |
| Run TUI | `cargo run` |
| Debug output | `cargo run -- debug` |
| Format | `cargo fmt --all` |
| Format check | `cargo fmt --all --check` |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` |
| All tests | `cargo test` |
| Single test | `cargo test test_name` (e.g., `cargo test battery::`) |
| Test verbose | `cargo test -- --nocapture` |
| All checks | `./scripts/check` (from repo root) |
| Checks + tests | `./scripts/check --test` |

### Website (Astro) - run from website/ directory

`bun dev` | `bun run lint` | `bun run format:check` | `bun run build`

## Code Style

### Imports
Group in order, separated by blank lines: 1) std, 2) external crates, 3) crate::
```rust
use std::collections::HashMap;
use color_eyre::eyre::Result;
use crate::config::UserConfig;
```

### Error Handling
- Use `color_eyre::eyre::Result` everywhere
- Propagate with `?`, fallback with `unwrap_or_default()`
- **NEVER** `unwrap()` in production paths

### Naming
- Types/Enums: `PascalCase` (`BatteryData`, `Action::ToggleHelp`)
- Functions: `snake_case` (`get_battery_status`)
- Constants: `SCREAMING_SNAKE_CASE` (`MIN_REFRESH_MS`)
- Files: `snake_case.rs`

### UI Render Pattern
```rust
pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default().title(" Title ").borders(Borders::ALL);
    frame.render_widget(block, area);
}
```

### Serde - always use defaults on config structs
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UserConfig { /* ... */ }
```

## Key Types

| Type | Location | Role |
|------|----------|------|
| `App` | app.rs | Central state, owns all data |
| `Action` | app.rs | 50+ variants, drives all interactions |
| `AppView` | app.rs | Modal state machine |
| `RuntimeConfig` | config.rs | Merged CLI args + user config |
| `BatteryData` | data/battery.rs | pmset/ioreg parsed data |
| `PowerData` | data/power.rs | IOReport framework wrapper |
| `ThemeColors` | theme/mod.rs | 14 semantic colors |

## Data Flow

```
main.rs -> App::new()
  ├─> BatteryData::new()    # pmset + ioreg
  ├─> PowerData::new()      # IOReport FFI
  ├─> ProcessData::new()    # sysinfo crate
  └─> RuntimeConfig::load() # ~/.config/jolt/config.toml

Event Loop:
  tick() -> refresh data
  handle_key() -> Action -> handle_action() -> state mutation
  render() -> ui::render(frame, app, theme)
```

## Common Tasks

### Add Config Option
1. Field in `UserConfig` (config.rs) with `#[serde(default)]`
2. UI in `ui/config_editor.rs`
3. Handler in `App::toggle_config_value`

### Add View/Modal
1. Variant in `AppView` enum
2. `Action::Toggle*` variant
3. Key binding in `input.rs`
4. Render fn in `ui/`, match arm in `ui/mod.rs`

### Add Theme
1. Create `cli/src/theme/themes/name.toml` with `[dark]` and/or `[light]`
2. Auto-loaded by `builtin.rs`

### Add Data Source
1. Struct in `cli/src/data/` with `new()` + `refresh()`
2. Re-export in `data/mod.rs`
3. Add field to `App`, init in `new()`, call in `tick()`

## Anti-Patterns

- **NEVER** `unwrap()` on user data paths
- **NEVER** block main thread with IO (use tokio spawn)
- **NEVER** hardcode colors (use ThemeColors)
- **NEVER** access BatteryData/PowerData fields directly outside data/

## Platform

- macOS only (ioreg, pmset, IOReport framework)
- Apple Silicon for power metrics
- Rust 2021 + tokio async
- Tests may require macOS hardware APIs

## Debugging

```bash
cargo run -- debug             # Dump system info
cargo run -- pipe --samples 3  # JSON output
RUST_LOG=debug cargo run       # Verbose logging
```

## Workflow

- Drafting: `./scratchpad/` (gitignored)
- Planning: GitHub Issues
- Branches: `fix/issue-description` or `feat/issue-description`
- Use `Fixes #N` in PR to auto-close issues
