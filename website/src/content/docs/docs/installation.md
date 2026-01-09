---
title: Installation
description: How to install jolt on macOS or Linux
---

jolt is available for macOS and Linux through multiple installation methods.

## Requirements

### macOS

- macOS 11.0 (Big Sur) or later
- Apple Silicon (M1/M2/M3/M4) or Intel Mac
- Terminal emulator with 256-color support

### Linux

- Linux kernel 3.13+ (for RAPL power metrics)
- Laptop with battery
- Intel or AMD CPU
- Terminal emulator with 256-color support

:::note[Platform-Specific Features]

- **macOS (Apple Silicon):** Full power metrics (CPU, GPU, ANE watts)
- **macOS (Intel):** Battery data only, no power breakdown
- **Linux:** Full power metrics via RAPL (requires permissions)
  :::

## Install Script (Recommended)

The easiest way to install jolt on any platform:

```bash
curl -fsSL https://getjolt.sh/install.sh | bash
```

This script automatically:

- Detects your OS (macOS or Linux) and architecture
- Downloads the correct binary for your platform
- Verifies checksums
- Installs to `~/.local/bin`

## Homebrew (Coming Soon)

Homebrew support is planned but not yet available:

```bash
# Future release
brew install jordond/tap/jolt
```

## Cargo

If you have Rust installed (works on both macOS and Linux):

```bash
cargo install jolt-tui
```

## From Source

For the latest development version:

```bash
git clone https://github.com/jordond/jolt.git
cd jolt
cargo build --release
```

The binary will be at `./target/release/jolt`. You can copy it to your PATH:

**macOS/Linux:**

```bash
cp ./target/release/jolt /usr/local/bin/
# or
cp ./target/release/jolt ~/.local/bin/
```

See [cli/README.md](https://github.com/jordond/jolt/blob/main/cli/README.md) for platform-specific build dependencies.

## Verify Installation

After installing, verify jolt is working:

```bash
jolt --version
```

Then launch the TUI:

```bash
jolt
```

## Linux Setup

On Linux, you'll need to grant permissions for power metrics. See the [Linux Permissions](#linux-permissions) section below or the [Troubleshooting](/docs/troubleshooting/) page.

### Linux Permissions

Power consumption metrics on Linux require read access to RAPL interfaces. Choose one option:

#### Option 1: Udev Rule (Recommended)

Create a persistent rule to make RAPL readable:

```bash
sudo tee /etc/udev/rules.d/99-rapl.rules << 'EOF'
SUBSYSTEM=="powercap", ACTION=="add", RUN+="/bin/sh -c 'chmod o+r /sys/class/powercap/intel-rapl/*/energy_uj /sys/class/powercap/intel-rapl/*/*/energy_uj 2>/dev/null || true'"
EOF

sudo udevadm control --reload-rules
sudo udevadm trigger
```

#### Option 2: Manual Permissions (Temporary)

Set permissions manually (resets on reboot):

```bash
sudo chmod o+r /sys/class/powercap/intel-rapl/*/energy_uj
sudo chmod o+r /sys/class/powercap/intel-rapl/*/*/energy_uj
```

#### Verify Setup

Check if you can read power metrics:

```bash
cat /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj
```

If successful, you'll see a number. If you get "Permission denied", try the steps above again.

For detailed Linux setup information, see [docs/linux-setup.md](https://github.com/jordond/jolt/blob/main/docs/linux-setup.md).

## Next Steps

Continue to [Quick Start](/docs/quick-start/) for a 5-minute tour of jolt's features.
