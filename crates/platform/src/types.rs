//! Shared types for battery and power monitoring.

use std::fmt;

/// Battery charging state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChargeState {
    /// Battery is actively charging
    Charging,
    /// Battery is discharging (on battery power)
    Discharging,
    /// Battery is full
    Full,
    /// External power connected but not charging (e.g., charge limit reached)
    NotCharging,
    /// State cannot be determined
    #[default]
    Unknown,
}

impl ChargeState {
    /// Returns a human-readable label for the charge state.
    pub fn label(&self) -> &'static str {
        match self {
            ChargeState::Charging => "Charging",
            ChargeState::Discharging => "On Battery",
            ChargeState::Full => "Full",
            ChargeState::NotCharging => "Not Charging",
            ChargeState::Unknown => "Unknown",
        }
    }

    /// Returns true if the battery is currently charging.
    pub fn is_charging(&self) -> bool {
        matches!(self, ChargeState::Charging)
    }

    /// Returns true if external power is connected.
    pub fn is_plugged_in(&self) -> bool {
        matches!(
            self,
            ChargeState::Charging | ChargeState::Full | ChargeState::NotCharging
        )
    }
}

impl fmt::Display for ChargeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl From<starship_battery::State> for ChargeState {
    fn from(state: starship_battery::State) -> Self {
        match state {
            starship_battery::State::Charging => ChargeState::Charging,
            starship_battery::State::Discharging => ChargeState::Discharging,
            starship_battery::State::Empty => ChargeState::Discharging,
            starship_battery::State::Full => ChargeState::Full,
            starship_battery::State::Unknown => ChargeState::Unknown,
        }
    }
}

/// System power mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PowerMode {
    /// Low power / battery saver mode
    LowPower,
    /// Automatic / balanced mode
    #[default]
    Automatic,
    /// High performance mode
    HighPerformance,
    /// Mode cannot be determined
    Unknown,
}

impl PowerMode {
    /// Returns a human-readable label for the power mode.
    pub fn label(&self) -> &'static str {
        match self {
            PowerMode::LowPower => "Low Power",
            PowerMode::Automatic => "Automatic",
            PowerMode::HighPerformance => "High Performance",
            PowerMode::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for PowerMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Battery technology/chemistry type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BatteryTechnology {
    /// Lithium-ion
    LithiumIon,
    /// Lithium-polymer
    LithiumPolymer,
    /// Nickel-metal hydride
    NickelMetalHydride,
    /// Nickel-cadmium
    NickelCadmium,
    /// Lead-acid
    LeadAcid,
    /// Unknown or unsupported technology
    #[default]
    Unknown,
}

impl BatteryTechnology {
    /// Returns a human-readable label for the battery technology.
    pub fn label(&self) -> &'static str {
        match self {
            BatteryTechnology::LithiumIon => "Li-ion",
            BatteryTechnology::LithiumPolymer => "Li-poly",
            BatteryTechnology::NickelMetalHydride => "NiMH",
            BatteryTechnology::NickelCadmium => "NiCd",
            BatteryTechnology::LeadAcid => "Lead-acid",
            BatteryTechnology::Unknown => "Unknown",
        }
    }

    /// Returns a longer description of the battery technology.
    pub fn description(&self) -> &'static str {
        match self {
            BatteryTechnology::LithiumIon => "Lithium-ion",
            BatteryTechnology::LithiumPolymer => "Lithium-polymer",
            BatteryTechnology::NickelMetalHydride => "Nickel-metal hydride",
            BatteryTechnology::NickelCadmium => "Nickel-cadmium",
            BatteryTechnology::LeadAcid => "Lead-acid",
            BatteryTechnology::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for BatteryTechnology {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl From<starship_battery::Technology> for BatteryTechnology {
    fn from(tech: starship_battery::Technology) -> Self {
        match tech {
            starship_battery::Technology::LithiumIon => BatteryTechnology::LithiumIon,
            starship_battery::Technology::LithiumPolymer => BatteryTechnology::LithiumPolymer,
            starship_battery::Technology::NickelMetalHydride => {
                BatteryTechnology::NickelMetalHydride
            }
            starship_battery::Technology::NickelCadmium => BatteryTechnology::NickelCadmium,
            starship_battery::Technology::LeadAcid => BatteryTechnology::LeadAcid,
            starship_battery::Technology::Unknown => BatteryTechnology::Unknown,
            _ => BatteryTechnology::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charge_state_labels() {
        assert_eq!(ChargeState::Charging.label(), "Charging");
        assert_eq!(ChargeState::Discharging.label(), "On Battery");
        assert_eq!(ChargeState::Full.label(), "Full");
        assert_eq!(ChargeState::NotCharging.label(), "Not Charging");
        assert_eq!(ChargeState::Unknown.label(), "Unknown");
    }

    #[test]
    fn test_charge_state_is_plugged_in() {
        assert!(ChargeState::Charging.is_plugged_in());
        assert!(ChargeState::Full.is_plugged_in());
        assert!(ChargeState::NotCharging.is_plugged_in());
        assert!(!ChargeState::Discharging.is_plugged_in());
        assert!(!ChargeState::Unknown.is_plugged_in());
    }

    #[test]
    fn test_battery_state_conversion() {
        assert_eq!(
            ChargeState::from(starship_battery::State::Charging),
            ChargeState::Charging
        );
        assert_eq!(
            ChargeState::from(starship_battery::State::Discharging),
            ChargeState::Discharging
        );
        assert_eq!(
            ChargeState::from(starship_battery::State::Full),
            ChargeState::Full
        );
        assert_eq!(
            ChargeState::from(starship_battery::State::Empty),
            ChargeState::Discharging
        );
        assert_eq!(
            ChargeState::from(starship_battery::State::Unknown),
            ChargeState::Unknown
        );
    }

    #[test]
    fn test_power_mode_labels() {
        assert_eq!(PowerMode::LowPower.label(), "Low Power");
        assert_eq!(PowerMode::Automatic.label(), "Automatic");
        assert_eq!(PowerMode::HighPerformance.label(), "High Performance");
        assert_eq!(PowerMode::Unknown.label(), "Unknown");
    }
}
