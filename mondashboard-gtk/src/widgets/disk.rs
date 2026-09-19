use std::cell::RefCell;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ProgressBar};
use mondashboard_core::disk::{AllDiskStats, DiskKind, DiskStats, SmartHealth};

use super::{appliquer_niveau, barre, carte, format_debit, format_temperature, ratio};

struct LigneDisque {
    titre: Label,
    barre: ProgressBar,
    occupation: Label,
    vitesses: Label,
    sante: Label,
}

pub struct DiskWidget {
    pub container: GtkBox,
    contenu: GtkBox,
    lignes: RefCell<Vec<LigneDisque>>,
}

impl DiskWidget {
    pub fn new() -> Self {
        let (container, contenu) = carte("Stockage");
        Self {
            container,
            contenu,
            lignes: RefCell::new(Vec::new()),
        }
    }

    pub fn update(&self, data: &AllDiskStats) {
        if data.disks.is_empty() {
            self.container.set_visible(false);
            return;
        }
        self.container.set_visible(true);

        if self.lignes.borrow().len() != data.disks.len() {
            self.reconstruire(data.disks.len());
        }

        for (ligne, disque) in self.lignes.borrow().iter().zip(&data.disks) {
            maj_ligne(ligne, disque);
        }
    }

    fn reconstruire(&self, nombre: usize) {
        while let Some(enfant) = self.contenu.first_child() {
            self.contenu.remove(&enfant);
        }
        let mut lignes = self.lignes.borrow_mut();
        lignes.clear();

        for _ in 0..nombre {
            let racine = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(3)
                .build();
            let titre = Label::builder().halign(Align::Start).build();
            let barre_occupation = barre();
            let occupation = Label::builder()
                .halign(Align::Start)
                .css_classes(["secondaire"])
                .build();
            let vitesses = Label::builder()
                .halign(Align::Start)
                .css_classes(["valeur-secondaire"])
                .build();
            let sante = Label::builder()
                .halign(Align::Start)
                .css_classes(["secondaire"])
                .build();

            racine.append(&titre);
            racine.append(&barre_occupation);
            racine.append(&occupation);
            racine.append(&vitesses);
            racine.append(&sante);
            self.contenu.append(&racine);

            lignes.push(LigneDisque {
                titre,
                barre: barre_occupation,
                occupation,
                vitesses,
                sante,
            });
        }
    }
}

fn maj_ligne(ligne: &LigneDisque, data: &DiskStats) {
    let occupation = ratio(data.used_gb, data.total_gb);

    // Dans un bac à sable, les points de montage sont ceux du conteneur
    // (« /usr », « /app ») et n'ont plus de sens pour l'utilisateur ; le nom
    // du périphérique, lui, reste exact.
    let emplacement = if mondashboard_core::is_sandboxed() {
        // `name` est la partition (« /dev/nvme0n1p2 »), qui distingue deux
        // volumes d'un même disque, là où `device` les confondrait.
        data.name.as_str()
    } else {
        data.mount_point.as_str()
    };
    ligne
        .titre
        .set_label(&format!("{} — {}", emplacement, etiquette_type(&data.kind)));
    ligne.barre.set_fraction((occupation / 100.0) as f64);
    appliquer_niveau(&ligne.barre, occupation);
    ligne.occupation.set_label(&format!(
        "{:.1} Go / {:.1} Go ({:.0} %)",
        data.used_gb, data.total_gb, occupation
    ));
    // Les vitesses sont celles du disque physique : deux partitions d'un même
    // disque affichent donc la même valeur.
    ligne.vitesses.set_label(&format!(
        "lecture {}   écriture {}",
        format_debit(data.read_bytes_per_sec),
        format_debit(data.write_bytes_per_sec)
    ));
    ligne.sante.set_label(&format!(
        "{}   SMART : {}",
        format_temperature(data.temperature_celsius),
        etiquette_sante(&data.smart_health)
    ));
}

fn etiquette_type(kind: &DiskKind) -> &'static str {
    match kind {
        DiskKind::Nvme => "NVMe",
        DiskKind::Ssd => "SSD",
        DiskKind::Hdd => "disque dur",
        DiskKind::Unknown => "type inconnu",
    }
}

fn etiquette_sante(sante: &SmartHealth) -> &'static str {
    match sante {
        SmartHealth::Good => "bon état",
        SmartHealth::Warning => "à surveiller",
        SmartHealth::Critical => "critique",
        // smartctl exige en général les droits root : l'information est
        // simplement indisponible, ce n'est pas une panne.
        SmartHealth::Unavailable => "indisponible",
    }
}

impl Default for DiskWidget {
    fn default() -> Self {
        Self::new()
    }
}
