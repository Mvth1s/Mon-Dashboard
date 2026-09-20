use std::cell::RefCell;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Grid, Label, ProgressBar};
use mondashboard_core::cpu::CpuStats;

use super::graph::{Echelle, Graph};
use super::{appliquer_niveau, barre, carte, format_temperature, ligne, sous_titre};

/// Nombre de colonnes de la grille des cœurs.
const COLONNES_COEURS: i32 = 8;

pub struct CpuWidget {
    pub container: GtkBox,
    modele: Label,
    charge: Label,
    barre: ProgressBar,
    frequence: Label,
    temperature: Label,
    charge_moyenne: Label,
    grille_coeurs: Grid,
    barres_coeurs: RefCell<Vec<ProgressBar>>,
    graph: Graph,
}

impl CpuWidget {
    pub fn new() -> Self {
        let (container, contenu) = carte("Processeur", "computer-symbolic");

        let modele = sous_titre();
        contenu.append(&modele);

        let charge = Label::builder()
            .halign(Align::Start)
            .css_classes(["valeur-principale", "tabulaire"])
            .build();
        contenu.append(&charge);

        let barre = barre();
        contenu.append(&barre);

        let graph = Graph::new(Echelle::Pourcentage);
        contenu.append(&graph.area);

        let (ligne_frequence, frequence) = ligne("Fréquence");
        contenu.append(&ligne_frequence);
        let (ligne_temperature, temperature) = ligne("Température");
        contenu.append(&ligne_temperature);
        let (ligne_charge_moyenne, charge_moyenne) = ligne("Charge moy.");
        ligne_charge_moyenne.set_tooltip_text(Some(
            "Nombre moyen de tâches en attente sur 1, 5 et 15 minutes",
        ));
        contenu.append(&ligne_charge_moyenne);

        let etiquette_coeurs = Label::builder()
            .label("Par cœur")
            .halign(Align::Start)
            .css_classes(["secondaire"])
            .build();
        contenu.append(&etiquette_coeurs);

        let grille_coeurs = Grid::builder()
            .row_spacing(4)
            .column_spacing(4)
            .column_homogeneous(true)
            .build();
        contenu.append(&grille_coeurs);

        Self {
            container,
            modele,
            charge,
            barre,
            frequence,
            temperature,
            charge_moyenne,
            grille_coeurs,
            barres_coeurs: RefCell::new(Vec::new()),
            graph,
        }
    }

    pub fn update(&self, data: &CpuStats) {
        self.modele.set_label(&format!(
            "{} — {} cœurs / {} threads",
            data.model, data.physical_cores, data.logical_cores
        ));
        self.charge
            .set_label(&format!("{:.0} %", data.global_usage));
        self.barre
            .set_fraction((data.global_usage / 100.0).clamp(0.0, 1.0) as f64);
        appliquer_niveau(&self.charge, data.global_usage);
        appliquer_niveau(&self.barre, data.global_usage);
        self.graph.push(data.global_usage);

        self.frequence.set_label(&format!(
            "{} MHz{}",
            data.frequency_mhz,
            if data.frequency_max_mhz > 0 {
                format!(" / {} MHz", data.frequency_max_mhz)
            } else {
                String::new()
            }
        ));
        self.temperature
            .set_label(&format_temperature(data.temperature_celsius));
        self.charge_moyenne.set_label(&format!(
            "{:.2}  ·  {:.2}  ·  {:.2}",
            data.load_average[0], data.load_average[1], data.load_average[2]
        ));

        self.maj_coeurs(&data.per_core_usage);
    }

    /// Le nombre de cœurs n'est connu qu'à la première mesure : la grille est
    /// donc construite une seule fois, puis seulement mise à jour.
    fn maj_coeurs(&self, charges: &[f32]) {
        let mut barres = self.barres_coeurs.borrow_mut();

        if barres.len() != charges.len() {
            while let Some(enfant) = self.grille_coeurs.first_child() {
                self.grille_coeurs.remove(&enfant);
            }
            barres.clear();
            for index in 0..charges.len() as i32 {
                let barre = ProgressBar::builder().hexpand(true).build();
                barre.add_css_class("barre-coeur");
                barre.set_tooltip_text(Some(&format!("Cœur {}", index + 1)));
                self.grille_coeurs.attach(
                    &barre,
                    index % COLONNES_COEURS,
                    index / COLONNES_COEURS,
                    1,
                    1,
                );
                barres.push(barre);
            }
        }

        for (barre, charge) in barres.iter().zip(charges) {
            barre.set_fraction((*charge / 100.0).clamp(0.0, 1.0) as f64);
            appliquer_niveau(barre, *charge);
        }
    }
}

impl Default for CpuWidget {
    fn default() -> Self {
        Self::new()
    }
}
