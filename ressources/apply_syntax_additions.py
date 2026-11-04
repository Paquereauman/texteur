path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs"
with open(path, "r", encoding="utf-8") as f:
    text = f.read()

# 1. Fix "Quand à" alert to cover both tokens so test passes 100%
old_quand_a = """            // « quand à » -> « quant à »
            if lower == "quand" && i + 1 < tokens.len() && ["à", "a", "au", "aux"].contains(&tokens[i + 1].mot.to_lowercase().as_str()) {
                let sug_casse = adapte_casse(t.mot, "quant");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Locution quant à".into(),
                    explication: "La locution prépositive signifiant « en ce qui concerne » s'écrit « quant à » (avec un « t »).".into(),
                    suggestions: vec![sug_casse],
                    genre: GenreErreur::Homophone,
                });
                continue;
            }"""

new_quand_a = """            // « quand à » -> « quant à »
            if lower == "quand" && i + 1 < tokens.len() && ["à", "a", "au", "aux"].contains(&tokens[i + 1].mot.to_lowercase().as_str()) {
                let sug_casse = adapte_casse(t.mot, "quant");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + tokens[i + 1].fin,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Locution quant à".into(),
                    explication: "La locution prépositive signifiant « en ce qui concerne » s'écrit « quant à » (avec un « t »).".into(),
                    suggestions: vec![format!("{} {}", sug_casse, tokens[i + 1].mot)],
                    genre: GenreErreur::Homophone,
                });
                continue;
            }"""

text = text.replace(old_quand_a, new_quand_a, 1)

# 2. Add CAS 6 & 7 in verifier_pronoms_relatifs_et_syntaxe
syntax_anchor = """            // CAS 5 : Pléonasme lourd « moi personnellement »
            if n1 == "moi" && n2 == "personnellement" {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t1.deb,
                    fin: decalage + t2.fin,
                    texte_fautif: format!("{} {}", t1.mot, t2.mot),
                    regle: "Redondance pronominale".into(),
                    explication: "« Moi personnellement » est un pléonasme lourd. Utilisez simplement « Personnellement » ou « Pour ma part ».".into(),
                    suggestions: vec!["Personnellement".into(), "Pour ma part".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }"""

syntax_addition = """            // CAS 5 : Pléonasme lourd « moi personnellement »
            if n1 == "moi" && n2 == "personnellement" {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t1.deb,
                    fin: decalage + t2.fin,
                    texte_fautif: format!("{} {}", t1.mot, t2.mot),
                    regle: "Redondance pronominale".into(),
                    explication: "« Moi personnellement » est un pléonasme lourd. Utilisez simplement « Personnellement » ou « Pour ma part ».".into(),
                    suggestions: vec!["Personnellement".into(), "Pour ma part".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }

            // CAS 6 : Redondance de préposition avec « dont » (« de cela dont » / « de ça dont »)
            if (n1 == "de" || n1 == "d'") && i + 2 < tokens.len() {
                let w2 = tokens[i + 1].mot.to_lowercase();
                let w3 = tokens[i + 2].mot.to_lowercase();
                if ["cela", "ça", "ceci", "lui", "elle", "eux"].contains(&w2.as_str()) && w3 == "dont" {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t1.deb,
                        fin: decalage + tokens[i + 2].fin,
                        texte_fautif: format!("{} {} {}", t1.mot, tokens[i + 1].mot, tokens[i + 2].mot),
                        regle: "Redondance syntaxique avec dont".into(),
                        explication: "Le pronom « dont » inclut déjà la préposition « de » : ne cumulez pas « de cela dont ».".into(),
                        suggestions: vec!["ce dont".into(), format!("{} {} que", t1.mot, tokens[i + 1].mot)],
                        genre: GenreErreur::Syntaxe,
                    });
                }
            }

            // CAS 7 : Ordre des compléments / pronoms (« a dit à moi » -> « m'a dit »)
            if (n1 == "à" || n1 == "a") && ["moi", "toi", "lui", "nous", "vous", "eux", "elles"].contains(&n2.as_str()) {
                if i > 0 {
                    let v_prec = tokens[i - 1].mot.to_lowercase();
                    if ["dit", "donne", "donné", "parlé", "envoyé", "demandé", "expliqué"].contains(&v_prec.as_str()) {
                        let sug = match n2.as_str() {
                            "moi" => "me",
                            "toi" => "te",
                            "lui" => "lui",
                            "nous" => "nous",
                            "vous" => "vous",
                            _ => "leur",
                        };
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t1.deb,
                            fin: decalage + t2.fin,
                            texte_fautif: format!("{} {}", t1.mot, t2.mot),
                            regle: "Placement du pronom complément conjoint".into(),
                            explication: format!("En français, le pronom personnel conjoint se place avant le verbe (« {} ... »).", sug),
                            suggestions: vec![format!("{} ...", sug)],
                            genre: GenreErreur::Syntaxe,
                        });
                    }
                }
            }"""

text = text.replace(syntax_anchor, syntax_addition, 1)

with open(path, "w", encoding="utf-8") as f:
    f.write(text)

print("Applied syntax additions and Quand à fix")
