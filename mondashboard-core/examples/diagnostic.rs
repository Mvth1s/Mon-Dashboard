//! Affiche tout ce que MonDashboard détecte sur la machine courante.
//!
//! Utile pour vérifier la détection matérielle et pour joindre un état des
//! lieux à un rapport de bug :
//!
//! ```sh
//! cargo run -p mondashboard-core --example diagnostic
//! ```

use std::thread::sleep;
use std::time::Duration;

use mondashboard_core::{
    battery, config::BatterySource, cpu, disk, fans, gpu, memory, network, process,
};
use sysinfo::{ProcessesToUpdate, System};

fn main() {
    let mut sys = System::new_all();
    // Un premier relevé sert de référence : les pourcentages CPU et les
    // débits se calculent toujours entre deux mesures.
    let premier_reseau = network::get_network_stats(&network::NetworkStats::default());
    let premiers_disques = disk::get_disk_stats(&disk::AllDiskStats::default());
    sleep(Duration::from_secs(1));
    sys.refresh_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let cpu = cpu::get_cpu_stats(&sys);
    println!("== Processeur ==");
    println!(
        "  {} ({} cœurs / {} threads)",
        cpu.model, cpu.physical_cores, cpu.logical_cores
    );
    println!(
        "  charge {:.1} % — {} MHz (max {} MHz)",
        cpu.global_usage, cpu.frequency_mhz, cpu.frequency_max_mhz
    );
    println!("  température {}", option_temp(cpu.temperature_celsius));

    let memoire = memory::get_memory_stats(&sys);
    println!("\n== Mémoire ==");
    println!(
        "  {} Mo utilisés / {} Mo — cache {} Mo",
        memoire.used_mb, memoire.total_mb, memoire.cached_mb
    );
    println!(
        "  swap {} Mo / {} Mo",
        memoire.swap_used_mb, memoire.swap_total_mb
    );

    println!("\n== Cartes graphiques ==");
    let gpus = gpu::get_gpu_stats();
    if gpus.gpus.is_empty() {
        println!("  aucune carte détectée");
    }
    for carte in &gpus.gpus {
        println!("  [{:?}] {}", carte.vendor, carte.model);
        println!(
            "    charge {} — température {}",
            option_percent(carte.usage_percent),
            option_temp(carte.temperature_celsius)
        );
        match (carte.vram_used_mb, carte.vram_total_mb) {
            (Some(utilisee), Some(totale)) => println!("    VRAM {utilisee} Mo / {totale} Mo"),
            _ => println!("    mémoire partagée {}", option_mb(carte.shared_memory_mb)),
        }
    }

    println!("\n== Réseau ==");
    let reseau = network::get_network_stats(&premier_reseau);
    for interface in &reseau.interfaces {
        println!(
            "  {} — {} ↓ / {} ↑ — IP {}",
            interface.name,
            debit(interface.rx_bytes_per_sec),
            debit(interface.tx_bytes_per_sec),
            interface.ip_local.clone().unwrap_or_else(|| "—".into())
        );
    }
    println!(
        "  interface active : {}",
        reseau.active_interface.unwrap_or_else(|| "—".into())
    );

    println!("\n== Stockage ==");
    for d in &disk::get_disk_stats(&premiers_disques).disks {
        println!(
            "  {} ({}) sur {} — {:?}",
            d.name, d.device, d.mount_point, d.kind
        );
        println!(
            "    {:.1} Go / {:.1} Go — lecture {} / écriture {}",
            d.used_gb,
            d.total_gb,
            debit(d.read_bytes_per_sec),
            debit(d.write_bytes_per_sec)
        );
        println!(
            "    température {} — SMART {:?}",
            option_temp(d.temperature_celsius),
            d.smart_health
        );
    }

    println!("\n== Refroidissement ==");
    let refroidissement = fans::get_cooling_stats();
    if refroidissement.fans.is_empty() {
        println!("  aucun ventilateur exposé par le noyau");
    }
    for ventilateur in &refroidissement.fans {
        println!(
            "  {} [{:?}] — {} tr/min",
            ventilateur.label, ventilateur.kind, ventilateur.rpm
        );
    }

    println!("\n== Batterie ==");
    for source in [BatterySource::Sysfs, BatterySource::UPower] {
        let batterie = battery::collect(source);
        if batterie.present {
            println!(
                "  [{source:?}] {:.0} % — {:?} — santé {:.0} % — {} / {}",
                batterie.percentage,
                batterie.state,
                batterie.health_percent,
                batterie.vendor.clone().unwrap_or_else(|| "—".into()),
                batterie.model.clone().unwrap_or_else(|| "—".into())
            );
        } else {
            println!("  [{source:?}] aucune batterie système");
        }
    }

    println!("\n== Processus (top 5) ==");
    for p in process::get_process_stats(&sys).processes.iter().take(5) {
        println!(
            "  {:>7} {:<24} {:>5.1} % CPU {:>6} Mo  {}",
            p.pid,
            tronque(&p.name, 24),
            p.cpu_percent,
            p.memory_mb,
            p.user
        );
    }
}

fn option_temp(valeur: Option<f32>) -> String {
    valeur
        .map(|v| format!("{v:.1} °C"))
        .unwrap_or_else(|| "—".into())
}

fn option_percent(valeur: Option<f32>) -> String {
    valeur
        .map(|v| format!("{v:.0} %"))
        .unwrap_or_else(|| "—".into())
}

fn option_mb(valeur: Option<u64>) -> String {
    valeur
        .map(|v| format!("{v} Mo"))
        .unwrap_or_else(|| "—".into())
}

fn debit(octets_par_sec: u64) -> String {
    let mo = octets_par_sec as f64 / (1024.0 * 1024.0);
    if mo >= 1.0 {
        format!("{mo:.1} Mo/s")
    } else {
        format!("{:.0} ko/s", octets_par_sec as f64 / 1024.0)
    }
}

fn tronque(texte: &str, taille: usize) -> String {
    if texte.len() <= taille {
        texte.to_string()
    } else {
        format!("{}…", &texte[..taille - 1])
    }
}
