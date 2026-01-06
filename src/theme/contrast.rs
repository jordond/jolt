use super::{NamedTheme, ThemeColors};
use ratatui::style::Color;

const AA_NORMAL: f64 = 4.5;
const AA_LARGE: f64 = 3.0;
const AAA_NORMAL: f64 = 7.0;

pub struct ContrastResult {
    pub theme_name: String,
    pub variant: String,
    pub pair: String,
    pub bg_hex: String,
    pub fg_hex: String,
    pub ratio: f64,
    pub grade: String,
    pub pass: bool,
}

fn color_to_hex(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
        _ => "#??????".to_string(),
    }
}

fn extract_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (128, 128, 128),
    }
}

fn linearize(val: u8) -> f64 {
    let v = val as f64 / 255.0;
    if v <= 0.03928 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(color: Color) -> f64 {
    let (r, g, b) = extract_rgb(color);
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

fn contrast_ratio(c1: Color, c2: Color) -> f64 {
    let l1 = luminance(c1);
    let l2 = luminance(c2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

fn wcag_grade(ratio: f64) -> String {
    if ratio >= AAA_NORMAL {
        "AAA".to_string()
    } else if ratio >= AA_NORMAL {
        "AA".to_string()
    } else if ratio >= AA_LARGE {
        "AA-large".to_string()
    } else {
        "FAIL".to_string()
    }
}

fn check_theme_colors(
    theme_name: &str,
    variant: &str,
    colors: &ThemeColors,
) -> Vec<ContrastResult> {
    let checks = [
        ("bg", "fg", colors.bg, colors.fg),
        ("bg", "accent", colors.bg, colors.accent),
        ("bg", "accent_secondary", colors.bg, colors.accent_secondary),
        ("bg", "muted", colors.bg, colors.muted),
        ("bg", "success", colors.bg, colors.success),
        ("bg", "warning", colors.bg, colors.warning),
        ("bg", "danger", colors.bg, colors.danger),
        ("bg", "highlight", colors.bg, colors.highlight),
        ("dialog_bg", "fg", colors.dialog_bg, colors.fg),
        ("dialog_bg", "accent", colors.dialog_bg, colors.accent),
        (
            "selection_bg",
            "selection_fg",
            colors.selection_bg,
            colors.selection_fg,
        ),
    ];

    checks
        .iter()
        .map(|(bg_name, fg_name, bg, fg)| {
            let ratio = contrast_ratio(*bg, *fg);
            let grade = wcag_grade(ratio);
            let pass = ratio >= AA_NORMAL;

            ContrastResult {
                theme_name: theme_name.to_string(),
                variant: variant.to_string(),
                pair: format!("{} ↔ {}", bg_name, fg_name),
                bg_hex: color_to_hex(*bg),
                fg_hex: color_to_hex(*fg),
                ratio,
                grade,
                pass,
            }
        })
        .collect()
}

pub fn check_all_themes(themes: &[NamedTheme]) -> Vec<ContrastResult> {
    let mut results = Vec::new();

    for theme in themes {
        if let Some(ref dark) = theme.variants.dark {
            results.extend(check_theme_colors(&theme.name, "dark", dark));
        }
        if let Some(ref light) = theme.variants.light {
            results.extend(check_theme_colors(&theme.name, "light", light));
        }
    }

    results
}

pub fn print_results(results: &[ContrastResult], verbose: bool) {
    let failures: Vec<_> = results.iter().filter(|r| !r.pass).collect();
    let passes: Vec<_> = results.iter().filter(|r| r.pass).collect();

    println!("{}", "=".repeat(80));
    println!("WCAG CONTRAST CHECK RESULTS");
    println!("Target: {}:1 (WCAG AA Normal Text)", AA_NORMAL);
    println!("{}", "=".repeat(80));

    if failures.is_empty() {
        println!(
            "\n✅ All {} color pairs pass WCAG AA requirements!",
            results.len()
        );
    } else {
        println!("\n❌ FAILURES ({} issues)\n", failures.len());
        println!(
            "{:<20} {:<8} {:<30} {:>8} {:<10}",
            "Theme", "Variant", "Pair", "Ratio", "Grade"
        );
        println!("{}", "-".repeat(80));

        let mut current_theme = String::new();
        for r in &failures {
            let theme_label = if r.theme_name != current_theme {
                current_theme = r.theme_name.clone();
                &r.theme_name
            } else {
                ""
            };
            println!(
                "{:<20} {:<8} {:<30} {:>7.2}:1 {:<10}",
                theme_label, r.variant, r.pair, r.ratio, r.grade
            );
            println!("{:20} {:8} bg: {}  fg: {}", "", "", r.bg_hex, r.fg_hex);
        }
    }

    if verbose && !passes.is_empty() {
        println!("\n✅ PASSING ({} checks)\n", passes.len());
        for r in &passes {
            println!(
                "{:<20} {:<8} {:<30} {:>7.2}:1 {}",
                r.theme_name, r.variant, r.pair, r.ratio, r.grade
            );
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("SUMMARY BY THEME");
    println!("{}", "=".repeat(80));

    let mut theme_stats: std::collections::HashMap<(String, String), (usize, usize)> =
        std::collections::HashMap::new();

    for r in results {
        let key = (r.theme_name.clone(), r.variant.clone());
        let entry = theme_stats.entry(key).or_insert((0, 0));
        if r.pass {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }

    println!(
        "\n{:<20} {:<8} {:>6} {:>6} {:<10}",
        "Theme", "Variant", "Pass", "Fail", "Status"
    );
    println!("{}", "-".repeat(55));

    let mut keys: Vec<_> = theme_stats.keys().collect();
    keys.sort();

    for (theme, variant) in keys {
        let (pass, fail) = theme_stats.get(&(theme.clone(), variant.clone())).unwrap();
        let status = if *fail == 0 {
            "✅ OK".to_string()
        } else {
            format!("❌ {} issues", fail)
        };
        println!(
            "{:<20} {:<8} {:>6} {:>6} {}",
            theme, variant, pass, fail, status
        );
    }
}
