//! Énumération des cartes graphiques via /sys/class/drm.
//!
//! Aucun `cardN` n'est supposé : la numérotation dépend de l'ordre de
//! chargement des pilotes. Une machine peut avoir zéro, une ou plusieurs
//! cartes, de constructeurs différents.

use std::path::PathBuf;

use crate::sysfs;

pub const VENDOR_AMD: &str = "0x1002";
pub const VENDOR_INTEL: &str = "0x8086";

/// En dessous de ce seuil, une puce AMD est considérée comme intégrée : un
/// APU ne réserve qu'une petite fenêtre de mémoire système, là où une carte
/// dédiée embarque sa propre VRAM. Heuristique assumée, faute de drapeau
/// « intégré » exposé par amdgpu.
const IGPU_VRAM_LIMIT_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub struct DrmCard {
    /// /sys/class/drm/cardN/device
    pub device: PathBuf,
    /// amdgpu, i915, xe, nouveau…
    pub driver: String,
    pub vendor_id: String,
    pub device_id: String,
}

/// Toutes les cartes réellement présentes. Les entrées `cardN-HDMI-A-1`
/// (connecteurs d'affichage) sont écartées.
pub fn cards() -> Vec<DrmCard> {
    sysfs::list_dir("/sys/class/drm")
        .into_iter()
        .filter_map(|path| {
            let name = sysfs::file_name(&path)?;
            let index = name.strip_prefix("card")?;
            if index.is_empty() || !index.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let device = path.join("device");
            Some(DrmCard {
                vendor_id: sysfs::read_string(device.join("vendor"))?,
                device_id: sysfs::read_string(device.join("device")).unwrap_or_default(),
                driver: sysfs::canonical(device.join("driver"))
                    .and_then(|driver| sysfs::file_name(&driver))
                    .unwrap_or_default(),
                device,
            })
        })
        .collect()
}

impl DrmCard {
    pub fn hwmon(&self) -> Option<PathBuf> {
        sysfs::list_dir(self.device.join("hwmon"))
            .into_iter()
            .next()
    }

    pub fn temperature_celsius(&self) -> Option<f32> {
        sysfs::hwmon_temp_by_label(&self.hwmon()?, &["edge", "junction"])
    }

    /// freq1_input est exprimé en Hz par les pilotes DRM.
    pub fn frequency_mhz(&self) -> Option<u64> {
        sysfs::read_u64(self.hwmon()?.join("freq1_input")).map(|hz| hz / 1_000_000)
    }

    /// Puissance instantanée, publiée en microwatts par hwmon.
    pub fn power_watts(&self) -> Option<f32> {
        sysfs::read_u64(self.hwmon()?.join("power1_average"))
            .map(|microwatts| microwatts as f32 / 1_000_000.0)
    }

    pub fn usage_percent(&self) -> Option<f32> {
        sysfs::read_u64(self.device.join("gpu_busy_percent")).map(|busy| busy as f32)
    }

    pub fn vram_total_bytes(&self) -> Option<u64> {
        sysfs::read_u64(self.device.join("mem_info_vram_total"))
            .or_else(|| sysfs::read_u64(self.device.join("lmem_total_bytes")))
    }

    pub fn vram_used_bytes(&self) -> Option<u64> {
        sysfs::read_u64(self.device.join("mem_info_vram_used"))
    }

    /// Mémoire système partagée (GTT), le « VRAM » des puces intégrées.
    pub fn shared_memory_bytes(&self) -> Option<u64> {
        sysfs::read_u64(self.device.join("mem_info_gtt_total"))
    }

    pub fn is_integrated(&self) -> bool {
        match self.vendor_id.as_str() {
            VENDOR_AMD => self
                .vram_total_bytes()
                .is_none_or(|bytes| bytes < IGPU_VRAM_LIMIT_BYTES),
            // Une carte Intel dédiée (Arc) déclare sa mémoire locale ; un iGPU non.
            VENDOR_INTEL => self.vram_total_bytes().is_none(),
            _ => false,
        }
    }

    /// Nom commercial, résolu depuis la base pci.ids du système quand elle est
    /// disponible. À défaut, on affiche les identifiants bruts plutôt que
    /// d'inventer un modèle.
    pub fn model(&self) -> String {
        pci_name(&self.vendor_id, &self.device_id).unwrap_or_else(|| {
            // Sans base pci.ids, on affiche le pilote et l'identifiant
            // réels plutôt qu'un modèle inventé.
            format!("GPU {} {}", self.driver, self.device_id)
        })
    }
}

/// Emplacements habituels de la base pci.ids selon les distributions.
const PCI_IDS_PATHS: &[&str] = &[
    "/usr/share/hwdata/pci.ids",
    "/usr/share/misc/pci.ids",
    "/usr/share/pci.ids",
];

/// Traduit « 0x1002 » / « 0x7550 » en nom lisible. Le format de pci.ids est :
/// un identifiant de constructeur en début de ligne, puis ses périphériques
/// indentés d'une tabulation.
fn pci_name(vendor_id: &str, device_id: &str) -> Option<String> {
    let vendor = vendor_id.trim_start_matches("0x").to_lowercase();
    let device = device_id.trim_start_matches("0x").to_lowercase();
    if vendor.is_empty() || device.is_empty() {
        return None;
    }

    let content = PCI_IDS_PATHS
        .iter()
        .find_map(|path| std::fs::read_to_string(path).ok())?;

    let mut in_vendor = false;
    for line in content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !line.starts_with('\t') {
            if in_vendor {
                return None; // constructeur suivant : le périphérique est absent
            }
            in_vendor = line.starts_with(&vendor);
            continue;
        }
        if in_vendor && !line.starts_with("\t\t") {
            let entry = line.trim_start();
            if let Some(name) = entry.strip_prefix(&device) {
                return Some(name.trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumeration_ne_panique_pas() {
        for card in cards() {
            assert!(!card.model().is_empty());
            let _ = card.is_integrated();
            let _ = card.temperature_celsius();
        }
    }

    #[test]
    fn identifiants_vides_sans_correspondance() {
        assert!(pci_name("", "").is_none());
    }
}
