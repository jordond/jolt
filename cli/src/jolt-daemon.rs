mod daemon;
mod logging;
mod data;
mod config;

use std::time::Duration;
use clap::Parser;
use color_eyre::eyre::Result;

use crate::daemon::{is_daemon_running, server::run_daemon, socket_path, DaemonClient, service::get_service_status};

use data::BatteryData;
use logging::{LogMode, LogLevel};
use config::ensure_dirs;
use crate::config::UserConfig;
use crate::daemon::cli::{DaemonCli, DaemonCommands};

fn main() -> Result<()> {
    color_eyre::install()?;
    let _ = ensure_dirs();
    
    let cli = DaemonCli::parse();
    let config = UserConfig::load();
    let log_level_override = cli.log_level.as_deref().map(LogLevel::from_str);

    let command = cli.command.unwrap_or_else(|| DaemonCommands::Start { foreground: true });

    let log_level = log_level_override.unwrap_or_else(|| config.log_level.clone());

    match command {
        DaemonCommands::Start { foreground } => daemon_start(foreground, log_level),
        DaemonCommands::Stop => daemon_stop(),
        DaemonCommands::Status => daemon_status(),
        #[cfg(target_os = "macos")]
        DaemonCommands::Install { force } => service::install_service(force),
        #[cfg(target_os = "macos")]
        DaemonCommands::Uninstall => service::uninstall_service(),
    }
}

fn daemon_start(foreground: bool, log_level: LogLevel) -> Result<()> {
    require_battery();
    if is_daemon_running() {
        println!("Daemon is already running.");
        return Ok(());
    }

    if foreground {
        let _guard = logging::init(log_level, LogMode::Both, None);
        println!("Starting daemon in foreground...");
        println!("Press Ctrl+C to stop.");
        run_daemon(true, log_level)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
    } else {
        println!("Starting daemon...");
        run_daemon(false, log_level)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
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
            println!("  jolt logs");
        }
    }
    Ok(())
}

fn daemon_stop() -> Result<()> {
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
    Ok(())
}

fn daemon_status() -> Result<()> {
    println!("Daemon Status");
    println!("{}", "-".repeat(40));

    let service_status = get_service_status();
    println!("{}", service_status.display());
    println!();

    if is_daemon_running() {
        match DaemonClient::connect() {
            Ok(mut client) => {
                let status = client
                    .get_status()
                    .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
                println!("Process Status");
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
    } else {
        println!("Process Status");
        println!("{}", "-".repeat(40));
        println!("Running:      no");
    }
    Ok(())
}

fn require_battery() {
    if !BatteryData::is_available() {
        eprintln!("No battery found. jolt is designed for laptops and battery-powered devices.");
        std::process::exit(1);
    }
}

