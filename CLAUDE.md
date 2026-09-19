# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

MonDashboard is in early development (v0.1 in progress). The workspace and both crates
build, and **every module is scaffolded but not implemented**: data structs and function
signatures are complete, bodies are `todo!()` (37 of them across 27 files). Implementation
work means filling in those bodies, not designing the types — the type layer is already the
contract.

`SPEC.md` (French) is the authoritative technical reference for the full planned
architecture; it describes v0.1 through v0.4, so always cross-check it against the v0.1
scope list below before building something.

Not yet created, despite being described in SPEC.md: `flatpak/`, `data/` (desktop file,
metainfo, hicolor icons), `LICENSE`.

## Commands

```bash
cargo build                       # build all crates
cargo build --release
cargo run -p mondashboard-gtk     # run the app (binary is named `mondashboard`)
cargo check --workspace           # fast feedback loop — preferred while filling in todo!()s

cargo test                        # all crates
cargo test -p mondashboard-core   # single crate
cargo test -p mondashboard-core <test_name>

# The `nvidia` feature is ON by default and pulls in nvml-wrapper.
# Always verify the non-NVIDIA build too — gpu/nvidia.rs has two cfg-gated bodies:
cargo check -p mondashboard-core --no-default-features
```

Rust **edition 2024** — requires a recent stable toolchain (developed on 1.95).

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

The separation is a hard rule: nothing in `mondashboard-core` may reference GTK, and the
frontend holds no collection logic. Core is consumed by direct Rust calls in the same
process — no IPC, no channels.

**Data flow:** the GTK frontend runs a polling loop via `glib::timeout_add_local` on the
main GTK thread. Each tick calls core functions (`get_cpu_stats()`, …), updates widgets,
then checks alert thresholds. No async channels or separate threads in v0.1 — core is
non-blocking, so synchronous polling suffices.

**Differential collectors:** `get_network_stats(previous)` and `get_disk_stats(previous)`
take the prior snapshot and derive per-second rates from the delta. The caller (the polling
loop) owns those snapshots and must keep them across ticks.

**`sysinfo::System` is caller-owned:** `get_cpu_stats(sys)`, `get_memory_stats(sys)` and
`get_process_stats(sys)` borrow a `&System` that the frontend creates once and refreshes
each tick — do not construct a new `System` inside a collector.

**Async in a sync API:** core depends on `zbus` + `tokio` solely for battery (UPower over
D-Bus). `get_battery_stats()` has a *synchronous* signature, so the async D-Bus call must be
driven on a runtime internally and must not block the GTK main loop for long.

**Configuration** is versioned JSON at
`~/.var/app/io.github.Mvth1s.MonDashboard/config/config.json` (XDG path under Flatpak).
Loaded at startup, saved on change. `AppConfig::version` exists for future migrations.
Note `AppConfig` deliberately already carries fields beyond v0.1 scope (`layouts`, `alerts`,
`overlay`) — keeping the schema stable now avoids a migration later. Serializing them is
fine; building UI for them is not.

### Core module pattern

Every module in `mondashboard-core/src/` follows the same convention:
- An immutable data struct (e.g. `CpuStats`), plus an `AllXxxStats { xxx: Vec<_> }` wrapper
  for the multi-entity ones (gpu, disk, process, fans)
- A pure collection function (e.g. `fn get_cpu_stats(sys: &sysinfo::System) -> CpuStats`)
- Unit tests at the bottom of the file under `#[cfg(test)]`

Optional hardware data is `Option<T>` (temperatures, VRAM on iGPUs, cycle count…) rather
than a sentinel value; widgets render "—" for `None`. Errors use
`Result<T, MonDashboardError>` (in `lib.rs`, `thiserror`) — no `unwrap()` in production code.

Only `cpu.rs`, `memory.rs` and `network.rs` currently have `#[cfg(test)]` modules; add one
when you implement any other module.

### GPU detection

`gpu::get_gpu_stats()` delegates to `multi::collect_all()`, which calls each vendor
backend's `pub(super) fn detect() -> Vec<GpuStats>` in order (nvidia, amd, amd_igpu, intel)
and concatenates. A backend that finds no hardware returns an empty vec — it never errors.
Multi-GPU (iGPU + discrete) is the expected case, not an edge case.

