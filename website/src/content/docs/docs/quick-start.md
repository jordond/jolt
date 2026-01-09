---
title: Quick Start
description: Get up and running with jolt in 5 minutes
---

This guide will get you up and running with jolt in under 5 minutes.

## Launch jolt

After [installation](/docs/installation/), simply run:

```bash
jolt
```

You'll see the main TUI interface with:

- Battery gauge showing charge level
- Power metrics (CPU, GPU, total watts)
- Process list sorted by energy impact
- Real-time graphs

:::tip[Platform Note]
All features work the same on macOS and Linux. Power metrics require:

- macOS: Apple Silicon (M1/M2/M3/M4)
- Linux: RAPL permissions ([setup guide](/docs/installation/#linux-permissions))
  :::

## Basic Navigation

| Key       | Action                        |
| --------- | ----------------------------- |
| `↑` / `k` | Move selection up             |
| `↓` / `j` | Move selection down           |
| `Enter`   | Expand/collapse process group |
| `?` / `h` | Show help                     |
| `q`       | Quit                          |

## Try These Features

### 1. Check Battery Health

Look at the battery panel in the top-left. You'll see:

- Current charge percentage
- Charging state (charging, discharging, full)
- Time remaining estimate
- Battery health percentage
- Cycle count

### 2. Monitor Power Usage

The power panel shows real-time wattage:

- **Total** — Combined system power draw
- **CPU** — Processor power consumption
- **GPU** — Graphics power consumption
- **ANE** — Neural Engine power (Apple Silicon only)

**Note:** On Intel Macs, power metrics are not available. On Linux, requires RAPL permissions.

### 3. Find Energy Hogs

The process list shows apps by energy impact:

- 🔴 **High** — Significantly impacting battery
- 🟠 **Elevated** — Above normal usage
- 🟡 **Moderate** — Some impact
- 🟢 **Low** — Minimal impact

Press `Enter` on a parent process to see its children.

### 4. Switch Themes

Press `t` to cycle through appearance modes:

- **Auto** — Follow system dark/light mode
- **Dark** — Force dark theme
- **Light** — Force light theme

Press `T` (shift) to open the theme picker for 300+ themes.

### 5. View Graphs

Press `g` to toggle the graph metric between:

- Battery percentage over time
- Power consumption over time

## Command-Line Options

```bash
# Faster refresh rate (500ms instead of 1000ms)
jolt --refresh-ms 500

# Force dark theme
jolt --appearance dark

# Low power mode (slower refresh, less CPU)
jolt --low-power
```

## Next Steps

- Learn about the [TUI Interface](/docs/tui-interface/) in detail
- See all [Keyboard Shortcuts](/docs/keyboard-shortcuts/)
- Configure jolt with the [Config File](/docs/configuration/)
