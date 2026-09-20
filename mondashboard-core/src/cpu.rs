use sysinfo::System;

use crate::sysfs;

#[derive(Debug, Clone)]
pub struct CpuStats {
    pub model: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub global_usage: f32,
    pub per_core_usage: Vec<f32>,
    pub frequency_mhz: u64,
    pub frequency_max_mhz: u64,
    pub temperature_celsius: Option<f32>,
    /// Charge moyenne sur 1, 5 et 15 minutes (/proc/loadavg).
    pub load_average: [f64; 3],
}

/// Contrôleurs hwmon exposant la température CPU, par ordre de préférence.
/// La liste couvre AMD (k10temp, zenpower) et Intel (coretemp), avec les
/// capteurs génériques en dernier recours. Le contrôleur présent est détecté
/// à l'exécution : rien n'est supposé sur le constructeur.
const CPU_HWMON_NAMES: &[&str] = &["k10temp", "zenpower", "coretemp", "cpu_thermal", "acpitz"];

/// Libellés de capteur préférés à l'intérieur du contrôleur retenu.
const CPU_TEMP_LABELS: &[&str] = &["Tctl", "Tdie", "Package id 0", "CPU"];

pub fn get_cpu_stats(sys: &System) -> CpuStats {
    let cpus = sys.cpus();
    let model = cpus
        .first()
        .map(|cpu| cpu.brand().trim().to_string())
        .filter(|brand| !brand.is_empty())
        .unwrap_or_else(|| "Processeur inconnu".to_string());

    let frequency_mhz = if cpus.is_empty() {
        0
    } else {
        cpus.iter().map(|cpu| cpu.frequency()).sum::<u64>() / cpus.len() as u64
    };

    CpuStats {
        model,
        physical_cores: sys.physical_core_count().unwrap_or(cpus.len()) as u32,
        logical_cores: cpus.len() as u32,
        global_usage: sys.global_cpu_usage(),
        per_core_usage: cpus.iter().map(|cpu| cpu.cpu_usage()).collect(),
        frequency_mhz,
        frequency_max_mhz: max_frequency_mhz(),
        temperature_celsius: temperature(),
        load_average: {
            let charge = System::load_average();
            [charge.one, charge.five, charge.fifteen]
        },
    }
}

/// Fréquence maximale annoncée par cpufreq (en kHz dans sysfs).
/// Renvoie 0 si cpufreq est absent (machine virtuelle, noyau sans pilote).
fn max_frequency_mhz() -> u64 {
    sysfs::list_dir("/sys/devices/system/cpu")
        .into_iter()
        .filter_map(|cpu| sysfs::read_u64(cpu.join("cpufreq/cpuinfo_max_freq")))
        .max()
        .map(|khz| khz / 1000)
        .unwrap_or(0)
}

/// Cherche le premier contrôleur connu réellement présent sur la machine.
fn temperature() -> Option<f32> {
    for wanted in CPU_HWMON_NAMES {
        for dir in sysfs::hwmon_dirs_named(wanted) {
            if let Some(value) = sysfs::hwmon_temp_by_label(&dir, CPU_TEMP_LABELS) {
                return Some(value);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collecte_coherente_sur_la_machine_courante() {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        let stats = get_cpu_stats(&sys);

        assert!(stats.logical_cores >= 1);
        assert_eq!(stats.per_core_usage.len(), stats.logical_cores as usize);
        assert!(stats.global_usage >= 0.0 && stats.global_usage <= 100.0);
    }

    #[test]
    fn temperature_absente_ne_panique_pas() {
        // Sur une machine sans capteur (CI), le résultat est None, pas une erreur.
        let _ = temperature();
    }
}
