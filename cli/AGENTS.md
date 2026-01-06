# AGENTS.md - jolt CLI

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

### data/
| File | Source | Refresh |
|------|--------|---------|
| battery.rs | `pmset -g batt` + `ioreg -r -c AppleSmartBattery` | 1s |
| power.rs | IOReport framework (FFI) | 1s |
| processes.rs | sysinfo crate | 1s |
| history.rs | In-memory VecDeque | on sample |
| history_store.rs | SQLite (~/.local/share/jolt/) | on write |
| recorder.rs | Samples data for daemon | configurable |
| aggregator.rs | Hourly/daily rollups | periodic |

### daemon/
Unix socket IPC at `/tmp/jolt-daemon.sock`:
- `DaemonRequest`: GetStatus, GetHourlyStats, GetDailyStats, Shutdown
- `DaemonResponse`: Status, HourlyStats, DailyStats, Ok, Error
- Wire format: newline-delimited JSON

### theme/
- Builtin: TOML files in `themes/` embedded at compile time
- User: `~/.config/jolt/themes/*.toml`
- iTerm2 import: Fetches from iterm2colorschemes.com
- Validation: WCAG contrast checking in `contrast.rs`

### ui/
Each file = one widget. Pattern:
```rust
pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors)
```
Modals: help, config_editor, theme_picker, theme_importer, history, daemon_info
