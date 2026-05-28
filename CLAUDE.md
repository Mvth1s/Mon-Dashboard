# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

MonDashboard is in early development (v0.1 in progress). The `mondashboard-core` and `mondashboard-gtk` crates currently contain only stubs. The root workspace `Cargo.toml` has not yet been created. `SPEC.md` is the authoritative technical reference for the full planned architecture.

## Commands

```bash
# Build all crates
cargo build

# Build release binary
cargo build --release

# Run the app
cargo run -p mondashboard-gtk

# Run tests (all crates)
cargo test

# Run tests for a single crate
cargo test -p mondashboard-core
cargo test -p mondashboard-gtk

# Run a single test
cargo test -p mondashboard-core <test_name>

# Check without building
cargo check
```

**System dependencies required:**
- GTK4 + libadwaita (`libgtk-4-dev`, `libadwaita-1-dev`)
- `lm-sensors` for temperatures and fans
- `smartmontools` for disk health (optional)

## Architecture

Two-crate Cargo workspace with strict layer separation:

```
mondashboard-core/   # Pure Rust library — system data collection, no GTK dependency
mondashboard-gtk/    # GTK4 binary — UI only, consumes core via direct Rust calls
```

**Data flow:** The GTK frontend runs a polling loop via `glib::timeout_add_local` in the main GTK thread. Each tick calls core functions (`get_cpu_stats()`, etc.), updates widgets, then checks alert thresholds. No async channels or separate threads in v0.1 — the core is non-blocking, so synchronous polling is sufficient.

**Configuration** is stored as versioned JSON at `~/.var/app/io.github.Mvth1s.MonDashboard/config/config.json` (XDG path under Flatpak). Loaded at startup, saved on change with a 500ms debounce.

### Core module pattern

Every module in `mondashboard-core/src/` follows the same convention:
- An immutable data struct (e.g., `CpuStats`)
- A pure collection function (e.g., `fn get_cpu_stats(sys: &sysinfo::System) -> CpuStats`)
- Unit tests at the bottom of the file under `#[cfg(test)]`

Errors use `Result<T, MonDashboardError>` — no `unwrap()` in production code.

### GTK widget pattern

All widgets in `mondashboard-gtk/src/widgets/` implement the same interface:
- `fn new() -> Self`
- `fn update(&self, data: &XxxStats)`

Real-time graphs use `GtkDrawingArea` + Cairo with a 60-value circular buffer (60s at 1 tick/s) maintained in the frontend.

### Key planned dependencies

| Crate | Role |
|---|---|
| `sysinfo` | CPU, RAM, processes |
| `nvml-wrapper` | NVIDIA GPU via NVML |
| `gtk4` + `libadwaita` | UI, adaptive theming |
| `zbus` | D-Bus for UPower (battery) and UDisks2 (disks) |
| `serde` + `serde_json` | Config serialization |
| `notify-rust` | System notifications |
| `log` + `env_logger` | Structured logging |

AMD GPU data is read directly from `sysfs` (`/sys/class/drm/`) with no external dependency.

## Flatpak

App ID: `io.github.Mvth1s.MonDashboard`
Runtime: `org.gnome.Platform` (v46+)
Manifest: `flatpak/io.github.Mvth1s.MonDashboard.yml`

```bash
# Build and install locally
flatpak-builder --install --user build-dir flatpak/io.github.Mvth1s.MonDashboard.yml
flatpak run io.github.Mvth1s.MonDashboard
```

## Scope v0.1 (current target)

Build ONLY what is listed below. Do not implement anything marked ❌.

✅ IN SCOPE: All 8 widgets (CPU, GPU, RAM, Network, Disk, Process, Battery, Fans)
✅ IN SCOPE: Tray icon with left-click show/hide and right-click quit
✅ IN SCOPE: Auto dark/light theme via AdwStyleManager
✅ IN SCOPE: Fixed layout (no drag & drop)
✅ IN SCOPE: Fixed 2s refresh interval (no settings window)

❌ OUT OF SCOPE: Drag & drop, multiple layouts
❌ OUT OF SCOPE: Mini overlay
❌ OUT OF SCOPE: Notifications & alerts
❌ OUT OF SCOPE: Settings window
❌ OUT OF SCOPE: Kill process

## UI Reference

The HTML mockup is at `assets/maquette.html` — use it as visual reference
when building GTK widgets. Match the layout, color codes and widget structure.
Icons are in `assets/icons/`.

Color coding for metric indicators:
- Green  : load < 60%
- Orange : load 60–85%  
- Red    : load > 85%

## Data sources per module

| Module       | Source                                      |
|---|---|
| cpu.rs       | `sysinfo` crate + `/sys/class/hwmon/` (temps) |
| memory.rs    | `sysinfo` crate                             |
| gpu/amd.rs   | `/sys/class/drm/` sysfs                     |
| gpu/nvidia.rs| `nvml-wrapper` crate                        |
| gpu/intel.rs | `/sys/class/drm/` sysfs                     |
| gpu/multi.rs | aggregates all detected GPUs                |
| network.rs   | `sysinfo` + differential snapshot for speed |
| disk.rs      | `sysinfo` + `/proc/diskstats` differential  |
| battery.rs   | UPower via D-Bus (`zbus` crate)             |
| fans.rs      | `/sys/class/hwmon/` (fan*_input labels)     |
