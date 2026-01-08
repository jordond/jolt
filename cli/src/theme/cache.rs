use crate::config::cache_dir;

pub use jolt_theme::cache::{CachedSchemeList, ThemeGroup};

pub fn fetch_and_cache_schemes(
    force: bool,
) -> Result<CachedSchemeList, jolt_theme::iterm2::Iterm2Error> {
    jolt_theme::cache::fetch_and_cache_schemes(&cache_dir(), force)
}

pub fn get_cached_or_empty() -> CachedSchemeList {
    jolt_theme::cache::get_cached_or_empty(&cache_dir())
}
