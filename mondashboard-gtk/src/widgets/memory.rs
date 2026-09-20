use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, ProgressBar};
use mondashboard_core::memory::MemoryStats;

use super::graph::{Echelle, Graph};
use super::{appliquer_niveau, barre, carte, format_mo, ligne, ratio};

pub struct MemoryWidget {
    pub container: GtkBox,
    resume: Label,
    barre: ProgressBar,
    disponible: Label,
    cache: Label,
    swap: Label,
    barre_swap: ProgressBar,
    graph: Graph,
}

impl MemoryWidget {
    pub fn new() -> Self {
        let (container, contenu) = carte("Mémoire vive", "media-flash-symbolic");

        let resume = Label::builder()
            .halign(Align::Start)
            .css_classes(["valeur-principale", "tabulaire"])
            .build();
        contenu.append(&resume);

        let barre_memoire = barre();
        contenu.append(&barre_memoire);

        let graph = Graph::new(Echelle::Pourcentage);
        contenu.append(&graph.area);

        let (ligne_disponible, disponible) = ligne("Disponible");
        contenu.append(&ligne_disponible);
        let (ligne_cache, cache) = ligne("Cache");
        contenu.append(&ligne_cache);
        let (ligne_swap, swap) = ligne("Swap");
        contenu.append(&ligne_swap);

        let barre_swap = barre();
        barre_swap.add_css_class("barre-fine");
        contenu.append(&barre_swap);

        Self {
            container,
            resume,
            barre: barre_memoire,
            disponible,
            cache,
            swap,
            barre_swap,
            graph,
        }
    }

    pub fn update(&self, data: &MemoryStats) {
        let occupation = ratio(data.used_mb as f64, data.total_mb as f64);

        self.resume.set_label(&format!(
            "{} / {} ({:.0} %)",
            format_mo(data.used_mb),
            format_mo(data.total_mb),
            occupation
        ));
        self.barre.set_fraction((occupation / 100.0) as f64);
        appliquer_niveau(&self.resume, occupation);
        appliquer_niveau(&self.barre, occupation);
        self.graph.push(occupation);

        self.disponible.set_label(&format_mo(data.available_mb));
        self.cache.set_label(&format_mo(data.cached_mb));

        // Une machine sans swap affiche « aucun » plutôt que 0 / 0.
        if data.swap_total_mb == 0 {
            self.swap.set_label("aucun");
            self.barre_swap.set_visible(false);
        } else {
            let occupation_swap = ratio(data.swap_used_mb as f64, data.swap_total_mb as f64);
            self.swap.set_label(&format!(
                "{} / {}",
                format_mo(data.swap_used_mb),
                format_mo(data.swap_total_mb)
            ));
            self.barre_swap.set_visible(true);
            self.barre_swap
                .set_fraction((occupation_swap / 100.0) as f64);
            appliquer_niveau(&self.barre_swap, occupation_swap);
        }
    }
}

impl Default for MemoryWidget {
    fn default() -> Self {
        Self::new()
    }
}
