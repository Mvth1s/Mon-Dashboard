use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use sysinfo::Networks;

#[derive(Debug, Clone)]
pub struct NetworkInterfaceStats {
    pub name: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub ip_local: Option<String>,
    /// Compteurs cumulés, conservés pour le calcul différentiel du tick suivant.
    pub rx_total_bytes: u64,
    pub tx_total_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub interfaces: Vec<NetworkInterfaceStats>,
    pub active_interface: Option<String>,
    pub ping_ms: Option<f32>,
    pub ping_host: String,
    /// Horodatage de la mesure : les débits se calculent sur l'écart réel
    /// entre deux relevés, pas sur l'intervalle théorique du minuteur.
    pub sampled_at: Instant,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            interfaces: vec![],
            active_interface: None,
            ping_ms: None,
            ping_host: "1.1.1.1".to_string(),
            sampled_at: Instant::now(),
        }
    }
}

/// Requiert le relevé précédent pour calculer les débits par différence.
pub fn get_network_stats(previous: &NetworkStats) -> NetworkStats {
    let sampled_at = Instant::now();
    let elapsed = sampled_at
        .saturating_duration_since(previous.sampled_at)
        .as_secs_f64();

    let networks = Networks::new_with_refreshed_list();
    let mut interfaces = Vec::new();

    for (name, data) in &networks {
        // La boucle locale n'est pas du trafic réseau utile.
        if name == "lo" {
            continue;
        }

        let rx_total_bytes = data.total_received();
        let tx_total_bytes = data.total_transmitted();

        // Docker, les machines virtuelles et les VPN créent des interfaces
        // qui encombrent la liste sans rien transporter. On ne les montre que
        // si elles ont réellement servi.
        if est_virtuelle(name) && rx_total_bytes == 0 && tx_total_bytes == 0 {
            continue;
        }
        let previous_interface = previous.interfaces.iter().find(|i| &i.name == name);

        interfaces.push(NetworkInterfaceStats {
            name: name.clone(),
            rx_bytes_per_sec: rate(
                rx_total_bytes,
                previous_interface.map(|i| i.rx_total_bytes),
                elapsed,
            ),
            tx_bytes_per_sec: rate(
                tx_total_bytes,
                previous_interface.map(|i| i.tx_total_bytes),
                elapsed,
            ),
            ip_local: first_ipv4(data),
            rx_total_bytes,
            tx_total_bytes,
        });
    }

    interfaces.sort_by_key(|interface| std::cmp::Reverse(interface.rx_total_bytes));

    NetworkStats {
        active_interface: active_interface(&interfaces),
        interfaces,
        ping_ms: last_ping_ms(),
        ping_host: previous.ping_host.clone(),
        sampled_at,
    }
}

/// Interfaces créées par un logiciel plutôt que par du matériel.
fn est_virtuelle(nom: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "docker", "veth", "br-", "virbr", "vmnet", "tap", "tun", "wg", "zt", "ham",
    ];
    PREFIXES.iter().any(|prefixe| nom.starts_with(prefixe))
}

/// Débit par seconde. Renvoie 0 au premier relevé ou si le compteur a été
/// remis à zéro (interface redémarrée), plutôt qu'une valeur absurde.
fn rate(current_total: u64, previous_total: Option<u64>, elapsed_secs: f64) -> u64 {
    let Some(previous_total) = previous_total else {
        return 0;
    };
    if elapsed_secs <= 0.0 || current_total < previous_total {
        return 0;
    }
    ((current_total - previous_total) as f64 / elapsed_secs) as u64
}

fn first_ipv4(data: &sysinfo::NetworkData) -> Option<String> {
    data.ip_networks()
        .iter()
        .find(|network| matches!(network.addr, IpAddr::V4(_)))
        .map(|network| network.addr.to_string())
}

/// L'interface active est celle qui a une IPv4 et le plus de trafic reçu.
/// Aucune interface n'est supposée s'appeler eth0 ou wlan0.
fn active_interface(interfaces: &[NetworkInterfaceStats]) -> Option<String> {
    interfaces
        .iter()
        .filter(|i| i.ip_local.is_some())
        .max_by_key(|i| i.rx_total_bytes)
        .or_else(|| interfaces.first())
        .map(|i| i.name.clone())
}

// --- Ping en arrière-plan -------------------------------------------------
//
// `ping` bloque jusqu'à une seconde. L'appeler depuis un tick de la boucle GTK
// figerait l'interface, donc un thread dédié entretient une valeur en cache
// que la boucle lit instantanément.

static PING_CACHE: OnceLock<Mutex<Option<f32>>> = OnceLock::new();
static PING_STARTED: OnceLock<()> = OnceLock::new();

