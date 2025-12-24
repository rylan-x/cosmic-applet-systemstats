# CLAUDE.md - Development Context

## Project Overview

A lightweight system monitoring applet for the COSMIC desktop environment, displaying CPU, memory, network, and temperature stats directly in the panel.

## Current Implementation (v0.4.0)

### Features
- ✅ Real-time CPU usage monitoring (global average across all cores)
- ✅ Memory usage tracking (used/total in GB)
- ✅ Network speed monitoring (download/upload in Mbps)
- ✅ CPU temperature sensor reading
- ✅ Auto-detects primary network interface
- ✅ Updates every 1 second automatically
- ✅ Clean horizontal panel display with compact formatting
- ✅ Modular architecture with separated monitoring components

### Display Format
```
CPU: 45% | RAM: 8.2/16.0 GB | ↓1.2 ↑0.5 Mbps | 55°C
```

### Architecture

**Modular structure:**
```
src/
├── main.rs              - Entry point (app initialization)
├── app.rs               - COSMIC Application trait implementation
└── monitors/            - System monitoring modules
    ├── mod.rs           - MonitorStats aggregator
    ├── cpu.rs           - CPU usage tracking
    ├── memory.rs        - Memory usage tracking
    ├── network.rs       - Network speed monitoring
    └── temperature.rs   - Temperature sensor reading
```

**Component Responsibilities:**

- **main.rs**: Module declarations and app launcher
- **app.rs**: COSMIC applet integration
  - `SystemStats` struct - Holds Core and MonitorStats
  - `Message::Tick` - Triggers periodic updates
  - `subscription()` - 1-second update interval
  - `update()` - Delegates to monitors.update()
  - `view()` - Renders formatted stats string

- **monitors/mod.rs**: Aggregates all monitoring components
  - `MonitorStats` struct - Contains all monitor instances
  - `new()` - Initializes all monitors
  - `update()` - Updates all monitors in sequence

- **monitors/cpu.rs**: CPU usage monitoring
  - `CpuStats` struct - Wraps sysinfo::System
  - `usage()` - Returns global CPU usage percentage

- **monitors/memory.rs**: Memory tracking
  - `MemoryStats` struct - Wraps sysinfo::System
  - `used_gb()` - Returns used memory in GB
  - `total_gb()` - Returns total memory in GB

- **monitors/network.rs**: Network speed tracking
  - `NetworkStats` struct - Tracks bytes transferred
  - Auto-detects primary interface (first non-loopback with traffic)
  - Calculates bytes/sec from delta between updates
  - `download_bps()` / `upload_bps()` - Returns bytes per second

- **monitors/temperature.rs**: CPU temperature reading
  - `TemperatureStats` struct - Wraps sysinfo::Components
  - Searches for CPU sensor (cpu/tdie/tctl/core labels)
  - `cpu_celsius()` - Returns Option<f32> temperature

### Dependencies

**Cargo.toml:**
```toml
[dependencies]
sysinfo = "0.37"

[dependencies.libcosmic]
git = "https://github.com/pop-os/libcosmic.git"
default-features = false
features = ["applet", "wayland"]

[profile.release]
lto = "thin"
strip = true
```

### Installation

```bash
# Build
cargo build --release

# Install
sudo install -Dm0755 target/release/cosmic-applet-systemstats /usr/bin/cosmic-applet-systemstats
sudo install -Dm0644 res/com.github.rylan-x.systemstats.desktop /usr/share/applications/com.github.rylan-x.systemstats.desktop

# Restart panel
killall cosmic-panel
```

**Quick rebuild and test:**
```bash
cargo build --release && sudo install -Dm0755 target/release/cosmic-applet-systemstats /usr/bin/cosmic-applet-systemstats && killall cosmic-panel
```

## Design Decisions

