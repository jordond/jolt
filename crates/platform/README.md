# jolt-platform

Cross-platform battery and power monitoring library for jolt.

## Overview

This crate provides platform-agnostic traits (`BatteryProvider`, `PowerProvider`) with platform-specific implementations for macOS and Linux. It abstracts away the differences in how each OS exposes battery and power metrics.

## Supported Platforms

| Platform | Battery | Power | Feature Flag |
|----------|---------|-------|--------------|
| macOS    | ✅      | ✅    | `macos`      |
| Linux    | ✅      | ✅    | `linux`      |
| Windows  | ❌      | ❌    | -            |

## Feature Comparison

### Battery Metrics

| Metric | macOS | Linux | Notes |
|--------|-------|-------|-------|
| Charge percent | ✅ | ✅ | |
| Charge state | ✅ | ✅ | Charging, Discharging, Full, NotCharging, Unknown |
| Max capacity (Wh) | ✅ | ✅ | Current full charge capacity |
| Design capacity (Wh) | ✅ | ✅ | Original factory capacity |
| Health percent | ✅ | ✅ | max/design × 100 |
| Cycle count | ✅ | ✅ | May be None on some hardware |
| Voltage (mV) | ✅ | ✅ | |
| Amperage (mA) | ✅ | ✅ | Negative when discharging |
| Temperature (°C) | ✅ | ✅ | May be None on some hardware |
| Time to full | ✅ | ✅ | Estimated by OS/battery crate |
| Time to empty | ✅ | ✅ | Estimated by OS/battery crate |
| External connected | ✅ | ✅ | AC adapter detection |
| Charger wattage | ✅ | ❌ | macOS only (e.g., 96W) |
| Daily min/max SoC | ✅ | ❌ | macOS only (battery health tracking) |

### Power Metrics

| Metric | macOS | Linux | Notes |
|--------|-------|-------|-------|
| CPU power (W) | ✅ | ✅ | |
| GPU power (W) | ✅ | ✅ | |
| System power (W) | ✅ | ✅ | |
| Power mode | ✅ | ❌ | Low Power, Automatic, High Performance |
| ANE power (W) | ✅ | ❌ | Apple Neural Engine (internal only) |

### Data Sources

#### macOS

| Data | Source | Notes |
|------|--------|-------|
| Battery basics | `battery` crate | Cross-platform Rust crate |
| Battery extras | `ioreg -rn AppleSmartBattery` | Charger watts, daily SoC, amperage |
| CPU/GPU power | IOReport framework | "Energy Model" channel group |
| System power | SMC (`PSTR` key) | Total system power from SMC |
| Power mode | `pmset -g` | lowpowermode/highpowermode flags |

#### Linux

| Data | Source | Notes |
|------|--------|-------|
| Battery basics | `battery` crate | Cross-platform Rust crate |
| Battery extras | `/sys/class/power_supply/BAT*/` | Status, current_now |
| AC detection | `/sys/class/power_supply/{AC,ADP}*/online` | |
| CPU power | RAPL (`/sys/class/powercap/intel-rapl/`) | Requires permissions |
| GPU power | hwmon (`/sys/class/hwmon/*/power1_input`) | amdgpu, i915, nouveau |
| Power mode | ❌ | Not implemented |

## Platform Differences

### Charge State Detection

**macOS**: Uses `ioreg` to detect the `NotCharging` state (plugged in but not charging, e.g., charge limit reached). This is common on modern MacBooks with battery health features.

**Linux**: Reads `/sys/class/power_supply/BAT*/status` directly. The kernel reports "Not charging" when the battery is full or a charge limit is active.

### Power Measurement Accuracy

**macOS (Apple Silicon)**:
- Direct energy measurement via IOReport framework
- Per-component breakdown (CPU, GPU, ANE, DRAM, etc.)
- System power from SMC sensor
- Very accurate, updated in real-time

**macOS (Intel)**:
- Falls back to CPU usage estimation if IOReport unavailable
- Less accurate, estimates based on utilization

**Linux (RAPL)**:
- Energy counter in microjoules
- Requires calculating power from delta over time
- Only measures CPU package (may include integrated GPU)
- Discrete GPU power via hwmon (if supported)
- Requires read permissions on `/sys/class/powercap/`

### GPU Power

**macOS**: Included in IOReport "Energy Model" - covers integrated and discrete GPUs on Apple Silicon.

**Linux**: Reads from hwmon interface. Supported drivers:
- `amdgpu` - AMD GPUs
- `i915` - Intel integrated graphics
- `nouveau` - NVIDIA (open source driver)
- NVIDIA proprietary driver is **not supported** (uses different interface)

### Power Mode

**macOS**: Reads current mode from `pmset`:
- `LowPower` - Battery saver enabled
- `Automatic` - Default balanced mode
- `HighPerformance` - Maximum performance

**Linux**: Currently returns `Unknown`. Power profiles could be read from:
- `power-profiles-daemon` (GNOME)
- `/sys/firmware/acpi/platform_profile` (kernel 5.18+)
- TLP configuration

## Permissions

### macOS

No special permissions required. All APIs are accessible to normal users.

### Linux

**Battery**: World-readable, no special permissions needed.

**Power (RAPL)**: Requires read access to `/sys/class/powercap/intel-rapl/*/energy_uj`. Options:
1. Run as root (not recommended)
2. udev rule to grant read access
3. Add user to a `power` group

See [Linux Setup Guide](../../docs/linux-setup.md) for detailed instructions.

## Usage

```rust
use jolt_platform::{BatteryProvider, PowerProvider};

#[cfg(target_os = "macos")]
use jolt_platform::macos::{MacOSBattery, MacOSPower};

#[cfg(target_os = "linux")]
use jolt_platform::linux::{LinuxBattery, LinuxPower};

fn main() -> color_eyre::Result<()> {
    #[cfg(target_os = "macos")]
    let mut battery = MacOSBattery::new()?;
    
    #[cfg(target_os = "linux")]
    let mut battery = LinuxBattery::new()?;
    
    battery.refresh()?;
    
    let info = battery.info();
    println!("Charge: {:.1}%", info.charge_percent);
    println!("State: {}", info.state);
    println!("Health: {:.1}%", info.health_percent);
    
    Ok(())
}
```

## Architecture

```
jolt-platform/
├── src/
│   ├── lib.rs           # Public API, feature gates
│   ├── battery.rs       # BatteryInfo struct, BatteryProvider trait
│   ├── power.rs         # PowerInfo struct, PowerProvider trait
│   ├── types.rs         # ChargeState, PowerMode enums
│   ├── macos/
│   │   ├── mod.rs
│   │   ├── battery.rs   # MacOSBattery (battery crate + ioreg)
│   │   └── power.rs     # MacOSPower (IOReport + SMC)
│   └── linux/
│       ├── mod.rs
│       ├── battery.rs   # LinuxBattery (battery crate + sysfs)
│       └── power.rs     # LinuxPower (RAPL + hwmon)
```

## Dependencies

| Dependency | Used For | Platforms |
|------------|----------|-----------|
| `battery` | Cross-platform battery basics | All |
| `color-eyre` | Error handling | All |
| `sysinfo` | CPU usage fallback | All |
| `core-foundation` | macOS API bindings | macOS |
| `core-foundation-sys` | macOS FFI types | macOS |
| `mach2` | macOS kernel interfaces | macOS |
| `libc` | System calls | All |
