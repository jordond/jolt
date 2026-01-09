use std::io::Write;

use color_eyre::eyre::Result;

use crate::cli::HistoryCommands;
use crate::data::{self, HistoryStore};

pub fn run(command: Option<HistoryCommands>) -> Result<()> {
    let cmd = command.unwrap_or(HistoryCommands::Summary {
        period: "week".to_string(),
    });

    let store = match HistoryStore::open() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open history database: {}", e);
            eprintln!("Make sure the daemon has been running to collect data.");
            std::process::exit(1);
        }
    };

    match cmd {
        HistoryCommands::Summary { period } => {
            let (from, to) = get_date_range(&period);

            println!("History Summary ({})", period);
            println!("{}", "=".repeat(50));

            match store.get_daily_stats(&from, &to) {
                Ok(stats) if stats.is_empty() => {
                    println!("No data for this period.");
                    println!("\nMake sure the daemon is running to collect data:");
                    println!("  jolt daemon start");
                }
                Ok(stats) => {
                    let total_energy: f32 = stats.iter().map(|s| s.total_energy_wh).sum();
                    let avg_power: f32 =
                        stats.iter().map(|s| s.avg_power).sum::<f32>() / stats.len() as f32;
                    let max_power: f32 = stats.iter().map(|s| s.max_power).fold(0.0, f32::max);

                    println!("Days recorded:    {}", stats.len());
                    println!(
                        "Total energy:     {:.1} Wh ({:.2} kWh)",
                        total_energy,
                        total_energy / 1000.0
                    );
                    println!("Avg power:        {:.1} W", avg_power);
                    println!("Max power:        {:.1} W", max_power);
                }
                Err(e) => {
                    eprintln!("Error reading stats: {}", e);
                }
            }
        }
        HistoryCommands::Top { period, limit } => {
            let (from, to) = get_date_range(&period);

            println!("Top Power Consumers ({})", period);
            println!("{}", "=".repeat(60));

            match store.get_top_processes_range(&from, &to, limit) {
                Ok(processes) if processes.is_empty() => {
                    println!("No process data for this period.");
                }
                Ok(processes) => {
                    println!(
                        "{:<4} {:<30} {:>10} {:>10}",
                        "Rank", "Process", "Avg CPU %", "Avg Mem MB"
                    );
                    println!("{}", "-".repeat(60));
                    for (i, p) in processes.iter().enumerate() {
                        println!(
                            "{:<4} {:<30} {:>10.1} {:>10.1}",
                            i + 1,
                            truncate_str(&p.process_name, 28),
                            p.avg_cpu,
                            p.avg_memory_mb
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error reading processes: {}", e);
                }
            }
        }
        HistoryCommands::Export {
            output,
            format,
            from,
            to,
            period,
            include_samples,
        } => {
            let (from_date, to_date) = if let (Some(f), Some(t)) = (from, to) {
                (f, t)
            } else if let Some(p) = period {
                get_date_range(&p)
            } else {
                get_date_range("week")
            };

            let daily_stats = store
                .get_daily_stats(&from_date, &to_date)
                .unwrap_or_default();
            let top_processes = store
                .get_top_processes_range(&from_date, &to_date, 20)
                .unwrap_or_default();

            let samples = if include_samples {
                store
                    .get_samples(
                        chrono::NaiveDate::parse_from_str(&from_date, "%Y-%m-%d")
                            .map(|d| {
                                let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                                d.and_time(time).and_utc().timestamp()
                            })
                            .unwrap_or(0),
                        chrono::NaiveDate::parse_from_str(&to_date, "%Y-%m-%d")
                            .map(|d| {
                                let time = chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap();
                                d.and_time(time).and_utc().timestamp()
                            })
                            .unwrap_or(i64::MAX),
                    )
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let content = match format.to_lowercase().as_str() {
                "csv" => {
                    export_to_csv(&from_date, &to_date, &daily_stats, &top_processes, &samples)
                }
                _ => export_to_json(&from_date, &to_date, &daily_stats, &top_processes, &samples),
            };

            if let Some(path) = output {
                std::fs::write(&path, &content)?;
                println!("Exported to: {}", path);
            } else {
                println!("{}", content);
            }
        }
        HistoryCommands::Prune { older_than, yes } => {
            let days = older_than.unwrap_or(30);
            let before_date = data::history_store::days_ago_date_string(days);

            let stats = store.get_stats().unwrap_or(data::DatabaseStats {
                sample_count: 0,
                hourly_count: 0,
                daily_count: 0,
                oldest_sample: None,
                newest_sample: None,
                size_bytes: 0,
            });

            println!("Current database stats:");
            println!("  Samples: {}", stats.sample_count);
            println!("  Size: {}", stats.size_formatted());
            println!(
                "\nWill delete data older than {} days (before {})",
                days, before_date
            );

            if !yes {
                print!("Proceed? [y/N] ");
                std::io::stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let before_ts = chrono::NaiveDate::parse_from_str(&before_date, "%Y-%m-%d")
                .map(|d| {
                    let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                    d.and_time(time).and_utc().timestamp()
                })
                .unwrap_or(0);

            let deleted_samples = store.delete_samples_before(before_ts).unwrap_or(0);
            let deleted_hourly = store.delete_hourly_stats_before(before_ts).unwrap_or(0);
            let deleted_daily = store.delete_daily_stats_before(&before_date).unwrap_or(0);
            let deleted_processes = store
                .delete_daily_processes_before(&before_date)
                .unwrap_or(0);

            println!("\nDeleted:");
            println!("  {} samples", deleted_samples);
            println!("  {} hourly stats", deleted_hourly);
            println!("  {} daily stats", deleted_daily);
            println!("  {} process entries", deleted_processes);

            if let Err(e) = store.vacuum() {
                eprintln!("Warning: vacuum failed: {}", e);
            } else {
                println!("\nDatabase vacuumed to reclaim space.");
            }
        }
    }

    Ok(())
}

pub fn get_date_range(period: &str) -> (String, String) {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    match period.to_lowercase().as_str() {
        "today" => (today.clone(), today),
        "week" => (data::history_store::days_ago_date_string(7), today),
        "month" => (data::history_store::days_ago_date_string(30), today),
        "all" => ("2000-01-01".to_string(), today),
        _ => (data::history_store::days_ago_date_string(7), today),
    }
}

pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn export_to_json(
    from: &str,
    to: &str,
    daily_stats: &[data::DailyStat],
    top_processes: &[data::DailyTopProcess],
    samples: &[data::Sample],
) -> String {
    let export_data = serde_json::json!({
        "period": {
            "from": from,
            "to": to,
        },
        "daily_stats": daily_stats,
        "top_processes": top_processes,
        "samples": samples,
    });
    serde_json::to_string_pretty(&export_data).unwrap_or_default()
}

fn export_to_csv(
    from: &str,
    to: &str,
    daily_stats: &[data::DailyStat],
    top_processes: &[data::DailyTopProcess],
    samples: &[data::Sample],
) -> String {
    let mut output = String::new();

    output.push_str(&format!("# Jolt History Export: {} to {}\n\n", from, to));

    output.push_str("# Daily Statistics\n");
    output
        .push_str("date,avg_power_w,max_power_w,total_energy_wh,screen_on_hours,charging_hours\n");
    for stat in daily_stats {
        output.push_str(&format!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
            stat.date,
            stat.avg_power,
            stat.max_power,
            stat.total_energy_wh,
            stat.screen_on_hours,
            stat.charging_hours
        ));
    }

    output.push_str("\n# Top Processes\n");
    output.push_str(
        "process_name,avg_power_w,total_energy_wh,avg_cpu_percent,avg_memory_mb,sample_count\n",
    );
    for proc in top_processes {
        output.push_str(&format!(
            "{},{:.2},{:.2},{:.2},{:.2},{}\n",
            escape_csv(&proc.process_name),
            proc.avg_power,
            proc.total_energy_wh,
            proc.avg_cpu,
            proc.avg_memory_mb,
            proc.sample_count
        ));
    }

    if !samples.is_empty() {
        output.push_str("\n# Raw Samples\n");
        output
            .push_str("timestamp,battery_percent,power_watts,cpu_power,gpu_power,charging_state\n");
        for sample in samples {
            let charging = match sample.charging_state {
                data::ChargingState::Discharging => "discharging",
                data::ChargingState::Charging => "charging",
                data::ChargingState::Full => "full",
                data::ChargingState::Unknown => "unknown",
            };
            output.push_str(&format!(
                "{},{:.1},{:.2},{:.2},{:.2},{}\n",
                sample.timestamp,
                sample.battery_percent,
                sample.power_watts,
                sample.cpu_power,
                sample.gpu_power,
                charging
            ));
        }
    }

    output
}

pub fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        let escaped = s.replace('"', "\"\"").replace('\n', " ");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}
