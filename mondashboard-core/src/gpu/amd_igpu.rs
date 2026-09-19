//! iGPU AMD (APU) — source : sysfs. La mémoire est partagée avec la RAM,
//! donc on renseigne `shared_memory_mb` plutôt que `vram_*`.

use super::{GpuStats, GpuVendor, drm};

const MB: u64 = 1024 * 1024;

pub(super) fn detect() -> Vec<GpuStats> {
    drm::cards()
        .into_iter()
        .filter(|card| card.vendor_id == drm::VENDOR_AMD && card.is_integrated())
        .map(|card| GpuStats {
            vendor: GpuVendor::AmdIgpu,
            model: card.model(),
            usage_percent: card.usage_percent(),
            vram_used_mb: None,
            vram_total_mb: None,
            shared_memory_mb: card.shared_memory_bytes().map(|bytes| bytes / MB),
            temperature_celsius: card.temperature_celsius(),
            frequency_mhz: card.frequency_mhz(),
        })
        .collect()
}
