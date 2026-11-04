//! Moteur de détection des répétitions et de suggestions de style en Rust.
//! Filtre strictement les mots grammaticaux (stopwords) pour n'alerter que sur les mots pleins
//! et propose des alternatives élégantes et des cooccurrences naturelles.

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Repetition {
    pub mot: String,
    pub racine: String,
    pub occurrences: Vec<(usize, usize)>, // (debut, fin) en indices de caractères
    pub nombre: usize,
    pub alternatives: Vec<String>,
}

pub struct AnalyseurStyle {
    stopwords: HashSet<&'static str>,
    alternatives_qualite: HashMap<&'static str, Vec<&'static str>>,
}

impl Default for AnalyseurStyle {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalyseurStyle {
    pub fn new() -> Self {
        // Liste exhaustive des mots fonctionnels / grammaticaux français à ne JAMAIS marquer comme répétition
        let mut stops = HashSet::new();
        stops.extend([
            // Articles & déterminants
            "le", "la", "les", "l'", "un", "une", "des", "du", "de", "d'", "au", "aux",
            "ce", "cet", "cette", "ces", "mon", "ma", "mes", "ton", "ta", "tes",
            "son", "sa", "ses", "notre", "nos", "votre", "vos", "leur", "leurs",
            "chaque", "quelque", "quelques", "plusieurs", "aucun", "aucune", "tout", "tous", "toute", "toutes",
            // Pronoms
            "je", "tu", "il", "elle", "on", "nous", "vous", "ils", "elles",
            "me", "te", "se", "lui", "eux", "moi", "toi", "soi", "en", "y",
            "qui", "que", "qu'", "quoi", "dont", "où", "lequel", "laquelle", "lesquels", "lesquelles",
            "ceci", "cela", "ça", "celui", "celle", "ceux", "celles",
            // Prépositions
            "à", "de", "dans", "par", "pour", "vers", "avec", "sans", "sous", "sur",
            "chez", "entre", "envers", "hors", "jusque", "jusqu'", "selon", "contre",
            "pendant", "depuis", "durant", "avant", "après", "dès", "devant", "derrière",
            // Conjonctions & adverbes grammaticaux
            "et", "ou", "mais", "donc", "or", "ni", "car", "si", "comme", "quand",
            "lorsque", "puisque", "quoique", "alors", "ainsi", "bien", "très", "trop",
            "plus", "moins", "aussi", "encore", "toujours", "jamais", "ne", "pas", "point",
            "guère", "non", "oui", "ci", "là", "tant", "autant", "même", "ici",
            // Verbes auxiliaires très fréquents
            "est", "sont", "été", "être", "suis", "es", "sommes", "êtes", "était", "étaient",
            "a", "ont", "eu", "avoir", "ai", "as", "avons", "avez", "avait", "avaient",
        ]);

        // Suggestions de formulations élégantes et précises (cooccurrences et vrais synonymes)
        let mut alts = HashMap::new();
        alts.insert("faire", vec!["effectuer", "réaliser", "accomplir", "concevoir", "établir"]);
        alts.insert("fait", vec!["réalise", "conçoit", "établit", "effectue", "produit"]);
        alts.insert("faire une réunion", vec!["tenir une réunion", "organiser une réunion", "animer une séance"]);
        alts.insert("faire attention", vec!["veiller à", "prendre garde", "redoubler de vigilance"]);
        alts.insert("problème", vec!["difficulté", "écueil", "obstacle", "anomalie", "défi"]);
        alts.insert("problèmes", vec!["difficultés", "obstacles", "anomalies", "écueils"]);
        alts.insert("important", vec!["majeur", "primordial", "déterminant", "crucial", "considérable"]);
        alts.insert("importante", vec!["majeure", "primordiale", "déterminante", "cruciale"]);
        alts.insert("importants", vec!["majeurs", "primordiaux", "déterminants", "cruciaux"]);
        alts.insert("importantes", vec!["majeures", "primordiales", "déterminantes", "cruciales"]);
        alts.insert("dire", vec!["indiquer", "préciser", "mentionner", "affirmer", "souligner"]);
        alts.insert("dit", vec!["indique", "précise", "mentionne", "affirme", "souligne"]);
        alts.insert("voir", vec!["constater", "observer", "remarquer", "examiner", "analyser"]);
        alts.insert("trouver", vec!["identifier", "repérer", "déceler", "découvrir", "dégager"]);
        alts.insert("donner", vec!["fournir", "transmettre", "accorder", "attribuer", "conférer"]);
        alts.insert("mettre", vec!["placer", "disposer", "installer", "intégrer", "introduire"]);
        alts.insert("chose", vec!["élément", "aspect", "point", "détail", "facteur"]);
        alts.insert("choses", vec!["éléments", "aspects", "points", "facteurs"]);
        alts.insert("projet", vec!["programme", "initiative", "démarche", "chantier", "entreprise"]);
        alts.insert("travail", vec!["activité", "mission", "tâche", "tâche assignée", "ouvrage"]);
        alts.insert("temps", vec!["délai", "période", "durée", "échéance"]);
        alts.insert("aide", vec!["soutien", "assistance", "appui", "concours"]);
        alts.insert("aider", vec!["épauler", "soutenir", "assister", "accompagner"]);
        alts.insert("changer", vec!["modifier", "ajuster", "transformer", "faire évoluer"]);
        alts.insert("changement", vec!["évolution", "ajustement", "mutation", "transition"]);
        alts.insert("commencer", vec!["engager", "entamer", "amorcer", "initier"]);
        alts.insert("finir", vec!["achever", "clore", "finaliser", "conclure"]);
        alts.insert("bon", vec!["favorable", "judicieux", "pertinent", "opportun"]);
        alts.insert("bonne", vec!["favorable", "judicieuse", "pertinente", "opportune"]);
        alts.insert("mauvais", vec!["défavorable", "inapproprié", "inadéquat", "critique"]);
        alts.insert("rapide", vec!["prompt", "diligent", "immédiat", "expéditif"]);
        alts.insert("vite", vec!["rapidement", "sans délai", "prestement"]);
        alts.insert("document", vec!["dossier", "rapport", "fichier", "support"]);
        alts.insert("fournisseur", vec!["prestataire", "partenaire", "sous-traitant"]);
        alts.insert("fournisseurs", vec!["prestataires", "partenaires", "sous-traitants"]);
        alts.insert("réunion", vec!["séance", "entrevue", "rencontre", "point", "comité"]);
        alts.insert("point", vec!["bilan", "état des lieux", "synthèse", "étape"]);
        alts.insert("livraison", vec!["fourniture", "remise", "expédition", "délivrance"]);
        alts.insert("décision", vec!["arbitrage", "résolution", "choix", "orientation"]);
        alts.insert("objectif", vec!["but", "finalité", "cible", "ambition"]);

        Self {
            stopwords: stops,
            alternatives_qualite: alts,
        }
    }

