//! Power monitoring traits and types.

use color_eyre::eyre::Result;

use crate::types::PowerMode;

/// Power information snapshot.
///
/// All power values are in watts.
#[derive(Debug, Clone, Default)]
pub struct PowerInfo {
    /// CPU package power consumption in watts.
    pub cpu_power_watts: f32,

    /// GPU power consumption in watts.
    pub gpu_power_watts: f32,

    /// Total system power consumption in watts.
    /// This may be measured (SMC/RAPL) or estimated (sum of components).
    pub system_power_watts: f32,

    /// Current power mode.
    pub power_mode: PowerMode,

    /// Whether enough samples have been collected for reliable readings.
    /// Power readings may be unstable during the first few samples.
    pub is_warmed_up: bool,
}

impl PowerInfo {
    /// Get combined CPU + GPU power.
    pub fn package_power_watts(&self) -> f32 {
        self.cpu_power_watts + self.gpu_power_watts
    }
}

/// Trait for platform-specific power providers.
pub trait PowerProvider {
    /// Create a new power provider instance.
    fn new() -> Result<Self>
    where
        Self: Sized;

    /// Refresh power information from the system.
    fn refresh(&mut self) -> Result<()>;

    /// Get the current power information.
    fn info(&self) -> &PowerInfo;

    /// Check if power monitoring is supported on this system.
    ///
    /// Returns false if the required hardware/permissions are not available.
    fn is_supported() -> bool
    where
        Self: Sized,
    {
        true
    }
}
