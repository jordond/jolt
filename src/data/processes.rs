use color_eyre::eyre::Result;
use std::collections::HashMap;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub energy_impact: f32,
    pub parent_pid: Option<u32>,
    pub children: Option<Vec<ProcessInfo>>,
}

impl ProcessInfo {
    pub fn energy_level(&self) -> EnergyLevel {
        if self.energy_impact >= 20.0 {
            EnergyLevel::High
        } else if self.energy_impact >= 5.0 {
            EnergyLevel::Medium
        } else {
            EnergyLevel::Low
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyLevel {
    Low,
    Medium,
    High,
}

pub struct ProcessData {
    system: System,
    pub processes: Vec<ProcessInfo>,
}

impl ProcessData {
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        let mut data = Self {
            system,
            processes: Vec::new(),
        };

        data.refresh()?;
        Ok(data)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.system.refresh_processes();

        let mut process_map: HashMap<u32, ProcessInfo> = HashMap::new();
        let mut children_map: HashMap<u32, Vec<ProcessInfo>> = HashMap::new();

        for (pid, process) in self.system.processes() {
            let pid_u32 = pid.as_u32();
            let parent_pid = process.parent().map(|p| p.as_u32());

            let cpu = process.cpu_usage();
            let memory_mb = process.memory() as f64 / (1024.0 * 1024.0);

            let energy_impact = calculate_energy_impact(cpu, memory_mb as f32);

            let info = ProcessInfo {
                pid: pid_u32,
                name: process.name().to_string(),
                cpu_usage: cpu,
                memory_mb,
                energy_impact,
                parent_pid,
                children: None,
            };

            process_map.insert(pid_u32, info.clone());

            if let Some(parent) = parent_pid {
                children_map.entry(parent).or_default().push(info);
            }
        }

        let mut top_processes: Vec<ProcessInfo> = Vec::new();

        for (pid, mut process) in process_map {
            if let Some(children) = children_map.remove(&pid) {
                let total_energy: f32 = children.iter().map(|c| c.energy_impact).sum();
                process.energy_impact += total_energy * 0.3;

                let mut sorted_children = children;
                sorted_children.sort_by(|a, b| {
                    b.energy_impact
                        .partial_cmp(&a.energy_impact)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                if !sorted_children.is_empty() {
                    process.children = Some(sorted_children);
                }
            }

            if process.energy_impact >= 0.5 {
                top_processes.push(process);
            }
        }

        top_processes.sort_by(|a, b| {
            b.energy_impact
                .partial_cmp(&a.energy_impact)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.processes = top_processes.into_iter().take(50).collect();

        Ok(())
    }

    pub fn kill_process(&self, pid: u32) -> Result<()> {
        use std::process::Command;

        Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_process(&self, pid: u32) -> Option<&ProcessInfo> {
        self.processes.iter().find(|p| p.pid == pid)
    }
}

fn calculate_energy_impact(cpu_usage: f32, memory_mb: f32) -> f32 {
    let cpu_factor = cpu_usage * 0.8;
    let memory_factor = (memory_mb / 100.0).min(20.0) * 0.2;

    cpu_factor + memory_factor
}
