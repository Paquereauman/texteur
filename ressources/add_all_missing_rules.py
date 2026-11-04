import os
import sys

print("Reading grammaire.rs to add all advanced syntax and agreement rules...")

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "r", encoding="utf-8") as f:
    text = f.read()

# 1. Add new methods to CorrecteurGrammaire:
# - verifier_participe_passe_auxiliaire_etre
# - verifier_confusion_er_e
# - verifier_transitivite_se_rappeler_et_affaire
# - verifier_connecteurs_style (par contre, au final)

new_verifiers_calls = """        // 18. Accord du participe passé avec l'auxiliaire être (Elle est venue, Elles sont parties)
        self.verifier_participe_passe_auxiliaire_etre(&tokens, decalage, alertes);

        // 19. Confusion infinitif en -er vs participe en -é (il a manger -> mangé / il faut mangé -> manger)
        self.verifier_confusion_er_e(&tokens, decalage, alertes);

        // 20. Transitivité se rappeler de & locution avoir affaire à
        self.verifier_transitivite_se_rappeler_et_affaire(&tokens, decalage, alertes);

        // 21. Connecteurs et style soigné (par contre en tête -> en revanche, au final -> finalement)
        self.verifier_connecteurs_style(&tokens, decalage, alertes);"""

target_call = "self.verifier_infinitif_et_prepositions(&tokens, decalage, alertes);"
assert target_call in text, "target_call not found in analyse_paragraphe"
text = text.replace(target_call, target_call + "\n\n" + new_verifiers_calls)

