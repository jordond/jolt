---
title: Themes
description: Customizing jolt's appearance with themes
---

jolt supports extensive theming with 300+ importable themes from iTerm2 Color Schemes, plus the ability to create your own.

## Appearance Modes

jolt supports three appearance modes:

| Mode | Description |
|------|-------------|
| **Auto** | Follow macOS system appearance |
| **Dark** | Always use dark theme variant |
| **Light** | Always use light theme variant |

Toggle with `t` key or set in config:

```toml
appearance = "auto"  # or "dark" or "light"
```

## Built-in Themes

jolt includes several built-in themes:

- **default** — Clean, minimal theme
- **dracula** — Popular dark purple theme
- **nord** — Arctic, bluish theme
- **gruvbox** — Retro, warm theme
- **tokyo-night** — Modern dark theme
- **catppuccin** — Pastel dark theme

## Switching Themes

### Quick Switch

Press `t` to cycle through appearance modes (Auto → Dark → Light).

### Theme Picker

Press `T` (shift+t) to open the theme picker:

1. Browse themes with `↑`/`↓`
2. Preview in real-time
3. Press `Enter` to apply
4. Press `/` to search

### Via Config

```toml
theme = "dracula"
```

### Via Command Line

```bash
jolt --theme nord
```

## Importing iTerm2 Themes

jolt can import themes from the [iTerm2 Color Schemes](https://iterm2colorschemes.com/) collection.

### From Theme Picker

1. Press `T` to open theme picker
2. Press `i` to open import dialog
3. Search for a theme name
4. Select and import

### Via CLI

```bash
# List available iTerm2 themes
jolt theme list --iterm2

# Search for themes
jolt theme list --search dracula

# Import a theme
jolt theme import "Dracula"
```

Imported themes are saved to `~/.config/jolt/themes/`.

## Theme Structure

Themes are TOML files with color definitions:

```toml
[dark]
background = "#282a36"
foreground = "#f8f8f2"
border = "#44475a"
accent = "#bd93f9"

# Battery gauge colors
battery_high = "#50fa7b"
battery_medium = "#f1fa8c"
battery_low = "#ff5555"

# Energy impact colors
impact_low = "#50fa7b"
impact_moderate = "#f1fa8c"
impact_elevated = "#ffb86c"
impact_high = "#ff5555"

[light]
# Light mode colors...
```

## Color Tokens

| Token | Usage |
|-------|-------|
| `background` | Main background |
| `foreground` | Default text |
| `border` | Panel borders |
| `accent` | Highlights, selection |
| `muted` | Secondary text |
| `battery_high` | Battery > 50% |
| `battery_medium` | Battery 20-50% |
| `battery_low` | Battery < 20% |
| `impact_low` | Low energy impact |
| `impact_moderate` | Moderate impact |
| `impact_elevated` | Elevated impact |
| `impact_high` | High impact |

## Creating Custom Themes

See [Custom Themes](/jolt/docs/custom-themes/) for a complete guide on creating your own themes.

## Theme Validation

jolt validates themes for contrast and accessibility:

```bash
# Check all themes for issues
jolt theme check

# Check specific theme
jolt theme check --theme my-theme
```

Warnings are shown for:
- Insufficient contrast ratios
- Missing required colors
- Invalid color values
