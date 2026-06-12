# System Stats Applet for COSMIC Desktop

A lightweight system monitoring applet for the COSMIC desktop environment.

![System Stats Screenshot](res/screenshots/systemstats.png)

## Features

- CPU usage monitoring
- GPU usage monitoring (NVIDIA, AMD)
- Memory usage display
- GPU VRAM usage display
- Network upload/download speeds
- CPU temperature
- GPU temperature
- Customizable label names and colors
- Multi-GPU support with switching capability

## Dependencies

Building requires `just` and `libxkbcommon-dev`

## Installation

### Build and Install

```bash
just build-release
sudo just install
```

### From Package

**.deb package:**

```bash
sudo dpkg -i cosmic-applet-systemstats_1.0.0-1_amd64.deb
```

**.flatpak package:**

```bash
flatpak install --user cosmic-applet-systemstats.flatpak
```

<!-- ### From Flathub
```bash
flatpak install flathub io.github.rylan_x.cosmic-applet-systemstats
```
-->

<!-- ### From COSMIC Store
Find "System Stats" in the COSMIC Store under COSMIC Applets.
-->

## Configuration

The applet can be configured via `~/.config/systemstats/config.toml`. A default configuration file is automatically created.

### Configuration Options

```toml
# Refresh interval in milliseconds (default: 1000 = 1 second)
refresh_interval_ms = 1000

[monitors]
# Toggle individual monitors on/off (default: all true)
cpu_usage = true
cpu_temperature = true
gpu_temperature = true
gpu_usage = true
gpu_vram = true
memory = true
network = true

[thresholds.cpu]
# CPU usage thresholds (percentage)
# Values < low_max = Low (blue)
# Values between low_max and high_min = Medium (orange)
# Values >= high_min = High (red)
low_max = 40.0
high_min = 75.0

[thresholds.gpu]
# GPU usage thresholds (percentage)
low_max = 50.0
high_min = 85.0

[thresholds.memory]
# Memory usage thresholds (percentage)
low_max = 50.0
high_min = 80.0

[thresholds.temperature]
# Temperature thresholds (Celsius)
low_max = 60.0
high_min = 80.0

[labels.cpu]
# CPU usage label name and color (hex without #)
name = "CPU: "
color = "00B4D8"

[labels.cpu_temp]
# CPU temperature label name and color
name = "TEMP: "
color = "FFD700"

[labels.gpu_temp]
# GPU temperature label name and color
name = "GPU: "
color = "00D4AA"

[labels.gpu_usage]
# GPU usage label name and color
name = "GPU: "
color = "00D4AA"

[labels.gpu_vram]
# GPU VRAM label name and color
name = "VRAM: "
color = "FF6B9D"

[labels.ram]
# Memory label name and color
name = "RAM: "
color = "9B5DE5"

[labels.network_download]
# Network download arrow color
color = "00B4D8"

[labels.network_upload]
# Network upload arrow color
color = "FB8500"
```

After editing the config file, restart the applet/panel for changes to take effect.

## Color-Coded Status Indicators

The applet uses color coding to provide at-a-glance status information:

| Status | Color  | CPU Usage | GPU Usage | Memory Usage | Temperature |
| ------ | ------ | --------- | --------- | ------------ | ----------- |
| Low    | Blue   | < 40%     | < 50%     | < 50%        | < 60°C      |
| Medium | Orange | 40-75%    | 50-85%    | 50-80%       | 60-80°C     |
| High   | Red    | > 75%     | > 85%     | > 80%        | > 80°C      |

**Note:** Values are only colored when in warning state (Medium/High). Normal state values use the default text color.

Network speeds are displayed with:

- **Download** arrow in configurable color (default: blue `#00B4D8`)
- **Upload** arrow in configurable color (default: orange `#FB8500`)

### Default Label Colors

| Label       | Name     | Color     | Preview |
| ----------- | -------- | --------- | ------- |
| `cpu`       | `CPU: `  | `#00B4D8` | Blue    |
| `cpu_temp`  | `TEMP: ` | `#FFD700` | Yellow  |
| `gpu_temp`  | `GPU: `  | `#00D4AA` | Teal    |
| `gpu_usage` | `GPU: `  | `#00D4AA` | Teal    |
| `gpu_vram`  | `VRAM: ` | `#FF6B9D` | Pink    |
| `ram`       | `RAM: `  | `#9B5DE5` | Purple  |

### Customizing Colors

All label names and colors can be customized in the `[labels]` section of the config file:

- `cpu` - CPU usage label
- `cpu_temp` - CPU temperature label
- `gpu_temp` - GPU temperature label
- `gpu_usage` - GPU usage label
- `gpu_vram` - GPU VRAM label
- `ram` - Memory label
- `network_download` - Download arrow color
- `network_upload` - Upload arrow color

Colors can be specified with or without the `#` prefix (e.g., `"00B4D8"` or `"#00B4D8"`).

Thresholds can be customized in the configuration file to match your preferences.

## GPU Support

The applet supports GPU monitoring for the following vendors:

| Vendor | Usage % | VRAM | Temperature | Requirements                          |
| ------ | ------- | ---- | ----------- | ------------------------------------- |
| NVIDIA | ✓       | ✓    | ✓           | `nvidia-smi` (part of NVIDIA drivers) |
| AMD    | ✓       | ✓    | ✓           | `amdgpu` kernel driver                |
| Intel  | Limited | -    | -           | Limited support via sysfs             |

### Multi-GPU Support

When multiple GPUs are detected, the applet displays a GPU indicator (e.g., "GPU1:", "GPU2:") to show which GPU is currently being monitored. The GPU switching feature allows cycling through detected GPUs (UI integration planned for future releases).
