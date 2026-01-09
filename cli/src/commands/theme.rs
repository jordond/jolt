use std::io::Write;

use color_eyre::eyre::Result;

use crate::cli::ThemeCommands;
use crate::config;

pub fn run(command: Option<ThemeCommands>) -> Result<()> {
    let themes_dir = config::themes_dir();
    let cache_dir = config::cache_dir();

    let cmd = command.unwrap_or(ThemeCommands::Check {
        all: false,
        verbose: false,
    });

    match cmd {
        ThemeCommands::Check { all, verbose } => {
            let mut has_validation_errors = false;

            let validation_results = jolt_theme::validation::validate_theme_files(&themes_dir);
            if !validation_results.is_empty() {
                jolt_theme::validation::print_validation_results(&validation_results, verbose);
                has_validation_errors = validation_results.iter().any(|r| !r.is_valid());
                println!();
            }

            let themes = if all {
                jolt_theme::get_all_themes(Some(&themes_dir))
            } else {
                jolt_theme::load_themes_from_dir(&themes_dir, false)
            };

            let valid_theme_ids: std::collections::HashSet<_> = validation_results
                .iter()
                .filter(|r| r.is_valid())
                .map(|r| r.theme_id.clone())
                .collect();

            let themes_for_contrast: Vec<_> = if all {
                themes
            } else {
                themes
                    .into_iter()
                    .filter(|t| valid_theme_ids.contains(&t.id))
                    .collect()
            };

            if themes_for_contrast.is_empty() && !all {
                if validation_results.is_empty() {
                    println!("No user themes found.");
                    println!("Use --all to check builtin themes, or create a theme with:");
                    println!("  jolt theme create <name>");
                } else if has_validation_errors {
                    println!(
                        "All user themes have validation errors. Fix errors above to run contrast checks."
                    );
                }
                if has_validation_errors {
                    std::process::exit(1);
                }
                return Ok(());
            }

            if !themes_for_contrast.is_empty() {
                let results = jolt_theme::contrast::check_all_themes(&themes_for_contrast);
                jolt_theme::contrast::print_results(&results, verbose);

                let has_contrast_failures = results.iter().any(|r| !r.pass);
                if has_contrast_failures || has_validation_errors {
                    std::process::exit(1);
                }
            }
        }
        ThemeCommands::Open => {
            if !themes_dir.exists() {
                std::fs::create_dir_all(&themes_dir)?;
            }

            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(&themes_dir)
                    .status()?;
            }

            println!("Themes directory: {}", themes_dir.display());
        }
        ThemeCommands::Create {
            name,
            template,
            base,
        } => {
            if !themes_dir.exists() {
                std::fs::create_dir_all(&themes_dir)?;
            }

            let file_name = name.to_lowercase().replace(' ', "-");
            let theme_path = themes_dir.join(format!("{}.toml", file_name));

            if theme_path.exists() {
                eprintln!(
                    "Theme '{}' already exists at: {}",
                    name,
                    theme_path.display()
                );
                std::process::exit(1);
            }

            let content = match template.as_str() {
                "copy" => {
                    let base_id = base.as_deref().unwrap_or("default");
                    let base_theme = jolt_theme::get_builtin_themes()
                        .into_iter()
                        .find(|t| t.id == base_id)
                        .ok_or_else(|| {
                            color_eyre::eyre::eyre!("Base theme '{}' not found", base_id)
                        })?;

                    jolt_theme::generate_theme_toml(&name, &base_theme)
                }
                _ => jolt_theme::generate_blank_theme_toml(&name),
            };

            std::fs::write(&theme_path, content)?;
            println!("Created theme '{}' at: {}", name, theme_path.display());
            println!("\nEdit the file to customize colors, then reload jolt to see changes.");
        }
        ThemeCommands::List {
            builtin,
            user,
            iterm2,
            search,
        } => {
            if iterm2 || search.is_some() {
                print!("Fetching iTerm2 schemes from GitHub...");
                std::io::stdout().flush()?;

                let schemes = if let Some(ref query) = search {
                    jolt_theme::iterm2::search_schemes(query)
                } else {
                    jolt_theme::iterm2::list_available_schemes()
                };

                match schemes {
                    Ok(list) => {
                        println!("\r{}", " ".repeat(50));
                        if list.is_empty() {
                            if let Some(query) = search {
                                println!("No iTerm2 schemes found matching '{}'", query);
                            } else {
                                println!("No iTerm2 schemes found.");
                            }
                        } else {
                            let title = if let Some(ref query) = search {
                                format!(
                                    "iTerm2 schemes matching '{}' ({} found)",
                                    query,
                                    list.len()
                                )
                            } else {
                                format!("Available iTerm2 schemes ({} total)", list.len())
                            };
                            println!("{}", title);
                            println!("{}", "-".repeat(60));
                            for (i, scheme) in list.iter().enumerate() {
                                print!("{:<30}", scheme);
                                if (i + 1) % 2 == 0 {
                                    println!();
                                }
                            }
                            if list.len() % 2 != 0 {
                                println!();
                            }
                            println!();
                            println!(
                                "Browse themes visually: {}",
                                jolt_theme::iterm2::ITERM2_GALLERY_URL
                            );
                            println!("Import with: jolt theme import <scheme-name>");
                        }
                    }
                    Err(e) => {
                        println!("\rError: {}", e);
                        std::process::exit(1);
                    }
                }
                return Ok(());
            }

            let themes = if builtin {
                jolt_theme::get_builtin_themes()
            } else if user {
                jolt_theme::load_themes_from_dir(&themes_dir, false)
            } else {
                jolt_theme::get_all_themes(Some(&themes_dir))
            };

            if themes.is_empty() {
                println!("No themes found.");
                return Ok(());
            }

            println!("{:<20} {:<12} Variants", "ID", "Type");
            println!("{}", "-".repeat(50));
            for theme in themes {
                let theme_type = if theme.is_builtin { "builtin" } else { "user" };
                println!(
                    "{:<20} {:<12} {}",
                    theme.id,
                    theme_type,
                    theme.variants_label()
                );
            }
        }
        ThemeCommands::Import { scheme, name } => {
            println!("Fetching iTerm2 scheme '{}'...", scheme);

            match jolt_theme::iterm2::import_scheme(&scheme, name.as_deref(), &themes_dir) {
                Ok(result) => {
                    let has_dark = result.dark_source.is_some();
                    let has_light = result.light_source.is_some();

                    println!("Imported theme to: {}", result.path.display());

                    if let Some(dark) = &result.dark_source {
                        println!("  dark variant:  {}", dark);
                    }
                    if let Some(light) = &result.light_source {
                        println!("  light variant: {}", light);
                    }

                    if has_dark && has_light {
                        println!(
                            "\nBoth dark and light variants imported! The theme will adapt to your system appearance."
                        );
                    } else {
                        let missing = if has_dark {
                            jolt_theme::iterm2::SchemeVariant::Light
                        } else {
                            jolt_theme::iterm2::SchemeVariant::Dark
                        };
                        let missing_name = if has_dark { "light" } else { "dark" };

                        println!(
                            "\nOnly {} variant found. Looking for {} variant suggestions...",
                            if has_dark { "dark" } else { "light" },
                            missing_name
                        );

                        match jolt_theme::iterm2::find_variant_suggestions(&scheme, missing) {
                            Ok(suggestions) if !suggestions.is_empty() => {
                                let suggestions: Vec<String> = suggestions;
                                println!("\nPossible {} variants:", missing_name);
                                for (i, suggestion) in suggestions.iter().take(10).enumerate() {
                                    println!("  {}. {}", i + 1, suggestion);
                                }
                                if suggestions.len() > 10 {
                                    println!("  ... and {} more", suggestions.len() - 10);
                                }
                                println!(
                                    "\nTo add a variant, edit the theme file or import separately."
                                );
                            }
                            Ok(_) => {
                                println!("No {} variant suggestions found.", missing_name);
                            }
                            Err(_) => {
                                println!("Could not fetch variant suggestions.");
                            }
                        }
                    }

                    println!("\nUse the theme picker (T key) to select it.");
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    if matches!(e, jolt_theme::iterm2::Iterm2Error::NotFound(_)) {
                        eprintln!(
                            "\nBrowse themes: {}",
                            jolt_theme::iterm2::ITERM2_GALLERY_URL
                        );
                        eprintln!("Search: jolt theme list --search <query>");
                    }
                    std::process::exit(1);
                }
            }
        }
        ThemeCommands::Fetch { force } => {
            println!("Fetching iTerm2 theme list...");

            match jolt_theme::cache::fetch_and_cache_schemes(&cache_dir, force) {
                Ok(cache) => {
                    println!(
                        "Cached {} themes in {} groups.",
                        cache.schemes.len(),
                        cache.groups.len()
                    );
                    if !force {
                        println!("Cache updated: {}", cache.age_description());
                    }
                    println!(
                        "\nUse 'jolt theme list --iterm2' to browse, or press 'i' in the theme picker to import."
                    );
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        ThemeCommands::Clean { yes } => {
            if !themes_dir.exists() {
                println!("No user themes directory found.");
                return Ok(());
            }

            let theme_files: Vec<_> = std::fs::read_dir(&themes_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.path()
                                .extension()
                                .map(|ext| ext == "toml")
                                .unwrap_or(false)
                        })
                        .collect()
                })
                .unwrap_or_default();

            if theme_files.is_empty() {
                println!("No user themes to delete.");
                return Ok(());
            }

            println!("Found {} user theme(s):", theme_files.len());
            for file in &theme_files {
                println!("  - {}", file.file_name().to_string_lossy());
            }

            if !yes {
                print!("\nDelete all user themes? [y/N] ");
                std::io::stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let mut deleted = 0;
            for file in theme_files {
                if std::fs::remove_file(file.path()).is_ok() {
                    deleted += 1;
                }
            }

            println!("Deleted {} theme(s).", deleted);
        }
    }

    Ok(())
}
