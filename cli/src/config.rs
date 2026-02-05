use std::{env, fs};
use std::path::PathBuf;

// XDG Base Directory environment variables
pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
pub const XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";

pub fn config_dir() -> PathBuf {
    env::var(XDG_CONFIG_HOME)
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("jolt")
}

pub fn cache_dir() -> PathBuf {
    env::var(XDG_CACHE_HOME)
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("jolt")
}

pub fn data_dir() -> PathBuf {
    env::var(XDG_DATA_HOME)
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("jolt")
}

pub fn runtime_dir() -> PathBuf {
    env::var(XDG_RUNTIME_DIR)
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(dirs::runtime_dir)
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("jolt")
}
pub fn ensure_dirs() -> std::io::Result<()> {
    fs::create_dir_all(config_dir())?;
    fs::create_dir_all(cache_dir())?;
    fs::create_dir_all(runtime_dir())?;
    Ok(())
}