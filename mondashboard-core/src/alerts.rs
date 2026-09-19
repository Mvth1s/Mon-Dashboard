use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::{
    cpu::CpuStats, disk::AllDiskStats, gpu::AllGpuStats, memory::MemoryStats, network::NetworkStats,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertConfig {
    pub cpu_threshold_percent: Option<f32>,
    pub cpu_duration_secs: u32,
    pub ram_threshold_percent: Option<f32>,
    pub gpu_temp_threshold_celsius: Option<f32>,
    pub gpu_usage_threshold_percent: Option<f32>,
    pub disk_free_threshold_gb: Option<f64>,
    pub ping_threshold_ms: Option<f32>,
    pub battery_threshold_percent: Option<f32>,
    pub fan_stopped_threshold_rpm: Option<u32>,
    /// Minimum seconds between two notifications of the same type.
    pub cooldown_secs: u32,
}

#[derive(Debug, Clone)]
pub enum AlertKind {
    Cpu,
    Ram,
    GpuTemp,
    GpuUsage,
    DiskFree,
    Ping,
    Battery,
    FanStopped,
}

#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub kind: AlertKind,
    pub message: String,
    pub triggered_at: Instant,
}

/// Hors périmètre v0.1 (notifications & alertes) — voir CLAUDE.md.
pub fn check_alerts(
    _config: &AlertConfig,
    _cpu: &CpuStats,
    _memory: &MemoryStats,
    _gpu: &AllGpuStats,
    _disks: &AllDiskStats,
    _network: &NetworkStats,
    _last_alerts: &[AlertEvent],
) -> Vec<AlertEvent> {
    todo!()
}
