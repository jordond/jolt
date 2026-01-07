mod app;
mod config;
mod daemon;
mod data;
mod input;
mod logging;
mod theme;
mod ui;

use std::io;
use std::os::unix::process::CommandExt;
use std::time::Duration;

use app::App;
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use config::{config_path, ensure_dirs, LogLevel, UserConfig};
use logging::LogMode;

#[derive(Debug, Subcommand)]
enum ThemeCommands {
    /// Check themes for WCAG contrast issues (default)
    #[command(alias = "c")]
    Check {
        /// Also check builtin themes
        #[arg(short = 'A', long)]
        all: bool,

        /// Show passing checks too
        #[arg(short, long)]
        verbose: bool,
    },

    /// Open user themes folder
    #[command(alias = "o")]
    Open,

    /// Create a new theme
    #[command(alias = "new")]
    Create {
        /// Theme name
        name: String,

        /// Create from template (blank, copy)
        #[arg(short, long, default_value = "blank")]
        template: String,

        /// Base theme to copy from (when template=copy)
        #[arg(short, long)]
        base: Option<String>,
    },

    /// List available themes
    #[command(alias = "ls")]
    List {
        /// Show only builtin themes
        #[arg(long)]
        builtin: bool,

        /// Show only user themes
        #[arg(long)]
        user: bool,

        /// List available iTerm2 color schemes (300+ themes)
        #[arg(long)]
        iterm2: bool,

        /// Search iTerm2 schemes by name
        #[arg(long)]
        search: Option<String>,
    },

    /// Import a theme from iTerm2 Color Schemes (https://iterm2colorschemes.com)
    #[command(alias = "i")]
    Import {
        /// iTerm2 scheme name (e.g., Dracula, Gruvbox Dark, Nord)
        scheme: String,

        /// Custom name for the imported theme
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Fetch and cache the list of available iTerm2 themes
    #[command(alias = "f")]
    Fetch {
        /// Force refresh even if cache is valid
        #[arg(short, long)]
        force: bool,
    },

    /// Delete all user-installed themes
    Clean {
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Launch the TUI interface (default)
    #[command(alias = "tui")]
    Ui {
        /// Update interval in milliseconds
        #[arg(short, long)]
        refresh_ms: Option<u64>,

        /// Appearance mode (auto, dark, light)
        #[arg(short, long)]
        appearance: Option<String>,

        /// Low power mode - reduced refresh rate
        #[arg(short = 'L', long)]
        low_power: bool,
    },

    /// Output metrics in JSON format (suitable for piping)
    #[command(alias = "raw")]
    Pipe {
        /// Number of samples to output (0 = infinite)
        #[arg(short, long, default_value_t = 0)]
        samples: u32,

        /// Update interval in milliseconds
        #[arg(short, long, default_value_t = 1000)]
        interval: u64,

        /// Compact JSON output (one line per sample)
        #[arg(short, long)]
        compact: bool,
    },

    /// Print debug information about power sources and sensors
    Debug,

    /// Show or edit configuration
    Config {
        /// Print config file path
        #[arg(long)]
        path: bool,

        /// Reset config to defaults
        #[arg(long)]
        reset: bool,

        /// Open config file in $EDITOR
        #[arg(short, long)]
        edit: bool,
    },

    /// Manage themes
    #[command(alias = "themes")]
    Theme {
        #[command(subcommand)]
        command: Option<ThemeCommands>,
    },

    /// Manage the background daemon
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },

    /// View and manage historical data
    History {
        #[command(subcommand)]
        command: Option<HistoryCommands>,
    },
}

#[derive(Debug, Subcommand)]
enum DaemonCommands {
    /// Start the daemon
    Start {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },

    /// Stop the running daemon
    Stop,

    /// Check daemon status
    Status,

    /// View daemon logs
    Logs {
        /// Number of lines to show
        #[arg(short, long, default_value_t = 50)]
        lines: usize,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },

    /// Install daemon to start on login (via launchd)
    Install,

    /// Uninstall daemon from launchd
    Uninstall,
}

#[derive(Debug, Subcommand)]
enum HistoryCommands {
    /// Show history summary (default)
    Summary {
        /// Time period: today, week, month, all
        #[arg(short, long, default_value = "week")]
        period: String,
    },

