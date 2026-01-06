---
title: Introduction
description: What is jolt and why use it?
---

# Introduction

jolt is a terminal-based battery and energy monitor designed specifically for macOS. Built in Rust for performance and reliability, it provides real-time insights into your Mac's power consumption.

## Why jolt?

macOS includes Activity Monitor, but it's a GUI app that doesn't work in terminal-only environments. jolt fills this gap by providing:

- **SSH-friendly** — Monitor battery over SSH connections
- **tmux compatible** — Works perfectly in terminal multiplexers
- **Apple Silicon native** — Direct IOReport access for accurate power metrics
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

Ready to get started? Head to [Installation](/jolt/docs/installation/) to set up jolt on your Mac.
