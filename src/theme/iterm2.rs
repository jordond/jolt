use crate::config::themes_dir;
use std::fmt;
use std::io::Read;
use std::path::PathBuf;

const ITERM2_REPO_URL: &str =
    "https://raw.githubusercontent.com/mbadolato/iTerm2-Color-Schemes/master/schemes";
const ITERM2_API_URL: &str =
    "https://api.github.com/repos/mbadolato/iTerm2-Color-Schemes/contents/schemes";

pub const ITERM2_GALLERY_URL: &str = "https://iterm2colorschemes.com/";

/// Known dark/light variant pairs from the iTerm2-Color-Schemes repository.
/// Format: (dark_variant, light_variant)
/// This table enables instant lookup without network requests.
static VARIANT_PAIRS: &[(&str, &str)] = &[
    // Standard Dark/Light suffix pairs
    ("3024 Night", "3024 Day"),
    ("Aizen Dark", "Aizen Light"),
    ("Atom One Dark", "Atom One Light"),
    ("Belafonte Night", "Belafonte Day"),
    ("Bluloco Dark", "Bluloco Light"),
    ("Builtin Dark", "Builtin Light"),
    ("Builtin Tango Dark", "Builtin Tango Light"),
    ("Farmhouse Dark", "Farmhouse Light"),
    ("Flexoki Dark", "Flexoki Light"),
    ("GitHub Dark", "GitHub"),
    ("GitHub Dark Colorblind", "GitHub Light Colorblind"),
    ("GitHub Dark Default", "GitHub Light Default"),
    ("GitHub Dark High Contrast", "GitHub Light High Contrast"),
    ("GitLab Dark", "GitLab Light"),
    ("Gruvbox Dark", "Gruvbox Light"),
    ("Gruvbox Dark Hard", "Gruvbox Light Hard"),
    ("Gruvbox Material Dark", "Gruvbox Material Light"),
    ("Iceberg Dark", "Iceberg Light"),
    ("Melange Dark", "Melange Light"),
    ("Neobones Dark", "Neobones Light"),
    ("Nvim Dark", "Nvim Light"),
    ("One Double Dark", "One Double Light"),
    ("One Half Dark", "One Half Light"),
    ("Pencil Dark", "Pencil Light"),
    ("Raycast Dark", "Raycast Light"),
    ("Selenized Dark", "Selenized Light"),
    ("Seoulbones Dark", "Seoulbones Light"),
    ("Tinacious Design Dark", "Tinacious Design Light"),
    ("Violet Dark", "Violet Light"),
    ("Xcode Dark", "Xcode Light"),
    ("Xcode Dark hc", "Xcode Light hc"),
    ("Zenbones Dark", "Zenbones Light"),
    ("Zenwritten Dark", "Zenwritten Light"),
    ("iTerm2 Dark Background", "iTerm2 Light Background"),
    ("iTerm2 Solarized Dark", "iTerm2 Solarized Light"),
    ("iTerm2 Tango Dark", "iTerm2 Tango Light"),
    // Base name = dark variant
    ("Adwaita Dark", "Adwaita"),
    ("Night Owl", "Light Owl"),
    ("Nord", "Nord Light"),
    ("Onenord", "Onenord Light"),
    ("Pro", "Pro Light"),
    ("Terminal Basic Dark", "Terminal Basic"),
    ("No Clown Fiesta", "No Clown Fiesta Light"),
    // Special naming patterns
    ("Rose Pine Moon", "Rose Pine Dawn"),
    ("Rose Pine", "Rose Pine Dawn"),
    ("TokyoNight Night", "TokyoNight Day"),
    ("TokyoNight Moon", "TokyoNight Day"),
    ("TokyoNight Storm", "TokyoNight Day"),
    ("TokyoNight", "TokyoNight Day"),
    ("Ayu", "Ayu Light"),
    ("Ayu Mirage", "Ayu Light"),
    ("Everforest Dark Hard", "Everforest Light Med"),
    ("Tomorrow Night", "Tomorrow"),
    ("Tomorrow Night Blue", "Tomorrow"),
    ("Tomorrow Night Bright", "Tomorrow"),
    ("Tomorrow Night Burns", "Tomorrow"),
    ("Tomorrow Night Eighties", "Tomorrow"),
    // Catppuccin family (Latte = light, others = dark)
    ("Catppuccin Frappe", "Catppuccin Latte"),
    ("Catppuccin Macchiato", "Catppuccin Latte"),
    ("Catppuccin Mocha", "Catppuccin Latte"),
];