    /// Show top power consumers
    Top {
        /// Time period: today, week, month, all
        #[arg(short, long, default_value = "week")]
        period: String,

        /// Number of processes to show
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },

    /// Export history data
    Export {
        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Output format: json, csv
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,

        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,

        /// Time period (alternative to from/to): today, week, month, all
        #[arg(short, long)]
        period: Option<String>,

        /// Include raw samples in export (can be large)
        #[arg(long)]
        include_samples: bool,
    },

    /// Prune old data
    Prune {
        /// Delete data older than N days
        #[arg(long)]
        older_than: Option<u32>,

        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Beautiful battery & energy monitor for macOS
/// https://github.com/jordond/jolt
#[derive(Debug, Parser)]
#[command(name = "jolt", version, verbatim_doc_comment)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Update interval in milliseconds (for default TUI mode)
    #[arg(short, long, global = true)]
    refresh_ms: Option<u64>,

    /// Appearance mode (auto, dark, light)
    #[arg(short, long, global = true)]
    appearance: Option<String>,

    /// Low power mode
    #[arg(short = 'L', long, global = true)]
    low_power: bool,

    /// Log level (off, error, warn, info, debug, trace)
    #[arg(long, global = true)]
    log_level: Option<String>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let _ = ensure_dirs();

    let cli = Cli::parse();
    let config = UserConfig::load();
    let log_level_override = cli.log_level.as_deref().map(LogLevel::from_str);

    match cli.command {
        Some(Commands::Pipe {
            samples,
            interval,
            compact,
        }) => {
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            run_pipe(samples, interval, compact)
        }
        Some(Commands::Debug) => {
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            run_debug()
        }
        Some(Commands::Config { path, reset, edit }) => {
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            run_config(path, reset, edit)
        }
        Some(Commands::Theme { command }) => {
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            run_theme(command)
        }
        Some(Commands::Daemon { command }) => {
            run_daemon_command(command, config.log_level, log_level_override)
        }
        Some(Commands::History { command }) => {
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            run_history_command(command)
        }
        Some(Commands::Ui {
            refresh_ms,
            appearance,
            low_power,
        }) => {
            let _guard = logging::init(config.log_level, LogMode::File, log_level_override);
            let mut config = config;
            let refresh_from_cli =
                config.merge_with_args(appearance.as_deref(), refresh_ms, low_power);
            run_tui(config, refresh_from_cli)
        }
        None => {
            let _guard = logging::init(config.log_level, LogMode::File, log_level_override);
            let mut config = config;
            let refresh_from_cli =
                config.merge_with_args(cli.appearance.as_deref(), cli.refresh_ms, cli.low_power);
            run_tui(config, refresh_from_cli)
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_tui(user_config: UserConfig, refresh_from_cli: bool) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_tui_loop(&mut terminal, user_config, refresh_from_cli);
    restore_terminal(&mut terminal)?;
    result
}

fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    user_config: UserConfig,
    refresh_from_cli: bool,
) -> Result<()> {
    let mut app = App::new(user_config, refresh_from_cli)?;

    loop {
        let tick_rate = Duration::from_millis(app.refresh_ms);
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        let should_tick = if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let action = input::handle_key(&app, key);
                    if !app.handle_action(action) {
                        break;
                    }
                    false
                }
                Event::Resize(_, _) => false,
                _ => false,
            }
        } else {
            true
        };

        if should_tick {
            app.tick()?;
        }
    }

    Ok(())
}