    /// Détecte les répétitions de mots pleins dans le texte.
    /// Renvoie la liste des groupes de répétitions triée de façon 100% stable et déterministe.
    pub fn analyse(&self, texte: &str) -> Vec<Repetition> {
        let mut occurrences_par_racine: HashMap<String, (String, Vec<(usize, usize)>)> = HashMap::new();

        let car: Vec<char> = texte.chars().collect();
        let mut debut_mot = None;

        for i in 0..car.len() {
            let c = car[i];
            if c.is_alphanumeric() || c == '-' {
                if debut_mot.is_none() {
                    debut_mot = Some(i);
                }
            } else if let Some(deb) = debut_mot {
                let mot_brut: String = car[deb..i].iter().collect();
                self.enregistre_mot(&mot_brut, deb, i, &mut occurrences_par_racine);
                debut_mot = None;
            }
        }
        if let Some(deb) = debut_mot {
            let mot_brut: String = car[deb..car.len()].iter().collect();
            self.enregistre_mot(&mot_brut, deb, car.len(), &mut occurrences_par_racine);
        }

        // Filtrer : ne garder que les mots répétés au moins 2 fois
        let mut resultat = Vec::new();
        for (racine, (mot_affiche, mut occs)) in occurrences_par_racine {
            if occs.len() >= 2 {
                occs.sort_by_key(|&(deb, _)| deb);
                let alternatives = self.trouve_alternatives(&racine);
                resultat.push(Repetition {
                    mot: mot_affiche,
                    racine: racine.clone(),
                    nombre: occs.len(),
                    occurrences: occs,
                    alternatives,
                });
            }
        }

        // Tri 100% stable et déterministe :
        // 1. Nombre d'occurrences décroissant
        // 2. Première apparition dans le texte (croissant)
        // 3. Ordre alphabétique de la racine
        resultat.sort_by(|a, b| {
            b.nombre.cmp(&a.nombre)
                .then_with(|| {
                    let pos_a = a.occurrences.first().map(|o| o.0).unwrap_or(0);
                    let pos_b = b.occurrences.first().map(|o| o.0).unwrap_or(0);
                    pos_a.cmp(&pos_b)
                })
                .then_with(|| a.racine.cmp(&b.racine))
        });
        resultat
    }

