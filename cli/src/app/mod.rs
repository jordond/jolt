//! Application core module.
//!
//! This module contains the main App struct and related types for the TUI application.

// Submodules will be added as they are created:
// pub mod actions;
mod daemon;
mod history;
mod process;
mod theme;
pub mod types;

// Re-exports will be added as submodules are created:
// pub use actions::*;
pub use types::{Action, AppView, HistoryPeriod, SortColumn};
pub use types::{MAX_REFRESH_MS, MIN_REFRESH_MS, REFRESH_STEP_MS};
