//! Dictionnaires personnels de Texteur (plusieurs, activables ensemble) + réglages
//! (langue, dictionnaires actifs).
//! Fichiers : `%APPDATA%\Texteur\dictionnaires\*.json` et `%APPDATA%\Texteur\reglages.json`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::masque::est_cjk;

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Terme {
    /// Forme latine : mot, nom propre, pinyin…
    #[serde(default, alias = "pinyin")]
    pub mot: String,
    /// Écriture d'origine (汉字, kana…).
    #[serde(default, alias = "hanzi")]
    pub caracteres: String,
    /// Variante (traditionnel…).
    #[serde(default, alias = "trad")]
    pub variante: String,
    #[serde(default)]
    pub sens: String,
}

impl Terme {
    pub fn resume(&self) -> String {
        [&self.caracteres, &self.mot].iter().filter(|s| !s.is_empty()).map(|s| s.as_str()).collect::<Vec<_>>().join(" ")
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Dictionnaire {
    pub nom: String,
    pub termes: Vec<Terme>,
}

impl Dictionnaire {
    /// Formes à accepter / protéger : mot, mot sans tons (pour le pinyin), caractères, variante.
    pub fn formes(&self) -> impl Iterator<Item = String> + '_ {
        self.termes.iter().flat_map(|t| {
            let sans_tons = (!t.caracteres.is_empty()).then(|| sans_accents(&t.mot));
            [Some(t.mot.clone()), sans_tons, Some(t.caracteres.clone()), Some(t.variante.clone())]
                .into_iter()
                .flatten()
                .filter(|s| !s.is_empty())
        })
    }

    /// Ajoute (ou complète) un terme ; renvoie faux si rien de neuf.
    pub fn ajoute(&mut self, t: Terme) -> bool {
        if t.caracteres.is_empty() && t.mot.is_empty() {
            return false;
        }
        let meme = |a: &Terme| {
            (!t.caracteres.is_empty() && a.caracteres == t.caracteres)
                || (t.caracteres.is_empty() && sans_accents(&a.mot).to_lowercase() == sans_accents(&t.mot).to_lowercase())
        };
        if let Some(e) = self.termes.iter_mut().find(|a| meme(a)) {
            let avant = e.clone();
            for (champ, neuf) in [(&mut e.variante, &t.variante), (&mut e.mot, &t.mot), (&mut e.sens, &t.sens)] {
                if champ.is_empty() {
                    *champ = neuf.clone();
                }
            }
            return *e != avant;
        }
        self.termes.insert(0, t);
        true
    }

}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Reglages {
    /// Langue de correction (code Windows, ex. « fr-FR »).
    pub langue: String,
    /// Noms des dictionnaires actifs.
    pub actifs: Vec<String>,
    /// Dictionnaire qui reçoit les ajouts rapides.
    pub cible: String,
    /// Protéger automatiquement sinogrammes, kana et hangul.
    pub proteger_ecritures: bool,
    /// Mode écriture : nombre de mots visé par séance.
    pub objectif_mots: usize,
    /// Mode écriture : nombre de pages visées pour le document (0 : pas d'objectif).
    pub pages_visees: usize,
    /// Mots par page imprimée (environ 250 pour un livre).
    pub mots_par_page: usize,
    /// Mode clair doux au lieu du mode sombre.
    pub theme_clair: bool,
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages {
            langue: "fr-FR".into(),
            actifs: vec!["Chinois".into()],
            cible: "Chinois".into(),
            proteger_ecritures: true,
            objectif_mots: 500,
            pages_visees: 0,
            mots_par_page: 250,
            theme_clair: false,
        }
    }
}

pub struct Bibliotheque {
    pub dicos: Vec<Dictionnaire>,
    pub reglages: Reglages,
}

pub fn sans_accents(s: &str) -> String {
    s.nfd().filter(|c| !is_combining_mark(*c)).collect()
}

fn dossier() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("Texteur")
}

