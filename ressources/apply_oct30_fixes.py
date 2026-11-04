import re

path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs"
with open(path, "r", encoding="utf-8") as f:
    text = f.read()

# 1. Update verifier_accord_determinant to:
#    a) Handle clitic pronoun pairs (le vous, la vous, les vous, le nous) and avoid treating "vous" as noun
#    b) Expand feminine determiners in CAS 4 to ["cette", "une", "la", "ma", "ta", "sa"] for "sa point", "ma problème", etc.

det_start_pat = r"fn verifier_accord_determinant\(&self, tokens: &\[Token\], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>\) \{\s+for i in 0\.\.tokens\.len\(\)\.saturating_sub\(1\) \{\s+let t_det = &tokens\[i\];\s+let det_lower = t_det\.mot\.to_lowercase\(\);\s+let t_nom = &tokens\[i \+ 1\];\s+let nom_lower = t_nom\.mot\.to_lowercase\(\);"

det_replacement = """fn verifier_accord_determinant(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        for i in 0..tokens.len().saturating_sub(1) {
            let t_det = &tokens[i];
            let det_lower = t_det.mot.to_lowercase();
            let t_nom = &tokens[i + 1];
            let nom_lower = t_nom.mot.to_lowercase();

            // Si le mot suivant est un pronom personnel (ex: "le vous", "le lui", "la leur", "les nous")
            if ["le", "la", "les", "l'"].contains(&det_lower.as_str()) && ["me", "te", "se", "nous", "vous", "lui", "leur", "y", "en"].contains(&nom_lower.as_str()) {
                // Inversion fautive de pronoms conjoints : me/te/se/nous/vous doit précéder le/la/les
                if ["me", "te", "se", "nous", "vous"].contains(&nom_lower.as_str()) {
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
                }
                // Ce n'est pas un nom, ne pas vérifier les règles d'accord sur ce token
                continue;
            }"""

text, count_det = re.subn(det_start_pat, det_replacement, text, count=1)
print(f"Replaced det start (pronoun pair handling): {count_det}")

# Update CAS 4 in verifier_accord_determinant
cas4_old = r'// CAS 4 : Déterminant féminin devant nom masculin[^\n]*\n\s+if \["cette", "une"\]\.contains\(&det_lower\.as_str\(\)\) && self\.est_nom_masculin\(&nom_cible_lower\) \{[\s\S]*?continue;\s+\}'

cas4_new = """// CAS 4 : Déterminant féminin devant nom masculin (« cette problème » -> « ce problème », « sa point » -> « son point »)
            if ["cette", "une", "la", "ma", "ta", "sa"].contains(&det_lower.as_str()) && self.est_nom_masculin(&nom_cible_lower) {
                let sug = match det_lower.as_str() {
                    "cette" => {
                        let commence_voyelle = nom_cible_lower.chars().next().map_or(false, |c| {
                            ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                        });
                        if commence_voyelle { "cet" } else { "ce" }
                    }
                    "une" => "un",
                    "la" => {
                        let commence_voyelle = nom_cible_lower.chars().next().map_or(false, |c| {
                            ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                        });
                        if commence_voyelle { "l'" } else { "le" }
                    }
                    "ma" => "mon",
                    "ta" => "ton",
                    "sa" => {
                        if i > 0 && tokens[i - 1].mot.to_lowercase() == "faire" && nom_cible_lower == "point" {
                            "le"
                        } else {
                            "son"
                        }
                    }
                    _ => "le",
                };
                let mut sugs = vec![adapte_casse(t_det.mot, sug)];
                if det_lower == "sa" && nom_cible_lower == "point" {
                    sugs.push(adapte_casse(t_det.mot, "son"));
                }
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_det.fin,
                    texte_fautif: t_det.mot.to_string(),
                    regle: "Accord en genre du déterminant (masculin requis)".into(),
                    explication: format!(
                        "Le nom « {} » est masculin : son déterminant doit être au masculin (« {} »).",
                        t_cible_nom.mot, sugs.join(" ou ")
                    ),
                    suggestions: sugs,
                    genre: GenreErreur::Accord,
                });
                continue;
            }"""

text, count_cas4 = re.subn(cas4_old, cas4_new, text, count=1)
print(f"Replaced CAS 4 (sa point, etc.): {count_cas4}")

# 2. In verifier_accord_sujet_verbe:
#    Add:
#    a) Auxiliaire (avoir ou être: ont, a, ai, as, avons, avez, est, sont...) + verbe à l'infinitif en -er (ont confirmer -> ont confirmé)
#    b) Accord sujet "je" / "j'" avec verbe au présent (je vous envoieS -> je vous envoie)

