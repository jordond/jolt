---
title: Introduction
description: What is jolt and why use it?
---

jolt is a terminal-based battery and energy monitor for macOS and Linux. It shows you what's draining your battery and how much power your system is using.

## Why jolt?

The built-in battery monitors (Activity Monitor on macOS, GNOME Power Statistics on Linux) are GUI-only. If you spend time in the terminal, SSH into machines, or use tmux, you need something that works there too.

jolt gives you:

- **Battery stats in your terminal** — works over SSH, in tmux, wherever
- **Actual power numbers** — CPU, GPU, and total system watts
- **Process energy tracking** — see which apps are killing your battery
- **JSON output** — pipe it to scripts, log it, do whatever

## What You Get

### Battery Status

Charge percentage, time remaining, health, cycle count, and charger wattage. The basics, but in your terminal.

### Power Metrics

System power draw broken down by CPU and GPU, plus power mode detection. On Apple Silicon you get ANE (Neural Engine) power too.

:::note
Power metrics need Apple Silicon on macOS or RAPL support on Linux. Intel Macs only get battery data.
:::

### Process Tracking

Processes sorted by energy impact with color-coded severity. Expand parent processes to see their children. Kill energy hogs directly from the UI.

### Themes

Import from 300+ iTerm2 color schemes or make your own. Supports dark, light, and auto modes.

## Next Steps

[Install jolt](/docs/installation/) and try it out.
