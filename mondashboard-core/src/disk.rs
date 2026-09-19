use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use sysinfo::Disks;

use crate::sysfs;

/// /proc/diskstats compte en secteurs de 512 octets, quelle que soit la
/// taille de secteur réelle du disque.
const SECTOR_BYTES: u64 = 512;
const GB: f64 = 1024.0 * 1024.0 * 1024.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskKind {
    Ssd,
    Hdd,
    Nvme,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmartHealth {
    Good,
    Warning,
    Critical,
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct DiskStats {
    pub name: String,
    pub mount_point: String,
    pub kind: DiskKind,
    pub total_gb: f64,
    pub used_gb: f64,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub temperature_celsius: Option<f32>,
    pub smart_health: SmartHealth,
    /// Périphérique physique sous-jacent (« nvme0n1 » pour « nvme0n1p2 »).
    pub device: String,
    /// Compteurs cumulés, conservés pour le calcul différentiel du tick suivant.
    pub read_total_bytes: u64,
    pub write_total_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct AllDiskStats {
    pub disks: Vec<DiskStats>,
    pub sampled_at: Instant,
}

impl Default for AllDiskStats {
    fn default() -> Self {
        Self {
            disks: vec![],
            sampled_at: Instant::now(),
        }
    }
}

/// Requiert le relevé précédent pour calculer les vitesses de lecture/écriture.
pub fn get_disk_stats(previous: &AllDiskStats) -> AllDiskStats {
    let sampled_at = Instant::now();
    let elapsed = sampled_at
        .saturating_duration_since(previous.sampled_at)
        .as_secs_f64();

    let counters = read_diskstats();
    let block_devices = block_devices();
    let mut disks: Vec<DiskStats> = Vec::new();

    // Un même volume peut être monté plusieurs fois (sous-volumes btrfs,
    // montages liés). On garde le point de montage le plus court, qui est le
    // plus parlant : « / » plutôt que « /var/log ».
    let system_disks = Disks::new_with_refreshed_list();
    let mut partitions: Vec<_> = system_disks.list().iter().collect();
    partitions.sort_by_key(|disk| disk.mount_point().as_os_str().len());

    for disk in partitions {
        let total_gb = disk.total_space() as f64 / GB;
        if total_gb <= 0.0 {
            continue;
        }

        let name = disk.name().to_string_lossy().to_string();
        if disks.iter().any(|known| known.name == name) {
            continue;
        }
        let device = physical_device(&name, &block_devices).unwrap_or_default();
        let (read_total_bytes, write_total_bytes) =
            counters.get(&device).copied().unwrap_or((0, 0));
        let previous_disk = previous.disks.iter().find(|d| d.name == name);

        disks.push(DiskStats {
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            kind: disk_kind(&device),
            total_gb,
            used_gb: (disk.total_space().saturating_sub(disk.available_space())) as f64 / GB,
            read_bytes_per_sec: rate(
                read_total_bytes,
                previous_disk.map(|d| d.read_total_bytes),
                elapsed,
            ),
            write_bytes_per_sec: rate(
                write_total_bytes,
                previous_disk.map(|d| d.write_total_bytes),
                elapsed,
            ),
            temperature_celsius: temperature(&device),
            smart_health: smart_health(&device),
            name,
            device,
            read_total_bytes,
            write_total_bytes,
        });
    }

    AllDiskStats { disks, sampled_at }
}

fn rate(current_total: u64, previous_total: Option<u64>, elapsed_secs: f64) -> u64 {
    let Some(previous_total) = previous_total else {
        return 0;
    };
    if elapsed_secs <= 0.0 || current_total < previous_total {
        return 0;
    }
    ((current_total - previous_total) as f64 / elapsed_secs) as u64
}

/// Périphériques physiques listés par le noyau, hors pseudo-disques.
fn block_devices() -> Vec<String> {
    sysfs::list_dir("/sys/block")
        .iter()
        .filter_map(|path| sysfs::file_name(path))
        .filter(|name| !name.starts_with("loop") && !name.starts_with("zram"))
        .collect()
}

/// Remonte d'une partition vers son disque : « /dev/nvme0n1p2 » → « nvme0n1 ».
/// On interroge le noyau (présence de /sys/block/<disque>/<partition>) au lieu
/// de deviner le suffixe, qui diffère entre NVMe, SATA et mmc.
fn physical_device(partition_path: &str, block_devices: &[String]) -> Option<String> {
    let partition = partition_path.rsplit('/').next()?;
    if block_devices.iter().any(|device| device == partition) {
        return Some(partition.to_string());
    }
    block_devices
        .iter()
        .find(|device| {
            Path::new("/sys/block")
                .join(device)
                .join(partition)
                .exists()
        })
        .cloned()
}

/// Lit les compteurs cumulés de /proc/diskstats, convertis en octets.
fn read_diskstats() -> HashMap<String, (u64, u64)> {
    let mut counters = HashMap::new();
    let Some(content) = sysfs::read_string("/proc/diskstats") else {
        return counters;
    };
    for line in content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 10 {
            continue;
        }
        let sectors_read: u64 = fields[5].parse().unwrap_or(0);
        let sectors_written: u64 = fields[9].parse().unwrap_or(0);
        counters.insert(
            fields[2].to_string(),
            (sectors_read * SECTOR_BYTES, sectors_written * SECTOR_BYTES),
        );
    }
    counters
}

/// Le noyau indique lui-même si le disque tourne ; le NVMe se reconnaît à son
/// sous-système, pas à son nom.
fn disk_kind(device: &str) -> DiskKind {
    if device.is_empty() {
        return DiskKind::Unknown;
    }
    let base = Path::new("/sys/block").join(device);
    if sysfs::canonical(base.join("device/subsystem"))
        .and_then(|path| sysfs::file_name(&path))
        .as_deref()
        == Some("nvme")
    {
        return DiskKind::Nvme;
    }
    match sysfs::read_u64(base.join("queue/rotational")) {
        Some(1) => DiskKind::Hdd,
        Some(0) => DiskKind::Ssd,
        _ => DiskKind::Unknown,
    }
}

/// Relie un hwmon à son disque en comparant les chemins réels, ce qui couvre
/// aussi bien le capteur NVMe intégré que `drivetemp` sur SATA.
fn temperature(device: &str) -> Option<f32> {
    let disk_device = sysfs::canonical(Path::new("/sys/block").join(device).join("device"))?;
    for hwmon in sysfs::hwmon_dirs() {
        let Some(hwmon_device) = sysfs::canonical(hwmon.join("device")) else {
            continue;
        };
        if (hwmon_device == disk_device || hwmon_device.starts_with(&disk_device))
            && let Some(value) = sysfs::hwmon_temp_by_label(&hwmon, &["Composite"])
        {
            return Some(value);
        }
    }
    None
}

// --- SMART en arrière-plan ------------------------------------------------
//
// `smartctl` lance une commande externe et demande en général les droits
// root : l'appeler dans la boucle GTK la bloquerait. Un thread rafraîchit un
// cache toutes les 60 s ; sans smartmontools, l'état reste « indisponible ».

static SMART_CACHE: OnceLock<Mutex<HashMap<String, SmartHealth>>> = OnceLock::new();
static SMART_STARTED: OnceLock<()> = OnceLock::new();

fn smart_cache() -> &'static Mutex<HashMap<String, SmartHealth>> {
    SMART_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Démarre la surveillance SMART. Appelée une seule fois au lancement.
pub fn start_smart_monitor() {
    if SMART_STARTED.set(()).is_err() {
        return;
    }
    thread::spawn(|| {
        loop {
            let measured: HashMap<String, SmartHealth> = block_devices()
                .into_iter()
                .map(|device| {
                    let health = query_smart(&device);
                    (device, health)
                })
                .collect();
            if let Ok(mut cache) = smart_cache().lock() {
                *cache = measured;
            }
            thread::sleep(Duration::from_secs(60));
        }
    });
}

fn smart_health(device: &str) -> SmartHealth {
    smart_cache()
        .lock()
        .ok()
        .and_then(|cache| cache.get(device).cloned())
        .unwrap_or(SmartHealth::Unavailable)
}

fn query_smart(device: &str) -> SmartHealth {
    let path = PathBuf::from("/dev").join(device);
    let Ok(output) = Command::new("smartctl")
        .arg("-H")
        .arg("-j")
        .arg(&path)
        .output()
    else {
        // smartmontools n'est pas installé : ce n'est pas une panne disque.
        return SmartHealth::Unavailable;
    };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        return SmartHealth::Unavailable;
    };
    match json
        .get("smart_status")
        .and_then(|status| status.get("passed"))
        .and_then(|passed| passed.as_bool())
    {
        Some(true) => SmartHealth::Good,
        Some(false) => SmartHealth::Critical,
        None => SmartHealth::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_remonte_vers_son_disque() {
        let devices = vec!["nvme0n1".to_string(), "sda".to_string()];
        assert_eq!(
            physical_device("/dev/nvme0n1", &devices),
            Some("nvme0n1".to_string())
        );
    }

    #[test]
    fn disque_inconnu_ne_panique_pas() {
        assert!(physical_device("/dev/inexistant9", &[]).is_none());
        assert_eq!(disk_kind("inexistant9"), DiskKind::Unknown);
        assert!(temperature("inexistant9").is_none());
    }

    #[test]
    fn smart_indisponible_par_defaut() {
        assert_eq!(smart_health("inexistant9"), SmartHealth::Unavailable);
    }

    #[test]
    fn collecte_ne_panique_pas() {
        let first = get_disk_stats(&AllDiskStats::default());
        let _ = get_disk_stats(&first);
    }

    #[test]
    fn un_volume_n_apparait_qu_une_fois() {
        let stats = get_disk_stats(&AllDiskStats::default());
        let mut noms: Vec<&str> = stats.disks.iter().map(|d| d.name.as_str()).collect();
        let total = noms.len();
        noms.sort_unstable();
        noms.dedup();
        assert_eq!(
            noms.len(),
            total,
            "un volume monté plusieurs fois est listé en double"
        );
    }
}