fn run_pipe(samples: u32, interval: u64, compact: bool) -> Result<()> {
    use data::{BatteryData, PowerData, ProcessData};
    use serde_json::json;

    let mut battery = BatteryData::new()?;
    let mut power = PowerData::new()?;
    let mut processes = ProcessData::new()?;
    let mut counter = 0u32;

    loop {
        battery.refresh()?;
        power.refresh()?;
        processes.refresh()?;

        let top_processes: Vec<_> = processes
            .processes
            .iter()
            .take(10)
            .map(|p| {
                json!({
                    "pid": p.pid,
                    "name": p.name,
                    "cpu": p.cpu_usage,
                    "memory_mb": p.memory_mb,
                    "energy": p.energy_impact,
                })
            })
            .collect();

        let doc = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "battery": {
                "percent": battery.charge_percent(),
                "state": battery.state_label(),
                "health": battery.health_percent(),
                "capacity_wh": battery.max_capacity_wh(),
                "time_remaining_min": battery.time_remaining_minutes(),
                "cycle_count": battery.cycle_count(),
            },
            "power": {
                "cpu_watts": power.cpu_power_watts(),
                "gpu_watts": power.gpu_power_watts(),
                "total_watts": power.total_power_watts(),
                "mode": power.power_mode_label(),
            },
            "top_processes": top_processes,
        });

        if compact {
            println!("{}", serde_json::to_string(&doc)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&doc)?);
        }

        counter += 1;
        if samples > 0 && counter >= samples {
            break;
        }

        std::thread::sleep(Duration::from_millis(interval));
    }

    Ok(())
}

