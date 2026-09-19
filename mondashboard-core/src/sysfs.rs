//! Lecture bas niveau de sysfs et procfs, partagée par tous les collecteurs.
//!
//! Principe : un fichier absent veut dire « ce matériel n'existe pas sur cette
//! machine », jamais « erreur ». Tout renvoie donc `Option` ou une liste vide.

use std::fs;
use std::path::{Path, PathBuf};

pub fn read_string(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|content| content.trim().to_string())
}

pub fn read_u64(path: impl AsRef<Path>) -> Option<u64> {
    read_string(path)?.parse().ok()
}

pub fn read_i64(path: impl AsRef<Path>) -> Option<i64> {
    read_string(path)?.parse().ok()
}

/// Contenu d'un répertoire, trié. Liste vide si le répertoire n'existe pas.
pub fn list_dir(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .collect();
    entries.sort();
    entries
}

/// Nom de fichier d'un chemin, en `String`.
pub fn file_name(path: &Path) -> Option<String> {
    path.file_name()?.to_str().map(str::to_string)
}

/// Tous les contrôleurs hwmon présents. Aucun index n'est supposé : la
/// numérotation hwmonN change d'un démarrage à l'autre.
pub fn hwmon_dirs() -> Vec<PathBuf> {
    list_dir("/sys/class/hwmon")
}

/// Nom du contrôleur hwmon (« k10temp », « amdgpu », « nvme »…).
pub fn hwmon_name(dir: &Path) -> Option<String> {
    read_string(dir.join("name"))
}

/// Tous les hwmon portant ce nom (il peut y en avoir plusieurs : deux NVMe,
/// deux cartes graphiques…).
pub fn hwmon_dirs_named(name: &str) -> Vec<PathBuf> {
    hwmon_dirs()
        .into_iter()
        .filter(|dir| hwmon_name(dir).as_deref() == Some(name))
        .collect()
}

/// Température en °C depuis un fichier `tempN_input`, exprimé en millidegrés.
pub fn read_temp_c(path: impl AsRef<Path>) -> Option<f32> {
    read_i64(path).map(|millidegrees| millidegrees as f32 / 1000.0)
}

/// Température d'un hwmon, en préférant un capteur dont le libellé figure dans
/// `preferred` (ex. « Tctl » sur AMD, « Package id 0 » sur Intel). À défaut,
/// renvoie le premier capteur trouvé plutôt que rien.
pub fn hwmon_temp_by_label(dir: &Path, preferred: &[&str]) -> Option<f32> {
    let mut first = None;
    for entry in list_dir(dir) {
        let Some(name) = file_name(&entry) else {
            continue;
        };
        let Some(index) = name
            .strip_prefix("temp")
            .and_then(|rest| rest.strip_suffix("_input"))
        else {
            continue;
        };
        let Some(value) = read_temp_c(&entry) else {
            continue;
        };
        if first.is_none() {
            first = Some(value);
        }
        let label = read_string(dir.join(format!("temp{index}_label"))).unwrap_or_default();
        if preferred
            .iter()
            .any(|wanted| label.eq_ignore_ascii_case(wanted))
        {
            return Some(value);
        }
    }
    first
}

/// Chemin réel derrière les liens symboliques, pour relier un hwmon au
/// périphérique qu'il mesure.
pub fn canonical(path: impl AsRef<Path>) -> Option<PathBuf> {
    fs::canonicalize(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repertoire_absent_donne_liste_vide() {
        assert!(list_dir("/sys/class/ce-chemin-n-existe-pas").is_empty());
    }

    #[test]
    fn fichier_absent_donne_none() {
        assert!(read_u64("/sys/class/ce-chemin-n-existe-pas/valeur").is_none());
    }

    #[test]
    fn enumeration_hwmon_ne_panique_pas() {
        for dir in hwmon_dirs() {
            let _ = hwmon_name(&dir);
        }
    }
}
