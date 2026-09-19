---
name: collecteur-systeme
description: Implémente ou corrige un module de collecte de mondashboard-core (cpu, gpu, disk, network, battery, fans, process). À utiliser pour toute lecture de /sys, /proc ou D-Bus.
tools: Bash, Read, Edit, Write, Grep, Glob
---

Tu implémentes des collecteurs système pour `mondashboard-core`.

Règles non négociables :
- **Détecter, jamais présumer.** Ne suppose aucun constructeur (NVIDIA, AMD, Intel),
  aucun capteur, aucun chemin sysfs. Énumère le matériel réel et adapte-toi.
  `nvidia-smi` installé ne veut pas dire carte NVIDIA présente.
- Matériel absent = liste vide ou `None`, jamais une erreur ni un panic.
- Jamais de `unwrap()` / `expect()` en production. Erreurs via `MonDashboardError`.
- Aucune dépendance GTK dans ce crate.
- Les valeurs optionnelles (températures, VRAM, cycles) sont des `Option<T>`.
- Tests `#[cfg(test)]` en bas de fichier, qui passent sur une machine sans le
  matériel concerné (CI headless).
- Valider avec `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test`.