new_methods_code = r'''
    /// Règle 18 : Accord du participe passé avec l'auxiliaire être
    /// « Elle est venu » -> « venue », « Elles sont parti » -> « parties », « Ils sont arrivé » -> « arrivés »
    fn verifier_participe_passe_auxiliaire_etre(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        for i in 0..tokens.len().saturating_sub(2) {
            let t_suj = &tokens[i];
            let s_lower = t_suj.mot.to_lowercase();
            let t_aux = &tokens[i + 1];
            let a_lower = t_aux.mot.to_lowercase();
            let t_part = &tokens[i + 2];
            let p_lower = t_part.mot.to_lowercase();

            let est_sujet_elle = s_lower == "elle";
            let est_sujet_elles = s_lower == "elles";
            let est_sujet_ils = s_lower == "ils";

            if !est_sujet_elle && !est_sujet_elles && !est_sujet_ils {
                continue;
            }

            let est_aux_etre = match a_lower.as_str() {
                "est" | "était" | "etait" | "sera" | "fut" if est_sujet_elle => true,
                "sont" | "étaient" | "etaient" | "seront" | "furent" if (est_sujet_elles || est_sujet_ils) => true,
                _ => false,
            };

            if est_aux_etre {
                let est_fem = est_sujet_elle || est_sujet_elles;
                let est_plur = est_sujet_elles || est_sujet_ils;

                let forme_accordee = accorder_participe_passe(t_part.mot, est_fem, est_plur);

                if !forme_accordee.is_empty() && forme_accordee.to_lowercase() != p_lower {
                    let suggestion = adapte_casse(t_part.mot, &forme_accordee);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_part.deb,
                        fin: decalage + t_part.fin,
                        texte_fautif: t_part.mot.to_string(),
                        regle: "Accord du participe passé avec être".into(),
                        explication: format!(
                            "Avec l'auxiliaire être, le participe passé s'accorde obligatoirement en genre et en nombre avec le sujet « {} » (« {} »).",
                            t_suj.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::ParticipePasse,
                    });
                }
            }
        }
    }

    /// Règle 19 : Confusion classique infinitif en -er vs participe en -é (« mordre / vendu »)
    /// « il a manger » -> « mangé » / « il faut mangé » -> « manger »
    fn verifier_confusion_er_e(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        for i in 0..tokens.len().saturating_sub(1) {
            let t_prev = &tokens[i];
            let prev_lower = t_prev.mot.to_lowercase();
            let t_verbe = &tokens[i + 1];
            let v_lower = t_verbe.mot.to_lowercase();

            // CAS A : Auxiliaire avoir + verbe en -er (au lieu de -é) -> « il a manger » -> « mangé »
            let est_aux_avoir = ["ai", "as", "a", "avons", "avez", "ont", "avais", "avait", "avions", "aviez", "avaient", "aurai", "auras", "aura", "aurons", "aurez", "auront"].contains(&prev_lower.as_str());

            if est_aux_avoir && v_lower.ends_with("er") && v_lower.len() > 3 {
                if !["premier", "dernier", "entier", "léger", "danger", "hiver", "fer"].contains(&v_lower.as_str()) {
                    let racine = &t_verbe.mot[..t_verbe.mot.len() - 2];
                    let suggestion = format!("{}é", racine);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_verbe.deb,
                        fin: decalage + t_verbe.fin,
                        texte_fautif: t_verbe.mot.to_string(),
                        regle: "Participe passé en -é après avoir (test de mordu)".into(),
                        explication: format!(
                            "Après l'auxiliaire avoir (« {} »), on emploie le participe passé (« {} ») et non l'infinitif (test : « il a vendu » et non « il a vendre »).",
                            t_prev.mot, suggestion
                        ),
                        suggestions: vec![suggestion],
                        genre: GenreErreur::Grammaire,
                    });
                    continue;
                }
            }

            // CAS B : Semi-auxiliaire (falloir, devoir, pouvoir, aller, vouloir) + verbe en -é (au lieu de -er) -> « il faut mangé » -> « manger »
            let est_semi_aux = ["faut", "dois", "doit", "devons", "devez", "doivent", "peux", "peut", "pouvons", "pouvez", "peuvent", "veux", "veut", "voulons", "voulez", "veulent", "vais", "vas", "va", "allons", "allez", "vont", "semble", "paraît"].contains(&prev_lower.as_str());

            if est_semi_aux && (v_lower.ends_with('é') || v_lower.ends_with("ée") || v_lower.ends_with("és") || v_lower.ends_with("ées")) && v_lower.len() > 3 {
                let base = v_lower.trim_end_matches(|c| c == 's' || c == 'e' || c == 'é');
                let suggestion = format!("{}er", base);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t_verbe.deb,
                    fin: decalage + t_verbe.fin,
                    texte_fautif: t_verbe.mot.to_string(),
                    regle: "Infinitif en -er après semi-auxiliaire (test de mordre)".into(),
                    explication: format!(
                        "Après le semi-auxiliaire « {} », le verbe se met à l'infinitif (« {} ») et non au participe passé (test : « on doit mordre/vendre »).",
                        t_prev.mot, suggestion
                    ),
                    suggestions: vec![suggestion],
                    genre: GenreErreur::Infinitif,
                });
            }
        }
    }

    /// Règle 20 : Transitivité « se rappeler de » et locution « avoir affaire à »
    fn verifier_transitivite_se_rappeler_et_affaire(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        for i in 0..tokens.len().saturating_sub(1) {
            let t1 = &tokens[i];
            let w1 = t1.mot.to_lowercase();
            let t2 = &tokens[i + 1];
            let w2 = t2.mot.to_lowercase();

            // CAS 1 : « se rappeler de » -> « se rappeler » ou « se souvenir de »
            let est_verbe_rappeler = ["rappelle", "rappelles", "rappelons", "rappelez", "rappellent", "rappeler"].contains(&w1.as_str());
            let est_preposition_de = w2 == "de" || w2 == "d'";

            if est_verbe_rappeler && est_preposition_de {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t1.deb,
                    fin: decalage + t2.fin,
                    texte_fautif: format!("{} {}", t1.mot, t2.mot),
                    regle: "Transitivité de se rappeler (sans de)".into(),
                    explication: "Le verbe « se rappeler » est transitif direct (on se rappelle quelque chose). Avec la préposition « de », employez le verbe « se souvenir » (on se souvient de quelque chose).".into(),
                    suggestions: vec![t1.mot.to_string(), "se souvenir de".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }

            // CAS 2 : « avoir à faire à » -> « avoir affaire à »
            if w1 == "à" || w1 == "a" {
                if w2 == "faire" && i + 2 < tokens.len() && (tokens[i + 2].mot.to_lowercase() == "à" || tokens[i + 2].mot.to_lowercase() == "a") {
                    let fin_span = tokens[i + 2].fin;
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t1.deb,
                        fin: decalage + fin_span,
                        texte_fautif: format!("{} {} {}", t1.mot, t2.mot, tokens[i + 2].mot),
                        regle: "Confusion avoir à faire / avoir affaire".into(),
                        explication: "Dans le sens d'être confronté à quelqu'un ou être en rapport, on écrit « avoir affaire à » (avec le nom affaire).".into(),
                        suggestions: vec!["affaire à".into()],
                        genre: GenreErreur::Homophone,
                    });
                }
            }
        }
    }

    /// Règle 21 : Connecteurs et style soigné (« par contre » en tête -> « en revanche », « au final » -> « finalement »)
    fn verifier_connecteurs_style(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        for i in 0..tokens.len().saturating_sub(1) {
            let t1 = &tokens[i];
            let w1 = t1.mot.to_lowercase();
            let t2 = &tokens[i + 1];
            let w2 = t2.mot.to_lowercase();

            // CAS 1 : « Par contre » en tête de phrase
            if w1 == "par" && w2 == "contre" && (i == 0 || t1.mot.chars().next().map_or(false, |c| c.is_uppercase())) {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t1.deb,
                    fin: decalage + t2.fin,
                    texte_fautif: format!("{} {}", t1.mot, t2.mot),
                    regle: "Registre de par contre en tête de phrase".into(),
                    explication: "En tête de phrase dans un style soigné, préférez « En revanche » ou « D'un autre côté » à « Par contre ».".into(),
                    suggestions: vec!["En revanche".into(), "D'un autre côté".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }

            // CAS 2 : « au final » -> « finalement », « en fin de compte »
            if w1 == "au" && w2 == "final" {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t1.deb,
                    fin: decalage + t2.fin,
                    texte_fautif: format!("{} {}", t1.mot, t2.mot),
                    regle: "Solécisme au final".into(),
                    explication: "La tournure « au final » est un barbarisme moderne. Employez « finalement », « à la fin » ou « en fin de compte ».".into(),
                    suggestions: vec!["finalement".into(), "en fin de compte".into()],
                    genre: GenreErreur::Syntaxe,
                });
            }
        }
    }
'''

