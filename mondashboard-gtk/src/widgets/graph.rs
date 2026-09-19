//! Graphe temps réel : un tampon circulaire dessiné avec Cairo.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{DrawingArea, cairo};

/// Nombre de points conservés. À un tick toutes les 2 s, cela représente
/// deux minutes d'historique.
pub const POINTS: usize = 60;

const HAUTEUR: i32 = 56;

/// Échelle verticale du graphe.
#[derive(Clone, Copy, PartialEq)]
pub enum Echelle {
    /// 0 à 100 % (CPU, RAM).
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
        area.set_draw_func(move |area, contexte, largeur, hauteur| {
            dessiner(
                area,
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
    area: &DrawingArea,
    contexte: &cairo::Context,
    largeur: f64,
    hauteur: f64,
    valeurs: &VecDeque<f32>,
    echelle: Echelle,
) {
    if valeurs.is_empty() {
        return;
    }

    let maximum = match echelle {
        Echelle::Pourcentage => 100.0,
        // On garde une borne minimale pour qu'une activité nulle ne produise
        // pas un graphe qui saute au moindre octet.
        Echelle::Automatique => valeurs.iter().cloned().fold(1.0_f32, f32::max),
    } as f64;

    // La couleur du tracé suit le code couleur de charge, en réutilisant la
    // couleur de premier plan du thème pour rester lisible en clair comme en
    // sombre lorsque la charge est normale.
    let derniere = *valeurs.back().unwrap_or(&0.0) as f64;
    let niveau = match echelle {
        Echelle::Pourcentage => derniere,
        Echelle::Automatique => derniere / maximum * 100.0,
    };
    let (rouge, vert, bleu) = couleur(niveau, area);
    contexte.set_line_width(2.0);
    contexte.set_source_rgb(rouge, vert, bleu);

    // Le graphe se remplit de la droite vers la gauche au démarrage.
    let pas = largeur / (POINTS - 1) as f64;
    let depart = largeur - (valeurs.len() - 1) as f64 * pas;

    for (index, valeur) in valeurs.iter().enumerate() {
        let x = depart + index as f64 * pas;
        let y = hauteur - (*valeur as f64 / maximum).clamp(0.0, 1.0) * (hauteur - 2.0) - 1.0;
        if index == 0 {
            contexte.move_to(x, y);
        } else {
            contexte.line_to(x, y);
        }
    }
    let _ = contexte.stroke_preserve();

    // Aplat léger sous la courbe.
    contexte.line_to(largeur, hauteur);
    contexte.line_to(depart, hauteur);
    contexte.close_path();
    contexte.set_source_rgba(rouge, vert, bleu, 0.18);
    let _ = contexte.fill();
}

fn couleur(niveau: f64, area: &DrawingArea) -> (f64, f64, f64) {
    if niveau > 85.0 {
        (0.88, 0.11, 0.14) // #e01b24
    } else if niveau >= 60.0 {
        (1.0, 0.47, 0.0) // #ff7800
    } else {
        let _ = area;
        (0.18, 0.76, 0.49) // #2ec27e
    }
}
