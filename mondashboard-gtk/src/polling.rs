//! Boucle de rafraîchissement : un tick collecte, puis met à jour l'affichage.

use std::cell::RefCell;
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

/// Délai du premier relevé. `sysinfo` a besoin de deux mesures espacées pour
/// donner un pourcentage d'occupation réaliste : afficher immédiatement
/// montrerait 0 % partout, ce qui est faux.
const PREMIER_RELEVE: Duration = Duration::from_millis(400);

/// Démarre la boucle.
pub fn start(tableau: Rc<Tableau>, tray: Rc<Tray>, intervalle: Duration, hote_ping: String) {
    let collecteur = Rc::new(RefCell::new(Collecteur::new(hote_ping)));

    {
        let collecteur = collecteur.clone();
        let tableau = tableau.clone();
        let tray = tray.clone();
        glib::timeout_add_local_once(PREMIER_RELEVE, move || {
            collecteur.borrow_mut().tick(&tableau, &tray);
        });
    }

    glib::timeout_add_local(intervalle, move || {
        collecteur.borrow_mut().tick(&tableau, &tray);
        glib::ControlFlow::Continue
    });
}
