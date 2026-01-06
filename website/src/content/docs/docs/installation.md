---
title: Installation
description: How to install jolt on macOS
---

# Installation

jolt is available through multiple installation methods.

## Requirements

- macOS 11.0 (Big Sur) or later
- Apple Silicon (M1/M2/M3/M4) or Intel Mac
- Terminal emulator with 256-color support

:::note
Power metrics (CPU/GPU watts) require Apple Silicon. Intel Macs will show battery data but not detailed power breakdown.
:::

## Homebrew (Recommended)

The easiest way to install jolt:

```bash
brew install jordond/tap/jolt
```

To update:

```bash
brew upgrade jolt
```

## Cargo

If you have Rust installed:

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

```bash
cp ./target/release/jolt /usr/local/bin/
```

## Verify Installation

After installing, verify jolt is working:

```bash
jolt --version
```

Then launch the TUI:

```bash
jolt
```

## Next Steps

Continue to [Quick Start](/jolt/docs/quick-start/) for a 5-minute tour of jolt's features.