fn ping_cache() -> &'static Mutex<Option<f32>> {
    PING_CACHE.get_or_init(|| Mutex::new(None))
}

/// Démarre la surveillance du ping. Appelée une seule fois au lancement ;
/// les appels suivants sont sans effet.
pub fn start_ping_monitor(host: &str) {
    if PING_STARTED.set(()).is_err() {
        return;
    }
    let host = host.to_string();
    thread::spawn(move || {
        loop {
            let measured = run_ping(&host);
            if let Ok(mut cache) = ping_cache().lock() {
                *cache = measured;
            }
            thread::sleep(Duration::from_secs(5));
        }
    });
}

/// Dernier ping mesuré. `None` tant qu'aucune mesure n'a abouti (hors ligne,
/// ICMP filtré, `ping` absent).
pub fn last_ping_ms() -> Option<f32> {
    ping_cache().lock().ok().and_then(|cache| *cache)
}

/// Port utilisé par la mesure de repli : HTTPS est ouvert partout.
const PORT_REPLI: u16 = 443;

fn run_ping(host: &str) -> Option<f32> {
    ping_icmp(host).or_else(|| ping_tcp(host))
}

fn ping_icmp(host: &str) -> Option<f32> {
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "2", "-n", host])
        // Sans cela, `ping` traduit sa sortie selon la langue du système
        // (« temps= » en français) et la mesure devient introuvable.
        .env("LC_ALL", "C")
        .output()
        .ok()?;
    parse_ping(&String::from_utf8_lossy(&output.stdout))
}

/// Repli lorsque la commande `ping` est absente, ce qui est le cas dans le
/// bac à sable Flatpak : on mesure le temps d'établissement d'une connexion
/// TCP. La valeur est un peu supérieure à un ICMP (poignée de main en plus)
/// mais reflète la même latence réseau.
fn ping_tcp(host: &str) -> Option<f32> {
    let adresse: SocketAddr = (host, PORT_REPLI).to_socket_addrs().ok()?.next()?;
    let debut = Instant::now();
    TcpStream::connect_timeout(&adresse, Duration::from_secs(2)).ok()?;
    Some(debut.elapsed().as_secs_f32() * 1000.0)
}

/// Extrait la latence de la sortie de `ping`, au format « time=4.76 ms ».
fn parse_ping(sortie: &str) -> Option<f32> {
    let position = sortie.find("time=")?;
    sortie[position + 5..]
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interfaces_virtuelles_reconnues() {
        assert!(est_virtuelle("docker0"));
        assert!(est_virtuelle("virbr0"));
        assert!(est_virtuelle("veth1a2b3c"));
        assert!(!est_virtuelle("wlan0"));
        assert!(!est_virtuelle("enp11s0"));
        // Une interface sans fil ne doit pas être confondue avec un tunnel.
        assert!(!est_virtuelle("wlp3s0"));
    }

    #[test]
    fn hote_par_defaut() {
        assert_eq!(NetworkStats::default().ping_host, "1.1.1.1");
    }

    #[test]
    fn premier_releve_ne_donne_pas_de_debit() {
        assert_eq!(rate(1_000, None, 2.0), 0);
    }

    #[test]
    fn compteur_remis_a_zero_ne_donne_pas_de_debit_absurde() {
        assert_eq!(rate(10, Some(1_000_000), 2.0), 0);
    }

    #[test]
    fn debit_calcule_sur_le_temps_ecoule() {
        assert_eq!(rate(3_000, Some(1_000), 2.0), 1_000);
    }

    #[test]
    fn repli_tcp_sur_hote_injoignable() {
        // Adresse réservée à la documentation : la connexion ne peut aboutir,
        // et la mesure doit renvoyer None sans bloquer indéfiniment.
        assert!(ping_tcp("192.0.2.1").is_none());
    }

    #[test]
    fn lecture_de_la_latence() {
        let sortie = "64 bytes from 1.1.1.1: icmp_seq=1 ttl=56 time=4.76 ms";
        assert_eq!(parse_ping(sortie), Some(4.76));
    }

    #[test]
    fn sortie_traduite_ou_illisible_donne_none() {
        // La locale C est imposée à l'appel ; si malgré tout la sortie change,
        // on renvoie None plutôt qu'une valeur fausse.
        assert_eq!(parse_ping("64 octets de 1.1.1.1 : temps=4.76 ms"), None);
        assert_eq!(parse_ping(""), None);
    }

    #[test]
    fn collecte_ne_panique_pas_sans_reseau() {
        let first = get_network_stats(&NetworkStats::default());
        let _ = get_network_stats(&first);
    }
}
