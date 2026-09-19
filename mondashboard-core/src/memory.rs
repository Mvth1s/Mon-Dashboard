use sysinfo::System;

use crate::sysfs;

const MB: u64 = 1024 * 1024;

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
    MemoryStats {
        total_mb: sys.total_memory() / MB,
        used_mb: sys.used_memory() / MB,
        available_mb: sys.available_memory() / MB,
        cached_mb: cached_bytes() / MB,
        swap_total_mb: sys.total_swap() / MB,
        swap_used_mb: sys.used_swap() / MB,
    }
}

/// `sysinfo` n'expose pas le cache : on le lit dans /proc/meminfo.
/// On additionne `Cached` et `SReclaimable`, ce que font `free` et les
/// moniteurs système, sinon le cache paraît anormalement bas.
fn cached_bytes() -> u64 {
    let Some(content) = sysfs::read_string("/proc/meminfo") else {
        return 0;
    };
    let mut total_kb = 0;
    for line in content.lines() {
        if let Some(value) = line
            .strip_prefix("Cached:")
            .or_else(|| line.strip_prefix("SReclaimable:"))
        {
            total_kb += parse_kb(value);
        }
    }
    total_kb * 1024
}

fn parse_kb(value: &str) -> u64 {
    value
        .split_whitespace()
        .next()
        .and_then(|number| number.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collecte_coherente_sur_la_machine_courante() {
        let mut sys = System::new();
        sys.refresh_memory();
        let stats = get_memory_stats(&sys);

        assert!(stats.total_mb > 0);
        assert!(stats.used_mb <= stats.total_mb);
    }

    #[test]
    fn parse_kb_lit_la_premiere_valeur() {
        assert_eq!(parse_kb("  123456 kB"), 123456);
        assert_eq!(parse_kb("illisible"), 0);
    }
}
