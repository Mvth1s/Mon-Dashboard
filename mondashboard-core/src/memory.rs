use sysinfo::System;

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub cached_mb: u64,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

pub fn get_memory_stats(sys: &System) -> MemoryStats {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_stats_fields_are_present() {
        let stats = MemoryStats {
            total_mb: 0,
            used_mb: 0,
            available_mb: 0,
            cached_mb: 0,
            swap_total_mb: 0,
            swap_used_mb: 0,
        };
        assert_eq!(stats.total_mb, 0);
    }
}
