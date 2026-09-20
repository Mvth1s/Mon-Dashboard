//! Icône de la zone de notification (protocole StatusNotifierItem).
//!
//! L'icône vit dans son propre thread D-Bus. La communication avec la boucle
//! GTK passe par des drapeaux atomiques, relevés périodiquement : c'est le
//! moyen le plus simple de ne jamais toucher aux widgets hors du thread
//! principal.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use ksni::blocking::TrayMethods;

/// Icône embarquée dans le binaire, en blanc sur fond transparent comme le
/// veut une barre système. Le logo est fourni en image plutôt que par son nom
/// car le thème du bureau ne contient celle de l'application qu'une fois le
/// paquet installé : hors installation, l'hôte affichait un point
/// d'interrogation.
const ICONE: &[u8] = include_bytes!("../../assets/icon-transparent-blanc-192.png");

/// Demandes de l'icône vers la fenêtre.
#[derive(Default)]
pub struct Demandes {
    basculer: AtomicBool,
    quitter: AtomicBool,
}

impl Demandes {
    pub fn prendre_basculer(&self) -> bool {
        self.basculer.swap(false, Ordering::Relaxed)
    }

    pub fn prendre_quitter(&self) -> bool {
        self.quitter.swap(false, Ordering::Relaxed)
    }
}

struct Icone {
    demandes: Arc<Demandes>,
    /// Charge CPU en pourcent, stockée en entier pour rester atomique.
    charge_cpu: Arc<AtomicU32>,
}

impl ksni::Tray for Icone {
    fn id(&self) -> String {
        "io.github.Mvth1s.MonDashboard".to_string()
    }

    fn title(&self) -> String {
        "MonDashboard".to_string()
    }

    /// Volontairement vide : le protocole prévoit que l'hôte utilise alors
    /// l'image de `icon_pixmap`. Renvoyer un nom absent du thème produisait
    /// une icône « inconnue » au lieu du logo.
    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        icone_embarquee().into_iter().collect()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "MonDashboard".to_string(),
            description: format!(
                "Processeur : {} %\nClic pour afficher ou masquer la fenêtre",
                self.charge_cpu.load(Ordering::Relaxed)
            ),
            ..Default::default()
        }
    }

    /// Clic gauche : afficher ou masquer la fenêtre.
    fn activate(&mut self, _x: i32, _y: i32) {
        self.demandes.basculer.store(true, Ordering::Relaxed);
    }

    /// Clic droit : menu contextuel.
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            ksni::menu::StandardItem {
                label: "Afficher / masquer".to_string(),
                activate: Box::new(|icone: &mut Icone| {
                    icone.demandes.basculer.store(true, Ordering::Relaxed)
                }),
                ..Default::default()
            }
            .into(),
            ksni::MenuItem::Separator,
            ksni::menu::StandardItem {
                label: "Quitter".to_string(),
                activate: Box::new(|icone: &mut Icone| {
                    icone.demandes.quitter.store(true, Ordering::Relaxed)
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Convertit le PNG embarqué au format attendu par le protocole : ARGB32 en
/// ordre réseau, alors que PNG fournit du RGBA.
fn icone_embarquee() -> Option<ksni::Icon> {
    let mut lecture = png::Decoder::new(std::io::Cursor::new(ICONE))
        .read_info()
        .ok()?;
    let mut tampon = vec![0; lecture.output_buffer_size()?];
    let info = lecture.next_frame(&mut tampon).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        log::warn!("icône embarquée dans un format inattendu, icône du thème utilisée");
        return None;
    }

    let mut argb = Vec::with_capacity(info.buffer_size());
    for pixel in tampon[..info.buffer_size()].chunks_exact(4) {
        argb.extend_from_slice(&[pixel[3], pixel[0], pixel[1], pixel[2]]);
    }

    Some(ksni::Icon {
        width: info.width as i32,
        height: info.height as i32,
        data: argb,
    })
}

pub struct Tray {
    poignee: Option<ksni::blocking::Handle<Icone>>,
    charge_cpu: Arc<AtomicU32>,
}

/// Démarre l'icône.
///
/// Aucun hôte n'accepte forcément les icônes (GNOME sans extension
/// AppIndicator, session minimale) : dans ce cas l'application continue
/// normalement, simplement sans icône.
pub fn setup(demandes: Arc<Demandes>) -> Tray {
    let charge_cpu = Arc::new(AtomicU32::new(0));
    let icone = Icone {
        demandes,
        charge_cpu: charge_cpu.clone(),
    };

    let poignee = match icone.spawn() {
        Ok(poignee) => Some(poignee),
        Err(erreur) => {
            log::info!("zone de notification indisponible, icône désactivée : {erreur}");
            None
        }
    };

    Tray {
        poignee,
        charge_cpu,
    }
}

impl Tray {
    /// Met l'infobulle à jour avec la charge courante.
    pub fn maj_charge(&self, pourcentage: f32) {
        self.charge_cpu.store(
            pourcentage.round().clamp(0.0, 100.0) as u32,
            Ordering::Relaxed,
        );
        if let Some(poignee) = &self.poignee {
            // `update` force l'hôte à relire l'infobulle.
            poignee.update(|_| {});
        }
    }
}
