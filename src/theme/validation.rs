use crate::config::themes_dir;
use std::fmt;
use std::path::Path;

const REQUIRED_COLOR_FIELDS: &[&str] = &[
    "bg",
    "dialog_bg",
    "fg",
    "accent",
    "accent_secondary",
    "highlight",
    "muted",
    "success",
    "warning",
    "danger",
    "border",
    "selection_bg",
    "selection_fg",
    "graph_line",
];

#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidToml {
        message: String,
        line: Option<usize>,
        col: Option<usize>,
    },
    MissingNameField,
    MissingField {
        variant: String,
        field: String,
    },
    InvalidColor {
        variant: String,
        field: String,
        value: String,
    },
    NoVariants,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToml { message, line, col } => {
                if let (Some(l), Some(c)) = (line, col) {
                    write!(f, "Invalid TOML at line {}, col {}: {}", l, c, message)
                } else {
                    write!(f, "Invalid TOML: {}", message)
                }
            }
            Self::MissingNameField => write!(f, "Missing required 'name' field"),
            Self::MissingField { variant, field } => {
                write!(f, "[{}] Missing required field: {}", variant, field)
            }
            Self::InvalidColor {
                variant,
                field,
                value,
            } => {
                write!(
                    f,
                    "[{}] Invalid color for '{}': \"{}\" (expected #RRGGBB hex)",
                    variant, field, value
                )
            }
            Self::NoVariants => {
                write!(f, "Theme must have at least one [dark] or [light] section")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValidationWarning {
    MissingDarkVariant,
    MissingLightVariant,
}

impl fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDarkVariant => {
                write!(
                    f,
                    "Missing [dark] variant (theme will only work in light mode)"
                )
            }
            Self::MissingLightVariant => {
                write!(
                    f,
                    "Missing [light] variant (theme will only work in dark mode)"
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub path: String,
    pub theme_id: String,
    pub theme_name: Option<String>,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

pub fn validate_hex_color(value: &str) -> bool {
    let hex = value.trim().trim_start_matches('#');
    (hex.len() == 3 || hex.len() == 6) && hex.chars().all(|c| c.is_ascii_hexdigit())
}

fn parse_toml_with_location(content: &str) -> Result<toml::Value, ValidationError> {
    toml::from_str(content).map_err(|e| {
        let message = e.message().to_string();
        let span = e.span();

        let (line, col) = if let Some(span) = span {
            let line = content[..span.start].matches('\n').count() + 1;
            let last_newline = content[..span.start]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            let col = span.start - last_newline + 1;
            (Some(line), Some(col))
        } else {
            (None, None)
        };

        ValidationError::InvalidToml { message, line, col }
    })
}

fn validate_variant_colors(
    table: &toml::Value,
    variant_name: &str,
) -> (Vec<ValidationError>, bool) {
    let mut errors = Vec::new();
    let mut has_valid_colors = false;

    let Some(variant) = table.get(variant_name) else {
        return (errors, false);
    };

    let Some(variant_table) = variant.as_table() else {
        errors.push(ValidationError::InvalidToml {
            message: format!("[{}] must be a table", variant_name),
            line: None,
            col: None,
        });
        return (errors, false);
    };

    for &field in REQUIRED_COLOR_FIELDS {
        match variant_table.get(field) {
            None => {
                errors.push(ValidationError::MissingField {
                    variant: variant_name.to_string(),
                    field: field.to_string(),
                });
            }
            Some(value) => {
                if let Some(color_str) = value.as_str() {
                    if validate_hex_color(color_str) {
                        has_valid_colors = true;
                    } else {
                        errors.push(ValidationError::InvalidColor {
                            variant: variant_name.to_string(),
                            field: field.to_string(),
                            value: color_str.to_string(),
                        });
                    }
                } else {
                    errors.push(ValidationError::InvalidColor {
                        variant: variant_name.to_string(),
                        field: field.to_string(),
                        value: format!("{:?}", value),
                    });
                }
            }
        }
    }

    let is_valid = has_valid_colors && errors.is_empty();
    (errors, is_valid)
}

pub fn validate_theme_content(content: &str, path: &str, theme_id: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut theme_name = None;

    let toml_value = match parse_toml_with_location(content) {
        Ok(v) => v,
        Err(e) => {
            return ValidationResult {
                path: path.to_string(),
                theme_id: theme_id.to_string(),
                theme_name: None,
                errors: vec![e],
                warnings: vec![],
            };
        }
    };

    match toml_value.get("name") {
        Some(name_val) => {
            if let Some(name_str) = name_val.as_str() {
                theme_name = Some(name_str.to_string());
            } else {
                errors.push(ValidationError::InvalidToml {
                    message: "'name' must be a string".to_string(),
                    line: None,
                    col: None,
                });
            }
        }
        None => {
            errors.push(ValidationError::MissingNameField);
        }
    }

    let (dark_errors, dark_valid) = validate_variant_colors(&toml_value, "dark");
    let (light_errors, light_valid) = validate_variant_colors(&toml_value, "light");

    let has_dark = toml_value.get("dark").is_some();
    let has_light = toml_value.get("light").is_some();

    errors.extend(dark_errors);
    errors.extend(light_errors);

    if !has_dark && !has_light {
        errors.push(ValidationError::NoVariants);
    } else if !dark_valid && !light_valid && has_dark && has_light {
    }

    if errors.is_empty() {
        if !has_dark {
            warnings.push(ValidationWarning::MissingDarkVariant);
        }
        if !has_light {
            warnings.push(ValidationWarning::MissingLightVariant);
        }
    }

    ValidationResult {
        path: path.to_string(),
        theme_id: theme_id.to_string(),
        theme_name,
        errors,
        warnings,
    }
}

pub fn validate_theme_files(dir: &Path) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    if !dir.exists() {
        return results;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return results;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "toml").unwrap_or(false) {
            let theme_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let path_str = path.display().to_string();

            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    results.push(validate_theme_content(&content, &path_str, &theme_id));
                }
                Err(e) => {
                    results.push(ValidationResult {
                        path: path_str,
                        theme_id,
                        theme_name: None,
                        errors: vec![ValidationError::InvalidToml {
                            message: format!("Could not read file: {}", e),
                            line: None,
                            col: None,
                        }],
                        warnings: vec![],
                    });
                }
            }
        }
    }

    results.sort_by(|a, b| a.theme_id.cmp(&b.theme_id));
    results
}

