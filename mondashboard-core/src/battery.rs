//! Batterie, avec deux sources interchangeables.
//!
//! - `Sysfs` : lecture directe de /sys/class/power_supply. Instantané, sans
//!   service tiers, mais nécessite l'accès à /sys.
//! - `UPower` : le service standard du bureau Linux, via D-Bus. C'est la voie
//!   recommandée sous Flatpak.
//!
//! Le choix se fait par `AppConfig::battery_source` ; `Auto` essaie UPower et
//! retombe sur sysfs si le service est absent.

use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use crate::config::BatterySource;
use crate::sysfs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BatteryStats {
    /// faux sur un ordinateur fixe — le widget est masqué dans ce cas.
    pub present: bool,
    pub percentage: f32,
    pub state: BatteryState,
    pub time_to_empty_min: Option<u32>,
    pub time_to_full_min: Option<u32>,
    pub energy_wh: f64,
    pub energy_full_wh: f64,
    pub energy_full_design_wh: f64,
    /// energy_full / energy_full_design * 100
    pub health_percent: f32,
    pub cycle_count: Option<u32>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub technology: Option<String>,
}

impl Default for BatteryStats {
    fn default() -> Self {
        Self {
            present: false,
            percentage: 0.0,
            state: BatteryState::Unknown,
            time_to_empty_min: None,
            time_to_full_min: None,
            energy_wh: 0.0,
            energy_full_wh: 0.0,
            energy_full_design_wh: 0.0,
            health_percent: 0.0,
            cycle_count: None,
            vendor: None,
            model: None,
            technology: None,
        }
    }
}

static CACHE: OnceLock<Mutex<Option<BatteryStats>>> = OnceLock::new();
static MONITOR_STARTED: OnceLock<()> = OnceLock::new();

fn cache() -> &'static Mutex<Option<BatteryStats>> {
    CACHE.get_or_init(|| Mutex::new(None))
}

/// Démarre le rafraîchissement en arrière-plan. UPower passe par D-Bus, ce
/// qui peut prendre quelques millisecondes : on ne veut pas de cet appel dans
/// un tick de la boucle GTK. Appelée une seule fois au lancement.
pub fn start_monitor(source: BatterySource) {
    if MONITOR_STARTED.set(()).is_err() {
        return;
    }
    thread::spawn(move || {
        loop {
            let measured = collect(source);
            if let Ok(mut cached) = cache().lock() {
                *cached = Some(measured);
            }
            thread::sleep(Duration::from_secs(5));
        }
    });
}

/// Dernier état connu de la batterie. Ne bloque jamais : tant que le thread
/// n'a rien publié, on répond par une lecture sysfs, qui est immédiate.
pub fn get_battery_stats() -> BatteryStats {
    if let Some(cached) = cache().lock().ok().and_then(|cached| cached.clone()) {
        return cached;
    }
    from_sysfs()
}

/// Collecte directe, sans passer par le cache. Utile pour les tests et pour
/// comparer les deux sources.
pub fn collect(source: BatterySource) -> BatteryStats {
    match source {
        BatterySource::Sysfs => from_sysfs(),
        BatterySource::UPower => from_upower().unwrap_or_default(),
        BatterySource::Auto => match from_upower() {
            Some(stats) if stats.present => stats,
            // UPower absent, inaccessible, ou sans batterie à signaler :
            // sysfs tranche, car lui seul distingue « pas de batterie » de
            // « service indisponible ».
            _ => from_sysfs(),
        },
    }
}

// --- Source sysfs ---------------------------------------------------------

/// Vraie batterie système, par opposition aux périphériques sans fil (souris,
/// clavier, casque) qui se déclarent aussi `type=Battery` mais avec
/// `scope=Device`. Sans ce filtre, le niveau de la souris s'afficherait comme
/// celui de l'ordinateur.
fn is_system_battery(dir: &Path) -> bool {
    let is_battery = sysfs::read_string(dir.join("type")).as_deref() == Some("Battery");
    let scope = sysfs::read_string(dir.join("scope"));
    is_battery && scope.as_deref() != Some("Device")
}

