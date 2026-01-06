---
title: Config File
description: Configure jolt with the config file
---

jolt stores its configuration in a TOML file that persists your preferences across sessions.

## Config Location

The config file is located at:

```
~/.config/jolt/config.toml
```

jolt creates this file automatically with default values on first run.

## Managing Config

```bash
# Show current configuration
jolt config

# Show config file path
jolt config --path

# Reset to defaults
jolt config --reset

# Open in your $EDITOR
jolt config --edit
```

## Configuration Options

### Appearance

```toml
# Theme appearance mode: "auto", "dark", or "light"
appearance = "auto"

# Active theme name
theme = "default"
```

### Refresh Rate

```toml
# Update interval in milliseconds (min: 100, max: 10000)
refresh_ms = 1000
```

Lower values = more responsive but higher CPU usage.

### Process Display

```toml
[processes]
# Show child processes expanded by default
expand_all = false

# Maximum processes to display
max_visible = 20

# Sort by: "energy", "cpu", "name", "pid"
sort_by = "energy"

# Sort direction: "desc" or "asc"
sort_direction = "desc"
```

### Graph Settings

```toml
[graph]
# Default metric: "battery" or "power"
default_metric = "battery"

# Show graph panel
visible = true

# Graph height in rows
height = 8
```

### Panel Visibility

```toml
[panels]
battery = true
power = true
processes = true
graph = true
```

### History Settings

```toml
[history]
# Enable historical data collection
enabled = true

# Retention period in days
retention_days = 30

# Sample interval for storage (seconds)
sample_interval = 60
```

## Full Example Config

```toml
appearance = "auto"
theme = "dracula"
refresh_ms = 1000

[processes]
expand_all = false
max_visible = 25
sort_by = "energy"
sort_direction = "desc"

[graph]
default_metric = "power"
visible = true
height = 10

[panels]
battery = true
power = true
processes = true
graph = true

[history]
enabled = true
retention_days = 30
sample_interval = 60
```

## In-TUI Config Editor

Press `c` in jolt to open the config editor, which provides a visual interface for changing settings without editing the file directly.

Changes made in the config editor are saved immediately to the config file.

## Environment Variables

Some settings can be overridden with environment variables:

| Variable | Description |
|----------|-------------|
| `JOLT_CONFIG` | Custom config file path |
| `JOLT_THEME` | Override theme |
| `JOLT_APPEARANCE` | Override appearance mode |

Example:

```bash
JOLT_THEME=nord jolt
```