fn run_debug() -> Result<()> {
    use data::{BatteryData, PowerData};

    println!("jolt debug information");
    println!("{}", "=".repeat(60));

    println!("\n--- System Info ---");
    if let Ok(output) = std::process::Command::new("system_profiler")
        .args(["SPHardwareDataType", "-json"])
        .output()
    {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
            if let Some(hw) = json.get("SPHardwareDataType").and_then(|v| v.get(0)) {
                println!(
                    "Chip: {}",
                    hw.get("chip_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                );
                println!(
                    "Model: {}",
                    hw.get("machine_model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                );
                println!(
                    "Cores: {}",
                    hw.get("number_processors")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                );
            }
        }
    }

    println!("\n--- Battery Info ---");
    let battery = BatteryData::new()?;
    println!("Charge: {:.1}%", battery.charge_percent());
    println!("State: {}", battery.state_label());
    if let Some(watts) = battery.charging_watts() {
        println!("Charging at: {:.1}W", watts);
    }
    if let Some(charger) = battery.charger_watts() {
        println!("Charger: {}W", charger);
    }
    println!("Health: {:.1}%", battery.health_percent());
    println!("Capacity: {:.1}Wh", battery.max_capacity_wh());
    if let Some(cycles) = battery.cycle_count() {
        println!("Cycles: {}", cycles);
    }
    if let Some(time) = battery.time_remaining_formatted() {
        println!("Time remaining: {}", time);
    }

    println!("\n--- Power Metrics ---");
    let mut power = PowerData::new()?;
    std::thread::sleep(Duration::from_millis(500));
    power.refresh()?;
    println!("CPU Power: {:.2}W", power.cpu_power_watts());
    println!("GPU Power: {:.2}W", power.gpu_power_watts());
    println!("Total Power: {:.2}W", power.total_power_watts());
    println!("Power Mode: {}", power.power_mode_label());

    println!("\n--- Config Paths ---");
    println!("Config: {}", config_path().display());
    println!("Cache: {}", config::cache_dir().display());

    println!("\n--- Current Config ---");
    let config = UserConfig::load();
    println!("{}", toml::to_string_pretty(&config)?);

    Ok(())
}

fn run_config(path: bool, reset: bool, edit: bool) -> Result<()> {
    let config_file = config_path();

    if path {
        println!("{}", config_file.display());
        return Ok(());
    }

    if reset {
        let config = UserConfig::default();
        config.save()?;
        println!("Config reset to defaults at: {}", config_file.display());
        return Ok(());
    }

    if edit {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

        if !config_file.exists() {
            let config = UserConfig::default();
            config.save()?;
        }

        std::process::Command::new(editor)
            .arg(&config_file)
            .status()?;

        return Ok(());
    }

    let config = UserConfig::load();
    println!("Config file: {}", config_file.display());
    println!();
    println!("{}", toml::to_string_pretty(&config)?);

    Ok(())
}

fn run_theme(command: Option<ThemeCommands>) -> Result<()> {
    use theme::{contrast, get_all_themes, get_builtin_themes, load_user_themes, validation};

    let cmd = command.unwrap_or(ThemeCommands::Check {
        all: false,
        verbose: false,
    });

    match cmd {
        ThemeCommands::Check { all, verbose } => {
            let mut has_validation_errors = false;

            let validation_results = validation::validate_user_themes();
            if !validation_results.is_empty() {
                validation::print_validation_results(&validation_results, verbose);
                has_validation_errors = validation_results.iter().any(|r| !r.is_valid());
                println!();
            }

            let themes = if all {
                get_all_themes()
            } else {
                load_user_themes()
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
                    println!("All user themes have validation errors. Fix errors above to run contrast checks.");
                }
                if has_validation_errors {
                    std::process::exit(1);
                }
                return Ok(());
            }

            if !themes_for_contrast.is_empty() {
                let results = contrast::check_all_themes(&themes_for_contrast);
                contrast::print_results(&results, verbose);

                let has_contrast_failures = results.iter().any(|r| !r.pass);
                if has_contrast_failures || has_validation_errors {
                    std::process::exit(1);
                }
            }
        }
        ThemeCommands::Open => {
            let themes_dir = config::themes_dir();
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
            let themes_dir = config::themes_dir();
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
                    let base_theme = get_builtin_themes()
                        .into_iter()
                        .find(|t| t.id == base_id)
                        .ok_or_else(|| {
                            color_eyre::eyre::eyre!("Base theme '{}' not found", base_id)
                        })?;

                    theme::generate_theme_toml(&name, &base_theme)
                }
                _ => theme::generate_blank_theme_toml(&name),
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
                std::io::Write::flush(&mut std::io::stdout())?;

                let schemes = if let Some(ref query) = search {
                    theme::iterm2::search_schemes(query)
                } else {
                    theme::iterm2::list_available_schemes()
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
                                theme::iterm2::ITERM2_GALLERY_URL
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
                get_builtin_themes()
            } else if user {
                load_user_themes()
            } else {
                get_all_themes()
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

            match theme::iterm2::import_scheme(&scheme, name.as_deref()) {
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
                            theme::iterm2::SchemeVariant::Light
                        } else {
                            theme::iterm2::SchemeVariant::Dark
                        };
                        let missing_name = if has_dark { "light" } else { "dark" };

                        println!(
                            "\nOnly {} variant found. Looking for {} variant suggestions...",
                            if has_dark { "dark" } else { "light" },
                            missing_name
                        );

                        match theme::iterm2::find_variant_suggestions(&scheme, missing) {
                            Ok(suggestions) if !suggestions.is_empty() => {
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
                    if matches!(e, theme::iterm2::Iterm2Error::NotFound(_)) {
                        eprintln!("\nBrowse themes: {}", theme::iterm2::ITERM2_GALLERY_URL);
                        eprintln!("Search: jolt theme list --search <query>");
                    }
                    std::process::exit(1);
                }
            }
        }
        ThemeCommands::Fetch { force } => {
            println!("Fetching iTerm2 theme list...");

            match theme::cache::fetch_and_cache_schemes(force) {
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
            let themes_path = config::themes_dir();

            if !themes_path.exists() {
                println!("No user themes directory found.");
                return Ok(());
            }

            let theme_files: Vec<_> = std::fs::read_dir(&themes_path)
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
                std::io::Write::flush(&mut std::io::stdout())?;

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

fn run_daemon_command(
    command: DaemonCommands,
    log_level: LogLevel,
    log_level_override: Option<LogLevel>,
) -> Result<()> {
    use daemon::{is_daemon_running, log_path, run_daemon, socket_path, DaemonClient};

    match command {
        DaemonCommands::Start { foreground } => {
            if is_daemon_running() {
                println!("Daemon is already running.");
                return Ok(());
            }

            let mode = if foreground {
                LogMode::Both
            } else {
                LogMode::File
            };
            let _guard = logging::init(log_level, mode, log_level_override);

            if foreground {
                println!("Starting daemon in foreground...");
                println!("Press Ctrl+C to stop.");
                run_daemon(true).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
            } else {
                println!("Starting daemon...");
                run_daemon(false).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
                std::thread::sleep(Duration::from_millis(500));

                let mut started = false;
                for _ in 0..3 {
                    if is_daemon_running() {
                        started = true;
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }

                if started {
                    println!("Daemon started successfully.");
                    println!("Socket: {:?}", socket_path());
                } else {
                    println!("Daemon may have failed to start. Check logs:");
                    println!("  jolt daemon logs");
                }
            }
        }
        DaemonCommands::Stop => {
            if !is_daemon_running() {
                println!("Daemon is not running.");
                return Ok(());
            }

            match DaemonClient::connect() {
                Ok(mut client) => {
                    client
                        .shutdown()
                        .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
                    println!("Daemon stopped.");
                }
                Err(e) => {
                    eprintln!("Failed to connect to daemon: {}", e);
                    std::process::exit(1);
                }
            }
        }
        DaemonCommands::Status => {
            if !is_daemon_running() {
                println!("Daemon is not running.");
                return Ok(());
            }

            match DaemonClient::connect() {
                Ok(mut client) => {
                    let status = client
                        .get_status()
                        .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
                    println!("Daemon Status");
                    println!("{}", "-".repeat(40));
                    println!("Running:      yes");
                    println!("Version:      {}", status.version);
                    println!("Uptime:       {} seconds", status.uptime_secs);
                    println!("Samples:      {}", status.sample_count);
                    println!("Database:     {} bytes", status.database_size_bytes);
                    if let Some(last) = status.last_sample_time {
                        let dt = chrono::DateTime::from_timestamp(last, 0);
                        if let Some(dt) = dt {
                            println!("Last sample:  {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to daemon: {}", e);
                    std::process::exit(1);
                }
            }
        }
        DaemonCommands::Logs { lines, follow } => {
            let path = log_path();
            if !path.exists() {
                println!("No log file found at {:?}", path);
                return Ok(());
            }

            if follow {
                // Use exec() to replace this process with tail, so Ctrl+C works properly
                let err = std::process::Command::new("tail")
                    .args(["-f", "-n", &lines.to_string()])
                    .arg(&path)
                    .exec();
                // exec() only returns if it fails
                return Err(err.into());
            } else {
                std::process::Command::new("tail")
                    .args(["-n", &lines.to_string()])
                    .arg(&path)
                    .status()?;
            }
        }
        DaemonCommands::Install => {
            let plist_path = dirs::home_dir()
                .unwrap_or_default()
                .join("Library/LaunchAgents/com.jolt.daemon.plist");

            let exe_path = std::env::current_exe()?;

            let plist_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.jolt.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>daemon</string>
        <string>start</string>
        <string>--foreground</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>10</integer>
    <key>StandardErrorPath</key>
    <string>{}/jolt-daemon-stderr.log</string>
    <key>StandardOutPath</key>
    <string>{}/jolt-daemon-stdout.log</string>
</dict>
</plist>"#,
                exe_path.display(),
                config::runtime_dir().display(),
                config::runtime_dir().display()
            );

            if let Some(parent) = plist_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&plist_path, plist_content)?;

            std::process::Command::new("launchctl")
                .args(["load", "-w"])
                .arg(&plist_path)
                .status()?;

            println!("Daemon installed and started.");
            println!("Plist: {:?}", plist_path);
            println!("\nTo uninstall: jolt daemon uninstall");
        }
        DaemonCommands::Uninstall => {
            let plist_path = dirs::home_dir()
                .unwrap_or_default()
                .join("Library/LaunchAgents/com.jolt.daemon.plist");

            if !plist_path.exists() {
                println!("Daemon is not installed.");
                return Ok(());
            }

            std::process::Command::new("launchctl")
                .args(["unload"])
                .arg(&plist_path)
                .status()?;

            std::fs::remove_file(&plist_path)?;
            println!("Daemon uninstalled.");
        }
    }

    Ok(())
}

fn run_history_command(command: Option<HistoryCommands>) -> Result<()> {
    use data::HistoryStore;

    let cmd = command.unwrap_or(HistoryCommands::Summary {
        period: "week".to_string(),
    });

    let store = match HistoryStore::open() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open history database: {}", e);
            eprintln!("Make sure the daemon has been running to collect data.");
            std::process::exit(1);
        }
    };

    match cmd {
        HistoryCommands::Summary { period } => {
            let (from, to) = get_date_range(&period);

            println!("History Summary ({})", period);
            println!("{}", "=".repeat(50));

            match store.get_daily_stats(&from, &to) {
                Ok(stats) if stats.is_empty() => {
                    println!("No data for this period.");
                    println!("\nMake sure the daemon is running to collect data:");
                    println!("  jolt daemon start");
                }
                Ok(stats) => {
                    let total_energy: f32 = stats.iter().map(|s| s.total_energy_wh).sum();
                    let avg_power: f32 =
                        stats.iter().map(|s| s.avg_power).sum::<f32>() / stats.len() as f32;
                    let max_power: f32 = stats.iter().map(|s| s.max_power).fold(0.0, f32::max);

                    println!("Days recorded:    {}", stats.len());
                    println!(
                        "Total energy:     {:.1} Wh ({:.2} kWh)",
                        total_energy,
                        total_energy / 1000.0
                    );
                    println!("Avg power:        {:.1} W", avg_power);
                    println!("Max power:        {:.1} W", max_power);
                }
                Err(e) => {
                    eprintln!("Error reading stats: {}", e);
                }
            }
        }
        HistoryCommands::Top { period, limit } => {
            let (from, to) = get_date_range(&period);

            println!("Top Power Consumers ({})", period);
            println!("{}", "=".repeat(60));

            match store.get_top_processes_range(&from, &to, limit) {
                Ok(processes) if processes.is_empty() => {
                    println!("No process data for this period.");
                }
                Ok(processes) => {
                    println!(
                        "{:<4} {:<30} {:>10} {:>10}",
                        "Rank", "Process", "Avg CPU %", "Avg Mem MB"
                    );
                    println!("{}", "-".repeat(60));
                    for (i, p) in processes.iter().enumerate() {
                        println!(
                            "{:<4} {:<30} {:>10.1} {:>10.1}",
                            i + 1,
                            truncate_str(&p.process_name, 28),
                            p.avg_cpu,
                            p.avg_memory_mb
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error reading processes: {}", e);
                }
            }
        }
        HistoryCommands::Export {
            output,
            format,
            from,
            to,
            period,
            include_samples,
        } => {
            let (from_date, to_date) = if let (Some(f), Some(t)) = (from, to) {
                (f, t)
            } else if let Some(p) = period {
                get_date_range(&p)
            } else {
                get_date_range("week")
            };

            let daily_stats = store
                .get_daily_stats(&from_date, &to_date)
                .unwrap_or_default();
            let top_processes = store
                .get_top_processes_range(&from_date, &to_date, 20)
                .unwrap_or_default();

            let samples = if include_samples {
                store
                    .get_samples(
                        chrono::NaiveDate::parse_from_str(&from_date, "%Y-%m-%d")
                            .map(|d| {
                                let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                                d.and_time(time).and_utc().timestamp()
                            })
                            .unwrap_or(0),
                        chrono::NaiveDate::parse_from_str(&to_date, "%Y-%m-%d")
                            .map(|d| {
                                let time = chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap();
                                d.and_time(time).and_utc().timestamp()
                            })
                            .unwrap_or(i64::MAX),
                    )
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let content = match format.to_lowercase().as_str() {
                "csv" => {
                    export_to_csv(&from_date, &to_date, &daily_stats, &top_processes, &samples)
                }
                _ => export_to_json(&from_date, &to_date, &daily_stats, &top_processes, &samples),
            };

            if let Some(path) = output {
                std::fs::write(&path, &content)?;
                println!("Exported to: {}", path);
            } else {
                println!("{}", content);
            }
        }
        HistoryCommands::Prune { older_than, yes } => {
            let days = older_than.unwrap_or(30);
            let before_date = data::history_store::days_ago_date_string(days);

            let stats = store.get_stats().unwrap_or(data::DatabaseStats {
                sample_count: 0,
                hourly_count: 0,
                daily_count: 0,
                oldest_sample: None,
                newest_sample: None,
                size_bytes: 0,
            });

            println!("Current database stats:");
            println!("  Samples: {}", stats.sample_count);
            println!("  Size: {}", stats.size_formatted());
            println!(
                "\nWill delete data older than {} days (before {})",
                days, before_date
            );

            if !yes {
                print!("Proceed? [y/N] ");
                std::io::Write::flush(&mut std::io::stdout())?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let before_ts = chrono::NaiveDate::parse_from_str(&before_date, "%Y-%m-%d")
                .map(|d| {
                    let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                    d.and_time(time).and_utc().timestamp()
                })
                .unwrap_or(0);

            let deleted_samples = store.delete_samples_before(before_ts).unwrap_or(0);
            let deleted_hourly = store.delete_hourly_stats_before(before_ts).unwrap_or(0);
            let deleted_daily = store.delete_daily_stats_before(&before_date).unwrap_or(0);
            let deleted_processes = store
                .delete_daily_processes_before(&before_date)
                .unwrap_or(0);

            println!("\nDeleted:");
            println!("  {} samples", deleted_samples);
            println!("  {} hourly stats", deleted_hourly);
            println!("  {} daily stats", deleted_daily);
            println!("  {} process entries", deleted_processes);

            if let Err(e) = store.vacuum() {
                eprintln!("Warning: vacuum failed: {}", e);
            } else {
                println!("\nDatabase vacuumed to reclaim space.");
            }
        }
    }

    Ok(())
}

fn get_date_range(period: &str) -> (String, String) {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    match period.to_lowercase().as_str() {
        "today" => (today.clone(), today),
        "week" => (data::history_store::days_ago_date_string(7), today),
        "month" => (data::history_store::days_ago_date_string(30), today),
        "all" => ("2000-01-01".to_string(), today),
        _ => (data::history_store::days_ago_date_string(7), today),
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn export_to_json(
    from: &str,
    to: &str,
    daily_stats: &[data::DailyStat],
    top_processes: &[data::DailyTopProcess],
    samples: &[data::Sample],
) -> String {
    let export_data = serde_json::json!({
        "period": {
            "from": from,
            "to": to,
        },
        "daily_stats": daily_stats,
        "top_processes": top_processes,
        "samples": samples,
    });
    serde_json::to_string_pretty(&export_data).unwrap_or_default()
}

fn export_to_csv(
    from: &str,
    to: &str,
    daily_stats: &[data::DailyStat],
    top_processes: &[data::DailyTopProcess],
    samples: &[data::Sample],
) -> String {
    let mut output = String::new();

    output.push_str(&format!("# Jolt History Export: {} to {}\n\n", from, to));

    output.push_str("# Daily Statistics\n");
    output
        .push_str("date,avg_power_w,max_power_w,total_energy_wh,screen_on_hours,charging_hours\n");
    for stat in daily_stats {
        output.push_str(&format!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
            stat.date,
            stat.avg_power,
            stat.max_power,
            stat.total_energy_wh,
            stat.screen_on_hours,
            stat.charging_hours
        ));
    }

    output.push_str("\n# Top Processes\n");
    output.push_str(
        "process_name,avg_power_w,total_energy_wh,avg_cpu_percent,avg_memory_mb,sample_count\n",
    );
    for proc in top_processes {
        output.push_str(&format!(
            "{},{:.2},{:.2},{:.2},{:.2},{}\n",
            escape_csv(&proc.process_name),
            proc.avg_power,
            proc.total_energy_wh,
            proc.avg_cpu,
            proc.avg_memory_mb,
            proc.sample_count
        ));
    }

    if !samples.is_empty() {
        output.push_str("\n# Raw Samples\n");
        output
            .push_str("timestamp,battery_percent,power_watts,cpu_power,gpu_power,charging_state\n");
        for sample in samples {
            let charging = match sample.charging_state {
                data::ChargingState::Discharging => "discharging",
                data::ChargingState::Charging => "charging",
                data::ChargingState::Full => "full",
                data::ChargingState::Unknown => "unknown",
            };
            output.push_str(&format!(
                "{},{:.1},{:.2},{:.2},{:.2},{}\n",
                sample.timestamp,
                sample.battery_percent,
                sample.power_watts,
                sample.cpu_power,
                sample.gpu_power,
                charging
            ));
        }
    }

    output
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        let escaped = s.replace('"', "\"\"").replace('\n', " ");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}
