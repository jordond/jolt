//! Daemon connection and data synchronization methods for App.
//!
//! This module contains methods for connecting to the daemon,
//! subscribing to real-time updates, and handling reconnection logic.

use std::time::Duration;

use color_eyre::eyre::Result;
use tracing::{debug, info};

use crate::daemon::{DaemonClient, DataSnapshot};

/// App struct placeholder - the actual App struct will be in mod.rs
/// once the refactoring is complete. For now, this impl block extends
/// the App type from the parent module.
use super::App;

impl App {
    /// Attempts to connect to the daemon and subscribe for real-time updates.
    /// If the daemon is not running, attempts to auto-start it.
    pub(crate) fn try_connect_daemon(&mut self) {
        if self.try_subscribe_to_daemon() {
            return;
        }

        if !crate::daemon::is_daemon_running() {
            debug!("Daemon not running, attempting auto-start");
            if self.auto_start_daemon() {
                std::thread::sleep(Duration::from_millis(500));
                for _ in 0..5 {
                    if self.try_subscribe_to_daemon() {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }
                debug!("Failed to subscribe after auto-start");
            }
        }
    }

    /// Attempts to subscribe to the daemon for real-time updates.
    /// Returns true if subscription was successful.
    fn try_subscribe_to_daemon(&mut self) -> bool {
        if let Ok(mut client) = DaemonClient::connect() {
            if client.subscribe().is_ok() && client.set_nonblocking(true).is_ok() {
                info!("Subscribed to daemon for real-time data");
                self.daemon_subscription = Some(client);
                self.using_daemon_data = true;
                self.daemon_connected = true;
                self.sync_daemon_broadcast_interval();
                return true;
            }
        }
        false
    }

    /// Attempts to auto-start the daemon process.
    /// Returns true if the spawn was initiated successfully.
    fn auto_start_daemon(&self) -> bool {
        let Ok(exe) = std::env::current_exe() else {
            debug!("Failed to get current exe path");
            return false;
        };

        match std::process::Command::new(&exe)
            .args(["daemon", "start"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(_) => {
                debug!("Daemon spawn initiated");
                true
            }
            Err(e) => {
                debug!("Failed to spawn daemon: {}", e);
                false
            }
        }
    }

    /// Checks if daemon data is stale (hasn't been updated in over 2 seconds).
    pub fn is_data_stale(&self) -> bool {
        if !self.using_daemon_data {
            return false;
        }
        if let Some(last_update) = self.last_daemon_update {
            last_update.elapsed() > Duration::from_secs(2)
        } else {
            false
        }
    }

    /// Checks if the app is currently attempting to reconnect to the daemon.
    pub fn is_reconnecting(&self) -> bool {
        self.using_daemon_data && self.daemon_subscription.is_none() && self.reconnect_attempts > 0
    }

    /// Reads data from the daemon subscription.
    /// Returns Ok(true) if new data was received, Ok(false) if no data,
    /// or falls back to local data collection if daemon is unavailable.
    pub(crate) fn tick_from_daemon(&mut self) -> Result<bool> {
        let mut received_data = false;
        let read_start = std::time::Instant::now();

        if let Some(ref mut client) = self.daemon_subscription {
            match client.read_update() {
                Ok(Some(snapshot)) => {
                    let read_duration = read_start.elapsed();
                    debug!(
                        read_duration_ms = read_duration.as_millis() as u64,
                        battery_percent = snapshot.battery.charge_percent,
                        battery_state = %snapshot.battery.state_label,
                        external_connected = snapshot.battery.external_connected,
                        power_watts = snapshot.power.total_power_watts,
                        process_count = snapshot.processes.len(),
                        "Received daemon snapshot"
                    );
                    self.apply_snapshot(&snapshot);
                    self.last_snapshot = Some(snapshot);
                    self.last_daemon_update = Some(std::time::Instant::now());
                    self.reconnect_attempts = 0;
                    received_data = true;
                }
                Ok(None) => {
                    let since_last = self
                        .last_daemon_update
                        .map(|t| t.elapsed().as_millis() as u64)
                        .unwrap_or(0);
                    if since_last > 1000 {
                        debug!(
                            since_last_update_ms = since_last,
                            "No daemon data received (waiting)"
                        );
                    }
                }
                Err(e) => {
                    debug!(error = %e, "Daemon connection lost");
                    self.daemon_subscription = None;
                    self.daemon_connected = false;
                    self.attempt_reconnect();
                }
            }
        }

        if !received_data {
            if let Some(last_update) = self.last_daemon_update {
                let elapsed = last_update.elapsed();
                if elapsed > Duration::from_secs(5) {
                    debug!(
                        elapsed_secs = elapsed.as_secs(),
                        "No daemon data for 5s, attempting reconnect"
                    );
                    self.daemon_subscription = None;
                    self.attempt_reconnect();
                }
            }
        }

        if !self.using_daemon_data {
            debug!("Falling back to local data collection");
            self.tick_from_local()?;
            return Ok(true);
        }

        Ok(received_data)
    }

    /// Attempts to reconnect to the daemon with exponential backoff.
    fn attempt_reconnect(&mut self) {
        const MAX_RECONNECT_ATTEMPTS: u32 = 3;
        const RECONNECT_BACKOFF_MS: u64 = 1000;

        if self.reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
            debug!("Max reconnect attempts reached, falling back to local data");
            self.using_daemon_data = false;
            self.daemon_connected = false;
            return;
        }

        let backoff_duration =
            Duration::from_millis(RECONNECT_BACKOFF_MS * (self.reconnect_attempts + 1) as u64);
        if let Some(last_attempt) = self.last_reconnect_attempt {
            if last_attempt.elapsed() < backoff_duration {
                return;
            }
        }

        self.reconnect_attempts += 1;
        self.last_reconnect_attempt = Some(std::time::Instant::now());
        debug!(
            attempt = self.reconnect_attempts,
            "Attempting daemon reconnect"
        );

        if self.try_subscribe_to_daemon() {
            self.reconnect_attempts = 0;
        }
    }

    /// Applies a data snapshot received from the daemon to update app state.
    fn apply_snapshot(&mut self, snapshot: &DataSnapshot) {
        let prev_battery_state = self.battery.state_label();
        let prev_external = self.battery.external_connected();

        self.battery.update_from_snapshot(&snapshot.battery);
        self.power.update_from_snapshot(&snapshot.power);

        if !self.selection_mode {
            self.processes
                .update_from_snapshots(snapshot.processes.clone());
        }

        let new_battery_state = self.battery.state_label();
        let new_external = self.battery.external_connected();

        if prev_battery_state != new_battery_state || prev_external != new_external {
            info!(
                prev_state = prev_battery_state,
                new_state = new_battery_state,
                prev_external,
                new_external,
                "Battery state changed"
            );
        }
    }

    /// Synchronizes the daemon's broadcast interval with the app's refresh rate.
    pub fn sync_daemon_broadcast_interval(&self) {
        if let Ok(mut client) = DaemonClient::connect() {
            let _ = client.set_broadcast_interval(self.refresh_ms);
        }
    }

    /// Refreshes the daemon status by connecting and querying its current state.
    pub(crate) fn refresh_daemon_status(&mut self) {
        if let Ok(mut client) = DaemonClient::connect() {
            self.daemon_connected = true;
            if let Ok(status) = client.get_status() {
                self.daemon_status = Some(status);
            }
        } else {
            self.daemon_connected = false;
            self.daemon_status = None;
        }
    }
}