### Modular Architecture
- **Why**: Separation of concerns - app.rs handles UI, monitors/ handles data
- Each monitor is self-contained with its own System/Networks/Components instance
- Easy to add new monitors without touching existing code
- Clean API: `new()`, `update()`, and getter methods

### Why sysinfo?
- **Proven approach** - Used by minimon and other COSMIC applets
- Cross-platform API (Linux/BSD/macOS)
- Simple refresh/read pattern
- Handles complexity of /proc parsing internally

### Network Interface Detection
- Prioritizes non-loopback interfaces with existing traffic
- Falls back to first non-loopback interface if none have traffic
- Handles interface disappearance gracefully (sets speeds to 0)

### Temperature Sensor Matching
- Searches for common CPU sensor labels (cpu/tdie/tctl/core)
- Not cached - supports hot-plug sensors
- Returns None if no matching sensor found

### Memory Calculation
- Uses `used_memory()` from sysinfo (includes cache)
- Matches minimon and other system monitors
- Different from `free` command which shows available memory
- Note: htop/top/free all calculate "used" differently - this is normal

### No Configuration Yet
- Hardcoded 1-second update interval
- Hardcoded display format
- Minimal complexity for initial release
- Will add configuration in future versions

## Roadmap to v1.0

### v0.5.0 - Error Handling & Formatting
**Goal: Robust and polished core functionality**

Error Handling:
- [ ] Handle missing sensors gracefully with user feedback
- [ ] Log warnings for sensor detection failures
- [ ] Handle network interface changes without crashing
- [ ] Validate sysfs paths before reading
- [ ] Add error recovery for nvidia-smi failures

Formatting Improvements:
- [ ] Fix excessive decimals in display (e.g., "1.234567 GB" → "1.2 GB")
- [ ] Consistent decimal places across all stats
- [ ] Handle very large/small numbers gracefully
- [ ] Format network speeds appropriately (switch to Gbps when >1000 Mbps)

### v0.6.0 - Basic Configuration
**Goal: User customization of core features**

- [ ] Configurable update interval (1s, 2s, 5s)
- [ ] Toggle individual stats on/off
  - [ ] Hide CPU/GPU temp if no sensor
  - [ ] Hide network if no interface
  - [ ] Show/hide specific stats
- [ ] Choose display units (GB/GiB, Mbps/MB/s)
- [ ] Custom stat ordering
- [ ] Save preferences to config file

### v1.0.0 - Production Release
**Goal: Stable, feature-complete, production-ready**

Release criteria:
- [x] Core monitoring stable (CPU, RAM, network, temps)
- [x] Stats match expected values (verified against minimon)
- [x] Clean, maintainable architecture
- [ ] Error handling complete (v0.5.0)
- [ ] Number formatting polished (v0.5.0)
- [ ] Basic configuration working (v0.6.0)
- [ ] No known bugs
- [ ] Documentation complete

Version 1.0 signifies:
- API/display format stability (no breaking changes)
- Core features complete and well-tested
- Safe for daily use
- Ready for wider distribution

### Post-1.0 Features (v1.x)
**Goal: Enhanced functionality without breaking stability**

Visualization (v1.1+):
- [ ] Hover/click popup for detailed stats
- [ ] Per-core CPU usage breakdown
- [ ] Historical graphs (sparklines)
- [ ] Color thresholds (red at >80% CPU)

Advanced Monitoring (v1.2+):
- [ ] Disk I/O monitoring (read/write speeds)
- [ ] Process viewer popup (top processes)
- [ ] Per-process CPU/memory breakdown

Developer Experience (v1.3+):
- [ ] Justfile for streamlined building
- [ ] Automated tests
- [ ] CI/CD pipeline

## Implementation Patterns

### Adding a New Monitor

1. Create `src/monitors/newstat.rs`:
```rust
use sysinfo::System; // or other sysinfo type

pub struct NewStat {
    system: System,
    cached_value: f32,
}

impl NewStat {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system,
            cached_value: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.system.refresh_all(); // or specific refresh
        self.cached_value = /* calculate value */;
    }

    pub fn value(&self) -> f32 {
        self.cached_value
    }
}
```

