//! Process management methods for App.
//!
//! This module contains methods for managing process selection, scrolling,
//! merging, and process killing functionality.

use std::collections::HashMap;

use crate::daemon::{DaemonClient, KillSignal};
use crate::data::ProcessInfo;

use super::types::SortColumn;
use super::App;

/// Extracts the base process name by stripping common suffixes.
///
/// This function removes helper, renderer, GPU, plugin, and web content
/// suffixes that are commonly appended to process names by browsers and
/// other applications.
fn get_base_process_name(name: &str) -> String {
    let name = name
        .trim_end_matches(" Helper")
        .trim_end_matches(" Helper (Renderer)")
        .trim_end_matches(" Helper (GPU)")
        .trim_end_matches(" Helper (Plugin)")
        .trim_end_matches(" Renderer")
        .trim_end_matches(" (GPU Process)")
        .trim_end_matches(" Web Content");

    if let Some(pos) = name.rfind(" (") {
        if name.ends_with(')') {
            return name[..pos].to_string();
        }
    }

    name.to_string()
}

impl App {
    /// Returns the count of currently visible processes.
    pub fn visible_process_count(&self) -> usize {
        self.get_visible_processes().len()
    }

    /// Returns a list of visible processes with their depth level.
    ///
    /// The depth level indicates nesting (0 for top-level, 1 for children).
    /// Processes are sorted according to the current sort column and direction.
    /// In merge mode, related processes are grouped together.
    pub fn get_visible_processes(&self) -> Vec<(ProcessInfo, u8)> {
        let processes = if let Some(ref frozen) = self.frozen_processes {
            frozen.clone()
        } else {
            self.processes.processes.clone()
        };

        let sorted = if self.merge_mode {
            self.merge_processes(processes)
        } else {
            processes
        };

        let mut sorted = sorted;
        let asc = self.sort_ascending;
        match self.sort_column {
            SortColumn::Pid => sorted.sort_by(|a, b| {
                if asc {
                    a.pid.cmp(&b.pid)
                } else {
                    b.pid.cmp(&a.pid)
                }
            }),
            SortColumn::Name => sorted.sort_by(|a, b| {
                let cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                if asc {
                    cmp
                } else {
                    cmp.reverse()
                }
            }),
            SortColumn::Cpu => sorted.sort_by(|a, b| {
                let cmp = a
                    .cpu_usage
                    .partial_cmp(&b.cpu_usage)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if asc {
                    cmp
                } else {
                    cmp.reverse()
                }
            }),
            SortColumn::Memory => sorted.sort_by(|a, b| {
                let cmp = a
                    .memory_mb
                    .partial_cmp(&b.memory_mb)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if asc {
                    cmp
                } else {
                    cmp.reverse()
                }
            }),
            SortColumn::Energy => sorted.sort_by(|a, b| {
                let cmp = a
                    .energy_impact
                    .partial_cmp(&b.energy_impact)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if asc {
                    cmp
                } else {
                    cmp.reverse()
                }
            }),
        }

        let mut visible = Vec::new();
        for process in sorted {
            let pid = process.pid;
            visible.push((process.clone(), 0));

            if self.expanded_groups.contains(&pid) {
                if let Some(children) = &process.children {
                    for child in children {
                        visible.push((child.clone(), 1));
                    }
                }
            }
        }

        visible
    }

    /// Merges related processes together by their base name.
    ///
    /// This groups processes like "Chrome Helper", "Chrome Helper (GPU)", etc.
    /// into a single "Chrome (N)" entry with aggregated resource usage.
    fn merge_processes(&self, processes: Vec<ProcessInfo>) -> Vec<ProcessInfo> {
        let mut merged: HashMap<String, ProcessInfo> = HashMap::new();

        for mut process in processes {
            let original_name = process.name.clone();
            let base_name = get_base_process_name(&original_name);

            process.children = None;

            if let Some(existing) = merged.get_mut(&base_name) {
                existing.cpu_usage += process.cpu_usage;
                existing.memory_mb += process.memory_mb;
                existing.energy_impact += process.energy_impact;
                existing.disk_read_bytes += process.disk_read_bytes;
                existing.disk_write_bytes += process.disk_write_bytes;
                existing.total_cpu_time_secs += process.total_cpu_time_secs;
                existing.run_time_secs = existing.run_time_secs.max(process.run_time_secs);
                if let Some(ref mut children) = existing.children {
                    children.push(process);
                    existing.name = format!("{} ({})", base_name, children.len());
                }
            } else {
                let group = ProcessInfo {
                    pid: process.pid,
                    parent_pid: process.parent_pid,
                    name: base_name.clone(),
                    command: process.command.clone(),
                    cpu_usage: process.cpu_usage,
                    memory_mb: process.memory_mb,
                    energy_impact: process.energy_impact,
                    is_killable: process.is_killable,
                    children: Some(vec![process.clone()]),
                    disk_read_bytes: process.disk_read_bytes,
                    disk_write_bytes: process.disk_write_bytes,
                    status: process.status,
                    run_time_secs: process.run_time_secs,
                    total_cpu_time_secs: process.total_cpu_time_secs,
                };
                merged.insert(base_name, group);
            }
        }

        merged.into_values().collect()
    }

