<div align="center">

# ⚡️ jolt

**A terminal-based battery and energy monitor for macOS and Linux.**

[![Release](https://img.shields.io/github/v/release/jordond/jolt?style=flat-square&label=stable)](https://github.com/jordond/jolt/releases/latest)
[![Pre-release](https://img.shields.io/github/v/release/jordond/jolt?style=flat-square&include_prereleases&label=pre-release)](https://github.com/jordond/jolt/releases)

[![Docs](https://img.shields.io/badge/docs-getjolt.sh-blue?style=flat-square)](https://getjolt.sh/docs)
[![CI](https://img.shields.io/github/actions/workflow/status/jordond/jolt/ci.yml?style=flat-square&label=ci)](https://github.com/jordond/jolt/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Contributing](#contributing)

</div>

<p align="center">
  <img src="docs/demo.gif" alt="jolt demo" width="1000">
</p>

## About

`jolt` helps laptop users understand what's draining their battery. It provides real-time insights into power consumption, process energy usage, and battery health—all in a clean, themeable TUI.

## Features

- **Battery Status** — Charge percentage, time remaining, health, and cycle count
- **Power Monitoring** — System power draw with CPU/GPU breakdown
- **Process Tracking** — Processes sorted by energy impact with color-coded severity
- **Historical Graphs** — Track battery and power trends over time
- **Themes** — 10+ built-in themes with dark/light auto-detection
- **Background Daemon** — Collect historical data even when the TUI isn't running
- **Process Management** — Kill energy-hungry processes directly

## Installation

### Quick Install

```shell
curl -fsSL https://getjolt.sh/install.sh | bash
```

### Homebrew

```shell
brew install jordond/tap/jolt
```

### Cargo

```shell
cargo install jolt-tui
```

### Nix

Run directly:
```shell
nix run github:jordond/jolt
```

Install to profile:
```shell
nix profile install github:jordond/jolt
```

Add to `configuration.nix`:
```nix
environment.systemPackages = [
  (builtins.getFlake "github:jordond/jolt").packages.${pkgs.system}.default
];
```

### From Source

```shell
git clone https://github.com/jordond/jolt.git
cd jolt
cargo build --release
./target/release/jolt
```

See [Building from Source](#building-from-source) for detailed instructions.

## Usage

Start jolt's TUI:

```shell
jolt
```

```shell
A beautiful battery and energy monitor for your terminal

Usage: jolt [OPTIONS] [COMMAND]

Commands:
  ui       Launch the terminal UI (default)
  pipe     Output metrics as JSON for scripting
  debug    Print system and battery debug info
  config   Manage configuration
  theme    Manage themes
  daemon   Control the background daemon
  history  View and export historical data
  logs     View daemon logs
  help     Print this message or the help of the given subcommand(s)

Options:
      --log-level <LOG_LEVEL>  Set log level (error, warn, info, debug, trace)
  -h, --help                   Print help
  -V, --version                Print version
```

### Keyboard Shortcuts

| Key       | Action                        |
| --------- | ----------------------------- |
| `j` / `k` | Navigate up/down              |
| `Enter`   | Expand/collapse process group |
| `K`       | Kill selected process         |
| `g`       | Toggle graph (battery/power)  |
| `t`       | Open theme picker             |
| `s`       | Open settings                 |
| `h`       | View history                  |
| `?`       | Show help                     |
| `q`       | Quit                          |

### Daemon

jolt includes a background daemon for collecting historical data:

```shell
# Start the daemon
jolt daemon start

# Check status
jolt daemon status

# Stop the daemon
jolt daemon stop
```

## Platform Support

| Platform              | Battery | Power Metrics | Notes                     |
| --------------------- | ------- | ------------- | ------------------------- |
| macOS (Apple Silicon) | ✅      | ✅            | Full support              |
| macOS (Intel)         | ✅      | ❌            | Battery data only         |
| Linux                 | ✅      | ✅            | Requires RAPL permissions |

See the [Linux setup guide](docs/linux-setup.md) for configuring power metrics on Linux.

## Building from Source

### Prerequisites

You will need to install Rust, see the [install docs](https://rust-lang.org/learn/get-started/)

### Build

```shell
git clone https://github.com/jordond/jolt.git
cd jolt

# Debug build
cargo build
./target/debug/jolt

# Release build (optimized)
cargo build --release
./target/release/jolt
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT — See [LICENSE](LICENSE) for details.

---

<div align="center">

**[Documentation](https://getjolt.sh)** • **[Report Bug](https://github.com/jordond/jolt/issues)** • **[Request Feature](https://github.com/jordond/jolt/issues)**

</div>

---

<sub>Built with the assistance of AI tools including Claude and GitHub Copilot.</sub>
