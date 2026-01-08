//! Persistent storage for historical battery and power metrics.
//!
//! Uses SQLite with WAL mode for efficient concurrent access between
//! the daemon (writer) and TUI (reader).

use std::path::PathBuf;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::config::data_dir;

const CURRENT_SCHEMA_VERSION: i32 = 3;
const DATABASE_NAME: &str = "history.db";

/// Charging state for a sample
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ChargingState {
    Discharging = 0,
    Charging = 1,
    Full = 2,
    Unknown = 3,
}

impl From<i32> for ChargingState {
    fn from(value: i32) -> Self {
        match value {
            0 => ChargingState::Discharging,
            1 => ChargingState::Charging,
            2 => ChargingState::Full,
            _ => ChargingState::Unknown,
        }
    }
}

/// A single sample of battery and power metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub id: Option<i64>,
    pub timestamp: i64,
    pub battery_percent: f32,
    pub power_watts: f32,
    pub cpu_power: f32,
    pub gpu_power: f32,
    pub charging_state: ChargingState,
}

/// Hourly aggregated statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStat {
    pub id: Option<i64>,
    pub hour_start: i64,
    pub avg_power: f32,
    pub max_power: f32,
    pub min_power: f32,
    pub avg_battery: f32,
    pub battery_delta: f32,
    pub total_samples: i32,
}

/// Daily aggregated statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStat {
    pub id: Option<i64>,
    pub date: String,
    pub avg_power: f32,
    pub max_power: f32,
    pub total_energy_wh: f32,
    pub screen_on_hours: f32,
    pub charging_hours: f32,
    pub battery_cycles: f32,
}

/// Daily top process aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTopProcess {
    pub id: Option<i64>,
    pub date: String,
    pub process_name: String,
    pub total_impact: f32,
    pub avg_cpu: f32,
    pub avg_memory_mb: f32,
    pub sample_count: i32,
    pub avg_power: f32,       // Average power consumption in watts
    pub total_energy_wh: f32, // Total energy consumed in Wh
}

/// Battery health snapshot (stored daily)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryHealthSnapshot {
    pub id: Option<i64>,
    pub date: String,
    pub health_percent: f32,
    pub cycle_count: Option<i32>,
    pub max_capacity_wh: f32,
    pub design_capacity_wh: f32,
}

/// Session type for charge/discharge tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum SessionType {
    Charge = 0,
    Discharge = 1,
}

impl From<i32> for SessionType {
    fn from(value: i32) -> Self {
        match value {
            0 => SessionType::Charge,
            _ => SessionType::Discharge,
        }
    }
}

/// Individual charge or discharge session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargeSession {
    pub id: Option<i64>,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub start_percent: f32,
    pub end_percent: Option<f32>,
    pub energy_wh: Option<f32>,
    pub charger_watts: Option<u32>,
    pub avg_power_watts: Option<f32>,
    pub session_type: SessionType,
    pub is_complete: bool,
}

impl ChargeSession {
    /// Create a new charge session starting now
    pub fn new_charge(start_time: i64, start_percent: f32, charger_watts: Option<u32>) -> Self {
        Self {
            id: None,
            start_time,
            end_time: None,
            start_percent,
            end_percent: None,
            energy_wh: None,
            charger_watts,
            avg_power_watts: None,
            session_type: SessionType::Charge,
            is_complete: false,
        }
    }

    /// Create a new discharge session starting now
    pub fn new_discharge(start_time: i64, start_percent: f32) -> Self {
        Self {
            id: None,
            start_time,
            end_time: None,
            start_percent,
            end_percent: None,
            energy_wh: None,
            charger_watts: None,
            avg_power_watts: None,
            session_type: SessionType::Discharge,
            is_complete: false,
        }
    }

    /// Duration of the session in seconds (if complete)
    pub fn duration_secs(&self) -> Option<i64> {
        self.end_time.map(|end| end - self.start_time)
    }

    /// Percentage change during session (positive for charge, negative for discharge)
    pub fn percent_delta(&self) -> Option<f32> {
        self.end_percent.map(|end| end - self.start_percent)
    }
}

