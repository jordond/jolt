---
title: Custom Themes
description: Creating your own jolt themes
---

Create personalized themes for jolt with TOML configuration files.

## Theme Location

Custom themes are stored in:

**macOS:**
```
~/Library/Application Support/jolt/themes/
```

**Linux:**
```
~/.config/jolt/themes/
```

Each theme is a `.toml` file with color definitions.

## Creating a Theme

### Generate Template

```bash
jolt theme create "My Theme"
```

This creates `my-theme.toml` in your themes directory with a template.

### Manual Creation

Create a new `.toml` file in the themes directory:

```toml
# my-theme.toml

[dark]
background = "#1a1b26"
foreground = "#c0caf5"
border = "#3b4261"
accent = "#7aa2f7"
muted = "#565f89"

battery_high = "#9ece6a"
battery_medium = "#e0af68"
battery_low = "#f7768e"

impact_low = "#9ece6a"
impact_moderate = "#e0af68"
impact_elevated = "#ff9e64"
impact_high = "#f7768e"

[light]
background = "#d5d6db"
foreground = "#343b58"
border = "#9699a3"
accent = "#34548a"
muted = "#9699a3"

battery_high = "#485e30"
battery_medium = "#8f5e15"
battery_low = "#8c4351"

impact_low = "#485e30"
impact_moderate = "#8f5e15"
impact_elevated = "#965027"
impact_high = "#8c4351"
```

## Color Tokens Reference

### Core Colors

| Token        | Description                  | Required |
| ------------ | ---------------------------- | -------- |
| `background` | Main background color        | Yes      |
| `foreground` | Default text color           | Yes      |
| `border`     | Panel and box borders        | Yes      |
| `accent`     | Highlights, selection, focus | Yes      |
| `muted`      | Secondary/dimmed text        | No       |

### Battery Colors

| Token              | Description        | Fallback |
| ------------------ | ------------------ | -------- |
| `battery_high`     | Battery > 50%      | Green    |
| `battery_medium`   | Battery 20-50%     | Yellow   |
| `battery_low`      | Battery < 20%      | Red      |
| `battery_charging` | Charging indicator | Accent   |

### Energy Impact Colors

| Token             | Description           | Fallback |
| ----------------- | --------------------- | -------- |
| `impact_low`      | Low energy processes  | Green    |
| `impact_moderate` | Moderate energy       | Yellow   |
| `impact_elevated` | Elevated energy       | Orange   |
| `impact_high`     | High energy processes | Red      |

### Additional Colors

| Token        | Description              | Fallback        |
| ------------ | ------------------------ | --------------- |
| `graph_line` | Graph line color         | Accent          |
| `graph_fill` | Graph fill color         | Accent (dimmed) |
| `selection`  | Selected item background | Accent          |
| `error`      | Error messages           | Red             |
| `warning`    | Warning messages         | Yellow          |
| `success`    | Success messages         | Green           |

## Color Formats

jolt accepts colors in these formats:

```toml
# Hex (6 digit)
background = "#282a36"

# Hex (3 digit shorthand)
accent = "#f0f"

# RGB
foreground = "rgb(248, 248, 242)"
```

## Dark and Light Variants

Themes can define both variants:

```toml
[dark]
background = "#282a36"
foreground = "#f8f8f2"

[light]
background = "#f8f8f2"
foreground = "#282a36"
```

If only one variant is defined, it will be used for both modes.

## Inheriting from Other Themes

You can base a theme on another:

```toml
extends = "dracula"

[dark]
# Only override specific colors
accent = "#ff79c6"
```

## Testing Your Theme

1. Create or edit your theme file
2. Run jolt: `jolt --theme my-theme`
3. Or select it from the theme picker (`T`)

Changes to theme files are picked up on next launch.

## Validating Themes

Check your theme for contrast issues:

```bash
jolt theme check --theme my-theme
```

This checks:

- Text/background contrast ratios (WCAG guidelines)
- Required colors are present
- Color values are valid

## Sharing Themes

To share a theme:

1. Copy the `.toml` file from your themes directory
2. Share it (GitHub gist, etc.)
3. Others can place it in their themes directory

## Converting iTerm2 Themes

If you have an iTerm2 `.itermcolors` file:

```bash
jolt theme import-file ~/Downloads/MyTheme.itermcolors
```

jolt will convert the color scheme to its format.

## Example: Ocean Theme

```toml
# A calming ocean-inspired theme

[dark]
background = "#0d1117"
foreground = "#c9d1d9"
border = "#21262d"
accent = "#58a6ff"
muted = "#8b949e"

battery_high = "#3fb950"
battery_medium = "#d29922"
battery_low = "#f85149"

impact_low = "#3fb950"
impact_moderate = "#d29922"
impact_elevated = "#db6d28"
impact_high = "#f85149"

graph_line = "#58a6ff"
graph_fill = "#1f3a5f"

[light]
background = "#ffffff"
foreground = "#24292f"
border = "#d0d7de"
accent = "#0969da"
muted = "#57606a"

battery_high = "#1a7f37"
battery_medium = "#9a6700"
battery_low = "#cf222e"

impact_low = "#1a7f37"
impact_moderate = "#9a6700"
impact_elevated = "#bc4c00"
impact_high = "#cf222e"
```
