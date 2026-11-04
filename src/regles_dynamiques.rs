//! Gestionnaire de règles grammaticales dynamiques et personnalisées.
//! Permet au moteur Rust d'apprendre automatiquement de nouvelles règles
//! détectées par le modèle local Qwen 2.5 sur votre GPU, et de les appliquer
//! instantanément (< 1 ms) lors de la frappe en temps réel.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegleDynamique {
    pub id: String,
    pub motif_fautif: String,
    pub suggestion: String,
    pub regle: String,
    pub explication: String,
    pub mot_entier: bool,
    pub active: bool,
    pub date_ajout: String,
    /// Mot qui doit précéder le motif (règles sur mots courts ambigus : « a » → « à »).
    #[serde(default)]
    pub contexte_avant: String,
    /// Nombre de fois où l'IA a proposé cette correction (confiance).
    #[serde(default = "un")]
    pub vues: u32,
    /// Nombre de fois où l'utilisateur a ignoré l'alerte produite par cette règle.
    #[serde(default)]
    pub rejets: u32,
}

fn un() -> u32 {
    1
}

/// Confirmations nécessaires avant qu'une règle apprise automatiquement ne s'applique.
pub const SEUIL_CONFIANCE: u32 = 2;
/// Rejets utilisateur au-delà desquels une règle est désactivée.
pub const SEUIL_REJETS: u32 = 2;

impl RegleDynamique {
    pub fn est_applicable(&self) -> bool {
        self.active && self.vues >= SEUIL_CONFIANCE && self.rejets < SEUIL_REJETS
    }
}

/// Un motif d'un seul mot court est trop ambigu pour être appliqué sans contexte.
fn exige_contexte(motif: &str) -> bool {
    !motif.contains(' ') && motif.chars().count() <= 4
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BanqueRegles {
    pub regles: Vec<RegleDynamique>,
    pub apprentissage_automatique: bool,
}

impl Default for BanqueRegles {
    fn default() -> Self {
        Self {
            regles: Vec::new(),
            apprentissage_automatique: true,
        }
    }
}

fn dossier_app() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("Texteur")
}

fn chemin_fichier_regles() -> PathBuf {
    dossier_app().join("regles_ia.json")
}

impl BanqueRegles {
    /// Charge les règles apprises par l'IA depuis le disque.
    pub fn charge() -> Self {
        let f = chemin_fichier_regles();
        if let Ok(s) = std::fs::read_to_string(&f) {
            if let Ok(b) = serde_json::from_str(&s) {
                return b;
            }
        }
        let b = Self::default();
        let _ = b.sauve();
        b
    }

