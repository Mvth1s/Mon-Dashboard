//! Disposition fixe du tableau de bord (v0.1 : pas de glisser-déposer).

use gtk4::prelude::*;
use gtk4::{Grid, Orientation};

use crate::widgets::battery::BatteryWidget;
use crate::widgets::cpu::CpuWidget;
use crate::widgets::disk::DiskWidget;
use crate::widgets::fans::FansWidget;
use crate::widgets::gpu::GpuWidget;
use crate::widgets::memory::MemoryWidget;
use crate::widgets::network::NetworkWidget;
use crate::widgets::process::ProcessWidget;

/// Tous les widgets, placés une fois pour toutes.
pub struct Tableau {
    pub grille: Grid,
    pub cpu: CpuWidget,
    pub gpu: GpuWidget,
    pub memoire: MemoryWidget,
    pub reseau: NetworkWidget,
    pub stockage: DiskWidget,
    pub processus: ProcessWidget,
    pub batterie: BatteryWidget,
    pub refroidissement: FansWidget,
}

impl Tableau {
    pub fn new() -> Self {
        let grille = Grid::builder()
            .row_spacing(14)
            .column_spacing(14)
            .margin_top(14)
            .margin_bottom(14)
            .margin_start(14)
            .margin_end(14)
            .column_homogeneous(true)
            .build();

        let cpu = CpuWidget::new();
        let gpu = GpuWidget::new();
        let memoire = MemoryWidget::new();
        let reseau = NetworkWidget::new();
        let stockage = DiskWidget::new();
        let processus = ProcessWidget::new();
        let batterie = BatteryWidget::new();
        let refroidissement = FansWidget::new();

        // Deux colonnes : le matériel principal à gauche, le reste à droite.
        // Les cartes sans matériel se masquent d'elles-mêmes et la grille se
        // referme autour.
        grille.attach(&cpu.container, 0, 0, 1, 1);
        grille.attach(&gpu.container, 1, 0, 1, 1);
        grille.attach(&memoire.container, 0, 1, 1, 1);
        grille.attach(&reseau.container, 1, 1, 1, 1);
        grille.attach(&stockage.container, 0, 2, 1, 1);
        grille.attach(&processus.container, 1, 2, 1, 1);
        grille.attach(&batterie.container, 0, 3, 1, 1);
        grille.attach(&refroidissement.container, 1, 3, 1, 1);

        // Sous une fenêtre étroite, la grille passerait à l'étroit : on laisse
        // le défilement vertical s'en charger.
        grille.set_orientation(Orientation::Horizontal);

        Self {
            grille,
            cpu,
            gpu,
            memoire,
            reseau,
            stockage,
            processus,
            batterie,
            refroidissement,
        }
    }
}

impl Default for Tableau {
    fn default() -> Self {
        Self::new()
    }
}
