import re

path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs"
with open(path, "r", encoding="utf-8") as f:
    text = f.read()

# 1. Upgrade verifier_accord_determinant to support intervening adjectives
old_accord_det_block = """            // CAS 3 : Déterminant masculin devant nom féminin (« ce difficulté » -> « cette difficulté », « un réunion » -> « une »)
            if ["ce", "un"].contains(&det_lower.as_str()) && self.est_nom_feminin(&nom_lower) {
                let sug = if det_lower == "ce" { "cette" } else { "une" };
                let sug_casse = adapte_casse(t_det.mot, sug);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_det.fin,
                    texte_fautif: t_det.mot.to_string(),
                    regle: "Accord en genre du déterminant (féminin requis)".into(),
                    explication: format!(
                        "Le nom « {} » est féminin : son déterminant doit être au féminin (« {} »).",
                        t_nom.mot, sug_casse
                    ),
                    suggestions: vec![sug_casse],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // CAS 4 : Déterminant féminin devant nom masculin (« cette problème » -> « ce problème », « cette accord » -> « cet accord »)
            if ["cette", "une"].contains(&det_lower.as_str()) && self.est_nom_masculin(&nom_lower) {
                let sug = if det_lower == "cette" {
                    let commence_voyelle = nom_lower.chars().next().map_or(false, |c| {
                        ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                    });
                    if commence_voyelle { "cet" } else { "ce" }
                } else {
                    "un"
                };
                let sug_casse = adapte_casse(t_det.mot, sug);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_det.fin,
                    texte_fautif: t_det.mot.to_string(),
                    regle: "Accord en genre du déterminant (masculin requis)".into(),
                    explication: format!(
                        "Le nom « {} » est masculin : son déterminant doit être au masculin (« {} »).",
                        t_nom.mot, sug_casse
                    ),
                    suggestions: vec![sug_casse],
                    genre: GenreErreur::Accord,
                });
                continue;
            }"""

new_accord_det_block = """            // Détection du nom tête (immédiat ou après adjectif intercalé ex: « une grand problème »)
            let (t_cible_nom, nom_cible_lower) = if self.est_nom_feminin(&nom_lower) || self.est_nom_masculin(&nom_lower) {
                (t_nom, nom_lower.clone())
            } else if i + 2 < tokens.len() {
                let w2 = tokens[i + 2].mot.to_lowercase();
                if self.est_nom_feminin(&w2) || self.est_nom_masculin(&w2) {
                    (&tokens[i + 2], w2)
                } else {
                    (t_nom, nom_lower.clone())
                }
            } else {
                (t_nom, nom_lower.clone())
            };

            // CAS 3 : Déterminant masculin devant nom féminin (« ce difficulté » -> « cette difficulté », « un réunion » -> « une »)
            if ["ce", "un"].contains(&det_lower.as_str()) && self.est_nom_feminin(&nom_cible_lower) {
                let sug = if det_lower == "ce" { "cette" } else { "une" };
                let sug_casse = adapte_casse(t_det.mot, sug);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_det.fin,
                    texte_fautif: t_det.mot.to_string(),
                    regle: "Accord en genre du déterminant (féminin requis)".into(),
                    explication: format!(
                        "Le nom « {} » est féminin : son déterminant doit être au féminin (« {} »).",
                        t_cible_nom.mot, sug_casse
                    ),
                    suggestions: vec![sug_casse],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // CAS 4 : Déterminant féminin devant nom masculin (« cette problème » -> « ce problème », « une grand problème » -> « un »)
            if ["cette", "une"].contains(&det_lower.as_str()) && self.est_nom_masculin(&nom_cible_lower) {
                let sug = if det_lower == "cette" {
                    let commence_voyelle = nom_cible_lower.chars().next().map_or(false, |c| {
                        ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                    });
                    if commence_voyelle { "cet" } else { "ce" }
                } else {
                    "un"
                };
                let sug_casse = adapte_casse(t_det.mot, sug);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_det.deb,
                    fin: decalage + t_det.fin,
                    texte_fautif: t_det.mot.to_string(),
                    regle: "Accord en genre du déterminant (masculin requis)".into(),
                    explication: format!(
                        "Le nom « {} » est masculin : son déterminant doit être au masculin (« {} »).",
                        t_cible_nom.mot, sug_casse
                    ),
                    suggestions: vec![sug_casse],
                    genre: GenreErreur::Accord,
                });
                continue;
            }"""

if old_accord_det_block in text:
    text = text.replace(old_accord_det_block, new_accord_det_block, 1)
    print("Upgraded accord determinant with intervening adjectives")
else:
    print("WARNING: old_accord_det_block not found exactly, attempting normalized replace")

with open(path, "w", encoding="utf-8") as f:
    f.write(text)