/// Daily cycle summary (aggregated from sessions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCycle {
    pub id: Option<i64>,
    pub date: String,
    pub charge_sessions: i32,
    pub discharge_sessions: i32,
    pub total_charging_mins: i32,
    pub total_discharge_mins: i32,
    pub deepest_discharge_percent: Option<f32>,
    pub energy_charged_wh: f32,
    pub energy_discharged_wh: f32,
    pub partial_cycles: f32,
    pub macos_cycle_count: Option<i32>,
    pub avg_temperature_c: Option<f32>,
    pub time_at_high_soc_mins: i32,
}

impl Default for DailyCycle {
    fn default() -> Self {
        Self {
            id: None,
            date: String::new(),
            charge_sessions: 0,
            discharge_sessions: 0,
            total_charging_mins: 0,
            total_discharge_mins: 0,
            deepest_discharge_percent: None,
            energy_charged_wh: 0.0,
            energy_discharged_wh: 0.0,
            partial_cycles: 0.0,
            macos_cycle_count: None,
            avg_temperature_c: None,
            time_at_high_soc_mins: 0,
        }
    }
}

/// Long-term cycle tracking snapshot (stored daily)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleSnapshot {
    pub id: Option<i64>,
    pub date: String,
    pub macos_cycle_count: i32,
    pub calculated_partial_cycles: f32,
    pub battery_health_percent: f32,
}

/// Errors that can occur during history storage operations
#[derive(Debug, thiserror::Error)]
pub enum HistoryStoreError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, HistoryStoreError>;

/// History storage backed by SQLite
pub struct HistoryStore {
    conn: Connection,
    path: PathBuf,
}

impl HistoryStore {
    /// Open or create the history database
    pub fn open() -> Result<Self> {
        let dir = data_dir();
        std::fs::create_dir_all(&dir)?;

        let path = dir.join(DATABASE_NAME);
        let conn = Connection::open(&path)?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;",
        )?;

        let mut store = Self { conn, path };
        store.initialize_schema()?;

