//! Graphe temps réel : un tampon circulaire dessiné avec Cairo.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{DrawingArea, cairo};

/// Nombre de points conservés. À un tick toutes les 2 s, cela représente
/// deux minutes d'historique.
pub const POINTS: usize = 60;

const HAUTEUR: i32 = 54;
/// Repères horizontaux, pour que la zone se lise comme un graphe même quand
/// la courbe est basse — sans eux, une charge faible donne l'impression d'un
/// espace vide dans la carte.
const REPERES: [f64; 3] = [0.25, 0.5, 0.75];

/// Échelle verticale du graphe.
#[derive(Clone, Copy, PartialEq)]
pub enum Echelle {
    /// 0 à 100 % (CPU, RAM, GPU).
    Pourcentage,
    /// Ajustée au maximum observé (réseau, disque).
    Automatique,
}

pub struct Graph {
    pub area: DrawingArea,
    valeurs: Rc<RefCell<VecDeque<f32>>>,
}

impl Graph {
    pub fn new(echelle: Echelle) -> Self {
        let valeurs = Rc::new(RefCell::new(VecDeque::with_capacity(POINTS)));
        let area = DrawingArea::builder()
            .content_height(HAUTEUR)
            .hexpand(true)
            .build();

        let donnees = valeurs.clone();
        area.set_draw_func(move |_, contexte, largeur, hauteur| {
            dessiner(
                contexte,
                largeur as f64,
                hauteur as f64,
                &donnees.borrow(),
                echelle,
            );
        });

        Self { area, valeurs }
    }

    /// Ajoute une valeur et fait défiler le graphe vers la gauche.
    pub fn push(&self, valeur: f32) {
        let mut valeurs = self.valeurs.borrow_mut();
        if valeurs.len() == POINTS {
            valeurs.pop_front();
        }
        valeurs.push_back(valeur.max(0.0));
        drop(valeurs);
        self.area.queue_draw();
    }
}

fn dessiner(
    contexte: &cairo::Context,
    largeur: f64,
    hauteur: f64,
    valeurs: &VecDeque<f32>,
    echelle: Echelle,
) {
    // Fond et repères en gris translucide : lisible sur thème clair comme
    // sombre, sans couleur codée en dur.
    contexte.set_source_rgba(0.5, 0.5, 0.5, 0.08);
    contexte.rectangle(0.0, 0.0, largeur, hauteur);
    let _ = contexte.fill();

    contexte.set_line_width(1.0);
    contexte.set_source_rgba(0.5, 0.5, 0.5, 0.15);
    for repere in REPERES {
        let y = (hauteur * repere).round() + 0.5;
        contexte.move_to(0.0, y);
        contexte.line_to(largeur, y);
    }
    let _ = contexte.stroke();

    if valeurs.is_empty() {
        return;
    }

    let maximum = match echelle {
        Echelle::Pourcentage => 100.0,
        // Borne minimale : sans elle, une activité nulle ferait sauter le
        // graphe au moindre octet.
        Echelle::Automatique => valeurs.iter().cloned().fold(1.0_f32, f32::max),
    } as f64;

    // Sur une échelle automatique, le dernier point est souvent le maximum
    // observé : le colorer par « pourcentage du maximum » peindrait en rouge
    // le moindre pic de trafic. Seules les échelles en pourcentage, qui ont
    // un vrai seuil, suivent le code couleur.
    let (rouge, vert, bleu) = match echelle {
        Echelle::Pourcentage => couleur(*valeurs.back().unwrap_or(&0.0) as f64),
        Echelle::Automatique => super::couleur_accent(),
    };

    let pas = largeur / (POINTS - 1) as f64;
    let depart = largeur - (valeurs.len().saturating_sub(1)) as f64 * pas;
    let ordonnee =
        |valeur: f32| hauteur - (valeur as f64 / maximum).clamp(0.0, 1.0) * (hauteur - 3.0) - 1.5;

    contexte.set_line_width(2.0);
    contexte.set_source_rgb(rouge, vert, bleu);
    // Avec un seul relevé, on trace un court segment plutôt que rien.
    if valeurs.len() == 1 {
        let y = ordonnee(valeurs[0]);
        contexte.move_to(largeur - pas, y);
        contexte.line_to(largeur, y);
    } else {
        for (index, valeur) in valeurs.iter().enumerate() {
            let x = depart + index as f64 * pas;
            let y = ordonnee(*valeur);
            if index == 0 {
                contexte.move_to(x, y);
            } else {
                contexte.line_to(x, y);
            }
        }
    }
    let _ = contexte.stroke_preserve();

    // Aplat léger sous la courbe.
    contexte.line_to(largeur, hauteur);
    contexte.line_to(depart.min(largeur - pas), hauteur);
    contexte.close_path();
    contexte.set_source_rgba(rouge, vert, bleu, 0.18);
    let _ = contexte.fill();
}

/// Orange et rouge sont fixes car ils signalent un problème ; en deçà, on
/// prend la couleur d'accentuation du système.
fn couleur(niveau: f64) -> (f64, f64, f64) {
    if niveau > 85.0 {
        (0.88, 0.11, 0.14) // #e01b24
    } else if niveau >= 60.0 {
        (1.0, 0.47, 0.0) // #ff7800
    } else {
        super::couleur_accent()
    }
}
