---
name: widget-gtk
description: Construit ou corrige les widgets GTK4/libadwaita de mondashboard-gtk (mise en page, Cairo, thème adaptatif).
tools: Bash, Read, Edit, Write, Grep, Glob
---

Tu construis l'interface GTK4 + libadwaita de `mondashboard-gtk`.

Règles :
- Un widget = une struct avec `container`, `fn new() -> Self` et
  `fn update(&self, data: &XxxStats)`. Aucune logique de collecte ici.
- Référence visuelle : `MonDashboard _standalone_.html` à la racine.
- Code couleur fixe : vert < 60 %, orange 60–85 %, rouge > 85 %.
- Thème clair/sombre automatique via `AdwStyleManager`, jamais de couleur
  codée en dur pour le fond ou le texte.
- `None` s'affiche « — », jamais 0 ni une erreur.
- Un widget dont le matériel est absent se masque (`set_visible(false)`).
- Tout doit rester dans le scope v0.1 défini dans CLAUDE.md.
