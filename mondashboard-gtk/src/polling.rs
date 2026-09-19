//! Boucle de rafraîchissement : un tick collecte, puis met à jour l'affichage.

use std::rc::Rc;
use std::time::Duration;

use mondashboard_core::disk::AllDiskStats;
use mondashboard_core::network::NetworkStats;
use mondashboard_core::{battery, cpu, disk, fans, gpu, memory, network, process};
use sysinfo::{ProcessesToUpdate, System};

use crate::layout::Tableau;
use crate::tray::Tray;

/// État conservé entre deux ticks : `sysinfo` et les compteurs cumulés dont
/// les débits se déduisent par différence.
struct Collecteur {
    systeme: System,
    reseau: NetworkStats,
    disques: AllDiskStats,
}

impl Collecteur {
    fn new(hote_ping: String) -> Self {
        Self {
            systeme: System::new_all(),
            reseau: NetworkStats {
                ping_host: hote_ping,
                ..NetworkStats::default()
            },
            disques: AllDiskStats::default(),
        }
    }

    fn tick(&mut self, tableau: &Tableau, tray: &Tray) {
        self.systeme.refresh_cpu_all();
        self.systeme.refresh_memory();
        self.systeme.refresh_processes(ProcessesToUpdate::All, true);

        let cpu = cpu::get_cpu_stats(&self.systeme);
        tableau.cpu.update(&cpu);
        tray.maj_charge(cpu.global_usage);

        tableau
            .memoire
            .update(&memory::get_memory_stats(&self.systeme));
        tableau.gpu.update(&gpu::get_gpu_stats());
        tableau
            .processus
            .update(&process::get_process_stats(&self.systeme));
        tableau.batterie.update(&battery::get_battery_stats());
        tableau.refroidissement.update(&fans::get_cooling_stats());

        // Les deux collecteurs différentiels consomment le relevé précédent
        // et le remplacent.
        self.reseau = network::get_network_stats(&self.reseau);
        tableau.reseau.update(&self.reseau);

        self.disques = disk::get_disk_stats(&self.disques);
        tableau.stockage.update(&self.disques);
    }
}

/// Démarre la boucle. Un premier tick a lieu immédiatement pour que la
/// fenêtre ne s'ouvre pas vide.
pub fn start(tableau: Rc<Tableau>, tray: Rc<Tray>, intervalle: Duration, hote_ping: String) {
    let mut collecteur = Collecteur::new(hote_ping);
    collecteur.tick(&tableau, &tray);

    glib::timeout_add_local(intervalle, move || {
        collecteur.tick(&tableau, &tray);
        glib::ControlFlow::Continue
    });
}
