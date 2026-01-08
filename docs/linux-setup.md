# Linux Setup Guide

This guide covers the setup required to run jolt on Linux systems.

## Requirements

- Linux kernel 3.13+ (for RAPL support)
- A laptop with a battery (for battery metrics)
- Intel or AMD CPU (for power metrics via RAPL)

## Permissions

### Battery Information

Battery information is read from `/sys/class/power_supply/BAT*/` which is typically world-readable. No special permissions required.

### Power Metrics (RAPL)

Power consumption metrics are read from Intel's Running Average Power Limit (RAPL) interface at `/sys/class/powercap/intel-rapl/`.

By default, these files require root access. You have three options:

#### Option 1: Run as root (not recommended)

```bash
sudo jolt
```

#### Option 2: Grant read access to RAPL (recommended)

Create a udev rule to make RAPL energy counters readable:

```bash
# Create udev rule
sudo tee /etc/udev/rules.d/99-rapl.rules << 'EOF'
# Allow reading RAPL energy counters
SUBSYSTEM=="powercap", ACTION=="add", RUN+="/bin/chmod o+r /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj /sys/class/powercap/intel-rapl/intel-rapl:0/*/energy_uj"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Or set permissions manually (needs to be done after each boot):

```bash
sudo chmod o+r /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj
sudo chmod o+r /sys/class/powercap/intel-rapl/intel-rapl:0/*/energy_uj
```

#### Option 3: Add user to a power group

Some distributions have a `power` or similar group:

```bash
# Check if a power group exists
getent group power

# If it exists, add your user
sudo usermod -aG power $USER
# Log out and back in for changes to take effect
```

### GPU Power (Optional)

GPU power consumption is read from hwmon when available:

- **Intel**: `/sys/class/drm/card*/device/hwmon/hwmon*/power1_input`
- **AMD**: `/sys/class/hwmon/hwmon*/power1_input` (amdgpu driver)
- **NVIDIA**: Not currently supported (proprietary driver uses different interface)

These are typically readable without special permissions.

## Verifying Setup

### Check battery access

```bash
cat /sys/class/power_supply/BAT0/capacity
# Should print a number 0-100
```

### Check RAPL access

```bash
cat /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj
# Should print a number (microjoules)
```

If you get "Permission denied", follow the permission setup above.

### Check AC adapter detection

```bash
cat /sys/class/power_supply/AC*/online 2>/dev/null || \
cat /sys/class/power_supply/ADP*/online 2>/dev/null
# Should print 1 (plugged in) or 0 (on battery)
```

## Troubleshooting

### "No battery found"

- Check if your system has a battery: `ls /sys/class/power_supply/`
- Desktop systems without batteries will show power metrics only

### "Power metrics showing 0"

- RAPL may not be available on older CPUs
- Check if RAPL is present: `ls /sys/class/powercap/intel-rapl/`
- AMD CPUs use the same interface but may have different paths

### "Not charging" status not detected

jolt reads the battery status directly from `/sys/class/power_supply/BAT*/status`. 
The possible values are:
- `Charging`
- `Discharging`
- `Not charging` (plugged in but not charging, e.g., battery full or charge limit)
- `Full`
- `Unknown`

If your system reports a different string, please open an issue.

## Distribution-Specific Notes

### Ubuntu/Debian

RAPL is typically available. Use the udev rule method for persistent access.

### Fedora

SELinux may block access to RAPL. You may need to create a policy exception or use `sudo`.

### Arch Linux

Works out of the box with the udev rule method.

### NixOS

Add the udev rule to your configuration:

```nix
services.udev.extraRules = ''
  SUBSYSTEM=="powercap", ACTION=="add", RUN+="${pkgs.coreutils}/bin/chmod o+r /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj /sys/class/powercap/intel-rapl/intel-rapl:0/*/energy_uj"
'';
```
