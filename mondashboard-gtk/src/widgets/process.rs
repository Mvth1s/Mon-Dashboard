use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Grid, Label};
use mondashboard_core::process::AllProcessStats;

use super::{appliquer_niveau, carte, format_mo};

/// Nombre de processus affichés.
const LIGNES: usize = 8;
/// Longueur au-delà de laquelle un nom de processus est tronqué.
const NOM_MAX: usize = 22;

pub struct ProcessWidget {
    pub container: GtkBox,
    cellules: Vec<[Label; 4]>,
}

impl ProcessWidget {
    pub fn new() -> Self {
        let (container, contenu) = carte("Processus");

        let grille = Grid::builder().row_spacing(4).column_spacing(12).build();
        for (colonne, entete) in ["Nom", "CPU", "Mémoire", "Utilisateur"]
            .into_iter()
            .enumerate()
        {
            let label = Label::builder()
                .label(entete)
                .halign(if colonne == 0 {
                    Align::Start
                } else {
                    Align::End
                })
                .css_classes(["secondaire"])
                .build();
            grille.attach(&label, colonne as i32, 0, 1, 1);
        }

        let mut cellules = Vec::with_capacity(LIGNES);
        for rang in 0..LIGNES {
            let ligne: [Label; 4] = std::array::from_fn(|colonne| {
                let label = Label::builder()
                    .halign(if colonne == 0 {
                        Align::Start
                    } else {
                        Align::End
                    })
                    .hexpand(colonne == 0)
                    .build();
                grille.attach(&label, colonne as i32, rang as i32 + 1, 1, 1);
                label
            });
            cellules.push(ligne);
        }

        contenu.append(&grille);

        Self {
            container,
            cellules,
        }
    }

    pub fn update(&self, data: &AllProcessStats) {
        for (rang, cellules) in self.cellules.iter().enumerate() {
            match data.processes.get(rang) {
                Some(processus) => {
                    cellules[0].set_label(&tronquer(&processus.name));
                    cellules[1].set_label(&format!("{:.1} %", processus.cpu_percent));
                    cellules[2].set_label(&format_mo(processus.memory_mb));
                    cellules[3].set_label(&processus.user);
                    appliquer_niveau(&cellules[1], processus.cpu_percent);
                }
                // Moins de processus que de lignes : on vide sans démonter la
                // grille, pour éviter que l'affichage ne saute.
                None => {
                    for cellule in cellules {
                        cellule.set_label("");
                    }
                }
            }
        }
    }
}

fn tronquer(nom: &str) -> String {
    if nom.chars().count() <= NOM_MAX {
        return nom.to_string();
    }
    let debut: String = nom.chars().take(NOM_MAX - 1).collect();
    format!("{debut}…")
}

impl Default for ProcessWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nom_long_tronque_sans_couper_un_caractere() {
        let nom = "un-nom-de-processus-vraiment-très-long";
        assert!(tronquer(nom).chars().count() <= NOM_MAX);
        assert_eq!(tronquer("court"), "court");
    }
}
