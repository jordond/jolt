---
title: TUI Interface
description: Overview of jolt's terminal user interface
---

jolt's terminal user interface is divided into several panels, each showing different aspects of your Mac's power state.

## Layout Overview

The interface is organized into these main areas:

```
┌─────────────────────────────────────────────────────────┐
│  Battery Panel          │  Power Panel                  │
│  (charge, health)       │  (CPU, GPU, total watts)      │
├─────────────────────────┴───────────────────────────────┤
│  Process List                                           │
│  (sorted by energy impact)                              │
├─────────────────────────────────────────────────────────┤
│  Graph Panel                                            │
│  (battery % or power over time)                         │
├─────────────────────────────────────────────────────────┤
│  Status Bar                                             │
└─────────────────────────────────────────────────────────┘
```

## Battery Panel

Shows your Mac's battery status:

| Field       | Description                                     |
| ----------- | ----------------------------------------------- |
| **Charge**  | Current battery percentage with visual gauge    |
| **State**   | Charging, Discharging, Full, or Not Charging    |
| **Time**    | Estimated time to full/empty                    |
| **Health**  | Battery health percentage (capacity vs. design) |
| **Cycles**  | Total charge cycle count                        |
| **Charger** | Connected charger wattage (if plugged in)       |

## Power Panel

Real-time power consumption metrics:

| Metric    | Description                                              |
| --------- | -------------------------------------------------------- |
| **Total** | Combined system power draw in watts                      |
| **CPU**   | Processor cores power consumption                        |
| **GPU**   | Graphics processor power                                 |
| **ANE**   | Apple Neural Engine power (ML workloads)                 |
| **Mode**  | Current power mode (Low Power, Normal, High Performance) |

:::note
Power metrics require Apple Silicon (M1/M2/M3/M4). Intel Macs will show battery data only.
:::

## Process List

Displays running processes sorted by energy impact:

- **Name** — Process name (truncated if too long)
- **PID** — Process ID
- **CPU** — Current CPU usage percentage
- **Energy** — Energy impact rating with color indicator

### Energy Impact Colors

| Color     | Level    | Description               |
| --------- | -------- | ------------------------- |
| 🟢 Green  | Low      | Minimal battery impact    |
| 🟡 Yellow | Moderate | Some battery drain        |
| 🟠 Orange | Elevated | Above normal usage        |
| 🔴 Red    | High     | Significant battery drain |

### Process Groups

Parent processes can be expanded to show child processes:

- Press `Enter` or `Space` to expand/collapse
- Collapsed groups show aggregated CPU/energy
- Expanded groups show individual children indented

## Graph Panel

Shows historical data as a sparkline graph:

- **Battery Mode** — Charge percentage over time
- **Power Mode** — Total watts over time

Press `g` to toggle between battery and power graphs.

The graph shows the last ~60 data points (approximately 1 minute at default refresh rate).

## Status Bar

The bottom status bar shows:

- Current theme and appearance mode
- Refresh rate
- Key hints for common actions
- Daemon connection status (if applicable)

## Modals and Dialogs

### Help Dialog (`?` or `h`)

Shows all keyboard shortcuts organized by category.

### Config Editor (`c`)

In-TUI configuration editor for:

- Appearance mode
- Refresh rate
- Process display options
- Graph settings

### Theme Picker (`T`)

Browse and select from 300+ themes:

- Preview themes in real-time
- Import iTerm2 color schemes
- Filter by light/dark variants

### About Dialog (`a`)

Shows jolt version and system information.
