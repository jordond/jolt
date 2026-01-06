# jolt

A beautiful terminal-based battery and energy monitor for macOS.

Built for MacBook users who want to understand what's draining their battery. Provides real-time insights into power consumption, process energy usage, and battery health - all in a clean, themeable TUI.

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

- macOS (optimized for Apple Silicon, works on Intel)
- Rust 1.70 or newer

## Installation

### Homebrew (coming soon)

```bash
brew install jordond/tap/jolt
```

### From Source

See [cli/README.md](./cli/README.md) for detailed build instructions.

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