    fn enregistre_mot(
        &self,
        mot_brut: &str,
        deb: usize,
        fin: usize,
        table: &mut HashMap<String, (String, Vec<(usize, usize)>)>,
    ) {
        let minuscule = mot_brut.to_lowercase();
        // Ignorer les mots trop courts ou dans les stopwords
        if minuscule.len() < 3 || self.stopwords.contains(minuscule.as_str()) {
            return;
        }

        // Lemmatisation basique de surface : enlever le 's' pluriel final
        let racine = if minuscule.ends_with('s') && !["temps", "corps", "cours", "mois", "pays", "poids", "sens", "prix", "voix", "choix"].contains(&minuscule.as_str()) {
            minuscule[..minuscule.len() - 1].to_string()
        } else {
            minuscule.clone()
        };

        let entree = table.entry(racine).or_insert_with(|| (minuscule, Vec::new()));
        entree.1.push((deb, fin));
    }

    fn trouve_alternatives(&self, racine: &str) -> Vec<String> {
        if let Some(alts) = self.alternatives_qualite.get(racine) {
            return alts.iter().map(|s| s.to_string()).collect();
        }
        // Recherche déterministe (clé la plus spécifique en premier, ordre stable)
        let mut matches: Vec<(&str, &Vec<&str>)> = self.alternatives_qualite.iter()
            .filter(|(k, _)| racine.starts_with(*k) || k.starts_with(racine))
            .map(|(&k, v)| (k, v))
            .collect();
        matches.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(b.0)));
        if let Some((_, alts)) = matches.first() {
            return alts.iter().map(|s| s.to_string()).collect();
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_repetitions() {
        let style = AnalyseurStyle::new();
        let texte = "Nous avons un problème et ce problème est sérieux. Le projet avance bien pour ce projet.";
        let reps = style.analyse(texte);
        assert!(reps.iter().any(|r| r.racine == "problème" && r.nombre == 2));
        assert!(reps.iter().any(|r| r.racine == "projet" && r.nombre == 2));
        // Les stopwords comme "ce", "et", "est", "un", "pour" ne doivent PAS être dans les répétitions
        assert!(!reps.iter().any(|r| r.racine == "ce"));
        assert!(!reps.iter().any(|r| r.racine == "et"));
    }

    #[test]
    fn test_suggestions_qualite() {
        let style = AnalyseurStyle::new();
        let alts = style.trouve_alternatives("problème");
        assert!(alts.contains(&"difficulté".to_string()));
        assert!(alts.contains(&"obstacle".to_string()));
    }
}