pub fn validate_user_themes() -> Vec<ValidationResult> {
    validate_theme_files(&themes_dir())
}

pub fn print_validation_results(results: &[ValidationResult], verbose: bool) {
    let errors: Vec<_> = results.iter().filter(|r| !r.is_valid()).collect();
    let warnings: Vec<_> = results
        .iter()
        .filter(|r| r.is_valid() && r.has_warnings())
        .collect();
    let valid: Vec<_> = results
        .iter()
        .filter(|r| r.is_valid() && !r.has_warnings())
        .collect();

    println!("{}", "=".repeat(80));
    println!("THEME VALIDATION");
    println!("{}", "=".repeat(80));

    if !errors.is_empty() {
        println!("\nX ERRORS ({} theme(s) with issues)\n", errors.len());

        for result in &errors {
            let display_name = result.theme_name.as_deref().unwrap_or(&result.theme_id);
            println!("{}:", display_name);

            if result.path != result.theme_id {
                println!("  File: {}", result.path);
            }

            for error in &result.errors {
                println!("  * {}", error);
            }
            println!();
        }
    }

    if !warnings.is_empty() {
        println!("\n! WARNINGS ({} theme(s))\n", warnings.len());

        for result in &warnings {
            let display_name = result.theme_name.as_deref().unwrap_or(&result.theme_id);
            println!("{}:", display_name);

            for warning in &result.warnings {
                println!("  * {}", warning);
            }
            println!();
        }
    }

    if verbose && !valid.is_empty() {
        println!("\n+ VALID ({} theme(s))\n", valid.len());

        for result in &valid {
            let display_name = result.theme_name.as_deref().unwrap_or(&result.theme_id);
            println!("  {}", display_name);
        }
        println!();
    }
    println!("{}", "=".repeat(80));
    println!("VALIDATION SUMMARY");
    println!("{}", "=".repeat(80));
    println!(
        "\nThemes checked: {} | Valid: {} | Errors: {} | Warnings: {}",
        results.len(),
        valid.len() + warnings.len(),
        errors.len(),
        warnings.len()
    );

    if errors.is_empty() && warnings.is_empty() {
        println!("\n+ All themes passed validation!");
    } else if errors.is_empty() {
        println!("\n+ All themes are valid (with some warnings)");
    }
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
