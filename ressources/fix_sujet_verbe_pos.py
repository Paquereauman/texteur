# -*- coding: utf-8 -*-
with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Remove the misplaced block from analyse()
misplaced_block_start = text.find("        // 19. FORMES VERBALES IRRÉGULIÈRES ET ACCORDS SUJET-VERBE DIRECTS")
misplaced_block_end = text.find("        alertes.sort_by_key(|a| a.debut);")
if misplaced_block_start != -1 and misplaced_block_end != -1 and misplaced_block_start < misplaced_block_end:
    text = text[:misplaced_block_start] + text[misplaced_block_end:]

sujet_verbe_code = '''
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
    }'''

target_end_para = "    fn verifier_contractions_articles("
idx_contra = text.find(target_end_para)
assert idx_contra != -1, "verifier_contractions_articles not found"

# Find preceding '    }'
idx_brace = text.rfind("    }", 0, idx_contra)
assert idx_brace != -1, "Preceding brace not found"

text = text[:idx_brace] + sujet_verbe_code + "\n\n" + text[idx_contra:]

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Repositioned sujet_verbe_code correctly inside analyse_paragraphe!")