    /// Returns the currently selected process, if any.
    pub fn get_selected_process(&self) -> Option<ProcessInfo> {
        let visible = self.get_visible_processes();
        visible
            .get(self.selected_process_index)
            .map(|(p, _)| p.clone())
    }

    /// Returns a reference to the process marked for killing, if any.
    pub fn process_to_kill(&self) -> Option<&ProcessInfo> {
        self.process_to_kill.as_ref()
    }

    /// Enters selection mode, freezing the current process list.
    ///
    /// When in selection mode, the process list is frozen to prevent
    /// the selection from jumping around as processes update.
    pub(crate) fn enter_selection_mode(&mut self) {
        if !self.selection_mode {
            self.selection_mode = true;
            self.frozen_processes = Some(self.processes.processes.clone());
        }
    }

    /// Exits selection mode, unfreezing the process list.
    ///
    /// This resets the selection index and scroll offset.
    pub(crate) fn exit_selection_mode(&mut self) {
        self.selection_mode = false;
        self.frozen_processes = None;
        self.selected_process_index = 0;
        self.process_scroll_offset = 0;
    }

    /// Adjusts the scroll offset to keep the selected process visible.
    pub(crate) fn adjust_scroll(&mut self) {
        const VISIBLE_ROWS: usize = 15;

        if self.selected_process_index < self.process_scroll_offset {
            self.process_scroll_offset = self.selected_process_index;
        } else if self.selected_process_index >= self.process_scroll_offset + VISIBLE_ROWS {
            self.process_scroll_offset = self.selected_process_index - VISIBLE_ROWS + 1;
        }
    }

    /// Kills a process by PID with the specified signal.
    ///
    /// If connected to the daemon, the kill request is sent through the daemon.
    /// Otherwise, the process is killed directly.
    pub(crate) fn kill_process_impl(&self, pid: u32, signal: KillSignal) {
        if self.using_daemon_data {
            if let Ok(mut client) = DaemonClient::connect() {
                let _ = client.kill_process(pid, signal);
                return;
            }
        }
        let _ = self.processes.kill_process(pid, signal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_base_process_name_strips_helper_suffix() {
        assert_eq!(get_base_process_name("Chrome Helper"), "Chrome");
    }

    #[test]
    fn get_base_process_name_strips_helper_renderer_suffix() {
        assert_eq!(get_base_process_name("Chrome Helper (Renderer)"), "Chrome");
    }

    #[test]
    fn get_base_process_name_strips_helper_gpu_suffix() {
        assert_eq!(get_base_process_name("Chrome Helper (GPU)"), "Chrome");
    }

    #[test]
    fn get_base_process_name_strips_helper_plugin_suffix() {
        assert_eq!(get_base_process_name("Chrome Helper (Plugin)"), "Chrome");
    }

    #[test]
    fn get_base_process_name_strips_renderer_suffix() {
        assert_eq!(get_base_process_name("Safari Renderer"), "Safari");
    }

    #[test]
    fn get_base_process_name_strips_gpu_process_suffix() {
        assert_eq!(get_base_process_name("Firefox (GPU Process)"), "Firefox");
    }

    #[test]
    fn get_base_process_name_strips_web_content_suffix() {
        assert_eq!(get_base_process_name("Firefox Web Content"), "Firefox");
    }

    #[test]
    fn get_base_process_name_strips_parenthesized_suffix() {
        assert_eq!(get_base_process_name("MyApp (Worker 1)"), "MyApp");
        assert_eq!(get_base_process_name("Node (12345)"), "Node");
    }

    #[test]
    fn get_base_process_name_preserves_plain_name() {
        assert_eq!(get_base_process_name("Safari"), "Safari");
        assert_eq!(get_base_process_name("Terminal"), "Terminal");
    }

    #[test]
    fn get_base_process_name_preserves_name_with_unclosed_paren() {
        assert_eq!(get_base_process_name("App (test"), "App (test");
    }

    #[test]
    fn get_base_process_name_strips_rightmost_suffix_first() {
        assert_eq!(
            get_base_process_name("Chrome Helper (GPU) Helper"),
            "Chrome"
        );
    }
}