sujet_verbe_head = "fn verifier_accord_sujet_verbe(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {"

sujet_verbe_head_replacement = """fn verifier_accord_sujet_verbe(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        // Auxiliaire avoir ou être suivi d'un infinitif en -er (ex: "ont confirmer" -> "ont confirmé", "a manger" -> "a mangé")
        for i in 0..tokens.len().saturating_sub(1) {
            let t_aux = &tokens[i];
            let aux_lower = t_aux.mot.to_lowercase();
            let est_aux = ["ai", "as", "a", "avons", "avez", "ont", "avait", "avaient", "auront", "aurait", "auraient", "aie", "ait", "ayons", "ayez", "aient",
                           "suis", "es", "est", "sommes", "êtes", "etes", "sont", "étais", "etais", "était", "etait", "étions", "etions", "étiez", "etiez", "étaient", "etaient", "serai", "seras", "sera", "serons", "serez", "seront", "serait", "seraient", "sois", "soit", "soyons", "soyez", "soient"].contains(&aux_lower.as_str());

            if est_aux {
                let mut v_idx = i + 1;
                // Sauter un éventuel adverbe court (ex: "ont déjà confirmer", "ont souvent manger")
                if v_idx + 1 < tokens.len() && ["déjà", "deja", "souvent", "toujours", "bien", "vite", "tard", "enfin", "clairement"].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                    v_idx += 1;
                }
                if v_idx < tokens.len() {
                    let t_v = &tokens[v_idx];
                    let v_lower = t_v.mot.to_lowercase();
                    if est_infinitif(&v_lower) && v_lower.ends_with("er") && v_lower.len() > 3 {
                        if let Some(pp) = estimer_participe_infinitif(&v_lower) {
                            let sug_casse = adapte_casse(t_v.mot, &pp);
                            alertes.push(AlerteGrammaticale {
                                debut: decalage + t_v.deb,
                                fin: decalage + t_v.fin,
                                texte_fautif: t_v.mot.to_string(),
                                regle: "Participe passé après auxiliaire".into(),
                                explication: format!("Après l'auxiliaire « {} », le verbe se met au participe passé (« {} ») et non à l'infinitif.", t_aux.mot, sug_casse),
                                suggestions: vec![sug_casse],
                                genre: GenreErreur::Grammaire,
                            });
                        }
                    }
                }
            }

            // Accord sujet-verbe avec « je » / « j' » (ex: "je LE vous envoieS" -> "envoie")
            let s_lower = t_aux.mot.to_lowercase();
            if s_lower == "je" || s_lower == "j'" {
                let mut v_idx = i + 1;
                while v_idx < tokens.len() && ["le", "la", "les", "l'", "lui", "leur", "nous", "vous", "me", "te", "se", "y", "en", "ne", "n'"].contains(&tokens[v_idx].mot.to_lowercase().as_str()) {
                    v_idx += 1;
                }
                if v_idx < tokens.len() {
                    let t_v = &tokens[v_idx];
                    let v_lower = t_v.mot.to_lowercase();
                    let verbes_excep_s = ["suis", "sais", "vois", "prends", "mets", "viens", "tiens", "pars", "fais", "dis", "peux", "veux", "vaux", "dois"];
                    if v_lower.ends_with('s') && v_lower.len() > 3 && !verbes_excep_s.contains(&v_lower.as_str()) {
                        let rad = if v_lower.ends_with("es") {
                            &v_lower[..v_lower.len() - 1] // enlève le 's'
                        } else if t_v.mot.ends_with('S') {
                            &v_lower[..v_lower.len() - 1]
                        } else {
                            ""
                        };
                        if !rad.is_empty() {
                            let sug_casse = adapte_casse(t_v.mot, rad);
                            alertes.push(AlerteGrammaticale {
                                debut: decalage + t_v.deb,
                                fin: decalage + t_v.fin,
                                texte_fautif: t_v.mot.to_string(),
                                regle: "Accord sujet-verbe avec je".into(),
                                explication: format!("Avec le sujet « {} », le verbe du premier groupe se termine par un « e » (sans « s ») : « {} ».", t_aux.mot, sug_casse),
                                suggestions: vec![sug_casse],
                                genre: GenreErreur::Accord,
                            });
                        }
                    }
                }
            }
        }
"""

text = text.replace(sujet_verbe_head, sujet_verbe_head_replacement, 1)

with open(path, "w", encoding="utf-8") as f:
    f.write(text)

print("Applied oct30 fixes successfully")
