---
title: Understanding Metrics
description: What the battery and power metrics mean
---

jolt displays various metrics about your laptop's battery and power consumption. Here's what they mean and how to interpret them.

## Battery Metrics

### Charge Percentage

The current battery charge level (0-100%). This matches the value shown in your system's battery indicator.

### Battery State

| State            | Meaning                                                    |
| ---------------- | ---------------------------------------------------------- |
| **Charging**     | Connected to power, battery is charging                    |
| **Discharging**  | Running on battery power                                   |
| **Full**         | Battery is at 100% and connected to power                  |
| **Not Charging** | Connected to power but not charging (battery optimization) |

### Time Remaining

Estimated time until the battery is fully charged (when charging) or depleted (when discharging).

:::note
This estimate is based on current power consumption and may change as your workload changes.
:::

### Battery Health

The battery's maximum capacity compared to its original design capacity, shown as a percentage.

- **95-100%** — Excellent condition
- **80-95%** — Good condition
- **Below 80%** — Consider battery service

### Cycle Count

The total number of charge cycles the battery has completed. One cycle = using 100% of battery capacity (can be spread across multiple charges).

Apple considers batteries consumed after ~1000 cycles for most MacBooks.

### Charger Wattage

When connected to power, shows the charger's wattage. Useful for identifying if you're using an underpowered charger.

## Power Metrics

### Total Power (Watts)

Combined power draw of all system components. This is the primary indicator of how fast your battery will drain.

| Power Level | Typical Activity           |
| ----------- | -------------------------- |
| **2-5W**    | Idle, light tasks          |
| **5-15W**   | Web browsing, documents    |
| **15-30W**  | Development, video calls   |
| **30-50W**  | Video editing, compilation |
| **50W+**    | Heavy workloads, gaming    |

**Platform Notes:**

- Intel Macs cannot report power consumption
- Linux requires RAPL support and permissions

### CPU Power

Power consumed by the processor cores (both efficiency and performance cores on Apple Silicon).

Higher values indicate:

- More active processes
- Computationally intensive tasks
- Background indexing or updates

### GPU Power

Power consumed by the graphics processor.

Higher values when:

- External display connected
- Video playback
- Graphics-intensive applications
- GPU compute workloads (Metal on macOS, OpenGL/Vulkan on Linux)

### ANE Power (Neural Engine)

Power consumed by Apple's Neural Engine for machine learning tasks.

Active during:

- Photo analysis
- Siri/dictation
- ML-based app features
- Core ML workloads

### Power Mode

System power management mode (macOS-specific):

| Mode                 | Description                           |
| -------------------- | ------------------------------------- |
| **Low Power**        | Reduced performance to save battery   |
| **Normal**           | Balanced performance and efficiency   |
| **High Performance** | Maximum performance (when plugged in) |

**Note:** Power mode detection is currently only available on macOS. Linux users can manage power profiles through system tools like `tlp` or `power-profiles-daemon`.

## Process Energy Impact

The energy impact rating is a composite score that considers:

- CPU usage over time
- GPU usage
- Disk activity
- Network activity

### Impact Levels

| Level        | Color  | Description               |
| ------------ | ------ | ------------------------- |
| **Low**      | Green  | Minimal battery impact    |
| **Moderate** | Yellow | Normal usage              |
| **Elevated** | Orange | Higher than typical       |
| **High**     | Red    | Significant battery drain |

### Interpreting Process Data

- **Steady high impact** — The process is consistently working hard
- **Spikes** — Occasional intensive tasks (usually normal)
- **Background processes with high impact** — May indicate runaway process

## Tips for Battery Life

1. **Monitor total power** — Keep it under 10W for best battery life
2. **Check high-impact processes** — Close apps you're not using
3. **Use Low Power Mode** — Great for travel or long meetings
4. **Watch for runaway processes** — Unusually high CPU from idle apps
5. **Reduce display brightness** — Major power consumer
6. **Disconnect external displays** — Significant GPU power draw
