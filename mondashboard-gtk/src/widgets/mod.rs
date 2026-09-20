pub mod battery;
pub mod cpu;
pub mod disk;
pub mod fans;
pub mod gpu;
pub mod graph;
pub mod memory;
pub mod network;
pub mod process;

use adw::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ProgressBar};

/// Seuils d'affichage, fixes et non configurables : ils servent à lire l'état
/// d'un coup d'œil, contrairement aux seuils d'alerte.
const SEUIL_MOYEN: f32 = 60.0;
const SEUIL_CRITIQUE: f32 = 85.0;

/// Texte affiché lorsqu'une valeur n'est pas disponible sur cette machine.
pub const ABSENT: &str = "—";

/// Classe CSS correspondant à une charge : vert, orange puis rouge.
pub fn classe_niveau(pourcentage: f32) -> &'static str {
    if pourcentage > SEUIL_CRITIQUE {
        "niveau-critique"
    } else if pourcentage >= SEUIL_MOYEN {
        "niveau-moyen"
    } else {
        "niveau-ok"
    }
}

/// Applique la classe de niveau à un composant en retirant les autres.
pub fn appliquer_niveau(widget: &impl IsA<gtk4::Widget>, pourcentage: f32) {
    let widget = widget.as_ref();
    for classe in ["niveau-ok", "niveau-moyen", "niveau-critique"] {
        widget.remove_css_class(classe);
    }
    widget.add_css_class(classe_niveau(pourcentage));
}

/// Carte d'un widget : une en-tête (icône + titre) et une zone de contenu,
/// au style libadwaita.
pub fn carte(titre: &str, icone: &str) -> (GtkBox, GtkBox) {
    let carte = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        // Sans cela, la carte s'étire sur toute la hauteur de sa rangée et
        // creuse de grands vides entre ses lignes.
        .valign(Align::Start)
        .build();
    carte.add_css_class("card");
    carte.add_css_class("carte-widget");

    let entete = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let image = gtk4::Image::from_icon_name(icone);
    image.add_css_class("icone-widget");
    let titre = Label::builder()
        .label(titre)
        .halign(Align::Start)
        .css_classes(["titre-widget"])
        .build();
    entete.append(&image);
    entete.append(&titre);
    carte.append(&entete);

    let contenu = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();
    carte.append(&contenu);

    (carte, contenu)
}

/// Ligne « intitulé  valeur ».
///
/// L'intitulé occupe une largeur fixe en caractères, ce qui aligne les valeurs
/// en colonne sans les projeter à l'autre bout de la carte : sur une carte
/// large, une valeur collée à droite se lit mal, trop loin de son libellé.
pub fn ligne(intitule: &str) -> (GtkBox, Label) {
    let ligne = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    let gauche = Label::builder()
        .label(intitule)
        .halign(Align::Start)
        .xalign(0.0)
        .width_chars(13)
        .max_width_chars(13)
        .css_classes(["secondaire"])
        .build();
    let valeur = Label::builder()
        .label(ABSENT)
        .halign(Align::Start)
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        // Chiffres à chasse fixe : sans cela, une valeur qui passe de 9 à 10
        // décale tout le texte à chaque seconde.
        .css_classes(["tabulaire"])
        .build();
    ligne.append(&gauche);
    ligne.append(&valeur);
    (ligne, valeur)
}

/// Couleur d'accentuation du système, lue via libadwaita (portail XDG sous
/// KDE, réglage GNOME sinon). Elle remplace le vert pour les charges
/// normales, afin que l'application s'accorde au bureau.
pub fn couleur_accent() -> (f64, f64, f64) {
    let accent = adw::StyleManager::default().accent_color_rgba();
    (
        accent.red() as f64,
        accent.green() as f64,
        accent.blue() as f64,
    )
}

/// Sous-titre pouvant être long (modèle de processeur, de carte graphique) :
/// il est tronqué pour ne pas imposer sa largeur à toute la fenêtre.
pub fn sous_titre() -> Label {
    Label::builder()
        .halign(Align::Start)
        .xalign(0.0)
        .max_width_chars(26)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .css_classes(["secondaire"])
        .build()
}

pub fn barre() -> ProgressBar {
    ProgressBar::builder().hexpand(true).build()
}

/// Débit lisible : on bascule d'unité au seuil, comme les moniteurs système.
pub fn format_debit(octets_par_sec: u64) -> String {
    let octets = octets_par_sec as f64;
    if octets >= 1024.0 * 1024.0 {
        format!("{:.1} Mo/s", octets / (1024.0 * 1024.0))
    } else if octets >= 1024.0 {
        format!("{:.0} ko/s", octets / 1024.0)
    } else {
        format!("{octets_par_sec} o/s")
    }
}

/// Volume total (et non un débit) : Go au-delà du gigaoctet, Mo sinon.
pub fn format_octets(octets: u64) -> String {
    const GO: f64 = 1024.0 * 1024.0 * 1024.0;
    let valeur = octets as f64;
    if valeur >= GO {
        format!("{:.1} Go", valeur / GO)
    } else {
        format!("{:.0} Mo", valeur / (1024.0 * 1024.0))
    }
}

pub fn format_temperature(valeur: Option<f32>) -> String {
    valeur
        .map(|degres| format!("{degres:.0} °C"))
        .unwrap_or_else(|| ABSENT.to_string())
}

pub fn format_pourcentage(valeur: Option<f32>) -> String {
    valeur
        .map(|pourcent| format!("{pourcent:.0} %"))
        .unwrap_or_else(|| ABSENT.to_string())
}

/// Gio Mo lisibles à partir de mégaoctets.
pub fn format_mo(mega_octets: u64) -> String {
    if mega_octets >= 1024 {
        format!("{:.1} Go", mega_octets as f64 / 1024.0)
    } else {
        format!("{mega_octets} Mo")
    }
}

/// Pourcentage d'occupation, protégé contre un total nul (matériel qui ne
/// renseigne pas sa capacité).
pub fn ratio(utilise: f64, total: f64) -> f32 {
    if total <= 0.0 {
        return 0.0;
    }
    (utilise / total * 100.0).clamp(0.0, 100.0) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seuils_de_couleur() {
        assert_eq!(classe_niveau(10.0), "niveau-ok");
        assert_eq!(classe_niveau(60.0), "niveau-moyen");
        assert_eq!(classe_niveau(85.1), "niveau-critique");
    }

    #[test]
    fn total_nul_ne_divise_pas_par_zero() {
        assert_eq!(ratio(5.0, 0.0), 0.0);
    }

    #[test]
    fn unites_de_debit() {
        assert_eq!(format_debit(512), "512 o/s");
        assert_eq!(format_debit(2048), "2 ko/s");
        assert_eq!(format_debit(5 * 1024 * 1024), "5.0 Mo/s");
    }
}
