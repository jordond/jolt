# AGENTS.md - jolt CLI

**Generated:** 2026-01-08 | **Commit:** c962424

> See root `/AGENTS.md` for commands, code style, and conventions.

## Architecture

```
src/
├── main.rs          # CLI entry (clap subcommands)
├── app.rs           # App state + Action dispatch (central hub)
├── config.rs        # UserConfig + RuntimeConfig
├── input.rs         # KeyEvent -> Action mapping
├── data/            # Data collection layer
├── daemon/          # Background recorder + IPC
├── theme/           # Theme system + iTerm2 import
└── ui/              # Ratatui widgets
```

## Module Details

### data/ → See `src/data/AGENTS.md`
Platform-agnostic data collection + SQLite persistence:
| File | Source | Notes |
|------|--------|-------|
| battery.rs | jolt_platform crate | BatteryData wrapper |
| power.rs | jolt_platform crate | 5-sample smoothing |
| processes.rs | sysinfo crate | Energy impact tracking |
| history_store.rs | SQLite WAL (1131 LOC) | Schema v3, complex queries |
| recorder.rs | Daemon orchestration | Session tracking |

### daemon/
Unix socket IPC at `~/.local/share/jolt/jolt.sock`:
- `DaemonRequest`: GetStatus, GetHourlyStats, GetDailyStats, KillProcess, Shutdown
- `DaemonResponse`: Status, HourlyStats, DailyStats, KillResult, Ok, Error
- Wire format: newline-delimited JSON
- Protocol version: 2 (MIN_SUPPORTED_VERSION for backwards compat)

### theme/
- Builtin: TOML files in `themes/` (10 themes: catppuccin, dracula, gruvbox, etc.)
- User: `~/.config/jolt/themes/*.toml`
- iTerm2 import: Fetches from iterm2colorschemes.com
- Validation: WCAG contrast checking in `contrast.rs`
- Core types live in `jolt-theme` workspace crate

### ui/ → See `src/ui/AGENTS.md`
Ratatui widget layer. Each file = one widget:
```rust
pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors)
```
Complex widgets: graphs.rs (550 LOC), history.rs (534 LOC)

## Complexity Hotspots

| File | Lines | Notes |
|------|-------|-------|
| main.rs | 1487 | CLI setup, event loop, daemon launch |
| app.rs | 1432 | Central state, 40+ Action variants |
| data/history_store.rs | 1131 | SQLite schema, migrations, queries |
| daemon/server.rs | 935 | IPC handler, subscriber broadcast |
