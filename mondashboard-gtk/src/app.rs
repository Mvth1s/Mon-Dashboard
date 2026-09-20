//! Application GTK4 / libadwaita.

use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use gtk4::{CssProvider, PolicyType, ScrolledWindow, gdk};
use mondashboard_core::{battery, disk, network};

use crate::layout::{Tableau, load_config, save_config};
use crate::tray::Demandes;

const ID_APPLICATION: &str = "io.github.Mvth1s.MonDashboard";

/// Fréquence de relève des demandes de l'icône. Assez court pour qu'un clic
/// paraisse instantané, assez long pour ne rien coûter.
const PERIODE_TRAY: Duration = Duration::from_millis(150);

/// Orange et rouge sont fixes : ils signalent un problème et doivent rester
/// reconnaissables. En deçà de 60 %, `@ACCENT@` est remplacé au démarrage par
/// la couleur d'accentuation du bureau, pour que l'application s'y accorde.
/// Fonds et textes viennent du thème libadwaita, qui suit le mode clair ou
/// sombre.
const STYLE: &str = "
.carte-widget { padding: 14px; }
.titre-widget { font-weight: 700; font-size: 1.05em; }
.valeur-principale { font-size: 1.7em; font-weight: 700; }
.valeur-secondaire { font-size: 0.95em; }
.secondaire { opacity: 0.65; font-size: 0.9em; }
.interface-active { font-weight: 700; }
.niveau-ok { color: @ACCENT@; }
.niveau-moyen { color: #ff7800; }
.niveau-critique { color: #e01b24; }
progressbar.niveau-ok > trough > progress { background-color: @ACCENT@; }
progressbar.niveau-moyen > trough > progress { background-color: #ff7800; }
progressbar.niveau-critique > trough > progress { background-color: #e01b24; }
/* Adwaita impose 150px de large à chaque barre : huit barres par cœur
   rendaient la carte si large qu'une seule tenait par ligne, et la fenêtre
   ne pouvait plus être rétrécie. */
progressbar > trough, progressbar > trough > progress { min-width: 0; }
progressbar.barre-coeur > trough, progressbar.barre-coeur > trough > progress { min-height: 6px; min-width: 14px; }
progressbar.barre-fine > trough, progressbar.barre-fine > trough > progress { min-height: 4px; }
";

pub fn run() {
    let application = adw::Application::builder()
        .application_id(ID_APPLICATION)
        .build();

    application.connect_startup(|_| charger_style());
    application.connect_activate(construire);
    application.run();
}

fn charger_style() {
    let Some(display) = gdk::Display::default() else {
        log::warn!("aucun affichage disponible, style par défaut conservé");
        return;
    };
    let fournisseur = CssProvider::new();
    fournisseur.load_from_string(&style_accentue());
    gtk4::style_context_add_provider_for_display(
        &display,
        &fournisseur,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // L'utilisateur peut changer la couleur d'accentuation sans redémarrer :
    // on recharge la feuille de style à la volée.
    adw::StyleManager::default().connect_accent_color_notify(move |_| {
        fournisseur.load_from_string(&style_accentue());
    });
}

/// Injecte la couleur d'accentuation du système dans la feuille de style.
/// libadwaita la lit du portail XDG, ce qui couvre GNOME comme KDE.
fn style_accentue() -> String {
    let accent = adw::StyleManager::default().accent_color_rgba();
    let couleur = format!(
        "rgb({}, {}, {})",
        (accent.red() * 255.0).round() as u8,
        (accent.green() * 255.0).round() as u8,
        (accent.blue() * 255.0).round() as u8
    );
    STYLE.replace("@ACCENT@", &couleur)
}

fn construire(application: &adw::Application) {
    // `activate` est ré-émis quand on relance l'application déjà ouverte :
    // on se contente alors de ramener la fenêtre au premier plan.
    if let Some(fenetre) = application.active_window() {
        fenetre.present();
        return;
    }

    let config = load_config();
    // Premier lancement : on matérialise le fichier pour qu'il soit
    // modifiable à la main tant qu'il n'y a pas de fenêtre de réglages.
    save_config(&config);

    // Les sources lentes (ping, SMART, D-Bus) tournent à part et ne bloquent
    // jamais la boucle graphique.
    network::start_ping_monitor(&config.ping_host);
    disk::start_smart_monitor();
    battery::start_monitor(config.battery_source);

    let tableau = Rc::new(Tableau::new());
    let demandes = Arc::new(Demandes::default());
    let tray = Rc::new(crate::tray::setup(demandes.clone()));

    // `Never` en horizontal force la fenêtre à rester aussi large que son
    // contenu : c'est ce qui empêchait de la rétrécir. En `Automatic`, une
    // barre apparaît au pire, et le FlowBox replie les cartes avant cela.
    let defilement = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vexpand(true)
        .child(&tableau.grille)
        .build();

    let barre = adw::HeaderBar::new();
    let vue = adw::ToolbarView::new();
    vue.add_top_bar(&barre);
    vue.set_content(Some(&defilement));

    let fenetre = adw::ApplicationWindow::builder()
        .application(application)
        .title("MonDashboard")
        .default_width(1080)
        .default_height(760)
        // Taille plancher volontairement basse : une seule colonne de cartes
        // reste lisible, par exemple à côté d'un jeu ou d'un éditeur.
        .width_request(360)
        .height_request(320)
        .content(&vue)
        .build();

    // Fermer la fenêtre met l'application en arrière-plan : l'icône de la
    // zone de notification reste le moyen de la retrouver ou de quitter.
    fenetre.connect_close_request(|fenetre| {
        fenetre.set_visible(false);
        glib::Propagation::Stop
    });

    crate::polling::start(
        tableau,
        tray,
        Duration::from_secs(config.refresh_interval_secs.max(1) as u64),
        config.ping_host.clone(),
    );

    surveiller_tray(application, &fenetre, demandes);

    fenetre.present();
}

/// Relève les demandes de l'icône dans le thread graphique, seul autorisé à
/// toucher à la fenêtre.
fn surveiller_tray(
    application: &adw::Application,
    fenetre: &adw::ApplicationWindow,
    demandes: Arc<Demandes>,
) {
    let application = application.clone();
    let fenetre = fenetre.clone();

    glib::timeout_add_local(PERIODE_TRAY, move || {
        if demandes.prendre_quitter() {
            application.quit();
            return glib::ControlFlow::Break;
        }
        if demandes.prendre_basculer() {
            if fenetre.is_visible() {
                fenetre.set_visible(false);
            } else {
                fenetre.present();
            }
        }
        glib::ControlFlow::Continue
    });
}
