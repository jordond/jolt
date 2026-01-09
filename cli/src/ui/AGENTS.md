# AGENTS.md - jolt UI Widgets

**Generated:** 2026-01-08 | **Commit:** c962424

> See `/cli/AGENTS.md` for module context, `/AGENTS.md` for code style.

Ratatui widget layer. Each file = one widget or modal.

## Widget Signature

All widgets follow:
```rust
pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors)
```

Modals use `&mut App` if they mutate state during render.

## Files

| File | Widget | Notes |
|------|--------|-------|
| mod.rs | Main render orchestrator | Layout logic, AppView dispatch |
| battery.rs | Battery gauge + info card | 2-column layout |
| battery_details.rs | Expanded battery modal | Health, cycles, capacity |
| power.rs | Power metrics bar | CPU/GPU/total watts |
| processes.rs | Process table | Scrollable, expandable groups |
| graphs.rs | Sparkline charts (550 LOC) | Battery % and power history |
| status_bar.rs | Title + status bars | System info, hints |
| help.rs | Help/About/Kill modals | Centered popups |
| settings.rs | Settings modal | Toggle list |
| theme_picker.rs | Theme selection | Scrollable list |
| theme_importer.rs | iTerm2 import | URL fetch + preview |
| history.rs | Historical data view (534 LOC) | Date picker + stats |
| cycles.rs | Cycle count details | Daily cycle breakdown |

## Layout System

`mod.rs` contains `LayoutSizes` for responsive layout:
- Minimum heights: battery=10, power=3, graph=8, processes=6
- Auto-compresses when terminal is small
- Graph can be hidden via config

## AppView → Modal Dispatch

```rust
match app.view {
    AppView::Help => help::render_help(frame, app, &theme),
    AppView::About => help::render_about(frame, app, &theme),
    AppView::Config => config_editor::render(frame, app, &theme),
    // ... etc
    AppView::Main => {} // No overlay
}
```

## Adding a Widget

1. Create `widget_name.rs` with `pub fn render(...)`
2. Add `mod widget_name;` to mod.rs imports
3. If modal: add `AppView::WidgetName` variant in app.rs
4. Add dispatch in mod.rs `match app.view`

## Patterns

### Centered Modal
```rust
let popup_area = centered_rect(60, 40, frame.area());
frame.render_widget(Clear, popup_area);
frame.render_widget(block, popup_area);
```

### Block with Theme
```rust
let block = Block::default()
    .title(" Title ")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(theme.border));
```

### Scrollable List State
Pass `&mut app.some_list_state` for stateful widgets.
