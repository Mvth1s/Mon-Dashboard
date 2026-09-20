# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

v0.1 is **implemented and runs**: both crates build, the GTK window displays live data,
and `cargo test` covers the collectors. `SPEC.md` (French) remains the reference for the
full v0.1–v0.4 architecture — it describes far more than v0.1, so always check the scope
list below before building something from it.

Exactly four `todo!()` remain, and all four are deliberately out of scope (see Scope):
`alerts::check_alerts`, `process::kill_process`, `overlay::build`, `settings::build`.

## Commands

```bash
cargo build                       # build all crates
cargo run -p mondashboard-gtk     # run the app (binary is named `mondashboard`)
cargo check --workspace           # fast feedback loop

cargo test                        # all crates
cargo test -p mondashboard-core   # single crate
cargo test -p mondashboard-core <test_name>

# Prints every piece of hardware detected on the current machine.
# The fastest way to tell a detection bug from a display bug.
cargo run -p mondashboard-core --example diagnostic

# The `nvidia` feature is ON by default and pulls in nvml-wrapper.
# gpu/nvidia.rs has two cfg-gated bodies, so always check the other one:
cargo check -p mondashboard-core --no-default-features
```

**The pipeline** (`.github/workflows/ci.yml`, and what to run before merging a branch):

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p mondashboard-core --no-default-features -- -D warnings
cargo test --workspace
```

Rust **edition 2024** — requires a recent stable toolchain (developed on 1.95; the
`if let … && let …` chains in `layout/persistence.rs` need 1.88+).

**System dependencies:** GTK4 + **libadwaita 1.6 or newer** (`libgtk-4-dev`,
`libadwaita-1-dev`). 1.6 is required for `StyleManager::accent_color_rgba()`; Ubuntu 24.04
ships 1.5, which is why CI builds inside a `fedora:latest` container rather than on the
Ubuntu runner image. `lm-sensors` and `smartmontools` are optional — their absence degrades
gracefully.

## The rule that shapes this codebase: detect, never assume

Most of the collector code exists to avoid guessing about hardware. Respect this when
touching anything under `mondashboard-core`:

- **No sysfs index is ever hardcoded.** `hwmonN`, `cardN`, `BATn` and disk numbering change
  between boots and do not correlate with each other (on the dev machine, `hwmon0` is the
  sensor for `nvme1`, not `nvme0`). Enumerate, then identify by name or by resolving the
  real device path with `sysfs::canonical`.
- **An installed driver is not a present device.** `nvidia-smi` can exist on a machine with
  an AMD GPU. `Nvml::init()` failing *is* the detection result: return an empty vec.
- **Absent hardware is `None` or an empty list, never an error and never zero.** A driver
  that does not publish GPU utilization (Intel i915/xe) must yield `None`, which the UI
  renders as "—". Rendering it as 0 % would be a lie.
- **A `type=Battery` device is not necessarily the computer's battery.** Wireless mice and
  keyboards declare themselves as batteries with `scope=Device`; `battery.rs` filters on
  that, and UPower's `PowerSupply` property serves the same purpose.
- **External commands are localized.** `ping` prints `temps=` under a French locale, so it
  is invoked with `LC_ALL=C`. Any new command parsing must do the same.

Documented heuristics are acceptable where the kernel exposes no clean signal — the AMD
integrated-vs-discrete split uses a VRAM threshold (`drm::IGPU_VRAM_LIMIT_BYTES`) and says
so in a comment. Silent assumptions are not.

## Architecture

```
mondashboard-core/   # Pure Rust library — system data collection, no GTK dependency
mondashboard-gtk/    # GTK4 binary — UI only, consumes core via direct Rust calls
```

The separation is a hard rule: nothing in core references GTK, and the frontend holds no
collection logic. Same process, direct Rust calls, no IPC.

**Data flow:** `polling.rs` runs `glib::timeout_add_local` on the GTK main thread. Each
tick refreshes `sysinfo`, calls the core collectors, and pushes the result into the
widgets. `Collecteur` owns the state that must survive between ticks.

**Nothing slow runs on that thread.** Three sources are too slow for a tick and each keeps
its own background thread plus a cache, started once from `app.rs`:
`network::start_ping_monitor`, `disk::start_smart_monitor`, `battery::start_monitor`.
The getters return the cached value instantly. Anything new that shells out or talks
D-Bus belongs in that pattern, not in a tick.

**Differential collectors:** `get_network_stats(previous)` and `get_disk_stats(previous)`
take the prior snapshot, which carries both the cumulative counters and a `sampled_at`
`Instant`. Rates are computed over the real elapsed time, and a counter that went backwards
(interface restarted) yields 0 rather than a nonsense spike.

**`sysinfo::System` is caller-owned** and passed by reference to `get_cpu_stats`,
`get_memory_stats` and `get_process_stats` — never construct one inside a collector.

**Battery has two interchangeable backends**, selected by `AppConfig::battery_source`:
`Sysfs` (direct, instant), `UPower` (D-Bus, the Flatpak-friendly route, driven by a small
dedicated tokio runtime), and `Auto`, which tries UPower and falls back to sysfs.

**Configuration** is versioned JSON. Under Flatpak (`FLATPAK_ID` set) it lands at
`$XDG_CONFIG_HOME/config.json`, i.e.
`~/.var/app/io.github.Mvth1s.MonDashboard/config/config.json`; otherwise at
`~/.config/mondashboard/config.json`. Any read failure falls back to defaults — the app
must always start. `AppConfig` deliberately carries fields beyond v0.1 (`layouts`,
`alerts`, `overlay`) so the schema stays stable; serializing them is fine, building UI for
them is not.

### Core module pattern

Each module in `mondashboard-core/src/` exposes an immutable data struct (`CpuStats`), an
`AllXxxStats { … : Vec<_> }` wrapper for the multi-entity ones, a pure collection
function, and `#[cfg(test)]` tests at the bottom. `sysfs.rs` holds the shared sysfs/procfs
readers — use them rather than calling `std::fs` directly, since they already turn a
missing file into `None`.

