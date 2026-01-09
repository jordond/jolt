---
title: Introduction
description: What is jolt and why use it?
---

jolt is a terminal-based battery and energy monitor for macOS and Linux. Built in Rust for performance and reliability, it provides real-time insights into your laptop's power consumption.

## Why jolt?

Desktop operating systems include GUI battery monitors (macOS Activity Monitor, GNOME Power Statistics), but they don't work in terminal-only environments. jolt fills this gap by providing:

- **SSH-friendly** — Monitor battery over SSH connections
- **tmux compatible** — Works perfectly in terminal multiplexers
- **Platform native** — Direct hardware access for accurate power metrics
- **Scriptable** — JSON output for automation

## Key Features

### Real-time Battery Status

- Charge percentage with visual gauge
- Charging state and time remaining
- Battery health and cycle count
- Charger wattage detection

### Power Monitoring

- Total system power draw in watts
- CPU and GPU power breakdown
- Power mode detection (Low Power, Normal, High Performance)

:::note
Power metrics require Apple Silicon on macOS or RAPL support on Linux.
:::

### Process Tracking

- Processes sorted by energy impact
- Color-coded severity levels
- Collapsible parent/child process groups
- Kill processes directly from the TUI

### Theming

- 300+ importable themes from iTerm2 Color Schemes
- Dark, light, and auto modes
- Create custom themes with TOML

## Next Steps

Ready to get started? Head to [Installation](/jolt/docs/installation/) to set up jolt on your system.
