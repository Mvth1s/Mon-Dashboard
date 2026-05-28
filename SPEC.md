# MonDashboard — Spécification Technique

> Application Linux de monitoring système, configurable, distribuée via Flatpak.

---

## 1. Vision & Objectifs

### Vision

MonDashboard est une application Linux de monitoring système pensée pour être **comprise au premier regard**, sans apprentissage préalable. L'utilisateur doit pouvoir ouvrir l'app et savoir immédiatement l'état de sa machine — sans chercher, sans décoder, sans lire de documentation.

L'application s'adresse autant au débutant qui veut juste savoir "pourquoi mon PC rame" qu'à l'utilisateur avancé qui surveille sa consommation en temps réel pendant une session de jeu ou une inférence d'IA locale.

### Problème résolu

Les outils existants comme `htop` ou `btop` sont puissants mais conçus pour des utilisateurs techniques. Leur interface en terminal, leurs raccourcis clavier et leur densité d'information les rendent inaccessibles à un public non averti. À l'inverse, les widgets de bureau (Conky, KDE Plasma widgets) sont peu lisibles et difficiles à configurer.

MonDashboard comble ce vide : une **interface graphique moderne, lisible et configurable**, qui centralise toutes les métriques système sans en sacrifier la complétude.

### Objectifs principaux

- Afficher en un coup d'œil les métriques essentielles (CPU, GPU, RAM, réseau, stockage, processus)
- Être compréhensible sans connaissances techniques préalables (labels clairs, unités explicites, codes couleur)
- Permettre une utilisation **ponctuelle** (ouvrir, lire, fermer) et **continue** (tourner en arrière-plan avec tray icon et mini overlay)
- Être configurable : l'utilisateur choisit ce qu'il voit et comment il le voit
- Être distribuable à tous les utilisateurs Linux via Flatpak sur Flathub

### Ce que MonDashboard n'est pas

- Un remplaçant de `htop` / `btop` pour les power users en terminal
- Un outil de monitoring serveur/réseau (pas de monitoring distant)
- Un outil de logging ou d'historique long terme (pas de base de données)

---

## 2. Public cible & distribution

### Public cible

**Utilisateur principal : le débutant curieux**
Quelqu'un qui utilise Linux au quotidien sans être développeur ou administrateur système. Il veut comprendre pourquoi son PC est lent, combien de RAM consomme son jeu, ou surveiller sa carte graphique pendant une inférence IA locale — sans avoir à apprendre un outil complexe.

**Utilisateur secondaire : l'intermédiaire**
Un utilisateur qui connaît déjà `htop` ou `btop` mais préfère une interface graphique plus lisible et configurable pour un usage quotidien.

MonDashboard ne cible pas les administrateurs système ou les profils DevOps qui ont des besoins de monitoring avancé (alerting distant, métriques longue durée, dashboards multi-machines).

### Distribution

- **Format :** Flatpak exclusivement
- **Store :** Flathub (`flathub.org`) — distribution publique, accessible à tous
- **Compatibilité distros :** Toutes les distributions Linux majeures via le runtime Flatpak (Ubuntu, Fedora, Arch, openSUSE, Debian, Linux Mint, etc.)
- **Runtime Flatpak cible :** `org.gnome.Platform` (pour GTK4 + libadwaita)
- **Permissions Flatpak requises :**
  - `--filesystem=host:ro` — lecture des fichiers système (`/proc`, `/sys`)
  - `--device=all` — accès aux infos GPU
  - `--talk-name=org.freedesktop.Notifications` — notifications système
  - `--system-talk-name=org.freedesktop.UDisks2` — infos disques

### Projet & gouvernance

- Projet **solo** au démarrage, open source sur GitHub
- Contributions externes envisageables à terme (issues, PRs), mais pas de gouvernance formelle dans un premier temps
- Licence : **GPL-3.0** (compatible Flathub, protège le caractère libre du projet)

---

## 3. Fonctionnalités

### 3.1 Widgets de monitoring

Chaque widget est une unité indépendante, affichable ou masquable. Tous les widgets partagent les mêmes conventions d'affichage : labels en français/anglais selon la locale, unités explicites (ex: "Go" et pas "G"), codes couleur (vert / orange / rouge) pour signaler un niveau critique.

---

#### CPU
- Utilisation globale en pourcentage (graphe en temps réel, historique ~60s)
- Utilisation par cœur (barres individuelles)
- Fréquence actuelle et fréquence maximale
- Température(s) via `lm-sensors` (si disponible)
- Modèle du processeur, nombre de cœurs physiques et threads
- Indicateur visuel de charge : bas / modéré / élevé / critique

#### GPU
- Utilisation du GPU en pourcentage
- Mémoire VRAM utilisée / totale (ou mémoire partagée pour les iGPU)
- Température
- Fréquence GPU et mémoire
- Support **AMD discret** via `sysfs` + `rocm-smi` si disponible
- Support **NVIDIA discret** via `nvml-wrapper`
- Support **Intel Arc discret** via `sysfs` (`/sys/class/drm/`)
- Support **iGPU Intel** via `sysfs` (présent sur quasi tous les portables Intel)
- Support **iGPU AMD (APU)** via `sysfs`
- **Multi-GPU** : affichage de chaque GPU dans un sous-widget dédié (ex: iGPU + discret sur portable gaming)
- Dégradation gracieuse si aucun GPU détecté ou driver absent

#### RAM
- Mémoire utilisée / disponible / totale (en Go)
- Swap utilisé / total
- Graphe en temps réel, historique ~60s
- Répartition visuelle (barre segmentée : utilisé / cache / libre)

