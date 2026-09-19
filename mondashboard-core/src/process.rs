use sysinfo::System;

use crate::MonDashboardError;

#[derive(Debug, Clone)]
pub struct ProcessStats {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub user: String,
    pub status: String,
}

#[derive(Debug, Clone, Default)]
pub struct AllProcessStats {
    /// Sorted by CPU% descending by default.
    pub processes: Vec<ProcessStats>,
}

pub fn get_process_stats(sys: &System) -> AllProcessStats {
    todo!()
}

pub fn kill_process(pid: u32) -> Result<(), MonDashboardError> {
    todo!()
}