    /// Sauvegarde les règles sur le disque.
    pub fn sauve(&self) -> Result<(), String> {
        if cfg!(test) {
            return Ok(()); // les tests ne doivent jamais écraser les règles de l'utilisateur
        }
        let f = chemin_fichier_regles();
        if let Some(p) = f.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&f, json).map_err(|e| e.to_string())
    }

    /// Ajout validé par l'utilisateur : la règle est applicable immédiatement.
    /// Retourne `true` si une nouvelle règle a été ajoutée.
    pub fn ajoute_si_absente(
        &mut self,
        motif: &str,
        suggestion: &str,
        nom_regle: &str,
        explication: &str,
    ) -> bool {
        self.ajoute_avec_contexte(motif, suggestion, nom_regle, explication, "", SEUIL_CONFIANCE)
    }

    /// Apprentissage automatique : chaque proposition de l'IA renforce la confiance ;
    /// la règle ne s'applique qu'après SEUIL_CONFIANCE observations.
    /// Retourne `true` si une règle vient de devenir applicable.
    pub fn observer_candidat(&mut self, c: &Candidat) -> bool {
        let (motif, sugg, ctx) = (c.motif.trim(), c.suggestion.trim(), c.contexte_avant.trim());
        if let Some(r) = self.regles.iter_mut().find(|r| {
            r.motif_fautif.to_lowercase() == motif.to_lowercase() && r.contexte_avant.to_lowercase() == ctx.to_lowercase()
        }) {
            if r.suggestion.to_lowercase() != sugg.to_lowercase() {
                return false;
            }
            let avant = r.est_applicable();
            r.vues = r.vues.saturating_add(1);
            let devient = !avant && r.est_applicable();
            let _ = self.sauve();
            return devient;
        }
        self.ajoute_avec_contexte(motif, sugg, &c.regle, &c.explication, ctx, 1);
        false
    }

    /// Validation explicite d'un candidat par l'utilisateur (contexte conservé).
    pub fn valider_candidat(&mut self, c: &Candidat) -> bool {
        self.ajoute_avec_contexte(&c.motif, &c.suggestion, &c.regle, &c.explication, &c.contexte_avant, SEUIL_CONFIANCE)
    }

    /// L'utilisateur a ignoré une alerte issue d'une règle apprise : on la pénalise.
    pub fn signaler_rejet(&mut self, texte_fautif: &str) {
        let t = texte_fautif.to_lowercase();
        let mut change = false;
        for r in self.regles.iter_mut().filter(|r| r.motif_fautif.to_lowercase() == t) {
            r.rejets += 1;
            if r.rejets >= SEUIL_REJETS {
                r.active = false;
            }
            change = true;
        }
        if change {
            let _ = self.sauve();
        }
    }

    fn prochain_id(&self) -> String {
        let max = self.regles.iter().filter_map(|r| r.id.strip_prefix("ia_")?.parse::<u32>().ok()).max().unwrap_or(0);
        format!("ia_{}", max + 1)
    }

    fn ajoute_avec_contexte(
        &mut self,
        motif: &str,
        suggestion: &str,
        nom_regle: &str,
        explication: &str,
        contexte_avant: &str,
        vues: u32,
    ) -> bool {
        let motif_clean = motif.trim();
        let sugg_clean = suggestion.trim();

        if motif_clean.is_empty() || sugg_clean.is_empty() {
            return false;
        }

        // Éviter les règles identiques ou triviales
        if motif_clean.eq_ignore_ascii_case(sugg_clean) {
            return false;
        }

        // Éviter de créer des règles sur de longs paragraphes entiers
        if motif_clean.split_whitespace().count() > 6 || sugg_clean.split_whitespace().count() > 6 {
            return false;
        }

        // Mot court ambigu sans contexte : trop risqué (« a » → « à » partout)
        if contexte_avant.is_empty() && vues < SEUIL_CONFIANCE && exige_contexte(motif_clean) {
            return false;
        }

        // Vérifier si une règle sur ce motif (et ce contexte) existe déjà
        if self.regles.iter().any(|r| {
            r.motif_fautif.to_lowercase() == motif_clean.to_lowercase()
                && r.contexte_avant.to_lowercase() == contexte_avant.to_lowercase()
        }) {
            return false;
        }

        let id = self.prochain_id();
        let regle_nom = if nom_regle.trim().is_empty() {
            "Règle apprise par Qwen".to_string()
        } else {
            nom_regle.trim().to_string()
        };

        let exp = if explication.trim().is_empty() {
            format!(
                "Remplacer « {} » par « {} » (règle apprise par l'IA locale Qwen 2.5).",
                motif_clean, sugg_clean
            )
        } else {
            explication.trim().to_string()
        };

        let nouvelle = RegleDynamique {
            id,
            motif_fautif: motif_clean.to_string(),
            suggestion: sugg_clean.to_string(),
            regle: regle_nom,
            explication: exp,
            mot_entier: true,
            active: true,
            date_ajout: format!("Règle Qwen #{}", self.regles.len() + 1),
            contexte_avant: contexte_avant.to_string(),
            vues,
            rejets: 0,
        };

        self.regles.push(nouvelle);
        let _ = self.sauve();
        true
    }

    /// Active ou désactive une règle par son identifiant.
    pub fn basculer_regle(&mut self, id: &str) {
        if let Some(r) = self.regles.iter_mut().find(|r| r.id == id) {
            r.active = !r.active;
            if r.active {
                // Réactivation manuelle = validation explicite par l'utilisateur
                r.rejets = 0;
                r.vues = r.vues.max(SEUIL_CONFIANCE);
            }
            let _ = self.sauve();
        }
    }

    /// Supprime une règle par son identifiant.
    pub fn supprimer_regle(&mut self, id: &str) {
        self.regles.retain(|r| r.id != id);
        let _ = self.sauve();
    }
}

/// Correction candidate extraite d'une réponse de l'IA.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Candidat {
    pub motif: String,
    pub suggestion: String,
    pub regle: String,
    pub explication: String,
    #[serde(default)]
    pub contexte_avant: String,
}

fn nettoie(m: &str) -> String {
    m.trim_matches(|c: char| c.is_ascii_punctuation() || c == '«' || c == '»' || c == '…').to_lowercase()
}

fn contient_cjk(s: &str) -> bool {
    s.chars().any(crate::masque::est_cjk)
}