2. Add to `src/monitors/mod.rs`:
```rust
pub mod newstat;

pub struct MonitorStats {
    // ... existing monitors
    pub newstat: newstat::NewStat,
}

impl MonitorStats {
    pub fn new() -> Self {
        Self {
            // ... existing
            newstat: newstat::NewStat::new(),
        }
    }

    pub fn update(&mut self) {
        // ... existing
        self.newstat.update();
    }
}
```

3. Use in `src/app.rs` view():
```rust
stats_text.push_str(&format!(" | New: {:.1}", self.monitors.newstat.value()));
```

### COSMIC Applet Basics

**Entry point:**
```rust
fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<SystemStats>(())
}
```

**Core Application trait:**
```rust
impl cosmic::Application for SystemStats {
    const APP_ID: &'static str = "com.github.rylan-x.systemstats";

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // Initialize state
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        // Handle messages
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Render UI
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Periodic updates
        time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }
}
```

### Panel Text Rendering

**Key requirements:**
- Use `Wrapping::None` to prevent vertical wrapping
- Use `Row::from_vec()` for horizontal layout
- Wrap in `autosize::autosize()` with appropriate limits
- Add padding for spacing
- Align vertically with `Alignment::Center`

**Pattern:**
```rust
let elements = vec![
    text(stats_text)
        .wrapping(Wrapping::None)
        .into()
];

let content = Row::from_vec(elements)
    .padding([0, 8])
    .align_y(Alignment::Center);

let limits = Limits::NONE
    .max_width(400.0)
    .min_height(1.0)
    .max_height(128.0);

autosize::autosize(content, cosmic::widget::Id::unique())
    .limits(limits)
    .into()
```

## Known Issues

- None currently - all monitoring features working as expected
- Temperature may return None on systems without CPU sensors

## Lessons Learned

### Modularization Benefits
- Cleaner separation between UI (app.rs) and data (monitors/)
- Each component can maintain its own sysinfo instance
- Easier to test individual monitors
- Future refactoring is simpler

### sysinfo Refresh Patterns
- Each component type needs specific refresh calls:
  - `refresh_cpu_usage()` for CPU
  - `refresh_memory()` for memory
  - `refresh(false)` for Networks (false = don't refresh interface list)
  - `refresh(false)` for Components
- Initial refresh with `new_with_refreshed_list()` or `new_all()` + `refresh_all()`
- Global `refresh_all()` works but is less efficient

### Network Speed Calculation
- Track total bytes (monotonically increasing counter)
- Calculate delta between updates (current - previous)
- Use `saturating_sub()` to handle counter wraparound
- Convert to desired units in view layer (bytes -> Mbps)

### Temperature Sensor Detection
- Sensor labels vary by platform (CPU/tdie/tctl/Core 0/etc.)
- Use case-insensitive matching
- Match multiple common patterns
- Return Option to handle missing sensors gracefully

## References

- [COSMIC Toolkit Docs](https://pop-os.github.io/libcosmic-book/)
- [Panel Applets Guide](https://pop-os.github.io/libcosmic-book/panel-applets.html)
- [minimon source](https://github.com/cosmic-utils/minimon-applet) - Reference implementation
- [sysinfo docs](https://docs.rs/sysinfo/) - System monitoring API

## Development Workflow

**Running from terminal (see debug output):**
```bash
cosmic-applet-systemstats
```

**Check applet registration:**
```bash
ls -la /usr/share/applications/com.github.rylan-x.systemstats.desktop
```

**Common issues:**
- Text not centered? Verify `.align_y(Alignment::Center)` and proper padding
- Applet not appearing? Check desktop file exists and has correct permissions (644)
- Stats not updating? Verify subscription() returns time::every()
- Temperature showing None? Check sensor labels with `sensors` command
