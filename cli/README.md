# jolt CLI

A beautiful terminal-based battery and energy monitor for macOS and Linux.

Built for laptop users who want to understand what's draining their battery. Provides real-time insights into power consumption, process energy usage, and battery health - all in a clean, themeable TUI.

## Features

- **Battery Status** - Real-time charge percentage, charging state, time remaining, battery health, and cycle count
- **Power Monitoring** - Total system power draw with CPU and GPU breakdown
- **Process Energy Tracking** - Processes sorted by energy impact with color-coded severity
- **Collapsible Process Groups** - Expand parent processes to see children consuming energy
- **Historical Graphs** - Track battery percentage and power draw over time
- **Theme Support** - Dark and light themes with automatic system detection
- **Keyboard Navigation** - Full keyboard control with help dialog
- **Process Management** - Kill energy-hungry processes with confirmation

## Requirements

### macOS
- macOS 11.0 (Big Sur) or later
- Apple Silicon (optimized) or Intel Mac

### Linux
- Linux kernel 3.13+ (for RAPL power metrics)
- Laptop with battery
- Intel or AMD CPU (for power metrics)

**Note:** On Linux, see [Linux setup guide](../docs/linux-setup.md) for required permissions.

## Installation

### Install Script (Recommended)

```bash
curl -fsSL https://getjolt.sh/install.sh | bash
```

Automatically detects your platform (macOS or Linux) and architecture.

### From Source

```bash
git clone https://github.com/jordond/jolt.git
cd jolt/cli
cargo build --release
./target/release/jolt
```

### Via Cargo

```bash
cargo install jolt-tui
```

### Homebrew (coming soon)

```bash
brew install jordond/tap/jolt
```

## Usage

```bash
# Run with default settings (1 second refresh)
jolt

# Faster refresh rate (500ms)
jolt --refresh-ms 500

# Force dark theme
jolt --theme dark

# Force light theme
jolt --theme light

# Show help
jolt --help
```

### CLI Options

| Option             | Default | Description                            |
| ------------------ | ------- | -------------------------------------- |
| `-r, --refresh-ms` | 1000    | Refresh interval in milliseconds       |
| `-t, --theme`      | auto    | Theme mode: `auto`, `dark`, or `light` |

## Keyboard Shortcuts

| Key               | Action                              |
| ----------------- | ----------------------------------- |
| `Up` / `k`        | Move selection up                   |
| `Down` / `j`      | Move selection down                 |
| `Enter` / `Space` | Expand/collapse process group       |
| `K`               | Kill selected process               |
| `g`               | Toggle graph metric (battery/power) |
| `m`               | Toggle merge mode (group similar)   |
| `t`               | Open theme picker                   |
| `a`               | Cycle appearance (Auto/Dark/Light)  |
| `o`               | Cycle sort column                   |
| `O`               | Toggle sort direction               |
| `s`               | Open settings                       |
| `h`               | View history                        |
| `PgUp` / `PgDn`   | Page up/down                        |
| `Home` / `End`    | Jump to start/end                   |
| `?`               | Show help dialog                    |
| `q` / `Esc`       | Quit                                |

## Building from Source

### Prerequisites

#### macOS

1. Install Rust via [rustup](https://rustup.rs/):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Ensure you have Xcode Command Line Tools:
   ```bash
   xcode-select --install
   ```

#### Linux

1. Install Rust via [rustup](https://rustup.rs/):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install build dependencies:

   **Ubuntu/Debian:**
   ```bash
   sudo apt-get install build-essential pkg-config
   ```

   **Fedora:**
   ```bash
   sudo dnf install gcc pkg-config
   ```

   **Arch:**
   ```bash
   sudo pacman -S base-devel
   ```

### Build

```bash
# From the cli directory
cd cli

# Debug build (faster compilation, slower runtime)
cargo build
./target/debug/jolt

# Release build (slower compilation, optimized runtime)
cargo build --release
./target/release/jolt
```

### Development

```bash
# Run in development mode
cargo run

# Run with arguments
cargo run -- --refresh-ms 500 --theme dark

# Check for errors without building
cargo check

# Run with warnings
cargo clippy
```

## How it Works

jolt collects system metrics using platform-native APIs:

### macOS
- **Battery Data** - Parsed from `pmset -g batt` and `ioreg -r -c AppleSmartBattery`
- **Power Metrics** - Real-time energy data via IOReport framework (CPU, GPU, ANE power in watts)
- **System Theme** - Detected via `defaults read -g AppleInterfaceStyle`

### Linux
- **Battery Data** - Read from `/sys/class/power_supply/BAT*/`
- **Power Metrics** - Intel RAPL interface at `/sys/class/powercap/intel-rapl/`
- **GPU Power** - hwmon interfaces (Intel/AMD)
- **System Theme** - Follows environment settings

### Cross-Platform
- **Process Info** - Collected via the [sysinfo](https://crates.io/crates/sysinfo) crate
- **Terminal UI** - Built with [ratatui](https://crates.io/crates/ratatui) and [crossterm](https://crates.io/crates/crossterm)

## License

MIT
