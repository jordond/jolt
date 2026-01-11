pub use jolt_protocol::{
    BatterySnapshot, BatteryState, ChargeSession, ChargingState, CycleSummary, DaemonRequest,
    DaemonResponse, DaemonStatus, DailyCycle, DailyStat, DailyTopProcess, DataSnapshot,
    ForecastSnapshot, ForecastSource, HourlyStat, KillProcessResult, KillSignal, PowerMode,
    PowerSnapshot, ProcessSnapshot, ProcessState, Sample, SessionType, SystemSnapshot,
    SystemStatsSnapshot, MAX_SUBSCRIBERS, MIN_SUPPORTED_VERSION, PROTOCOL_VERSION,
};

use crate::data;

impl From<&data::HourlyStat> for HourlyStat {
    fn from(s: &data::HourlyStat) -> Self {
        Self {
            hour_start: s.hour_start,
            avg_power: s.avg_power,
            max_power: s.max_power,
            min_power: s.min_power,
            avg_battery: s.avg_battery,
            battery_delta: s.battery_delta,
            total_samples: s.total_samples,
        }
    }
}

impl From<&data::DailyStat> for DailyStat {
    fn from(s: &data::DailyStat) -> Self {
        Self {
            date: s.date.clone(),
            avg_power: s.avg_power,
            max_power: s.max_power,
            total_energy_wh: s.total_energy_wh,
            screen_on_hours: s.screen_on_hours,
            charging_hours: s.charging_hours,
            battery_cycles: s.battery_cycles,
        }
    }
}

impl From<&data::DailyTopProcess> for DailyTopProcess {
    fn from(p: &data::DailyTopProcess) -> Self {
        Self {
            date: p.date.clone(),
            process_name: p.process_name.clone(),
            total_impact: p.total_impact,
            avg_cpu: p.avg_cpu,
            avg_memory_mb: p.avg_memory_mb,
            sample_count: p.sample_count,
            avg_power: p.avg_power,
            total_energy_wh: p.total_energy_wh,
        }
    }
}

impl From<&data::Sample> for Sample {
    fn from(s: &data::Sample) -> Self {
        Self {
            timestamp: s.timestamp,
            battery_percent: s.battery_percent,
            power_watts: s.power_watts,
            cpu_power: s.cpu_power,
            gpu_power: s.gpu_power,
            charging_state: s.charging_state.into(),
        }
    }
}

impl From<data::ChargingState> for ChargingState {
    fn from(s: data::ChargingState) -> Self {
        match s {
            data::ChargingState::Discharging => ChargingState::Discharging,
            data::ChargingState::Charging => ChargingState::Charging,
            data::ChargingState::Full => ChargingState::Full,
            data::ChargingState::Unknown => ChargingState::Unknown,
        }
    }
}

impl From<data::SessionType> for SessionType {
    fn from(s: data::SessionType) -> Self {
        match s {
            data::SessionType::Charge => SessionType::Charge,
            data::SessionType::Discharge => SessionType::Discharge,
        }
    }
}

impl From<&data::ChargeSession> for ChargeSession {
    fn from(s: &data::ChargeSession) -> Self {
        Self {
            start_time: s.start_time,
            end_time: s.end_time,
            start_percent: s.start_percent,
            end_percent: s.end_percent,
            energy_wh: s.energy_wh,
            charger_watts: s.charger_watts,
            avg_power_watts: s.avg_power_watts,
            session_type: s.session_type.into(),
            is_complete: s.is_complete,
        }
    }
}

impl From<&data::DailyCycle> for DailyCycle {
    fn from(c: &data::DailyCycle) -> Self {
        Self {
            date: c.date.clone(),
            charge_sessions: c.charge_sessions,
            discharge_sessions: c.discharge_sessions,
            total_charging_mins: c.total_charging_mins,
            total_discharge_mins: c.total_discharge_mins,
            deepest_discharge_percent: c.deepest_discharge_percent,
            energy_charged_wh: c.energy_charged_wh,
            energy_discharged_wh: c.energy_discharged_wh,
            partial_cycles: c.partial_cycles,
            macos_cycle_count: c.macos_cycle_count,
            avg_temperature_c: c.avg_temperature_c,
            time_at_high_soc_mins: c.time_at_high_soc_mins,
        }
    }
}

impl From<&data::SystemInfo> for SystemSnapshot {
    fn from(s: &data::SystemInfo) -> Self {
        Self {
            chip: s.chip.clone(),
            os_version: s.os_version.clone(),
            p_cores: s.p_cores,
            e_cores: s.e_cores,
        }
    }
}

