---
title: FAQ
description: Frequently asked questions about jolt
---

Common questions and answers about jolt.

## General

### Does jolt work on Linux?

Yes! jolt supports Linux with the same features as macOS:

- ✅ Battery status (percentage, state, health, cycles)
- ✅ Power metrics (watts) via RAPL
- ✅ Process list with CPU usage
- ✅ All TUI features

**Note:** Power metrics on Linux require permissions. See [installation guide](/docs/installation/#linux-permissions).

### Does jolt work on Intel Macs?

Yes, but with limited functionality. Intel Macs will show:

- ✅ Battery status (percentage, state, health, cycles)
- ✅ Process list with CPU usage
- ❌ Power metrics (watts) — requires Apple Silicon

For full power metrics on Intel hardware, use Linux with RAPL support.

### Does jolt work over SSH?

Yes! jolt is fully terminal-based and works great over SSH:

```bash
ssh myserver
jolt
```

Make sure your terminal supports 256 colors for the best experience.

### Does jolt work in tmux/screen?

Yes, jolt works in terminal multiplexers. If you experience display issues:

```bash
# In tmux, ensure 256 color support
set -g default-terminal "screen-256color"
```

### How much battery does jolt itself use?

Very little. At the default 1-second refresh rate:

- ~0.1-0.2% CPU usage
- ~15-20 MB memory

Use `--low-power` mode to reduce even further.

### Can I use jolt in scripts?

Yes! Use pipe mode for JSON output:

```bash
jolt pipe --samples 1 | jq '.battery.percentage'
```

## Battery & Power

### Why is "Time Remaining" sometimes inaccurate?

The time remaining is calculated based on current power consumption. It changes as your workload changes. A sudden increase in CPU usage will immediately affect the estimate.

### What's the difference between battery health and charge?

- **Charge** — Current battery level (0-100%)
- **Health** — Maximum capacity compared to original design (degrades over time)

A battery at 100% charge but 80% health can only hold 80% of its original capacity.

### Why don't I see power metrics (watts)?

Power metrics require platform-specific hardware support:

**macOS:**

1. Apple Silicon Mac (M1/M2/M3/M4)
2. macOS 11.0 or later

Intel Macs cannot report per-component power consumption.

**Linux:**

1. Intel or AMD CPU with RAPL support (kernel 3.13+)
2. Proper permissions configured

See the [installation guide](/docs/installation/#linux-permissions) for Linux setup.

### What does "Not Charging" mean?

Your Mac is connected to power but not charging. This happens when:

- Battery is optimizing charge (staying at 80%)
- Charger wattage is too low
- Battery temperature is too high/low

### How accurate are the power numbers?

Very accurate. jolt reads directly from Apple's IOReport framework, the same source used by Activity Monitor and system power management.

## Processes

### Why do some processes show high energy impact with low CPU?

Energy impact considers more than CPU:

- Disk I/O
- Network activity
- GPU usage
- Wake frequency (waking from idle)

A process can have low CPU but high disk I/O, resulting in elevated energy impact.

### Can I kill processes from jolt?

Yes! Select a process and press `K`. You'll be asked to confirm before the process is killed.

:::caution
Some system processes may restart automatically or cause issues if killed.
:::

### Why are some processes grouped together?

jolt groups parent and child processes. Press `Enter` to expand a group and see individual children.

## Themes

### Where are themes stored?

- Built-in themes: Bundled with jolt
- Custom themes:
  - **macOS:** `~/Library/Application Support/jolt/themes/`
  - **Linux:** `~/.config/jolt/themes/`
- Imported themes: Same as custom themes

### How do I import an iTerm2 theme?

```bash
# From CLI
jolt theme import "Dracula"

# Or from TUI
# Press T, then i, search and select
```

### My theme looks wrong

1. Check terminal color support:

   ```bash
   echo $TERM
   # Should show xterm-256color or similar
   ```

2. Validate theme:

   ```bash
   jolt theme check --theme my-theme
   ```

3. Try a built-in theme to compare:
   ```bash
   jolt --theme default
   ```

## Daemon

### Why use the daemon?

The daemon collects data continuously, even when you're not running jolt. This enables:

- Historical analysis
- Trend tracking
- Export and reporting

### Does the daemon use a lot of resources?

No. The daemon:

- Uses <0.1% CPU
- Uses ~10 MB memory
- Writes to disk once per minute (by default)

### How do I know if the daemon is running?

```bash
jolt daemon status
```

Or in the TUI, press `d` to see daemon info.

### Can I run jolt without the daemon?

Yes! The daemon is optional. The TUI works independently with real-time data.

## Troubleshooting

### jolt shows "Permission denied"

Some features require permissions:

1. **Full Disk Access** — For some process information
2. **Automation** — For system preference detection

Grant permissions in System Settings → Privacy & Security.

### The display looks corrupted

Try:

1. Resize your terminal window
2. Press `Ctrl+L` to redraw
3. Check terminal color support
4. Try a different terminal emulator

### jolt is slow/laggy

1. Increase refresh interval:

   ```bash
   jolt --refresh-ms 2000
   ```

2. Use low power mode:

   ```bash
   jolt --low-power
   ```

3. Check for resource-heavy processes in the list

### Config changes aren't applying

1. Check config file location:

   ```bash
   jolt config --path
   ```

2. Validate config syntax:

   **macOS:**

   ```bash
   cat ~/Library/Application\ Support/jolt/config.toml
   ```

   **Linux:**

   ```bash
   cat ~/.config/jolt/config.toml
   ```

3. Reset to defaults:
   ```bash
   jolt config --reset
   ```