### GTK widget pattern

All widgets in `mondashboard-gtk/src/widgets/` are plain structs wrapping a
`container: gtk4::Box`, with the same inherent interface:
- `fn new() -> Self`
- `fn update(&self, data: &XxxStats)`

(SPEC.md calls this a "trait commun"; in the code it is a convention, not an actual Rust
trait — keep it that way unless there's a reason to change.)

Real-time graphs use `GtkDrawingArea` + Cairo with a 60-value circular buffer maintained in
the frontend. SPEC.md describes that as "60s" because it was written against a 1s tick; at
the v0.1 fixed 2s tick the same buffer spans 2 minutes.

## Flatpak

App ID: `io.github.Mvth1s.MonDashboard`
Runtime: `org.gnome.Platform` (v46+)
Manifest: `flatpak/io.github.Mvth1s.MonDashboard.yml` — **not written yet**

```bash
flatpak-builder --install --user build-dir flatpak/io.github.Mvth1s.MonDashboard.yml
flatpak run io.github.Mvth1s.MonDashboard
```

## Scope v0.1 (current target)

Build ONLY what is listed below. Do not implement anything marked ❌ — several ❌ items
already have stub files (`overlay.rs`, `settings.rs`, `layout/grid.rs`,
`process::kill_process`, `alerts.rs`); leave those as `todo!()`.

✅ IN SCOPE: All 8 widgets (CPU, GPU, RAM, Network, Disk, Process, Battery, Fans)
✅ IN SCOPE: Tray icon with left-click show/hide and right-click quit
✅ IN SCOPE: Auto dark/light theme via AdwStyleManager
✅ IN SCOPE: Fixed layout (no drag & drop)
✅ IN SCOPE: Fixed 2s refresh interval, no settings window
   (`AppConfig::refresh_interval_secs` defaults to 2; SPEC.md §4.2 says 1s — 2s wins)

❌ OUT OF SCOPE: Drag & drop, multiple layouts
❌ OUT OF SCOPE: Mini overlay
❌ OUT OF SCOPE: Notifications & alerts
❌ OUT OF SCOPE: Settings window
❌ OUT OF SCOPE: Kill process

The battery widget must hide itself entirely when `BatteryStats::present` is false
(desktops) — it is not an error state.

## UI Reference

The HTML mockup is `MonDashboard _standalone_.html` at the repo root (note the spaces in
the filename) — use it as visual reference when building GTK widgets: match the layout,
color codes and widget structure. Logos and app icons are flat in `assets/` (there is no
`assets/icons/` directory).

Color coding for metric indicators (fixed, never user-configurable, and distinct from alert
thresholds):
- Green  : load < 60%
- Orange : load 60–85%
- Red    : load > 85%

## Data sources per module

| Module       | Source                                      |
|---|---|
| cpu.rs       | `sysinfo` crate + `/sys/class/hwmon/` (temps) |
| memory.rs    | `sysinfo` crate                             |
| gpu/amd.rs   | `/sys/class/drm/` sysfs                     |
| gpu/nvidia.rs| `nvml-wrapper` crate (feature-gated)        |
| gpu/intel.rs | `/sys/class/drm/` sysfs                     |
| gpu/multi.rs | aggregates all detected GPUs                |
| network.rs   | `sysinfo` + differential snapshot for speed |
| disk.rs      | `sysinfo` + `/proc/diskstats` differential  |
| battery.rs   | UPower via D-Bus (`zbus` crate)             |
| fans.rs      | `/sys/class/hwmon/` (fan*_input labels)     |

AMD GPU data is read directly from `sysfs` — deliberately no `rocm-smi` dependency, so
users don't need it installed.

## Conventions

- Code, identifiers and code comments in **English**; user-facing docs (`README.md`,
  `SPEC.md`) are in **French**. Keep new docs consistent with that split.
- Doc comments on stub functions record their data source (e.g. `/// Sources:
  /sys/class/hwmon/*/fan*_input and fan*_label.`) — preserve them when implementing.
