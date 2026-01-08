//! Core theme types.
//!
//! This module defines the fundamental types for the theme system:
//! - `Color` - RGB color representation
//! - `ThemeColors` - The 14 semantic colors used in the UI
//! - `ThemeVariants` - Dark and light variants of a theme
//! - `NamedTheme` - A complete theme with metadata

use serde::{Deserialize, Serialize};

/// RGB color representation.
///
/// Each component is a value from 0-255.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color from RGB components.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a hex color string (e.g., "#ffffff" or "ffffff").
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    /// Convert to hex string (e.g., "#ffffff").
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Convert sRGB channel to linear for luminance calculation.
    fn linearize(val: u8) -> f64 {
        let v = val as f64 / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Calculate relative luminance (0.0 = black, 1.0 = white).
    pub fn luminance(&self) -> f64 {
        0.2126 * Self::linearize(self.r)
            + 0.7152 * Self::linearize(self.g)
            + 0.0722 * Self::linearize(self.b)
    }

    /// Calculate WCAG contrast ratio between two colors (1:1 to 21:1).
    pub fn contrast_ratio(&self, other: &Color) -> f64 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(128, 128, 128)
    }
}

/// The 14 semantic colors used in the UI.
///
/// These colors define the visual appearance of all UI elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    /// Main background color
    pub bg: Color,
    /// Dialog/modal background color
    pub dialog_bg: Color,
    /// Primary foreground/text color
    pub fg: Color,
    /// Primary accent color (links, focus)
    pub accent: Color,
    /// Secondary accent color
    pub accent_secondary: Color,
    /// Highlight color (warnings, emphasis)
    pub highlight: Color,
    /// Muted/dimmed text color
    pub muted: Color,
    /// Success state color
    pub success: Color,
    /// Warning state color
    pub warning: Color,
    /// Danger/error state color
    pub danger: Color,
    /// Border color
    pub border: Color,
    /// Selection background color
    pub selection_bg: Color,
    /// Selection foreground color
    pub selection_fg: Color,
    /// Graph line color
    pub graph_line: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        // Default dark theme
        Self {
            bg: Color::from_hex("#16161e").unwrap(),
            dialog_bg: Color::from_hex("#23232d").unwrap(),
            fg: Color::from_hex("#e6e6f0").unwrap(),
            accent: Color::from_hex("#8ab4f8").unwrap(),
            accent_secondary: Color::from_hex("#bb86fc").unwrap(),
            highlight: Color::from_hex("#ffcb6b").unwrap(),
            muted: Color::from_hex("#80808c").unwrap(),
            success: Color::from_hex("#81c784").unwrap(),
            warning: Color::from_hex("#ffb74d").unwrap(),
            danger: Color::from_hex("#ef5350").unwrap(),
            border: Color::from_hex("#3c3c50").unwrap(),
            selection_bg: Color::from_hex("#323246").unwrap(),
            selection_fg: Color::from_hex("#ffffff").unwrap(),
            graph_line: Color::from_hex("#8ab4f8").unwrap(),
        }
    }
}

/// Dark and light variants of a theme.
#[derive(Debug, Clone, Default)]
pub struct ThemeVariants {
    pub dark: Option<ThemeColors>,
    pub light: Option<ThemeColors>,
}

/// A complete theme with metadata.
#[derive(Debug, Clone)]
pub struct NamedTheme {
    /// Unique identifier (filename without extension)
    pub id: String,
    /// Display name
    pub name: String,
    /// Theme color variants
    pub variants: ThemeVariants,
    /// Whether this is a built-in theme
    pub is_builtin: bool,
}

impl NamedTheme {
    /// Get the appropriate colors for the current appearance mode.
    pub fn get_colors(&self, is_dark: bool) -> ThemeColors {
        if is_dark {
            self.variants
                .dark
                .or(self.variants.light)
                .expect("Theme must have at least one variant")
        } else {
            self.variants
                .light
                .or(self.variants.dark)
                .expect("Theme must have at least one variant")
        }
    }

    /// Check if the theme has a dark variant.
    pub fn has_dark(&self) -> bool {
        self.variants.dark.is_some()
    }

    /// Check if the theme has a light variant.
    pub fn has_light(&self) -> bool {
        self.variants.light.is_some()
    }

    /// Get a label describing available variants.
    pub fn variants_label(&self) -> &'static str {
        match (self.has_dark(), self.has_light()) {
            (true, true) => "dark + light",
            (true, false) => "dark only",
            (false, true) => "light only",
            _ => "unknown",
        }
    }
}
