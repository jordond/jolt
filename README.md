# jolt

A beautiful terminal-based battery and energy monitor for macOS and Linux.

Built for laptop users who want to understand what's draining their battery. Provides real-time insights into power consumption, process energy usage, and battery health - all in a clean, themeable TUI.

## Packages

| Package | Description |
| ------- | ----------- |
| [cli](./cli) | Terminal UI application |
| [website](./website) | Documentation website |

## Quick Start

```bash
# Clone the repository
git clone https://github.com/jordond/jolt.git
cd jolt

# Build and run the CLI
cd cli
cargo build --release
./target/release/jolt
```

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
- Apple Silicon (M1/M2/M3/M4) or Intel Mac
- Rust 1.70 or newer

### Linux
- Linux kernel 3.13+ (for RAPL power metrics)
- Laptop with battery
- Intel or AMD CPU (for power metrics)
- Rust 1.70 or newer

See [docs/linux-setup.md](./docs/linux-setup.md) for Linux-specific setup instructions.

## Installation

### Install Script (Recommended)

Automatically detects your platform and downloads the correct binary:

```bash
curl -fsSL https://getjolt.sh/install.sh | bash
```

### Homebrew (coming soon)

```bash
brew install jordond/tap/jolt
```

### Cargo

```bash
cargo install jolt-tui
```

### From Source

See [cli/README.md](./cli/README.md) for detailed build instructions.

## Platform Notes

- **macOS**: Power metrics work on Apple Silicon. Intel Macs show battery data only.
- **Linux**: Requires permissions for power metrics (RAPL). See [Linux setup guide](./docs/linux-setup.md).

## Contributing

1. Fork and clone the repo
2. Create a branch and make your changes
3. Ensure your code passes checks:
   ```bash
   cd cli
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo build
   ```
4. Commit using [conventional commits](https://www.conventionalcommits.org/) (feat, fix, docs, etc)
5. Open a PR targeting `main`

## License

MIT