fn from_sysfs() -> BatteryStats {
    let Some(dir) = sysfs::list_dir("/sys/class/power_supply")
        .into_iter()
        .find(|dir| is_system_battery(dir))
    else {
        return BatteryStats::default();
    };

    let energy_wh = read_energy_wh(&dir, "energy_now", "charge_now");
    let energy_full_wh = read_energy_wh(&dir, "energy_full", "charge_full");
    let energy_full_design_wh = read_energy_wh(&dir, "energy_full_design", "charge_full_design");
    let power_w = sysfs::read_u64(dir.join("power_now")).map(|micro| micro as f64 / 1_000_000.0);

    let state = match sysfs::read_string(dir.join("status")).as_deref() {
        Some("Charging") => BatteryState::Charging,
        Some("Discharging") => BatteryState::Discharging,
        Some("Full") => BatteryState::Full,
        _ => BatteryState::Unknown,
    };

    BatteryStats {
        present: true,
        percentage: sysfs::read_u64(dir.join("capacity")).unwrap_or(0) as f32,
        time_to_empty_min: match state {
            BatteryState::Discharging => minutes(energy_wh, power_w),
            _ => None,
        },
        time_to_full_min: match state {
            BatteryState::Charging => minutes(energy_full_wh - energy_wh, power_w),
            _ => None,
        },
        state,
        energy_wh,
        energy_full_wh,
        energy_full_design_wh,
        health_percent: health(energy_full_wh, energy_full_design_wh),
        cycle_count: sysfs::read_u64(dir.join("cycle_count"))
            .filter(|count| *count > 0)
            .map(|count| count as u32),
        vendor: sysfs::read_string(dir.join("manufacturer")),
        model: sysfs::read_string(dir.join("model_name")),
        technology: sysfs::read_string(dir.join("technology")),
    }
}

/// Certaines batteries publient des microwattheures (`energy_*`), d'autres des
/// microampèreheures (`charge_*`) qu'il faut multiplier par la tension.
fn read_energy_wh(dir: &Path, energy_file: &str, charge_file: &str) -> f64 {
    if let Some(micro_wh) = sysfs::read_u64(dir.join(energy_file)) {
        return micro_wh as f64 / 1_000_000.0;
    }
    let Some(micro_ah) = sysfs::read_u64(dir.join(charge_file)) else {
        return 0.0;
    };
    let micro_volts = sysfs::read_u64(dir.join("voltage_now"))
        .or_else(|| sysfs::read_u64(dir.join("voltage_min_design")))
        .unwrap_or(0);
    micro_ah as f64 * micro_volts as f64 / 1e12
}

fn minutes(remaining_wh: f64, power_w: Option<f64>) -> Option<u32> {
    let power_w = power_w?;
    if power_w <= 0.0 || remaining_wh <= 0.0 {
        return None;
    }
    Some((remaining_wh / power_w * 60.0) as u32)
}

fn health(full_wh: f64, design_wh: f64) -> f32 {
    if design_wh <= 0.0 {
        return 0.0;
    }
    (full_wh / design_wh * 100.0) as f32
}

// --- Source UPower (D-Bus) ------------------------------------------------

/// Type de périphérique UPower correspondant à une batterie.
const UPOWER_TYPE_BATTERY: u32 = 2;

fn from_upower() -> Option<BatteryStats> {
    runtime().block_on(upower_query()).ok().flatten()
}

/// UPower est asynchrone alors que l'API de collecte est synchrone : on garde
/// un petit runtime dédié, utilisé uniquement par le thread de surveillance.
fn runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("création du runtime tokio pour D-Bus")
    })
}

