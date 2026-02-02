---
title: Config File
description: Configure jolt with the config file
---

jolt stores its configuration in a TOML file that persists your preferences across sessions.

## Config Location

The config file is located at:

**macOS:**

```
~/Library/Application Support/jolt/config.toml
```

**Linux:**

```
~/.config/jolt/config.toml
```

### XDG Base Directory Support

jolt respects the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/). Path resolution follows this priority:

1. **XDG environment variable** (if set and non-empty)
2. **Platform default** (via `dirs` crate)
3. **Hardcoded fallback** (e.g., `~/.config`)

| Variable          | Default (macOS)                 | Default (Linux)  | Used for                   |
| ----------------- | ------------------------------- | ---------------- | -------------------------- |
| `XDG_CONFIG_HOME` | `~/Library/Application Support` | `~/.config`      | Config file, custom themes |
| `XDG_DATA_HOME`   | `~/Library/Application Support` | `~/.local/share` | Database, history          |
| `XDG_CACHE_HOME`  | `~/Library/Caches`              | `~/.cache`       | Temporary cache            |
| `XDG_RUNTIME_DIR` | (falls back to cache)           | `/run/user/$UID` | Runtime files, logs        |

This means macOS users who prefer XDG-style paths (`~/.config`, `~/.local/share`) can set these environment variables and jolt will use them instead of the Apple-specific locations.

For example, to store jolt's config in `~/.config` on macOS:

```bash
export XDG_CONFIG_HOME="$HOME/.config"
```

With this set, jolt will use `~/.config/jolt/config.toml` instead of `~/Library/Application Support/jolt/config.toml`.

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
# Update interval in milliseconds (min: 500, max: 10000)
refresh_ms = 2000
```

Lower values = more responsive but higher CPU usage.

### Unit Preferences

```toml
[units]
# Energy display: "wh" (Watt-hours) or "mah" (milliamp-hours)
energy = "wh"

# Temperature display: "celsius" or "fahrenheit"
temperature = "celsius"

# Data size display: "si" (KB, MB, GB) or "binary" (KiB, MiB, GiB)
data_size = "si"
```

These settings affect how values are displayed throughout the TUI, including battery capacity, temperature readings, and data sizes.

### Process Display

```toml
# Maximum processes to display
process_count = 50

# Minimum energy impact (watts) to show a process
energy_threshold = 0.5

# Group similar processes together
merge_mode = true

# Processes to hide from the list
excluded_processes = []
```

Example with exclusions:

```toml
excluded_processes = ["kernel_task", "WindowServer"]
```

### Graph Settings

```toml
# Show the graph panel
show_graph = true

# Graph metric: "battery", "power", "split", or "merged"
graph_metric = "merged"
```

| Metric  | Description                    |
| ------- | ------------------------------ |
| battery | Battery percentage only        |
| power   | System power draw only         |
| split   | Battery and power side-by-side |
| merged  | Combined view (default)        |

### History Settings

```toml
[history]
# Enable background data collection (requires daemon)
background_recording = false

# Sample interval in seconds
sample_interval_secs = 60

# Keep raw samples for N days
retention_raw_days = 30

# Keep hourly aggregates for N days
retention_hourly_days = 180

# Keep daily aggregates for N days (0 = forever)
retention_daily_days = 0

# Keep session data for N days
retention_sessions_days = 90

# Maximum database size in MB
max_database_mb = 500
```

:::note
The `enabled` alias still works for `background_recording` for backwards compatibility.
:::

## Full Example Config

```toml
# Display settings
appearance = "auto"
theme = "dracula"
refresh_ms = 2000

# Graph settings
show_graph = true
graph_metric = "merged"

# Process settings
process_count = 50
energy_threshold = 0.5
merge_mode = true
excluded_processes = []

# Unit preferences
[units]
energy = "wh"
temperature = "celsius"
data_size = "si"

# History settings (requires daemon)
[history]
background_recording = false
sample_interval_secs = 60
retention_raw_days = 30
retention_hourly_days = 180
retention_daily_days = 0
retention_sessions_days = 90
max_database_mb = 500
```

## In-TUI Settings

Press `s` in jolt to open the settings panel, which provides a visual interface for changing settings without editing the file directly.

Changes made in settings are saved immediately to the config file.

## Environment Variables

Some settings can be overridden with environment variables:

| Variable          | Description                                                                |
| ----------------- | -------------------------------------------------------------------------- |
| `JOLT_CONFIG`     | Custom config file path                                                    |
| `JOLT_THEME`      | Override theme                                                             |
| `JOLT_APPEARANCE` | Override appearance mode                                                   |
| `XDG_CONFIG_HOME` | Override config directory (see [XDG support](#xdg-base-directory-support)) |
| `XDG_DATA_HOME`   | Override data directory                                                    |
| `XDG_CACHE_HOME`  | Override cache directory                                                   |
| `XDG_RUNTIME_DIR` | Override runtime directory                                                 |

Example:

```bash
JOLT_THEME=nord jolt
```
