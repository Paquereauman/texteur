//! Moteur de règles grammaticales, syntaxiques et stylistiques 100 % local et autonome en Rust.
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
    pub villes_sans_article: HashSet<&'static str>,
    pub banque_regles: crate::regles_dynamiques::BanqueRegles,
    /// Édition choisie (règles maison), appliquée à tout le texte.
    pub edition: Option<crate::editions::Edition>,
}

impl Default for CorrecteurGrammaire {
    fn default() -> Self {
        Self::new()
    }
}

impl CorrecteurGrammaire {
    pub fn new() -> Self {
        let mut villes = HashSet::new();
        villes.extend([
            "shenzhen", "shanghai", "beijing", "pekin", "pékin", "canton", "guangzhou", "chengdu",
            "wuhan", "tianjin", "hangzhou", "nankin", "nanjing", "chongqing", "xian", "xi'an",
            "suzhou", "zhengzhou", "dongguan", "foshan", "hong kong", "hongkong", "taipei",
            "paris", "lyon", "marseille", "toulouse", "nice", "nantes", "strasbourg", "montpellier",
            "bordeaux", "lille", "rennes", "reims", "toulon", "saint-étienne", "le havre", "grenoble",
            "dijon", "angers", "nîmes", "villeurbanne", "clermont-ferrand", "le mans", "aix-en-provence",
            "brest", "tours", "amiens", "limoges", "annecy", "perpignan", "boulogne-billancourt",
            "metz", "besançon", "orléans", "saint-denis", "argenteuil", "rouen", "mulhouse", "montreuil",
            "caen", "nancy", "londres", "berlin", "madrid", "rome", "tokyo", "seoul", "séoul",
            "singapour", "new york", "los angeles", "chicago", "san francisco", "toronto", "montreal",
            "montréal", "genève", "geneve", "bruxelles",
        ]);
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
            "amie", "amies", "école", "écoles", "adresse", "adresses", "équipe", "équipes", "avance", "avances",
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
            "arbre", "arbres", "avion", "avions", "oiseau", "oiseaux", "hôtel", "hôtels", "animal", "animaux",
            "instant", "instants", "ordinateur", "ordinateurs", "exemple", "exemples", "acte", "actes",
            "espace", "espaces", "événement", "événements", "exercice", "exercices",
        ]);

        // Lexique de genre étendu (ressources/genre_noms.txt)
        for ligne in include_str!("../ressources/genre_noms.txt").lines() {
            let mut it = ligne.split_whitespace();
            let cible = match it.next() {
                Some("f") => &mut noms_fem,
                Some("m") => &mut noms_masc,
                _ => continue,
            };
            cible.extend(it);
        }

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
            villes_sans_article: villes,
            banque_regles: crate::regles_dynamiques::BanqueRegles::charge(),
            edition: None,
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
        if let Some(e) = &self.edition {
            alertes.extend(crate::editions::alertes(e, texte));
        }

                alertes.sort_by_key(|a| (a.debut, a.fin));
        alertes.dedup_by(|a, b| a.debut == b.debut && a.fin == b.fin);
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

        // 20. RÈGLES DYNAMIQUES APPRISES PAR L'IA LOCALE QWEN
        self.verifier_regles_dynamiques(para, decalage, alertes);

        // 21. EXPRESSIONS FAUTIVES, BARBARISMES ET CONFUSIONS COURANTES
        verifier_expressions_courantes(para, decalage, alertes);

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
            if det_norm == "leurs" {
                let t_mot = &tokens[i + 1];
                let m_norm = t_mot.mot.to_lowercase();
                if !m_norm.ends_with('s') && !m_norm.ends_with('x') && !m_norm.ends_with('z') && m_norm.len() > 2 {
                    if !est_infinitif(&m_norm) && !["est", "sont", "ont", "a", "fait", "peut", "doit", "vont", "sera", "seront", "marron", "orange", "kaki", "turquoise"].contains(&m_norm.as_str()) {
                        let suggestion = adapte_casse(t_det.mot, "leur");
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_det.deb,
                            fin: decalage + t_det.fin,
                            texte_fautif: t_det.mot.to_string(),
                            regle: "Accord du déterminant possessif leur".into(),
                            explication: format!("Devant le mot singulier « {} », le déterminant possessif s'écrit sans « s » (« {} »).", t_mot.mot, suggestion),
                            suggestions: vec![suggestion],
                            genre: GenreErreur::Accord,
                        });
                        continue;
                    }
                }
                continue;
            }
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
                    "toujours", "jamais", "parfois", "ailleurs", "nous", "vous", "eux", "ceux", "plus", "moins"
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

        // ====================================================================
        // 20. FORMES VERBALES IRRÉGULIÈRES ET ACCORDS SUJET-VERBE DIRECTS
        // ====================================================================
        for i in 0..tokens.len().saturating_sub(1) {
            let t_s = &tokens[i];
            let s_lower = t_s.mot.to_lowercase();
            let t_v = &tokens[i + 1];
            let v_lower = t_v.mot.to_lowercase();

            // « Vous faisez » -> « Vous faites »
            if s_lower == "vous" && v_lower == "faisez" {
                let sug = adapte_casse(t_v.mot, "faites");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_v.deb,
                    fin: decalage + t_v.fin,
                    texte_fautif: t_v.mot.to_string(),
                    regle: "Forme verbale irrégulière vous faites".into(),
                    explication: "Avec le pronom « vous », la forme correcte du présent de « faire » est « faites » (et non « faisez »).".into(),
                    suggestions: vec![sug],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // « Vous disez » -> « Vous dites »
            if s_lower == "vous" && v_lower == "disez" {
                let sug = adapte_casse(t_v.mot, "dites");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_v.deb,
                    fin: decalage + t_v.fin,
                    texte_fautif: t_v.mot.to_string(),
                    regle: "Forme verbale irrégulière vous dites".into(),
                    explication: "Avec le pronom « vous », la forme correcte du présent de « dire » est « dites » (et non « disez »).".into(),
                    suggestions: vec![sug],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // « Il faut mangé » -> « Il faut manger »
            if s_lower == "faut" && (v_lower.ends_with('é') || v_lower.ends_with("ée")) && v_lower.len() > 3 {
                let sug = format!("{}er", &t_v.mot[..t_v.mot.len() - 2]);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_v.deb,
                    fin: decalage + t_v.fin,
                    texte_fautif: t_v.mot.to_string(),
                    regle: "Infinitif après falloir".into(),
                    explication: format!("Après le verbe « falloir », le verbe qui suit se met à l'infinitif (« {} »).", sug),
                    suggestions: vec![sug],
                    genre: GenreErreur::Infinitif,
                });
                continue;
            }
        }

        // ====================================================================
        // 21. CONDITIONNEL INTERDIT APRÈS « SI » (Si j'aurais -> Si j'avais)
        // ====================================================================
        self.verifier_si_conditionnel(&tokens, decalage, alertes);
    }

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

            let est_h_aspire = [
                "héros", "heros", "haricot", "haricots", "hérisson", "herisson", "haine", "haute", "haut", "hauts",
                "hache", "haches", "hall", "halls", "hibou", "hiboux", "honte", "hamac", "hamacs", "hameau", "hameaux",
                "hausse", "hausses", "hasard", "hasards", "hâte", "hate", "handball", "hacker", "haie", "haies",
                "halo", "halos", "handicap", "hangar", "harceler", "harnais", "harpe", "hasardeux", "havre",
                "hiérarchie", "hierarchie", "horde", "houle", "housse", "huche", "hurler", "hutte"
            ].contains(&w2.as_str());

            let est_exception_voyelle = ["onze", "onzième", "onzieme", "huit", "huitième", "huitieme", "oui"].contains(&w2.as_str());

            let c_first = w2.chars().next().unwrap_or(' ');
            let est_voyelle = ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'ë', 'à', 'â', 'î', 'ï', 'ô', 'û', 'ù'].contains(&c_first);
            let est_h_muet = c_first == 'h' && !est_h_aspire;
            let commence_voyelle = (est_voyelle || est_h_muet) && !est_exception_voyelle;

            if commence_voyelle && !w2.is_empty() {
                let elision = match w1.as_str() {
                    "le" | "la" => Some(format!("l'{}", t2.mot)),
                    "de" => Some(format!("d'{}", t2.mot)),
                    "je" => Some(format!("j'{}", t2.mot)),
                    "me" => Some(format!("m'{}", t2.mot)),
                    "te" => Some(format!("t'{}", t2.mot)),
                    "se" => Some(format!("s'{}", t2.mot)),
                    "ne" => Some(format!("n'{}", t2.mot)),
                    "que" => Some(format!("qu'{}", t2.mot)),
                    "jusque" => Some(format!("jusqu'{}", t2.mot)),
                    "puisque" => Some(format!("puisqu'{}", t2.mot)),
                    "lorsque" => Some(format!("lorsqu'{}", t2.mot)),
                    "quoique" => Some(format!("quoiqu'{}", t2.mot)),
                    "ce" if ["est", "était", "etait", "en", "a"].contains(&w2.as_str()) => Some(format!("c'{}", t2.mot)),
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
                            "Devant un mot commençant par une voyelle ou un h muet, « {} » s'élide obligatoirement en « {} ».",
                            t1.mot, sug_casse
                        ),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // « si il » -> « s'il », « si ils » -> « s'ils »
            if w1 == "si" && (w2 == "il" || w2 == "ils") {
                let sug = if w2 == "ils" { "s'ils" } else { "s'il" };
                let sug_casse = adapte_casse(t1.mot, sug);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t1.deb,
                    fin: decalage + t2.fin,
                    texte_fautif: format!("{} {}", t1.mot, t2.mot),
                    regle: "Élision obligatoire de si".into(),
                    explication: "La conjonction « si » s'élide obligatoirement devant « il » ou « ils ».".into(),
                    suggestions: vec![sug_casse],
                    genre: GenreErreur::Syntaxe,
                });
            }
        }
    }

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

            // Repérer le nom tête (immédiat ou après adjectif intercalé, ex: « une grand problème »)
            let mut offset_nom = 1;
            let adjectifs_courts = [
                "grand", "grande", "grands", "grandes", "petit", "petite", "petits", "petites",
                "gros", "grosse", "gros", "grosses", "beau", "bel", "belle", "beaux", "belles",
                "nouveau", "nouvel", "nouvelle", "nouveaux", "nouvelles", "vieux", "vieil", "vieille", "vieilles",
                "bon", "bonne", "bons", "bonnes", "mauvais", "mauvaise", "jeune", "jeunes",
                "long", "longue", "longs", "longues", "court", "courte", "vrai", "vraie", "faux", "fausse"
            ];
            if i + 2 < tokens.len() && adjectifs_courts.contains(&tokens[i + 1].mot.to_lowercase().as_str()) {
                let w2 = tokens[i + 2].mot.to_lowercase();
                if self.est_nom_masculin(&w2) || self.est_nom_feminin(&w2) {
                    offset_nom = 2;
                }
            }

            let t_nom = &tokens[i + offset_nom];
            let nom_lower = t_nom.mot.to_lowercase();

            // Inversion de pronoms conjoints (ex: « le vous envoie » -> « vous l'envoie »)
            if ["le", "la", "les", "l'"].contains(&det_lower.as_str()) && ["me", "te", "se", "nous", "vous"].contains(&nom_lower.as_str()) {
                let mut suggestions = Vec::new();
                if i + 2 < tokens.len() {
                    let w_suiv = tokens[i + 2].mot.to_lowercase();
                    let commence_voyelle = w_suiv.chars().next().map_or(false, |c| {
                        ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                    });
                    if commence_voyelle && (det_lower == "le" || det_lower == "la" || det_lower == "l'") {
                        suggestions.push(format!("{} l'", t_nom.mot));
                    }
                }
                suggestions.push(format!("{} {}", t_nom.mot, t_det.mot));
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_nom.fin,
                    texte_fautif: format!("{} {}", t_det.mot, t_nom.mot),
                    regle: "Ordre des pronoms personnels conjoints".into(),
                    explication: format!("En français, le pronom « {} » se place avant « {} » (« {} »).", t_nom.mot, t_det.mot, suggestions.join(" ou ")),
                    suggestions,
                    genre: GenreErreur::Syntaxe,
                });
                continue;
            }

            // Inversion de pronoms « lui le » -> « le lui », « leur le » -> « le leur »
            if ["lui", "leur"].contains(&det_lower.as_str()) && ["le", "la", "les", "l'"].contains(&nom_lower.as_str()) {
                let sug = format!("{} {}", t_nom.mot, t_det.mot);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_nom.fin,
                    texte_fautif: format!("{} {}", t_det.mot, t_nom.mot),
                    regle: "Ordre des pronoms personnels conjoints".into(),
                    explication: format!("Le pronom complément d'objet direct « {} » se place avant le pronom indirect « {} » (« {} »).", t_nom.mot, t_det.mot, sug),
                    suggestions: vec![sug],
                    genre: GenreErreur::Syntaxe,
                });
                continue;
            }

            // Euphonie masculine : « ce » devant voyelle/h muet -> « cet » (ex: « Ce arbre »)
            if det_lower == "ce" {
                let voyelle = nom_lower.chars().next().map_or(false, |c| {
                    ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                });
                if voyelle && !["est", "était", "étaient", "eut", "a été"].contains(&nom_lower.as_str()) {
                    let sug = if self.est_nom_feminin(&nom_lower) { "cette" } else { "cet" };
                    let suggestion = adapte_casse(t_det.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Euphonie du démonstratif devant voyelle".into(),
                        explication: format!("Devant le mot « {} » commençant par une voyelle ou un h muet, employez « {} ».", t_nom.mot, suggestion),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // Euphonie féminine : « ma », « ta », « sa » devant voyelle/h muet -> « mon », « ton », « son » (ex: « Ma amie », « Sa entreprise »)
            if ["ma", "ta", "sa"].contains(&det_lower.as_str()) {
                let voyelle = nom_lower.chars().next().map_or(false, |c| {
                    ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                });
                let h_aspire = ["hache", "haie", "haine", "halle", "halte", "honte", "horde", "housse", "hutte", "harpe", "hausse"].contains(&nom_lower.as_str());
                if voyelle && !h_aspire {
                    let sug = match det_lower.as_str() {
                        "ma" => "mon",
                        "ta" => "ton",
                        "sa" => "son",
                        _ => "son",
                    };
                    let suggestion = adapte_casse(t_det.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Euphonie du possessif féminin devant voyelle".into(),
                        explication: format!("Devant le mot « {} » commençant par une voyelle ou un h muet, employez « {} » pour éviter le hiatus.", t_nom.mot, suggestion),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // Pluriels exceptionnels en -al -> -s (ex: des bal -> des bals)
            if det_lower == "des" || det_lower == "les" || det_lower == "ces" || det_lower == "mes" || det_lower == "tes" || det_lower == "ses" {
                if ["bal", "carnaval", "chacal", "festival", "régal", "récital"].contains(&nom_lower.as_str()) {
                    let suggestion = format!("{}s", t_nom.mot);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_nom.deb,
                        fin: decalage + t_nom.fin,
                        texte_fautif: t_nom.mot.to_string(),
                        regle: "Pluriel en -s des noms en -al".into(),
                        explication: format!("Le nom « {} » fait exception et prend un « s » au pluriel (« {} »).", t_nom.mot, suggestion),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
                if nom_lower == "chovals" {
                    let suggestion = adapte_casse(t_nom.mot, "chevaux");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_nom.deb,
                        fin: decalage + t_nom.fin,
                        texte_fautif: t_nom.mot.to_string(),
                        regle: "Pluriel irrégulier de cheval".into(),
                        explication: "Le pluriel de « cheval » est « chevaux ».".into(),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // « Tout les » -> « Tous les » / « Tous le » -> « Tout le »
            if (det_lower == "tout" || det_lower == "tous" || det_lower == "toute" || det_lower == "toutes") && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if det_lower == "tout" && suiv == "les" {
                    let sug = adapte_casse(t_det.mot, "tous");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Accord de tout devant les".into(),
                        explication: "Devant l'article pluriel « les », écrivez « tous » au masculin pluriel.".into(),
                        suggestions: vec![sug],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
                if det_lower == "tous" && suiv == "le" {
                    let sug = adapte_casse(t_det.mot, "tout");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Accord de tout devant le".into(),
                        explication: "Devant l'article singulier « le », écrivez « tout » au singulier.".into(),
                        suggestions: vec![sug],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // « Quelque soit » -> « Quel que soit » / « Quelle que soit »
            if (det_lower == "quelque" || det_lower == "quelques") && i + 1 < tokens.len() && (tokens[i + 1].mot.to_lowercase() == "soit" || tokens[i + 1].mot.to_lowercase() == "soient") {
                let t_soit = &tokens[i + 1];
                let fin_span = t_soit.fin;
                let mut sug = "Quel que soit";
                if i + 2 < tokens.len() {
                    let n = tokens[i + 2].mot.to_lowercase();
                    if self.est_nom_feminin(&n) || n == "la" || n == "une" || n == "cette" {
                        sug = "Quelle que soit";
                    }
                }
                let sug_casse = adapte_casse(t_det.mot, sug);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {}", t_det.mot, t_soit.mot),
                    regle: "Locution quel que soit".into(),
                    explication: "Devant le verbe être au subjonctif, « quel que » s'écrit en deux mots et s'accorde avec le sujet.".into(),
                    suggestions: vec![sug_casse],
                    genre: GenreErreur::Syntaxe,
                });
                continue;
            }

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

            // CAS C : Déterminant masculin singulier devant nom féminin (ex: « un réunion », « le solution », « ce difficulté »)
            // CAS C' : « du / au » devant nom féminin (« du démarche » → « de la démarche »)
            // Suffixes les plus fiables seulement, « du / au » étant très fréquents
            let fem_sur = self.noms_feminins_courants.contains(nom_lower.as_str())
                || (!self.noms_masculins_courants.contains(nom_lower.as_str())
                    && ["tion", "sion", "ie", "ence", "ance"].iter().any(|s| nom_lower.ends_with(s)));
            if (det_lower == "du" || det_lower == "au") && fem_sur {
                let voyelle = nom_lower.chars().next().is_some_and(|c| "aeiouyéèêàâîôûh".contains(c));
                let sug = match (det_lower.as_str(), voyelle) {
                    ("du", false) => "de la",
                    ("du", true) => "de l'",
                    (_, false) => "à la",
                    (_, true) => "à l'",
                };
                let suggestion = adapte_casse(t_det.mot, sug);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_det.fin,
                    texte_fautif: t_det.mot.to_string(),
                    regle: "Article contracté devant un nom féminin".into(),
                    explication: format!("« {} » est la contraction de « {} le » (masculin) ; « {} » est féminin : « {} {} ».", t_det.mot, if det_lower == "du" { "de" } else { "à" }, t_nom.mot, suggestion, t_nom.mot),
                    suggestions: vec![suggestion],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            if ["un", "le", "ce", "cet", "mon", "ton", "son"].contains(&det_lower.as_str()) && self.est_nom_feminin(&nom_lower) {
                let commence_voyelle = nom_lower.chars().next().map_or(false, |c| {
                    ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                });
                if !(["mon", "ton", "son"].contains(&det_lower.as_str()) && commence_voyelle) && !(det_lower == "le" && commence_voyelle) {
                    let sug = match det_lower.as_str() {
                        "un" => "une",
                        "le" => "la",
                        "ce" | "cet" => "cette",
                        "mon" => "ma",
                        "ton" => "ta",
                        "son" => "sa",
                        _ => "une",
                    };
                    let suggestion = adapte_casse(t_det.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Accord en genre du déterminant (féminin requis)".into(),
                        explication: format!("Le nom « {} » est féminin : on emploie « {} » (et non « {} »).", t_nom.mot, suggestion, t_det.mot),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // CAS D : Déterminant féminin singulier devant nom masculin (ex: « une problème », « la point », « cette projet », « sa point »)
            if ["une", "la", "cette", "ma", "ta", "sa"].contains(&det_lower.as_str()) && self.est_nom_masculin(&nom_lower) {
                let commence_voyelle = nom_lower.chars().next().map_or(false, |c| {
                    ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                });
                let sug = match det_lower.as_str() {
                    "une" => "un",
                    "la" => if commence_voyelle { "l'" } else { "le" },
                    "cette" => if commence_voyelle { "cet" } else { "ce" },
                    "ma" => "mon",
                    "ta" => "ton",
                    "sa" => if i > 0 && tokens[i - 1].mot.to_lowercase() == "faire" && nom_lower == "point" { "le" } else { "son" },
                    _ => "le",
                };
                let mut sugs = vec![adapte_casse(t_det.mot, sug)];
                if det_lower == "sa" && nom_lower == "point" {
                    sugs.push(adapte_casse(t_det.mot, "son"));
                }
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_det.fin,
                    texte_fautif: t_det.mot.to_string(),
                    regle: "Accord en genre du déterminant (masculin requis)".into(),
                    explication: format!("Le nom « {} » est masculin : on emploie « {} » (et non « {} »).", t_nom.mot, sugs.join(" ou "), t_det.mot),
                    suggestions: sugs,
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
            // « Par contre » en début de phrase -> « En revanche »
            if (lower == "par" || lower == "par contre") && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "contre" {
                let est_debut = i == 0 || [".", ";", "!", "?"].contains(&tokens[i - 1].mot);
                if est_debut {
                    let fin_span = tokens[i + 1].fin;
                    let sug = adapte_casse(t.mot, "En revanche");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + fin_span,
                        texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                        regle: "Locution en revanche vs par contre".into(),
                        explication: "En tête de phrase dans un style soigné, préférez « En revanche » ou « Toutefois » à « Par contre ».".into(),
                        suggestions: vec![sug, adapte_casse(t.mot, "Toutefois")],
                        genre: GenreErreur::Syntaxe,
                    });
                    continue;
                }
            }

            // « Comme par exemple » -> « Comme » ou « Par exemple »
            if lower == "comme" && i + 2 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "par" && tokens[i + 2].mot.to_lowercase() == "exemple" {
                let fin_span = tokens[i + 2].fin;
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {} {}", t.mot, tokens[i + 1].mot, tokens[i + 2].mot),
                    regle: "Pléonasme comme par exemple".into(),
                    explication: "« Comme par exemple » est un pléonasme. Choisissez « Comme » ou « Par exemple ».".into(),
                    suggestions: vec![adapte_casse(t.mot, "comme"), adapte_casse(t.mot, "par exemple")],
                    genre: GenreErreur::Syntaxe,
                });
                continue;
            }

            // « Où que » populaire -> supprimer « que » (ex: « Où que tu vas »)
            if lower == "que" && i > 0 && ["où", "ou", "quand"].contains(&tokens[i - 1].mot.to_lowercase().as_str()) {
                if i + 1 < tokens.len() && ["tu", "vous", "il", "elle", "on", "ils", "elles", "je"].contains(&tokens[i + 1].mot.to_lowercase().as_str()) {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Tournure interrogative avec que superflue".into(),
                        explication: "Dans une interrogation directe ou indirecte, « que » est superflu après « où » ou « quand ».".into(),
                        suggestions: vec!["".into()],
                        genre: GenreErreur::Syntaxe,
                    });
                    continue;
                }
            }

            // « Quand à » -> « Quant à »
            if lower == "quand" && i + 1 < tokens.len() && (tokens[i + 1].mot.to_lowercase() == "à" || tokens[i + 1].mot.to_lowercase() == "a") {
                let t_a = &tokens[i + 1];
                let fin_span = t_a.fin;
                let sug = adapte_casse(t.mot, "Quant à");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + fin_span,
                    texte_fautif: format!("{} {}", t.mot, t_a.mot),
                    regle: "Confusion quand à / quant à".into(),
                    explication: "Dans le sens de « pour ce qui est de », écrivez « quant à » (avec un t final).".into(),
                    suggestions: vec![sug],
                    genre: GenreErreur::Homophone,
                });
                continue;
            }

            // « sa va » -> « ça va »
            if lower == "sa" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if ["va", "marche", "dépend", "depend", "fait", "semble", "suffit"].contains(&suiv.as_str()) {
                    let sug = adapte_casse(t.mot, "ça");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion sa / ça".into(),
                        explication: format!("Devant le verbe « {} », écrivez le pronom démonstratif « ça » (avec cédille) et non « sa ».", tokens[i + 1].mot),
                        suggestions: vec![sug],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « Machine a café » -> « Machine à café »
            if lower == "a" && i > 0 {
                let prec = tokens[i - 1].mot.to_lowercase();
                if ["machine", "fer", "tasse", "sac", "boîte", "boite", "verre", "salle", "chambre", "moulin", "planche", "grâce", "grace", "face", "quant"].contains(&prec.as_str()) {
                    let sug = adapte_casse(t.mot, "à");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Préposition à dans nom composé".into(),
                        explication: "Il s'agit de la préposition « à » (avec accent) reliant les deux termes.".into(),
                        suggestions: vec![sug],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « ces arrivé » / « il ces trompé »
            if lower == "ces" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if suiv.ends_with('é') || suiv.ends_with("ée") || suiv.ends_with('i') || suiv.ends_with('u') || suiv.ends_with('t') {
                    if ["arrivé", "arrive", "passé", "passe", "déroulé", "deroule", "produit", "trompé", "trompe", "terminé", "termine", "fini"].contains(&suiv.as_str()) {
                        let a_sujet = i > 0 && ["il", "elle", "on", "qui"].contains(&tokens[i - 1].mot.to_lowercase().as_str());
                        let sug = if a_sujet { "s'est" } else { "c'est" };
                        let sug_casse = adapte_casse(t.mot, sug);
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion ces / c'est / s'est".into(),
                            explication: format!("Devant le participe passé « {} », écrivez « {} » (et non le déterminant pluriel « ces »).", tokens[i + 1].mot, sug_casse),
                            suggestions: vec![sug_casse],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            }

            // « Ils son » -> « Ils sont »
            if lower == "son" && i > 0 && ["ils", "elles"].contains(&tokens[i - 1].mot.to_lowercase().as_str()) {
                let sug = adapte_casse(t.mot, "sont");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Confusion son / sont".into(),
                    explication: "Après le pronom sujet pluriel, il s'agit du verbe être « sont » (avec t).".into(),
                    suggestions: vec![sug],
                    genre: GenreErreur::Homophone,
                });
                continue;
            }

            // « leurs adresse » -> « leur adresse »
            if lower == "leurs" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if !suiv.ends_with('s') && !suiv.ends_with('x') && (self.est_nom_feminin(&suiv) || self.est_nom_masculin(&suiv)) {
                    let sug = adapte_casse(t.mot, "leur");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Accord de leur devant nom singulier".into(),
                        explication: format!("Devant le nom singulier « {} », le possessif s'écrit « leur » (sans s).", tokens[i + 1].mot),
                        suggestions: vec![sug],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // « un peut de » -> « un peu de »
            if (lower == "peut" || lower == "peux") && i > 0 && tokens[i - 1].mot.to_lowercase() == "un" {
                let sug = adapte_casse(t.mot, "peu");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Adverbe de quantité un peu".into(),
                    explication: "Dans la locution « un peu », « peu » est invariable et ne prend ni t ni x.".into(),
                    suggestions: vec![sug],
                    genre: GenreErreur::Homophone,
                });
                continue;
            }

            // « Je peu » -> « Je peux »
            if lower == "peu" && i > 0 && ["je", "tu"].contains(&tokens[i - 1].mot.to_lowercase().as_str()) {
                let sug = adapte_casse(t.mot, "peux");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Conjugaison du verbe pouvoir".into(),
                    explication: format!("Avec « {} », le verbe pouvoir s'écrit « peux » (avec un x).", tokens[i - 1].mot),
                    suggestions: vec![sug],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // « Il peux » -> « Il peut »
            if lower == "peux" && i > 0 && ["il", "elle", "on", "qui"].contains(&tokens[i - 1].mot.to_lowercase().as_str()) {
                let sug = adapte_casse(t.mot, "peut");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Conjugaison du verbe pouvoir".into(),
                    explication: format!("À la 3e personne (« {} »), le verbe pouvoir s'écrit « peut » (avec un t).", tokens[i - 1].mot),
                    suggestions: vec![sug],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // « on confirmai » -> « ont confirmé »
            if (lower == "confirmai" || lower == "mangai" || lower == "participai") && i > 0 && ["on", "ont", "a"].contains(&tokens[i - 1].mot.to_lowercase().as_str()) {
                let pp = format!("{}é", &t.mot[..t.mot.len() - 2]);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Participe passé après avoir".into(),
                    explication: format!("Après l'auxiliaire avoir, le verbe se met au participe passé (« {} ») et non au passé simple.", pp),
                    suggestions: vec![pp],
                    genre: GenreErreur::ParticipePasse,
                });
                continue;
            }

            // « concernent » -> « concernant » (ou « concerne »)
            if lower == "concernent" {
                let mut est_faute = false;
                let mut sug = "concernant";

                if i > 0 {
                    let prec = tokens[i - 1].mot.to_lowercase();
                    if prec == "qui" {
                        if i >= 2 {
                            let ante = tokens[i - 2].mot.to_lowercase();
                            if !ante.ends_with('s') && !ante.ends_with('x') || self.determinants_singulier_masc.contains(ante.as_str()) || self.determinants_singulier_fem.contains(ante.as_str()) {
                                est_faute = true;
                                sug = "concerne";
                            }
                        }
                    } else if ["en", "de"].contains(&prec.as_str()) && i >= 2 && tokens[i - 2].mot.to_lowercase() == "ce" {
                        est_faute = true;
                        sug = "concerne";
                    } else {
                        let prec_est_sujet_pluriel = if ["ils", "elles", "ceux-ci", "celles-ci"].contains(&prec.as_str()) {
                            true
                        } else if prec.ends_with('s') || prec.ends_with('x') {
                            i >= 2 && self.determinants_pluriel.contains(tokens[i - 2].mot.to_lowercase().as_str())
                        } else {
                            false
                        };

                        if !prec_est_sujet_pluriel {
                            est_faute = true;
                            sug = "concernant";
                        }
                    }
                } else {
                    est_faute = true;
                    sug = "concernant";
                }

                if est_faute {
                    let sug_casse = adapte_casse(t.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion concernent / concernant".into(),
                        explication: format!(
                            "Dans ce contexte, employez la préposition / participe présent « {} » (et non la forme verbale conjuguée à la 3e personne du pluriel « concernent »).",
                            sug_casse
                        ),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « du » ou « au » + nom de ville -> « de [ville] » ou « à [ville] »
            if (lower == "du" || lower == "au") && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if self.villes_sans_article.contains(suiv.as_str()) {
                    let sug = if lower == "du" { "de" } else { "à" };
                    let sug_casse = adapte_casse(t.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: format!("Préposition {} devant nom de ville", sug),
                        explication: format!(
                            "Les noms de ville ne prennent pas d'article contracté : employez la préposition « {} » (« {} {} ») et non « {} ».",
                            sug, sug, tokens[i + 1].mot, t.mot
                        ),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Grammaire,
                    });
                    continue;
                }
            }

            // « se » vs « ce » (devant un nom) : ex. « se problème » -> « ce problème »
            if lower == "se" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                let est_nom = self.est_nom_masculin(&suiv) || self.est_nom_feminin(&suiv) || [
                    "problème", "probleme", "projet", "document", "dossier", "contrat", "rapport", "matin", "soir", "jour",
                    "client", "type", "cas", "modèle", "modele", "système", "systeme", "rôle", "role", "service", "point",
                    "monde", "temps", "groupe", "texte", "sens", "prix", "fait", "moment", "cours", "sujet", "livraison", "réunion", "reunion"
                ].contains(&suiv.as_str());

                let est_verbe = est_infinitif(&suiv) || [
                    "passe", "rend", "souvient", "trompe", "trouve", "font", "sont", "déroule", "deroule", "pose", "met", "situe", "veut", "doit", "peut"
                ].contains(&suiv.as_str());

                if est_nom && !est_verbe {
                    let sug = if self.est_nom_feminin(&suiv) {
                        "cette"
                    } else {
                        let commence_voyelle = suiv.chars().next().map_or(false, |c| {
                            ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                        });
                        if commence_voyelle { "cet" } else { "ce" }
                    };
                    let sug_casse = adapte_casse(t.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion se / ce (pronom vs déterminant démonstratif)".into(),
                        explication: format!(
                            "Devant le nom « {} », on emploie le déterminant démonstratif « {} » et non le pronom réfléchi « se ».",
                            tokens[i + 1].mot, sug_casse
                        ),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « on » vs « ont » (homophone pronom indéfini vs verbe avoir pluriel)
            if lower == "on" {
                let mut sujet_pluriel = false;
                let mut p = i;
                while p > 0 {
                    p -= 1;
                    let w = tokens[p].mot.to_lowercase();
                    if w == "." || w == ";" || w == "!" || w == "?" || w == "," {
                        break;
                    }
                    if ["ils", "elles", "les", "ces", "des", "mes", "tes", "ses", "nos", "vos", "leurs"].contains(&w.as_str()) {
                        sujet_pluriel = true;
                        break;
                    }
                }

                let mut suiv_est_participe = false;
                if i + 1 < tokens.len() {
                    let s = tokens[i + 1].mot.to_lowercase();
                    if s.ends_with('é') || s.ends_with("és") || s.ends_with("ée") || s.ends_with("ées")
                        || s.ends_with('i') || s.ends_with('u') || s.ends_with('t')
                        || s.ends_with("ai") || s.ends_with("er") {
                        suiv_est_participe = true;
                    }
                }

                if sujet_pluriel && suiv_est_participe {
                    let sug_casse = adapte_casse(t.mot, "ont");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion on / ont (pronom vs auxiliaire avoir)".into(),
                        explication: "Avec le sujet pluriel précédent, employez la 3e personne du pluriel du verbe avoir « ont » et non le pronom indéfini « on ».".into(),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }
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
            if lower == "au" && i + 2 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "jour" {
                if tokens[i + 2].mot.to_lowercase() == "d'aujourd'hui" {
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
                } else if tokens[i + 2].mot.to_lowercase() == "d'" && i + 3 < tokens.len() && tokens[i + 3].mot.to_lowercase() == "aujourd'hui" {
                    let fin_span = tokens[i + 3].fin;
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + fin_span,
                        texte_fautif: format!("{} {} {}{}", t.mot, tokens[i + 1].mot, tokens[i + 2].mot, tokens[i + 3].mot),
                        regle: "Pléonasme lourd au jour d'aujourd'hui".into(),
                        explication: "L'expression « au jour d'aujourd'hui » est un pléonasme. Utilisez simplement « aujourd'hui », « actuellement » ou « de nos jours ».".into(),
                        suggestions: vec!["aujourd'hui".into(), "actuellement".into(), "de nos jours".into()],
                        genre: GenreErreur::Syntaxe,
                    });
                }
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
        for t in tokens {
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

            // CAS 0 : « je » / « j' » + verbe régulier avec terminaison incorrecte en 'es' (ex: « je envoies », « je vous envoies »)
            if s_lower == "je" || s_lower == "j'" {
                let mut v_idx = i + 1;
                while v_idx < tokens.len() && [
                    "me", "m'", "te", "t'", "se", "s'", "le", "la", "les", "l'",
                    "lui", "leur", "nous", "vous", "en", "y", "ne", "n'"
                ].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                    v_idx += 1;
                }

                if v_idx < tokens.len() {
                    let t_v = &tokens[v_idx];
                    let v_lower = t_v.mot.to_lowercase();
                    if v_lower.ends_with("es") && v_lower.len() > 3 && ![
                        "des", "les", "mes", "tes", "ses", "ces", "près", "après", "très", "certes"
                    ].contains(&v_lower.as_str()) {
                        let suggestion = t_v.mot[..t_v.mot.len() - 1].to_string();
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_v.deb,
                            fin: decalage + t_v.fin,
                            texte_fautif: t_v.mot.to_string(),
                            regle: "Accord sujet-verbe avec je".into(),
                            explication: format!(
                                "Avec le pronom sujet « {} », les verbes du 1er groupe se terminent par -e (« {} ») et non par -es.",
                                t_suj.mot, suggestion
                            ),
                            suggestions: vec![suggestion],
                            genre: GenreErreur::Accord,
                        });
                    }
                }
            }

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

    /// Règle : Conditionnel interdit après « si » ou « s' » (ex: « Si j'aurais » -> « Si j'avais »)
    fn verifier_si_conditionnel(
        &self,
        tokens: &[Token],
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        for i in 0..tokens.len() {
            let mot_si = tokens[i].mot.to_lowercase();
            if mot_si != "si" && mot_si != "s'" {
                continue;
            }

            // Sauter les pronoms sujets et clitiques
            let mut v_idx = i + 1;
            while v_idx < tokens.len() && [
                "j'", "je", "tu", "il", "ils", "elle", "elles", "on", "nous", "vous",
                "ce", "c'", "ne", "n'", "se", "s'", "me", "m'", "te", "t'", "lui", "leur", "y", "en"
            ].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                v_idx += 1;
            }

            if v_idx < tokens.len() {
                let t_v = &tokens[v_idx];
                let v_lower = t_v.mot.to_lowercase();

                let sug_opt: Option<&'static str> = match v_lower.as_str() {
                    "aurais" => Some("avais"),
                    "aurait" => Some("avait"),
                    "aurions" => Some("avions"),
                    "auriez" => Some("aviez"),
                    "auraient" => Some("avaient"),
                    "serais" => Some("étais"),
                    "serait" => Some("était"),
                    "serions" => Some("étions"),
                    "seriez" => Some("étiez"),
                    "seraient" => Some("étaient"),
                    "ferais" => Some("faisais"),
                    "ferait" => Some("faisait"),
                    "irais" => Some("allais"),
                    "irait" => Some("allait"),
                    "pourrais" => Some("pouvais"),
                    "pourrait" => Some("pouvait"),
                    "saurais" => Some("savais"),
                    "saurait" => Some("savait"),
                    "voudrais" => Some("voulais"),
                    "voudrait" => Some("voulait"),
                    _ => {
                        if (v_lower.ends_with("erais") || v_lower.ends_with("erait")) && v_lower.len() > 5 {
                            Some("")
                        } else {
                            None
                        }
                    }
                };

                if let Some(sug_base) = sug_opt {
                    let sug_calc = if sug_base.is_empty() {
                        if v_lower.ends_with("erais") {
                            format!("{}ais", &t_v.mot[..t_v.mot.len() - 5])
                        } else {
                            format!("{}ait", &t_v.mot[..t_v.mot.len() - 5])
                        }
                    } else {
                        adapte_casse(t_v.mot, sug_base)
                    };

                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_v.deb,
                        fin: decalage + t_v.fin,
                        texte_fautif: t_v.mot.to_string(),
                        regle: "Conditionnel interdit après si".into(),
                        explication: format!("Après la conjonction « si », on emploie l'imparfait (« {} ») et non le conditionnel (« les si n'aiment pas les -rais »).", sug_calc),
                        suggestions: vec![sug_calc],
                        genre: GenreErreur::Syntaxe,
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
        if self.noms_masculins_courants.contains(m.as_str()) {
            return false;
        }
        m.ends_with("ie")
            || m.ends_with("tion")
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
        if self.noms_feminins_courants.contains(m.as_str()) {
            return false;
        }
        m.ends_with("ment")
            || m.ends_with("age")
            || m.ends_with("isme")
            || m.ends_with("eau")
            || m.ends_with("oir")
            || m.ends_with("teur")
    }

    /// Analyse et applique les règles dynamiques apprises par l'IA locale Qwen
    pub fn verifier_regles_dynamiques(
        &self,
        para: &str,
        decalage: usize,
        alertes: &mut Vec<AlerteGrammaticale>,
    ) {
        if self.banque_regles.regles.is_empty() {
            return;
        }

        let car: Vec<char> = para.chars().collect();
        for r in &self.banque_regles.regles {
            if !r.est_applicable() {
                continue;
            }
            for (deb_char, fin_char) in chercher_motif(&car, &r.motif_fautif, r.mot_entier) {
                // Règle contextuelle : le mot précédent doit correspondre (ex. « il a » ≠ « à »)
                if !r.contexte_avant.is_empty()
                    && !mot_precedent(&car, deb_char).is_some_and(|m| m.to_lowercase() == r.contexte_avant.to_lowercase())
                {
                    continue;
                }
                let texte_fautif: String = car[deb_char..fin_char].iter().collect();
                let suggestion = adapte_casse(&texte_fautif, &r.suggestion);

                if !alertes.iter().any(|a| a.debut == decalage + deb_char && a.fin == decalage + fin_char) {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + deb_char,
                        fin: decalage + fin_char,
                        texte_fautif,
                        regle: format!("✦ {}", r.regle),
                        explication: r.explication.clone(),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Syntaxe,
                    });
                }
            }
        }
    }


}

/// Normalise l'apostrophe typographique pour les comparaisons.
fn norm_car(c: char) -> char {
    if c == '’' { '\'' } else { c.to_lowercase().next().unwrap_or(c) }
}

/// Recherche insensible à la casse (et à l'apostrophe ’/') d'un motif, en indices de caractères.
pub fn chercher_motif(car: &[char], motif: &str, mot_entier: bool) -> Vec<(usize, usize)> {
    let m: Vec<char> = motif.chars().map(norm_car).collect();
    let mut res = Vec::new();
    if m.is_empty() || m.len() > car.len() {
        return res;
    }
    let mut i = 0;
    while i + m.len() <= car.len() {
        if (0..m.len()).all(|k| norm_car(car[i + k]) == m[k]) {
            let f = i + m.len();
            let bord_ok = !mot_entier
                || (!(i > 0 && car[i - 1].is_alphanumeric()) && !car.get(f).is_some_and(|c| c.is_alphanumeric()));
            if bord_ok {
                res.push((i, f));
                i = f;
                continue;
            }
        }
        i += 1;
    }
    res
}

/// Mot qui précède immédiatement la position `deb` (en caractères).
pub fn mot_precedent(car: &[char], deb: usize) -> Option<String> {
    let mut f = deb;
    while f > 0 && !car[f - 1].is_alphanumeric() {
        if car[f - 1] == '.' || car[f - 1] == '!' || car[f - 1] == '?' {
            return None;
        }
        f -= 1;
    }
    let mut d = f;
    while d > 0 && car[d - 1].is_alphanumeric() {
        d -= 1;
    }
    (d < f).then(|| car[d..f].iter().collect())
}

/// (motif fautif, correction, règle, explication)
const EXPRESSIONS_FAUTIVES: &[(&str, &str, &str, &str)] = &[
    // Relevées en relisant un livre entier (rapport Suhou)
    ("de la moitié de la dynastie", "du milieu de la dynastie", "Expression figée", "On dit « le milieu de la dynastie », pas « la moitié »."),
    ("à la moitié de la dynastie", "au milieu de la dynastie", "Expression figée", "On dit « le milieu de la dynastie », pas « la moitié »."),
    ("se passionnait à", "se passionnait pour", "Construction verbale", "On se passionne « pour » quelque chose."),
    ("se passionne à", "se passionne pour", "Construction verbale", "On se passionne « pour » quelque chose."),
    ("se passionner à", "se passionner pour", "Construction verbale", "On se passionne « pour » quelque chose."),
    ("pousser de l'avant", "aller de l'avant", "Expression figée", "La locution est « aller de l'avant »."),
    ("prêt sur gage", "prêt sur gages", "Expression figée", "On écrit « prêt sur gages » (gages au pluriel)."),
    ("prêts sur gage", "prêts sur gages", "Expression figée", "On écrit « prêts sur gages » (gages au pluriel)."),
    ("risquaient grand", "risquaient gros", "Expression figée", "La locution est « risquer gros »."),
    ("risquait grand", "risquait gros", "Expression figée", "La locution est « risquer gros »."),
    ("risquer grand", "risquer gros", "Expression figée", "La locution est « risquer gros »."),
    ("années troublantes", "années troublées", "Confusion", "« Troublant » = déconcertant ; une période agitée est « troublée »."),
    ("période troublante", "période troublée", "Confusion", "« Troublant » = déconcertant ; une période agitée est « troublée »."),
    ("inconvenable", "inconvenant", "Barbarisme", "« Inconvenable » n'existe pas : « inconvenant »."),
    ("inconvenables", "inconvenants", "Barbarisme", "« Inconvenable » n'existe pas : « inconvenant »."),
    ("inconvenablement", "de manière inconvenante", "Barbarisme", "« Inconvenablement » n'existe pas."),
    ("indivisaire", "indivis", "Barbarisme", "« Indivisaire » n'existe pas : un patrimoine « indivis »."),
    ("piégeux", "piégé", "Barbarisme", "« Piégeux » n'existe pas."),
    ("connu comme étant", "connu comme", "Pléonasme", "« comme » suffit : « étant » est superflu."),
    ("connue comme étant", "connue comme", "Pléonasme", "« comme » suffit : « étant » est superflu."),
    ("tenu en compte", "pris en compte", "Expression figée", "On dit « prendre en compte » ou « tenir compte de »."),
    ("tenue en compte", "prise en compte", "Expression figée", "On dit « prendre en compte » ou « tenir compte de »."),
    ("tenir en compte", "prendre en compte", "Expression figée", "On dit « prendre en compte » ou « tenir compte de »."),
    ("insistons de", "insistons pour", "Construction verbale", "On insiste « pour » faire quelque chose."),
    ("insistait de", "insistait pour", "Construction verbale", "On insiste « pour » faire quelque chose."),
    ("insiste de", "insiste pour", "Construction verbale", "On insiste « pour » faire quelque chose."),
    ("deux centaines de", "deux cents", "Nombre", "On dit « deux cents » (ou « environ deux cents »)."),
    ("trois centaines de", "trois cents", "Nombre", "On dit « trois cents » (ou « environ trois cents »)."),
    ("à aux", "aux", "Préposition en double", "« à » est répété : « aux » contient déjà « à »."),
    ("aplatir une route", "aplanir une route", "Confusion", "On « aplanit » une route."),
    ("en écriant", "en écrivant", "Barbarisme", "« écrier » n'existe pas sans « s' » : « en écrivant » ?"),
    ("jouer avec des instruments", "jouer des instruments", "Construction verbale", "On joue « d'un » instrument."),
    ("renchéri cette", "renchéri sur cette", "Construction verbale", "« renchérir » se construit avec « sur »."),
    ("renchéri ce", "renchéri sur ce", "Construction verbale", "« renchérir » se construit avec « sur »."),
    ("renchéri ces", "renchéri sur ces", "Construction verbale", "« renchérir » se construit avec « sur »."),
    ("une abîme", "un abîme", "Genre", "« abîme » est masculin."),
    ("de la jade", "du jade", "Genre", "« jade » est masculin."),
    ("jades précieuses", "jades précieux", "Genre", "« jade » est masculin."),
    ("jade précieuse", "jade précieux", "Genre", "« jade » est masculin."),
    ("une vingtième de", "un vingtième de", "Genre", "La fraction « vingtième » est masculine."),
    ("une dixième de", "un dixième de", "Genre", "La fraction « dixième » est masculine."),
    ("une centième de", "un centième de", "Genre", "La fraction « centième » est masculine."),
    ("œuvrer une", "mener une", "Construction verbale", "« œuvrer » est intransitif."),
    ("sur un pied de l'égalité", "sur un pied d'égalité", "Expression figée", "La locution est « sur un pied d'égalité »."),
    ("faisait dons de", "faisait don de", "Expression figée", "La locution est « faire don de »."),
    ("faire dons de", "faire don de", "Expression figée", "La locution est « faire don de »."),
    ("pour l'objectif de", "pour objectif de", "Expression figée", "La locution est « avoir pour objectif de »."),
    ("dans les perspectives de", "dans la perspective de", "Expression figée", "La locution est « dans la perspective de »."),
    ("la dynastie Ming et Qing", "les dynasties Ming et Qing", "Accord", "Deux dynasties : pluriel."),
    ("la l'", "l'", "Article en double", "Deux articles se suivent."),
    ("le l'", "l'", "Article en double", "Deux articles se suivent."),
    ("leur l'", "l'", "Article en double", "Deux déterminants se suivent."),
    ("e l'an", "en l'an", "Coquille", "Il manque le « n » de « en »."),
    // Tournures lourdes et pléonasmes courants
    ("suite à", "à la suite de", "Style : tournure critiquée", "« Suite à » relève du jargon administratif ; à l'écrit soigné : « à la suite de », « après »."),
    ("prévoir à l'avance", "prévoir", "Pléonasme", "« Prévoir » signifie déjà « voir à l'avance »."),
    ("prévu à l'avance", "prévu", "Pléonasme", "« Prévoir » signifie déjà « voir à l'avance »."),
    ("collaborer ensemble", "collaborer", "Pléonasme", "« Collaborer » signifie déjà « travailler ensemble »."),
    ("réunir ensemble", "réunir", "Pléonasme", "« Réunir » implique déjà « ensemble »."),
    ("puis ensuite", "puis", "Pléonasme", "« Puis » et « ensuite » disent la même chose : gardez-en un."),
    ("et puis ensuite", "puis", "Pléonasme", "« Puis » et « ensuite » disent la même chose : gardez-en un."),
    ("car en effet", "car", "Pléonasme", "« Car » et « en effet » expriment tous deux la cause : gardez-en un."),
    ("mais cependant", "mais", "Pléonasme", "« Mais » et « cependant » expriment tous deux l'opposition : gardez-en un."),
    ("mais pourtant", "mais", "Pléonasme", "« Mais » et « pourtant » expriment tous deux l'opposition : gardez-en un."),
    ("mais néanmoins", "mais", "Pléonasme", "« Mais » et « néanmoins » expriment tous deux l'opposition : gardez-en un."),
    ("panacée universelle", "panacée", "Pléonasme", "Une « panacée » est déjà un remède universel."),
    ("s'entraider mutuellement", "s'entraider", "Pléonasme", "« S'entraider » implique déjà la réciprocité."),
    ("reculer en arrière", "reculer", "Pléonasme", "« Reculer » implique déjà « en arrière »."),
    ("hasard imprévu", "hasard", "Pléonasme", "Un hasard est par définition imprévu."),
    ("s'avérer vrai", "se révéler vrai", "Pléonasme", "« S'avérer » signifie déjà « se révéler vrai » ; « s'avérer vrai » est redondant."),
    ("s'est avéré vrai", "s'est vérifié", "Pléonasme", "« S'avérer » signifie déjà « se révéler vrai »."),
    ("afin de pouvoir", "afin de", "Style : lourdeur", "« Pouvoir » est souvent superflu après « afin de » : « afin de partir »."),
    ("ajouter en plus", "ajouter", "Pléonasme", "« Ajouter » implique déjà « en plus »."),
    ("ajoute en plus", "ajoute", "Pléonasme", "« Ajouter » implique déjà « en plus »."),
    ("se succéder les uns après les autres", "se succéder", "Pléonasme", "« Se succéder » implique déjà « l'un après l'autre »."),
    ("tous sans exception", "tous", "Pléonasme", "« Tous » ne souffre déjà aucune exception."),
    ("préparer à l'avance", "préparer", "Pléonasme", "« Préparer » se fait toujours à l'avance."),
    ("comme même", "quand même", "Expression figée", "On écrit « quand même » (« comme même » est une déformation orale)."),
    ("de soit", "de soi", "Expression figée", "« Cela va de soi » : « soi » est le pronom, pas le verbe « être »."),
    ("soit disant", "soi-disant", "Expression figée", "« Soi-disant » s'écrit avec « soi » et un trait d'union, et reste invariable."),
    ("soit-disant", "soi-disant", "Expression figée", "« Soi-disant » s'écrit avec « soi » (pronom), pas « soit »."),
    ("quelque soit", "quel que soit", "Quel que / quelque", "Devant le verbe « être », on écrit « quel que » en deux mots, accordé avec le sujet."),
    ("quelques soient", "quels que soient", "Quel que / quelque", "Devant le verbe « être », on écrit « quels que » en deux mots."),
    ("en terme de", "en termes de", "Expression figée", "La locution s'écrit au pluriel : « en termes de »."),
    ("au jour d'aujourd'hui", "aujourd'hui", "Pléonasme", "« Aujourd'hui » contient déjà « jour » ; « au jour d'aujourd'hui » est redondant."),
    ("se rappeler de", "se rappeler", "Construction verbale", "« Se rappeler » est transitif direct : on se rappelle quelque chose."),
    ("je me rappelle de", "je me rappelle", "Construction verbale", "« Se rappeler » est transitif direct : je me rappelle quelque chose."),
    ("pallier à", "pallier", "Construction verbale", "« Pallier » est transitif direct : on pallie un manque."),
    ("malgré que", "bien que", "Locution déconseillée", "« Malgré que » est critiqué ; préférez « bien que » ou « quoique »."),
    ("après qu'il soit", "après qu'il est", "Après que + indicatif", "« Après que » introduit un fait accompli : il se construit avec l'indicatif."),
    ("après qu'elle soit", "après qu'elle est", "Après que + indicatif", "« Après que » introduit un fait accompli : il se construit avec l'indicatif."),
    ("si il", "s'il", "Élision", "« Si » s'élide devant « il » et « ils »."),
    ("si ils", "s'ils", "Élision", "« Si » s'élide devant « il » et « ils »."),
    ("sa va", "ça va", "Homophone ça / sa", "« Ça » (cela) est un pronom ; « sa » est un déterminant possessif."),
    ("il à", "il a", "Homophone a / à", "Après un sujet, c'est le verbe « avoir » : « a » sans accent."),
    ("elle à", "elle a", "Homophone a / à", "Après un sujet, c'est le verbe « avoir » : « a » sans accent."),
    ("on à", "on a", "Homophone a / à", "Après un sujet, c'est le verbe « avoir » : « a » sans accent."),
    ("ils on", "ils ont", "Homophone on / ont", "Avec « ils », le verbe avoir s'écrit « ont »."),
    ("elles on", "elles ont", "Homophone on / ont", "Avec « elles », le verbe avoir s'écrit « ont »."),
    ("ils croivent", "ils croient", "Conjugaison", "Le verbe « croire » donne « ils croient »."),
    ("ils voyent", "ils voient", "Conjugaison", "Le verbe « voir » donne « ils voient »."),
    ("vous disez", "vous dites", "Conjugaison", "Le verbe « dire » donne « vous dites »."),
    ("vous faisez", "vous faites", "Conjugaison", "Le verbe « faire » donne « vous faites »."),
    ("c'est de ma faute", "c'est ma faute", "Expression", "La forme soutenue est « c'est ma faute »."),
    ("autant pour moi", "au temps pour moi", "Expression figée", "Forme recommandée par l'Académie : « au temps pour moi »."),
    ("près à", "prêt à", "Homophone près / prêt", "« Prêt à » (disposé à) ; « près de » indique la proximité."),
    ("prêt de", "près de", "Homophone près / prêt", "« Près de » indique la proximité ; « prêt à » signifie disposé à."),
    ("dilemne", "dilemme", "Barbarisme", "On écrit « dilemme » (deux m, pas de n)."),
    ("aréoport", "aéroport", "Barbarisme", "On écrit « aéroport » (aéro-, comme « aérien »)."),
    ("infractus", "infarctus", "Barbarisme", "On écrit « infarctus »."),
    ("rénumération", "rémunération", "Barbarisme", "« Rémunération » vient de « munus » (cadeau), pas de « numéro »."),
    ("rénumérer", "rémunérer", "Barbarisme", "« Rémunérer » vient de « munus », pas de « numéro »."),
    ("language", "langage", "Anglicisme orthographique", "En français, on écrit « langage » (sans u)."),
    ("dévelloppement", "développement", "Orthographe", "« Développement » : un l, deux p."),
    ("apparement", "apparemment", "Orthographe", "Adverbe en -emment formé sur « apparent »."),
    ("évidement", "évidemment", "Orthographe", "Adverbe en -emment formé sur « évident » (« évidement » = action d'évider)."),
    ("courrament", "couramment", "Orthographe", "Adverbe en -amment formé sur « courant »."),
    ("bien sûre", "bien sûr", "Expression figée", "« Bien sûr » est invariable."),
    ("quand à", "quant à", "Homophone quand / quant", "« Quant à » signifie « en ce qui concerne »."),
    ("quand au", "quant au", "Homophone quand / quant", "« Quant au » signifie « en ce qui concerne le »."),
    ("quand aux", "quant aux", "Homophone quand / quant", "« Quant aux » signifie « en ce qui concerne les »."),
    ("d'avantage", "davantage", "Davantage / d'avantage", "« Davantage » (plus) s'écrit en un mot."),
    ("plus pire", "pire", "Pléonasme", "« Pire » est déjà un comparatif : « plus pire » est fautif."),
    ("monter en haut", "monter", "Pléonasme", "« Monter » implique déjà « en haut »."),
    ("descendre en bas", "descendre", "Pléonasme", "« Descendre » implique déjà « en bas »."),
    ("prévoir à l'avance", "prévoir", "Pléonasme", "« Prévoir » signifie déjà « voir à l'avance »."),
    ("collaborer ensemble", "collaborer", "Pléonasme", "« Collaborer » implique déjà « ensemble »."),
    ("comme par exemple", "par exemple", "Pléonasme", "« Comme » et « par exemple » font double emploi."),
    ("voire même", "voire", "Pléonasme", "« Voire » suffit ; « même » est redondant."),
    ("réunir ensemble", "réunir", "Pléonasme", "« Réunir » implique déjà « ensemble »."),
    ("ça m'étonnerait pas", "ça ne m'étonnerait pas", "Négation", "À l'écrit, la négation complète exige « ne … pas »."),
    ("on est pas", "on n'est pas", "Négation", "Le « n' » de la négation disparaît à l'oral après « on » : rétablissez-le à l'écrit."),
    ("on a pas", "on n'a pas", "Négation", "Le « n' » de la négation disparaît à l'oral après « on » : rétablissez-le à l'écrit."),
    ("on avait pas", "on n'avait pas", "Négation", "Le « n' » de la négation disparaît à l'oral après « on » : rétablissez-le à l'écrit."),
    // Homophones point / poing
    ("au poing mort", "au point mort", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « au point mort »."),
    ("mis au poing", "mis au point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « mis au point »."),
    ("mise au poing", "mise au point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « mise au point »."),
    ("mises au poing", "mises au point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « mises au point »."),
    ("mettre au poing", "mettre au point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « mettre au point »."),
    ("met au poing", "met au point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « met au point »."),
    ("mettait au poing", "mettait au point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « mettait au point »."),
    ("mettent au poing", "mettent au point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « mettent au point »."),
    ("sur ce poing", "sur ce point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « sur ce point »."),
    ("à ce poing", "à ce point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « à ce point »."),
    ("sur un poing", "sur un point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « sur un point »."),
    ("poing de vue", "point de vue", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « point de vue »."),
    ("poings de vue", "points de vue", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « points de vue »."),
    ("poing de départ", "point de départ", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « point de départ »."),
    ("poing commun", "point commun", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « point commun »."),
    ("poings communs", "points communs", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « points communs »."),
    ("poing d'interrogation", "point d'interrogation", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « point d'interrogation »."),
    ("poing final", "point final", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « point final »."),
    ("poing culminant", "point culminant", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « point culminant »."),
    ("poing de repère", "point de repère", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « point de repère »."),
    ("poings de repère", "points de repère", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « points de repère »."),
    ("faire le poing", "faire le point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « faire le point »."),
    ("fait le poing", "fait le point", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « fait le point »."),
    ("à poing nommé", "à point nommé", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « à point nommé »."),
    ("au poing de vue", "au point de vue", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « au point de vue »."),
    ("à points fermés", "à poings fermés", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « à poings fermés »."),
    ("pieds et points liés", "pieds et poings liés", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « pieds et poings liés »."),
    ("le point levé", "le poing levé", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « le poing levé »."),
    ("serrer les points", "serrer les poings", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « serrer les poings »."),
    ("serra les points", "serra les poings", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « serra les poings »."),
    ("serrait les points", "serrait les poings", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « serrait les poings »."),
    ("coups de point", "coups de poing", "Homophone : point / poing", "« Point » (endroit, moment, détail, ponctuation) ; « poing » (main fermée). Ici : « coups de poing »."),
];

/// Repère les expressions figées mal formées, barbarismes et confusions fréquentes.
/// N'écrase jamais une alerte déjà posée par une règle plus spécifique.
pub fn verifier_expressions_courantes(para: &str, decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
    let car: Vec<char> = para.chars().collect();
    for &(motif, sugg, regle, exp) in EXPRESSIONS_FAUTIVES {
        for (d, f) in chercher_motif(&car, motif, true) {
            let (a, b) = (decalage + d, decalage + f);
            if alertes.iter().any(|x| x.debut < b && a < x.fin) {
                continue;
            }
            // Ne jamais corriger à l'intérieur d'une zone en écriture chinoise
            if car[d..f].iter().any(|&c| crate::masque::est_cjk(c)) {
                continue;
            }
            // « assurait-elle à » : pronom inversé ; « par la suite à » : locution correcte
            let avant: String = car[d.saturating_sub(3)..d].iter().collect::<String>().to_lowercase();
            if d > 0 && car[d - 1] == '-' || motif == "suite à" && avant == "la " {
                continue;
            }
            let texte_fautif: String = car[d..f].iter().collect();
            alertes.push(AlerteGrammaticale {
                debut: a,
                fin: b,
                suggestions: vec![adapte_casse(&texte_fautif, sugg)],
                texte_fautif,
                regle: regle.to_string(),
                explication: exp.to_string(),
                genre: GenreErreur::Syntaxe,
            });
        }
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

fn estimer_participe_infinitif(inf: &str) -> Option<String> {
    if inf.ends_with("er") && inf.len() > 2 {
        Some(format!("{}é", &inf[..inf.len() - 2]))
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

    #[test]
    fn test_user_exact_screenshot_oct31() {
        let g = CorrecteurGrammaire::new();
        let texte = "Bonjour, je te envoie le document concernent le projet 長城 (Chángchéng).\nLes fournisseur du Shenzhen 深圳 ont confirmer la livraison du projet.\nNous devons faire un réunion pour faire la point sur se problème et raigler ce problème rapidement.";
        let alertes = g.analyse(texte);
        println!("=== ALERTES DU TEST EXACT OCT 31 ===");
        for a in &alertes {
            println!("  [{}] '{}' -> {:?}", a.regle, a.texte_fautif, a.suggestions);
        }

        let fautes: Vec<String> = alertes.iter().map(|a| a.texte_fautif.clone()).collect();
        assert!(fautes.contains(&"te envoie".to_string()), "Élision 'te envoie' manquante");
        assert!(fautes.contains(&"concernent".to_string()), "'concernent' manquant");
        assert!(fautes.contains(&"fournisseur".to_string()), "'fournisseur' manquant");
        assert!(fautes.contains(&"du".to_string()), "'du' devant Shenzhen manquant");
        assert!(fautes.contains(&"confirmer".to_string()), "'confirmer' manquant");
        assert!(fautes.contains(&"un".to_string()), "'un réunion' manquant");
        assert!(fautes.contains(&"la".to_string()), "'la point' manquant");
        assert!(fautes.contains(&"se".to_string()), "'se problème' manquant");
    }


    #[test]
    fn test_benchmark_exhaustif_diagnostique() {
        let g = CorrecteurGrammaire::new();
        let tests = vec![
            ("Les fournisseur du Shenzhen on confirmai les livraison du projet.", vec!["fournisseur", "du", "on", "confirmai", "livraison"]),
            ("Nous devons faire une réunion pour faire le point sur ce problème et régler se problème rapidement.", vec!["se"]),
            ("Malgré que le projet avance, il reste des blocages.", vec!["Malgré que"]),
            ("Après qu'il soit venu, nous avons discuté.", vec!["soit"]),
            ("Où que tu vas pour les vacances ?", vec!["que"]),
            ("Par contre nous devons agir vite.", vec!["Par contre"]),
            ("Quelque soit la décision, nous sommes prêts.", vec!["Quelque soit"]),
            ("Comme par exemple lors de la réunion.", vec!["Comme par exemple"]),
            ("Il a réussi voire même dépassé les objectifs.", vec!["voire même"]),
            ("Il a rencontré un difficulté majeure.", vec!["un"]),
            ("C'est une grand problème pour l'équipe.", vec!["une"]),
            ("Ce arbre est très vieux.", vec!["Ce"]),
            ("Cette accord a été signé hier.", vec!["Cette"]),
            ("Ma amie est venue me voir.", vec!["Ma"]),
            ("Sa entreprise recrute actuellement.", vec!["Sa"]),
            ("Des bal ont été organisés.", vec!["bal"]),
            ("Les chovals courent dans le pré.", vec!["chovals"]),
            ("Toutes les document sont archivés.", vec!["document"]),
            ("Chaque jours nous faisons une vérification.", vec!["jours"]),
            ("Sa va très bien aujourd'hui.", vec!["Sa"]),
            ("Machine a café en panne.", vec!["a"]),
            ("La ville ou il habite est agréable.", vec!["ou"]),
            ("Ces arrivé très vite sans prévenir.", vec!["Ces"]),
            ("Il ces trompé dans son calcul.", vec!["ces"]),
            ("Ils son très contents du résultat.", vec!["son"]),
            ("Je lui donne leurs adresse.", vec!["leurs"]),
            ("Il a mangé un peut de pain.", vec!["peut"]),
            ("Je peu le faire sans problème.", vec!["peu"]),
            ("Il peux venir demain matin.", vec!["peux"]),
            ("Tout les jours nous apprenons.", vec!["Tout"]),
            ("Tous le monde est invité.", vec!["Tous"]),
            ("Quand à moi, je préfère attendre.", vec!["Quand à"]),
            ("Si j'aurais su, je ne serais pas venu.", vec!["aurais"]),
            ("Elle a manger à midi.", vec!["manger"]),
            ("Il faut mangé pour avoir des forces.", vec!["mangé"]),
            ("Sans hésité, il a accepté l'offre.", vec!["hésité"]),
            ("Pour validé ce document, il faut signer.", vec!["validé"]),
            ("La réunion que nous avons organisé hier.", vec!["organisé"]),
            ("Elles se sont parlées pendant une heure.", vec!["parlées"]),
            ("Ils se sont téléphonés hier soir.", vec!["téléphonés"]),
            ("Des vestes marrons.", vec!["marrons"]),
            ("Tu mange trop vite.", vec!["mange"]),
            ("Ils mange ensemble chaque jour.", vec!["mange"]),
            ("Vous faisez fausse route.", vec!["faisez"]),
            ("Vous disez n'importe quoi.", vec!["disez"]),
        ];

        let mut reussis = 0;
        let mut echecs = 0;
        for (phrase, cibles) in &tests {
            let alertes = g.analyse(phrase);
            let alertes_str: Vec<String> = alertes.iter().map(|a| a.texte_fautif.clone()).collect();
            let mut manque = Vec::new();
            for c in cibles {
                if !alertes_str.iter().any(|a| a.eq_ignore_ascii_case(c) || a.to_lowercase().contains(&c.to_lowercase())) {
                    manque.push(*c);
                }
            }
            if manque.is_empty() {
                reussis += 1;
            } else {
                echecs += 1;
                println!("MANQUE dans '{}' : {:?}", phrase, manque);
                println!("  Alertes obtenues : {:?}", alertes_str);
            }
        }
        println!("BILAN DIAGNOSTIQUE : {} réussis, {} avec éléments manquants sur {} tests.", reussis, echecs, tests.len());
        assert_eq!(echecs, 0, "Tous les tests du benchmark doivent réussir !");
    }

    #[test]
    fn test_apprentissage_dynamique_regles() {
        let mut g = CorrecteurGrammaire::new();
        g.banque_regles.regles.clear();
        assert!(g.banque_regles.ajoute_si_absente("au jour d'aujourd'hui", "aujourd'hui", "Pléonasme", "Employez aujourd'hui."));

        let texte = "Nous devons agir au jour d'aujourd'hui pour réussir.";
        let alertes = g.analyse(texte);
        let fautif: Vec<String> = alertes.iter().map(|a| a.texte_fautif.clone()).collect();
        assert!(fautif.contains(&"au jour d'aujourd'hui".to_string()), "Règle apprise non détectée !");
    }


#[cfg(test)]
mod tests_expressions {
    use super::*;

    #[test]
    fn expressions_courantes_detectees() {
        let mut a = Vec::new();
        verifier_expressions_courantes("Quelque soit le prix, ça va de soit et quand au délai…", 0, &mut a);
        let s: Vec<&str> = a.iter().map(|x| x.suggestions[0].as_str()).collect();
        assert!(s.contains(&"Quel que soit"));
        assert!(s.contains(&"de soi"));
        assert!(s.contains(&"quant au"));
    }

    #[test]
    fn phrase_utilisateur_genre() {
        let g = CorrecteurGrammaire::new();
        let a = g.analyse("Les fournisseur ont confirmé la livraison du démarche et régler cet anomalie.");
        let s: Vec<String> = a.iter().flat_map(|x| x.suggestions.clone()).collect();
        assert!(s.iter().any(|x| x == "de la"), "{:?}", s);
        assert!(s.iter().any(|x| x == "cette"), "{:?}", s);
    }
}
}
#[cfg(test)]
mod tests_expressions_livre {
    use super::*;

    fn trouve(t: &str) -> Vec<String> {
        let mut a = Vec::new();
        verifier_expressions_courantes(t, 0, &mut a);
        a.into_iter().flat_map(|x| x.suggestions).collect()
    }

    #[test]
    fn fautes_du_livre_et_pleonasmes() {
        assert_eq!(trouve("patron d’une boutique de prêt sur gage à Suzhou"), vec!["prêt sur gages"]);
        assert_eq!(trouve("Il donne à manger à aux personnes"), vec!["aux"]);
        assert_eq!(trouve("la proposition inconvenable"), vec!["inconvenant"]);
        assert_eq!(trouve("saut dans une abîme"), vec!["un abîme"]);
        assert_eq!(trouve("Nous avons prévu à l'avance la réunion."), vec!["prévu"]);
        assert_eq!(trouve("Suite à votre demande"), vec!["À la suite de"]);
        // Formes justes : rien
        assert!(trouve("Une boutique de prêt sur gages ; à la suite de cela ; un abîme ; du jade précieux.").is_empty());
        assert!(trouve("Il pense à au moins deux solutions.").is_empty());
        assert!(trouve("Elle s'est développée par la suite à Xin'an ; assurait-elle à la fois ce rôle ?").is_empty());
    }
}
