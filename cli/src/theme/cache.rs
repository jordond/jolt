use crate::config::cache_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::iterm2::{list_available_schemes, lookup_variant_pair, Iterm2Error, SchemeVariant};

const CACHE_TTL_DAYS: u64 = 5;
const CACHE_FILENAME: &str = "iterm2_schemes.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeGroup {
    pub name: String,
    pub dark: Option<String>,
    pub light: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSchemeList {
    pub timestamp: u64,
    pub schemes: Vec<String>,
    pub groups: Vec<ThemeGroup>,
}

impl CachedSchemeList {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let age_days = (now - self.timestamp) / 86400;
        age_days >= CACHE_TTL_DAYS
    }

    pub fn age_description(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let age_secs = now.saturating_sub(self.timestamp);

        if age_secs < 60 {
            "just now".to_string()
        } else if age_secs < 3600 {
            format!("{} minutes ago", age_secs / 60)
        } else if age_secs < 86400 {
            format!("{} hours ago", age_secs / 3600)
        } else {
            format!("{} days ago", age_secs / 86400)
        }
    }
}

fn cache_path() -> PathBuf {
    cache_dir().join(CACHE_FILENAME)
}

pub fn load_cached_schemes() -> Option<CachedSchemeList> {
    let path = cache_path();
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_cached_schemes(cache: &CachedSchemeList) -> std::io::Result<()> {
    let path = cache_path();
    fs::create_dir_all(cache_dir())?;
    let content = serde_json::to_string_pretty(cache)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, content)
}

pub fn group_schemes(schemes: &[String]) -> Vec<ThemeGroup> {
    let scheme_set: std::collections::HashSet<&str> = schemes.iter().map(|s| s.as_str()).collect();
    let mut grouped: HashMap<String, ThemeGroup> = HashMap::new();
    let mut processed: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for scheme in schemes {
        if processed.contains(scheme.as_str()) {
            continue;
        }

        if let Some((dark, light)) = lookup_variant_pair(scheme) {
            let group_name = derive_group_name(dark, light);
            let has_dark = scheme_set.contains(dark);
            let has_light = scheme_set.contains(light);

            grouped.insert(
                group_name.clone(),
                ThemeGroup {
                    name: group_name,
                    dark: if has_dark {
                        Some(dark.to_string())
                    } else {
                        None
                    },
                    light: if has_light {
                        Some(light.to_string())
                    } else {
                        None
                    },
                },
            );

            processed.insert(dark);
            processed.insert(light);
        }
    }

    for scheme in schemes {
        if processed.contains(scheme.as_str()) {
            continue;
        }

        let group_name = scheme.clone();
        let variant = detect_variant_from_name(scheme);

        grouped.insert(
            group_name.clone(),
            ThemeGroup {
                name: group_name,
                dark: if variant != SchemeVariant::Light {
                    Some(scheme.clone())
                } else {
                    None
                },
                light: if variant == SchemeVariant::Light {
                    Some(scheme.clone())
                } else {
                    None
                },
            },
        );
        processed.insert(scheme);
    }

    let mut groups: Vec<ThemeGroup> = grouped.into_values().collect();
    groups.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    groups
}

fn derive_group_name(dark: &str, light: &str) -> String {
    let dark_lower = dark.to_lowercase();
    let light_lower = light.to_lowercase();

    for suffix in [" dark", " light", " night", " day", " moon", " dawn"] {
        if dark_lower.ends_with(suffix) {
            return dark[..dark.len() - suffix.len()].to_string();
        }
        if light_lower.ends_with(suffix) {
            return light[..light.len() - suffix.len()].to_string();
        }
    }

    if dark.len() <= light.len() {
        dark.to_string()
    } else {
        light.to_string()
    }
}

fn detect_variant_from_name(name: &str) -> SchemeVariant {
    let lower = name.to_lowercase();
    if lower.contains("light") || lower.contains("day") || lower.contains("dawn") {
        SchemeVariant::Light
    } else if lower.contains("dark") || lower.contains("night") || lower.contains("moon") {
        SchemeVariant::Dark
    } else {
        SchemeVariant::Unknown
    }
}

pub fn fetch_and_cache_schemes(force: bool) -> Result<CachedSchemeList, Iterm2Error> {
    if !force {
        if let Some(cached) = load_cached_schemes() {
            if !cached.is_expired() {
                return Ok(cached);
            }
        }
    }

    let schemes = list_available_schemes()?;
    let groups = group_schemes(&schemes);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let cache = CachedSchemeList {
        timestamp,
        schemes,
        groups,
    };

    let _ = save_cached_schemes(&cache);

    Ok(cache)
}

pub fn get_cached_or_empty() -> CachedSchemeList {
    load_cached_schemes().unwrap_or_else(|| CachedSchemeList {
        timestamp: 0,
        schemes: Vec::new(),
        groups: Vec::new(),
    })
}
