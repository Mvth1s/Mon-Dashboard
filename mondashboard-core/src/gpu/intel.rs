//! GPU Intel — Arc dédié et iGPU, tous deux via sysfs.
//!
//! Les pilotes i915 et xe n'exposent pas de taux d'occupation dans sysfs
//! (il faudrait l'interface perf du noyau) : `usage_percent` vaut donc `None`
//! plutôt qu'une valeur inventée.

use super::{GpuStats, GpuVendor, drm};

const MB: u64 = 1024 * 1024;

pub(super) fn detect() -> Vec<GpuStats> {
    drm::cards()
        .into_iter()
        .filter(|card| card.vendor_id == drm::VENDOR_INTEL)
        .map(|card| {
            let integrated = card.is_integrated();
            GpuStats {
                vendor: if integrated {
                    GpuVendor::IntelIgpu
                } else {
                    GpuVendor::IntelArc
                },
                model: card.model(),
                usage_percent: card.usage_percent(),
                vram_used_mb: None,
                vram_total_mb: if integrated {
                    None
                } else {
                    card.vram_total_bytes().map(|bytes| bytes / MB)
                },
                shared_memory_mb: if integrated {
                    card.shared_memory_bytes().map(|bytes| bytes / MB)
                } else {
                    None
                },
                temperature_celsius: card.temperature_celsius(),
                frequency_mhz: card.frequency_mhz(),
            }
        })
        .collect()
}
