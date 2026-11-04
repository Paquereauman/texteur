import os

grammaire_code = r'''//! Moteur de règles grammaticales françaises 100 % local et autonome en Rust.
//! Couvre les accords (sujet-verbe, participe passé avec être, participe passé avec avoir et COD antéposé),
//! les adjectifs de couleur invariables, les homophones majeurs (a/à, ce/se, c'est/s'est, ou/où, leur/leurs, si il),
//! les confusions infinitif/participe (-er/-é) et les doublons.

use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum GenreErreur {
    Grammaire,
    Accord,
    ParticipePasse,
    Infinitif,
    Homophone,
    Doublon,
}

#[derive(Clone, Debug)]
pub struct AlerteGrammaticale {
    pub debut: usize, // indice de caractère
    pub fin: usize,
    pub texte_fautif: String,
    pub regle: String,
    pub explication: String,
    pub suggestions: Vec<String>,
    pub genre: GenreErreur,
}

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub mot: &'a str,
    pub deb: usize, // indice de caractère dans le paragraphe
    pub fin: usize, // indice de caractère dans le paragraphe
}

pub struct CorrecteurGrammaire {
    auxiliaires_avoir: HashSet<&'static str>,
    auxiliaires_etre: HashSet<&'static str>,
    prepositions_infinitif: HashSet<&'static str>,
    determinants_pluriel: HashSet<&'static str>,
    determinants_singulier_masc: HashSet<&'static str>,
    determinants_singulier_fem: HashSet<&'static str>,
    noms_feminins_courants: HashSet<&'static str>,
}

impl Default for CorrecteurGrammaire {
    fn default() -> Self {
        Self::new()
    }
}

impl CorrecteurGrammaire {
    pub fn new() -> Self {
        let mut avoir = HashSet::new();
        avoir.extend([
            "ai", "as", "a", "avons", "avez", "ont",
            "avais", "avait", "avions", "aviez", "avaient",
            "aurai", "auras", "aura", "aurons", "aurez", "auront",
            "eut", "eurent", "eu", "ayant",
        ]);

        let mut etre = HashSet::new();
        etre.extend([
            "suis", "es", "est", "sommes", "êtes", "sont",
            "étais", "était", "étions", "étiez", "étaient",
            "serai", "seras", "sera", "serons", "serez", "seront",
            "fut", "furent", "été", "étant",
        ]);

        let mut prep_inf = HashSet::new();
        prep_inf.extend([
            "pour", "afin de", "de", "d'", "sans", "avant de", "à", "au lieu de",
            "en train de", "façon de", "histoire de", "par",
        ]);

        let mut det_plur = HashSet::new();
        det_plur.extend([
            "les", "des", "ces", "mes", "tes", "ses", "nos", "vos", "leurs",
            "plusieurs", "quelques", "tous les", "toutes les", "deux", "trois", "quatre", "cinq",
        ]);

        let mut det_sing_masc = HashSet::new();
        det_sing_masc.extend(["le", "l'", "un", "ce", "cet", "mon", "ton", "son", "notre", "votre", "leur", "chaque"]);

        let mut det_sing_fem = HashSet::new();
        det_sing_fem.extend(["la", "l'", "une", "cette", "ma", "ta", "sa", "notre", "votre", "leur", "chaque"]);

        let mut noms_fem = HashSet::new();
        noms_fem.extend([
            "fille", "femme", "lettre", "lettres", "erreur", "erreurs", "pomme", "pommes",
            "photo", "photos", "lunette", "lunettes", "robe", "robes", "chaussure", "chaussures",
            "voiture", "voitures", "maison", "maisons", "décision", "décisions", "idée", "idées",
            "solution", "solutions", "fleur", "fleurs", "question", "questions", "affaire", "affaires",
            "commande", "commandes", "demande", "demandes", "réunion", "réunions", "remarque", "remarques",
            "personne", "personnes", "page", "pages", "version", "versions", "copie", "copies",
            "règle", "règles", "pièce", "pièces", "porte", "portes", "fenêtre", "fenêtres",
            "table", "tables", "semaine", "semaines", "année", "années", "heure", "heures",
            "chanson", "chansons", "histoire", "histoires", "phrase", "phrases", "étape", "étapes",
        ]);

        Self {
            auxiliaires_avoir: avoir,
            auxiliaires_etre: etre,
            prepositions_infinitif: prep_inf,
            determinants_pluriel: det_plur,
            determinants_singulier_masc: det_sing_masc,
            determinants_singulier_fem: det_sing_fem,
            noms_feminins_courants: noms_fem,
        }
    }

    /// Analyse un texte complet et retourne toutes les alertes grammaticales avec leurs positions en caractères.
    pub fn analyse(&self, texte: &str) -> Vec<AlerteGrammaticale> {
        let mut alertes = Vec::new();

        let mut decalage_ligne = 0;
        for ligne in texte.split('\n') {
            self.analyse_paragraphe(ligne, decalage_ligne, &mut alertes);
            decalage_ligne += ligne.chars().count() + 1; // +1 pour le '\n'
        }

        alertes.sort_by_key(|a| a.debut);
        alertes
    }

    fn analyse_paragraphe(&self, para: &str, decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        if para.trim().is_empty() {
            return;
        }

        let tokens = tokenize(para);
        if tokens.is_empty() {
            return;
        }

        // ====================================================================
        // RÈGLE 1 : MOTS RÉPÉTÉS CONSÉCUTIFS (DOUBLONS : « la la », « de de »)
        // ====================================================================
        for w in tokens.windows(2) {
            let t1 = &w[0];
            let t2 = &w[1];
            let n1 = t1.mot.to_lowercase();
            let n2 = t2.mot.to_lowercase();
            if n1 == n2 && !n1.is_empty() && [
                "la", "le", "les", "de", "des", "du", "un", "une", "que", "qui", "et", "en", "à", "dans", "pour"
            ].contains(&n1.as_str()) {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t1.deb,
                    fin: decalage + t2.fin,
                    texte_fautif: format!("{} {}", t1.mot, t2.mot),
                    regle: "Mot répété (doublon)".into(),
                    explication: format!("Le mot « {} » est répété consécutivement.", t1.mot),
                    suggestions: vec![t1.mot.to_string()],
                    genre: GenreErreur::Doublon,
                });
            }
        }

        // ====================================================================
        // RÈGLE 2 : ACCORD DU PARTICIPE PASSÉ AVEC COD ANTÉPOSÉ (AVOIR)
        // Exemple : « La fille que tu as vu » -> « vue »
        // Exemple : « Les lettres que j'ai écrit » -> « écrites »
        // Exemple : « Je les ai vu » -> « vus »
        // ====================================================================
        self.verifier_participe_passe_cod_antepose(para, &tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 3 : ADJECTIFS DE COULEUR INVARIABLES (marron, orange, kaki...)
        // Exemple : « Les lunettes marrons » -> « marron »
        // Exemple : « Des yeux noisettes » -> « noisette »
        // ====================================================================
        self.verifier_adjectifs_couleur_invariables(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 4 : HOMOPHONES MAJEURS & CONFUSIONS FRÉQUENTES
        // a / à, ce / se, c'est / s'est, ou / où, leur / leurs, si il, etc.
        // ====================================================================
        self.verifier_homophones(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 5 : AUXILIAIRE AVOIR + INFINITIF EN -ER AU LIEU DE PARTICIPE EN -É
        // Exemple : « ont confirmer » -> « confirmé »
        // ====================================================================
        for i in 0..tokens.len().saturating_sub(1) {
            let t_aux = &tokens[i];
            let aux_norm = t_aux.mot.to_lowercase();
            if self.auxiliaires_avoir.contains(aux_norm.as_str()) {
                let cible_idx = if i + 2 < tokens.len() && [
                    "bien", "déjà", "souvent", "toujours", "vite", "beaucoup", "encore", "clairement", "rapidement"
                ].contains(&tokens[i + 1].mot.to_lowercase().as_str()) {
                    i + 2
                } else {
                    i + 1
                };

                let t_v = &tokens[cible_idx];
                let v_norm = t_v.mot.to_lowercase();
                if v_norm.ends_with("er") && v_norm.len() > 3 && ![
                    "danger", "hiver", "enfer", "mer", "fer", "cher", "fier", "hier"
                ].contains(&v_norm.as_str()) {
                    let suggestion = format!("{}é", &t_v.mot[..t_v.mot.len() - 2]);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_v.deb,
                        fin: decalage + t_v.fin,
                        texte_fautif: t_v.mot.to_string(),
                        regle: "Participe passé après avoir".into(),
                        explication: format!(
                            "Après l'auxiliaire avoir (« {} »), le verbe doit être au participe passé (« {} ») et non à l'infinitif.",
                            t_aux.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::ParticipePasse,
                    });
                }
            }
        }

        // ====================================================================
        // RÈGLE 6 : AUXILIAIRE ÊTRE + INFINITIF EN -ER AU LIEU DE PARTICIPE EN -É
        // Exemple : « sera effectuer » -> « effectué »
        // ====================================================================
        for i in 0..tokens.len().saturating_sub(1) {
            let t_aux = &tokens[i];
            let aux_norm = t_aux.mot.to_lowercase();
            if self.auxiliaires_etre.contains(aux_norm.as_str()) {
                let cible_idx = if i + 2 < tokens.len() && [
                    "bien", "déjà", "souvent", "toujours", "vite", "beaucoup", "encore"
                ].contains(&tokens[i + 1].mot.to_lowercase().as_str()) {
                    i + 2
                } else {
                    i + 1
                };

                let t_v = &tokens[cible_idx];
                let v_norm = t_v.mot.to_lowercase();
                if v_norm.ends_with("er") && v_norm.len() > 3 && ![
                    "danger", "hiver", "enfer", "mer", "fer", "cher", "hier"
                ].contains(&v_norm.as_str()) {
                    let racine = &t_v.mot[..t_v.mot.len() - 2];
                    let suggestions = vec![
                        format!("{}é", racine),
                        format!("{}ée", racine),
                        format!("{}és", racine),
                        format!("{}ées", racine),
                    ];
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_v.deb,
                        fin: decalage + t_v.fin,
                        texte_fautif: t_v.mot.to_string(),
                        regle: "Participe passé avec être".into(),
                        explication: format!(
                            "Après l'auxiliaire être (« {} »), le verbe doit être un participe passé accordé avec le sujet.",
                            t_aux.mot
                        ),
                        suggestions,
                        genre: GenreErreur::ParticipePasse,
                    });
                }
            }
        }

        // ====================================================================
        // RÈGLE 7 : ACCORD PARTICIPE PASSÉ AVEC ÊTRE (SUJET-VERBE)
        // Exemple : « elle est parti » -> « partie »
        // Exemple : « ils sont arrivé » -> « arrivés »
        // ====================================================================
        self.verifier_participe_passe_avec_etre(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 8 : PRÉPOSITION + PARTICIPE EN -É AU LIEU DE L'INFINITIF EN -ER
        // Exemple : « pour mangé » -> « pour manger »
        // ====================================================================
        for i in 0..tokens.len().saturating_sub(1) {
            let t_prep = &tokens[i];
            let prep_norm = t_prep.mot.to_lowercase();
            if self.prepositions_infinitif.contains(prep_norm.as_str()) {
                let t_suiv = &tokens[i + 1];
                let s_norm = t_suiv.mot.to_lowercase();
                if (s_norm.ends_with('é') || s_norm.ends_with("és") || s_norm.ends_with("ée") || s_norm.ends_with("ées")) && s_norm.len() > 3 {
                    let racine = if s_norm.ends_with("ées") {
                        &t_suiv.mot[..t_suiv.mot.len() - 4]
                    } else if s_norm.ends_with("ée") || s_norm.ends_with("és") {
                        &t_suiv.mot[..t_suiv.mot.len() - 3]
                    } else {
                        &t_suiv.mot[..t_suiv.mot.len() - 2]
                    };
                    let suggestion = format!("{}er", racine);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_suiv.deb,
                        fin: decalage + t_suiv.fin,
                        texte_fautif: t_suiv.mot.to_string(),
                        regle: "Infinitif après préposition".into(),
                        explication: format!(
                            "Après la préposition « {} », le verbe se met à l'infinitif (« {} »).",
                            t_prep.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Infinitif,
                    });
                }
            }
        }

        // ====================================================================
        // RÈGLE 9 : DÉTERMINANT PLURIEL SUIVI D'UN NOM SINGULIER
        // Exemple : « Les fournisseur » -> « fournisseurs »
        // ====================================================================
        for i in 0..tokens.len().saturating_sub(1) {
            let t_det = &tokens[i];
            let det_norm = t_det.mot.to_lowercase();
            if self.determinants_pluriel.contains(det_norm.as_str()) {
                let t_mot = &tokens[i + 1];
                let m_norm = t_mot.mot.to_lowercase();
                if m_norm.ends_with('s') || m_norm.ends_with('x') || m_norm.ends_with('z') || m_norm.len() <= 2 {
                    continue;
                }
                // Si c'est un infinitif verbal (ex: « leurs donner »), ce n'est pas un nom à pluraliser
                if est_infinitif(&m_norm) {
                    continue;
                }
                // Si ce n'est pas un verbe évidemment conjugué
                if ![
                    "est", "sont", "ont", "a", "fait", "peut", "doit", "vont", "sera", "seront",
                    "marron", "orange", "kaki", "turquoise"
                ].contains(&m_norm.as_str()) {
                    let suggestion = if m_norm.ends_with("al") && ![
                        "banal", "fatal", "naval", "bancal", "natal"
                    ].contains(&m_norm.as_str()) {
                        format!("{}aux", &t_mot.mot[..t_mot.mot.len() - 2])
                    } else if m_norm.ends_with("eau") || m_norm.ends_with("eu") {
                        format!("{}x", t_mot.mot)
                    } else {
                        format!("{}s", t_mot.mot)
                    };

                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_mot.deb,
                        fin: decalage + t_mot.fin,
                        texte_fautif: t_mot.mot.to_string(),
                        regle: "Accord du pluriel".into(),
                        explication: format!(
                            "Après le déterminant pluriel « {} », le nom doit s'accorder au pluriel (« {} »).",
                            t_det.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                }
            }
        }

        // ====================================================================
        // RÈGLE 10 : DÉTERMINANT SINGULIER SUIVI D'UN MOT AU PLURIEL
        // Exemple : « un documents » -> « document »
        // ====================================================================
        for i in 0..tokens.len().saturating_sub(1) {
            let t_det = &tokens[i];
            let det_norm = t_det.mot.to_lowercase();
            if self.determinants_singulier_masc.contains(det_norm.as_str()) || self.determinants_singulier_fem.contains(det_norm.as_str()) {
                let t_mot = &tokens[i + 1];
                let m_norm = t_mot.mot.to_lowercase();
                if m_norm.ends_with('s') && m_norm.len() > 3 && ![
                    "mois", "fois", "temps", "corps", "cours", "vers", "poids", "sens", "bras",
                    "cas", "avis", "pays", "choix", "prix", "voix", "refus", "souris", "devis",
                    "toujours", "jamais", "parfois", "ailleurs"
                ].contains(&m_norm.as_str()) {
                    let suggestion = t_mot.mot[..t_mot.mot.len() - 1].to_string();
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_mot.deb,
                        fin: decalage + t_mot.fin,
                        texte_fautif: t_mot.mot.to_string(),
                        regle: "Accord du singulier".into(),
                        explication: format!(
                            "Après le déterminant singulier « {} », le nom doit être au singulier (« {} »).",
                            t_det.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                }
            }
        }

        // ====================================================================
        // RÈGLE 11 : ACCORD SUJET-VERBE (PRONOMS PLURIEL ils, elles)
        // Exemple : « ils mange » -> « mangent »
        // ====================================================================
        for i in 0..tokens.len().saturating_sub(1) {
            let t_suj = &tokens[i];
            let s_norm = t_suj.mot.to_lowercase();
            if s_norm == "ils" || s_norm == "elles" {
                let cible_idx = if i + 2 < tokens.len() && [
                    "lui", "leur", "nous", "vous", "le", "la", "les", "y", "en", "se", "ne"
                ].contains(&tokens[i + 1].mot.to_lowercase().as_str()) {
                    i + 2
                } else {
                    i + 1
                };

                let t_v = &tokens[cible_idx];
                let v_norm = t_v.mot.to_lowercase();
                if (v_norm.ends_with('e') || v_norm.ends_with('a')) && v_norm.len() > 3 && ![
                    "ne", "se", "y", "en", "le", "la", "les", "leur", "lui", "même", "aussi", "bien"
                ].contains(&v_norm.as_str()) {
                    let suggestion = format!("{}nt", t_v.mot);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_v.deb,
                        fin: decalage + t_v.fin,
                        texte_fautif: t_v.mot.to_string(),
                        regle: "Accord sujet-verbe au pluriel".into(),
                        explication: format!(
                            "Après le pronom pluriel « {} », le verbe doit se terminer par -nt (« {} »).",
                            t_suj.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                }
            }
        }
    }

    /// Règle essentielle : Accord du participe passé avec COD antéposé
    /// « La fille que tu as vu » -> « vue »
    /// « Les lettres que j'ai écrit » -> « écrites »
    /// « Les erreurs que vous avez fait » -> « faites »
    /// « Je les ai vu » -> « vus »
    fn verifier_participe_passe_cod_antepose(
        &self,
        _para: &str,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        // Forme 1 : [Déterminant] [Nom] que / qu' [Sujet] [Auxiliaire Avoir] [Participe]
        for i in 0..tokens.len() {
            let t = &tokens[i];
            let t_norm = t.mot.to_lowercase();

            if t_norm == "que" || t_norm == "qu'" {
                // Il nous faut au moins un nom antécédent avant 'que'
                if i == 0 {
                    continue;
                }

                let nom_ant = &tokens[i - 1];
                let nom_lower = nom_ant.mot.to_lowercase();

                // Déterminer le genre et nombre du COD antécédent
                let mut feminin = false;
                let mut pluriel = false;

                if i >= 2 {
                    let det_ant = &tokens[i - 2];
                    let det_lower = det_ant.mot.to_lowercase();

                    if ["la", "une", "cette", "sa", "ta", "ma"].contains(&det_lower.as_str()) {
                        feminin = true;
                        pluriel = false;
                    } else if ["les", "des", "ces", "mes", "tes", "ses", "nos", "vos", "leurs"].contains(&det_lower.as_str()) {
                        pluriel = true;
                        // Est-ce un nom féminin connu ou avec terminaison féminine ?
                        if self.est_nom_feminin(&nom_lower) {
                            feminin = true;
                        }
                    }
                } else if self.est_nom_feminin(&nom_lower) {
                    feminin = true;
                    if nom_lower.ends_with('s') {
                        pluriel = true;
                    }
                }

                // Si l'antécédent est masculin singulier, pas d'accord spécial nécessaire (le participe reste masc. sing.)
                if !feminin && !pluriel {
                    continue;
                }

                // Après 'que' : trouver sujet, auxiliaire avoir, participe passé
                let mut idx = i + 1;
                // Sauter le sujet (ex: tu, il, elle, on, nous, vous, ils, elles, j', un nom...)
                if idx < tokens.len() {
                    idx += 1;
                }
                // Trouver l'auxiliaire avoir
                if idx < tokens.len() {
                    let aux_mot = tokens[idx].mot.to_lowercase();
                    if self.auxiliaires_avoir.contains(aux_mot.as_str()) {
                        idx += 1;
                        // Sauter un éventuel adverbe (déjà, bien, souvent, etc.)
                        if idx < tokens.len() && [
                            "déjà", "bien", "souvent", "toujours", "vite", "beaucoup", "encore", "clairement"
                        ].contains(&tokens[idx].mot.to_lowercase().as_str()) {
                            idx += 1;
                        }

                        // Le mot suivant est le participe passé
                        if idx < tokens.len() {
                            let part_tok = &tokens[idx];
                            if let Some(suggestion) = accorder_participe_passe(part_tok.mot, feminin, pluriel) {
                                let genre_str = if feminin && pluriel {
                                    "féminin pluriel"
                                } else if feminin {
                                    "féminin singulier"
                                } else {
                                    "masculin pluriel"
                                };

                                let antec_texte = if i >= 2 {
                                    format!("{} {}", tokens[i - 2].mot, nom_ant.mot)
                                } else {
                                    nom_ant.mot.to_string()
                                };

                                alertes.push(AlerteGrammaticale {
                                    debut: decalage + part_tok.deb,
                                    fin: decalage + part_tok.fin,
                                    texte_fautif: part_tok.mot.to_string(),
                                    regle: "Accord du participe passé avec COD antéposé".into(),
                                    explication: format!(
                                        "Le participe passé employé avec l'auxiliaire avoir s'accorde avec le COD antéposé « {} » ({}).",
                                        antec_texte, genre_str
                                    ),
                                    suggestions: vec![suggestion],
                                    genre: GenreErreur::ParticipePasse,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Forme 2 : Pronom COD direct « les » devant l'auxiliaire avoir
        // Exemple : « Je les ai vu » -> « vus »
        for i in 0..tokens.len().saturating_sub(2) {
            let t_les = &tokens[i];
            if t_les.mot.to_lowercase() == "les" {
                let t_aux = &tokens[i + 1];
                if self.auxiliaires_avoir.contains(t_aux.mot.to_lowercase().as_str()) {
                    let t_part = &tokens[i + 2];
                    if let Some(sug) = accorder_participe_passe(t_part.mot, false, true) {
                        let mut sugs = vec![sug];
                        if let Some(sug_fem) = accorder_participe_passe(t_part.mot, true, true) {
                            if !sugs.contains(&sug_fem) {
                                sugs.push(sug_fem);
                            }
                        }

                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_part.deb,
                            fin: decalage + t_part.fin,
                            texte_fautif: t_part.mot.to_string(),
                            regle: "Accord du participe passé avec COD antéposé".into(),
                            explication: "Le participe passé employé avec l'auxiliaire avoir s'accorde avec le pronom COD « les » placé avant le verbe.".into(),
                            suggestions: sugs,
                            genre: GenreErreur::ParticipePasse,
                        });
                    }
                }
            }
        }
    }

    /// Règle : Adjectifs de couleur invariables issus de noms
    /// « Les lunettes marrons » -> « marron »
    /// « Des yeux noisettes » -> « noisette »
    fn verifier_adjectifs_couleur_invariables(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for (i, t) in tokens.iter().enumerate() {
            let lower = t.mot.to_lowercase();
            // Liste d'adjectifs de couleur dérivés de noms qui sont strictement invariables
            let est_couleur_invariable = match lower.as_str() {
                "marrons" => Some("marron"),
                "noisettes" => Some("noisette"),
                "oranges" => Some("orange"),
                "cerises" => Some("cerise"),
                "turquoises" => Some("turquoise"),
                "kakies" | "kakis" => Some("kaki"),
                "abricots" => Some("abricot"),
                "crèmes" => Some("crème"),
                "olives" => Some("olive"),
                "saumons" => Some("saumon"),
                _ => None,
            };

            if let Some(forme_invariable) = est_couleur_invariable {
                // Pour 'marrons' : c'est presque toujours une faute d'accord quand utilisé comme adjectif
                // Si précédé d'un nom ou déterminant pluriel
                let mut est_adjectif = false;
                if lower == "marrons" {
                    // En français courant, 'marrons' comme adjectif est TOUJOURS une faute.
                    // Seul le nom pluriel existe ("manger des marrons").
                    // S'il suit un nom pluriel (lunettes, yeux, chaussures, etc.), c'est un adjectif.
                    if i > 0 {
                        let prev = tokens[i - 1].mot.to_lowercase();
                        if prev.ends_with('s') || prev.ends_with('x') || ["yeux", "lunettes", "chaussures", "vestes", "cheveux", "murs", "sacs", "portes"].contains(&prev.as_str()) {
                            est_adjectif = true;
                        }
                    }
                } else if i > 0 {
                    let prev = tokens[i - 1].mot.to_lowercase();
                    if prev.ends_with('s') || prev.ends_with('x') {
                        est_adjectif = true;
                    }
                }

                if est_adjectif {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Adjectif de couleur invariable".into(),
                        explication: format!(
                            "L'adjectif de couleur « {} » provient d'un nom et reste invariable en genre et en nombre (pas de « s »).",
                            forme_invariable
                        ),
                        suggestions: vec![forme_invariable.to_string()],
                        genre: GenreErreur::Accord,
                    });
                }
            }
        }
    }

    /// Règle : Homophones et confusions grammaticales fréquentes
    fn verifier_homophones(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for (i, t) in tokens.iter().enumerate() {
            let lower = t.mot.to_lowercase();

            // 1. « a » vs « à »
            if lower == "a" {
                // Suivi d'un verbe à l'infinitif (-er, -ir, -re, -oir)
                if i + 1 < tokens.len() {
                    let suiv = tokens[i + 1].mot.to_lowercase();
                    if est_infinitif(&suiv) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion a / à".into(),
                            explication: format!(
                                "Devant l'infinitif « {} », on emploie la préposition « à » (avec accent), non le verbe avoir.",
                                tokens[i + 1].mot
                            ),
                            suggestions: vec![adapte_casse(t.mot, "à")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
                // Précédé de locutions courantes demandant « à »
                if i > 0 {
                    let prec = tokens[i - 1].mot.to_lowercase();
                    if ["grâce", "face", "quant", "jusqu", "prêt", "apte", "tendance"].contains(&prec.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion a / à".into(),
                            explication: format!(
                                "Après « {} », il faut écrire la préposition « à » avec un accent.",
                                tokens[i - 1].mot
                            ),
                            suggestions: vec![adapte_casse(t.mot, "à")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
                // Expressions courantes : a nouveau, a bientôt, a côté, a fond, a peine...
                if i + 1 < tokens.len() {
                    let suiv = tokens[i + 1].mot.to_lowercase();
                    if ["nouveau", "bientôt", "demain", "côté", "peine", "jour", "fond", "court", "propos", "travers"].contains(&suiv.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion a / à".into(),
                            explication: format!("Dans la locution « à {} », « à » prend un accent.", tokens[i + 1].mot),
                            suggestions: vec![adapte_casse(t.mot, "à")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            } else if lower == "à" {
                // Précédé de pronoms sujets 3e personne singulier : « il à », « elle à », « on à »
                if i > 0 {
                    let prec = tokens[i - 1].mot.to_lowercase();
                    if ["il", "elle", "on", "qui", "ça"].contains(&prec.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion a / à".into(),
                            explication: format!(
                                "Après « {} », il s'agit du verbe avoir conjugué (« a » sans accent).",
                                tokens[i - 1].mot
                            ),
                            suggestions: vec![adapte_casse(t.mot, "a")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            }

            // 2. « ce » vs « se »
            if lower == "ce" {
                // Suivi d'un verbe réfléchi courant ou précédé d'un sujet (ex: « il ce demande », « ce passe »)
                if i + 1 < tokens.len() {
                    let suiv = tokens[i + 1].mot.to_lowercase();
                    if [
                        "passe", "passent", "passera", "passait", "fait", "font", "fera", "faire",
                        "trompe", "trompent", "trouve", "trouvent", "retrouve", "compose", "déroule",
                        "rend", "met", "situe", "demande", "souvient", "rappelle", "produit"
                    ].contains(&suiv.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion ce / se".into(),
                            explication: format!("Devant le verbe « {} », on écrit « se » (pronom réfléchi) et non « ce ».", tokens[i + 1].mot),
                            suggestions: vec![adapte_casse(t.mot, "se")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            } else if lower == "se" {
                // Suivi d'un nom masculin singulier (ex: « se matin », « se projet »)
                if i + 1 < tokens.len() {
                    let suiv = tokens[i + 1].mot.to_lowercase();
                    if [
                        "matin", "soir", "jour", "mois", "projet", "problème", "livre", "document",
                        "cas", "type", "travail", "point", "sujet", "moment", "temps", "dossier", "week-end"
                    ].contains(&suiv.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion ce / se".into(),
                            explication: format!("Devant le nom « {} », on écrit l'adjectif démonstratif « ce » et non « se ».", tokens[i + 1].mot),
                            suggestions: vec![adapte_casse(t.mot, "ce")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            }

            // 3. « c'est » vs « s'est »
            if lower == "s'" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "est" {
                // « s'est vrai », « s'est possible », « s'est bon »...
                if i + 2 < tokens.len() {
                    let suiv = tokens[i + 2].mot.to_lowercase();
                    if [
                        "vrai", "faux", "possible", "impossible", "bon", "bien", "super", "parfait",
                        "clair", "évident", "facile", "difficile", "important", "nécessaire", "utile",
                        "grave", "normal", "bizarre", "étrange", "sûr", "certain", "exact", "tout"
                    ].contains(&suiv.as_str()) {
                        let fin_span = tokens[i + 1].fin;
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + fin_span,
                            texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                            regle: "Confusion c'est / s'est".into(),
                            explication: format!("Devant « {} », on écrit « c'est » (démonstratif) et non « s'est ».", tokens[i + 2].mot),
                            suggestions: vec![adapte_casse(t.mot, "c'est")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            } else if lower == "c'" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "est" {
                // Précédé d'un sujet : « il c'est trompé »
                if i > 0 {
                    let prec = tokens[i - 1].mot.to_lowercase();
                    if ["il", "elle", "on", "qui"].contains(&prec.as_str()) && i + 2 < tokens.len() {
                        let fin_span = tokens[i + 1].fin;
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + fin_span,
                            texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                            regle: "Confusion c'est / s'est".into(),
                            explication: format!("Après le sujet « {} », on écrit « s'est » (forme pronominale).", tokens[i - 1].mot),
                            suggestions: vec![adapte_casse(t.mot, "s'est")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            }

            // 4. « ou » vs « où »
            if lower == "ou" {
                if i > 0 {
                    let prec = tokens[i - 1].mot.to_lowercase();
                    if ["jour", "moment", "fois", "endroit", "lieu", "ville", "pays", "époque", "semaine", "année"].contains(&prec.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion ou / où".into(),
                            explication: format!("Pour marquer le temps ou le lieu après « {} », on écrit « où » avec un accent.", tokens[i - 1].mot),
                            suggestions: vec![adapte_casse(t.mot, "où")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            } else if lower == "où" {
                if i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "bien" {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion ou / où".into(),
                        explication: "Dans la conjonction de choix « ou bien », « ou » ne prend pas d'accent.".into(),
                        suggestions: vec![adapte_casse(t.mot, "ou")],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // 5. « leur » vs « leurs »
            if lower == "leurs" {
                // Suivi d'un verbe (infinitif ou conjugué)
                if i + 1 < tokens.len() {
                    let suiv = tokens[i + 1].mot.to_lowercase();
                    if [
                        "donne", "donnent", "donner", "dit", "disent", "dire", "envoie", "envoient",
                        "envoyer", "fait", "font", "faire", "parle", "parlent", "parler", "demande",
                        "demandent", "demander", "montre", "montrent", "montrer", "explique",
                        "expliquent", "expliquer", "propose", "proposent", "proposer", "accorde"
                    ].contains(&suiv.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Pronom leur invariable".into(),
                            explication: format!("Le pronom personnel « leur » placé devant le verbe « {} » est invariable (jamais de « s »).", tokens[i + 1].mot),
                            suggestions: vec![adapte_casse(t.mot, "leur")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            } else if lower == "leur" {
                // Suivi d'un nom pluriel
                if i + 1 < tokens.len() {
                    let suiv = tokens[i + 1].mot.to_lowercase();
                    if [
                        "enfants", "parents", "amis", "collègues", "clients", "équipes", "projets",
                        "droits", "responsabilités", "documents", "décisions", "affaires", "idées"
                    ].contains(&suiv.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Déterminant leurs au pluriel".into(),
                            explication: format!("Devant le nom pluriel « {} », le déterminant possessif prend un « s » (« leurs »).", tokens[i + 1].mot),
                            suggestions: vec![adapte_casse(t.mot, "leurs")],
                            genre: GenreErreur::Accord,
                        });
                        continue;
                    }
                }
            }

            // 6. « si il » / « si ils » -> élision obligatoire « s'il » / « s'ils »
            if lower == "si" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if suiv == "il" || suiv == "ils" {
                    let suggestion = if suiv == "il" { "s'il" } else { "s'ils" };
                    let suggestion_adaptee = adapte_casse(t.mot, suggestion);
                    let fin_span = tokens[i + 1].fin;
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + fin_span,
                        texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                        regle: "Élision obligatoire de si".into(),
                        explication: format!("En français, l'élision de « si » est obligatoire devant « {} » (« {} »).", suiv, suggestion),
                        suggestions: vec![suggestion_adaptee],
                        genre: GenreErreur::Grammaire,
                    });
                    continue;
                }
            }

            // 7. « sans doutes » -> « sans doute »
            if lower == "sans" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "doutes" {
                let fin_span = tokens[i + 1].fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Locution sans doute".into(),
                    explication: "La locution adverbiale « sans doute » s'écrit toujours au singulier (sans « s » à doute).".into(),
                    suggestions: vec!["sans doute".into()],
                    genre: GenreErreur::Grammaire,
                });
                continue;
            }

            // 8. « tous le » / « tous le monde » -> « tout le monde »
            if lower == "tous" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "le" {
                let fin_span = tokens[i + 1].fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Accord de tout devant le singulier".into(),
                    explication: "Devant le déterminant masculin singulier « le », on écrit « tout » (avec un « t »).".into(),
                    suggestions: vec!["tout le".into()],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // 9. « je vous envoit » -> « envoie »
            if lower == "envoit" {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Conjugaison du verbe envoyer".into(),
                    explication: "Le verbe envoyer à la 1re personne du présent s'écrit « envoie » (et non « envoit »).".into(),
                    suggestions: vec!["envoie".into()],
                    genre: GenreErreur::Grammaire,
                });
            }
        }
    }

    /// Règle : Participe passé avec être accordé avec le sujet
    /// « elle est parti » -> « partie »
    /// « ils sont arrivé » -> « arrivés »
    fn verifier_participe_passe_avec_etre(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len().saturating_sub(2) {
            let t_suj = &tokens[i];
            let s_norm = t_suj.mot.to_lowercase();
            let t_etre = &tokens[i + 1];
            let etre_norm = t_etre.mot.to_lowercase();

            if self.auxiliaires_etre.contains(etre_norm.as_str()) {
                let t_part = &tokens[i + 2];
                // Sujet féminin singulier explicite : « elle est »
                if s_norm == "elle" && etre_norm == "est" {
                    if let Some(sug) = accorder_participe_passe(t_part.mot, true, false) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_part.deb,
                            fin: decalage + t_part.fin,
                            texte_fautif: t_part.mot.to_string(),
                            regle: "Accord du participe passé avec être".into(),
                            explication: "Avec l'auxiliaire être, le participe passé s'accorde en genre et en nombre avec le sujet (« elle » féminin singulier).".into(),
                            suggestions: vec![sug],
                            genre: GenreErreur::Accord,
                        });
                    }
                }
                // Sujet pluriel explicite : « ils sont » ou « elles sont »
                else if (s_norm == "ils" || s_norm == "elles") && etre_norm == "sont" {
                    let feminin = s_norm == "elles";
                    if let Some(sug) = accorder_participe_passe(t_part.mot, feminin, true) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_part.deb,
                            fin: decalage + t_part.fin,
                            texte_fautif: t_part.mot.to_string(),
                            regle: "Accord du participe passé avec être".into(),
                            explication: format!(
                                "Avec l'auxiliaire être, le participe passé s'accorde avec le sujet pluriel « {} ».",
                                t_suj.mot
                            ),
                            suggestions: vec![sug],
                            genre: GenreErreur::Accord,
                        });
                    }
                }
            }
        }
    }

    fn est_nom_feminin(&self, mot: &str) -> bool {
        let m = mot.to_lowercase();
        if self.noms_feminins_courants.contains(m.as_str()) {
            return true;
        }
        // Terminaisons très fortement féminines en français
        m.ends_with("tion")
            || m.ends_with("sion")
            || m.ends_with("ette")
            || m.ends_with("té")
            || m.ends_with("ure")
            || m.ends_with("ence")
            || m.ends_with("ance")
            || m.ends_with("ade")
    }
}

/// Adapte la casse d'une suggestion selon le modèle d'origine (ex: Se -> Ce).
pub fn adapte_casse(modele: &str, suggestion: &str) -> String {
    if modele.chars().next().map_or(false, |c| c.is_uppercase()) {
        let mut chs = suggestion.chars();
        match chs.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chs.as_str(),
        }
    } else {
        suggestion.to_string()
    }
}

/// Tokeniseur qui extrait les mots avec positions exactes en caractères
/// et gère les élisions françaises (qu', j', c', s', l', d'...).
pub fn tokenize<'a>(para: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();
    let char_indices: Vec<(usize, char)> = para.char_indices().collect();
    let total_chars = char_indices.len();

    let mut i = 0;
    while i < total_chars {
        let (_byte_pos, c) = char_indices[i];

        if c.is_alphanumeric() {
            let mut deb_char = i;
            let mut fin_char = i + 1;

            while fin_char < total_chars {
                let (_, ch) = char_indices[fin_char];
                if ch.is_alphanumeric() || ch == '-' {
                    fin_char += 1;
                } else if ch == '\'' || ch == '’' {
                    // Vérifier si la partie actuelle est un préfixe d'élision français courant
                    let sub_str = slice_chars(para, &char_indices, deb_char, fin_char);
                    let sub_lower = sub_str.to_lowercase();
                    if ["c", "d", "j", "l", "m", "n", "s", "t", "qu", "jusqu", "puisqu", "lorsqu"].contains(&sub_lower.as_str()) {
                        // Inclure l'apostrophe dans le préfixe élidé
                        fin_char += 1;
                        let token_elide = slice_chars(para, &char_indices, deb_char, fin_char);
                        tokens.push(Token {
                            mot: token_elide,
                            deb: deb_char,
                            fin: fin_char,
                        });
                        deb_char = fin_char;
                        // Le reste du mot continuera au tour suivant
                    } else {
                        // Mot composé avec apostrophe (ex: aujourd'hui), continuer
                        fin_char += 1;
                    }
                } else {
                    break;
                }
            }

            if deb_char < fin_char {
                let mot = slice_chars(para, &char_indices, deb_char, fin_char);
                tokens.push(Token {
                    mot,
                    deb: deb_char,
                    fin: fin_char,
                });
            }

            i = fin_char;
        } else {
            i += 1;
        }
    }

    tokens
}

fn slice_chars<'a>(s: &'a str, char_indices: &[(usize, char)], deb_char: usize, fin_char: usize) -> &'a str {
    let start_byte = char_indices[deb_char].0;
    let end_byte = if fin_char < char_indices.len() {
        char_indices[fin_char].0
    } else {
        s.len()
    };
    &s[start_byte..end_byte]
}

/// Accorde un participe passé en genre (féminin) et en nombre (pluriel).
/// Retourne Some(forme_accordée) si différente de l'originale, sinon None.
pub fn accorder_participe_passe(part: &str, feminin: bool, pluriel: bool) -> Option<String> {
    let lower = part.to_lowercase();

    // Table de verbes irréguliers très courants et participes fréquents
    let (base_masc_sing, base_fem_sing) = match lower.as_str() {
        "vu" | "vue" | "vus" | "vues" => ("vu", "vue"),
        "fait" | "faite" | "faits" | "faites" => ("fait", "faite"),
        "écrit" | "écrite" | "écrits" | "écrites" => ("écrit", "écrite"),
        "dit" | "dite" | "dits" | "dites" => ("dit", "dite"),
        "pris" | "prise" | "prises" => ("pris", "prise"),
        "mis" | "mise" | "mises" => ("mis", "mise"),
        "appris" | "apprise" | "apprises" => ("appris", "apprise"),
        "compris" | "comprise" | "comprises" => ("compris", "comprise"),
        "reçu" | "reçue" | "reçus" | "reçues" => ("reçu", "reçue"),
        "voulu" | "voulue" | "voulus" | "voulues" => ("voulu", "voulue"),
        "su" | "sue" | "sus" | "sues" => ("su", "sue"),
        "tenu" | "tenue" | "tenus" | "tenues" => ("tenu", "tenue"),
        "obtenu" | "obtenue" | "obtenus" | "obtenues" => ("obtenu", "obtenue"),
        "retenu" | "retenue" | "retenus" | "retenues" => ("retenu", "retenue"),
        "perdu" | "perdue" | "perdus" | "perdues" => ("perdu", "perdue"),
        "vendu" | "vendue" | "vendus" | "vendues" => ("vendu", "vendue"),
        "rendu" | "rendue" | "rendus" | "rendues" => ("rendu", "rendue"),
        "attendu" | "attendue" | "attendus" | "attendues" => ("attendu", "attendue"),
        "entendu" | "entendue" | "entendus" | "entendues" => ("entendu", "entendue"),
        "ouvert" | "ouverte" | "ouverts" | "ouvertes" => ("ouvert", "ouverte"),
        "couvert" | "couverte" | "couverts" | "couvertes" => ("couvert", "couverte"),
        "offert" | "offerte" | "offerts" | "offertes" => ("offert", "offerte"),
        "conduit" | "conduite" | "conduits" | "conduites" => ("conduit", "conduite"),
        "produit" | "produite" | "produits" | "produites" => ("produit", "produite"),
        "construit" | "construite" | "construits" | "construites" => ("construit", "construite"),
        "traduit" | "traduite" | "traduits" | "traduites" => ("traduit", "traduite"),
        "choisi" | "choisie" | "choisis" | "choisies" => ("choisi", "choisie"),
        "fini" | "finie" | "finis" | "finies" => ("fini", "finie"),
        "réussi" | "réussie" | "réussis" | "réussies" => ("réussi", "réussie"),
        "cueilli" | "cueillie" | "cueillis" | "cueillies" => ("cueilli", "cueillie"),
        "suivi" | "suivie" | "suivis" | "suivies" => ("suivi", "suivie"),
        "parti" | "partie" | "partis" | "parties" => ("parti", "partie"),
        "allé" | "allée" | "allés" | "allées" => ("allé", "allée"),
        "venu" | "venue" | "venus" | "venues" => ("venu", "venue"),
        "arrivé" | "arrivée" | "arrivés" | "arrivées" => ("arrivé", "arrivée"),
        "resté" | "restée" | "restés" | "restées" => ("resté", "restée"),
        "tombé" | "tombée" | "tombés" | "tombées" => ("tombé", "tombée"),
        "mort" | "morte" | "morts" | "mortes" => ("mort", "morte"),
        _ => {
            if lower.ends_with("ées") || lower.ends_with("ée") || lower.ends_with("és") || lower.ends_with('é') {
                let stem = lower.trim_end_matches(|c| c == 's' || c == 'e');
                (stem, stem)
            } else if lower.ends_with("ies") || lower.ends_with("ie") || lower.ends_with("is") || lower.ends_with('i') {
                let stem = lower.trim_end_matches(|c| c == 's' || c == 'e');
                (stem, stem)
            } else if lower.ends_with("ues") || lower.ends_with("ue") || lower.ends_with("us") || lower.ends_with('u') {
                let stem = lower.trim_end_matches(|c| c == 's' || c == 'e');
                (stem, stem)
            } else {
                return None;
            }
        }
    };

    let target = match (feminin, pluriel) {
        (false, false) => base_masc_sing.to_string(),
        (false, true) => {
            if base_masc_sing.ends_with('s') {
                base_masc_sing.to_string()
            } else {
                format!("{}s", base_masc_sing)
            }
        }
        (true, false) => {
            if base_fem_sing.ends_with('e') {
                base_fem_sing.to_string()
            } else {
                format!("{}e", base_fem_sing)
            }
        }
        (true, true) => {
            let fem = if base_fem_sing.ends_with('e') {
                base_fem_sing.to_string()
            } else {
                format!("{}e", base_fem_sing)
            };
            format!("{}s", fem)
        }
    };

    if target.to_lowercase() != lower {
        if part.chars().next().map_or(false, |c| c.is_uppercase()) {
            let mut chs = target.chars();
            let cap = match chs.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chs.as_str(),
            };
            Some(cap)
        } else {
            Some(target)
        }
    } else {
        None
    }
}

fn est_infinitif(mot: &str) -> bool {
    let m = mot.to_lowercase();
    if m.len() <= 2 {
        return false;
    }
    if ["air", "mer", "fer", "soir", "hiver", "danger", "enfer", "hier", "cher"].contains(&m.as_str()) {
        return false;
    }
    m.ends_with("er") || m.ends_with("ir") || m.ends_with("re") || m.ends_with("oir")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_la_fille_que_tu_as_vu() {
        let g = CorrecteurGrammaire::new();
        let texte = "La fille que tu as vu";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1, "Devrait détecter l'accord du participe passé avec COD antéposé");
        assert_eq!(alertes[0].texte_fautif, "vu");
        assert_eq!(alertes[0].suggestions, vec!["vue"]);
        assert_eq!(alertes[0].genre, GenreErreur::ParticipePasse);
    }

    #[test]
    fn test_les_lunettes_marrons() {
        let g = CorrecteurGrammaire::new();
        let texte = "Les lunettes marrons.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1, "Devrait détecter que l'adjectif marron est invariable");
        assert_eq!(alertes[0].texte_fautif, "marrons");
        assert_eq!(alertes[0].suggestions, vec!["marron"]);
        assert_eq!(alertes[0].genre, GenreErreur::Accord);
    }

    #[test]
    fn test_autres_participes_cod_antepose() {
        let g = CorrecteurGrammaire::new();
        
        let a1 = g.analyse("Les lettres que j'ai écrit");
        assert!(!a1.is_empty());
        assert_eq!(a1[0].texte_fautif, "écrit");
        assert_eq!(a1[0].suggestions, vec!["écrites"]);

        let a2 = g.analyse("La pomme qu'il a mangé");
        assert!(!a2.is_empty());
        assert_eq!(a2[0].texte_fautif, "mangé");
        assert_eq!(a2[0].suggestions, vec!["mangée"]);

        let a3 = g.analyse("Je les ai vu");
        assert!(!a3.is_empty());
        assert_eq!(a3[0].texte_fautif, "vu");
        assert!(a3[0].suggestions.contains(&"vus".to_string()));
    }

    #[test]
    fn test_homophones() {
        let g = CorrecteurGrammaire::new();

        let a1 = g.analyse("Il a a faire");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].texte_fautif, "a");
        assert_eq!(a1[0].suggestions, vec!["à"]);

        let a2 = g.analyse("Il ce passe un problème");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].texte_fautif, "ce");
        assert_eq!(a2[0].suggestions, vec!["se"]);

        let a3 = g.analyse("Se matin il fait beau");
        assert_eq!(a3.len(), 1);
        assert_eq!(a3[0].texte_fautif, "Se");
        assert_eq!(a3[0].suggestions, vec!["Ce"]);

        let a4 = g.analyse("S'est vrai");
        assert_eq!(a4.len(), 1);
        assert_eq!(a4[0].texte_fautif, "S' est");
        assert_eq!(a4[0].suggestions, vec!["C'est"]);

        let a5 = g.analyse("Le jour ou il arrive");
        assert_eq!(a5.len(), 1);
        assert_eq!(a5[0].texte_fautif, "ou");
        assert_eq!(a5[0].suggestions, vec!["où"]);

        let a6 = g.analyse("Leurs donner le document");
        assert_eq!(a6.len(), 1);
        assert_eq!(a6[0].texte_fautif, "Leurs");
        assert_eq!(a6[0].suggestions, vec!["Leur"]);

        let a7 = g.analyse("Si il vient demain");
        assert_eq!(a7.len(), 1);
        assert_eq!(a7[0].suggestions, vec!["S'il"]);
    }

    #[test]
    fn test_accords_et_participes_existants() {
        let g = CorrecteurGrammaire::new();
        let texte = "Les fournisseur ont confirmer la livraison.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 2);
        assert_eq!(alertes[0].texte_fautif, "fournisseur");
        assert_eq!(alertes[0].suggestions, vec!["fournisseurs"]);
        assert_eq!(alertes[1].texte_fautif, "confirmer");
        assert_eq!(alertes[1].suggestions, vec!["confirmé"]);
    }

    #[test]
    fn test_doublon() {
        let g = CorrecteurGrammaire::new();
        let texte = "Voici la la réponse.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].genre, GenreErreur::Doublon);
        assert_eq!(alertes[0].suggestions, vec!["la"]);
    }

    #[test]
    fn test_infinitif_apres_preposition() {
        let g = CorrecteurGrammaire::new();
        let texte = "Il faut venir pour terminé le projet.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "terminé");
        assert_eq!(alertes[0].suggestions, vec!["terminer"]);
    }

    #[test]
    fn test_je_vous_envoie_pas_d_erreur() {
        let g = CorrecteurGrammaire::new();
        let texte = "Bonjour, je vous envoie le document concernant le projet 长城 (Chángchéng).";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 0);
    }
}
'''

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'w', encoding='utf-8') as f:
    f.write(grammaire_code)

print("grammaire.rs updated successfully!")
