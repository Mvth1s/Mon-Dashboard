use std::cell::RefCell;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation};
use mondashboard_core::fans::{CoolingStats, FanKind};

use super::{carte, format_temperature, ligne};

pub struct FansWidget {
    pub container: GtkBox,
    contenu: GtkBox,
    lignes: RefCell<Vec<(Label, Label)>>,
    liquide: GtkBox,
    temperature_liquide: Label,
}

impl FansWidget {
    pub fn new() -> Self {
        let (container, contenu_carte) = carte("Refroidissement");

        let contenu = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();
        contenu_carte.append(&contenu);

        let (liquide, temperature_liquide) = ligne("Liquide");
        contenu_carte.append(&liquide);

        Self {
            container,
            contenu,
            lignes: RefCell::new(Vec::new()),
            liquide,
            temperature_liquide,
        }
    }

    pub fn update(&self, data: &CoolingStats) {
        // Beaucoup de machines n'exposent aucun tachymètre au noyau.
        if data.fans.is_empty() {
            self.container.set_visible(false);
            return;
        }
        self.container.set_visible(true);

        if self.lignes.borrow().len() != data.fans.len() {
            self.reconstruire(data.fans.len());
        }

        for ((etiquette, valeur), ventilateur) in self.lignes.borrow().iter().zip(&data.fans) {
            etiquette.set_label(&format!(
                "{} ({})",
                ventilateur.label,
                etiquette_type(&ventilateur.kind)
            ));
            // 0 tr/min est une information utile : ventilateur à l'arrêt,
            // mode silencieux ou courbe agressive.
            valeur.set_label(&format!("{} tr/min", ventilateur.rpm));
        }

        match data.coolant_temp_celsius {
            Some(_) => {
                self.liquide.set_visible(true);
                self.temperature_liquide
                    .set_label(&format_temperature(data.coolant_temp_celsius));
            }
            None => self.liquide.set_visible(false),
        }
    }

    fn reconstruire(&self, nombre: usize) {
        while let Some(enfant) = self.contenu.first_child() {
            self.contenu.remove(&enfant);
        }
        let mut lignes = self.lignes.borrow_mut();
        lignes.clear();

        for _ in 0..nombre {
            let ligne_ventilateur = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .build();
            let etiquette = Label::builder()
                .halign(Align::Start)
                .hexpand(true)
                .css_classes(["secondaire"])
                .build();
            let valeur = Label::builder().halign(Align::End).build();
            ligne_ventilateur.append(&etiquette);
            ligne_ventilateur.append(&valeur);
            self.contenu.append(&ligne_ventilateur);
            lignes.push((etiquette, valeur));
        }
    }
}

fn etiquette_type(kind: &FanKind) -> &'static str {
    match kind {
        FanKind::CpuCooler => "processeur",
        FanKind::CaseFan => "boîtier",
        FanKind::AioPump => "pompe",
        FanKind::AioRadiator => "radiateur",
        FanKind::GpuFan => "carte graphique",
        FanKind::Unknown => "non identifié",
    }
}

impl Default for FansWidget {
    fn default() -> Self {
        Self::new()
    }
}
