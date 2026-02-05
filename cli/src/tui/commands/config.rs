use color_eyre::eyre::Result;

use crate::config::{config_path, UserConfig};

pub fn run(path: bool, reset: bool, edit: bool) -> Result<()> {
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
