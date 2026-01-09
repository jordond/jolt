# AGENTS.md - jolt Data Layer

**Generated:** 2026-01-08 | **Commit:** c962424

> See `/cli/AGENTS.md` for module context, `/AGENTS.md` for code style.

Data collection, storage, and aggregation. Platform-agnostic interfaces over macOS/Linux providers.

## Files

| File | Purpose | Key Types |
|------|---------|-----------|
| mod.rs | Re-exports all public types | - |
| battery.rs | Battery metrics via `jolt_platform` | `BatteryData`, `ChargeState` |
| power.rs | Power draw via `jolt_platform` | `PowerData`, `PowerMode` |
| processes.rs | Process energy via `sysinfo` | `ProcessData`, `ProcessInfo` |
| history.rs | In-memory ring buffer for graphs | `HistoryData`, `HistoryMetric` |
| history_store.rs | SQLite persistence (1131 lines) | `HistoryStore`, `Sample`, `HourlyStat`, `DailyStat` |
| aggregator.rs | Hourly/daily rollup calculations | `Aggregator` |
| recorder.rs | Daemon sampling orchestration | `Recorder` |
| session_tracker.rs | Charge/discharge session tracking | `SessionTracker`, `SessionEvent` |
| forecast.rs | Battery time estimates | `ForecastData` |
| system.rs | System info (hostname, model) | `SystemInfo` |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Providers                               │
│  jolt_platform::macos  │  jolt_platform::linux  │  sysinfo      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Data Structs                                │
│   BatteryData          PowerData          ProcessData           │
│   └─ new() + refresh() └─ new() + refresh() └─ new() + refresh()│
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Persistence                                 │
│   Recorder ──▶ HistoryStore (SQLite WAL) ◀── Aggregator         │
│                     │                                           │
│             Sample, HourlyStat, DailyStat, DailyCycle           │
└─────────────────────────────────────────────────────────────────┘
```

## Patterns

### Refresh Cycle
All data structs follow:
```rust
impl DataStruct {
    pub fn new() -> Result<Self> { /* init provider */ }
    pub fn refresh(&mut self) -> Result<()> { /* update from provider */ }
    pub fn some_metric(&self) -> T { /* cached value */ }
}
```

### Power Smoothing
`PowerData` uses 5-sample moving average (VecDeque) to reduce noise:
```rust
const SMOOTHING_SAMPLE_COUNT: usize = 5;
const MIN_WARMUP_SAMPLES: usize = 3;
```

### SQLite WAL Mode
`HistoryStore` uses WAL for concurrent daemon writes + TUI reads:
- Schema version: 3 (see `CURRENT_SCHEMA_VERSION`)
- Location: `~/.local/share/jolt/history.db`
- Migrations handled in `ensure_schema()`

## Key Types (history_store.rs)

| Type | Purpose |
|------|---------|
| `Sample` | Raw per-tick battery/power snapshot |
| `HourlyStat` | Aggregated hourly averages/extremes |
| `DailyStat` | Daily energy totals, screen time |
| `DailyTopProcess` | Process energy ranking per day |
| `ChargeSession` | Charge/discharge event with Wh consumed |
| `DailyCycle` | Daily cycle count calculation |
| `BatteryHealthSnapshot` | Daily health/capacity snapshot |
| `CycleSnapshot` | macOS cycle count tracking |

## Recorder Flow (daemon)

```
Recorder::maybe_record()
  │
  ├─▶ Check sample_interval_secs elapsed
  ├─▶ Create Sample from BatteryData + PowerData
  ├─▶ store.insert_sample()
  ├─▶ session_tracker.track() → detect charge/discharge transitions
  ├─▶ record_top_processes() if enabled
  └─▶ maybe_record_health_snapshot() (once per day)
```

## Anti-Patterns

- **NEVER** access `BatteryData`/`PowerData` fields directly from ui/app
- **NEVER** call `store.insert_*()` outside Recorder (daemon owns writes)
- Use `unwrap_or_default()` for optional metrics, not `unwrap()`