/// Look up a known variant pair by theme name.
/// Returns (dark_name, light_name) if found.
pub fn lookup_variant_pair(name: &str) -> Option<(&'static str, &'static str)> {
    let lower = name.to_lowercase();
    for &(dark, light) in VARIANT_PAIRS {
        if dark.to_lowercase() == lower || light.to_lowercase() == lower {
            return Some((dark, light));
        }
    }
    None
}

#[derive(Debug)]
pub enum Iterm2Error {
    NetworkError(String),
    ParseError(String),
    NotFound(String),
    IoError(String),
}

impl fmt::Display for Iterm2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for Iterm2Error {}

#[derive(Debug, Clone)]
pub struct Iterm2Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

/// WCAG AA requires 4.5:1 for normal text. We target slightly higher to avoid edge cases.
const MIN_CONTRAST_RATIO: f64 = 4.6;

impl Iterm2Color {
    pub fn to_hex(&self) -> String {
        let r = (self.r * 255.0).round() as u8;
        let g = (self.g * 255.0).round() as u8;
        let b = (self.b * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    fn blend(&self, other: &Iterm2Color, ratio: f64) -> Iterm2Color {
        Iterm2Color {
            r: self.r * (1.0 - ratio) + other.r * ratio,
            g: self.g * (1.0 - ratio) + other.g * ratio,
            b: self.b * (1.0 - ratio) + other.b * ratio,
        }
    }

    /// Convert sRGB channel to linear for luminance calculation
    fn linearize(val: f64) -> f64 {
        if val <= 0.03928 {
            val / 12.92
        } else {
            ((val + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Calculate relative luminance (0.0 = black, 1.0 = white)
    fn luminance(&self) -> f64 {
        0.2126 * Self::linearize(self.r)
            + 0.7152 * Self::linearize(self.g)
            + 0.0722 * Self::linearize(self.b)
    }

    /// Calculate contrast ratio between two colors (1:1 to 21:1)
    fn contrast_ratio(&self, other: &Iterm2Color) -> f64 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    fn lighten(&self, amount: f64) -> Iterm2Color {
        Iterm2Color {
            r: self.r + (1.0 - self.r) * amount,
            g: self.g + (1.0 - self.g) * amount,
            b: self.b + (1.0 - self.b) * amount,
        }
    }

    fn darken(&self, amount: f64) -> Iterm2Color {
        Iterm2Color {
            r: self.r * (1.0 - amount),
            g: self.g * (1.0 - amount),
            b: self.b * (1.0 - amount),
        }
    }

    /// Binary search to find minimal color adjustment that meets target contrast ratio.
    fn ensure_contrast(&self, bg: &Iterm2Color, min_ratio: f64) -> Iterm2Color {
        if self.contrast_ratio(bg) >= min_ratio {
            return self.clone();
        }

        let lighten_result = self.adjust_for_contrast(bg, min_ratio, true);
        let darken_result = self.adjust_for_contrast(bg, min_ratio, false);

        let lighten_passes = lighten_result.contrast_ratio(bg) >= min_ratio;
        let darken_passes = darken_result.contrast_ratio(bg) >= min_ratio;

        match (lighten_passes, darken_passes) {
            (true, false) => lighten_result,
            (false, true) => darken_result,
            (true, true) => {
                let lighten_dist = self.color_distance(&lighten_result);
                let darken_dist = self.color_distance(&darken_result);
                if lighten_dist <= darken_dist {
                    lighten_result
                } else {
                    darken_result
                }
            }
            (false, false) => {
                if lighten_result.contrast_ratio(bg) > darken_result.contrast_ratio(bg) {
                    lighten_result
                } else {
                    darken_result
                }
            }
        }
    }

    fn adjust_for_contrast(&self, bg: &Iterm2Color, min_ratio: f64, lighten: bool) -> Iterm2Color {
        let mut low = 0.0;
        let mut high = 1.0;
        let mut best = self.clone();

        for _ in 0..20 {
            let mid = (low + high) / 2.0;
            let adjusted = if lighten {
                self.lighten(mid)
            } else {
                self.darken(mid)
            };

            if adjusted.contrast_ratio(bg) >= min_ratio {
                best = adjusted;
                high = mid;
            } else {
                low = mid;
            }
        }

        best
    }

    fn color_distance(&self, other: &Iterm2Color) -> f64 {
        let dr = self.r - other.r;
        let dg = self.g - other.g;
        let db = self.b - other.b;
        (dr * dr + dg * dg + db * db).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SchemeVariant {
    Dark,
    Light,
    Unknown,
}

#[derive(Debug)]
pub struct Iterm2Scheme {
    pub background: Iterm2Color,
    pub foreground: Iterm2Color,
    pub selection_bg: Iterm2Color,
    pub selection_fg: Iterm2Color,
    pub ansi: [Iterm2Color; 16],
}

fn detect_variant(name: &str) -> SchemeVariant {
    if let Some((dark, light)) = lookup_variant_pair(name) {
        let lower = name.to_lowercase();
        if dark.to_lowercase() == lower {
            return SchemeVariant::Dark;
        } else if light.to_lowercase() == lower {
            return SchemeVariant::Light;
        }
    }

    let lower = name.to_lowercase();
    if lower.contains("light") || lower.contains("day") || lower.contains("dawn") {
        SchemeVariant::Light
    } else if lower.contains("dark") || lower.contains("night") || lower.contains("moon") {
        SchemeVariant::Dark
    } else {
        SchemeVariant::Unknown
    }
}

fn find_counterpart_name(name: &str) -> Option<String> {
    if let Some((dark, light)) = lookup_variant_pair(name) {
        let lower = name.to_lowercase();
        if dark.to_lowercase() == lower {
            return Some(light.to_string());
        } else {
            return Some(dark.to_string());
        }
    }

    let variant = detect_variant(name);
    match variant {
        SchemeVariant::Dark => {
            let lower = name.to_lowercase();
            if let Some(pos) = lower.find("dark") {
                let mut result = name.to_string();
                let replacement = if &name[pos..pos + 4] == "Dark" {
                    "Light"
                } else {
                    "light"
                };
                result.replace_range(pos..pos + 4, replacement);
                Some(result)
            } else {
                None
            }
        }
        SchemeVariant::Light => {
            let lower = name.to_lowercase();
            if let Some(pos) = lower.find("light") {
                let mut result = name.to_string();
                let replacement = if &name[pos..pos + 5] == "Light" {
                    "Dark"
                } else {
                    "dark"
                };
                result.replace_range(pos..pos + 5, replacement);
                Some(result)
            } else {
                None
            }
        }
        SchemeVariant::Unknown => None,
    }
}

fn find_variant_names(name: &str) -> Vec<String> {
    let variant = detect_variant(name);

    if variant == SchemeVariant::Unknown {
        vec![
            format!("{} Light", name),
            format!("{} Dark", name),
            format!("{}-light", name),
            format!("{}-dark", name),
        ]
    } else {
        vec![]
    }
}

fn parse_color_dict(dict: &plist::Dictionary) -> Option<Iterm2Color> {
    let r = dict.get("Red Component")?.as_real()?;
    let g = dict.get("Green Component")?.as_real()?;
    let b = dict.get("Blue Component")?.as_real()?;
    Some(Iterm2Color { r, g, b })
}

fn extract_color(root: &plist::Dictionary, key: &str) -> Option<Iterm2Color> {
    let dict = root.get(key)?.as_dictionary()?;
    parse_color_dict(dict)
}

pub fn parse_scheme(plist_content: &[u8]) -> Result<Iterm2Scheme, Iterm2Error> {
    let value: plist::Value =
        plist::from_bytes(plist_content).map_err(|e| Iterm2Error::ParseError(e.to_string()))?;

    let root = value
        .as_dictionary()
        .ok_or_else(|| Iterm2Error::ParseError("Expected dictionary at root".to_string()))?;

    let background = extract_color(root, "Background Color")
        .ok_or_else(|| Iterm2Error::ParseError("Missing Background Color".to_string()))?;

    let foreground = extract_color(root, "Foreground Color")
        .ok_or_else(|| Iterm2Error::ParseError("Missing Foreground Color".to_string()))?;

    let selection_bg = extract_color(root, "Selection Color")
        .unwrap_or_else(|| background.blend(&foreground, 0.3));

    let selection_fg =
        extract_color(root, "Selected Text Color").unwrap_or_else(|| foreground.clone());

    let mut ansi = Vec::with_capacity(16);
    for i in 0..16 {
        let key = format!("Ansi {} Color", i);
        let color = extract_color(root, &key).unwrap_or(if i < 8 {
            Iterm2Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
            }
        } else {
            Iterm2Color {
                r: 0.7,
                g: 0.7,
                b: 0.7,
            }
        });
        ansi.push(color);
    }

    Ok(Iterm2Scheme {
        background,
        foreground,
        selection_bg,
        selection_fg,
        ansi: ansi.try_into().unwrap(),
    })
}

impl Iterm2Scheme {
    fn to_colors_toml(&self) -> String {
        let bg = &self.background;
        let fg = self.foreground.ensure_contrast(bg, MIN_CONTRAST_RATIO);
        let dialog_bg = bg.blend(&self.ansi[8], 0.15);
        let border_color = bg.blend(&self.ansi[8], 0.4);

        let accent = self.ansi[4]
            .ensure_contrast(bg, MIN_CONTRAST_RATIO)
            .ensure_contrast(&dialog_bg, MIN_CONTRAST_RATIO);
        let accent_secondary = self.ansi[5]
            .ensure_contrast(bg, MIN_CONTRAST_RATIO)
            .ensure_contrast(&dialog_bg, MIN_CONTRAST_RATIO);
        let highlight = self.ansi[3].ensure_contrast(bg, MIN_CONTRAST_RATIO);
        let success = self.ansi[2].ensure_contrast(bg, MIN_CONTRAST_RATIO);
        let danger = self.ansi[1].ensure_contrast(bg, MIN_CONTRAST_RATIO);

        let muted = self.derive_muted_color(bg);
        let warning = self.derive_warning_color(bg);

        let graph_line = accent.clone();

        let selection_fg = self
            .selection_fg
            .ensure_contrast(&self.selection_bg, MIN_CONTRAST_RATIO);

        format!(
            r##"bg = "{}"
dialog_bg = "{}"
fg = "{}"
accent = "{}"
accent_secondary = "{}"
highlight = "{}"
muted = "{}"
success = "{}"
warning = "{}"
danger = "{}"
border = "{}"
selection_bg = "{}"
selection_fg = "{}"
graph_line = "{}""##,
            bg.to_hex(),
            dialog_bg.to_hex(),
            fg.to_hex(),
            accent.to_hex(),
            accent_secondary.to_hex(),
            highlight.to_hex(),
            muted.to_hex(),
            success.to_hex(),
            warning.to_hex(),
            danger.to_hex(),
            border_color.to_hex(),
            self.selection_bg.to_hex(),
            selection_fg.to_hex(),
            graph_line.to_hex(),
        )
    }

    fn derive_muted_color(&self, bg: &Iterm2Color) -> Iterm2Color {
        let candidates = [&self.ansi[8], &self.foreground.blend(bg, 0.5)];

        for candidate in candidates {
            let adjusted = candidate.ensure_contrast(bg, MIN_CONTRAST_RATIO);
            if adjusted.contrast_ratio(bg) >= MIN_CONTRAST_RATIO {
                return adjusted;
            }
        }

        self.foreground.blend(bg, 0.4)
    }

    fn derive_warning_color(&self, bg: &Iterm2Color) -> Iterm2Color {
        let candidates = [&self.ansi[11], &self.ansi[3], &self.ansi[9]];

        for candidate in candidates {
            let adjusted = candidate.ensure_contrast(bg, MIN_CONTRAST_RATIO);
            if adjusted.contrast_ratio(bg) >= MIN_CONTRAST_RATIO {
                return adjusted;
            }
        }

        self.ansi[3].ensure_contrast(bg, MIN_CONTRAST_RATIO)
    }
}

pub struct ImportResult {
    pub path: PathBuf,
    pub dark_source: Option<String>,
    pub light_source: Option<String>,
}

fn try_fetch_scheme(name: &str) -> Option<Iterm2Scheme> {
    let url = format!("{}/{}.itermcolors", ITERM2_REPO_URL, name);

    let response = ureq::get(&url).call().ok()?;

    let mut bytes = Vec::new();
    response.into_reader().read_to_end(&mut bytes).ok()?;

    parse_scheme(&bytes).ok()
}

pub fn fetch_scheme(name: &str) -> Result<Iterm2Scheme, Iterm2Error> {
    let url = format!("{}/{}.itermcolors", ITERM2_REPO_URL, name);

    let response = ureq::get(&url).call().map_err(|e| match e {
        ureq::Error::Status(404, _) => Iterm2Error::NotFound(format!(
            "Scheme '{}' not found. Browse available themes at: {}",
            name, ITERM2_GALLERY_URL
        )),
        _ => Iterm2Error::NetworkError(e.to_string()),
    })?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| Iterm2Error::NetworkError(e.to_string()))?;

    parse_scheme(&bytes)
}

#[derive(Debug, serde::Deserialize)]
struct GitHubFile {
    name: String,
    #[serde(rename = "type")]
    file_type: String,
}

pub fn list_available_schemes() -> Result<Vec<String>, Iterm2Error> {
    let response = ureq::get(ITERM2_API_URL)
        .set("User-Agent", "jolt-theme-importer")
        .call()
        .map_err(|e| Iterm2Error::NetworkError(e.to_string()))?;

    let body = response
        .into_string()
        .map_err(|e| Iterm2Error::NetworkError(e.to_string()))?;

    let files: Vec<GitHubFile> =
        serde_json::from_str(&body).map_err(|e| Iterm2Error::ParseError(e.to_string()))?;

    let schemes: Vec<String> = files
        .into_iter()
        .filter(|f| f.file_type == "file" && f.name.ends_with(".itermcolors"))
        .map(|f| f.name.trim_end_matches(".itermcolors").to_string())
        .collect();

    Ok(schemes)
}

fn derive_base_name(name: &str) -> String {
    let lower = name.to_lowercase();

    for suffix in [" light", " dark", "-light", "-dark"] {
        if lower.ends_with(suffix) {
            return name[..name.len() - suffix.len()].to_string();
        }
    }

    name.to_string()
}

pub fn import_scheme(name: &str, custom_name: Option<&str>) -> Result<ImportResult, Iterm2Error> {
    let primary = fetch_scheme(name)?;
    let primary_variant = detect_variant(name);

    let mut dark_scheme: Option<Iterm2Scheme> = None;
    let mut light_scheme: Option<Iterm2Scheme> = None;
    let mut dark_source: Option<String> = None;
    let mut light_source: Option<String> = None;

    match primary_variant {
        SchemeVariant::Dark => {
            dark_scheme = Some(primary);
            dark_source = Some(name.to_string());

            if let Some(counterpart) = find_counterpart_name(name) {
                if let Some(light) = try_fetch_scheme(&counterpart) {
                    light_scheme = Some(light);
                    light_source = Some(counterpart);
                }
            }
        }
        SchemeVariant::Light => {
            light_scheme = Some(primary);
            light_source = Some(name.to_string());

            if let Some(counterpart) = find_counterpart_name(name) {
                if let Some(dark) = try_fetch_scheme(&counterpart) {
                    dark_scheme = Some(dark);
                    dark_source = Some(counterpart);
                }
            }
        }
        SchemeVariant::Unknown => {
            dark_scheme = Some(primary);
            dark_source = Some(name.to_string());

            for variant_name in find_variant_names(name) {
                if let Some(scheme) = try_fetch_scheme(&variant_name) {
                    let variant = detect_variant(&variant_name);
                    if variant == SchemeVariant::Light && light_scheme.is_none() {
                        light_scheme = Some(scheme);
                        light_source = Some(variant_name);
                        break;
                    }
                }
            }
        }
    }

    let base_name = custom_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| derive_base_name(name));

    let file_name = base_name.to_lowercase().replace(' ', "-");

    let mut toml_content = format!("name = \"{}\"\n", base_name);

    if let Some(ref dark) = dark_scheme {
        toml_content.push_str("\n[dark]\n");
        toml_content.push_str(&dark.to_colors_toml());
        toml_content.push('\n');
    }

    if let Some(ref light) = light_scheme {
        toml_content.push_str("\n[light]\n");
        toml_content.push_str(&light.to_colors_toml());
        toml_content.push('\n');
    }

    let themes_path = themes_dir();
    std::fs::create_dir_all(&themes_path).map_err(|e| Iterm2Error::IoError(e.to_string()))?;

    let theme_path = themes_path.join(format!("{}.toml", file_name));

    std::fs::write(&theme_path, toml_content).map_err(|e| Iterm2Error::IoError(e.to_string()))?;

    Ok(ImportResult {
        path: theme_path,
        dark_source,
        light_source,
    })
}

pub fn search_schemes(query: &str) -> Result<Vec<String>, Iterm2Error> {
    let all_schemes = list_available_schemes()?;
    let query_lower = query.to_lowercase();

    let matches: Vec<String> = all_schemes
        .into_iter()
        .filter(|s| s.to_lowercase().contains(&query_lower))
        .collect();

    Ok(matches)
}

pub fn find_variant_suggestions(
    name: &str,
    target_variant: SchemeVariant,
) -> Result<Vec<String>, Iterm2Error> {
    let base_name = derive_base_name(name);
    let all_schemes = list_available_schemes()?;
    let base_lower = base_name.to_lowercase();

    let mut suggestions: Vec<String> = all_schemes
        .into_iter()
        .filter(|s| {
            let s_lower = s.to_lowercase();
            if s_lower == name.to_lowercase() {
                return false;
            }

            let matches_base = s_lower.contains(&base_lower)
                || base_lower.contains(&derive_base_name(s).to_lowercase());

            if !matches_base {
                return false;
            }

            let variant = detect_variant(s);
            variant == target_variant || variant == SchemeVariant::Unknown
        })
        .collect();

    suggestions.sort();
    Ok(suggestions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_hex() {
        let color = Iterm2Color {
            r: 1.0,
            g: 0.5,
            b: 0.0,
        };
        assert_eq!(color.to_hex(), "#ff8000");
    }

    #[test]
    fn test_color_blend() {
        let black = Iterm2Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let white = Iterm2Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        };
        let gray = black.blend(&white, 0.5);
        assert!((gray.r - 0.5).abs() < 0.01);
        assert!((gray.g - 0.5).abs() < 0.01);
        assert!((gray.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_detect_variant() {
        assert_eq!(detect_variant("Gruvbox Dark"), SchemeVariant::Dark);
        assert_eq!(detect_variant("Gruvbox Light"), SchemeVariant::Light);
        assert_eq!(detect_variant("Dracula"), SchemeVariant::Unknown);
        assert_eq!(detect_variant("One Dark"), SchemeVariant::Dark);
        assert_eq!(detect_variant("Solarized Light"), SchemeVariant::Light);
    }

    #[test]
    fn test_find_counterpart_name() {
        assert_eq!(
            find_counterpart_name("Gruvbox Dark"),
            Some("Gruvbox Light".to_string())
        );
        assert_eq!(
            find_counterpart_name("Gruvbox Light"),
            Some("Gruvbox Dark".to_string())
        );
        assert_eq!(
            find_counterpart_name("Gruvbox Dark Hard"),
            Some("Gruvbox Light Hard".to_string())
        );
        assert_eq!(find_counterpart_name("Dracula"), None);
    }

    #[test]
    fn test_derive_base_name() {
        assert_eq!(derive_base_name("Gruvbox Dark"), "Gruvbox");
        assert_eq!(derive_base_name("Gruvbox Light"), "Gruvbox");
        assert_eq!(derive_base_name("Gruvbox Dark Hard"), "Gruvbox Dark Hard");
        assert_eq!(derive_base_name("One Dark"), "One");
        assert_eq!(derive_base_name("Dracula"), "Dracula");
    }

    #[test]
    fn test_lookup_variant_pair() {
        let mocha = lookup_variant_pair("Catppuccin Mocha");
        assert!(mocha.is_some());
        assert_eq!(mocha.unwrap().1, "Catppuccin Latte");

        let latte = lookup_variant_pair("Catppuccin Latte");
        assert!(latte.is_some());
        assert_eq!(latte.unwrap().1, "Catppuccin Latte");

        assert_eq!(
            lookup_variant_pair("Tomorrow Night"),
            Some(("Tomorrow Night", "Tomorrow"))
        );
        assert_eq!(lookup_variant_pair("Nord"), Some(("Nord", "Nord Light")));
        assert_eq!(lookup_variant_pair("Dracula"), None);
    }

    #[test]
    fn test_find_counterpart_via_lookup() {
        assert_eq!(
            find_counterpart_name("Catppuccin Mocha"),
            Some("Catppuccin Latte".to_string())
        );
        let latte_counterpart = find_counterpart_name("Catppuccin Latte");
        assert!(latte_counterpart.is_some());
        assert!(latte_counterpart.unwrap().starts_with("Catppuccin"));

        assert_eq!(
            find_counterpart_name("Tomorrow Night"),
            Some("Tomorrow".to_string())
        );
        assert_eq!(
            find_counterpart_name("Nord"),
            Some("Nord Light".to_string())
        );
        assert_eq!(
            find_counterpart_name("Nord Light"),
            Some("Nord".to_string())
        );
    }

    #[test]
    fn test_detect_variant_via_lookup() {
        assert_eq!(detect_variant("Catppuccin Mocha"), SchemeVariant::Dark);
        assert_eq!(detect_variant("Catppuccin Latte"), SchemeVariant::Light);
        assert_eq!(detect_variant("Nord"), SchemeVariant::Dark);
        assert_eq!(detect_variant("Tomorrow"), SchemeVariant::Light);
    }
}