#### Réseau
- Débit montant et descendant en temps réel par interface (Ko/s, Mo/s)
- Sélecteur d'interface active (Ethernet, Wi-Fi, VPN, loopback masqué)
- Ping vers un hôte configurable (défaut : `1.1.1.1`) avec indicateur de latence
- IP locale affichée
- Graphe de débit temps réel, historique ~60s

#### Stockage
- Liste de toutes les partitions montées avec espace utilisé / total (barre de progression)
- Vitesse de lecture et d'écriture en temps réel (Mo/s)
- Température des disques via `smartctl` (si disponible et permissions ok)
- Indicateur de santé SMART : Bon / Attention / Critique / Non disponible
- Affichage du type de disque (SSD, HDD, NVMe)

#### Ventilateurs & refroidissement
- Vitesse de chaque ventilateur détecté en RPM (ventirads CPU, ventilateurs boîtier)
- **AIO** : vitesse de la pompe (RPM) + vitesse du/des ventilateurs du radiateur séparément
- Température du liquide de refroidissement si exposée par le driver AIO
- Nom/label de chaque capteur tel que fourni par `lm-sensors` (ex: "fan1", "CPU Fan", "Pump")
- Indicateur visuel : arrêté (0 RPM) / bas / normal / élevé
- Widget masqué automatiquement si aucun capteur de ventilateur détecté
#### Batterie *(portables uniquement)*
- Pourcentage de charge actuel
- État : en charge / décharge / plein / inconnu
- Autonomie estimée restante (si disponible via le noyau)
- Santé de la batterie : capacité actuelle vs capacité d'origine (en %)
- Nombre de cycles de charge
- Capacité en Wh (conception vs actuelle)
- Marque et modèle du fabricant
- Technologie (Li-ion, Li-poly, etc.)
- Widget masqué automatiquement si aucune batterie détectée (desktop)
- Toutes les données lues via **UPower** (D-Bus — `org.freedesktop.UPower`), présent sur toutes les distros desktop

#### Processus
- Liste des processus en cours, triés par CPU ou RAM (configurable)
- Colonnes : nom, PID, CPU%, RAM utilisée, utilisateur
- Barre de recherche pour filtrer par nom
- Action : terminer un processus (avec confirmation)
- Widget compact "Top 5" pour affichage en petit format sur le dashboard

---

### 3.2 Dashboard principal

- Grille configurable par drag & drop
- Widgets redimensionnables (tailles prédéfinies : petit / moyen / large / plein)
- Sauvegarde de plusieurs **layouts** nommés (ex: "Bureau", "Gaming", "IA locale")
- Sélection du layout actif depuis la barre supérieure
- Thème clair / sombre (suit le thème système par défaut, modifiable manuellement)

---

### 3.3 Mini overlay

Fenêtre flottante compacte, always-on-top, pour un usage pendant le jeu ou une inférence IA.

- Affiche un sous-ensemble de métriques configurable (ex: CPU%, GPU%, VRAM, RAM)
- Transparence réglable
- Position fixable sur l'écran (coin haut-gauche, haut-droit, etc.)
- Activable via raccourci clavier global ou depuis la tray icon
- Taille compacte non intrusive (~250×150px par défaut)

---

### 3.4 Tray icon

- Icône permanente dans la barre système (indicateur de charge, couleur dynamique)
- Clic gauche : afficher / masquer le dashboard principal
- Clic droit : menu contextuel
  - Afficher le dashboard
  - Activer / désactiver le mini overlay
  - Accéder aux paramètres
  - Quitter MonDashboard
- L'application continue de tourner en arrière-plan quand la fenêtre principale est fermée

---

### 3.5 Notifications & alertes

- Alertes configurables par seuil sur chaque métrique :
  - CPU > X% pendant Y secondes
  - RAM utilisée > X Go ou X%
  - GPU > X°C
  - Disque < X Go libres
  - Ping > X ms
