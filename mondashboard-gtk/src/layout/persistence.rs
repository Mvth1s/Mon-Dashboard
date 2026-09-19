//! Chargement et sauvegarde de la configuration JSON.

use std::fs;
use std::path::PathBuf;

use mondashboard_core::config::AppConfig;

/// Emplacement du fichier de configuration.
///
/// Sous Flatpak, `XDG_CONFIG_HOME` pointe déjà vers
/// `~/.var/app/io.github.Mvth1s.MonDashboard/config` : le fichier s'y place
/// directement. Hors bac à sable, on range dans un sous-dossier dédié pour ne
/// pas encombrer `~/.config`.
pub fn chemin_config() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;

    if std::env::var_os("FLATPAK_ID").is_some() {
        Some(base.join("config.json"))
    } else {
        Some(base.join("mondashboard").join("config.json"))
    }
}

/// Charge la configuration. Toute erreur (fichier absent, JSON invalide,
/// écrit par une version plus récente) retombe sur les valeurs par défaut :
/// l'application doit démarrer quoi qu'il arrive.
pub fn load_config() -> AppConfig {
    let Some(chemin) = chemin_config() else {
        return AppConfig::default();
    };
    let Ok(contenu) = fs::read_to_string(&chemin) else {
        return AppConfig::default();
    };
    match serde_json::from_str(&contenu) {
        Ok(config) => config,
        Err(erreur) => {
            log::warn!(
                "configuration illisible ({}), valeurs par défaut utilisées : {erreur}",
                chemin.display()
            );
            AppConfig::default()
        }
    }
}

/// Écrit la configuration. L'échec est journalisé sans interrompre
/// l'application : ne pas pouvoir sauvegarder ne doit pas faire perdre
/// la session en cours.
pub fn save_config(config: &AppConfig) {
    let Some(chemin) = chemin_config() else {
        return;
    };
    if let Some(dossier) = chemin.parent()
        && let Err(erreur) = fs::create_dir_all(dossier)
    {
        log::warn!("dossier de configuration inaccessible : {erreur}");
        return;
    }
    let Ok(contenu) = serde_json::to_string_pretty(config) else {
        return;
    };

    // Écriture puis renommage : une coupure en cours d'écriture ne laisse pas
    // un fichier tronqué.
    let temporaire = chemin.with_extension("json.tmp");
    if fs::write(&temporaire, contenu).is_ok()
        && let Err(erreur) = fs::rename(&temporaire, &chemin)
    {
        log::warn!("configuration non sauvegardée : {erreur}");
    }
}
