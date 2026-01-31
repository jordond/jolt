//! Protocol versioning for daemon IPC communication.
//!
//! # Version History
//!
//! | Version | Changes |
//! |---------|---------|
//! | 1 | Initial protocol version |
//! | 2 | Added `os_name` to SystemSnapshot, forecast fields |
//!
//! # Breaking Changes (require PROTOCOL_VERSION bump)
//!
//! - Removing fields from request/response types
//! - Changing field types
//! - Renaming fields without `#[serde(alias)]`
//! - Removing enum variants
//!
//! # Non-Breaking Changes (safe without version bump)
//!
//! - Adding new optional fields with `#[serde(default)]`
//! - Adding new request/response variants
//! - Adding new enum variants
//!
//! # Support Policy
//!
//! We maintain N-1 backwards compatibility, meaning the current version
//! supports communication with the previous version. When updating:
//!
//! 1. Bump `PROTOCOL_VERSION` for breaking changes
//! 2. Keep `MIN_SUPPORTED_VERSION` one behind to allow gradual upgrades
//! 3. Only bump `MIN_SUPPORTED_VERSION` when dropping support for old versions

/// Current protocol version. Bump when making breaking changes.
pub const PROTOCOL_VERSION: u32 = 2;

/// Minimum protocol version this build can communicate with.
/// Kept at N-1 to allow one version of backwards compatibility.
pub const MIN_SUPPORTED_VERSION: u32 = 1;