Tests must pass on a machine that has none of the hardware (the CI runner has no GPU, no
battery, no fans): assert on invariants and on absence, not on values.

Errors use `Result<T, MonDashboardError>` — no `unwrap()`/`expect()` in production code.

### GPU detection

`gpu::get_gpu_stats()` → `multi::collect_all()` calls each backend's
`pub(super) fn detect() -> Vec<GpuStats>` in order (nvidia, amd, amd_igpu, intel) and
concatenates. `drm.rs` does the shared `/sys/class/drm` enumeration and resolves the
commercial name from the system's `pci.ids` when available. Multi-GPU is the expected case.

### GTK widget pattern

Widgets in `mondashboard-gtk/src/widgets/` are structs wrapping `container: gtk4::Box`,
with `fn new() -> Self` and `fn update(&self, data: &XxxStats)` (a convention, not a Rust
trait). Shared helpers live in `widgets/mod.rs`: `carte()`, `ligne()`, `appliquer_niveau()`
and the formatters — a widget should not format a byte count itself.

A widget whose hardware is absent calls `container.set_visible(false)`; the grid closes up
around it. Rows whose count is only known at runtime (GPUs, disks, interfaces, fans) are
rebuilt only when that count changes, never on every tick.

The dashboard is a `GtkFlowBox` (`layout/grid.rs`), not a fixed grid: a grid forces its
minimum width onto the window, which then cannot be shrunk. Because the FlowBox is
homogeneous, **the widest card decides how many columns fit** — that is why long labels are
ellipsized (`sous_titre()`, the process name) and why Adwaita's 150 px minimum width on
progress bars is overridden in CSS. Three cards fit around 1200 px, two around 800 px, one
below that.

`graph.rs` is the Cairo graph: a 60-value circular buffer that fills right to left, fixed
0–100 % or auto-scaled. At the 2 s tick, 60 points is two minutes of history (SPEC.md says
"60 s" because it was written against a 1 s tick).

## Conventions

- **Comments, doc comments and commit messages are in French**, like `README.md` and
  `SPEC.md`. Internal identifiers are French too (`carte`, `ligne`, `appliquer_niveau`).
- **Public core API names stay English** — `CpuStats`, `get_cpu_stats`, the `SPEC.md`
  contract. Do not rename them.
- Comments explain *why*, especially every hardware quirk worked around. Those comments are
  the reason the next reader does not reintroduce the bug.

## Scope v0.1 (current target)

Build ONLY what is listed. The ❌ items that already have stub files
(`overlay.rs`, `settings.rs`, `alerts.rs`, `process::kill_process`) stay `todo!()`.

