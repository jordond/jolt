use crate::config::themes_dir;

pub use jolt_theme::iterm2::{
    find_variant_suggestions, list_available_schemes, search_schemes, ImportResult, Iterm2Error,
    SchemeVariant, ITERM2_GALLERY_URL,
};

pub fn import_scheme(name: &str, custom_name: Option<&str>) -> Result<ImportResult, Iterm2Error> {
    jolt_theme::iterm2::import_scheme(name, custom_name, &themes_dir())
}
