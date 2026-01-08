//! Cross-platform battery and power monitoring for jolt.
//!
//! This crate provides platform-agnostic traits and types for accessing
//! battery and power information, with platform-specific implementations.
//!
//! # Features
//!
//! - `macos` - Enable macOS support (IOReport, SMC, ioreg)
//! - `linux` - Enable Linux support (RAPL, sysfs)
//!
//! # Example
//!
//! ```ignore
//! use jolt_platform::{BatteryProvider, PowerProvider};
//!
//! #[cfg(target_os = "macos")]
//! use jolt_platform::macos::{MacOSBattery, MacOSPower};
//!
//! let mut battery = MacOSBattery::new()?;
//! battery.refresh()?;
//! println!("Charge: {}%", battery.info().charge_percent);
//! ```

mod battery;
mod power;
mod types;

pub use battery::{BatteryInfo, BatteryProvider};
pub use power::{PowerInfo, PowerProvider};
pub use types::{ChargeState, PowerMode};

#[cfg(target_os = "macos")]
#[cfg(feature = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
#[cfg(feature = "linux")]
pub mod linux;

/// Re-export battery crate units for convenience
pub mod units {
    pub use ::battery::units::electric_potential::millivolt;
    pub use ::battery::units::energy::watt_hour;
    pub use ::battery::units::power::watt;
    pub use ::battery::units::ratio::percent;
    pub use ::battery::units::thermodynamic_temperature::degree_celsius;
    pub use ::battery::units::time::second;
}
