# -*- coding: utf-8 -*-
with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Replace CAS C and CAS D in verifier_accord_determinant
det_loop = '''        for i in 0..tokens.len().saturating_sub(1) {
            let t_det = &tokens[i];
            let det_lower = t_det.mot.to_lowercase();
            let t_nom = &tokens[i + 1];
            let nom_lower = t_nom.mot.to_lowercase();'''

new_det_body = '''        for i in 0..tokens.len().saturating_sub(1) {
            let t_det = &tokens[i];
            let det_lower = t_det.mot.to_lowercase();
            let t_nom = &tokens[i + 1];
            let nom_lower = t_nom.mot.to_lowercase();

            // Inversion de pronoms conjoints (ex: « le vous envoie » -> « vous l\'envoie »)
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
            }'''

assert det_loop in text, 'det_loop not found'
text = text.replace(det_loop, new_det_body, 1)

# Now replace CAS C and CAS D
cas_cd_old = '''            // CAS C : « un » devant nom féminin
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
            }'''

cas_cd_new = '''            // CAS C : Déterminant masculin singulier devant nom féminin (ex: « un réunion », « le solution », « ce difficulté »)
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
            }'''

assert cas_cd_old in text, 'cas_cd_old not found'
text = text.replace(cas_cd_old, cas_cd_new, 1)

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print('Updated verifier_accord_determinant with pronoun and full gender agreement!')