fn chemin_dico(nom: &str) -> PathBuf {
    let fichier: String = nom.chars().map(|c| if r#"\/:*?"<>|"#.contains(c) { '_' } else { c }).collect();
    dossier().join("dictionnaires").join(format!("{fichier}.json"))
}

impl Bibliotheque {
    pub fn charge() -> Self {
        let mut reglages: Reglages = std::fs::read_to_string(dossier().join("reglages.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let mut dicos: Vec<Dictionnaire> = std::fs::read_dir(dossier().join("dictionnaires"))
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .filter_map(|e| serde_json::from_str(&std::fs::read_to_string(e.path()).ok()?).ok())
            .collect();

        // Base de référence officielle enrichie (villes, provinces, entreprises, personnalités, commerce)
        let mut base_termes: Vec<Terme> = serde_json::from_str(include_str!("../ressources/base_chinois.json")).unwrap_or_default();
        // termes personnels (ressources/base_chinois_perso.json, hors du dépôt public), copiés par build.rs
        base_termes.extend(serde_json::from_str::<Vec<Terme>>(include_str!(concat!(env!("OUT_DIR"), "/base_chinois_perso.json"))).unwrap_or_default());

        if let Some(pos) = dicos.iter_mut().position(|d| d.nom.eq_ignore_ascii_case("Chinois")) {
            // Nettoyer les collages accidentels de longs paragraphes
            dicos[pos].termes.retain(|t| !(t.mot.chars().count() > 80 && t.caracteres.is_empty()));
            // Fusionner les termes de la base complète
            for t in base_termes {
                dicos[pos].ajoute(t);
            }
        } else {
            dicos.push(Dictionnaire { nom: "Chinois".into(), termes: base_termes });
        }

        dicos.sort_by_key(|d| d.nom.to_lowercase());
        reglages.actifs.retain(|a| dicos.iter().any(|d| d.nom == *a));
        if !reglages.actifs.iter().any(|a| a.eq_ignore_ascii_case("Chinois")) {
            reglages.actifs.push("Chinois".into());
        }
        let b = Bibliotheque { dicos, reglages };
        for i in 0..b.dicos.len() {
            let _ = b.sauve_dico(i);
        }
        let _ = b.sauve_reglages();
        b
    }

    pub fn sauve_dico(&self, i: usize) -> Result<(), String> {
        let d = &self.dicos[i];
        let f = chemin_dico(&d.nom);
        std::fs::create_dir_all(f.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(f, serde_json::to_string_pretty(d).unwrap()).map_err(|e| e.to_string())
    }

    pub fn sauve_reglages(&self) -> Result<(), String> {
        std::fs::create_dir_all(dossier()).map_err(|e| e.to_string())?;
        std::fs::write(dossier().join("reglages.json"), serde_json::to_string_pretty(&self.reglages).unwrap())
            .map_err(|e| e.to_string())
    }

    pub fn est_actif(&self, nom: &str) -> bool {
        self.reglages.actifs.iter().any(|a| a == nom)
    }

    pub fn active(&mut self, nom: &str, actif: bool) {
        self.reglages.actifs.retain(|a| a != nom);
        if actif {
            self.reglages.actifs.push(nom.to_string());
        }
    }

    /// Formes de tous les dictionnaires actifs.
    pub fn formes_actives(&self) -> Vec<String> {
        self.dicos.iter().filter(|d| self.est_actif(&d.nom)).flat_map(|d| d.formes()).collect()
    }

    /// Dictionnaire modifiable qui reçoit les ajouts rapides.
    pub fn cible(&self) -> Option<usize> {
        let i = self.dicos.iter().position(|d| d.nom == self.reglages.cible);
        i.or((!self.dicos.is_empty()).then_some(0))
    }

    pub fn cree(&mut self, nom: &str) -> Result<usize, String> {
        let nom = nom.trim();
        if nom.is_empty() || self.dicos.iter().any(|d| d.nom.eq_ignore_ascii_case(nom)) {
            return Err("nom vide ou déjà utilisé".into());
        }
        self.dicos.push(Dictionnaire { nom: nom.to_string(), termes: Vec::new() });
        let i = self.dicos.len() - 1;
        self.sauve_dico(i)?;
        self.active(nom, true);
        self.reglages.cible = nom.to_string();
        Ok(i)
    }

    pub fn supprime(&mut self, i: usize) -> Result<(), String> {
        let d = self.dicos.remove(i);
        self.active(&d.nom, false);
        std::fs::remove_file(chemin_dico(&d.nom)).map_err(|e| e.to_string())
    }
}


/// Analyse une ligne copiée : « 长城 Chángchéng », « 长城⇥Chángchéng⇥Grande Muraille »,
/// « Chángchéng ; Grande Muraille », « 长城 - Grande Muraille », « Wei Zhongxian »…
pub fn analyse_ligne(ligne: &str) -> Option<Terme> {
    let l = ligne.trim().replace(" – ", "\t").replace(" - ", "\t").replace(" = ", "\t").replace(" : ", "\t");
    let champs: Vec<&str> = l.split(['\t', ';', '|', '；']).map(str::trim).filter(|s| !s.is_empty()).collect();
    let mut t = Terme::default();
    match champs.as_slice() {
        [] => return None,
        [seul] => {
            t.caracteres = seul.chars().filter(|c| est_cjk(*c)).collect();
            let reste: String = seul.chars().filter(|c| !est_cjk(*c)).collect();
            t.mot = reste.split_whitespace().collect::<Vec<_>>().join(" ");
            t.mot = t.mot.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
        }
        _ => {
            let mut latins = Vec::new();
            for c in champs {
                if c.chars().any(est_cjk) {
                    if t.caracteres.is_empty() {
                        t.caracteres = c.into();
                    } else if t.variante.is_empty() {
                        t.variante = c.into();
                    }
                } else {
                    latins.push(c);
                }
            }
            if t.caracteres.is_empty() || latins.len() > 1 {
                t.mot = latins.first().map(|s| s.to_string()).unwrap_or_default();
                t.sens = latins.iter().skip(1).copied().collect::<Vec<_>>().join(" ; ");
            } else if let Some(s) = latins.first() {
                // « 汉字 ⇥ x » : x est du pinyin s'il porte des tons ou n'a pas d'espace
                if sans_accents(s) != *s || !s.contains(' ') {
                    t.mot = s.to_string();
                } else {
                    t.sens = s.to_string();
                }
            }
        }
    }
    (!t.caracteres.is_empty() || !t.mot.is_empty()).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyse() {
        let t = analyse_ligne("长城 Chángchéng").unwrap();
        assert_eq!((t.caracteres.as_str(), t.mot.as_str()), ("长城", "Chángchéng"));
        let t = analyse_ligne("深圳\tShēnzhèn\tville du Guangdong").unwrap();
        assert_eq!((t.caracteres.as_str(), t.mot.as_str(), t.sens.as_str()), ("深圳", "Shēnzhèn", "ville du Guangdong"));
        let t = analyse_ligne("Wei Zhongxian").unwrap();
        assert_eq!((t.caracteres.as_str(), t.mot.as_str()), ("", "Wei Zhongxian"));
        let t = analyse_ligne("长城 - Grande Muraille de Chine").unwrap();
        assert_eq!((t.caracteres.as_str(), t.sens.as_str()), ("长城", "Grande Muraille de Chine"));
        assert!(analyse_ligne("   ").is_none());
    }

    #[test]
    fn ancien_format_json() {
        let t: Terme = serde_json::from_str(r#"{"hanzi":"长城","trad":"長城","pinyin":"Chángchéng","sens":"x"}"#).unwrap();
        assert_eq!((t.caracteres.as_str(), t.variante.as_str(), t.mot.as_str()), ("长城", "長城", "Chángchéng"));
    }

    #[test]
    fn formes() {
        let d = Dictionnaire {
            nom: "x".into(),
            termes: vec![
                Terme { mot: "Chángchéng".into(), caracteres: "长城".into(), ..Default::default() },
                Terme { mot: "Côte-d'Ivoire".into(), ..Default::default() },
            ],
        };
        assert_eq!(d.formes().collect::<Vec<_>>(), ["Chángchéng", "Changcheng", "长城", "Côte-d'Ivoire"]);
    }

    #[test]
    fn accents() {
        assert_eq!(sans_accents("Chángchéng Lǜ"), "Changcheng Lu");
    }
}
