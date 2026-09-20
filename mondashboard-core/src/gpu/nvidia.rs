//! GPU NVIDIA — source : NVML via nvml-wrapper.
//!
//! `nvidia-smi` ou le pilote peuvent être installés sans qu'aucune carte
//! NVIDIA ne soit présente. L'initialisation de NVML est donc la véritable
//! détection : si elle échoue, la machine n'a pas de GPU NVIDIA exploitable
//! et la liste renvoyée est vide.

use super::GpuStats;

#[cfg(feature = "nvidia")]
pub(super) fn detect() -> Vec<GpuStats> {
    use std::sync::OnceLock;

    use nvml_wrapper::Nvml;
    use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};

    use super::GpuVendor;

    const MB: u64 = 1024 * 1024;

    // NVML est coûteux à initialiser : on le fait une seule fois pour toute
    // la durée de vie du processus.
    static NVML: OnceLock<Option<Nvml>> = OnceLock::new();
    let Some(nvml) = NVML.get_or_init(|| Nvml::init().ok()).as_ref() else {
        return vec![];
    };

    let Ok(count) = nvml.device_count() else {
        return vec![];
    };

    (0..count)
        .filter_map(|index| {
            let device = nvml.device_by_index(index).ok()?;
            let memory = device.memory_info().ok();
            Some(GpuStats {
                vendor: GpuVendor::Nvidia,
                model: device
                    .name()
                    .unwrap_or_else(|_| format!("GPU NVIDIA {index}")),
                usage_percent: device
                    .utilization_rates()
                    .ok()
                    .map(|rates| rates.gpu as f32),
                vram_used_mb: memory.as_ref().map(|memory| memory.used / MB),
                vram_total_mb: memory.as_ref().map(|memory| memory.total / MB),
                shared_memory_mb: None,
                temperature_celsius: device
                    .temperature(TemperatureSensor::Gpu)
                    .ok()
                    .map(|celsius| celsius as f32),
                frequency_mhz: device.clock_info(Clock::Graphics).ok().map(u64::from),
                // NVML publie la puissance en milliwatts.
                power_watts: device
                    .power_usage()
                    .ok()
                    .map(|milliwatts| milliwatts as f32 / 1000.0),
            })
        })
        .collect()
}

#[cfg(not(feature = "nvidia"))]
pub(super) fn detect() -> Vec<GpuStats> {
    vec![]
}
