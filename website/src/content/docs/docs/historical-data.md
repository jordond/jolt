---
title: Historical Data
description: Viewing and managing jolt's historical data
---

# Historical Data

jolt can track battery and power data over time, enabling analysis of usage patterns and trends.

## Enabling History

History collection requires the [daemon](/jolt/docs/daemon/) to be running:

```bash
jolt daemon start
```

Or enable auto-start:

```bash
jolt daemon install
```

## Viewing History

### Summary

Get an overview of collected data:

```bash
jolt history summary
```

Shows:
- Total records
- Date range
- Average battery level
- Average power consumption
- Peak power usage

### Time Periods

Filter by time period:

```bash
# Today's data
jolt history summary --period today

# Last 7 days
jolt history summary --period week

# Last 30 days
jolt history summary --period month

# All data
jolt history summary --period all
```

## Top Power Consumers

See which processes used the most energy:

```bash
jolt history top
```

Options:

```bash
# More results
jolt history top --limit 20

# Specific time period
jolt history top --period week

# Sort by CPU time instead of energy
jolt history top --sort cpu
```

## Exporting Data

Export historical data to JSON:

```bash
# Export all data
jolt history export --output data.json

# Export specific period
jolt history export --period week --output week.json

# Export with process data
jolt history export --include-processes --output full.json
```

### Export Format

```json
{
  "exported_at": "2024-01-15T10:30:00Z",
  "period": {
    "start": "2024-01-08T00:00:00Z",
    "end": "2024-01-15T10:30:00Z"
  },
  "samples": [
    {
      "timestamp": "2024-01-15T10:00:00Z",
      "battery": {
        "percentage": 85,
        "state": "discharging",
        "health": 92
      },
      "power": {
        "total_watts": 12.5,
        "cpu_watts": 8.2,
        "gpu_watts": 3.1
      }
    }
  ]
}
```

## Managing Storage

### Check Storage Usage

```bash
jolt history summary
```

Shows database size and record count.

### Pruning Old Data

Remove data older than a specified number of days:

```bash
# Remove data older than 30 days
jolt history prune --older-than 30

# Dry run (show what would be removed)
jolt history prune --older-than 30 --dry-run
```

### Clearing All Data

```bash
jolt history clear
```

:::caution
This permanently deletes all historical data. Use with care.
:::

## Configuration

History settings in `~/.config/jolt/config.toml`:

```toml
[history]
# Enable history collection
enabled = true

# Days to retain data (0 = forever)
retention_days = 30

# Sample interval in seconds
sample_interval = 60

# Include process snapshots
include_processes = true

# Maximum database size in MB (0 = unlimited)
max_size_mb = 500
```

### Automatic Pruning

With `retention_days` set, old data is automatically pruned:

```toml
[history]
retention_days = 30  # Auto-delete data older than 30 days
```

## TUI History View

Press `H` in the TUI to open the history view:

- Browse historical data by date
- View graphs of past battery/power levels
- Compare different time periods
- See daily summaries

### History Graph

The main TUI graph can show historical data:

1. Press `g` to toggle graph metric
2. Press `[` / `]` to change time range
3. Scroll through history with `<` / `>`

## Data Aggregation

For long-term storage efficiency, jolt aggregates old data:

| Age | Resolution |
|-----|------------|
| < 24 hours | Full resolution |
| 1-7 days | Hourly averages |
| 7-30 days | 4-hour averages |
| > 30 days | Daily averages |

This keeps the database size manageable while preserving useful trends.

## Integration with Scripts

Use exported data in your own scripts:

```bash
# Get today's average power
jolt history export --period today --output - | \
  jq '.samples | map(.power.total_watts) | add / length'

# Find peak battery drain
jolt history export --period week --output - | \
  jq '.samples | max_by(.power.total_watts)'
```