# Insert new methods before pub fn est_nom_feminin
target_methods_insert = "    pub fn est_nom_feminin(&self, mot: &str) -> bool {"
assert target_methods_insert in text, "target_methods_insert not found"
text = text.replace(target_methods_insert, new_methods_code + "\n" + target_methods_insert)

# Improve quel que soit in verifier_homophones to check gender
old_quelque_soit = r'''            // « quelque soit » -> « quel que soit »
            if lower == "quelque" && i + 1 < tokens.len() && ["soit", "soient"].contains(&tokens[i + 1].mot.to_lowercase().as_str()) {
                let corr = if tokens[i + 1].mot.to_lowercase() == "soient" { "quels que soient" } else { "quel que soit" };
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + tokens[i + 1].fin,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Confusion quelque soit / quel que soit".into(),
                    explication: "La locution s'écrit en trois mots disjoints (« quel que soit ») et s'accorde avec le nom qui suit.".into(),
                    suggestions: vec![corr.into()],
                    genre: GenreErreur::Grammaire,
                });
                continue;
            }'''

new_quelque_soit = r'''            // « quelque soit » -> « quel que soit » avec accord précis en genre et nombre
            if lower == "quelque" && i + 1 < tokens.len() && ["soit", "soient"].contains(&tokens[i + 1].mot.to_lowercase().as_str()) {
                let est_pluriel = tokens[i + 1].mot.to_lowercase() == "soient";
                let mut est_feminin = false;

                // Inspecter le nom qui suit (i+2 ou i+3)
                if i + 2 < tokens.len() {
                    let nxt = tokens[i + 2].mot.to_lowercase();
                    if nxt == "la" || nxt == "une" || nxt == "cette" || self.est_nom_feminin(&nxt) {
                        est_feminin = true;
                    } else if (nxt == "les" || nxt == "des" || nxt == "ces") && i + 3 < tokens.len() {
                        let nxt_nom = tokens[i + 3].mot.to_lowercase();
                        if self.est_nom_feminin(&nxt_nom) {
                            est_feminin = true;
                        }
                    }
                }

                let corr = match (est_feminin, est_pluriel) {
                    (true, true) => "quelles que soient",
                    (true, false) => "quelle que soit",
                    (false, true) => "quels que soient",
                    (false, false) => "quel que soit",
                };

                let sug_casse = adapte_casse(t.mot, corr);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + tokens[i + 1].fin,
                    texte_fautif: format!("{} {}", t.mot, tokens[i + 1].mot),
                    regle: "Confusion quelque soit / quel que soit".into(),
                    explication: format!("La locution s'écrit en trois mots disjoints et s'accorde avec le sujet (« {} »).", sug_casse),
                    suggestions: vec![sug_casse],
                    genre: GenreErreur::Grammaire,
                });
                continue;
            }'''

