use sysinfo::{System, Users};

use crate::MonDashboardError;

/// Nombre de processus conservés. La liste complète (souvent 400+) n'est
/// jamais affichée et coûterait cher à trier à chaque tick.
const MAX_PROCESSES: usize = 50;

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
    /// Triés par % CPU décroissant.
    pub processes: Vec<ProcessStats>,
}

pub fn get_process_stats(sys: &System) -> AllProcessStats {
    let users = Users::new_with_refreshed_list();

    let mut processes: Vec<ProcessStats> = sys
        .processes()
        .values()
        .map(|process| ProcessStats {
            pid: process.pid().as_u32(),
            name: process.name().to_string_lossy().to_string(),
            cpu_percent: process.cpu_usage(),
            memory_mb: process.memory() / (1024 * 1024),
            user: process
                .user_id()
                .and_then(|uid| users.get_user_by_id(uid))
                .map(|user| user.name().to_string())
                .unwrap_or_else(|| "—".to_string()),
            status: process.status().to_string(),
        })
        .collect();

    processes.sort_by(|a, b| {
        b.cpu_percent
            .total_cmp(&a.cpu_percent)
            .then_with(|| b.memory_mb.cmp(&a.memory_mb))
    });
    processes.truncate(MAX_PROCESSES);

    AllProcessStats { processes }
}

/// Hors périmètre v0.1 — voir CLAUDE.md.
pub fn kill_process(_pid: u32) -> Result<(), MonDashboardError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::ProcessesToUpdate;

    #[test]
    fn liste_triee_et_bornee() {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let stats = get_process_stats(&sys);

        assert!(!stats.processes.is_empty());
        assert!(stats.processes.len() <= MAX_PROCESSES);
        assert!(
            stats
                .processes
                .windows(2)
                .all(|pair| pair[0].cpu_percent >= pair[1].cpu_percent)
        );
    }
}
