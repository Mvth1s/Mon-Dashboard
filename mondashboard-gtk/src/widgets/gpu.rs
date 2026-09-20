use std::cell::RefCell;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ProgressBar};
use mondashboard_core::gpu::{AllGpuStats, GpuStats, GpuVendor};

use super::graph::{Echelle, Graph};
use super::{
    ABSENT, appliquer_niveau, barre, carte, format_mo, format_pourcentage, format_temperature,
    ligne, ratio, sous_titre,
};

struct LigneGpu {
    racine: GtkBox,
    modele: Label,
    charge: Label,
    barre: ProgressBar,
    memoire: Label,
    barre_memoire: ProgressBar,
    temperature: Label,
    frequence: Label,
    puissance: Label,
    graph: Graph,
}

pub struct GpuWidget {
    pub container: GtkBox,
    contenu: GtkBox,
    lignes: RefCell<Vec<LigneGpu>>,
}

impl GpuWidget {
    pub fn new() -> Self {
        let (container, contenu) = carte("Carte graphique", "video-display-symbolic");
        Self {
            container,
            contenu,
            lignes: RefCell::new(Vec::new()),
        }
    }

    pub fn update(&self, data: &AllGpuStats) {
        // Sans carte détectée, le widget disparaît au lieu d'afficher des zéros.
        if data.gpus.is_empty() {
            self.container.set_visible(false);
            return;
        }
        self.container.set_visible(true);

        // Le nombre de cartes ne change pas en cours d'exécution : la
        // reconstruction n'a lieu qu'au premier affichage.
        if self.lignes.borrow().len() != data.gpus.len() {
            self.reconstruire(data.gpus.len());
        }

        for (ligne, carte) in self.lignes.borrow().iter().zip(&data.gpus) {
            maj_ligne(ligne, carte);
        }
    }

    fn reconstruire(&self, nombre: usize) {
        while let Some(enfant) = self.contenu.first_child() {
            self.contenu.remove(&enfant);
        }
        let mut lignes = self.lignes.borrow_mut();
        lignes.clear();

        for index in 0..nombre {
            if index > 0 {
                self.contenu
                    .append(&gtk4::Separator::new(Orientation::Horizontal));
            }
            let ligne_gpu = construire_ligne();
            self.contenu.append(&ligne_gpu.racine);
            lignes.push(ligne_gpu);
        }
    }
}

fn construire_ligne() -> LigneGpu {
    let racine = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();

    let modele = sous_titre();
    racine.append(&modele);

    let charge = Label::builder()
        .halign(Align::Start)
        .css_classes(["valeur-principale", "tabulaire"])
        .build();
    racine.append(&charge);

    let barre_charge = barre();
    racine.append(&barre_charge);

    let graph = Graph::new(Echelle::Pourcentage);
    racine.append(&graph.area);

    let (ligne_memoire, memoire) = ligne("Mémoire");
    racine.append(&ligne_memoire);

    let barre_memoire = barre();
    barre_memoire.add_css_class("barre-fine");
    racine.append(&barre_memoire);

    let (ligne_temperature, temperature) = ligne("Température");
    racine.append(&ligne_temperature);
    let (ligne_frequence, frequence) = ligne("Fréquence");
    racine.append(&ligne_frequence);
    let (ligne_puissance, puissance) = ligne("Consommation");
    racine.append(&ligne_puissance);

    LigneGpu {
        racine,
        modele,
        charge,
        barre: barre_charge,
        memoire,
        barre_memoire,
        temperature,
        frequence,
        puissance,
        graph,
    }
}

fn maj_ligne(ligne: &LigneGpu, data: &GpuStats) {
    ligne.modele.set_label(&format!(
        "{} — {}",
        etiquette_vendeur(&data.vendor),
        data.model
    ));

    // Les pilotes Intel n'exposent pas le taux d'occupation : on l'indique
    // au lieu d'afficher 0 %.
    match data.usage_percent {
        Some(charge) => {
            ligne.charge.set_label(&format!("{charge:.0} %"));
            ligne.barre.set_visible(true);
            ligne
                .barre
                .set_fraction((charge / 100.0).clamp(0.0, 1.0) as f64);
            appliquer_niveau(&ligne.charge, charge);
            appliquer_niveau(&ligne.barre, charge);
        }
        None => {
            ligne.charge.set_label(&format_pourcentage(None));
            ligne.barre.set_visible(false);
        }
    }
    // Le graphe suit la charge ; sans mesure disponible, il reste à plat
    // plutôt que de disparaître et de faire sauter la carte.
    ligne.graph.push(data.usage_percent.unwrap_or(0.0));

    match (data.vram_used_mb, data.vram_total_mb) {
        (Some(utilisee), Some(totale)) => {
            let occupation = ratio(utilisee as f64, totale as f64);
            ligne
                .memoire
                .set_label(&format!("{} / {}", format_mo(utilisee), format_mo(totale)));
            ligne.barre_memoire.set_visible(true);
            ligne
                .barre_memoire
                .set_fraction((occupation / 100.0) as f64);
            appliquer_niveau(&ligne.barre_memoire, occupation);
        }
        _ => {
            // Puce intégrée : la mémoire est partagée avec la RAM système.
            ligne.barre_memoire.set_visible(false);
            ligne.memoire.set_label(
                &data
                    .shared_memory_mb
                    .map(|partagee| format!("{} partagés", format_mo(partagee)))
                    .unwrap_or_else(|| ABSENT.to_string()),
            );
        }
    }

    ligne
        .temperature
        .set_label(&format_temperature(data.temperature_celsius));
    ligne.puissance.set_label(
        &data
            .power_watts
            .map(|watts| format!("{watts:.0} W"))
            .unwrap_or_else(|| ABSENT.to_string()),
    );
    ligne.frequence.set_label(
        &data
            .frequency_mhz
            .map(|mhz| format!("{mhz} MHz"))
            .unwrap_or_else(|| ABSENT.to_string()),
    );
}

fn etiquette_vendeur(vendeur: &GpuVendor) -> &'static str {
    match vendeur {
        GpuVendor::Nvidia => "NVIDIA",
        GpuVendor::AmdDiscrete => "AMD",
        GpuVendor::AmdIgpu => "AMD intégré",
        GpuVendor::IntelArc => "Intel Arc",
        GpuVendor::IntelIgpu => "Intel intégré",
        GpuVendor::Unknown => "GPU",
    }
}

impl Default for GpuWidget {
    fn default() -> Self {
        Self::new()
    }
}