impl From<&data::SystemStatsData> for SystemStatsSnapshot {
    fn from(s: &data::SystemStatsData) -> Self {
        Self {
            cpu_usage_percent: s.cpu_usage_percent(),
            load_one: s.load_one(),
            load_five: s.load_five(),
            load_fifteen: s.load_fifteen(),
            memory_used_bytes: s.memory_used_bytes(),
            memory_total_bytes: s.memory_total_bytes(),
            uptime_secs: s.uptime_secs(),
            is_warmed_up: s.is_warmed_up(),
        }
    }
}

impl From<&data::ForecastData> for ForecastSnapshot {
    fn from(f: &data::ForecastData) -> Self {
        Self {
            duration_secs: f.duration_secs(),
            avg_power_watts: f.avg_power_watts(),
            sample_count: f.sample_count(),
            source: f.source().into(),
        }
    }
}

impl From<data::ForecastSource> for ForecastSource {
    fn from(s: data::ForecastSource) -> Self {
        match s {
            data::ForecastSource::Daemon => ForecastSource::Daemon,
            data::ForecastSource::Session => ForecastSource::Session,
            data::ForecastSource::None => ForecastSource::None,
        }
    }
}

impl From<ForecastSource> for data::ForecastSource {
    fn from(s: ForecastSource) -> Self {
        match s {
            ForecastSource::Daemon => data::ForecastSource::Daemon,
            ForecastSource::Session => data::ForecastSource::Session,
            ForecastSource::None => data::ForecastSource::None,
        }
    }
}

impl From<HourlyStat> for data::HourlyStat {
    fn from(s: HourlyStat) -> Self {
        Self {
            id: None,
            hour_start: s.hour_start,
            avg_power: s.avg_power,
            max_power: s.max_power,
            min_power: s.min_power,
            avg_battery: s.avg_battery,
            battery_delta: s.battery_delta,
            total_samples: s.total_samples,
        }
    }
}

impl From<DailyStat> for data::DailyStat {
    fn from(s: DailyStat) -> Self {
        Self {
            id: None,
            date: s.date,
            avg_power: s.avg_power,
            max_power: s.max_power,
            total_energy_wh: s.total_energy_wh,
            screen_on_hours: s.screen_on_hours,
            charging_hours: s.charging_hours,
            battery_cycles: s.battery_cycles,
        }
    }
}

impl From<DailyTopProcess> for data::DailyTopProcess {
    fn from(p: DailyTopProcess) -> Self {
        Self {
            id: None,
            date: p.date,
            process_name: p.process_name,
            total_impact: p.total_impact,
            avg_cpu: p.avg_cpu,
            avg_memory_mb: p.avg_memory_mb,
            sample_count: p.sample_count,
            avg_power: p.avg_power,
            total_energy_wh: p.total_energy_wh,
        }
    }
}

impl From<Sample> for data::Sample {
    fn from(s: Sample) -> Self {
        Self {
            id: None,
            timestamp: s.timestamp,
            battery_percent: s.battery_percent,
            power_watts: s.power_watts,
            cpu_power: s.cpu_power,
            gpu_power: s.gpu_power,
            charging_state: s.charging_state.into(),
        }
    }
}

impl From<ChargingState> for data::ChargingState {
    fn from(s: ChargingState) -> Self {
        match s {
            ChargingState::Discharging => data::ChargingState::Discharging,
            ChargingState::Charging => data::ChargingState::Charging,
            ChargingState::Full => data::ChargingState::Full,
            ChargingState::Unknown => data::ChargingState::Unknown,
        }
    }
}

impl From<SessionType> for data::SessionType {
    fn from(s: SessionType) -> Self {
        match s {
            SessionType::Charge => data::SessionType::Charge,
            SessionType::Discharge => data::SessionType::Discharge,
        }
    }
}

impl From<ChargeSession> for data::ChargeSession {
    fn from(s: ChargeSession) -> Self {
        Self {
            id: None,
            start_time: s.start_time,
            end_time: s.end_time,
            start_percent: s.start_percent,
            end_percent: s.end_percent,
            energy_wh: s.energy_wh,
            charger_watts: s.charger_watts,
            avg_power_watts: s.avg_power_watts,
            session_type: s.session_type.into(),
            is_complete: s.is_complete,
        }
    }
}

impl From<DailyCycle> for data::DailyCycle {
    fn from(c: DailyCycle) -> Self {
        Self {
            id: None,
            date: c.date,
            charge_sessions: c.charge_sessions,
            discharge_sessions: c.discharge_sessions,
            total_charging_mins: c.total_charging_mins,
            total_discharge_mins: c.total_discharge_mins,
            deepest_discharge_percent: c.deepest_discharge_percent,
            energy_charged_wh: c.energy_charged_wh,
            energy_discharged_wh: c.energy_discharged_wh,
            partial_cycles: c.partial_cycles,
            macos_cycle_count: c.macos_cycle_count,
            avg_temperature_c: c.avg_temperature_c,
            time_at_high_soc_mins: c.time_at_high_soc_mins,
        }
    }
}
