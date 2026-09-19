#[derive(Debug, Clone)]
pub enum DiskKind {
    Ssd,
    Hdd,
    Nvme,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum SmartHealth {
    Good,
    Warning,
    Critical,
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct DiskStats {
    pub name: String,
    pub mount_point: String,
    pub kind: DiskKind,
    pub total_gb: f64,
    pub used_gb: f64,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub temperature_celsius: Option<f32>,
    pub smart_health: SmartHealth,
}

#[derive(Debug, Clone, Default)]
pub struct AllDiskStats {
    pub disks: Vec<DiskStats>,
}

/// Requires a previous snapshot to compute differential R/W speeds.
pub fn get_disk_stats(previous: &AllDiskStats) -> AllDiskStats {
    todo!()
}
