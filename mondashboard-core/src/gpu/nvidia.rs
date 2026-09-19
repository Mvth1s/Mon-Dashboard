/// Source: nvml-wrapper crate (NVIDIA Management Library).
use super::GpuStats;

#[cfg(feature = "nvidia")]
pub(super) fn detect() -> Vec<GpuStats> {
    todo!()
}

#[cfg(not(feature = "nvidia"))]
pub(super) fn detect() -> Vec<GpuStats> {
    vec![]
}
