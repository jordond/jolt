# Contributing to jolt

Thanks for your interest in contributing! This document outlines how to get started.

## Development Setup

### Prerequisites

- Rust 1.70 or newer
- macOS 11+ or Linux with kernel 3.13+

### Getting Started

```bash
# Clone the repository
git clone https://github.com/jordond/jolt.git
cd jolt

# Build
cargo build

# Run
cargo run

# Run tests
cargo test

# Run all checks (format, lint, build)
./scripts/check
```

## Making Changes

### Branch Naming

Use descriptive branch names:

- `feat/add-new-widget` — New features
- `fix/battery-parsing` — Bug fixes
- `docs/update-readme` — Documentation
- `refactor/cleanup-config` — Code improvements

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```text
feat: add power graph smoothing
fix: correct battery percentage calculation
docs: update installation instructions
refactor: simplify config loading
test: add battery parser tests
```

### Code Style

- Run `cargo fmt` before committing
- Ensure `cargo clippy -- -D warnings` passes
- Follow existing patterns in the codebase

### Pre-commit Checks

Run the check script before opening a PR:

```bash
./scripts/check
```

## Pull Requests

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Ensure all checks pass
5. Open a PR with a clear description

### PR Guidelines

- Keep PRs focused — one feature or fix per PR
- Update documentation if needed
- Add tests for new functionality
- Reference related issues

## Project Structure

```shell
jolt/
├── cli/                # Main application
│   └── src/
│       ├── main.rs     # Entry point
│       ├── app.rs      # Application state
│       ├── data/       # Data collection
│       ├── daemon/     # Background service
│       ├── theme/      # Theme system
│       └── ui/         # Terminal UI widgets
├── crates/
│   ├── platform/       # Platform-specific code
│   ├── protocol/       # IPC protocol
│   └── theme/          # Theme types
├── docs/               # Documentation
└── scripts/            # Development scripts
```

## Adding Features

### New UI Widget

1. Create `cli/src/ui/your_widget.rs`
2. Implement the render function:
   ```rust
   pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
       // ...
   }
   ```
3. Add to `cli/src/ui/mod.rs`
4. Call from main render function

### New Theme

1. Create `cli/src/theme/themes/your_theme.toml`
2. Include `[dark]` and/or `[light]` sections
3. Theme is auto-loaded on startup

### New Config Option

1. Add field to `UserConfig` in `cli/src/config.rs`
2. Use `#[serde(default)]` for backwards compatibility
3. Add UI in `cli/src/ui/config_editor.rs` if needed

## Getting Help

- Open an [issue](https://github.com/jordond/jolt/issues) for bugs or feature requests
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
