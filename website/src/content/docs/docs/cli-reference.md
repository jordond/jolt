---
title: CLI Reference
description: Complete command-line reference for jolt
---

Complete reference for all jolt commands and options.

## Global Options

| Option      | Short | Description           |
| ----------- | ----- | --------------------- |
| `--help`    | `-h`  | Show help information |
| `--version` | `-V`  | Show version          |

## Commands

### `jolt` / `jolt ui`

Launch the interactive TUI.

```bash
jolt [OPTIONS]
```

| Option         | Short | Default | Description                              |
| -------------- | ----- | ------- | ---------------------------------------- |
| `--refresh-ms` | `-r`  | 1000    | Update interval in milliseconds          |
| `--appearance` | `-a`  | auto    | Theme mode: `auto`, `dark`, `light`      |
| `--theme`      | `-t`  | default | Theme name                               |
| `--low-power`  | `-L`  | false   | Reduced refresh rate for battery savings |

Examples:

```bash
# Default (1s refresh)
jolt

# Faster refresh
jolt --refresh-ms 500

# Force dark theme
jolt --appearance dark

# Use specific theme
jolt --theme dracula
```

### `jolt pipe`

Output metrics as JSON for scripting.

```bash
jolt pipe [OPTIONS]
```

| Option                | Short | Default | Description                      |
| --------------------- | ----- | ------- | -------------------------------- |
| `--samples`           | `-s`  | 0       | Number of samples (0 = infinite) |
| `--interval`          | `-i`  | 1000    | Interval between samples (ms)    |
| `--compact`           | `-c`  | false   | One-line JSON output (JSONL)     |
| `--include-processes` | `-p`  | false   | Include process data             |

Examples:

```bash
# Single sample
jolt pipe --samples 1

# Continuous at 500ms
jolt pipe --interval 500

# Compact JSONL format
jolt pipe --compact
```

### `jolt debug`

Print system and battery information for troubleshooting.

```bash
jolt debug
```

Shows:

- System model and chip
- macOS version
- Battery information source
- Power metrics availability
- Terminal capabilities

### `jolt config`

Manage configuration.

```bash
jolt config [OPTIONS]
```

| Option    | Description           |
| --------- | --------------------- |
| (none)    | Show current config   |
| `--path`  | Show config file path |
| `--reset` | Reset to defaults     |
| `--edit`  | Open in $EDITOR       |

Examples:

```bash
# Show current config
jolt config

# Show config file path
jolt config --path

# Reset to defaults
jolt config --reset

# Open in editor
jolt config --edit
```

### `jolt theme`

Manage themes.

```bash
jolt theme <SUBCOMMAND>
```

#### `jolt theme list`

List available themes.

```bash
jolt theme list [OPTIONS]
```

| Option                | Description                               |
| --------------------- | ----------------------------------------- |
| `--iterm2`            | List importable iTerm2 themes             |
| `--search <QUERY>`    | Search themes by name                     |
| `--variant <VARIANT>` | Filter by variant: `dark`, `light`, `all` |

#### `jolt theme check`

Validate themes for contrast issues.

```bash
jolt theme check [OPTIONS]
```

| Option           | Description                |
| ---------------- | -------------------------- |
| `--theme <NAME>` | Check specific theme       |
| `--fix`          | Attempt to auto-fix issues |

#### `jolt theme import`

Import an iTerm2 theme.

```bash
jolt theme import <NAME>
```

#### `jolt theme create`

Create a new theme from template.

```bash
jolt theme create <NAME>
```

#### `jolt theme open`

Open themes folder in file manager.

```bash
jolt theme open
```

Opens the themes directory in Finder (macOS) or default file manager (Linux):
- **macOS:** `~/Library/Application Support/jolt/themes/`
- **Linux:** `~/.config/jolt/themes/`

### `jolt daemon`

Manage background data collection.

```bash
jolt daemon <SUBCOMMAND>
```

#### `jolt daemon start`

Start the background daemon.

```bash
jolt daemon start [OPTIONS]
```

| Option         | Description                         |
| -------------- | ----------------------------------- |
| `--foreground` | Run in foreground (don't daemonize) |

#### `jolt daemon stop`

Stop the running daemon.

```bash
jolt daemon stop
```

#### `jolt daemon status`

Show daemon status.

```bash
jolt daemon status
```

#### `jolt daemon logs`

View daemon logs.

```bash
jolt daemon logs [OPTIONS]
```

| Option            | Description             |
| ----------------- | ----------------------- |
| `--follow` / `-f` | Follow log output       |
| `--lines <N>`     | Number of lines to show |

#### `jolt daemon install`

Install daemon to run on login.

```bash
jolt daemon install
```

#### `jolt daemon uninstall`

Remove daemon from login items.

```bash
jolt daemon uninstall
```

### `jolt history`

View and manage historical data.

```bash
jolt history <SUBCOMMAND>
```

#### `jolt history summary`

Show data summary.

```bash
jolt history summary [OPTIONS]
```

| Option              | Description                                  |
| ------------------- | -------------------------------------------- |
| `--period <PERIOD>` | Time period: `today`, `week`, `month`, `all` |

#### `jolt history top`

Show top power consumers.

```bash
jolt history top [OPTIONS]
```

| Option     | Default | Description                      |
| ---------- | ------- | -------------------------------- |
| `--limit`  | 10      | Number of results                |
| `--period` | week    | Time period                      |
| `--sort`   | energy  | Sort by: `energy`, `cpu`, `time` |

#### `jolt history export`

Export data to JSON.

```bash
jolt history export [OPTIONS]
```

| Option                | Description                |
| --------------------- | -------------------------- |
| `--output <FILE>`     | Output file (- for stdout) |
| `--period <PERIOD>`   | Time period to export      |
| `--include-processes` | Include process snapshots  |

#### `jolt history prune`

Remove old data.

```bash
jolt history prune [OPTIONS]
```

| Option                | Description                   |
| --------------------- | ----------------------------- |
| `--older-than <DAYS>` | Delete data older than N days |
| `--dry-run`           | Show what would be deleted    |

#### `jolt history clear`

Delete all historical data.

```bash
jolt history clear [--force]
```

## Exit Codes

| Code | Meaning            |
| ---- | ------------------ |
| 0    | Success            |
| 1    | General error      |
| 2    | Invalid arguments  |
| 3    | Permission denied  |
| 4    | Daemon not running |
| 5    | Config error       |

## Environment Variables

| Variable          | Description                              |
| ----------------- | ---------------------------------------- |
| `JOLT_CONFIG`     | Custom config file path                  |
| `JOLT_THEME`      | Override theme                           |
| `JOLT_APPEARANCE` | Override appearance mode                 |
| `JOLT_LOG_LEVEL`  | Logging level (error, warn, info, debug) |
| `NO_COLOR`        | Disable colored output                   |
