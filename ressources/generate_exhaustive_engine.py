import sys

exhaustive_grammaire = r'''//! Moteur de règles grammaticales, syntaxiques et stylistiques 100 % local et autonome en Rust.
//! Niveau de détection exhaustif (style Antidote) :
//! 1. Accords déterminants-noms (genre et nombre, ce/cette, un/une, elision l'/d').
//! 2. Contractions obligatoires d'articles (à le -> au, de le -> du, à les -> aux, de les -> des).
//! 3. Élision obligatoire des articles et prépositions (la entreprise -> l'entreprise, le accord -> l'accord).
//! 4. Accord du participe passé (COD antéposé avec avoir, accord avec être, verbes pronominaux COI invariables, exception laissé + infinitif).
//! 5. Accords adjectif-nom (genre et nombre : des documents important -> importants, démarche administratif -> administrative).
//! 6. Accord sujet-verbe complet (tu + s, ils/elles + nt, tout le monde est, chacun a, aucun sans s).
//! 7. Pronoms relatifs et syntaxe de subordination (ce que j'ai besoin -> ce dont, la raison pourquoi, la façon dont).
//! 8. Subjonctif obligatoire vs indicatif (bien qu'il fait -> fasse, après qu'il soit -> est).
//! 9. Adjectifs de couleur invariables et exceptions (marron, orange, turquoise... vs rose, mauve, châtain).
//! 10. Pluriels exceptionnels (-al en -aux, -ou en -oux, -ail en -aux).
//! 11. Adjectifs numéraux (quatre-vingts, deux mille invariable, une heure et demie).
//! 12. Transitivité et constructions verbales (pallier à -> pallier, malgré que -> bien que).
//! 13. Homophones lexicaux et grammaticaux majeurs (sensé/censé, faire partie, a/à, ce/se, c'est/s'est, ou/où, leur/leurs).
//! 14. Pléonasmes syntaxiques lourds (au jour d'aujourd'hui, voire même, monter en haut).

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
            "chance", "chances", "stratégie", "stratégies", "opportunité", "opportunités", "partie", "parties",
        ]);

        let mut noms_masc = HashSet::new();
        noms_masc.extend([
            "problème", "problèmes", "projet", "projets", "document", "documents", "message", "messages",
            "rapport", "rapports", "client", "clients", "dossier", "dossiers", "cas", "type", "types",
            "travail", "travaux", "sujet", "sujets", "point", "points", "temps", "moment", "moments",
            "secteur", "secteurs", "pays", "service", "services", "compte", "comptes", "livre", "livres",
            "fichier", "fichiers", "courrier", "courriers", "matin", "soir", "jour", "jours", "mois",
            "accord", "accords", "ami", "amis", "homme", "hommes", "enfant", "enfants", "devoir", "devoirs",
            "bureau", "bureaux", "prix", "impact", "impacts", "rôle", "rôles", "membre", "membres",
            "chiffre", "chiffres", "groupe", "groupes", "poste", "postes", "résultat", "résultats",
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
        // 1. DOUBLONS CONSÉCUTIFS (« la la », « de de »)
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
        // 2. CONTRACTIONS OBLIGATOIRES DES ARTICLES (à le -> au, de le -> du...)
        // Exemple : « aller à le bureau » -> « au bureau »
        // Exemple : « le prix de le projet » -> « du projet »
        // ====================================================================
        self.verifier_contractions_articles(&tokens, decalage, alertes);

        // ====================================================================
        // 3. ÉLISION OBLIGATOIRE DES ARTICLES & PRÉPOSITIONS DEVANT VOYELLE
        // Exemple : « la entreprise » -> « l'entreprise »
        // Exemple : « le accord » -> « l'accord »
        // ====================================================================
        self.verifier_elisions_obligatoires(&tokens, decalage, alertes);

        // ====================================================================
        // 4. ACCORD DU DÉTERMINANT EN GENRE AVEC LE NOM
        // Exemple : « sur ce difficulté » -> « cette difficulté »
        // Exemple : « cette problème » -> « ce problème »
        // ====================================================================
        self.verifier_accord_determinant_genre(&tokens, decalage, alertes);

        // ====================================================================
        // 5. ACCORD PARTICIPE PASSÉ AVEC COD ANTÉPOSÉ (AVOIR)
        // Exemple : « La fille que tu as vu » -> « vue »
        // ====================================================================
        self.verifier_participe_passe_cod_antepose(para, &tokens, decalage, alertes);

        // ====================================================================
        // 6. PARTICIPES PASSÉS PRONOMINAUX COI INVARIABLES
        // Exemple : « ils se sont parlés » -> « parlé »
        // ====================================================================
        self.verifier_verbes_pronominaux_coi(&tokens, decalage, alertes);

        // ====================================================================
        // 7. ADJECTIFS DE COULEUR INVARIABLES (marron, orange...)
        // Exemple : « Les lunettes marrons » -> « marron »
        // ====================================================================
        self.verifier_adjectifs_couleur_invariables(&tokens, decalage, alertes);

        // ====================================================================
        // 8. PRONOMS RELATIFS & SYNTAXE DE SUBORDINATION
        // Exemple : « ce que j'ai besoin » -> « ce dont j'ai besoin »
        // Exemple : « la raison pourquoi » -> « la raison pour laquelle »
        // ====================================================================
        self.verifier_pronoms_relatifs_et_syntaxe(&tokens, decalage, alertes);

        // ====================================================================
        // 9. HOMOPHONES LEXICAUX ET GRAMMATICAUX AVANCÉS
        // sensé/censé, faire partie, a/à, ce/se, c'est/s'est, ou/où, leur/leurs, si il...
        // ====================================================================
        self.verifier_homophones(&tokens, decalage, alertes);

        // ====================================================================
        // 10. SUBJONCTIF OBLIGATOIRE VS INDICATIF
        // Exemple : « bien qu'il fait » -> « fasse », « après qu'il soit » -> « est »
        // ====================================================================
        self.verifier_subjonctif_et_concordance(&tokens, decalage, alertes);

        // ====================================================================
        // 11. FAUTES DE TRANSITIVITÉ, PLÉONASMES & CONSTRUCTIONS FAUTIVES
        // Exemple : « pallier à » -> « pallier », « au jour d'aujourd'hui » -> « aujourd'hui »
        // ====================================================================
        self.verifier_transitivite_et_pleonasmes(&tokens, decalage, alertes);

        // ====================================================================
        // 12. PLURIELS EXCEPTIONNELS (-al, -ou, -ail)
        // ====================================================================
        self.verifier_pluriels_exceptionnels(&tokens, decalage, alertes);

        // ====================================================================
        // 13. ACCORD SUJET-VERBE (TU, ILS/ELLES, TOUT LE MONDE, CHACUN, AUCUN)
        // Exemple : « tout le monde sont » -> « est », « tu mange » -> « manges »
        // ====================================================================
        self.verifier_accord_sujet_verbe(&tokens, decalage, alertes);

        // ====================================================================
        // 14. ACCORD QUALIFICATIF NOM-ADJECTIF (GENRE ET NOMBRE)
        // Exemple : « des documents important » -> « importants »
        // ====================================================================
        self.verifier_accord_nom_adjectif(&tokens, decalage, alertes);

        // ====================================================================
        // 15. ADJECTIFS NUMÉRAUX (vingt, cent, mille invariable, demi)
        // Exemple : « quatre-vingt personnes » -> « quatre-vingts », « deux milles » -> « mille »
        // ====================================================================
        self.verifier_adjectifs_numeraux(&tokens, decalage, alertes);

        // ====================================================================
        // 16. AUXILIAIRE AVOIR + INFINITIF EN -ER AU LIEU DE PARTICIPE EN -É
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
        // 17. PRÉPOSITION + PARTICIPE EN -É AU LIEU DE L'INFINITIF EN -ER
        // ====================================================================
        for i in 0..tokens.len().saturating_sub(1) {
            let t_prep = &tokens[i];
            let prep_norm = t_prep.mot.to_lowercase();
            if self.prepositions_infinitif.contains(prep_norm.as_str()) {
                let t_suiv = &tokens[i + 1];
                let s_norm = t_suiv.mot.to_lowercase();
                if (s_norm.ends_with('é') || s_norm.ends_with("és") || s_norm.ends_with("ée") || s_norm.ends_with("ées")) && s_norm.len() > 3 {
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
        // 18. DÉTERMINANT PLURIEL SUIVI D'UN NOM SINGULIER
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
        // 19. DÉTERMINANT SINGULIER SUIVI D'UN MOT AU PLURIEL
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

    /// Règle : Contractions obligatoires des articles
    /// « à le » -> « au », « de le » -> « du », « à les » -> « aux », « de les » -> « des »
    fn verifier_contractions_articles(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len().saturating_sub(1) {
            let t1 = &tokens[i];
            let t2 = &tokens[i + 1];
            let p1 = t1.mot.to_lowercase();
            let a2 = t2.mot.to_lowercase();

            // Si suivi d'un infinitif (ex: « afin de le faire », « le fait de le voir »), « le » est pronom et non article !
            let est_suivi_infinitif = i + 2 < tokens.len() && est_infinitif(&tokens[i + 2].mot.to_lowercase());

            if !est_suivi_infinitif {
                let contraction = match (p1.as_str(), a2.as_str()) {
                    ("à", "le") => Some("au"),
                    ("a", "le") if i > 0 && !["il", "elle", "on", "qui"].contains(&tokens[i-1].mot.to_lowercase().as_str()) => Some("au"),
                    ("à", "les") | ("a", "les") => Some("aux"),
                    ("de", "le") => Some("du"),
                    ("de", "les") => Some("des"),
                    _ => None,
                };

                if let Some(sug) = contraction {
                    let fin_span = t2.fin;
                    let sug_casse = adapte_casse(t1.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t1.deb,
                        fin: decalage + fin_span,
                        texte_fautif: format!("{} {}", t1.mot, t2.mot),
                        regle: "Contraction obligatoire de l'article".into(),
                        explication: format!(
                            "En français, la préposition « {} » et l'article défini « {} » fusionnent obligatoirement en « {} ».",
                            t1.mot, t2.mot, sug_casse
                        ),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Syntaxe,
                    });
                }
            }
        }
    }

    /// Règle : Élision obligatoire devant voyelle ou h muet
    /// « la entreprise » -> « l'entreprise », « le accord » -> « l'accord »
    fn verifier_elisions_obligatoires(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len().saturating_sub(1) {
            let t1 = &tokens[i];
            let t2 = &tokens[i + 1];
            let w1 = t1.mot.to_lowercase();
            let w2 = t2.mot.to_lowercase();

            let commence_voyelle = w2.chars().next().map_or(false, |c| {
                ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û'].contains(&c)
            });

            if commence_voyelle && w2.len() > 1 {
                let elision = match w1.as_str() {
                    "le" | "la" => Some(format!("l'{}", t2.mot)),
                    "de" => Some(format!("d'{}", t2.mot)),
                    _ => None,
                };

                if let Some(sug) = elision {
                    let fin_span = t2.fin;
                    let sug_casse = adapte_casse(t1.mot, &sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t1.deb,
                        fin: decalage + fin_span,
                        texte_fautif: format!("{} {}", t1.mot, t2.mot),
                        regle: "Élision obligatoire devant voyelle".into(),
                        explication: format!(
                            "Devant un mot commençant par une voyelle, « {} » s'élide obligatoirement en « {} ».",
                            t1.mot, sug_casse
                        ),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Accord,
                    });
                }
            }
        }
    }

    /// Règle : Pronoms relatifs et constructions de subordination
    /// « ce que j'ai besoin » -> « ce dont j'ai besoin »
    /// « la raison pourquoi » -> « la raison pour laquelle »
    /// « la façon que » -> « la façon dont »
    fn verifier_pronoms_relatifs_et_syntaxe(
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

            // CAS 1 : « ce que » suivi de « j'ai besoin » ou « nous avons besoin »
            if n1 == "ce" && (n2 == "que" || n2 == "qu'") {
                // Chercher si "besoin", "envie", "peur", "parle", "souviens" suit dans la proposition
                let mut a_besoin = false;
                for j in (i + 2)..tokens.len().min(i + 6) {
                    let w = tokens[j].mot.to_lowercase();
                    if ["besoin", "envie", "peur"].contains(&w.as_str()) {
                        a_besoin = true;
                        break;
                    }
                }

                if a_besoin {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t2.deb,
                        fin: decalage + t2.fin,
                        texte_fautif: t2.mot.to_string(),
                        regle: "Pronom relatif dont (complément en de)".into(),
                        explication: "Le verbe 'avoir besoin' se construit avec la préposition 'de' : on emploie le pronom relatif « dont » (« ce dont j'ai besoin »).".into(),
                        suggestions: vec!["dont".into()],
                        genre: GenreErreur::Syntaxe,
                    });
                }
            }

            // CAS 2 : « la raison pourquoi » -> « la raison pour laquelle »
            if n1 == "raison" && n2 == "pourquoi" {
                let fin_span = t2.fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t2.deb,
                    fin: decalage + fin_span,
                    texte_fautif: t2.mot.to_string(),
                    regle: "Subordination fautive la raison pourquoi".into(),
                    explication: "L'expression « la raison pourquoi » est un calque incorrect. Utilisez « la raison pour laquelle ».".into(),
                    suggestions: vec!["pour laquelle".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }

            // CAS 3 : « la façon que » / « la manière que » -> « la façon dont »
            if (n1 == "façon" || n1 == "facon" || n1 == "manière" || n1 == "maniere") && (n2 == "que" || n2 == "qu'") {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t2.deb,
                    fin: decalage + t2.fin,
                    texte_fautif: t2.mot.to_string(),
                    regle: "Subordination la façon dont".into(),
                    explication: "On dit « la façon dont » (ou « la manière dont ») et non « la façon que ».".into(),
                    suggestions: vec!["dont".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }
        }
    }

    /// Règle : Accord déterminant en genre avec le nom (ce/cette, un/une...)
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

            // CAS A : « ce » ou « cet » devant nom féminin
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

            // CAS B : « cette » devant nom masculin
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

            // CAS C : « un » devant nom féminin
            if det_lower == "un" && self.est_nom_feminin(&nom_lower) {
                let suggestion = adapte_casse(t_det.mot, "une");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_det.fin,
                    texte_fautif: t_det.mot.to_string(),
                    regle: "Accord en genre de l'article".into(),
                    explication: format!("Le nom « {} » est féminin : on emploie « {} » (et non « un »).", t_nom.mot, suggestion),
                    suggestions: vec![suggestion],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // CAS D : « une » devant nom masculin
            if det_lower == "une" && self.est_nom_masculin(&nom_lower) {
                let suggestion = adapte_casse(t_det.mot, "un");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_det.fin,
                    texte_fautif: t_det.mot.to_string(),
                    regle: "Accord en genre de l'article".into(),
                    explication: format!("Le nom « {} » est masculin : on emploie « {} » (et non « une »).", t_nom.mot, suggestion),
                    suggestions: vec![suggestion],
                    genre: GenreErreur::Accord,
                });
                continue;
            }
        }
    }

    /// Règle : Accord du participe passé avec COD antéposé
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
                    idx += 1;
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

        // Pronom COD direct « les » devant l'auxiliaire avoir (« Je les ai vu »)
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

    /// Règle : Verbes pronominaux réciproques COI (invariables)
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

                    let infinitif = estimer_infinitif_participe(&part_lower);
                    if self.verbes_coi_pronominaux.contains(infinitif.as_str()) {
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

    /// Règle : Adjectifs de couleur invariables issus de noms
    fn verifier_adjectifs_couleur_invariables(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for (i, t) in tokens.iter().enumerate() {
            let lower = t.mot.to_lowercase();
            if self.couleurs_variables_exceptions.contains(lower.as_str()) || self.couleurs_variables_exceptions.contains(lower.trim_end_matches('s')) {
                continue;
            }
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

    /// Règle : Homophones lexicaux & grammaticaux avancés
    fn verifier_homophones(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for (i, t) in tokens.iter().enumerate() {
            let lower = t.mot.to_lowercase();

            // « censé » vs « sensé »
            if (lower == "sensé" || lower == "sensée" || lower == "sensés" || lower == "sensées") && i + 1 < tokens.len() {
                // S'il est suivi d'un infinitif (ex: « il est sensé venir »), c'est « censé » !
                if est_infinitif(&tokens[i + 1].mot.to_lowercase()) {
                    let c_sug = if lower == "sensée" {
                        "censée"
                    } else if lower == "sensés" {
                        "censés"
                    } else if lower == "sensées" {
                        "censées"
                    } else {
                        "censé"
                    };
                    let sug_casse = adapte_casse(t.mot, c_sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion sensé / censé".into(),
                        explication: "Devant un verbe à l'infinitif, on écrit « censé » (qui signifie supposé faire quelque chose), et non « sensé » (qui a du bon sens).".into(),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « faire parti » -> « faire partie »
            if lower == "parti" && i > 0 && tokens[i - 1].mot.to_lowercase() == "faire" {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Locution faire partie".into(),
                    explication: "Dans la locution verbale « faire partie », « partie » s'écrit avec un « e » final.".into(),
                    suggestions: vec!["partie".into()],
                    genre: GenreErreur::Grammaire,
                });
                continue;
            }

            // « a » vs « à »
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

            // « ce » vs « se »
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

            // « c'est » vs « s'est »
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

            // « ou » vs « où »
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

            // « leur » vs « leurs »
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

            // « si il » / « si ils »
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

            // « quelque soit » -> « quel que soit »
            if (lower == "quelque" || lower == "quelques") && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if suiv == "soit" || suiv == "soient" {
                    let sug = if suiv == "soit" { "quel que soit" } else { "quels que soient" };
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

            // « sans doutes » -> « sans doute »
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

            // « tous le » -> « tout le »
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

            // « je vous envoit » -> « envoie »
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

    /// Règle : Subjonctif obligatoire vs Indicatif après conjonctions
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

            let est_loc_subjonctif = (n1 == "bien" && (n2 == "que" || n2 == "qu'"))
                || (n1 == "quoique" || n1 == "quoiqu'")
                || (n1 == "afin" && (n2 == "que" || n2 == "qu'"))
                || (n1 == "pour" && (n2 == "que" || n2 == "qu'"))
                || (n1 == "avant" && (n2 == "que" || n2 == "qu'"));

            if est_loc_subjonctif {
                let mut v_idx = if n1 == "quoique" || n1 == "quoiqu'" { i + 1 } else { i + 2 };
                if v_idx < tokens.len() && ["il", "elle", "on", "ils", "elles", "tu", "je", "nous", "vous"].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                    v_idx += 1;
                }

                if v_idx < tokens.len() {
                    let t_v = &tokens[v_idx];
                    let v_lower = t_v.mot.to_lowercase();
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

            // « après que » suivi du subjonctif (erreur fréquente : « après qu'il soit venu » -> « est venu »)
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

    /// Règle : Fautes de transitivité & pléonasmes
    fn verifier_transitivite_et_pleonasmes(
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

            // C : Pléonasme « au jour d'aujourd'hui »
            if lower == "au" && i + 2 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "jour" && tokens[i + 2].mot.to_lowercase() == "d'aujourd'hui" {
                let fin_span = tokens[i + 2].fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {} {}", t.mot, tokens[i + 1].mot, tokens[i + 2].mot),
                    regle: "Pléonasme lourd au jour d'aujourd'hui".into(),
                    explication: "L'expression « au jour d'aujourd'hui » est un pléonasme. Utilisez simplement « aujourd'hui », « actuellement » ou « de nos jours ».".into(),
                    suggestions: vec!["aujourd'hui".into(), "actuellement".into(), "de nos jours".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }

            // D : Pléonasme « voire même »
            if lower == "voire" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "même" {
                let fin_span = tokens[i + 1].fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Pléonasme voire même".into(),
                    explication: "« Voire » signifiant déjà « et même », l'association « voire même » constitue un pléonasme. Utilisez « voire » ou « et même ».".into(),
                    suggestions: vec!["voire".into(), "et même".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }
        }
    }

    /// Règle : Pluriels exceptionnels (-al, -ou, -ail)
    fn verifier_pluriels_exceptionnels(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for (i, t) in tokens.iter().enumerate() {
            let lower = t.mot.to_lowercase();

            // A : Noms en -al faisant -aux
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

            // B : 7 exceptions en -ou prenant un -x (bijoux, cailloux...)
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

            // C : Exceptions en -ail prenant -aux (travaux...)
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

    /// Règle : Accord Sujet-Verbe complet (tu, ils/elles, tout le monde, chacun, aucun)
    fn verifier_accord_sujet_verbe(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len().saturating_sub(1) {
            let t_suj = &tokens[i];
            let s_lower = t_suj.mot.to_lowercase();

            // CAS 1 : « tu » + verbe régulier du 1er groupe sans 's'
            if s_lower == "tu" {
                let mut v_idx = i + 1;
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

            // CAS 3 : « tout le monde » + verbe pluriel (« tout le monde sont » -> « est »)
            if s_lower == "tout" && i + 2 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "le" && tokens[i + 2].mot.to_lowercase() == "monde" {
                if i + 3 < tokens.len() {
                    let t_v = &tokens[i + 3];
                    let v_lower = t_v.mot.to_lowercase();
                    let corr = match v_lower.as_str() {
                        "sont" => Some("est"),
                        "ont" => Some("a"),
                        "vont" => Some("va"),
                        "font" => Some("fait"),
                        "savent" => Some("sait"),
                        "peuvent" => Some("peut"),
                        "veulent" => Some("veut"),
                        _ => None,
                    };

                    if let Some(sug) = corr {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_v.deb,
                            fin: decalage + t_v.fin,
                            texte_fautif: t_v.mot.to_string(),
                            regle: "Accord singulier tout le monde".into(),
                            explication: "« Tout le monde » est un sujet singulier : le verbe s'accorde à la 3e personne du singulier.".into(),
                            suggestions: vec![sug.into()],
                            genre: GenreErreur::Accord,
                        });
                    }
                }
            }

            // CAS 4 : « chacun » / « chacune » + verbe pluriel (« chacun ont » -> « chacun a »)
            if s_lower == "chacun" || s_lower == "chacune" {
                let mut v_idx = i + 1;
                if v_idx + 1 < tokens.len() && ["de", "d'"].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                    v_idx += 2; // ex: chacun d'entre nous
                }
                if v_idx < tokens.len() {
                    let t_v = &tokens[v_idx];
                    let v_lower = t_v.mot.to_lowercase();
                    let corr = match v_lower.as_str() {
                        "sont" => Some("est"),
                        "ont" => Some("a"),
                        "peuvent" => Some("peut"),
                        "veulent" => Some("veut"),
                        "savent" => Some("sait"),
                        _ => None,
                    };
                    if let Some(sug) = corr {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_v.deb,
                            fin: decalage + t_v.fin,
                            texte_fautif: t_v.mot.to_string(),
                            regle: "Accord singulier avec chacun".into(),
                            explication: "Le pronom « chacun » est singulier : le verbe doit être à la 3e personne du singulier.".into(),
                            suggestions: vec![sug.into()],
                            genre: GenreErreur::Accord,
                        });
                    }
                }
            }

            // CAS 5 : « aucun » / « aucune » avec un mot au pluriel (« aucune fautes » -> « faute »)
            if s_lower == "aucun" || s_lower == "aucune" {
                let t_nom = &tokens[i + 1];
                let n_lower = t_nom.mot.to_lowercase();
                if n_lower.ends_with('s') && n_lower.len() > 3 && !["frais", "temps", "corps", "cours", "cas"].contains(&n_lower.as_str()) {
                    let sug = t_nom.mot[..t_nom.mot.len() - 1].to_string();
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_nom.deb,
                        fin: decalage + t_nom.fin,
                        texte_fautif: t_nom.mot.to_string(),
                        regle: "Singulier après aucun/aucune".into(),
                        explication: format!("Après « {} », le nom reste toujours au singulier (« {} »).", t_suj.mot, sug),
                        suggestions: vec![sug],
                        genre: GenreErreur::Accord,
                    });
                }
            }
        }
    }

    /// Règle : Accord Déterminant + Nom + Adjectif
    /// « des documents important » -> « importants »
    /// « une démarche administratif » -> « administrative »
    fn verifier_accord_nom_adjectif(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len().saturating_sub(2) {
            let t_det = &tokens[i];
            let det_lower = t_det.mot.to_lowercase();
            let t_nom = &tokens[i + 1];
            let nom_lower = t_nom.mot.to_lowercase();
            let t_adj = &tokens[i + 2];
            let adj_lower = t_adj.mot.to_lowercase();

            // CAS 1 : Déterminant pluriel + nom pluriel + adjectif au singulier
            if self.determinants_pluriel.contains(det_lower.as_str()) && (nom_lower.ends_with('s') || nom_lower.ends_with('x')) {
                // Si l'adjectif est au singulier (ne finit ni par s ni par x)
                if !adj_lower.ends_with('s') && !adj_lower.ends_with('x') && adj_lower.len() > 3 {
                    // Vérifier si ce n'est pas un adverbe ou verbe
                    if ![
                        "bien", "déjà", "souvent", "toujours", "vite", "très", "trop", "assez", "encore",
                        "marron", "orange", "kaki", "turquoise", "est", "sont", "ont", "fait"
                    ].contains(&adj_lower.as_str()) {
                        let suggestion = format!("{}s", t_adj.mot);
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_adj.deb,
                            fin: decalage + t_adj.fin,
                            texte_fautif: t_adj.mot.to_string(),
                            regle: "Accord de l'adjectif au pluriel".into(),
                            explication: format!(
                                "L'adjectif « {} » qualifie le groupe nominal pluriel « {} {} » et doit s'accorder au pluriel (« {} »).",
                                t_adj.mot, t_det.mot, t_nom.mot, suggestion
                            ),
                            suggestions: vec![suggestion],
                            genre: GenreErreur::Accord,
                        });
                    }
                }
            }

            // CAS 2 : Déterminant féminin singulier + nom féminin + adjectif au masculin en -if
            if self.determinants_singulier_fem.contains(det_lower.as_str()) && self.est_nom_feminin(&nom_lower) {
                if adj_lower.ends_with("if") && adj_lower.len() > 3 {
                    let racine = &t_adj.mot[..t_adj.mot.len() - 2];
                    let suggestion = format!("{}ive", racine);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_adj.deb,
                        fin: decalage + t_adj.fin,
                        texte_fautif: t_adj.mot.to_string(),
                        regle: "Accord de l'adjectif au féminin".into(),
                        explication: format!(
                            "Le nom « {} » est féminin : l'adjectif se met au féminin (« {} »).",
                            t_nom.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                }
            }
        }
    }

    /// Règle : Adjectifs numéraux (quatre-vingts, mille invariable, demi)
    fn verifier_adjectifs_numeraux(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for (i, t) in tokens.iter().enumerate() {
            let lower = t.mot.to_lowercase();

            // « milles » avec un s -> toujours invariable « mille »
            if lower == "milles" && i > 0 && ["un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "cent", "plusieurs"].contains(&tokens[i-1].mot.to_lowercase().as_str()) {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Invariabilité du numéral mille".into(),
                    explication: "Le nombre « mille » est toujours invariable en français (jamais de « s »).".into(),
                    suggestions: vec!["mille".into()],
                    genre: GenreErreur::Accord,
                });
            }

            // « quatre-vingt » suivi d'un nom sans chiffre après -> « quatre-vingts »
            if lower == "quatre-vingt" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if ["personnes", "euros", "francs", "ans", "jours", "livres", "fichiers", "documents", "clients"].contains(&suiv.as_str()) {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Accord de quatre-vingts".into(),
                        explication: "« Vingt » prend un « s » lorsqu'il est multiplié et qu'il n'est pas suivi d'un autre chiffre.".into(),
                        suggestions: vec![adapte_casse(t.mot, "quatre-vingts")],
                        genre: GenreErreur::Accord,
                    });
                }
            }

            // « une heure et demi » -> « demie »
            if lower == "demi" && i >= 3 {
                let h = tokens[i - 2].mot.to_lowercase();
                let et = tokens[i - 1].mot.to_lowercase();
                if et == "et" && (h == "heure" || h == "heures") {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Accord de demi après le nom".into(),
                        explication: "Placé après le nom « heure » (féminin), l'adjectif s'accorde en genre : « une heure et demie ».".into(),
                        suggestions: vec!["demie".into()],
                        genre: GenreErreur::Accord,
                    });
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

/// Tokeniseur qui gère les élisions françaises et positions de caractères
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
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "ce");
        assert_eq!(alertes[0].suggestions, vec!["cette"]);
    }

    #[test]
    fn test_contractions_articles() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("aller à le bureau");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].texte_fautif, "à le");
        assert_eq!(a1[0].suggestions, vec!["au"]);

        let a2 = g.analyse("le prix de le projet");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].texte_fautif, "de le");
        assert_eq!(a2[0].suggestions, vec!["du"]);

        let a3 = g.analyse("donner à les fournisseurs");
        assert_eq!(a3.len(), 1);
        assert_eq!(a3[0].texte_fautif, "à les");
        assert_eq!(a3[0].suggestions, vec!["aux"]);

        // Respect du pronom objet direct devant infinitif
        let a4 = g.analyse("afin de le faire correctement");
        assert_eq!(a4.len(), 0);
    }

    #[test]
    fn test_elisions_obligatoires() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("la entreprise");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].suggestions, vec!["l'entreprise"]);

        let a2 = g.analyse("le accord");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].suggestions, vec!["l'accord"]);
    }

    #[test]
    fn test_pronoms_relatifs() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("c'est ce que j'ai besoin");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].texte_fautif, "que");
        assert_eq!(a1[0].suggestions, vec!["dont"]);

        let a2 = g.analyse("la raison pourquoi il est parti");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].texte_fautif, "pourquoi");
        assert_eq!(a2[0].suggestions, vec!["pour laquelle"]);
    }

    #[test]
    fn test_sujets_collectifs() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("tout le monde sont présents");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].texte_fautif, "sont");
        assert_eq!(a1[0].suggestions, vec!["est"]);

        let a2 = g.analyse("chacun ont leur opinion");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].texte_fautif, "ont");
        assert_eq!(a2[0].suggestions, vec!["a"]);

        let a3 = g.analyse("aucune fautes détectées");
        assert_eq!(a3.len(), 1);
        assert_eq!(a3[0].texte_fautif, "fautes");
        assert_eq!(a3[0].suggestions, vec!["faute"]);
    }

    #[test]
    fn test_accords_nom_adjectif() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("des documents important");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].texte_fautif, "important");
        assert_eq!(a1[0].suggestions, vec!["importants"]);

        let a2 = g.analyse("une démarche administratif");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].texte_fautif, "administratif");
        assert_eq!(a2[0].suggestions, vec!["administrative"]);
    }

    #[test]
    fn test_adjectifs_numeraux() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("quatre-vingt personnes");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].texte_fautif, "quatre-vingt");
        assert_eq!(a1[0].suggestions, vec!["quatre-vingts"]);

        let a2 = g.analyse("deux milles euros");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].texte_fautif, "milles");
        assert_eq!(a2[0].suggestions, vec!["mille"]);

        let a3 = g.analyse("une heure et demi");
        assert_eq!(a3.len(), 1);
        assert_eq!(a3[0].texte_fautif, "demi");
        assert_eq!(a3[0].suggestions, vec!["demie"]);
    }

    #[test]
    fn test_homophones_avances() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("il est sensé venir demain");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].texte_fautif, "sensé");
        assert_eq!(a1[0].suggestions, vec!["censé"]);

        let a2 = g.analyse("faire parti du comité");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].texte_fautif, "parti");
        assert_eq!(a2[0].suggestions, vec!["partie"]);

        let a3 = g.analyse("au jour d'aujourd'hui tout va bien");
        assert_eq!(a3.len(), 1);
        assert_eq!(a3[0].texte_fautif, "au jour d'aujourd'hui");

        let a4 = g.analyse("voire même nécessaire");
        assert_eq!(a4.len(), 1);
        assert_eq!(a4[0].texte_fautif, "voire même");
    }

    #[test]
    fn test_la_fille_que_tu_as_vu() {
        let g = CorrecteurGrammaire::new();
        let alertes = g.analyse("La fille que tu as vu");
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "vu");
        assert_eq!(alertes[0].suggestions, vec!["vue"]);
    }

    #[test]
    fn test_les_lunettes_marrons() {
        let g = CorrecteurGrammaire::new();
        let alertes = g.analyse("Les lunettes marrons.");
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "marrons");
        assert_eq!(alertes[0].suggestions, vec!["marron"]);
    }

    #[test]
    fn test_pallier_a() {
        let g = CorrecteurGrammaire::new();
        let alertes = g.analyse("Il faut pallier à ce manque.");
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "pallier à");
        assert_eq!(alertes[0].suggestions, vec!["pallier"]);
    }

    #[test]
    fn test_tu_mange() {
        let g = CorrecteurGrammaire::new();
        let alertes = g.analyse("Tu mange une pomme.");
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "mange");
        assert_eq!(alertes[0].suggestions, vec!["manges"]);
    }

    #[test]
    fn test_bien_qu_il_fait() {
        let g = CorrecteurGrammaire::new();
        let alertes = g.analyse("Bien qu'il fait beau.");
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "fait");
        assert_eq!(alertes[0].suggestions, vec!["fasse"]);
    }

    #[test]
    fn test_pronominal_coi_invariable() {
        let g = CorrecteurGrammaire::new();
        let alertes = g.analyse("Ils se sont parlés longuement.");
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
        assert_eq!(alertes.len(), 1);
        assert_eq!(alertes[0].texte_fautif, "ce");
        assert_eq!(alertes[0].suggestions, vec!["cette"]);
    }
}
'''

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'w', encoding='utf-8') as f:
    f.write(exhaustive_grammaire)

print("Comprehensive French Grammar Engine written successfully!")