- Notification via `libnotify` (notifications système natives)
- Chaque alerte est activable/désactivable indépendamment
- Cooldown configurable pour éviter le spam (ex: pas plus d'une notif toutes les 5 minutes pour la même alerte)

---

### 3.6 Paramètres

- Intervalle de rafraîchissement global (1s / 2s / 5s)
- Unités : température en °C ou °F, taille en Go/Gio
- Langue de l'interface (selon la locale système, français et anglais au minimum)
- Configuration des alertes
- Hôte de ping personnalisable
- Import / export de la configuration complète (fichier JSON)
- Réinitialisation aux valeurs par défaut

---

## 4. Architecture technique

### 4.1 Principe général

MonDashboard adopte une architecture **deux couches strictement séparées** :

- **Backend (`mondashboard-core`)** : librairie Rust pure, sans dépendance GTK. Responsable de toute la collecte de données système. Réutilisable et testable indépendamment.
- **Frontend (`mondashboard-gtk`)** : binaire GTK4 en Rust. Responsable uniquement de l'affichage. Consomme le backend via des appels directs (même processus).

Cette séparation garantit que chaque couche peut être lue, modifiée et testée isolément — ce qui est essentiel dans un contexte de vibecoding.

```
┌─────────────────────────────────────────┐
│           mondashboard-gtk              │
│  (GTK4 + libadwaita — interface)        │
│                                         │
│  Widgets ──► Polling loop ──► Core API  │
└─────────────────┬───────────────────────┘
                  │ appels directs (Rust)
┌─────────────────▼───────────────────────┐
│           mondashboard-core             │
│  (Rust pur — collecte système)          │
│                                         │
│  cpu  │ gpu  │ memory  │ network        │
│  disk │ process │ alerts               │
└─────────────────────────────────────────┘
```

---

### 4.2 Flux de données

Le frontend tourne une **boucle de polling** via `glib::timeout_add_local` (dans le thread principal GTK). À chaque tick :

1. Appel des fonctions du core (`get_cpu_stats()`, `get_memory_stats()`, etc.)
2. Mise à jour des widgets avec les nouvelles valeurs
3. Vérification des seuils d'alerte
4. Envoi de notification si seuil dépassé et cooldown écoulé

L'intervalle du tick est configurable (1s par défaut). Il n'y a pas de thread séparé ni de channel async dans un premier temps — le core étant non-bloquant, le polling synchrone dans le thread GTK est suffisant.

---

### 4.3 Gestion de la configuration

La configuration (layouts, seuils d'alerte, préférences) est stockée dans un fichier **JSON** à l'emplacement standard XDG :

```
~/.var/app/io.github.<user>.MonDashboard/config/config.json
```

Elle est chargée au démarrage et sauvegardée à chaque modification. La structure est versionnée pour permettre des migrations futures.

---

### 4.4 Dépendances principales

| Crate | Rôle |
|---|---|
| `sysinfo` | CPU, RAM, processus (cross-platform, bien maintenu) |
| `nvml-wrapper` | GPU NVIDIA via NVML |
| `gtk4` + `libadwaita` | Interface graphique, thème adaptatif |
| `serde` + `serde_json` | Sérialisation de la config |
| `notify-rust` | Notifications système via libnotify |
| `log` + `env_logger` | Logging structuré |

Les données GPU AMD sont lues directement depuis `sysfs` (`/sys/class/drm/`) sans dépendance externe, pour éviter d'avoir `rocm-smi` comme prérequis.

---

### 4.5 ID d'application Flatpak

```
io.github.Mvth1s.MonDashboard
```

Cet identifiant sera à fixer définitivement avant la soumission sur Flathub.

---

## 5. Structure du projet

Le projet est un **workspace Cargo** composé de deux crates. Cette organisation permet de compiler et tester le core indépendamment du frontend GTK.

```
mondashboard/
│
├── Cargo.toml                  # Workspace Cargo (membres : core + gtk)
├── Cargo.lock
├── README.md
├── SPEC.md
├── LICENSE                     # GPL-3.0
│
├── mondashboard-core/          # Librairie Rust pure (pas de GTK)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Point d'entrée public de la crate
│       ├── cpu.rs              # Collecte CPU
│       ├── gpu/
│       │   ├── mod.rs          # Interface commune GPU + détection automatique
│       │   ├── amd.rs          # GPU discret AMD via sysfs + rocm-smi
│       │   ├── nvidia.rs       # GPU discret NVIDIA via nvml-wrapper
│       │   ├── intel.rs        # Intel Arc discret + iGPU Intel via sysfs
│       │   ├── amd_igpu.rs     # iGPU AMD (APU) via sysfs
│       │   └── multi.rs        # Gestion multi-GPU (ex: iGPU + discret)
│       ├── memory.rs           # RAM + Swap
│       ├── network.rs          # Interfaces, débit, ping
│       ├── disk.rs             # Partitions, vitesses, SMART
│       ├── process.rs          # Liste et gestion des processus
│       ├── battery.rs          # UPower via D-Bus
│       ├── fans.rs             # Ventilateurs & refroidissement via hwmon
│       ├── alerts.rs           # Logique de seuils et cooldowns
│       └── config.rs           # Structures de configuration (serde)
│
├── mondashboard-gtk/           # Binaire GTK4 (frontend)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # Point d'entrée, init GTK
│       ├── app.rs              # Application GTK (GtkApplication)
│       ├── polling.rs          # Boucle de rafraîchissement (glib::timeout)
│       ├── tray.rs             # Tray icon et menu contextuel
│       ├── overlay.rs          # Fenêtre mini overlay always-on-top
│       ├── settings.rs         # Fenêtre paramètres
│       ├── layout/
│       │   ├── mod.rs          # Gestionnaire de layouts
│       │   ├── grid.rs         # Grille drag & drop
│       │   └── persistence.rs  # Sauvegarde/chargement des layouts
│       └── widgets/
│           ├── mod.rs          # Trait commun Widget
│           ├── cpu.rs          # Widget CPU
│           ├── gpu.rs          # Widget GPU
│           ├── memory.rs       # Widget RAM
│           ├── network.rs      # Widget Réseau
│           ├── disk.rs         # Widget Stockage
│           ├── process.rs      # Widget Processus
│           ├── battery.rs      # Widget Batterie
│           └── fans.rs         # Widget Ventilateurs & refroidissement
│
├── data/                       # Ressources pour Flatpak & desktop
│   ├── io.github.Mvth1s.MonDashboard.desktop
│   ├── io.github.Mvth1s.MonDashboard.metainfo.xml
│   └── icons/
│       ├── hicolor/
│       │   ├── 48x48/apps/io.github.Mvth1s.MonDashboard.png
│       │   └── scalable/apps/io.github.Mvth1s.MonDashboard.svg
│       └── tray/
│           └── mondashboard-tray.svg
│
└── flatpak/
    └── io.github.Mvth1s.MonDashboard.yml   # Manifest Flatpak
```

### Conventions de code

- Chaque fichier du core expose une **struct de données** (ex: `CpuStats`) et une **fonction de collecte** (ex: `fn get_cpu_stats() -> CpuStats`)
- Les erreurs sont gérées avec `Result<T, MonDashboardError>` — pas de `unwrap()` dans le code de production
- Chaque module du core a ses **tests unitaires** en bas de fichier (`#[cfg(test)]`)
- Les widgets GTK suivent tous le même trait : `fn new() -> Self`, `fn update(&self, data: &XxxStats)`

---

## 6. Modules de collecte de données

Tous les modules vivent dans `mondashboard-core/src/`. Chaque module expose une **struct de données** immuable et une **fonction de collecte** pure. Aucun module ne connaît GTK.

---

### 6.1 `cpu.rs`

**Source :** crate `sysinfo` + `/proc/cpuinfo` + `lm-sensors` (via `/sys/class/hwmon/`)

```rust
pub struct CpuStats {
    pub model: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub global_usage: f32,          // 0.0 - 100.0
    pub per_core_usage: Vec<f32>,
    pub frequency_mhz: u64,
    pub frequency_max_mhz: u64,
    pub temperature_celsius: Option<f32>,
}

pub fn get_cpu_stats(sys: &sysinfo::System) -> CpuStats
```

---

### 6.2 `gpu/mod.rs`

**Sources :** `sysfs`, `nvml-wrapper`, détection automatique au démarrage

```rust
pub enum GpuVendor { Nvidia, AmdDiscrete, AmdIgpu, IntelArc, IntelIgpu, Unknown }

pub struct GpuStats {
    pub vendor: GpuVendor,
    pub model: String,
    pub usage_percent: Option<f32>,
    pub vram_used_mb: Option<u64>,      // None si iGPU (mémoire partagée)
    pub vram_total_mb: Option<u64>,
    pub shared_memory_mb: Option<u64>,  // Pour iGPU uniquement
    pub temperature_celsius: Option<f32>,
    pub frequency_mhz: Option<u64>,
}

pub struct AllGpuStats {
    pub gpus: Vec<GpuStats>,            // 1 entrée par GPU détecté
}

pub fn get_gpu_stats() -> AllGpuStats
```

---

### 6.3 `memory.rs`

**Source :** crate `sysinfo`

```rust
pub struct MemoryStats {
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub cached_mb: u64,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

pub fn get_memory_stats(sys: &sysinfo::System) -> MemoryStats
```

---

### 6.4 `network.rs`

**Source :** crate `sysinfo` + `/proc/net/dev` + ping via socket ICMP

```rust
pub struct NetworkInterfaceStats {
    pub name: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub ip_local: Option<String>,
}

pub struct NetworkStats {
    pub interfaces: Vec<NetworkInterfaceStats>,
    pub active_interface: Option<String>,
    pub ping_ms: Option<f32>,           // None si hôte injoignable
    pub ping_host: String,
}

pub fn get_network_stats(previous: &NetworkStats) -> NetworkStats
// Note : nécessite un snapshot précédent pour calculer le débit différentiel
```

---

### 6.5 `disk.rs`

**Source :** crate `sysinfo` + `/proc/diskstats` + `smartctl` (optionnel)

```rust
pub enum DiskKind { Ssd, Hdd, Nvme, Unknown }
pub enum SmartHealth { Good, Warning, Critical, Unavailable }

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
}

pub struct AllDiskStats {
    pub disks: Vec<DiskStats>,
}

pub fn get_disk_stats(previous: &AllDiskStats) -> AllDiskStats
```

---

### 6.6 `process.rs`

**Source :** crate `sysinfo`

```rust
pub struct ProcessStats {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub user: String,
    pub status: String,
}

pub struct AllProcessStats {
    pub processes: Vec<ProcessStats>,   // Triés par CPU% desc par défaut
}

pub fn get_process_stats(sys: &sysinfo::System) -> AllProcessStats
pub fn kill_process(pid: u32) -> Result<(), MonDashboardError>
```

---

### 6.7 `battery.rs`

**Source :** UPower via D-Bus (`zbus` crate)

```rust
pub enum BatteryState { Charging, Discharging, Full, Unknown }

pub struct BatteryStats {
    pub present: bool,                  // false = desktop, widget masqué
    pub percentage: f32,
    pub state: BatteryState,
    pub time_to_empty_min: Option<u32>,
    pub time_to_full_min: Option<u32>,
    pub energy_wh: f64,                 // Capacité actuelle
    pub energy_full_wh: f64,            // Capacité actuelle maximale
    pub energy_full_design_wh: f64,     // Capacité d'origine constructeur
    pub health_percent: f32,            // energy_full / energy_full_design * 100
    pub cycle_count: Option<u32>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub technology: Option<String>,     // Li-ion, LiPolymer, etc.
}

pub fn get_battery_stats() -> BatteryStats
```

---

### 6.8 `alerts.rs`

**Source :** données produites par les autres modules

```rust
pub struct AlertConfig {
    pub cpu_threshold_percent: Option<f32>,
    pub cpu_duration_secs: u32,
    pub ram_threshold_percent: Option<f32>,
    pub gpu_temp_threshold_celsius: Option<f32>,
    pub disk_free_threshold_gb: Option<f64>,
    pub ping_threshold_ms: Option<f32>,
    pub cooldown_secs: u32,
}

pub struct AlertEvent {
    pub kind: AlertKind,
    pub message: String,
    pub triggered_at: std::time::Instant,
}

pub fn check_alerts(
    config: &AlertConfig,
    cpu: &CpuStats,
    memory: &MemoryStats,
    gpu: &AllGpuStats,
    disks: &AllDiskStats,
    network: &NetworkStats,
    last_alerts: &[AlertEvent],
) -> Vec<AlertEvent>
```

---

### 6.10 `fans.rs`

**Source :** `/sys/class/hwmon/*/fan*_input` + `/sys/class/hwmon/*/fan*_label`

```rust
pub enum FanKind {
    CpuCooler,      // Ventirad CPU
    CaseFan,        // Ventilateur boîtier
    AioPump,        // Pompe AIO
    AioRadiator,    // Ventilateur(s) radiateur AIO
    GpuFan,         // Ventilateur GPU (si exposé par hwmon)
    Unknown,
}

pub struct FanStats {
    pub label: String,          // Label hwmon (ex: "fan1", "Pump", "CPU Fan")
    pub kind: FanKind,
    pub rpm: u32,
    pub rpm_min: Option<u32>,   // Si exposé par le driver
    pub rpm_max: Option<u32>,
}

pub struct CoolingStats {
    pub fans: Vec<FanStats>,
    pub coolant_temp_celsius: Option<f32>,  // AIO uniquement, si disponible
}

pub fn get_cooling_stats() -> CoolingStats
```

La détection du type de ventilateur (`FanKind`) est faite sur la base du label hwmon — heuristique, pas parfaite, mais suffisante pour la grande majorité des configurations courantes.

---

### 6.9 `config.rs`

Structure de configuration complète, sérialisable en JSON.

```rust
pub struct AppConfig {
    pub version: u32,                   // Pour migrations futures
    pub refresh_interval_secs: u32,
    pub temperature_unit: TemperatureUnit,
    pub ping_host: String,
    pub alerts: AlertConfig,
    pub layouts: Vec<LayoutConfig>,
    pub active_layout: String,
    pub overlay: OverlayConfig,
}

impl Default for AppConfig { ... }     // Valeurs par défaut raisonnables
```

---

## 7. Interface utilisateur (GTK4)

### 7.1 Bibliothèques UI

| Bibliothèque | Rôle |
|---|---|
| `gtk4` | Widgets de base, fenêtres, layout |
| `libadwaita` | Thème adaptatif GNOME, composants modernes (`AdwApplicationWindow`, `AdwHeaderBar`, etc.) |

`libadwaita` est utilisé pour bénéficier du thème système automatiquement (clair/sombre) et d'une apparence cohérente sur toutes les distros, pas seulement GNOME.

---

### 7.2 Fenêtre principale

```
┌─────────────────────────────────────────────────────┐
│  AdwHeaderBar                                        │
│  [≡ Menu]  MonDashboard  [Layout: Gaming ▾] [⚙]    │
├─────────────────────────────────────────────────────┤
│                                                      │
│   ┌──────────┐  ┌──────────┐  ┌───────────────────┐ │
│   │  CPU     │  │  RAM     │  │  GPU              │ │
│   │  [widget]│  │  [widget]│  │  [widget]         │ │
│   └──────────┘  └──────────┘  └───────────────────┘ │
│                                                      │
│   ┌──────────────────────┐  ┌───────────────────┐   │
│   │  Réseau              │  │  Stockage         │   │
│   │  [widget]            │  │  [widget]         │   │
│   └──────────────────────┘  └───────────────────┘   │
│                                                      │
│   ┌─────────────────────────────────────────────┐   │
│   │  Processus                    [widget]       │   │
│   └─────────────────────────────────────────────┘   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

- Fenêtre redimensionnable, taille minimum 800×600
- Fermer la fenêtre ne quitte pas l'app — elle passe en tray
- En-tête : menu hamburger à gauche, sélecteur de layout au centre, bouton paramètres à droite

---

### 7.3 Composants GTK utilisés

| Besoin | Composant GTK4 / Adwaita |
|---|---|
| Fenêtre principale | `AdwApplicationWindow` |
| En-tête | `AdwHeaderBar` |
| Grille de widgets | `GtkGridView` + `GtkDropTarget` / `GtkDragSource` |
| Graphes temps réel | `GtkDrawingArea` + Cairo (dessin custom) |
| Barres de progression | `GtkLevelBar` ou `GtkProgressBar` stylisé |
| Listes (processus) | `GtkColumnView` + `GtkSortListModel` |
| Notifications | `libnotify` (hors GTK, appel système) |
| Boîtes de dialogue | `AdwMessageDialog` |
| Paramètres | `AdwPreferencesWindow` |
| Sélecteur de layout | `GtkDropDown` |
| Menu hamburger | `GtkMenuButton` + `GtkPopoverMenu` |

---

### 7.4 Thème et apparence

- Suit le thème système (clair/sombre) via `AdwStyleManager` — automatique, sans action de l'utilisateur
- Surcharge possible dans les paramètres (forcer clair ou sombre)
- Codes couleur des métriques :
  - **Vert** : charge normale (< 60%)
  - **Orange** : charge modérée (60–85%)
  - **Rouge** : charge critique (> 85%)
- Ces seuils de couleur sont distincts des seuils d'alerte notification — ils sont fixes et non configurables pour garder l'interface lisible

---

### 7.5 Graphes temps réel

Les graphes (CPU, RAM, réseau) sont dessinés avec `GtkDrawingArea` + Cairo. Chaque widget maintient un **buffer circulaire** de 60 valeurs (60 secondes à 1 tick/s) stocké dans le frontend.

Comportement :
- Le graphe se remplit de droite à gauche au démarrage
- Axe Y auto-ajusté selon le maximum observé sur la fenêtre (réseau) ou fixe 0–100% (CPU, RAM)
- Couleur de la courbe cohérente avec le code couleur de charge

---

### 7.6 Accessibilité

- Labels explicites sur tous les widgets (`gtk_accessible_update_property`)
- Tooltips sur les valeurs numériques pour expliquer l'unité
- Navigation clavier possible sur tous les éléments interactifs
- Taille de police respecte les paramètres d'accessibilité du système

---

## 8. Système de widgets & layouts

### 8.1 Principe

Le dashboard est une **grille configurable** où chaque cellule peut accueillir un widget. L'utilisateur arrange librement les widgets par drag & drop et sauvegarde plusieurs configurations nommées (layouts). Un layout est un snapshot complet de la disposition et des préférences de chaque widget.

---

### 8.2 Grille

- Grille de **12 colonnes** (standard CSS Grid adapté à GTK)
- Hauteur de rangée fixe (~120px), nombre de rangées illimité avec scroll vertical
- Chaque widget occupe un nombre de colonnes et de rangées défini par sa taille :

| Taille | Colonnes | Rangées | Usage typique |
|---|---|---|---|
| Petit | 3 | 1 | Valeur unique (RAM %, CPU %) |
| Moyen | 6 | 1 | Métrique + graphe compact |
| Large | 6 | 2 | Métrique + graphe + détails |
| Plein | 12 | 2 | Processus, stockage multi-disques |

- Un widget peut être redimensionné via une poignée en coin bas-droit
- Les widgets se repositionnent automatiquement pour éviter les chevauchements (auto-flow)

---

### 8.3 Drag & drop

- Implémenté avec `GtkDragSource` et `GtkDropTarget` (API native GTK4)
- Pendant le drag : placeholder semi-transparent à la position cible
- Snap sur la grille au relâchement
- Annulation par touche `Échap`
- Mode édition activable/désactivable (évite les déplacements accidentels) via un bouton dédié dans l'en-tête

---

### 8.4 Layouts

```rust
pub struct WidgetPlacement {
    pub widget_id: String,      // ex: "cpu", "gpu_0", "disk_sda"
    pub col: u32,
    pub row: u32,
    pub col_span: u32,
    pub row_span: u32,
    pub visible: bool,
    pub widget_config: WidgetSpecificConfig,
}

pub struct LayoutConfig {
    pub name: String,           // ex: "Gaming", "Bureau", "IA locale"
    pub placements: Vec<WidgetPlacement>,
}
```

- Un layout par défaut est fourni au premier lancement (disposition équilibrée de tous les widgets)
- L'utilisateur peut créer, renommer, dupliquer et supprimer des layouts
- Le layout actif est sauvegardé dans `config.json` et restauré au prochain démarrage
- Maximum 10 layouts (limite arbitraire pour garder le sélecteur lisible)

---

### 8.5 Configuration par widget

Chaque widget expose ses propres options, accessibles via un clic droit → "Configurer" :

| Widget | Options spécifiques |
|---|---|
| CPU | Afficher par cœur oui/non |
| GPU | Choisir quel GPU afficher (multi-GPU) |
| Réseau | Interface à afficher, hôte de ping |
| Processus | Tri par défaut (CPU/RAM), nombre de lignes |
| Overlay | Quelles métriques afficher |
| Ventilateurs | Masquer les capteurs non pertinents |

---

### 8.6 Persistance

La configuration des layouts est sérialisée en JSON dans `config.json` à chaque modification. La sauvegarde est **debounced** (délai de 500ms après la dernière action) pour éviter des écritures disque trop fréquentes pendant un drag & drop.

---

## 9. Notifications & alertes

### 9.1 Principe

Le système d'alertes surveille en continu les métriques collectées et envoie une notification système native lorsqu'un seuil configuré est dépassé. Il est entièrement optionnel — toutes les alertes sont désactivées par défaut.

---

### 9.2 Alertes disponibles

| Métrique | Condition | Valeur par défaut |
|---|---|---|
| CPU | Usage global > X% pendant Y secondes | Désactivé |
| RAM | Usage > X% | Désactivé |
| GPU | Température > X°C | Désactivé |
| GPU | Usage > X% | Désactivé |
| Disque | Espace libre < X Go | Désactivé |
| Réseau | Ping > X ms | Désactivé |
| Batterie | Charge < X% | Désactivé |
| Ventilateur | Vitesse < X RPM (détection arrêt) | Désactivé |

---

### 9.3 Logique de déclenchement

```
Chaque tick de polling :
  1. check_alerts() compare les stats actuelles aux seuils configurés
  2. Si seuil dépassé ET alerte activée :
       → Vérifie le cooldown (temps écoulé depuis la dernière notif du même type)
       → Si cooldown écoulé : envoie la notification + met à jour last_triggered
       → Sinon : ignore
  3. CPU uniquement : le seuil doit être dépassé pendant Y secondes consécutives
     (évite les pics ponctuels qui génèrent du bruit)
```

---

### 9.4 Notifications système

- Envoyées via `notify-rust` (wrapper de `libnotify`)
- Icône MonDashboard dans la notification
- Titre explicite : "MonDashboard — Alerte CPU"
- Corps : valeur actuelle + seuil configuré (ex: "CPU à 94% depuis 10s — seuil : 90%")
- Urgence : `Normal` par défaut, `Critical` si la valeur dépasse largement le seuil (> 1.2× le seuil)
- Permission Flatpak requise : `--talk-name=org.freedesktop.Notifications`

---

### 9.5 Cooldown et anti-spam

- Cooldown global configurable (défaut : 5 minutes)
- Chaque type d'alerte a son propre timer de cooldown indépendant
- Pendant le cooldown, l'indicateur visuel du widget reste rouge mais aucune nouvelle notification n'est envoyée
- Le cooldown se réinitialise si la métrique repasse sous le seuil

---

### 9.6 Configuration des alertes

Accessible depuis `AdwPreferencesWindow` → onglet "Alertes" :

- Activation/désactivation par interrupteur pour chaque alerte
- Champ numérique pour le seuil
- Champ durée pour le CPU (secondes)
- Champ cooldown global (minutes)
- Bouton "Tester" qui envoie une notification de test immédiatement

---

## 10. Packaging Flatpak

### 10.1 Identifiants

```
App ID   : io.github.Mvth1s.MonDashboard
Runtime  : org.gnome.Platform (version 46 minimum)
SDK      : org.gnome.Sdk
```

---

### 10.2 Permissions requises

```yaml
finish-args:
  # Affichage
  - --socket=wayland
  - --socket=fallback-x11
  - --device=dri                          # Accès GPU (rendu + infos matérielles)

  # Données système (lecture seule)
  - --filesystem=/sys/class/hwmon:ro      # Températures, ventilateurs
  - --filesystem=/sys/class/drm:ro        # Infos GPU AMD / Intel
  - --filesystem=/sys/class/power_supply:ro  # Batterie (fallback sysfs)
  - --filesystem=/proc:ro                 # CPU, RAM, processus, réseau

  # Services système
  - --talk-name=org.freedesktop.Notifications   # Notifications
  - --system-talk-name=org.freedesktop.UPower   # Batterie
  - --system-talk-name=org.freedesktop.UDisks2  # Infos disques
  - --system-talk-name=org.freedesktop.hostname1 # Infos machine

  # Tray icon
  - --talk-name=org.kde.StatusNotifierWatcher
  - --talk-name=com.canonical.AppMenu.Registrar

  # Kill processus (nécessite un portail ou permission élevée)
  - --allow=per-app-dev-shm
```

> **Note :** Le kill de processus depuis un sandbox Flatpak est une limitation connue. Dans un premier temps, seuls les processus appartenant à l'utilisateur courant pourront être terminés via `signal(2)`. Les processus root nécessiteront une élévation via `pkexec` — fonctionnalité à implémenter en v0.2.

---

### 10.3 Structure du manifest

```yaml
# flatpak/io.github.Mvth1s.MonDashboard.yml

app-id: io.github.Mvth1s.MonDashboard
runtime: org.gnome.Platform
runtime-version: '46'
sdk: org.gnome.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable

command: mondashboard

modules:
  - name: mondashboard
    buildsystem: simple
    build-commands:
      - cargo build --release
      - install -Dm755 target/release/mondashboard /app/bin/mondashboard
      - install -Dm644 data/io.github.Mvth1s.MonDashboard.desktop
          /app/share/applications/io.github.Mvth1s.MonDashboard.desktop
      - install -Dm644 data/io.github.Mvth1s.MonDashboard.metainfo.xml
          /app/share/metainfo/io.github.Mvth1s.MonDashboard.metainfo.xml
      - install -Dm644 data/icons/hicolor/scalable/apps/io.github.Mvth1s.MonDashboard.svg
          /app/share/icons/hicolor/scalable/apps/io.github.Mvth1s.MonDashboard.svg
    sources:
      - type: dir
        path: ..
```

---

### 10.4 Fichiers de données requis par Flathub

**`.desktop`**
```ini
[Desktop Entry]
Name=MonDashboard
Comment=Monitoring système Linux
Exec=mondashboard
Icon=io.github.Mvth1s.MonDashboard
Type=Application
Categories=System;Monitor;
Keywords=cpu;ram;gpu;réseau;système;monitoring;
```

**`metainfo.xml`** — obligatoire pour Flathub, doit contenir :
- Description courte et longue
- Captures d'écran (minimum 1, recommandé 3)
- Historique des versions (`<releases>`)
- Informations de contact et URL du dépôt
- Catégorie OARS (classification du contenu)

---

### 10.5 Prérequis pour la soumission Flathub

- [ ] Dépôt GitHub public avec le code source
- [ ] Fichier `metainfo.xml` valide (vérifié avec `appstreamcli validate`)
- [ ] Au moins une release taguée (`v0.1.0`)
- [ ] Icône SVG propre en 128×128 minimum
- [ ] Captures d'écran de l'application
- [ ] Ouvrir une Pull Request sur `github.com/flathub/flathub` avec le manifest
- [ ] Passer la revue manuelle de l'équipe Flathub (délai ~1–2 semaines)

---

## 11. Roadmap & MVP

### 11.1 Philosophie

Chaque version doit être **utilisable et distribuable** — pas de version "work in progress" publiée sur Flathub. On préfère un périmètre réduit mais stable à une liste de features incomplètes.

---

### 11.2 v0.1 — MVP (premier lancement Flathub)

**Objectif :** une app fonctionnelle, stable, avec les métriques essentielles. Pas encore configurable.

| Fonctionnalité | Inclus |
|---|---|
| Widget CPU (global + température) | ✅ |
| Widget RAM | ✅ |
| Widget GPU (AMD + NVIDIA + Intel) | ✅ |
| Widget Réseau (débit) | ✅ |
| Widget Stockage (partitions) | ✅ |
| Widget Processus (top 10, tri CPU/RAM) | ✅ |
| Widget Batterie | ✅ |
| Widget Ventilateurs | ✅ |
| Tray icon | ✅ |
| Thème clair/sombre automatique | ✅ |
| Layout fixe (non configurable) | ✅ |
| Intervalle de rafraîchissement (fixe 2s) | ✅ |
| Drag & drop / layouts multiples | ❌ |
| Mini overlay | ❌ |
| Notifications & alertes | ❌ |
| Paramètres | ❌ |
| Kill de processus | ❌ |

---

### 11.3 v0.2 — Configuration & layouts

**Objectif :** rendre l'app personnalisable.

- Système de drag & drop et layouts nommés
- Redimensionnement des widgets
- Fenêtre paramètres (intervalle, unités, langue)
- Kill de processus (utilisateur courant uniquement)
- Sélecteur d'interface réseau

---

### 11.4 v0.3 — Overlay & alertes

**Objectif :** usage gaming et surveillance continue.

- Mini overlay always-on-top configurable
- Raccourci clavier global pour l'overlay
- Système de notifications et alertes avec cooldown
- Bouton "Tester" les alertes

---

### 11.5 v0.4 — Finitions & Flathub

**Objectif :** qualité de distribution publique.

- Import / export de la configuration (JSON)
- Graphes historiques (~60s) sur tous les widgets
- Santé SMART des disques
- Température liquide AIO (si disponible)
- Kill de processus root via `pkexec`
- Métadonnées Flathub complètes, captures d'écran, traductions
- Soumission officielle sur Flathub

---

### 11.6 Au-delà de v0.4 (idées futures)

Ces fonctionnalités ne sont pas planifiées mais peuvent être envisagées selon l'intérêt de la communauté :

- Support `liquidctl` pour les AIO avec drivers propriétaires (Corsair, NZXT…)
- Historique long terme (base de données SQLite)
- Profils d'alertes (ex: profil "Gaming" avec seuils différents)
- Support multi-moniteur (afficher l'overlay sur un écran spécifique)
- Contributions communautaires (thèmes, widgets tiers)

---

### 11.7 Versioning

Le projet suit la **gestion sémantique de versions** (`semver`) :
- `0.x.y` : phase de développement initial, breaking changes possibles entre mineures
- `1.0.0` : première version considérée stable et feature-complete
- Chaque version publiée sur Flathub correspond à un tag Git (`v0.1.0`, `v0.2.0`…)

---

## 12. Décisions techniques (ADR)

Les ADR (*Architecture Decision Records*) documentent les choix techniques structurants et leurs justifications. Utile pour ne pas remettre en question les mêmes décisions six mois plus tard.

---

### ADR-001 — GTK4 + Rust plutôt que Tauri

**Décision :** utiliser GTK4 avec `gtk4-rs` pour le frontend.

**Raisons :**
- Intégration native Linux, consommation mémoire minimale
- Pas de WebView embarqué (~100Mo de base avec Tauri/Electron)
- Thème système automatique via `libadwaita`
- Cohérence avec l'écosystème GNOME/Flatpak

**Compromis acceptés :**
- Courbe d'apprentissage plus élevée que Tauri + Vue
- UI configurable (drag & drop) plus difficile à implémenter qu'en CSS Grid

---

### ADR-002 — Architecture deux crates (core + gtk)

**Décision :** séparer la collecte de données (`mondashboard-core`) du frontend (`mondashboard-gtk`) en deux crates distinctes dans un workspace Cargo.

**Raisons :**
- Chaque crate est lisible et testable indépendamment
- Facilite le vibecoding : chaque fichier a un rôle unique et clair
- Le core pourrait être réutilisé (CLI, autre frontend, lib externe)
- Les tests unitaires du core ne nécessitent pas d'environnement graphique

**Compromis acceptés :**
- Légère complexité de workspace au démarrage

---

### ADR-003 — `sysinfo` comme crate principale de collecte

**Décision :** utiliser la crate `sysinfo` pour CPU, RAM et processus.

**Raisons :**
- Crate la plus utilisée et maintenue pour ce cas d'usage en Rust
- API simple et bien documentée
- Gère correctement les mises à jour différentielles (CPU%)

**Compromis acceptés :**
- Abstraction qui peut masquer certains détails bas niveau
- GPU non supporté — géré séparément via `sysfs` et `nvml-wrapper`

---

### ADR-004 — Graphes via Cairo plutôt qu'une lib externe

**Décision :** dessiner les graphes temps réel avec `GtkDrawingArea` + Cairo.

**Raisons :**
- Pas de crate de graphes GTK4 mature et maintenue en Rust à ce jour
- Cairo est la lib de dessin native de GTK, zéro dépendance supplémentaire
- Contrôle total sur le rendu et les animations

**Compromis acceptés :**
- Plus de code à écrire qu'avec une lib dédiée
- À réévaluer si une crate mature émerge

---

### ADR-005 — UPower via D-Bus pour la batterie

**Décision :** utiliser UPower (`org.freedesktop.UPower`) via `zbus` pour les données batterie.

**Raisons :**
- UPower est présent sur toutes les distros desktop Linux
- Abstrait les différences matérielles entre fabricants
- Données plus fiables et complètes que la lecture brute de `/sys/class/power_supply/`
- API D-Bus stable et bien documentée

**Compromis acceptés :**
- Dépendance à un service système (non disponible sur serveur)
- Permission Flatpak supplémentaire (`--system-talk-name=org.freedesktop.UPower`)

---

### ADR-006 — Configuration en JSON plutôt que TOML ou binaire

**Décision :** stocker la configuration dans un fichier JSON.

**Raisons :**
- Lisible et éditable manuellement par l'utilisateur si besoin
- Sérialisé/désérialisé avec `serde_json` sans dépendance supplémentaire
- Import/export trivial
- Versioning de schéma facile à implémenter

**Compromis acceptés :**
- Légèrement plus verbeux que TOML pour la config humaine
- TOML aurait été plus lisible mais nécessite une dépendance supplémentaire

---

### ADR-007 — Mode édition explicite pour le drag & drop

**Décision :** le drag & drop des widgets n'est actif que lorsque le mode édition est activé via un bouton dédié.

**Raisons :**
- Évite les déplacements accidentels lors d'une utilisation normale
- L'app est avant tout un outil de lecture — l'édition est secondaire
- Cohérent avec des apps similaires (KDE Dashboard, Home Assistant)

**Compromis acceptés :**
- Une action supplémentaire pour modifier le layout