✅ All 8 widgets (CPU, GPU, RAM, Network, Disk, Process, Battery, Fans)
✅ Tray icon, left-click show/hide and a menu to quit
✅ Auto dark/light theme via libadwaita
✅ Adaptive layout, 1 to 3 cards per row depending on window width (no drag & drop)
✅ Fixed 2 s refresh (`AppConfig::refresh_interval_secs` = 2; SPEC.md §4.2 says 1 s — 2 s wins)

❌ Drag & drop, multiple layouts · Mini overlay · Notifications & alerts · Settings window · Kill process

## Flatpak

App ID `io.github.Mvth1s.MonDashboard`, manifest at
`flatpak/io.github.Mvth1s.MonDashboard.yml`, runtime `org.gnome.Platform` 49.

```bash
flatpak-builder --install --user build-dir flatpak/io.github.Mvth1s.MonDashboard.yml
flatpak run io.github.Mvth1s.MonDashboard
```

The manifest builds with `--share=network` to fetch crates; a Flathub submission will need
frozen sources from `flatpak-cargo-generator`. **The build has been verified**: it compiles,
installs and runs.

What the sandbox changes, all confirmed by running the packaged app — check these whenever
you touch a collector, they are invisible in a native run:

| Inside the sandbox | Consequence |
|---|---|
| PIDs are isolated | The app only sees its own processes. `ProcessWidget` says so instead of listing four internal threads. Flatpak always unshares the PID namespace, so there is no permission that fixes this. |
| Mount points are the container's (`/usr`, `/app`) | `DiskWidget` titles rows by partition (`/dev/nvme0n1p2`) when `is_sandboxed()`. Only the volumes the sandbox can see are listed. |
| `smartctl` is absent | SMART reports "indisponible". |
| `ping` is absent | Latency falls back to a TCP connect (`network::ping_tcp`). |
| `/sys` is readable | CPU/GPU/disk temperatures, fans and `pci.ids` name resolution all work normally. |

`mondashboard_core::is_sandboxed()` is the check (`FLATPAK_ID` or `/.flatpak-info`).

## UI Reference

The HTML mockup is `MonDashboard _standalone_.html` at the repo root (note the spaces).
Logos and icons are flat in `assets/` (there is no `assets/icons/`).

Metric colors, distinct from alert thresholds. Orange (60–85 %) and red (> 85 %) are
fixed — they signal a problem and must stay recognizable. Below 60 %, the desktop's own
accent color is used instead of green, so the app matches the user's theme; `@ACCENT@` in
`app.rs`'s stylesheet is substituted at startup and again on `notify::accent-color`.

## Data sources per module

| Module | Source |
|---|---|
| cpu.rs | `sysinfo` + hwmon by controller name (k10temp, zenpower, coretemp…) |
| memory.rs | `sysinfo` + `/proc/meminfo` for the cache figure |
| gpu/drm.rs | `/sys/class/drm` enumeration, shared by the AMD and Intel backends |
| gpu/nvidia.rs | `nvml-wrapper` (feature-gated) |
| network.rs | `sysinfo` + differential snapshot; `ping`, then TCP connect as fallback |
| disk.rs | `sysinfo` + `/proc/diskstats` differential; `smartctl` in a background thread |
| battery.rs | `/sys/class/power_supply` and/or UPower over D-Bus |
| fans.rs | every hwmon's `fan*_input` / `fan*_label` |

AMD GPU data comes from sysfs on purpose — no `rocm-smi` dependency for users.

## Agents

`.claude/agents/` holds three project agents: `collecteur-systeme` (collectors),
`widget-gtk` (UI), and `revue-materiel`, a pre-merge reviewer that hunts hardware
assumptions, panics and anything blocking the GTK loop.

## Debugging the running app — two traps that cost hours

- **The compositor remembers the window size.** KWin reopens the window at whatever size
  the user last dragged it to, keyed by application id, and `default_width` is then
  ignored. A layout that looks broken ("only one column!") may just be a narrow remembered
  window. To test sizing honestly, temporarily change `ID_APPLICATION` — a class the
  compositor has never seen gets the requested size.
- **An installed Flatpak hijacks native launches.** Both builds claim the same D-Bus name,
  so running `./target/debug/mondashboard` can silently activate the *installed Flatpak*
  instead, showing stale code. The tell is sandbox-only text (the process widget's
  message). Run `flatpak uninstall --user io.github.Mvth1s.MonDashboard` while iterating
  natively. Also note `pkill -f mondashboard` matches the shell command itself and kills
  the caller — use `pkill -x mondashboard`.
