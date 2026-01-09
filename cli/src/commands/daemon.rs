use std::time::Duration;

use color_eyre::eyre::Result;

use crate::cli::DaemonCommands;
use crate::config::LogLevel;
use crate::daemon::{is_daemon_running, run_daemon, socket_path, DaemonClient};
use crate::logging::{self, LogMode};

pub fn run(
    command: DaemonCommands,
    log_level: LogLevel,
    log_level_override: Option<LogLevel>,
) -> Result<()> {
    match command {
        DaemonCommands::Start { foreground } => {
            if is_daemon_running() {
                println!("Daemon is already running.");
                return Ok(());
            }

            if foreground {
                let _guard = logging::init(log_level, LogMode::Both, log_level_override);
                println!("Starting daemon in foreground...");
                println!("Press Ctrl+C to stop.");
                run_daemon(true, log_level, log_level_override)
                    .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
            } else {
                println!("Starting daemon...");
                run_daemon(false, log_level, log_level_override)
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
            println!("Daemon Status");
            println!("{}", "-".repeat(40));

            let service_status = crate::daemon::service::get_service_status();
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
        }
        DaemonCommands::Install { force } => {
            crate::daemon::service::install_service(force)?;
        }
        DaemonCommands::Uninstall => {
            crate::daemon::service::uninstall_service()?;
        }
    }

    Ok(())
}