        Ok(store)
    }

    /// Get the database file size in bytes
    pub fn size_bytes(&self) -> Result<u64> {
        if self.path.to_string_lossy() == ":memory:" {
            return Ok(0);
        }
        Ok(std::fs::metadata(&self.path)?.len())
    }

    /// Initialize or migrate the database schema
    fn initialize_schema(&mut self) -> Result<()> {
        let version = self.get_schema_version()?;

        if version == 0 {
            self.create_initial_schema()?;
        } else if version < CURRENT_SCHEMA_VERSION {
            self.run_migrations(version)?;
        }

        Ok(())
    }

    /// Get the current schema version (0 if not initialized)
    fn get_schema_version(&self) -> Result<i32> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(0);
        }

        let version: i32 = self
            .conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
                row.get(0)
            })
            .optional()?
            .unwrap_or(0);

        Ok(version)
    }

    /// Create the initial database schema
    fn create_initial_schema(&mut self) -> Result<()> {
        let tx = self.conn.transaction()?;

        tx.execute_batch(
            r#"
            -- Schema version tracking
            CREATE TABLE schema_version (
                version INTEGER NOT NULL
            );

            -- Raw data points (sampled every interval when recording)
            CREATE TABLE samples (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                battery_percent REAL NOT NULL,
                power_watts REAL NOT NULL,
                cpu_power REAL NOT NULL,
                gpu_power REAL NOT NULL,
                charging_state INTEGER NOT NULL DEFAULT 0
            );

            -- Hourly aggregates for efficient long-term queries
            CREATE TABLE hourly_stats (
                id INTEGER PRIMARY KEY,
                hour_start INTEGER NOT NULL UNIQUE,
                avg_power REAL NOT NULL,
                max_power REAL NOT NULL,
                min_power REAL NOT NULL,
                avg_battery REAL NOT NULL,
                battery_delta REAL NOT NULL,
                total_samples INTEGER NOT NULL
            );

            -- Daily summaries
            CREATE TABLE daily_stats (
                id INTEGER PRIMARY KEY,
                date TEXT NOT NULL UNIQUE,
                avg_power REAL NOT NULL,
                max_power REAL NOT NULL,
                total_energy_wh REAL NOT NULL,
                screen_on_hours REAL NOT NULL DEFAULT 0,
                charging_hours REAL NOT NULL DEFAULT 0,
                battery_cycles REAL NOT NULL DEFAULT 0
            );

            -- Top power consumers per day
            CREATE TABLE daily_top_processes (
                id INTEGER PRIMARY KEY,
                date TEXT NOT NULL,
                process_name TEXT NOT NULL,
                total_impact REAL NOT NULL,
                avg_cpu REAL NOT NULL,
                avg_memory_mb REAL NOT NULL,
                sample_count INTEGER NOT NULL,
                avg_power REAL NOT NULL DEFAULT 0.0,
                total_energy_wh REAL NOT NULL DEFAULT 0.0,
                UNIQUE(date, process_name)
            );

            -- Battery health snapshots (daily)
            CREATE TABLE battery_health (
                id INTEGER PRIMARY KEY,
                date TEXT NOT NULL UNIQUE,
                health_percent REAL NOT NULL,
                cycle_count INTEGER,
                max_capacity_wh REAL NOT NULL,
                design_capacity_wh REAL NOT NULL
            );

            -- Charge/discharge session tracking
            CREATE TABLE charge_sessions (
                id INTEGER PRIMARY KEY,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                start_percent REAL NOT NULL,
                end_percent REAL,
                energy_wh REAL,
                charger_watts INTEGER,
                avg_power_watts REAL,
                session_type INTEGER NOT NULL,
                is_complete INTEGER NOT NULL DEFAULT 0
            );

            -- Daily cycle summary (aggregated from sessions)
            CREATE TABLE daily_cycles (
                id INTEGER PRIMARY KEY,
                date TEXT NOT NULL UNIQUE,
                charge_sessions INTEGER NOT NULL DEFAULT 0,
                discharge_sessions INTEGER NOT NULL DEFAULT 0,
                total_charging_mins INTEGER NOT NULL DEFAULT 0,
                total_discharge_mins INTEGER NOT NULL DEFAULT 0,
                deepest_discharge_percent REAL,
                energy_charged_wh REAL NOT NULL DEFAULT 0,
                energy_discharged_wh REAL NOT NULL DEFAULT 0,
                partial_cycles REAL NOT NULL DEFAULT 0,
                macos_cycle_count INTEGER,
                avg_temperature_c REAL,
                time_at_high_soc_mins INTEGER NOT NULL DEFAULT 0
            );

            -- Long-term cycle tracking snapshots
            CREATE TABLE cycle_snapshots (
                id INTEGER PRIMARY KEY,
                date TEXT NOT NULL UNIQUE,
                macos_cycle_count INTEGER NOT NULL,
                calculated_partial_cycles REAL NOT NULL,
                battery_health_percent REAL NOT NULL
            );

            -- Indexes for efficient queries
            CREATE INDEX idx_samples_timestamp ON samples(timestamp);
            CREATE INDEX idx_hourly_hour ON hourly_stats(hour_start);
            CREATE INDEX idx_daily_date ON daily_stats(date);
            CREATE INDEX idx_daily_processes_date ON daily_top_processes(date);
            CREATE INDEX idx_battery_health_date ON battery_health(date);
            CREATE INDEX idx_charge_sessions_start ON charge_sessions(start_time);
            CREATE INDEX idx_charge_sessions_type ON charge_sessions(session_type);
            CREATE INDEX idx_daily_cycles_date ON daily_cycles(date);
            CREATE INDEX idx_cycle_snapshots_date ON cycle_snapshots(date);
            "#,
        )?;

        tx.execute(
            "INSERT INTO schema_version (version) VALUES (?)",
            [CURRENT_SCHEMA_VERSION],
        )?;

        tx.commit()?;
        Ok(())
    }

    fn run_migrations(&mut self, from_version: i32) -> Result<()> {
        let tx = self.conn.transaction()?;

        if from_version < 2 {
            tx.execute_batch(
                r#"
                ALTER TABLE daily_top_processes ADD COLUMN avg_power REAL NOT NULL DEFAULT 0.0;
                ALTER TABLE daily_top_processes ADD COLUMN total_energy_wh REAL NOT NULL DEFAULT 0.0;
                "#,
            )?;
        }

        if from_version < 3 {
            tx.execute_batch(
                r#"
                -- Charge/discharge session tracking
                CREATE TABLE charge_sessions (
                    id INTEGER PRIMARY KEY,
                    start_time INTEGER NOT NULL,
                    end_time INTEGER,
                    start_percent REAL NOT NULL,
                    end_percent REAL,
                    energy_wh REAL,
                    charger_watts INTEGER,
                    avg_power_watts REAL,
                    session_type INTEGER NOT NULL,
                    is_complete INTEGER NOT NULL DEFAULT 0
                );

                -- Daily cycle summary
                CREATE TABLE daily_cycles (
                    id INTEGER PRIMARY KEY,
                    date TEXT NOT NULL UNIQUE,
                    charge_sessions INTEGER NOT NULL DEFAULT 0,
                    discharge_sessions INTEGER NOT NULL DEFAULT 0,
                    total_charging_mins INTEGER NOT NULL DEFAULT 0,
                    total_discharge_mins INTEGER NOT NULL DEFAULT 0,
                    deepest_discharge_percent REAL,
                    energy_charged_wh REAL NOT NULL DEFAULT 0,
                    energy_discharged_wh REAL NOT NULL DEFAULT 0,
                    partial_cycles REAL NOT NULL DEFAULT 0,
                    macos_cycle_count INTEGER,
                    avg_temperature_c REAL,
                    time_at_high_soc_mins INTEGER NOT NULL DEFAULT 0
                );

                -- Long-term cycle tracking snapshots
                CREATE TABLE cycle_snapshots (
                    id INTEGER PRIMARY KEY,
                    date TEXT NOT NULL UNIQUE,
                    macos_cycle_count INTEGER NOT NULL,
                    calculated_partial_cycles REAL NOT NULL,
                    battery_health_percent REAL NOT NULL
                );

                -- Indexes for new tables
                CREATE INDEX idx_charge_sessions_start ON charge_sessions(start_time);
                CREATE INDEX idx_charge_sessions_type ON charge_sessions(session_type);
                CREATE INDEX idx_daily_cycles_date ON daily_cycles(date);
                CREATE INDEX idx_cycle_snapshots_date ON cycle_snapshots(date);
                "#,
            )?;
        }

        tx.execute(
            "UPDATE schema_version SET version = ?",
            [CURRENT_SCHEMA_VERSION],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn insert_sample(&self, sample: &Sample) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO samples (timestamp, battery_percent, power_watts, cpu_power, gpu_power, charging_state)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                sample.timestamp,
                sample.battery_percent,
                sample.power_watts,
                sample.cpu_power,
                sample.gpu_power,
                sample.charging_state as i32,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get samples in a time range
    pub fn get_samples(&self, from: i64, to: i64) -> Result<Vec<Sample>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, battery_percent, power_watts, cpu_power, gpu_power, charging_state
             FROM samples
             WHERE timestamp >= ? AND timestamp <= ?
             ORDER BY timestamp ASC",
        )?;

        let samples = stmt
            .query_map(params![from, to], |row| {
                Ok(Sample {
                    id: Some(row.get(0)?),
                    timestamp: row.get(1)?,
                    battery_percent: row.get(2)?,
                    power_watts: row.get(3)?,
                    cpu_power: row.get(4)?,
                    gpu_power: row.get(5)?,
                    charging_state: ChargingState::from(row.get::<_, i32>(6)?),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(samples)
    }

    /// Delete samples older than the given timestamp
    pub fn delete_samples_before(&self, before: i64) -> Result<usize> {
        let deleted = self
            .conn
            .execute("DELETE FROM samples WHERE timestamp < ?", [before])?;
        Ok(deleted)
    }

    pub fn upsert_hourly_stat(&self, stat: &HourlyStat) -> Result<()> {
        self.conn.execute(
            "INSERT INTO hourly_stats (hour_start, avg_power, max_power, min_power, avg_battery, battery_delta, total_samples)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(hour_start) DO UPDATE SET
                avg_power = excluded.avg_power,
                max_power = excluded.max_power,
                min_power = excluded.min_power,
                avg_battery = excluded.avg_battery,
                battery_delta = excluded.battery_delta,
                total_samples = excluded.total_samples",
            params![
                stat.hour_start,
                stat.avg_power,
                stat.max_power,
                stat.min_power,
                stat.avg_battery,
                stat.battery_delta,
                stat.total_samples,
            ],
        )?;
        Ok(())
    }

    /// Get hourly stats in a time range
    pub fn get_hourly_stats(&self, from: i64, to: i64) -> Result<Vec<HourlyStat>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hour_start, avg_power, max_power, min_power, avg_battery, battery_delta, total_samples
             FROM hourly_stats
             WHERE hour_start >= ? AND hour_start <= ?
             ORDER BY hour_start ASC",
        )?;

        let stats = stmt
            .query_map(params![from, to], |row| {
                Ok(HourlyStat {
                    id: Some(row.get(0)?),
                    hour_start: row.get(1)?,
                    avg_power: row.get(2)?,
                    max_power: row.get(3)?,
                    min_power: row.get(4)?,
                    avg_battery: row.get(5)?,
                    battery_delta: row.get(6)?,
                    total_samples: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(stats)
    }

    /// Delete hourly stats older than the given timestamp
    pub fn delete_hourly_stats_before(&self, before: i64) -> Result<usize> {
        let deleted = self
            .conn
            .execute("DELETE FROM hourly_stats WHERE hour_start < ?", [before])?;
        Ok(deleted)
    }

    pub fn upsert_daily_stat(&self, stat: &DailyStat) -> Result<()> {
        self.conn.execute(
            "INSERT INTO daily_stats (date, avg_power, max_power, total_energy_wh, screen_on_hours, charging_hours, battery_cycles)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(date) DO UPDATE SET
                avg_power = excluded.avg_power,
                max_power = excluded.max_power,
                total_energy_wh = excluded.total_energy_wh,
                screen_on_hours = excluded.screen_on_hours,
                charging_hours = excluded.charging_hours,
                battery_cycles = excluded.battery_cycles",
            params![
                stat.date,
                stat.avg_power,
                stat.max_power,
                stat.total_energy_wh,
                stat.screen_on_hours,
                stat.charging_hours,
                stat.battery_cycles,
            ],
        )?;
        Ok(())
    }

    /// Get daily stats for a date range (max 365 entries for display)
    pub fn get_daily_stats(&self, from: &str, to: &str) -> Result<Vec<DailyStat>> {
        self.get_daily_stats_limited(from, to, 365)
    }

    /// Get daily stats with custom limit
    pub fn get_daily_stats_limited(
        &self,
        from: &str,
        to: &str,
        limit: usize,
    ) -> Result<Vec<DailyStat>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, date, avg_power, max_power, total_energy_wh, screen_on_hours, charging_hours, battery_cycles
             FROM daily_stats
             WHERE date >= ? AND date <= ?
             ORDER BY date DESC
             LIMIT ?",
        )?;

        let mut stats: Vec<DailyStat> = stmt
            .query_map(params![from, to, limit as i64], |row| {
                Ok(DailyStat {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    avg_power: row.get(2)?,
                    max_power: row.get(3)?,
                    total_energy_wh: row.get(4)?,
                    screen_on_hours: row.get(5)?,
                    charging_hours: row.get(6)?,
                    battery_cycles: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        stats.reverse();
        Ok(stats)
    }

    /// Get a single day's stats
    pub fn get_daily_stat(&self, date: &str) -> Result<Option<DailyStat>> {
        let stat = self
            .conn
            .query_row(
                "SELECT id, date, avg_power, max_power, total_energy_wh, screen_on_hours, charging_hours, battery_cycles
                 FROM daily_stats WHERE date = ?",
                [date],
                |row| {
                    Ok(DailyStat {
                        id: Some(row.get(0)?),
                        date: row.get(1)?,
                        avg_power: row.get(2)?,
                        max_power: row.get(3)?,
                        total_energy_wh: row.get(4)?,
                        screen_on_hours: row.get(5)?,
                        charging_hours: row.get(6)?,
                        battery_cycles: row.get(7)?,
                    })
                },
            )
            .optional()?;

        Ok(stat)
    }

    /// Delete daily stats older than the given date
    pub fn delete_daily_stats_before(&self, before: &str) -> Result<usize> {
        let deleted = self
            .conn
            .execute("DELETE FROM daily_stats WHERE date < ?", [before])?;
        Ok(deleted)
    }

    pub fn upsert_daily_process(&self, process: &DailyTopProcess) -> Result<()> {
        self.conn.execute(
            "INSERT INTO daily_top_processes (date, process_name, total_impact, avg_cpu, avg_memory_mb, sample_count, avg_power, total_energy_wh)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(date, process_name) DO UPDATE SET
                total_impact = daily_top_processes.total_impact + excluded.total_impact,
                avg_cpu = (daily_top_processes.avg_cpu * daily_top_processes.sample_count + excluded.avg_cpu * excluded.sample_count) 
                          / (daily_top_processes.sample_count + excluded.sample_count),
                avg_memory_mb = (daily_top_processes.avg_memory_mb * daily_top_processes.sample_count + excluded.avg_memory_mb * excluded.sample_count)
                               / (daily_top_processes.sample_count + excluded.sample_count),
                avg_power = (daily_top_processes.avg_power * daily_top_processes.sample_count + excluded.avg_power * excluded.sample_count)
                           / (daily_top_processes.sample_count + excluded.sample_count),
                total_energy_wh = daily_top_processes.total_energy_wh + excluded.total_energy_wh,
                sample_count = daily_top_processes.sample_count + excluded.sample_count",
            params![
                process.date,
                process.process_name,
                process.total_impact,
                process.avg_cpu,
                process.avg_memory_mb,
                process.sample_count,
                process.avg_power,
                process.total_energy_wh,
            ],
        )?;
        Ok(())
    }

    pub fn get_top_processes_range(
        &self,
        from: &str,
        to: &str,
        limit: usize,
    ) -> Result<Vec<DailyTopProcess>> {
        let mut stmt = self.conn.prepare(
            "SELECT NULL as id, ? as date, process_name, 
                    SUM(total_impact) as total_impact,
                    SUM(avg_cpu * sample_count) / SUM(sample_count) as avg_cpu,
                    SUM(avg_memory_mb * sample_count) / SUM(sample_count) as avg_memory_mb,
                    SUM(sample_count) as sample_count,
                    SUM(avg_power * sample_count) / SUM(sample_count) as avg_power,
                    SUM(total_energy_wh) as total_energy_wh
             FROM daily_top_processes
             WHERE date >= ? AND date <= ?
             GROUP BY process_name
             ORDER BY total_energy_wh DESC
             LIMIT ?",
        )?;

        let processes = stmt
            .query_map(params![from, from, to, limit as i64], |row| {
                Ok(DailyTopProcess {
                    id: None,
                    date: row.get(1)?,
                    process_name: row.get(2)?,
                    total_impact: row.get(3)?,
                    avg_cpu: row.get(4)?,
                    avg_memory_mb: row.get(5)?,
                    sample_count: row.get(6)?,
                    avg_power: row.get(7)?,
                    total_energy_wh: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(processes)
    }

    /// Delete process entries older than the given date
    pub fn delete_daily_processes_before(&self, before: &str) -> Result<usize> {
        let deleted = self
            .conn
            .execute("DELETE FROM daily_top_processes WHERE date < ?", [before])?;
        Ok(deleted)
    }

    pub fn upsert_battery_health(&self, snapshot: &BatteryHealthSnapshot) -> Result<()> {
        self.conn.execute(
            "INSERT INTO battery_health (date, health_percent, cycle_count, max_capacity_wh, design_capacity_wh)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(date) DO UPDATE SET
                health_percent = excluded.health_percent,
                cycle_count = excluded.cycle_count,
                max_capacity_wh = excluded.max_capacity_wh,
                design_capacity_wh = excluded.design_capacity_wh",
            params![
                snapshot.date,
                snapshot.health_percent,
                snapshot.cycle_count,
                snapshot.max_capacity_wh,
                snapshot.design_capacity_wh,
            ],
        )?;
        Ok(())
    }

    pub fn insert_charge_session(&self, session: &ChargeSession) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO charge_sessions (start_time, end_time, start_percent, end_percent, energy_wh, charger_watts, avg_power_watts, session_type, is_complete)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                session.start_time,
                session.end_time,
                session.start_percent,
                session.end_percent,
                session.energy_wh,
                session.charger_watts,
                session.avg_power_watts,
                session.session_type as i32,
                session.is_complete as i32,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_charge_session(&self, session: &ChargeSession) -> Result<()> {
        let id = session.id.ok_or_else(|| {
            HistoryStoreError::Database(rusqlite::Error::InvalidParameterName(
                "Session must have an id to update".to_string(),
            ))
        })?;

        self.conn.execute(
            "UPDATE charge_sessions SET
                end_time = ?,
                end_percent = ?,
                energy_wh = ?,
                avg_power_watts = ?,
                is_complete = ?
             WHERE id = ?",
            params![
                session.end_time,
                session.end_percent,
                session.energy_wh,
                session.avg_power_watts,
                session.is_complete as i32,
                id,
            ],
        )?;
        Ok(())
    }

    pub fn get_charge_sessions(
        &self,
        from: i64,
        to: i64,
        session_type: Option<SessionType>,
    ) -> Result<Vec<ChargeSession>> {
        let sql = match session_type {
            Some(_) => {
                "SELECT id, start_time, end_time, start_percent, end_percent, energy_wh, charger_watts, avg_power_watts, session_type, is_complete
                 FROM charge_sessions
                 WHERE start_time >= ? AND start_time <= ? AND session_type = ?
                 ORDER BY start_time DESC"
            }
            None => {
                "SELECT id, start_time, end_time, start_percent, end_percent, energy_wh, charger_watts, avg_power_watts, session_type, is_complete
                 FROM charge_sessions
                 WHERE start_time >= ? AND start_time <= ?
                 ORDER BY start_time DESC"
            }
        };

        let mut stmt = self.conn.prepare(sql)?;

        let sessions: Vec<ChargeSession> = match session_type {
            Some(st) => stmt
                .query_map(params![from, to, st as i32], Self::map_charge_session)?
                .collect::<std::result::Result<Vec<_>, _>>()?,
            None => stmt
                .query_map(params![from, to], Self::map_charge_session)?
                .collect::<std::result::Result<Vec<_>, _>>()?,
        };

        Ok(sessions)
    }

    pub fn get_incomplete_session(&self) -> Result<Option<ChargeSession>> {
        let session = self
            .conn
            .query_row(
                "SELECT id, start_time, end_time, start_percent, end_percent, energy_wh, charger_watts, avg_power_watts, session_type, is_complete
                 FROM charge_sessions
                 WHERE is_complete = 0
                 ORDER BY start_time DESC
                 LIMIT 1",
                [],
                Self::map_charge_session,
            )
            .optional()?;

        Ok(session)
    }

    fn map_charge_session(row: &rusqlite::Row) -> rusqlite::Result<ChargeSession> {
        Ok(ChargeSession {
            id: Some(row.get(0)?),
            start_time: row.get(1)?,
            end_time: row.get(2)?,
            start_percent: row.get(3)?,
            end_percent: row.get(4)?,
            energy_wh: row.get(5)?,
            charger_watts: row.get(6)?,
            avg_power_watts: row.get(7)?,
            session_type: SessionType::from(row.get::<_, i32>(8)?),
            is_complete: row.get::<_, i32>(9)? != 0,
        })
    }

    pub fn delete_charge_sessions_before(&self, before: i64) -> Result<usize> {
        let deleted = self
            .conn
            .execute("DELETE FROM charge_sessions WHERE start_time < ?", [before])?;
        Ok(deleted)
    }

    pub fn upsert_daily_cycle(&self, cycle: &DailyCycle) -> Result<()> {
        self.conn.execute(
            "INSERT INTO daily_cycles (date, charge_sessions, discharge_sessions, total_charging_mins, total_discharge_mins, deepest_discharge_percent, energy_charged_wh, energy_discharged_wh, partial_cycles, macos_cycle_count, avg_temperature_c, time_at_high_soc_mins)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(date) DO UPDATE SET
                charge_sessions = excluded.charge_sessions,
                discharge_sessions = excluded.discharge_sessions,
                total_charging_mins = excluded.total_charging_mins,
                total_discharge_mins = excluded.total_discharge_mins,
                deepest_discharge_percent = excluded.deepest_discharge_percent,
                energy_charged_wh = excluded.energy_charged_wh,
                energy_discharged_wh = excluded.energy_discharged_wh,
                partial_cycles = excluded.partial_cycles,
                macos_cycle_count = excluded.macos_cycle_count,
                avg_temperature_c = excluded.avg_temperature_c,
                time_at_high_soc_mins = excluded.time_at_high_soc_mins",
            params![
                cycle.date,
                cycle.charge_sessions,
                cycle.discharge_sessions,
                cycle.total_charging_mins,
                cycle.total_discharge_mins,
                cycle.deepest_discharge_percent,
                cycle.energy_charged_wh,
                cycle.energy_discharged_wh,
                cycle.partial_cycles,
                cycle.macos_cycle_count,
                cycle.avg_temperature_c,
                cycle.time_at_high_soc_mins,
            ],
        )?;
        Ok(())
    }

    pub fn get_daily_cycles(&self, from: &str, to: &str) -> Result<Vec<DailyCycle>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, date, charge_sessions, discharge_sessions, total_charging_mins, total_discharge_mins, deepest_discharge_percent, energy_charged_wh, energy_discharged_wh, partial_cycles, macos_cycle_count, avg_temperature_c, time_at_high_soc_mins
             FROM daily_cycles
             WHERE date >= ? AND date <= ?
             ORDER BY date DESC",
        )?;

        let cycles = stmt
            .query_map(params![from, to], |row| {
                Ok(DailyCycle {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    charge_sessions: row.get(2)?,
                    discharge_sessions: row.get(3)?,
                    total_charging_mins: row.get(4)?,
                    total_discharge_mins: row.get(5)?,
                    deepest_discharge_percent: row.get(6)?,
                    energy_charged_wh: row.get(7)?,
                    energy_discharged_wh: row.get(8)?,
                    partial_cycles: row.get(9)?,
                    macos_cycle_count: row.get(10)?,
                    avg_temperature_c: row.get(11)?,
                    time_at_high_soc_mins: row.get(12)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(cycles)
    }

    pub fn get_daily_cycle(&self, date: &str) -> Result<Option<DailyCycle>> {
        let cycle = self
            .conn
            .query_row(
                "SELECT id, date, charge_sessions, discharge_sessions, total_charging_mins, total_discharge_mins, deepest_discharge_percent, energy_charged_wh, energy_discharged_wh, partial_cycles, macos_cycle_count, avg_temperature_c, time_at_high_soc_mins
                 FROM daily_cycles WHERE date = ?",
                [date],
                |row| {
                    Ok(DailyCycle {
                        id: Some(row.get(0)?),
                        date: row.get(1)?,
                        charge_sessions: row.get(2)?,
                        discharge_sessions: row.get(3)?,
                        total_charging_mins: row.get(4)?,
                        total_discharge_mins: row.get(5)?,
                        deepest_discharge_percent: row.get(6)?,
                        energy_charged_wh: row.get(7)?,
                        energy_discharged_wh: row.get(8)?,
                        partial_cycles: row.get(9)?,
                        macos_cycle_count: row.get(10)?,
                        avg_temperature_c: row.get(11)?,
                        time_at_high_soc_mins: row.get(12)?,
                    })
                },
            )
            .optional()?;

        Ok(cycle)
    }

    pub fn delete_daily_cycles_before(&self, before: &str) -> Result<usize> {
        let deleted = self
            .conn
            .execute("DELETE FROM daily_cycles WHERE date < ?", [before])?;
        Ok(deleted)
    }

    pub fn upsert_cycle_snapshot(&self, snapshot: &CycleSnapshot) -> Result<()> {
        self.conn.execute(
            "INSERT INTO cycle_snapshots (date, macos_cycle_count, calculated_partial_cycles, battery_health_percent)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(date) DO UPDATE SET
                macos_cycle_count = excluded.macos_cycle_count,
                calculated_partial_cycles = excluded.calculated_partial_cycles,
                battery_health_percent = excluded.battery_health_percent",
            params![
                snapshot.date,
                snapshot.macos_cycle_count,
                snapshot.calculated_partial_cycles,
                snapshot.battery_health_percent,
            ],
        )?;
        Ok(())
    }

    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute("VACUUM", [])?;
        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let sample_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM samples", [], |row| row.get(0))?;

        let hourly_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM hourly_stats", [], |row| row.get(0))?;

        let daily_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM daily_stats", [], |row| row.get(0))?;

        let oldest_sample: Option<i64> = self
            .conn
            .query_row("SELECT MIN(timestamp) FROM samples", [], |row| row.get(0))
            .optional()?
            .flatten();

        let newest_sample: Option<i64> = self
            .conn
            .query_row("SELECT MAX(timestamp) FROM samples", [], |row| row.get(0))
            .optional()?
            .flatten();

        let size_bytes = self.size_bytes()?;

        Ok(DatabaseStats {
            sample_count,
            hourly_count,
            daily_count,
            oldest_sample,
            newest_sample,
            size_bytes,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub sample_count: i64,
    pub hourly_count: i64,
    pub daily_count: i64,
    pub oldest_sample: Option<i64>,
    pub newest_sample: Option<i64>,
    pub size_bytes: u64,
}

impl DatabaseStats {
    /// Format size as human-readable string
    pub fn size_formatted(&self) -> String {
        let bytes = self.size_bytes as f64;
        if bytes < 1024.0 {
            format!("{} B", self.size_bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.1} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Helper to get a date string for N days ago
pub fn days_ago_date_string(days: u32) -> String {
    let date = Utc::now() - chrono::Duration::days(days as i64);
    date.format("%Y-%m-%d").to_string()
}
