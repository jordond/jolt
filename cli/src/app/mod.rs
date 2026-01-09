//! Application core module.
//!
//! This module contains the main App struct and related types for the TUI application.

mod actions;
mod daemon;
mod history;
mod process;
mod theme;
pub mod types;

pub use types::{Action, AppView, HistoryPeriod, SortColumn};
pub use types::{MAX_REFRESH_MS, MIN_REFRESH_MS, REFRESH_STEP_MS};
