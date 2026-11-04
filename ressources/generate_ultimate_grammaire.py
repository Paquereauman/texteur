import sys

grammaire_code = r'''//! Moteur de règles grammaticales et syntaxiques françaises 100 % local et autonome en Rust.
//! Couvre les accords (déterminant-nom en genre et nombre, participe passé avec être/avoir/COD antéposé,
//! verbes pronominaux réciproques COI, sujet-verbe), adjectifs de couleur invariables et exceptions,
//! subjonctif obligatoire vs indicatif, fautes de transitivité (pallier à), pluriels exceptionnels (-al, -ou, -ail),
//! homophones majeurs (a/à, ce/se, c'est/s'est, ou/où, leur/leurs, si il, quel que soit, tout le/tous les).

use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum GenreErreur {
    Grammaire,
    Accord,
    ParticipePasse,
    Infinitif,
    Homophone,
    Syntaxe,
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
    noms_masculins_courants: HashSet<&'static str>,
    verbes_coi_pronominaux: HashSet<&'static str>,
    exceptions_pluriel_al_en_s: HashSet<&'static str>,
    exceptions_pluriel_ou_en_x: HashSet<&'static str>,
    exceptions_pluriel_ail_en_aux: HashSet<&'static str>,
    couleurs_invariables: HashSet<&'static str>,
    couleurs_variables_exceptions: HashSet<&'static str>,
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
            "difficulté", "difficultés", "décision", "décisions", "réunion", "réunions", "solution", "solutions",
            "situation", "situations", "erreur", "erreurs", "lettre", "lettres", "fille", "filles", "femme", "femmes",
            "pomme", "pommes", "photo", "photos", "lunette", "lunettes", "robe", "robes", "chaussure", "chaussures",
            "voiture", "voitures", "maison", "maisons", "idée", "idées", "fleur", "fleurs", "question", "questions",
            "affaire", "affaires", "commande", "commandes", "demande", "demandes", "remarque", "remarques",
            "personne", "personnes", "page", "pages", "version", "versions", "copie", "copies", "règle", "règles",
            "pièce", "pièces", "porte", "portes", "fenêtre", "fenêtres", "table", "tables", "semaine", "semaines",
            "année", "années", "heure", "heures", "chanson", "chansons", "histoire", "histoires", "phrase", "phrases",
            "étape", "étapes", "démarche", "démarches", "entreprise", "entreprises", "société", "sociétés",
            "activité", "activités", "qualité", "qualités", "séance", "séances", "rencontre", "rencontres",
            "nouvelle", "nouvelles", "mission", "missions", "voie", "voies", "fois", "ville", "villes",
        ]);

        let mut noms_masc = HashSet::new();
        noms_masc.extend([
            "problème", "problèmes", "projet", "projets", "document", "documents", "message", "messages",
            "rapport", "rapports", "client", "clients", "dossier", "dossiers", "cas", "type", "types",
            "travail", "travaux", "sujet", "sujets", "point", "points", "temps", "moment", "moments",
            "secteur", "secteurs", "pays", "service", "services", "compte", "comptes", "livre", "livres",
            "fichier", "fichiers", "courrier", "courriers", "matin", "soir", "jour", "jours", "mois",
            "accord", "accords", "ami", "amis", "homme", "hommes", "enfant", "enfants", "devoir", "devoirs",
        ]);

        let mut coi_pron = HashSet::new();
        coi_pron.extend([
            "parler", "téléphoner", "succéder", "nuire", "plaire", "mentir", "rire", "survivre", "ressembler", "écrire", "convenir"
        ]);

        let mut al_en_s = HashSet::new();
        al_en_s.extend(["bal", "carnaval", "chacal", "festival", "régal", "récital", "gala", "narval"]);

        let mut ou_en_x = HashSet::new();
        ou_en_x.extend(["bijou", "caillou", "chou", "genou", "hibou", "joujou", "pou"]);

        let mut ail_en_aux = HashSet::new();
        ail_en_aux.extend(["travail", "vitrail", "corail", "émail", "bail", "soupirail", "vantail"]);

        let mut coul_inv = HashSet::new();
        coul_inv.extend([
            "marron", "orange", "turquoise", "safran", "vermillon", "cerise", "tabac", "pistache",
            "noisette", "marine", "perle", "ardoise", "abricot", "kaki", "crème", "olive", "saumon",
            "lilas", "bordeaux", "citron", "chocolat", "corail", "ambre", "indigo", "prune", "saphir",
        ]);

        let mut coul_var = HashSet::new();
        coul_var.extend(["rose", "mauve", "pourpre", "fauve", "écarlate", "châtain"]);

        Self {
            auxiliaires_avoir: avoir,
            auxiliaires_etre: etre,
            prepositions_infinitif: prep_inf,
            determinants_pluriel: det_plur,
            determinants_singulier_masc: det_sing_masc,
            determinants_singulier_fem: det_sing_fem,
            noms_feminins_courants: noms_fem,
            noms_masculins_courants: noms_masc,
            verbes_coi_pronominaux: coi_pron,
            exceptions_pluriel_al_en_s: al_en_s,
            exceptions_pluriel_ou_en_x: ou_en_x,
            exceptions_pluriel_ail_en_aux: ail_en_aux,
            couleurs_invariables: coul_inv,
            couleurs_variables_exceptions: coul_var,
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
        // RÈGLE 1 : DOUBLONS CONSÉCUTIFS (« la la », « de de »)
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
        // RÈGLE 2 : ACCORD DU DÉTERMINANT EN GENRE AVEC LE NOM
        // Exemple : « ce difficulté » -> « cette difficulté » !
        // Exemple : « cette problème » -> « ce problème » !
        // Exemple : « un réunion » -> « une réunion » !
        // ====================================================================
        self.verifier_accord_determinant_genre(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 3 : ACCORD PARTICIPE PASSÉ AVEC COD ANTÉPOSÉ (AVOIR)
        // Exemple : « La fille que tu as vu » -> « vue »
        // Exemple : « Les lettres que j'ai écrit » -> « écrites »
        // ====================================================================
        self.verifier_participe_passe_cod_antepose(para, &tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 4 : PARTICIPES PASSÉS PRONOMINAUX COI INVARIABLES
        // Exemple : « ils se sont parlés » -> « parlé »
        // Exemple : « elles se sont téléphonées » -> « téléphoné »
        // ====================================================================
        self.verifier_verbes_pronominaux_coi(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 5 : ADJECTIFS DE COULEUR INVARIABLES (marron, orange...)
        // Exemple : « Les lunettes marrons » -> « marron »
        // ====================================================================
        self.verifier_adjectifs_couleur_invariables(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 6 : HOMOPHONES & SYNTAXE
        // a / à, ce / se, c'est / s'est, ou / où, leur / leurs, si il, quel que soit...
        // ====================================================================
        self.verifier_homophones(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 7 : SUBJONCTIF OBLIGATOIRE VS INDICATIF
        // Exemple : « bien qu'il fait » -> « fasse »
        // Exemple : « après qu'il soit » -> « est »
        // ====================================================================
        self.verifier_subjonctif_et_concordance(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 8 : FAUTES DE TRANSITIVITÉ & CONSTRUCTIONS VERBALES
        // Exemple : « pallier à » -> « pallier »
        // Exemple : « malgré que » -> « bien que » / « quoique »
        // ====================================================================
        self.verifier_transitivite_et_constructions(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 9 : PLURIELS EXCEPTIONNELS (-al, -ou, -ail)
        // Exemple : « des chevals » -> « chevaux », « des bijous » -> « bijoux »
        // ====================================================================
        self.verifier_pluriels_exceptionnels(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 10 : ACCORD SUJET-VERBE (TU + verbe sans s, ILS/ELLES + verbe singulier)
        // Exemple : « tu mange » -> « manges »
        // Exemple : « ils mange » -> « mangent »
        // ====================================================================
        self.verifier_accord_sujet_verbe(&tokens, decalage, alertes);

        // ====================================================================
        // RÈGLE 11 : AUXILIAIRE AVOIR + INFINITIF EN -ER AU LIEU DE PARTICIPE EN -É
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
        // RÈGLE 12 : AUXILIAIRE ÊTRE + INFINITIF EN -ER AU LIEU DE PARTICIPE EN -É
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
        // RÈGLE 13 : PRÉPOSITION + PARTICIPE EN -É AU LIEU DE L'INFINITIF EN -ER
        // ====================================================================
        for i in 0..tokens.len().saturating_sub(1) {
            let t_prep = &tokens[i];
            let prep_norm = t_prep.mot.to_lowercase();
            if self.prepositions_infinitif.contains(prep_norm.as_str()) {
                let t_suiv = &tokens[i + 1];
                let s_norm = t_suiv.mot.to_lowercase();
                if (s_norm.ends_with('é') || s_norm.ends_with("és") || s_norm.ends_with("ée") || s_norm.ends_with("ées")) && s_norm.len() > 3 {
                    // Exception si temps composé (ex: "pour avoir mangé", "sans être venu")
                    if i > 0 && (tokens[i - 1].mot.to_lowercase() == "avoir" || tokens[i - 1].mot.to_lowercase() == "être") {
                        continue;
                    }
                    let racine = t_suiv.mot.trim_end_matches(|c| c == 's' || c == 'e' || c == 'é');
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
        // RÈGLE 14 : DÉTERMINANT PLURIEL SUIVI D'UN NOM SINGULIER
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
                if est_infinitif(&m_norm) {
                    continue;
                }
                if ![
                    "est", "sont", "ont", "a", "fait", "peut", "doit", "vont", "sera", "seront",
                    "marron", "orange", "kaki", "turquoise"
                ].contains(&m_norm.as_str()) {
                    let suggestion = if m_norm.ends_with("al") && !self.exceptions_pluriel_al_en_s.contains(m_norm.as_str()) {
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
        // RÈGLE 15 : DÉTERMINANT SINGULIER SUIVI D'UN MOT AU PLURIEL
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
                            "Après le déterminant singulier « {} », le mot doit être au singulier (« {} »).",
                            t_det.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                }
            }
        }
    }

    /// Règle 2 : Accord en genre du déterminant avec le nom
    /// « sur ce difficulté » -> « cette difficulté »
    /// « cette problème » -> « ce problème »
    /// « un réunion » -> « une réunion »
    fn verifier_accord_determinant_genre(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len().saturating_sub(1) {
            let t_det = &tokens[i];
            let det_lower = t_det.mot.to_lowercase();
            let t_nom = &tokens[i + 1];
            let nom_lower = t_nom.mot.to_lowercase();

            // CAS A : Déterminant masculin « ce » ou « cet » devant un nom féminin
            if det_lower == "ce" || det_lower == "cet" {
                if self.est_nom_feminin(&nom_lower) {
                    let suggestion = adapte_casse(t_det.mot, "cette");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Accord en genre du déterminant".into(),
                        explication: format!(
                            "Le nom « {} » est féminin : on emploie le déterminant démonstratif « {} » (et non « {} »).",
                            t_nom.mot, suggestion, t_det.mot
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // CAS B : Déterminant féminin « cette » devant un nom masculin
            if det_lower == "cette" {
                if self.est_nom_masculin(&nom_lower) {
                    let voyelle = nom_lower.chars().next().map_or(false, |c| ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'h'].contains(&c));
                    let base_sug = if voyelle { "cet" } else { "ce" };
                    let suggestion = adapte_casse(t_det.mot, base_sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Accord en genre du déterminant".into(),
                        explication: format!(
                            "Le nom « {} » est masculin : on emploie le déterminant démonstratif « {} » (et non « cette »).",
                            t_nom.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // CAS C : « un » devant un nom féminin
            if det_lower == "un" {
                if self.est_nom_feminin(&nom_lower) {
                    let suggestion = adapte_casse(t_det.mot, "une");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Accord en genre de l'article".into(),
                        explication: format!(
                            "Le nom « {} » est féminin : on emploie « {} » (et non « un »).",
                            t_nom.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // CAS D : « une » devant un nom masculin
            if det_lower == "une" {
                if self.est_nom_masculin(&nom_lower) {
                    let suggestion = adapte_casse(t_det.mot, "un");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Accord en genre de l'article".into(),
                        explication: format!(
                            "Le nom « {} » est masculin : on emploie « {} » (et non « une »).",
                            t_nom.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }
        }
    }

    /// Règle 3 : Accord du participe passé avec COD antéposé
    fn verifier_participe_passe_cod_antepose(
        &self,
        _para: &str,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len() {
            let t = &tokens[i];
            let t_norm = t.mot.to_lowercase();

            if t_norm == "que" || t_norm == "qu'" {
                if i == 0 {
                    continue;
                }

                let nom_ant = &tokens[i - 1];
                let nom_lower = nom_ant.mot.to_lowercase();

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

                if !feminin && !pluriel {
                    continue;
                }

                let mut idx = i + 1;
                if idx < tokens.len() {
                    idx += 1; // sauter sujet
                }
                if idx < tokens.len() {
                    let aux_mot = tokens[idx].mot.to_lowercase();
                    if self.auxiliaires_avoir.contains(aux_mot.as_str()) {
                        idx += 1;
                        if idx < tokens.len() && [
                            "déjà", "bien", "souvent", "toujours", "vite", "beaucoup", "encore", "clairement"
                        ].contains(&tokens[idx].mot.to_lowercase().as_str()) {
                            idx += 1;
                        }

                        if idx < tokens.len() {
                            let part_tok = &tokens[idx];
                            // Exception pour le verbe 'laisser' suivi d'un infinitif : invariable (« que j'ai laissé partir »)
                            if part_tok.mot.to_lowercase() == "laissé" && idx + 1 < tokens.len() && est_infinitif(&tokens[idx + 1].mot) {
                                continue;
                            }

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

        // Forme 2 : Pronom COD direct « les » devant l'auxiliaire avoir (« Je les ai vu »)
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

    /// Règle 4 : Verbes pronominaux réciproques COI (invariables)
    /// « ils se sont parlés » -> « parlé »
    /// « elles se sont téléphonées » -> « téléphoné »
    fn verifier_verbes_pronominaux_coi(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len().saturating_sub(2) {
            let t_pron = &tokens[i];
            let pron_lower = t_pron.mot.to_lowercase();
            if pron_lower == "se" || pron_lower == "s'" {
                let t_aux = &tokens[i + 1];
                let aux_lower = t_aux.mot.to_lowercase();
                if aux_lower == "sont" || aux_lower == "étaient" || aux_lower == "seront" {
                    let t_part = &tokens[i + 2];
                    let part_lower = t_part.mot.to_lowercase();

                    // Vérifier si le participe correspond à un verbe intransitif / indirect COI
                    let infinitif = estimer_infinitif_participe(&part_lower);
                    if self.verbes_coi_pronominaux.contains(infinitif.as_str()) {
                        // Si le participe est accordé au pluriel ou au féminin, c'est une faute !
                        if part_lower.ends_with('s') || part_lower.ends_with('e') {
                            let racine_invariable = if part_lower.ends_with("ées") || part_lower.ends_with("és") {
                                t_part.mot.trim_end_matches(|c| c == 's' || c == 'e').to_string()
                            } else if part_lower.ends_with('s') {
                                t_part.mot.trim_end_matches('s').to_string()
                            } else {
                                t_part.mot.to_string()
                            };

                            alertes.push(AlerteGrammaticale {
                                debut: decalage + t_part.deb,
                                fin: decalage + t_part.fin,
                                texte_fautif: t_part.mot.to_string(),
                                regle: "Participe passé pronominal COI invariable".into(),
                                explication: format!(
                                    "Le verbe « {} » se construit avec un complément d'objet indirect (on parle à quelqu'un) : son participe passé reste invariable.",
                                    infinitif
                                ),
                                suggestions: vec![racine_invariable],
                                genre: GenreErreur::Accord,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Règle 5 : Adjectifs de couleur invariables issus de noms
    fn verifier_adjectifs_couleur_invariables(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for (i, t) in tokens.iter().enumerate() {
            let lower = t.mot.to_lowercase();
            let mut est_invariable = None;

            for inv in &self.couleurs_invariables {
                let plur = format!("{}s", inv);
                let fem = format!("{}e", inv);
                let femplur = format!("{}es", inv);
                if lower == plur || lower == fem || lower == femplur {
                    est_invariable = Some(*inv);
                    break;
                }
            }

            if let Some(forme_invariable) = est_invariable {
                let mut est_adjectif = false;
                if lower == "marrons" {
                    if i > 0 {
                        let prev = tokens[i - 1].mot.to_lowercase();
                        if prev.ends_with('s') || prev.ends_with('x') || [
                            "yeux", "lunettes", "chaussures", "vestes", "cheveux", "murs", "sacs", "portes"
                        ].contains(&prev.as_str()) {
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

    /// Règle 6 : Homophones et syntaxe courante
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
                if i > 0 {
                    let prec = tokens[i - 1].mot.to_lowercase();
                    if ["grâce", "face", "quant", "jusqu", "prêt", "apte", "tendance"].contains(&prec.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion a / à".into(),
                            explication: format!("Après « {} », il faut écrire la préposition « à » avec un accent.", tokens[i - 1].mot),
                            suggestions: vec![adapte_casse(t.mot, "à")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
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
                if i > 0 {
                    let prec = tokens[i - 1].mot.to_lowercase();
                    if ["il", "elle", "on", "qui", "ça"].contains(&prec.as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion a / à".into(),
                            explication: format!("Après « {} », il s'agit du verbe avoir conjugué (« a » sans accent).", tokens[i - 1].mot),
                            suggestions: vec![adapte_casse(t.mot, "a")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            }

            // 2. « ce » vs « se » devant verbe
            if lower == "ce" && i + 1 < tokens.len() {
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
            } else if lower == "se" && i + 1 < tokens.len() {
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

            // 3. « c'est » vs « s'est »
            if lower == "s'" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "est" {
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

            // 6. « si il » / « si ils » -> « s'il » / « s'ils »
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

            // 7. « quelque soit » -> « quel que soit »
            if (lower == "quelque" || lower == "quelques") && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if suiv == "soit" || suiv == "soient" {
                    let sug = if suiv == "soit" {
                        "quel que soit"
                    } else {
                        "quels que soient"
                    };
                    let fin_span = tokens[i + 1].fin;
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + fin_span,
                        texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                        regle: "Locution quel que soit".into(),
                        explication: "« Quel que soit » s'écrit en mots séparés et s'accorde avec le nom sujet.".into(),
                        suggestions: vec![sug.into()],
                        genre: GenreErreur::Grammaire,
                    });
                    continue;
                }
            }

            // 8. « sans doutes » -> « sans doute »
            if lower == "sans" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "doutes" {
                let fin_span = tokens[i + 1].fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Locution sans doute".into(),
                    explication: "La locution adverbiale « sans doute » s'écrit toujours au singulier (sans « s » à doute).".into(),
                    suggestions: vec![adapte_casse(t.mot, "sans doute")],
                    genre: GenreErreur::Grammaire,
                });
                continue;
            }

            // 9. « tous le » -> « tout le » / « tout les » -> « tous les »
            if lower == "tous" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "le" {
                let fin_span = tokens[i + 1].fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Accord de tout devant le singulier".into(),
                    explication: "Devant le déterminant masculin singulier « le », on écrit « tout » (avec un « t »).".into(),
                    suggestions: vec![adapte_casse(t.mot, "tout le")],
                    genre: GenreErreur::Accord,
                });
                continue;
            } else if lower == "tout" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "les" {
                let fin_span = tokens[i + 1].fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Accord de tous devant le pluriel".into(),
                    explication: "Devant l'article pluriel « les », on écrit « tous » (avec un « s »).".into(),
                    suggestions: vec![adapte_casse(t.mot, "tous les")],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // 10. « je vous envoit » -> « envoie »
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

    /// Règle 7 : Subjonctif obligatoire vs Indicatif après conjonctions
    fn verifier_subjonctif_et_concordance(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len().saturating_sub(2) {
            let t1 = &tokens[i];
            let t2 = &tokens[i + 1];
            let n1 = t1.mot.to_lowercase();
            let n2 = t2.mot.to_lowercase();

            // CAS 1 : « bien que » / « quoique » / « afin que » / « pour que » suivi de l'indicatif
            let est_loc_subjonctif = (n1 == "bien" && n2 == "que")
                || (n1 == "bien" && n2 == "qu'")
                || (n1 == "quoique" || n1 == "quoiqu'")
                || (n1 == "afin" && (n2 == "que" || n2 == "qu'"))
                || (n1 == "pour" && (n2 == "que" || n2 == "qu'"))
                || (n1 == "avant" && (n2 == "que" || n2 == "qu'"));

            if est_loc_subjonctif {
                // Trouver le verbe suivant (sauter le pronom sujet ex: il, elle, on...)
                let mut v_idx = if n1 == "quoique" || n1 == "quoiqu'" { i + 1 } else { i + 2 };
                if v_idx < tokens.len() && ["il", "elle", "on", "ils", "elles", "tu", "je", "nous", "vous"].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                    v_idx += 1;
                }

                if v_idx < tokens.len() {
                    let t_v = &tokens[v_idx];
                    let v_lower = t_v.mot.to_lowercase();
                    // Verbes fréquents mis à l'indicatif par erreur
                    let correction_subjonctif = match v_lower.as_str() {
                        "fait" => Some("fasse"),
                        "est" => Some("soit"),
                        "a" => Some("ait"),
                        "va" => Some("aille"),
                        "veut" => Some("veuille"),
                        "peut" => Some("puisse"),
                        "sait" => Some("sache"),
                        _ => None,
                    };

                    if let Some(sug) = correction_subjonctif {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_v.deb,
                            fin: decalage + t_v.fin,
                            texte_fautif: t_v.mot.to_string(),
                            regle: "Subjonctif requis".into(),
                            explication: format!(
                                "Après la conjonction « {} {} », le verbe doit être au subjonctif (« {} ») et non à l'indicatif.",
                                t1.mot, t2.mot, sug
                            ),
                            suggestions: vec![sug.into()],
                            genre: GenreErreur::Syntaxe,
                        });
                    }
                }
            }

            // CAS 2 : « après que » suivi du subjonctif (erreur fréquente : « après qu'il soit venu » -> « est venu »)
            if (n1 == "après" || n1 == "apres") && (n2 == "que" || n2 == "qu'") {
                let mut v_idx = i + 2;
                if v_idx < tokens.len() && ["il", "elle", "on", "ils", "elles", "tu", "je", "nous", "vous"].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                    v_idx += 1;
                }
                if v_idx < tokens.len() {
                    let t_v = &tokens[v_idx];
                    let v_lower = t_v.mot.to_lowercase();
                    let correction_indicatif = match v_lower.as_str() {
                        "soit" => Some("est"),
                        "ait" => Some("a"),
                        "fasse" => Some("fait"),
                        "puisse" => Some("peut"),
                        "vienne" => Some("vient"),
                        _ => None,
                    };

                    if let Some(sug) = correction_indicatif {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_v.deb,
                            fin: decalage + t_v.fin,
                            texte_fautif: t_v.mot.to_string(),
                            regle: "Indicatif après après que".into(),
                            explication: "La locution « après que » exprime une action accomplie dans le passé et exige l'indicatif (et non le subjonctif).".into(),
                            suggestions: vec![sug.into()],
                            genre: GenreErreur::Syntaxe,
                        });
                    }
                }
            }
        }
    }

    /// Règle 8 : Fautes de transitivité & constructions
    /// - « pallier à » -> « pallier »
    /// - « malgré que » -> « bien que » / « quoique »
    /// - « où que tu vas » -> « où vas-tu ? »
    fn verifier_transitivite_et_constructions(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len() {
            let t = &tokens[i];
            let lower = t.mot.to_lowercase();

            // A : « pallier à »
            if (lower == "pallier" || lower == "pallie" || lower == "pallient" || lower == "pallié") && i + 1 < tokens.len() {
                if tokens[i + 1].mot.to_lowercase() == "à" || tokens[i + 1].mot.to_lowercase() == "a" {
                    let fin_span = tokens[i + 1].fin;
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + fin_span,
                        texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                        regle: "Transitivité directe du verbe pallier".into(),
                        explication: "Le verbe « pallier » est transitif direct : on pallie un manque (et non « pallier à »).".into(),
                        suggestions: vec![t.mot.to_string()],
                        genre: GenreErreur::Syntaxe,
                    });
                }
            }

            // B : « malgré que »
            if lower == "malgré" && i + 1 < tokens.len() && (tokens[i + 1].mot.to_lowercase() == "que" || tokens[i + 1].mot.to_lowercase() == "qu'") {
                let fin_span = tokens[i + 1].fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Tour fautif malgré que".into(),
                    explication: "La tournure « malgré que » est considérée comme fautive en français soigné. Utilisez « bien que », « quoique » ou « malgré le fait que ».".into(),
                    suggestions: vec!["bien que".into(), "quoique".into(), "malgré le fait que".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }

            // C : « où que tu vas »
            if lower == "où" && i + 2 < tokens.len() {
                let s1 = tokens[i + 1].mot.to_lowercase();
                let s2 = tokens[i + 2].mot.to_lowercase();
                if (s1 == "que" || s1 == "qu'") && (s2 == "tu" || s2 == "vous" || s2 == "il") {
                    let fin_span = tokens[i + 2].fin;
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + fin_span,
                        texte_fautif: format!("{} {} {}", t.mot, tokens[i + 1].mot, tokens[i + 2].mot),
                        regle: "Tour interrogatif fautif".into(),
                        explication: "Tournure orale incorrecte pour une question directe. Privilégiez l'inversion ou « Où est-ce que... ».".into(),
                        suggestions: vec!["Où vas-tu ?".into(), "Où est-ce que tu vas ?".into()],
                        genre: GenreErreur::Syntaxe,
                    });
                }
            }
        }
    }

    /// Règle 9 : Pluriels exceptionnels (-al, -ou, -ail)
    fn verifier_pluriels_exceptionnels(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for (i, t) in tokens.iter().enumerate() {
            let lower = t.mot.to_lowercase();

            // A : Exceptions en -al qui prennent un -s (bals, festivals, carnavals...)
            // Si écrit « baux » au lieu de « bals »
            if lower == "baux" && i > 0 && (tokens[i - 1].mot.to_lowercase() == "des" || tokens[i - 1].mot.to_lowercase() == "les") {
                // 'baux' est le pluriel de bail, valide.
            }
            // Si un nom en -al régulier est écrit en -als (ex: « chevals » -> « chevaux »)
            if lower.ends_with("als") && lower.len() > 4 {
                if !self.exceptions_pluriel_al_en_s.contains(&lower[..lower.len() - 1]) {
                    let correct = format!("{}aux", &t.mot[..t.mot.len() - 3]);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Pluriel régulier en -aux".into(),
                        explication: format!("Les noms en -al font généralement leur pluriel en -aux (« {} »).", correct),
                        suggestions: vec![correct],
                        genre: GenreErreur::Accord,
                    });
                }
            }

            // B : Les 7 exceptions en -ou qui prennent un -x (bijoux, cailloux, choux, genoux, hiboux, joujoux, poux)
            if lower.ends_with("ous") && lower.len() > 3 {
                let sing = &lower[..lower.len() - 1];
                if self.exceptions_pluriel_ou_en_x.contains(sing) {
                    let correct = format!("{}x", &t.mot[..t.mot.len() - 1]);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Pluriel exceptionnel en -oux".into(),
                        explication: format!("Le nom « {} » fait partie des 7 exceptions en -ou prenant un « x » au pluriel (« {} »).", sing, correct),
                        suggestions: vec![correct],
                        genre: GenreErreur::Accord,
                    });
                }
            }

            // C : Exceptions en -ail qui prennent -aux (travaux, vitraux...)
            if lower.ends_with("ails") && lower.len() > 4 {
                let sing = &lower[..lower.len() - 1];
                if self.exceptions_pluriel_ail_en_aux.contains(sing) {
                    let correct = format!("{}ux", &t.mot[..t.mot.len() - 2]);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Pluriel exceptionnel en -aux".into(),
                        explication: format!("Le nom « {} » prend la terminaison -aux au pluriel (« {} »).", sing, correct),
                        suggestions: vec![correct],
                        genre: GenreErreur::Accord,
                    });
                }
            }
        }
    }

    /// Règle 10 : Accord Sujet-Verbe étendu (Tu + verbe sans 's', ils/elles sans -nt)
    fn verifier_accord_sujet_verbe(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len().saturating_sub(1) {
            let t_suj = &tokens[i];
            let s_lower = t_suj.mot.to_lowercase();

            // CAS 1 : « tu » + verbe régulier du 1er groupe sans 's' final
            if s_lower == "tu" {
                let mut v_idx = i + 1;
                // Sauter d'éventuels pronoms compléments (ex: « tu me dis », « tu lui parles »)
                if v_idx + 1 < tokens.len() && ["me", "te", "le", "la", "les", "lui", "leur", "en", "y", "ne"].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                    v_idx += 1;
                }

                if v_idx < tokens.len() {
                    let t_v = &tokens[v_idx];
                    let v_lower = t_v.mot.to_lowercase();
                    let verbes_excep_x_s = ["veux", "peux", "vaux", "dois", "sais", "dis", "ris", "fis", "mets", "prends", "viens", "tiens", "pars", "fais", "vois"];
                    if v_lower.ends_with('e') && !v_lower.ends_with("es") && v_lower.len() > 3 && !verbes_excep_x_s.contains(&v_lower.as_str()) {
                        let suggestion = format!("{}s", t_v.mot);
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_v.deb,
                            fin: decalage + t_v.fin,
                            texte_fautif: t_v.mot.to_string(),
                            regle: "Accord sujet-verbe avec tu".into(),
                            explication: format!("Avec le pronom sujet « tu », le verbe prend un « s » (« {} »).", suggestion),
                            suggestions: vec![suggestion],
                            genre: GenreErreur::Accord,
                        });
                    }
                }
            }

            // CAS 2 : « ils » / « elles » + verbe singulier
            if s_lower == "ils" || s_lower == "elles" {
                let mut v_idx = i + 1;
                if v_idx + 1 < tokens.len() && ["lui", "leur", "nous", "vous", "le", "la", "les", "y", "en", "se", "ne"].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                    v_idx += 1;
                }

                if v_idx < tokens.len() {
                    let t_v = &tokens[v_idx];
                    let v_lower = t_v.mot.to_lowercase();
                    if (v_lower.ends_with('e') || v_lower.ends_with('a')) && v_lower.len() > 3 && ![
                        "ne", "se", "y", "en", "le", "la", "les", "leur", "lui", "même", "aussi", "bien"
                    ].contains(&v_lower.as_str()) {
                        let suggestion = format!("{}nt", t_v.mot);
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_v.deb,
                            fin: decalage + t_v.fin,
                            texte_fautif: t_v.mot.to_string(),
                            regle: "Accord sujet-verbe au pluriel".into(),
                            explication: format!("Après le pronom pluriel « {} », le verbe doit se terminer par -nt (« {} »).", t_suj.mot, suggestion),
                            suggestions: vec![suggestion],
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
        m.ends_with("tion")
            || m.ends_with("sion")
            || m.ends_with("ette")
            || m.ends_with("té")
            || m.ends_with("ure")
            || m.ends_with("ence")
            || m.ends_with("ance")
            || m.ends_with("ade")
    }

    fn est_nom_masculin(&self, mot: &str) -> bool {
        let m = mot.to_lowercase();
        if self.noms_masculins_courants.contains(m.as_str()) {
            return true;
        }
        m.ends_with("ment")
            || m.ends_with("age")
            || m.ends_with("isme")
            || m.ends_with("eau")
            || m.ends_with("oir")
            || m.ends_with("teur")
    }
}

/// Tokeniseur avec gestion des élisions et positions exactes en caractères
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
                    let sub_str = slice_chars(para, &char_indices, deb_char, fin_char);
                    let sub_lower = sub_str.to_lowercase();
                    if ["c", "d", "j", "l", "m", "n", "s", "t", "qu", "jusqu", "puisqu", "lorsqu"].contains(&sub_lower.as_str()) {
                        fin_char += 1;
                        let token_elide = slice_chars(para, &char_indices, deb_char, fin_char);
                        tokens.push(Token {
                            mot: token_elide,
                            deb: deb_char,
                            fin: fin_char,
                        });
                        deb_char = fin_char;
                    } else {
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

pub fn accorder_participe_passe(part: &str, feminin: bool, pluriel: bool) -> Option<String> {
    let lower = part.to_lowercase();

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
        "perdu" | "perdue" | "perdus" | "perdues" => ("perdu", "perdue"),
        "vendu" | "vendue" | "vendus" | "vendues" => ("vendu", "vendue"),
        "attendu" | "attendue" | "attendus" | "attendues" => ("attendu", "attendue"),
        "entendu" | "entendue" | "entendus" | "entendues" => ("entendu", "entendue"),
        "ouvert" | "ouverte" | "ouverts" | "ouvertes" => ("ouvert", "ouverte"),
        "choisi" | "choisie" | "choisis" | "choisies" => ("choisi", "choisie"),
        "fini" | "finie" | "finis" | "finies" => ("fini", "finie"),
        "réussi" | "réussie" | "réussis" | "réussies" => ("réussi", "réussie"),
        "cueilli" | "cueillie" | "cueillis" | "cueillies" => ("cueilli", "cueillie"),
        "suivi" | "suivie" | "suivis" | "suivies" => ("suivi", "suivie"),
        "parti" | "partie" | "partis" | "parties" => ("parti", "partie"),
        "allé" | "allée" | "allés" | "allées" => ("allé", "allée"),
        "venu" | "venue" | "venus" | "venues" => ("venu", "venue"),
        "arrivé" | "arrivée" | "arrivés" | "arrivées" => ("arrivé", "arrivée"),
        "acheté" | "achetée" | "achetés" | "achetées" => ("acheté", "achetée"),
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

fn estimer_infinitif_participe(mot: &str) -> String {
    let pur = mot.to_lowercase();
    if pur.ends_with("ées") || pur.ends_with("és") || pur.ends_with("ée") || pur.ends_with('é') {
        let racine = pur.trim_end_matches(|c| c == 's' || c == 'e' || c == 'é');
        format!("{}er", racine)
    } else if pur.ends_with("is") || pur.ends_with("ie") || pur.ends_with('i') {
        let racine = pur.trim_end_matches(|c| c == 's' || c == 'e');
        format!("{}r", racine)
    } else {
        pur
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
    fn test_ce_difficulte_resolu() {
        let g = CorrecteurGrammaire::new();
        let texte = "faire le point sur ce difficulté et régler ce problème";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1, "Devrait détecter l'accord en genre sur 'ce difficulté'");
        assert_eq!(alertes[0].texte_fautif, "ce");
        assert_eq!(alertes[0].suggestions, vec!["cette"]);
        assert_eq!(alertes[0].genre, GenreErreur::Accord);
    }

    #[test]
    fn test_la_fille_que_tu_as_vu() {
        let g = CorrecteurGrammaire::new();
        let texte = "La fille que tu as vu";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "vu");
        assert_eq!(alertes[0].suggestions, vec!["vue"]);
    }

    #[test]
    fn test_les_lunettes_marrons() {
        let g = CorrecteurGrammaire::new();
        let texte = "Les lunettes marrons.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "marrons");
        assert_eq!(alertes[0].suggestions, vec!["marron"]);
    }

    #[test]
    fn test_pallier_a() {
        let g = CorrecteurGrammaire::new();
        let texte = "Il faut pallier à ce manque.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "pallier à");
        assert_eq!(alertes[0].suggestions, vec!["pallier"]);
    }

    #[test]
    fn test_tu_mange() {
        let g = CorrecteurGrammaire::new();
        let texte = "Tu mange une pomme.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "mange");
        assert_eq!(alertes[0].suggestions, vec!["manges"]);
    }

    #[test]
    fn test_bien_qu_il_fait() {
        let g = CorrecteurGrammaire::new();
        let texte = "Bien qu'il fait beau.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "fait");
        assert_eq!(alertes[0].suggestions, vec!["fasse"]);
    }

    #[test]
    fn test_pronominal_coi_invariable() {
        let g = CorrecteurGrammaire::new();
        let texte = "Ils se sont parlés longuement.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "parlés");
        assert_eq!(alertes[0].suggestions, vec!["parlé"]);
    }

    #[test]
    fn test_pluriels_exceptionnels() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("des chevals");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].suggestions, vec!["chevaux"]);

        let a2 = g.analyse("des bijous");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].suggestions, vec!["bijoux"]);
    }

    #[test]
    fn test_ecran_utilisateur_complet() {
        let g = CorrecteurGrammaire::new();
        let texte = "Bonjour, je vous envoie le document concernant le projet 长城 (Chángchéng).\nLes fournisseurs de Shenzhen 深圳 ont confirmé la livraison du projet.\nNous devons faire une réunion pour faire le point sur ce difficulté et régler ce problème rapidement.";
        let alertes = g.analyse(texte);
        assert_eq!(alertes.len(), 1, "Devrait avoir uniquement 'ce difficulté' dans cet écran !");
        assert_eq!(alertes[0].texte_fautif, "ce");
        assert_eq!(alertes[0].suggestions, vec!["cette"]);
    }
}
'''

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'w', encoding='utf-8') as f:
    f.write(grammaire_code)

print("Updated grammaire.rs with all comprehensive rules & exceptions!")
