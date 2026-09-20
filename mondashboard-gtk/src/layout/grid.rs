//! Disposition fixe du tableau de bord (v0.1 : pas de glisser-déposer).

use gtk4::{Align, FlowBox, Orientation, SelectionMode};

use crate::widgets::battery::BatteryWidget;
use crate::widgets::cpu::CpuWidget;
use crate::widgets::disk::DiskWidget;
use crate::widgets::fans::FansWidget;
use crate::widgets::gpu::GpuWidget;
use crate::widgets::memory::MemoryWidget;
use crate::widgets::network::NetworkWidget;
use crate::widgets::process::ProcessWidget;

/// Nombre maximal de cartes par ligne. Au-delà, elles deviendraient trop
/// étroites pour leurs graphes.
const CARTES_PAR_LIGNE: u32 = 3;

/// Tous les widgets, répartis selon la largeur disponible.
pub struct Tableau {
    pub grille: FlowBox,
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
        // Une grille à colonnes fixes impose sa largeur minimale à la
        // fenêtre, qui devient alors impossible à rétrécir. Le FlowBox, lui,
        // replie les cartes en une seule colonne quand la place manque.
        let grille = FlowBox::builder()
            .orientation(Orientation::Horizontal)
            .selection_mode(SelectionMode::None)
            .min_children_per_line(1)
            .max_children_per_line(CARTES_PAR_LIGNE)
            .homogeneous(true)
            .row_spacing(14)
            .column_spacing(14)
            .margin_top(14)
            .margin_bottom(14)
            .margin_start(14)
            .margin_end(14)
            // Les cartes se rangent en haut : le reste de la hauteur revient
            // au défilement, pas à des cartes étirées.
            .valign(Align::Start)
            .build();

        let cpu = CpuWidget::new();
        let gpu = GpuWidget::new();
        let memoire = MemoryWidget::new();
        let reseau = NetworkWidget::new();
        let stockage = DiskWidget::new();
        let processus = ProcessWidget::new();
        let batterie = BatteryWidget::new();
        let refroidissement = FansWidget::new();

        // Ordre d'importance : les cartes masquées faute de matériel ne
        // laissent pas de trou, le FlowBox referme la disposition.
        for carte in [
            &cpu.container,
            &gpu.container,
            &memoire.container,
            &reseau.container,
            &stockage.container,
            &processus.container,
            &batterie.container,
            &refroidissement.container,
        ] {
            grille.append(carte);
        }

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
