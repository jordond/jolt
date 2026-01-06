mod app;
mod config;
mod data;
mod input;
mod theme;
mod ui;

use std::io;
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

use config::{config_path, ensure_dirs, UserConfig};

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
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let _ = ensure_dirs();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Pipe {
            samples,
            interval,
            compact,
        }) => run_pipe(samples, interval, compact),
        Some(Commands::Debug) => run_debug(),
        Some(Commands::Config { path, reset, edit }) => run_config(path, reset, edit),
        Some(Commands::Theme { command }) => run_theme(command),
        Some(Commands::Ui {
            refresh_ms,
            appearance,
            low_power,
        }) => {
            let mut config = UserConfig::load();
            let refresh_from_cli =
                config.merge_with_args(appearance.as_deref(), refresh_ms, low_power);
            run_tui(config, refresh_from_cli)
        }
        None => {
            let mut config = UserConfig::load();
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
    use theme::{contrast, get_all_themes, get_builtin_themes, load_user_themes};

    let cmd = command.unwrap_or(ThemeCommands::Check {
        all: false,
        verbose: false,
    });

    match cmd {
        ThemeCommands::Check { all, verbose } => {
            let themes = if all {
                get_all_themes()
            } else {
                load_user_themes()
            };

            if themes.is_empty() {
                if all {
                    println!("No themes found.");
                } else {
                    println!("No user themes found.");
                    println!("Use --all to check builtin themes, or create a theme with:");
                    println!("  jolt theme create <name>");
                }
                return Ok(());
            }

            let results = contrast::check_all_themes(&themes);
            contrast::print_results(&results, verbose);

            let has_failures = results.iter().any(|r| !r.pass);
            if has_failures {
                std::process::exit(1);
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
        ThemeCommands::List { builtin, user } => {
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
    }

    Ok(())
}