async fn upower_query() -> zbus::Result<Option<BatteryStats>> {
    let connection = zbus::Connection::system().await?;
    let upower = zbus::Proxy::new(
        &connection,
        "org.freedesktop.UPower",
        "/org/freedesktop/UPower",
        "org.freedesktop.UPower",
    )
    .await?;

    let devices: Vec<zbus::zvariant::OwnedObjectPath> =
        upower.call("EnumerateDevices", &()).await?;

    for path in devices {
        let device = zbus::Proxy::new(
            &connection,
            "org.freedesktop.UPower",
            path,
            "org.freedesktop.UPower.Device",
        )
        .await?;

        // `PowerSupply` distingue la batterie de l'ordinateur de celle d'une
        // souris ou d'un casque, que UPower expose aussi.
        let kind: u32 = device.get_property("Type").await.unwrap_or(0);
        let power_supply: bool = device.get_property("PowerSupply").await.unwrap_or(false);
        if kind != UPOWER_TYPE_BATTERY || !power_supply {
            continue;
        }

        let energy_full_wh: f64 = device.get_property("EnergyFull").await.unwrap_or(0.0);
        let energy_full_design_wh: f64 =
            device.get_property("EnergyFullDesign").await.unwrap_or(0.0);
        let state: u32 = device.get_property("State").await.unwrap_or(0);

        return Ok(Some(BatteryStats {
            present: true,
            percentage: device
                .get_property::<f64>("Percentage")
                .await
                .unwrap_or(0.0) as f32,
            state: match state {
                1 => BatteryState::Charging,
                2 => BatteryState::Discharging,
                4 => BatteryState::Full,
                _ => BatteryState::Unknown,
            },
            time_to_empty_min: positive_minutes(
                device.get_property::<i64>("TimeToEmpty").await.unwrap_or(0),
            ),
            time_to_full_min: positive_minutes(
                device.get_property::<i64>("TimeToFull").await.unwrap_or(0),
            ),
            energy_wh: device.get_property("Energy").await.unwrap_or(0.0),
            energy_full_wh,
            energy_full_design_wh,
            health_percent: health(energy_full_wh, energy_full_design_wh),
            cycle_count: device
                .get_property::<i32>("ChargeCycles")
                .await
                .ok()
                .filter(|count| *count > 0)
                .map(|count| count as u32),
            vendor: non_empty(device.get_property("Vendor").await.ok()),
            model: non_empty(device.get_property("Model").await.ok()),
            technology: device
                .get_property::<u32>("Technology")
                .await
                .ok()
                .and_then(technology_name),
        }));
    }

    Ok(None)
}

/// UPower renvoie 0 quand la durée est inconnue (secteur branché, estimation
/// en cours), et non une absence de valeur.
fn positive_minutes(seconds: i64) -> Option<u32> {
    (seconds > 0).then_some((seconds / 60) as u32)
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.trim().is_empty())
}

fn technology_name(code: u32) -> Option<String> {
    let name = match code {
        1 => "Li-ion",
        2 => "Li-polymère",
        3 => "LiFePO4",
        4 => "Plomb",
        5 => "Ni-Cd",
        6 => "Ni-MH",
        _ => return None,
    };
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sans_batterie_le_widget_est_masque() {
        // Sur un fixe, `present` doit être faux sans que rien ne panique.
        let stats = from_sysfs();
        if !stats.present {
            assert_eq!(stats.percentage, 0.0);
        }
    }

    #[test]
    fn batterie_de_peripherique_ignoree() {
        // Une souris sans fil se déclare Battery mais avec scope=Device.
        let mouse = Path::new("/sys/class/power_supply/hidpp_battery_0");
        if mouse.exists() {
            assert!(!is_system_battery(mouse));
        }
    }

    #[test]
    fn sante_sans_valeur_de_conception() {
        assert_eq!(health(50.0, 0.0), 0.0);
        assert_eq!(health(45.0, 50.0), 90.0);
    }

    #[test]
    fn duree_inconnue_donne_none() {
        assert_eq!(positive_minutes(0), None);
        assert_eq!(positive_minutes(-1), None);
        assert_eq!(positive_minutes(3600), Some(60));
    }
}
