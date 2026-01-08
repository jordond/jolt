use crate::config::themes_dir;

pub use jolt_theme::validation::{
    print_validation_results, validate_theme_files, ValidationResult,
};
// Re-export for tests
#[cfg(test)]
pub use jolt_theme::validation::{validate_hex_color, validate_theme_content, ValidationError};

pub fn validate_user_themes() -> Vec<ValidationResult> {
    validate_theme_files(&themes_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hex_color_valid() {
        assert!(validate_hex_color("#ffffff"));
        assert!(validate_hex_color("#FFFFFF"));
        assert!(validate_hex_color("#000000"));
        assert!(validate_hex_color("#abc"));
        assert!(validate_hex_color("#ABC"));
        assert!(validate_hex_color("#1a2b3c"));
        assert!(validate_hex_color("ffffff"));
        assert!(validate_hex_color("  #ffffff  "));
    }

    #[test]
    fn test_validate_hex_color_invalid() {
        assert!(!validate_hex_color("#gggggg"));
        assert!(!validate_hex_color("#12345"));
        assert!(!validate_hex_color("#1234567"));
        assert!(!validate_hex_color(""));
        assert!(!validate_hex_color("#"));
        assert!(!validate_hex_color("not-a-color"));
    }

    #[test]
    fn test_validate_valid_theme() {
        let content = r##"
name = "Test Theme"

[dark]
bg = "#1e1e2e"
dialog_bg = "#313244"
fg = "#cdd6f4"
accent = "#89b4fa"
accent_secondary = "#cba6f7"
highlight = "#f9e2af"
muted = "#6c7086"
success = "#a6e3a1"
warning = "#fab387"
danger = "#f38ba8"
border = "#45475a"
selection_bg = "#585b70"
selection_fg = "#cdd6f4"
graph_line = "#89b4fa"
"##;

        let result = validate_theme_content(content, "test.toml", "test");
        assert!(
            result.is_valid(),
            "Expected valid, got errors: {:?}",
            result.errors
        );
        assert!(result.has_warnings());
    }

    #[test]
    fn test_validate_missing_name() {
        let content = r##"
[dark]
bg = "#1e1e2e"
"##;

        let result = validate_theme_content(content, "test.toml", "test");
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::MissingNameField)));
    }

    #[test]
    fn test_validate_invalid_toml() {
        let content = r##"
name = "Test
broken syntax
"##;

        let result = validate_theme_content(content, "test.toml", "test");
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidToml { .. })));
    }

    #[test]
    fn test_validate_missing_field() {
        let content = r##"
name = "Test Theme"

[dark]
bg = "#1e1e2e"
fg = "#cdd6f4"
"##;

        let result = validate_theme_content(content, "test.toml", "test");
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::MissingField { .. })));
    }

    #[test]
    fn test_validate_invalid_color() {
        let content = r##"
name = "Test Theme"

[dark]
bg = "not-a-color"
dialog_bg = "#313244"
fg = "#cdd6f4"
accent = "#89b4fa"
accent_secondary = "#cba6f7"
highlight = "#f9e2af"
muted = "#6c7086"
success = "#a6e3a1"
warning = "#fab387"
danger = "#f38ba8"
border = "#45475a"
selection_bg = "#585b70"
selection_fg = "#cdd6f4"
graph_line = "#89b4fa"
"##;

        let result = validate_theme_content(content, "test.toml", "test");
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidColor { field, .. } if field == "bg")));
    }

    #[test]
    fn test_validate_no_variants() {
        let content = r##"
name = "Test Theme"
"##;

        let result = validate_theme_content(content, "test.toml", "test");
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::NoVariants)));
    }

    #[test]
    fn test_validate_both_variants_valid() {
        let content = r##"
name = "Full Theme"

[dark]
bg = "#1e1e2e"
dialog_bg = "#313244"
fg = "#cdd6f4"
accent = "#89b4fa"
accent_secondary = "#cba6f7"
highlight = "#f9e2af"
muted = "#6c7086"
success = "#a6e3a1"
warning = "#fab387"
danger = "#f38ba8"
border = "#45475a"
selection_bg = "#585b70"
selection_fg = "#cdd6f4"
graph_line = "#89b4fa"

[light]
bg = "#eff1f5"
dialog_bg = "#e6e9ef"
fg = "#4c4f69"
accent = "#1e66f5"
accent_secondary = "#8839ef"
highlight = "#df8e1d"
muted = "#6c6f85"
success = "#40a02b"
warning = "#fe640b"
danger = "#d20f39"
border = "#bcc0cc"
selection_bg = "#acb0be"
selection_fg = "#4c4f69"
graph_line = "#1e66f5"
"##;

        let result = validate_theme_content(content, "test.toml", "test");
        assert!(
            result.is_valid(),
            "Expected valid, got errors: {:?}",
            result.errors
        );
        assert!(!result.has_warnings());
    }
}
