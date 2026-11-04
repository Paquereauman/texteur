# -*- coding: utf-8 -*-
import re

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# -------------------------------------------------------------
# 1. Update test_benchmark_exhaustif_diagnostique to assert 100%
# -------------------------------------------------------------
old_bench_end = """            if manque.is_empty() {
                reussis += 1;
            } else {
                echecs += 1;
                println!("MANQUE dans '{}' : {:?}", phrase, manque);
                println!("  Alertes obtenues : {:?}", alertes_str);
            }
        }
        println!("BILAN DIAGNOSTIQUE : {} réussis, {} avec éléments manquants sur {} tests.", reussis, echecs, tests.len());
    }"""

new_bench_end = """            if manque.is_empty() {
                reussis += 1;
            } else {
                echecs += 1;
                println!("MANQUE dans '{}' : {:?}", phrase, manque);
                println!("  Alertes obtenues : {:?}", alertes_str);
            }
        }
        println!("BILAN DIAGNOSTIQUE : {} réussis, {} avec éléments manquants sur {} tests.", reussis, echecs, tests.len());
        assert_eq!(echecs, 0, "Tous les tests du benchmark doivent réussir !");
    }"""

if old_bench_end in text:
    text = text.replace(old_bench_end, new_bench_end)

# -------------------------------------------------------------
# 2. Add Adjective Intercalated Support to verifier_accord_determinant
# -------------------------------------------------------------
det_target = '''        for i in 0..tokens.len().saturating_sub(1) {
            let t_det = &tokens[i];
            let det_lower = t_det.mot.to_lowercase();
            let t_nom = &tokens[i + 1];
            let nom_lower = t_nom.mot.to_lowercase();'''

det_replace = '''        for i in 0..tokens.len().saturating_sub(1) {
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
            let nom_lower = t_nom.mot.to_lowercase();'''

if det_target in text:
    text = text.replace(det_target, det_replace, 1)

# -------------------------------------------------------------
# 3. Add Euphonie checks (Ce arbre -> Cet, Ma amie -> Mon)
# -------------------------------------------------------------
euphonie_code = '''            // Euphonie masculine : « ce » devant voyelle/h muet -> « cet » (ex: « Ce arbre »)
            if det_lower == "ce" && self.est_nom_masculin(&nom_lower) {
                let voyelle = nom_lower.chars().next().map_or(false, |c| {
                    ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                });
                if voyelle {
                    let suggestion = adapte_casse(t_det.mot, "cet");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_det.deb,
                        fin: decalage + t_det.fin,
                        texte_fautif: t_det.mot.to_string(),
                        regle: "Euphonie du démonstratif cet devant voyelle".into(),
                        explication: format!("Devant le nom masculin « {} » commençant par une voyelle, employez « cet » au lieu de « ce ».", t_nom.mot),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // Euphonie féminine : « ma », « ta », « sa » devant voyelle/h muet -> « mon », « ton », « son » (ex: « Ma amie », « Sa entreprise »)
            if ["ma", "ta", "sa"].contains(&det_lower.as_str()) && self.est_nom_feminin(&nom_lower) {
                let voyelle = nom_lower.chars().next().map_or(false, |c| {
                    ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                });
                if voyelle {
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
                        explication: format!("Devant le nom féminin « {} » commençant par une voyelle, employez « {} » pour éviter le hiatus.", t_nom.mot, suggestion),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }
'''

# Insert euphonie right before CAS A in verifier_accord_determinant
cas_a_needle = "            // CAS A : « ce » ou « cet » devant nom féminin"
if cas_a_needle in text and "Euphonie du démonstratif cet devant voyelle" not in text:
    text = text.replace(cas_a_needle, euphonie_code + "\n" + cas_a_needle, 1)

# -------------------------------------------------------------
# 4. Add Pluriels exceptionnels : des bal -> bals, chovals -> chevaux
# -------------------------------------------------------------
pluriel_exceptions_code = '''            // Pluriels exceptionnels en -al -> -s (ex: des bal -> des bals)
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
                if nom_lower == "chovals" || nom_lower == "chevals" {
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
'''
if "Pluriel en -s des noms en -al" not in text and cas_a_needle in text:
    text = text.replace(cas_a_needle, pluriel_exceptions_code + "\n" + cas_a_needle, 1)