/// Extrait les règles candidates en alignant (plus longue sous-séquence commune)
/// le texte initial et le texte corrigé par l'IA, puis en isolant chaque bloc modifié.
pub fn extraire_regles_candidats(texte_initial: &str, texte_corrige: &str, explications: &[String]) -> Vec<Candidat> {
    let a: Vec<&str> = texte_initial.split_whitespace().collect();
    let b: Vec<&str> = texte_corrige.split_whitespace().collect();
    let (na, nb) = (a.len(), b.len());
    // Garde-fou mémoire : au-delà, l'IA a probablement réécrit le texte en entier
    if na == 0 || nb == 0 || na * nb > 4_000_000 {
        return Vec::new();
    }
    let ka: Vec<String> = a.iter().map(|m| nettoie(m)).collect();
    let kb: Vec<String> = b.iter().map(|m| nettoie(m)).collect();

    let mut lcs = vec![vec![0u32; nb + 1]; na + 1];
    for i in (0..na).rev() {
        for j in (0..nb).rev() {
            lcs[i][j] = if ka[i] == kb[j] { lcs[i + 1][j + 1] + 1 } else { lcs[i + 1][j].max(lcs[i][j + 1]) };
        }
    }

    let mut candidats: Vec<Candidat> = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < na || j < nb {
        if i < na && j < nb && ka[i] == kb[j] {
            i += 1;
            j += 1;
            continue;
        }
        let (di, dj) = (i, j);
        while (i < na || j < nb) && !(i < na && j < nb && ka[i] == kb[j]) {
            if j < nb && (i >= na || lcs[i][j + 1] >= lcs[i + 1][j]) {
                j += 1;
            } else {
                i += 1;
            }
        }
        // Les ajouts purs (mot inséré) sont rattachés au mot précédent pour garder un motif non vide
        let (di, dj) = if di == i && di > 0 && dj > 0 { (di - 1, dj - 1) } else { (di, dj) };
        if di == i || i - di > 4 || j - dj > 4 {
            continue;
        }
        let motif = a[di..i].join(" ").trim_matches(|c: char| c.is_ascii_punctuation()).to_string();
        let sugg = b[dj..j].join(" ").trim_matches(|c: char| c.is_ascii_punctuation()).to_string();
        if motif.is_empty() || sugg.is_empty() || motif.to_lowercase() == sugg.to_lowercase() {
            continue;
        }
        // Jamais de règle sur l'écriture chinoise ni sur des nombres (dates, montants)
        if contient_cjk(&motif) || contient_cjk(&sugg) || motif.chars().any(|c| c.is_ascii_digit()) {
            continue;
        }
        let contexte_avant = if exige_contexte(&motif) && di > 0 { nettoie(a[di - 1]) } else { String::new() };
        let m_low = motif.to_lowercase();
        let explication = explications
            .iter()
            .find(|e| e.to_lowercase().contains(&m_low))
            .cloned()
            .unwrap_or_else(|| format!("Correction suggérée par l'IA : « {} » → « {} ».", motif, sugg));
        let c = Candidat { motif, suggestion: sugg, regle: "Règle apprise par Qwen".to_string(), explication, contexte_avant };
        if !candidats.contains(&c) {
            candidats.push(c);
        }
    }
    candidats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignement_lcs_et_contexte() {
        let c = extraire_regles_candidats(
            "Il a parler a son frère de la dilemne.",
            "Il a parlé à son frère du dilemme.",
            &[],
        );
        let paires: Vec<(&str, &str, &str)> =
            c.iter().map(|c| (c.motif.as_str(), c.suggestion.as_str(), c.contexte_avant.as_str())).collect();
        assert!(paires.contains(&("parler a", "parlé à", "")));
        assert!(paires.iter().any(|p| p.1 == "du dilemme"));
    }

    #[test]
    fn confiance_et_rejets() {
        let mut b = BanqueRegles { regles: Vec::new(), apprentissage_automatique: true };
        let c = Candidat {
            motif: "a".into(),
            suggestion: "à".into(),
            regle: "x".into(),
            explication: "y".into(),
            contexte_avant: "parlé".into(),
        };
        assert!(!b.observer_candidat(&c));
        assert!(!b.regles[0].est_applicable());
        assert!(b.observer_candidat(&c));
        assert!(b.regles[0].est_applicable());
        b.signaler_rejet("a");
        b.signaler_rejet("a");
        assert!(!b.regles[0].est_applicable());
    }

    #[test]
    fn ignore_le_chinois() {
        let c = extraire_regles_candidats("Le fournisseur 深圳 livre", "Le fournisseur 深圳市 livre", &[]);
        assert!(c.is_empty());
    }
}