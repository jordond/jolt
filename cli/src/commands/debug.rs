use std::time::Duration;

use color_eyre::eyre::Result;

use crate::config::{self, config_path, UserConfig};
use crate::data::{BatteryData, PowerData};

pub fn run() -> Result<()> {
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
