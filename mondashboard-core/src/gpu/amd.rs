//! GPU AMD dédié — source : sysfs /sys/class/drm, sans dépendance à rocm-smi.

use super::{GpuStats, GpuVendor, drm};

const MB: u64 = 1024 * 1024;

pub(super) fn detect() -> Vec<GpuStats> {
    drm::cards()
        .into_iter()
        .filter(|card| card.vendor_id == drm::VENDOR_AMD && !card.is_integrated())
        .map(|card| GpuStats {
            vendor: GpuVendor::AmdDiscrete,
            model: card.model(),
            usage_percent: card.usage_percent(),
            vram_used_mb: card.vram_used_bytes().map(|bytes| bytes / MB),
            vram_total_mb: card.vram_total_bytes().map(|bytes| bytes / MB),
            shared_memory_mb: None,
            temperature_celsius: card.temperature_celsius(),
            frequency_mhz: card.frequency_mhz(),
            power_watts: card.power_watts(),
        })
        .collect()
}
