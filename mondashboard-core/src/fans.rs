use std::path::Path;

use crate::sysfs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FanKind {
    CpuCooler,
    CaseFan,
    AioPump,
    AioRadiator,
    GpuFan,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FanStats {
    /// Libellé issu de hwmon (« fan1 », « Pump », « CPU Fan »).
    pub label: String,
    pub kind: FanKind,
    pub rpm: u32,
    pub rpm_min: Option<u32>,
    pub rpm_max: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct CoolingStats {
    pub fans: Vec<FanStats>,
    /// Watercooling uniquement, si le pilote l'expose.
    pub coolant_temp_celsius: Option<f32>,
}

/// Pilotes de carte graphique : leurs ventilateurs sont ceux du GPU.
const GPU_DRIVERS: &[&str] = &["amdgpu", "nouveau", "nvidia", "i915", "xe"];

/// Sources : /sys/class/hwmon/*/fan*_input et fan*_label.
/// Tous les contrôleurs présents sont parcourus : aucun hwmonN n'est supposé,
/// leur numérotation changeant d'un démarrage à l'autre.
pub fn get_cooling_stats() -> CoolingStats {
    let mut fans = Vec::new();
    let mut coolant_temp_celsius = None;

    for hwmon in sysfs::hwmon_dirs() {
        let controller = sysfs::hwmon_name(&hwmon).unwrap_or_default();
        fans.extend(fans_of(&hwmon, &controller));

        if coolant_temp_celsius.is_none() {
            coolant_temp_celsius = sysfs::hwmon_temp_by_label(&hwmon, &[])
                .filter(|_| has_coolant_label(&hwmon))
                .or(coolant_temp_celsius);
        }
    }

    CoolingStats {
        fans,
        coolant_temp_celsius,
    }
}

fn fans_of(hwmon: &Path, controller: &str) -> Vec<FanStats> {
    let mut fans = Vec::new();
    for entry in sysfs::list_dir(hwmon) {
        let Some(name) = sysfs::file_name(&entry) else {
            continue;
        };
        let Some(index) = name
            .strip_prefix("fan")
            .and_then(|rest| rest.strip_suffix("_input"))
        else {
            continue;
        };
        let Some(rpm) = sysfs::read_u64(&entry) else {
            continue;
        };

        let label = sysfs::read_string(hwmon.join(format!("fan{index}_label")))
            .filter(|label| !label.is_empty())
            .unwrap_or_else(|| default_label(controller, index));

        fans.push(FanStats {
            kind: classify(&label, controller),
            label,
            rpm: rpm as u32,
            rpm_min: sysfs::read_u64(hwmon.join(format!("fan{index}_min"))).map(|v| v as u32),
            rpm_max: sysfs::read_u64(hwmon.join(format!("fan{index}_max"))).map(|v| v as u32),
        });
    }
    fans
}

/// Sans libellé fourni par le pilote, on nomme le ventilateur d'après son
/// contrôleur — « amdgpu fan1 » reste plus parlant que « fan1 ».
fn default_label(controller: &str, index: &str) -> String {
    if controller.is_empty() {
        format!("fan{index}")
    } else {
        format!("{controller} fan{index}")
    }
}

/// Le type se déduit du libellé du pilote, puis du contrôleur.
fn classify(label: &str, controller: &str) -> FanKind {
    let label = label.to_ascii_lowercase();
    if label.contains("pump") || label.contains("pompe") {
        return FanKind::AioPump;
    }
    if label.contains("rad") {
        return FanKind::AioRadiator;
    }
    if GPU_DRIVERS.contains(&controller) || label.contains("gpu") {
        return FanKind::GpuFan;
    }
    if label.contains("cpu") {
        return FanKind::CpuCooler;
    }
    if label.contains("case") || label.contains("sys") || label.contains("chassis") {
        return FanKind::CaseFan;
    }
    FanKind::Unknown
}

fn has_coolant_label(hwmon: &Path) -> bool {
    sysfs::list_dir(hwmon)
        .iter()
        .filter_map(|entry| sysfs::file_name(entry))
        .filter(|name| name.starts_with("temp") && name.ends_with("_label"))
        .filter_map(|name| sysfs::read_string(hwmon.join(name)))
        .any(|label| {
            let label = label.to_ascii_lowercase();
            label.contains("liquid") || label.contains("coolant") || label.contains("water")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collecte_ne_panique_pas_sans_ventilateur() {
        let stats = get_cooling_stats();
        assert!(stats.fans.iter().all(|fan| !fan.label.is_empty()));
    }

    #[test]
    fn classement_par_libelle() {
        assert_eq!(classify("Pump", "nct6798"), FanKind::AioPump);
        assert_eq!(classify("CPU Fan", "nct6798"), FanKind::CpuCooler);
        assert_eq!(classify("fan1", "amdgpu"), FanKind::GpuFan);
        assert_eq!(classify("fan2", "nct6798"), FanKind::Unknown);
    }

    #[test]
    fn libelle_par_defaut_mentionne_le_controleur() {
        assert_eq!(default_label("amdgpu", "1"), "amdgpu fan1");
        assert_eq!(default_label("", "1"), "fan1");
    }
}
