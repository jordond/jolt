---
title: Troubleshooting
description: Solutions to common jolt issues
---

Solutions to common issues you might encounter with jolt.

## Installation Issues

### "command not found: jolt"

The jolt binary isn't in your PATH.

**Homebrew install:**
```bash
# Verify installation
brew list jolt

# If installed, try:
eval "$(/opt/homebrew/bin/brew shellenv)"
```

**Cargo install:**
```bash
# Add cargo bin to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Or reinstall
cargo install jolt-tui --force
```

**Source install:**
```bash
# Copy to PATH
sudo cp ./target/release/jolt /usr/local/bin/
```

### Build fails from source

**Missing Xcode tools:**
```bash
xcode-select --install
```

**Rust not installed:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Outdated Rust:**
```bash
rustup update
```

## Display Issues

### Corrupted/garbled display

1. **Resize terminal** — Often fixes rendering issues

2. **Force redraw** — Press `Ctrl+L`

3. **Check TERM variable:**
   ```bash
   echo $TERM
   # Should be xterm-256color or similar
   ```

4. **Set correct TERM:**
   ```bash
   export TERM=xterm-256color
   jolt
   ```

### Colors look wrong

1. **Verify 256 color support:**
   ```bash
   # Should show colored gradient
   for i in {0..255}; do printf "\e[48;5;${i}m \e[0m"; done; echo
   ```

2. **Check theme:**
   ```bash
   jolt --theme default
   ```

3. **Try different terminal:**
   - iTerm2, Alacritty, and Kitty have best color support
   - Default Terminal.app may have issues with some themes

### UI doesn't fit terminal

jolt needs a minimum terminal size:
- **Width:** 80 columns
- **Height:** 24 rows

Resize your terminal or reduce font size.

## Data Issues

### "No power metrics available"

Power metrics require Apple Silicon:
- M1, M2, M3, M4 series chips
- Intel Macs cannot report power consumption

Verify your chip:
```bash
sysctl -n machdep.cpu.brand_string
```

### Battery data is stale/wrong

1. **Check macOS battery:**
   ```bash
   pmset -g batt
   ```

2. **Compare with ioreg:**
   ```bash
   ioreg -r -c AppleSmartBattery | grep -E "Cycle|Capacity|Health"
   ```

3. **Report bug if mismatched** — jolt should match these sources

### Process list is empty

1. **Check permissions:**
   - System Settings → Privacy & Security → Full Disk Access
   - Add your terminal app

2. **Run debug:**
   ```bash
   jolt debug
   ```

3. **Check for errors:**
   ```bash
   jolt 2>&1 | grep -i error
   ```

## Daemon Issues

### Daemon won't start

1. **Check if already running:**
   ```bash
   jolt daemon status
   pgrep -f "jolt daemon"
   ```

2. **Check logs:**
   ```bash
   jolt daemon logs
   ```

3. **Try foreground mode:**
   ```bash
   jolt daemon start --foreground
   ```

4. **Check socket:**
   ```bash
   ls -la ~/.local/share/jolt/daemon.sock
   ```

### TUI can't connect to daemon

1. **Verify daemon is running:**
   ```bash
   jolt daemon status
   ```

2. **Check socket permissions:**
   ```bash
   ls -la ~/.local/share/jolt/daemon.sock
   ```

3. **Restart daemon:**
   ```bash
   jolt daemon stop
   jolt daemon start
   ```

### Daemon uses too much CPU

Increase sample interval:

```toml
# ~/.config/jolt/config.toml
[daemon]
sample_interval = 120  # Every 2 minutes
```

Then restart:
```bash
jolt daemon stop && jolt daemon start
```

## Config Issues

### Config file not found

jolt creates it on first run. If missing:

```bash
# Check location
jolt config --path

# Create directory
mkdir -p ~/.config/jolt

# Create default config
jolt config --reset
```

### Config changes not taking effect

1. **Verify syntax:**
   ```bash
   cat ~/.config/jolt/config.toml
   ```

2. **Check for typos** — TOML is case-sensitive

3. **Restart jolt** — Some changes require restart

4. **Check for overrides:**
   ```bash
   # Environment variables override config
   echo $JOLT_THEME
   ```

### Reset to defaults

```bash
jolt config --reset
```

Or manually delete:
```bash
rm ~/.config/jolt/config.toml
```

## Performance Issues

### jolt is slow/laggy

1. **Reduce refresh rate:**
   ```bash
   jolt --refresh-ms 2000
   ```

2. **Enable low power mode:**
   ```bash
   jolt --low-power
   ```

3. **Close other heavy apps** — jolt shows you what's using resources!

### High CPU from jolt itself

1. **Check refresh rate:**
   ```toml
   refresh_ms = 1000  # Don't go below 500
   ```

2. **Disable process tracking:**
   ```toml
   [panels]
   processes = false
   ```

3. **Report bug** — jolt should use minimal CPU

## Getting Help

### Collect debug info

```bash
jolt debug > debug.txt
```

### Check version

```bash
jolt --version
```

### Report issues

Include in bug reports:
- jolt version
- macOS version
- Chip type (Intel/M1/M2/etc)
- Terminal app
- Output of `jolt debug`
- Steps to reproduce

Report at: https://github.com/jordond/jolt/issues
