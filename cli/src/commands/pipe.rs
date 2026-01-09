use std::time::Duration;

use color_eyre::eyre::Result;
use serde_json::json;

use crate::data::{BatteryData, PowerData, ProcessData};

pub fn run(samples: u32, interval: u64, compact: bool) -> Result<()> {
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
