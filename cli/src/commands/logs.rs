use std::os::unix::process::CommandExt;

use color_eyre::eyre::Result;

use crate::config;

pub fn run(lines: usize, follow: bool) -> Result<()> {
    let log_dir = config::runtime_dir();

    let mut log_files: Vec<_> = std::fs::read_dir(&log_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name();
                    let name = name.to_string_lossy();
                    name.starts_with("jolt.") && name.ends_with(".log")
                })
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default();

    log_files.sort();

    let Some(path) = log_files.last() else {
        println!("No log files found in {:?}", log_dir);
        println!("Log files are created when running jolt or the daemon.");
        return Ok(());
    };

    if follow {
        let err = std::process::Command::new("tail")
            .args(["-f", "-n", &lines.to_string()])
            .arg(path)
            .exec();
        return Err(err.into());
    } else {
        std::process::Command::new("tail")
            .args(["-n", &lines.to_string()])
            .arg(path)
            .status()?;
    }

    Ok(())
}
