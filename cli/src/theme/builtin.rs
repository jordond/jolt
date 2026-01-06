use super::loader::parse_theme_toml;
use super::NamedTheme;

const THEMES: &[(&str, &str)] = &[
    ("default", include_str!("themes/default.toml")),
    ("catppuccin", include_str!("themes/catppuccin.toml")),
    ("dracula", include_str!("themes/dracula.toml")),
    ("nord", include_str!("themes/nord.toml")),
    ("gruvbox", include_str!("themes/gruvbox.toml")),
    ("tokyo-night", include_str!("themes/tokyo-night.toml")),
    ("solarized", include_str!("themes/solarized.toml")),
    ("rose-pine", include_str!("themes/rose-pine.toml")),
    ("one-dark", include_str!("themes/one-dark.toml")),
    ("monokai", include_str!("themes/monokai.toml")),
];

pub fn get_builtin_themes() -> Vec<NamedTheme> {
    THEMES
        .iter()
        .filter_map(|(id, content)| parse_theme_toml(id, content, true))
        .collect()
}
