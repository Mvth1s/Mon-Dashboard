use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, ProgressBar};
use mondashboard_core::battery::{BatteryState, BatteryStats};

use super::{ABSENT, appliquer_niveau, barre, carte, ligne};

pub struct BatteryWidget {
    pub container: GtkBox,
    niveau: Label,
    barre: ProgressBar,
    etat: Label,
    autonomie: Label,
    sante: Label,
    cycles: Label,
    modele: Label,
}

impl BatteryWidget {
    pub fn new() -> Self {
        let (container, contenu) = carte("Batterie");

        let niveau = Label::builder()
            .halign(Align::Start)
            .css_classes(["valeur-principale"])
            .build();
        contenu.append(&niveau);

        let barre = barre();
        contenu.append(&barre);

        let (ligne_etat, etat) = ligne("État");
        contenu.append(&ligne_etat);
        let (ligne_autonomie, autonomie) = ligne("Autonomie");
        contenu.append(&ligne_autonomie);
        let (ligne_sante, sante) = ligne("Santé");
        contenu.append(&ligne_sante);
        let (ligne_cycles, cycles) = ligne("Cycles");
        contenu.append(&ligne_cycles);
        let (ligne_modele, modele) = ligne("Modèle");
        contenu.append(&ligne_modele);

        Self {
            container,
            niveau,
            barre,
            etat,
            autonomie,
            sante,
            cycles,
            modele,
        }
    }

    pub fn update(&self, data: &BatteryStats) {
        // Sur un ordinateur fixe, le widget n'a pas lieu d'être : il est
        // masqué, ce qui n'est pas une erreur.
        if !data.present {
            self.container.set_visible(false);
            return;
        }
        self.container.set_visible(true);

        self.niveau.set_label(&format!("{:.0} %", data.percentage));
        self.barre
            .set_fraction((data.percentage / 100.0).clamp(0.0, 1.0) as f64);
        // Le code couleur est inversé : c'est une batterie basse qui alerte.
        appliquer_niveau(&self.niveau, 100.0 - data.percentage);
        appliquer_niveau(&self.barre, 100.0 - data.percentage);

        self.etat.set_label(match data.state {
            BatteryState::Charging => "en charge",
            BatteryState::Discharging => "sur batterie",
            BatteryState::Full => "chargée",
            BatteryState::Unknown => ABSENT,
        });

        self.autonomie.set_label(
            &data
                .time_to_empty_min
                .map(|minutes| format!("{} restantes", duree(minutes)))
                .or_else(|| {
                    data.time_to_full_min
                        .map(|minutes| format!("{} avant charge complète", duree(minutes)))
                })
                .unwrap_or_else(|| ABSENT.to_string()),
        );

        self.sante.set_label(&format!(
            "{:.0} % ({:.1} / {:.1} Wh)",
            data.health_percent, data.energy_full_wh, data.energy_full_design_wh
        ));
        self.cycles.set_label(
            &data
                .cycle_count
                .map(|nombre| nombre.to_string())
                .unwrap_or_else(|| ABSENT.to_string()),
        );

        let identite = [
            data.vendor.clone(),
            data.model.clone(),
            data.technology.clone(),
        ]
        .into_iter()
        .flatten()
        .filter(|texte| !texte.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
        self.modele.set_label(if identite.is_empty() {
            ABSENT
        } else {
            &identite
        });
    }
}

fn duree(minutes: u32) -> String {
    if minutes >= 60 {
        format!("{} h {:02}", minutes / 60, minutes % 60)
    } else {
        format!("{minutes} min")
    }
}

impl Default for BatteryWidget {
    fn default() -> Self {
        Self::new()
    }
}