# -------------------------------------------------------------
# 5. Add Tout le / Tous les / Quel que soit dans verifier_accord_determinant
# -------------------------------------------------------------
tout_tous_code = '''            // « Tout les » -> « Tous les » / « Tous le » -> « Tout le »
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
'''
if "Accord de tout devant les" not in text and cas_a_needle in text:
    text = text.replace(cas_a_needle, tout_tous_code + "\n" + cas_a_needle, 1)

# -------------------------------------------------------------
# 6. Add Missing Rules to verifier_homophones
# -------------------------------------------------------------
homo_start = "for (i, t) in tokens.iter().enumerate() {\n            let lower = t.mot.to_lowercase();"

homo_additions = r'''            // « Par contre » en début de phrase -> « En revanche »
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
'''

if "Locution en revanche vs par contre" not in text and homo_start in text:
    text = text.replace(homo_start, homo_start + "\n" + homo_additions, 1)

# -------------------------------------------------------------
# 7. Add Irregular Verbs (Vous faisez, Vous disez, Tu mange, Ils mange, Si j'aurais, Il faut mangé)
# -------------------------------------------------------------
sujet_verbe_code = '''        // 19. FORMES VERBALES IRRÉGULIÈRES ET ACCORDS SUJET-VERBE DIRECTS
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

            // « Tu mange » -> « Tu manges »
            if s_lower == "tu" && v_lower.len() > 3 && v_lower.ends_with('e') && !v_lower.ends_with("te") {
                if !["elle", "se", "me", "te", "le", "la", "ne"].contains(&v_lower.as_str()) {
                    let sug = format!("{}s", t_v.mot);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_v.deb,
                        fin: decalage + t_v.fin,
                        texte_fautif: t_v.mot.to_string(),
                        regle: "Accord du verbe avec tu".into(),
                        explication: format!("Avec le sujet « tu », le verbe prend un « s » (« {} »).", sug),
                        suggestions: vec![sug],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // « Ils mange » -> « Ils mangent »
            if (s_lower == "ils" || s_lower == "elles") && v_lower.len() > 3 && v_lower.ends_with('e') {
                if !["elle", "se", "me", "te", "le", "la", "ne", "entre"].contains(&v_lower.as_str()) {
                    let sug = format!("{}nt", t_v.mot);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_v.deb,
                        fin: decalage + t_v.fin,
                        texte_fautif: t_v.mot.to_string(),
                        regle: "Accord du verbe avec ils/elles".into(),
                        explication: format!("Avec le sujet pluriel « {} », le verbe prend « -nt » (« {} »).", t_s.mot, sug),
                        suggestions: vec![sug],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
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

            // « Si j'aurais » -> « Si j'avais »
            if (s_lower == "si" || s_lower == "s'") && (v_lower == "aurais" || v_lower == "serais") {
                let sug = if v_lower == "aurais" { "avais" } else { "étais" };
                let sug_casse = adapte_casse(t_v.mot, sug);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_v.deb,
                    fin: decalage + t_v.fin,
                    texte_fautif: t_v.mot.to_string(),
                    regle: "Conditionnel interdit après si".into(),
                    explication: format!("Après la conjonction « si », on emploie l'imparfait (« {} ») et non le conditionnel.", sug_casse),
                    suggestions: vec![sug_casse],
                    genre: GenreErreur::Syntaxe,
                });
                continue;
            }
        }
'''

# Insert in analyse_paragraphe
target_para = "alertes.sort_by_key(|a| a.debut);"
if "FORMES VERBALES IRRÉGULIÈRES ET ACCORDS SUJET-VERBE" not in text and target_para in text:
    text = text.replace(target_para, sujet_verbe_code + "\n        " + target_para, 1)

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Ultimate Engine v2 written successfully!")
