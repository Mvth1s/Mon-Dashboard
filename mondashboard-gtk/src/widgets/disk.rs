use std::cell::RefCell;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ProgressBar};
use mondashboard_core::disk::{AllDiskStats, DiskKind, DiskStats, SmartHealth};

use super::{appliquer_niveau, barre, carte, format_debit, format_temperature, ligne, ratio};

/// Un volume monté : c'est la seule information qui varie d'une partition à
/// l'autre.
struct LigneVolume {
    titre: Label,
    barre: ProgressBar,
    occupation: Label,
}

/// Un disque physique et les volumes qu'il porte. Température, état SMART et
/// vitesses appartiennent au disque : les répéter sous chaque partition
/// laissait croire à des mesures distinctes alors qu'elles sont identiques.
struct SectionDisque {
    titre: Label,
    sante: Label,
    vitesses: Label,
    volumes: Vec<LigneVolume>,
}

pub struct DiskWidget {
    pub container: GtkBox,
    contenu: GtkBox,
    sections: RefCell<Vec<SectionDisque>>,
    /// Disques et nombre de volumes de chacun : la reconstruction n'a lieu
    /// que si ce découpage change (montage ou démontage d'un volume).
    decoupage: RefCell<Vec<(String, usize)>>,
}

impl DiskWidget {
    pub fn new() -> Self {
        let (container, contenu) = carte("Stockage", "drive-harddisk-symbolic");
        Self {
            container,
            contenu,
            sections: RefCell::new(Vec::new()),
            decoupage: RefCell::new(Vec::new()),
        }
    }

    pub fn update(&self, data: &AllDiskStats) {
        if data.disks.is_empty() {
            self.container.set_visible(false);
            return;
        }
        self.container.set_visible(true);

        let groupes = grouper(data);
        let decoupage: Vec<(String, usize)> = groupes
            .iter()
            .map(|(disque, volumes)| (disque.clone(), volumes.len()))
            .collect();

        if *self.decoupage.borrow() != decoupage {
            self.reconstruire(&decoupage);
            *self.decoupage.borrow_mut() = decoupage;
        }

        for (section, (_, volumes)) in self.sections.borrow().iter().zip(&groupes) {
            maj_section(section, volumes);
        }
    }

    fn reconstruire(&self, decoupage: &[(String, usize)]) {
        while let Some(enfant) = self.contenu.first_child() {
            self.contenu.remove(&enfant);
        }
        let mut sections = self.sections.borrow_mut();
        sections.clear();

        for (index, (_, nombre_volumes)) in decoupage.iter().enumerate() {
            if index > 0 {
                self.contenu
                    .append(&gtk4::Separator::new(Orientation::Horizontal));
            }
            sections.push(construire_section(&self.contenu, *nombre_volumes));
        }
    }
}

/// Regroupe les volumes par disque physique, en conservant l'ordre
/// d'apparition pour que l'affichage ne saute pas d'un tick à l'autre.
fn grouper(data: &AllDiskStats) -> Vec<(String, Vec<&DiskStats>)> {
    let mut groupes: Vec<(String, Vec<&DiskStats>)> = Vec::new();
    for volume in &data.disks {
        match groupes
            .iter_mut()
            .find(|(disque, _)| disque == &volume.device)
        {
            Some((_, volumes)) => volumes.push(volume),
            None => groupes.push((volume.device.clone(), vec![volume])),
        }
    }
    groupes
}

fn construire_section(parent: &GtkBox, nombre_volumes: usize) -> SectionDisque {
    let racine = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();

    let entete = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let titre = Label::builder().halign(Align::Start).hexpand(true).build();
    let sante = Label::builder()
        .halign(Align::End)
        .css_classes(["secondaire", "tabulaire"])
        .build();
    entete.append(&titre);
    entete.append(&sante);
    racine.append(&entete);

    let (ligne_vitesses, vitesses) = ligne("Débit");
    racine.append(&ligne_vitesses);

    let mut volumes = Vec::with_capacity(nombre_volumes);
    for _ in 0..nombre_volumes {
        let bloc = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .margin_top(4)
            .build();
        let titre_volume = Label::builder()
            .halign(Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::Middle)
            .max_width_chars(24)
            .css_classes(["secondaire"])
            .build();
        let barre_volume = barre();
        let occupation = Label::builder()
            .halign(Align::Start)
            .css_classes(["secondaire", "tabulaire"])
            .build();

        bloc.append(&titre_volume);
        bloc.append(&barre_volume);
        bloc.append(&occupation);
        racine.append(&bloc);

        volumes.push(LigneVolume {
            titre: titre_volume,
            barre: barre_volume,
            occupation,
        });
    }

    parent.append(&racine);
    SectionDisque {
        titre,
        sante,
        vitesses,
        volumes,
    }
}

fn maj_section(section: &SectionDisque, volumes: &[&DiskStats]) {
    let Some(premier) = volumes.first() else {
        return;
    };

    section.titre.set_label(&format!(
        "{} — {}",
        premier.device,
        etiquette_type(&premier.kind)
    ));
    section.sante.set_label(&format!(
        "{} · {}",
        format_temperature(premier.temperature_celsius),
        etiquette_sante(&premier.smart_health)
    ));
    // Les compteurs viennent du disque : une seule ligne, pas une par volume.
    section.vitesses.set_label(&format!(
        "↓ {}   ↑ {}",
        format_debit(premier.read_bytes_per_sec),
        format_debit(premier.write_bytes_per_sec)
    ));

    for (ligne_volume, volume) in section.volumes.iter().zip(volumes) {
        let occupation = ratio(volume.used_gb, volume.total_gb);
        ligne_volume.titre.set_label(&emplacement(volume));
        ligne_volume.barre.set_fraction((occupation / 100.0) as f64);
        appliquer_niveau(&ligne_volume.barre, occupation);
        ligne_volume.occupation.set_label(&format!(
            "{:.1} Go / {:.1} Go ({:.0} %)",
            volume.used_gb, volume.total_gb, occupation
        ));
    }
}

/// Dans un bac à sable, les points de montage sont ceux du conteneur
/// (« /usr », « /app ») ; le nom de la partition, lui, reste exact.
fn emplacement(volume: &DiskStats) -> String {
    if mondashboard_core::is_sandboxed() {
        volume.name.clone()
    } else {
        volume.mount_point.clone()
    }
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
        SmartHealth::Good => "SMART bon",
        SmartHealth::Warning => "SMART à surveiller",
        SmartHealth::Critical => "SMART critique",
        // smartctl exige en général les droits root : l'information est
        // indisponible, ce n'est pas une panne.
        SmartHealth::Unavailable => "SMART indisponible",
    }
}

impl Default for DiskWidget {
    fn default() -> Self {
        Self::new()
    }
}
