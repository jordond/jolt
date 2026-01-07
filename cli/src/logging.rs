use std::sync::OnceLock;

use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::{runtime_dir, LogLevel};

static INIT: OnceLock<()> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogMode {
    File,
    Stderr,
    Both,
}

pub struct LogGuard {
    _guard: Option<WorkerGuard>,
}

pub fn init(level: LogLevel, mode: LogMode, cli_override: Option<LogLevel>) -> LogGuard {
    let mut guard = None;

    INIT.get_or_init(|| {
        let effective_level = cli_override.unwrap_or(level);

        let Some(tracing_level) = effective_level.as_tracing_level() else {
            return;
        };

        guard = match mode {
            LogMode::File => init_file_logging(tracing_level),
            LogMode::Stderr => {
                init_stderr_logging(tracing_level);
                None
            }
            LogMode::Both => init_both_logging(tracing_level),
        };
    });

    LogGuard { _guard: guard }
}

fn build_env_filter(level: Level) -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy()
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("reqwest=warn".parse().unwrap())
        .add_directive("rusqlite=warn".parse().unwrap())
}

fn init_file_logging(level: Level) -> Option<WorkerGuard> {
    let log_dir = runtime_dir();

    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!(
            "Warning: Failed to create log directory {:?}: {}",
            log_dir, e
        );
        return None;
    }

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("jolt")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
        .ok()?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_timer(UtcTime::rfc_3339())
        .with_ansi(false)
        .with_target(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(build_env_filter(level))
        .with(file_layer)
        .init();

    Some(guard)
}

fn init_stderr_logging(level: Level) {
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_timer(UtcTime::rfc_3339())
        .with_ansi(true)
        .with_target(true);

    tracing_subscriber::registry()
        .with(build_env_filter(level))
        .with(stderr_layer)
        .init();
}

fn init_both_logging(level: Level) -> Option<WorkerGuard> {
    let log_dir = runtime_dir();
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("jolt-daemon")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
        .ok()?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_timer(UtcTime::rfc_3339())
        .with_ansi(false)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(build_env_filter(level));

    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_timer(UtcTime::rfc_3339())
        .with_ansi(true)
        .with_target(true)
        .with_filter(build_env_filter(level));

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .init();

    Some(guard)
}

#[allow(dead_code)]
pub fn log_file_location() -> std::path::PathBuf {
    runtime_dir().join("jolt.log")
}
