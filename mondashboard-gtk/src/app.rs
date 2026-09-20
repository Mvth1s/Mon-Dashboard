//! Application GTK4 / libadwaita.

use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use gtk4::{CssProvider, MenuButton, PolicyType, ScrolledWindow, gdk, gio};
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
.icone-widget { -gtk-icon-size: 16px; opacity: 0.7; }
/* Chiffres de même largeur : les valeurs ne se décalent plus à chaque tick. */
.tabulaire { font-feature-settings: \"tnum\"; }
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

    application.connect_startup(|_| {
        charger_style();
        charger_icone();
    });
    application.connect_activate(construire);
    application.run();
}

/// Rend l'icône embarquée visible sous son nom d'application, pour la
/// fenêtre et la boîte « À propos », même sans installation dans le système.
fn charger_icone() {
    // Une icône manquante n'est pas une raison d'interrompre l'application.
    if let Err(erreur) = gio::resources_register_include!("mondashboard.gresource") {
        log::warn!("icône embarquée indisponible : {erreur}");
        return;
    }
    if let Some(display) = gdk::Display::default() {
        gtk4::IconTheme::for_display(&display)
            .add_resource_path("/io/github/Mvth1s/MonDashboard/icons");
    }
    gtk4::Window::set_default_icon_name(ID_APPLICATION);
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
    barre.pack_end(
        &MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&menu_principal())
            .tooltip_text("Menu principal")
            // Ouvre aussi au clavier avec F10, comme les applications GNOME.
            .primary(true)
            .build(),
    );

    let vue = adw::ToolbarView::new();
    vue.add_top_bar(&barre);
    vue.set_content(Some(&defilement));

    let fenetre = adw::ApplicationWindow::builder()
        .application(application)
        .title("MonDashboard")
        .default_width(config.window.width)
        .default_height(config.window.height)
        // Taille plancher volontairement basse : une seule colonne de cartes
        // reste lisible, par exemple à côté d'un jeu ou d'un éditeur.
        .width_request(360)
        .height_request(320)
        .content(&vue)
        .build();

    // Fermer la fenêtre met l'application en arrière-plan : l'icône de la
    // zone de notification reste le moyen de la retrouver ou de quitter.
    fenetre.connect_close_request(|fenetre| {
        retenir_geometrie(fenetre);
        fenetre.set_visible(false);
        glib::Propagation::Stop
    });

    crate::polling::start(
        tableau,
        tray,
        Duration::from_secs(config.refresh_interval_secs.max(1) as u64),
        config.ping_host.clone(),
    );

    installer_actions(application, &fenetre);
    surveiller_tray(application, &fenetre, demandes);

    fenetre.present();
}

/// Enregistre la taille de la fenêtre pour la prochaine ouverture. La
/// configuration est relue juste avant d'écrire, afin de ne rien perdre de ce
/// qui aurait été modifié à la main entre-temps.
fn retenir_geometrie(fenetre: &adw::ApplicationWindow) {
    let (largeur, hauteur) = (fenetre.width(), fenetre.height());
    if largeur <= 0 || hauteur <= 0 {
        return;
    }
    let mut config = load_config();
    if config.window.width == largeur && config.window.height == hauteur {
        return;
    }
    config.window.width = largeur;
    config.window.height = hauteur;
    save_config(&config);
}

fn menu_principal() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Masquer la fenêtre"), Some("app.masquer"));
    menu.append(Some("À propos de MonDashboard"), Some("app.a-propos"));
    menu.append(Some("Quitter"), Some("app.quitter"));
    menu
}

/// Actions du menu et raccourcis clavier.
fn installer_actions(application: &adw::Application, fenetre: &adw::ApplicationWindow) {
    let a_propos = gio::SimpleAction::new("a-propos", None);
    let parent = fenetre.clone();
    a_propos.connect_activate(move |_, _| afficher_a_propos(&parent));
    application.add_action(&a_propos);

    let masquer = gio::SimpleAction::new("masquer", None);
    let a_masquer = fenetre.clone();
    masquer.connect_activate(move |_, _| a_masquer.set_visible(false));
    application.add_action(&masquer);

    let quitter = gio::SimpleAction::new("quitter", None);
    let a_quitter = application.clone();
    let avant_de_quitter = fenetre.clone();
    quitter.connect_activate(move |_, _| {
        retenir_geometrie(&avant_de_quitter);
        a_quitter.quit();
    });
    application.add_action(&quitter);

    // Ctrl+W masque sans quitter : l'application continue derrière son icône.
    application.set_accels_for_action("app.masquer", &["<Control>w"]);
    application.set_accels_for_action("app.quitter", &["<Control>q"]);
}

fn afficher_a_propos(parent: &adw::ApplicationWindow) {
    adw::AboutDialog::builder()
        .application_name("MonDashboard")
        .application_icon(ID_APPLICATION)
        .version(env!("CARGO_PKG_VERSION"))
        .developer_name("Mvth1s")
        .developers(vec!["Mvth1s".to_string()])
        .license_type(gtk4::License::Gpl30)
        .comments("Tableau de bord système lisible au premier regard.")
        .website("https://github.com/Mvth1s/Mon-Dashboard")
        .issue_url("https://github.com/Mvth1s/Mon-Dashboard/issues")
        .build()
        .present(Some(parent));
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
            retenir_geometrie(&fenetre);
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
