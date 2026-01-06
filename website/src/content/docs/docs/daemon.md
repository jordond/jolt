---
title: Background Daemon
description: Using jolt's background daemon for continuous data collection
---

jolt includes a background daemon that collects battery and power data even when the TUI isn't running. This enables historical tracking and analysis over time.

## Why Use the Daemon?

- **Continuous monitoring** — Collect data 24/7
- **Historical analysis** — Track trends over days/weeks
- **Low overhead** — Minimal resource usage
- **TUI integration** — Seamless data sharing with the TUI

## Starting the Daemon

### Basic Start

```bash
jolt daemon start
```

The daemon runs in the background and automatically starts collecting data.

### Foreground Mode

For debugging or testing:

```bash
jolt daemon start --foreground
```

Press `Ctrl+C` to stop.

## Stopping the Daemon

```bash
jolt daemon stop
```

## Checking Status

```bash
jolt daemon status
```

Shows:

- Running state
- PID
- Uptime
- Data collection stats
- Last sample time

## Viewing Logs

```bash
# Show recent logs
jolt daemon logs

# Follow logs in real-time
jolt daemon logs --follow
```

## Auto-Start on Login

### Install

```bash
jolt daemon install
```

This creates a LaunchAgent that starts the daemon automatically when you log in.

### Uninstall

```bash
jolt daemon uninstall
```

## Configuration

Daemon settings in `~/.config/jolt/config.toml`:

```toml
[daemon]
# Enable daemon functionality
enabled = true

# Sample interval in seconds
sample_interval = 60

# Socket path for IPC
socket_path = "~/.local/share/jolt/daemon.sock"
```

### Sample Interval

Controls how often the daemon records data:

| Interval | Storage/Day | Use Case          |
| -------- | ----------- | ----------------- |
| 30s      | ~2.8 MB     | Detailed analysis |
| 60s      | ~1.4 MB     | Default, balanced |
| 120s     | ~0.7 MB     | Long-term storage |
| 300s     | ~0.3 MB     | Minimal overhead  |

```toml
[daemon]
sample_interval = 60
```

## Data Storage

The daemon stores data in SQLite:

```
~/.local/share/jolt/history.db
```

### Storage Management

```bash
# View storage usage
jolt history summary

# Prune old data
jolt history prune --older-than 30
```

## TUI Integration

When the daemon is running, the TUI automatically:

- Connects via IPC socket
- Displays daemon status in the UI
- Uses daemon data for extended history
- Shows daemon stats in About dialog

Press `d` in the TUI to view daemon status.

## Troubleshooting

### Daemon Won't Start

1. Check if already running:

   ```bash
   jolt daemon status
   ```

2. Check logs for errors:

   ```bash
   jolt daemon logs
   ```

3. Try foreground mode for debugging:
   ```bash
   jolt daemon start --foreground
   ```

### High CPU Usage

Increase the sample interval:

```toml
[daemon]
sample_interval = 120
```

### Connection Issues

If TUI can't connect to daemon:

1. Verify daemon is running:

   ```bash
   jolt daemon status
   ```

2. Check socket file exists:

   ```bash
   ls ~/.local/share/jolt/daemon.sock
   ```

3. Restart daemon:
   ```bash
   jolt daemon stop && jolt daemon start
   ```

### Data Not Appearing

1. Wait for at least one sample interval
2. Check daemon logs for errors
3. Verify history is enabled:
   ```bash
   jolt config | grep history
   ```
