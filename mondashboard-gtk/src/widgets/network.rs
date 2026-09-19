use std::cell::RefCell;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation};
use mondashboard_core::network::NetworkStats;

use super::graph::{Echelle, Graph};
use super::{ABSENT, carte, format_debit, ligne};

struct LigneInterface {
    nom: Label,
    debits: Label,
    adresse: Label,
}

pub struct NetworkWidget {
    pub container: GtkBox,
    interfaces: GtkBox,
    lignes: RefCell<Vec<LigneInterface>>,
    ping: Label,
    graph: Graph,
}

impl NetworkWidget {
    pub fn new() -> Self {
        let (container, contenu) = carte("Réseau");

        let graph = Graph::new(Echelle::Automatique);
        contenu.append(&graph.area);

        let interfaces = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();
        contenu.append(&interfaces);

        let (ligne_ping, ping) = ligne("Ping");
        contenu.append(&ligne_ping);

        Self {
            container,
            interfaces,
            lignes: RefCell::new(Vec::new()),
            ping,
            graph,
        }
    }

    pub fn update(&self, data: &NetworkStats) {
        if self.lignes.borrow().len() != data.interfaces.len() {
            self.reconstruire(data.interfaces.len());
        }

        for (ligne, interface) in self.lignes.borrow().iter().zip(&data.interfaces) {
            let active = data.active_interface.as_deref() == Some(interface.name.as_str());
            ligne.nom.set_label(&interface.name);
            if active {
                ligne.nom.add_css_class("interface-active");
            } else {
                ligne.nom.remove_css_class("interface-active");
            }
            ligne.debits.set_label(&format!(
                "↓ {}   ↑ {}",
                format_debit(interface.rx_bytes_per_sec),
                format_debit(interface.tx_bytes_per_sec)
            ));
            ligne
                .adresse
                .set_label(interface.ip_local.as_deref().unwrap_or("non connectée"));
        }

        // Le graphe suit l'interface active, celle qui porte le trafic utile.
        let debit_actif = data
            .interfaces
            .iter()
            .find(|interface| data.active_interface.as_deref() == Some(interface.name.as_str()))
            .map(|interface| interface.rx_bytes_per_sec)
            .unwrap_or(0);
        self.graph.push(debit_actif as f32);

        self.ping.set_label(
            &data
                .ping_ms
                .map(|millisecondes| format!("{millisecondes:.0} ms ({})", data.ping_host))
                .unwrap_or_else(|| ABSENT.to_string()),
        );
    }

    fn reconstruire(&self, nombre: usize) {
        while let Some(enfant) = self.interfaces.first_child() {
            self.interfaces.remove(&enfant);
        }
        let mut lignes = self.lignes.borrow_mut();
        lignes.clear();

        for _ in 0..nombre {
            let racine = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(2)
                .build();
            let nom = Label::builder().halign(Align::Start).build();
            let debits = Label::builder()
                .halign(Align::Start)
                .css_classes(["valeur-secondaire"])
                .build();
            let adresse = Label::builder()
                .halign(Align::Start)
                .css_classes(["secondaire"])
                .build();
            racine.append(&nom);
            racine.append(&debits);
            racine.append(&adresse);
            self.interfaces.append(&racine);
            lignes.push(LigneInterface {
                nom,
                debits,
                adresse,
            });
        }
    }
}

impl Default for NetworkWidget {
    fn default() -> Self {
        Self::new()
    }
}
