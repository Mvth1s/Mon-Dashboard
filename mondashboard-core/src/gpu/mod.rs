mod amd;
mod amd_igpu;
mod drm;
mod intel;
mod multi;
mod nvidia;

#[derive(Debug, Clone)]
pub enum GpuVendor {
    Nvidia,
    AmdDiscrete,
    AmdIgpu,
    IntelArc,
    IntelIgpu,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct GpuStats {
    pub vendor: GpuVendor,
    pub model: String,
    pub usage_percent: Option<f32>,
    /// None for iGPUs (shared memory).
    pub vram_used_mb: Option<u64>,
    pub vram_total_mb: Option<u64>,
    /// iGPUs only.
    pub shared_memory_mb: Option<u64>,
    pub temperature_celsius: Option<f32>,
    pub frequency_mhz: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct AllGpuStats {
    /// One entry per detected GPU (e.g. iGPU + discrete on gaming laptops).
    pub gpus: Vec<GpuStats>,
}

/// Auto-detects all GPUs and returns their stats.
pub fn get_gpu_stats() -> AllGpuStats {
    multi::collect_all()
}