assert old_quelque_soit in text, "old_quelque_soit not found"
text = text.replace(old_quelque_soit, new_quelque_soit)

# Add new unit tests to tests module
new_tests_code = r'''
    #[test]
    fn test_participe_passe_avec_etre() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("Elle est venu me voir.");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].suggestions, vec!["venue"]);

        let a2 = g.analyse("Elles sont parti hier soir.");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].suggestions, vec!["parties"]);

        let a3 = g.analyse("Ils sont arrivé en retard.");
        assert_eq!(a3.len(), 1);
        assert_eq!(a3[0].suggestions, vec!["arrivés"]);
    }

    #[test]
    fn test_confusion_er_e() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("Il a manger une pomme.");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].suggestions, vec!["mangé"]);

        let a2 = g.analyse("Il faut mangé pour vivre.");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].suggestions, vec!["manger"]);

        let a3 = g.analyse("Je vais chanté ce soir.");
        assert_eq!(a3.len(), 1);
        assert_eq!(a3[0].suggestions, vec!["chanter"]);
    }

    #[test]
    fn test_se_rappeler_de() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("Je me rappelle de ce dossier.");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].texte_fautif, "rappelle de");
    }

    #[test]
    fn test_avoir_affaire_a() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("Nous avons à faire à des professionnels.");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].suggestions, vec!["affaire à"]);
    }

    #[test]
    fn test_par_contre_et_au_final() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("Par contre nous viendrons demain.");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].suggestions, vec!["En revanche", "D'un autre côté"]);

        let a2 = g.analyse("Au final nous avons gagné.");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].suggestions, vec!["finalement", "en fin de compte"]);
    }

    #[test]
    fn test_quelque_soit_accords() {
        let g = CorrecteurGrammaire::new();
        let a1 = g.analyse("quelque soit la décision prise.");
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].suggestions, vec!["quelle que soit"]);

        let a2 = g.analyse("quelque soient les décisions prises.");
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].suggestions, vec!["quelles que soient"]);
    }
'''

# Insert before end of tests module
target_end_tests = "    #[test]\n    fn test_pleonasmes() {"
assert target_end_tests in text, "target_end_tests not found"
text = text.replace(target_end_tests, new_tests_code + "\n" + target_end_tests)

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "w", encoding="utf-8", newline="\r\n") as f:
    f.write(text)

print("Added all rules and tests to grammaire.rs successfully!")
