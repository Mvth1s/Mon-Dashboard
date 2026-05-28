# MonDashboard

> Dashboard système Linux — simple, lisible, configurable.

MonDashboard est une application de monitoring système pensée pour être **comprise au premier regard**, sans apprentissage préalable. CPU, GPU, RAM, réseau, stockage, processus, batterie, ventilateurs — tout en un seul endroit, avec une interface graphique moderne qui tourne en arrière-plan.

---

## Aperçu

_Captures d'écran à venir après la première release._

---

## Fonctionnalités

- **CPU** — utilisation globale et par cœur, fréquence, température
- **GPU** — AMD, NVIDIA, Intel Arc et iGPU, utilisation, VRAM, température (multi-GPU supporté)
- **RAM** — utilisée / disponible / cache, swap, graphe temps réel
- **Réseau** — débit montant/descendant par interface, ping, IP locale
- **Stockage** — partitions, vitesses R/W, santé SMART, type de disque
- **Processus** — top consommateurs CPU/RAM, recherche, kill de processus
- **Batterie** — charge, santé, cycles, Wh, informations fabricant *(portables)*
- **Ventilateurs** — RPM par capteur, pompe AIO, température liquide
- **Tray icon** — tourne en arrière-plan, indicateur de charge dynamique
- **Mini overlay** — fenêtre compacte always-on-top pour le gaming et l'IA locale *(v0.3)*
- **Layouts multiples** — drag & drop, plusieurs configurations nommées *(v0.2)*
- **Alertes** — notifications système configurables par seuil *(v0.3)*

---

## Installation

### Via Flathub *(bientôt disponible)*

```bash
flatpak install flathub io.github.Mvth1s.MonDashboard
```

### Depuis les sources

**Prérequis :**
- Rust stable (`rustup` recommandé)
- GTK4 + libadwaita (`libgtk-4-dev`, `libadwaita-1-dev`)
- `lm-sensors` pour les températures et ventilateurs
- `smartmontools` pour la santé des disques *(optionnel)*

```bash
git clone https://github.com/Mvth1s/MonDashboard.git
cd MonDashboard
cargo build --release
./target/release/mondashboard
```

**Via Flatpak (local) :**
```bash
flatpak-builder --install --user build-dir flatpak/io.github.Mvth1s.MonDashboard.yml
flatpak run io.github.Mvth1s.MonDashboard
```

---

## Compatibilité

| Élément | Détail |
|---|---|
| **OS** | Toutes les distributions Linux |
| **Format** | Flatpak (Flathub) |
| **GPU** | AMD (discret + APU), NVIDIA, Intel Arc, iGPU Intel |
| **Affichage** | Wayland *(recommandé)*, X11 |
| **Thème** | Suit automatiquement le thème système (clair/sombre) |

---

## Architecture

Le projet est structuré en deux crates dans un workspace Cargo :

```
mondashboard-core/   # Librairie Rust pure — collecte des données système
mondashboard-gtk/    # Frontend GTK4 — interface graphique
```

Cette séparation garantit que la couche de collecte est testable indépendamment du frontend. Voir [`SPEC.md`](SPEC.md) pour la spécification technique complète.

**Stack :**
- Rust stable
- GTK4 + libadwaita (`gtk4-rs`)
- `sysinfo` pour CPU / RAM / processus
- `nvml-wrapper` pour GPU NVIDIA
- `zbus` pour UPower (batterie) et UDisks2 (disques)
- `notify-rust` pour les notifications système

---

## Roadmap

| Version | Objectif | Statut |
|---|---|---|
| v0.1 | MVP — tous les widgets, layout fixe, tray icon | 🚧 En cours |
| v0.2 | Drag & drop, layouts multiples, paramètres | 📋 Planifié |
| v0.3 | Mini overlay, alertes & notifications | 📋 Planifié |
| v0.4 | Finitions, traductions, soumission Flathub | 📋 Planifié |

---

## Contribuer

Le projet est en phase de démarrage solo. Les issues et suggestions sont les bienvenues.

Une fois la v0.1 publiée, les contributions sous forme de Pull Requests seront acceptées. En attendant, n'hésitez pas à ouvrir une issue pour signaler un bug ou proposer une fonctionnalité.

---

## Licence

[GPL-3.0](LICENSE) © 2024 Mvth1s
