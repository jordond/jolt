---
title: JSON Output
description: Using jolt's pipe mode for scripting and automation
---

jolt's pipe mode outputs metrics as JSON, perfect for scripting, monitoring, and integration with other tools.

## Basic Usage

```bash
jolt pipe
```

This outputs continuous JSON samples to stdout.

## Options

### Sample Count

Limit the number of samples:

```bash
# Single sample
jolt pipe --samples 1

# 10 samples then exit
jolt pipe --samples 10
```

### Interval

Control time between samples:

```bash
# Every 500ms
jolt pipe --interval 500

# Every 5 seconds
jolt pipe --interval 5000
```

### Compact Mode

One JSON object per line (JSONL format):

```bash
jolt pipe --compact
```

This is easier to parse line-by-line in scripts.

## Output Format

### Standard Output

```json
{
  "timestamp": "2024-01-15T10:30:00.123Z",
  "battery": {
    "percentage": 85,
    "state": "discharging",
    "time_remaining_minutes": 240,
    "health": 92,
    "cycle_count": 127,
    "temperature_celsius": 32.5
  },
  "power": {
    "total_watts": 12.5,
    "cpu_watts": 8.2,
    "gpu_watts": 3.1,
    "ane_watts": 0.0,
    "mode": "normal"
  },
  "system": {
    "model": "MacBook Pro",
    "chip": "Apple M2 Pro"
  }
}
```

### Compact Output

```json
{"timestamp":"2024-01-15T10:30:00.123Z","battery":{"percentage":85,"state":"discharging"},"power":{"total_watts":12.5}}
```

### With Process Data

```bash
jolt pipe --include-processes
```

Adds:

```json
{
  "processes": [
    {
      "name": "Safari",
      "pid": 1234,
      "cpu_percent": 15.2,
      "energy_impact": "elevated",
      "parent_pid": 1
    }
  ]
}
```

## Scripting Examples

### Single Value Extraction

```bash
# Get current battery percentage
jolt pipe --samples 1 | jq '.battery.percentage'

# Get total power draw
jolt pipe --samples 1 | jq '.power.total_watts'
```

### Battery Monitor Script

```bash
#!/bin/bash
# Alert when battery drops below threshold

THRESHOLD=20

while true; do
  LEVEL=$(jolt pipe --samples 1 | jq '.battery.percentage')
  
  if [ "$LEVEL" -lt "$THRESHOLD" ]; then
    osascript -e "display notification \"Battery at ${LEVEL}%\" with title \"Low Battery\""
  fi
  
  sleep 300
done
```

### Power Logging

```bash
# Log power usage every minute
jolt pipe --interval 60000 --compact >> ~/power_log.jsonl
```

### Menu Bar Integration

Use with tools like [xbar](https://xbarapp.com/):

```bash
#!/bin/bash
# xbar plugin for jolt

DATA=$(jolt pipe --samples 1)
BATTERY=$(echo $DATA | jq -r '.battery.percentage')
WATTS=$(echo $DATA | jq -r '.power.total_watts')

echo "⚡ ${BATTERY}% | ${WATTS}W"
echo "---"
echo "Battery: ${BATTERY}%"
echo "Power: ${WATTS}W"
```

### CSV Export

```bash
# Convert JSON stream to CSV
echo "timestamp,battery,watts" > power.csv
jolt pipe --samples 60 --interval 1000 --compact | \
  jq -r '[.timestamp, .battery.percentage, .power.total_watts] | @csv' >> power.csv
```

## Integration Examples

### Prometheus/Grafana

Create a simple exporter:

```python
from flask import Flask
from prometheus_client import Gauge, generate_latest
import subprocess
import json

app = Flask(__name__)

battery_gauge = Gauge('jolt_battery_percent', 'Battery percentage')
power_gauge = Gauge('jolt_power_watts', 'Power consumption in watts')

@app.route('/metrics')
def metrics():
    data = json.loads(subprocess.check_output(['jolt', 'pipe', '--samples', '1']))
    battery_gauge.set(data['battery']['percentage'])
    power_gauge.set(data['power']['total_watts'])
    return generate_latest()
```

### Home Assistant

```yaml
sensor:
  - platform: command_line
    name: Mac Battery
    command: "jolt pipe --samples 1 | jq '.battery.percentage'"
    unit_of_measurement: "%"
    scan_interval: 60
```

### Slack/Discord Alerts

```bash
#!/bin/bash
# Send alert to Slack when power is high

THRESHOLD=40
WEBHOOK_URL="https://hooks.slack.com/..."

WATTS=$(jolt pipe --samples 1 | jq '.power.total_watts')

if (( $(echo "$WATTS > $THRESHOLD" | bc -l) )); then
  curl -X POST -H 'Content-type: application/json' \
    --data "{\"text\":\"⚠️ High power usage: ${WATTS}W\"}" \
    $WEBHOOK_URL
fi
```

## Error Handling

JSON output includes errors when they occur:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Cannot access power metrics"
  }
}
```

Handle in scripts:

```bash
DATA=$(jolt pipe --samples 1)
if echo "$DATA" | jq -e '.error' > /dev/null; then
  echo "Error: $(echo $DATA | jq -r '.error.message')"
  exit 1
fi
```
