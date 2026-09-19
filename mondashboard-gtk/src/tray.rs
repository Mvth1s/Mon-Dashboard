//! Icône de la zone de notification (protocole StatusNotifierItem).
//!
//! L'icône vit dans son propre thread D-Bus. La communication avec la boucle
//! GTK passe par des drapeaux atomiques, relevés périodiquement : c'est le
//! moyen le plus simple de ne jamais toucher aux widgets hors du thread
//! principal.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use ksni::blocking::TrayMethods;

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

    fn icon_name(&self) -> String {
        "utilities-system-monitor-symbolic".to_string()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "MonDashboard".to_string(),
            description: format!("Processeur : {} %", self.charge_cpu.load(Ordering::Relaxed)),
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
