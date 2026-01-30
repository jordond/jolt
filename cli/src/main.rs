mod app;
mod cli;
mod commands;
mod config;
mod daemon;
mod data;
mod input;
mod logging;
mod settings;
mod theme;
mod ui;

use app::run_tui;
use clap::Parser;
use color_eyre::eyre::Result;

use cli::{Cli, Commands};
use config::{ensure_dirs, LogLevel, UserConfig};
use data::BatteryData;
use logging::LogMode;

fn require_battery() {
    if !BatteryData::is_available() {
        eprintln!("No battery found. jolt is designed for laptops and battery-powered devices.");
        std::process::exit(1);
    }
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
            require_battery();
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            commands::pipe::run(samples, interval, compact)
        }
        Some(Commands::Debug) => {
            require_battery();
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            commands::debug::run()
        }
        Some(Commands::Config { path, reset, edit }) => {
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            commands::config::run(path, reset, edit)
        }
        Some(Commands::Theme { command }) => {
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            commands::theme::run(command)
        }
        Some(Commands::Daemon { command }) => {
            commands::daemon::run(command, config.log_level, log_level_override)
        }
        Some(Commands::History { command }) => {
            let _guard = logging::init(config.log_level, LogMode::Stderr, log_level_override);
            commands::history::run(command)
        }
        Some(Commands::Logs { lines, follow }) => commands::logs::run(lines, follow),
        Some(Commands::Ui) | None => {
            require_battery();
            let _guard = logging::init(config.log_level, LogMode::File, log_level_override);
            run_tui(config)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    use crate::cli::{DaemonCommands, HistoryCommands, ThemeCommands};
    use crate::commands::history::{escape_csv, get_date_range};
    use crate::ui::utils::truncate_str;

    #[test]
    fn cli_configuration_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn cli_parse_no_args_returns_none() {
        let cli = Cli::try_parse_from(["jolt"]).unwrap();
        assert!(cli.command.is_none());
        assert!(cli.log_level.is_none());
    }

    #[test]
    fn cli_parse_ui_command() {
        let cli = Cli::try_parse_from(["jolt", "ui"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Ui)));
    }

    #[test]
    fn cli_parse_pipe_has_correct_defaults() {
        let cli = Cli::try_parse_from(["jolt", "pipe"]).unwrap();
        match cli.command {
            Some(Commands::Pipe {
                samples,
                interval,
                compact,
            }) => {
                assert_eq!(samples, 0);
                assert_eq!(interval, 1000);
                assert!(!compact);
            }
            _ => panic!("Expected Pipe command"),
        }
    }

    #[test]
    fn cli_parse_pipe_with_custom_args() {
        let cli = Cli::try_parse_from(["jolt", "pipe", "-s", "5", "-i", "500", "-c"]).unwrap();
        match cli.command {
            Some(Commands::Pipe {
                samples,
                interval,
                compact,
            }) => {
                assert_eq!(samples, 5);
                assert_eq!(interval, 500);
                assert!(compact);
            }
            _ => panic!("Expected Pipe command"),
        }
    }

    #[test]
    fn cli_parse_debug_command() {
        let cli = Cli::try_parse_from(["jolt", "debug"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Debug)));
    }

    #[test]
    fn cli_parse_config_path_flag() {
        let cli = Cli::try_parse_from(["jolt", "config", "--path"]).unwrap();
        match cli.command {
            Some(Commands::Config { path, reset, edit }) => {
                assert!(path);
                assert!(!reset);
                assert!(!edit);
            }
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn cli_parse_daemon_start_foreground() {
        let cli = Cli::try_parse_from(["jolt", "daemon", "start", "--foreground"]).unwrap();
        match cli.command {
            Some(Commands::Daemon { command }) => {
                assert!(matches!(
                    command,
                    DaemonCommands::Start { foreground: true }
                ));
            }
            _ => panic!("Expected Daemon command"),
        }
    }

    #[test]
    fn cli_parse_theme_check_all() {
        let cli = Cli::try_parse_from(["jolt", "theme", "check", "--all"]).unwrap();
        match cli.command {
            Some(Commands::Theme { command }) => {
                assert!(matches!(
                    command,
                    Some(ThemeCommands::Check {
                        all: true,
                        verbose: false
                    })
                ));
            }
            _ => panic!("Expected Theme command"),
        }
    }

    #[test]
    fn cli_parse_theme_list_iterm2() {
        let cli = Cli::try_parse_from(["jolt", "theme", "list", "--iterm2"]).unwrap();
        match cli.command {
            Some(Commands::Theme { command }) => {
                assert!(matches!(
                    command,
                    Some(ThemeCommands::List { iterm2: true, .. })
                ));
            }
            _ => panic!("Expected Theme command"),
        }
    }

    #[test]
    fn cli_parse_history_top_with_period_and_limit() {
        let cli =
            Cli::try_parse_from(["jolt", "history", "top", "-p", "month", "-l", "20"]).unwrap();
        match cli.command {
            Some(Commands::History { command }) => match command {
                Some(HistoryCommands::Top { period, limit }) => {
                    assert_eq!(period, "month");
                    assert_eq!(limit, 20);
                }
                _ => panic!("Expected Top subcommand"),
            },
            _ => panic!("Expected History command"),
        }
    }

    #[test]
    fn cli_parse_global_log_level_before_subcommand() {
        let cli = Cli::try_parse_from(["jolt", "--log-level", "debug", "ui"]).unwrap();
        assert_eq!(cli.log_level, Some("debug".to_string()));
    }

    #[test]
    fn get_date_range_today_returns_same_from_and_to() {
        let (from, to) = get_date_range("today");
        assert_eq!(from, to);
    }

    #[test]
    fn get_date_range_returns_yyyy_mm_dd_format() {
        let (from, _) = get_date_range("today");
        assert_eq!(from.len(), 10, "date should be 10 chars: YYYY-MM-DD");
        assert_eq!(from.chars().nth(4), Some('-'));
        assert_eq!(from.chars().nth(7), Some('-'));
    }

    #[test]
    fn get_date_range_week_to_equals_today() {
        let (from, to) = get_date_range("week");
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert_ne!(from, to);
        assert_eq!(to, today);
    }

    #[test]
    fn get_date_range_month_to_equals_today() {
        let (from, to) = get_date_range("month");
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert_ne!(from, to);
        assert_eq!(to, today);
    }

    #[test]
    fn get_date_range_all_starts_from_2000() {
        let (from, to) = get_date_range("all");
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert_eq!(from, "2000-01-01");
        assert_eq!(to, today);
    }

    #[test]
    fn get_date_range_is_case_insensitive() {
        let (from1, _) = get_date_range("TODAY");
        let (from2, _) = get_date_range("today");
        let (from3, _) = get_date_range("Today");
        assert_eq!(from1, from2);
        assert_eq!(from2, from3);
    }

    #[test]
    fn get_date_range_unknown_period_defaults_to_week() {
        let (from_unknown, to_unknown) = get_date_range("unknown");
        let (from_week, to_week) = get_date_range("week");
        assert_eq!(from_unknown, from_week);
        assert_eq!(to_unknown, to_week);
    }

    #[test]
    fn truncate_str_returns_unchanged_when_shorter_than_max() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn truncate_str_returns_unchanged_at_exact_max_length() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn truncate_str_adds_ellipsis_when_exceeds_max() {
        assert_eq!(truncate_str("hello world", 8), "hello...");
    }

    #[test]
    fn truncate_str_with_max_4_keeps_1_char_plus_ellipsis() {
        assert_eq!(truncate_str("hello", 4), "h...");
    }

    #[test]
    fn escape_csv_returns_unchanged_for_plain_text() {
        assert_eq!(escape_csv("hello world"), "hello world");
    }

    #[test]
    fn escape_csv_quotes_string_with_comma() {
        assert_eq!(escape_csv("hello, world"), "\"hello, world\"");
    }

    #[test]
    fn escape_csv_doubles_quotes_inside_string() {
        assert_eq!(escape_csv("hello \"world\""), "\"hello \"\"world\"\"\"");
    }

    #[test]
    fn escape_csv_replaces_newline_with_space() {
        assert_eq!(escape_csv("hello\nworld"), "\"hello world\"");
    }

    #[test]
    fn escape_csv_handles_multiple_special_chars() {
        assert_eq!(
            escape_csv("hello, \"world\"\nnew line"),
            "\"hello, \"\"world\"\" new line\""
        );
    }

    #[test]
    fn escape_csv_returns_empty_for_empty_input() {
        assert_eq!(escape_csv(""), "");
    }
}
