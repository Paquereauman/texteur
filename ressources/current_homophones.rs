fn verifier_homophones(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        for (i, t) in tokens.iter().enumerate() {
            let lower = t.mot.to_lowercase();

            // « censé » vs « sensé »
            if (lower == "sensé" || lower == "sensée" || lower == "sensés" || lower == "sensées") && i + 1 < tokens.len() {
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
            }

            // « ce » vs « se » (devant verbe pronominal)
            if lower == "ce" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if est_infinitif(&suiv) || [
                    "passer", "rendre", "souvenir", "tromper", "concentrer", "trouver", "faire", "voir",
                    "passe", "rend", "souvient", "trompe", "trouve", "font", "sont", "déroule", "deroule"
                ].contains(&suiv.as_str()) {
                    if i > 0 && ["ils", "elles", "on", "qui", "pour", "sans", "de", "d'"].contains(&tokens[i - 1].mot.to_lowercase().as_str()) {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion ce / se".into(),
                            explication: "Devant un verbe, on écrit le pronom réfléchi « se » (et non le démonstratif « ce »).".into(),
                            suggestions: vec![adapte_casse(t.mot, "se")],
                            genre: GenreErreur::Homophone,
                        });
                        continue;
                    }
                }
            }

            // « c'est » vs « s'est »
            if (lower == "c'est" || lower == "c’est") && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if ["passé", "produit", "déroulé", "avéré", "terminé", "passe", "deroule"].contains(&suiv.as_str()) {
                    let sug = adapte_casse(t.mot, "s'est");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion c'est / s'est".into(),
                        explication: format!(
                            "Avec le participe « {} », il s'agit du verbe pronominal « s'est {} » (et non « c'est »).",
                            tokens[i + 1].mot, tokens[i + 1].mot
                        ),
                        suggestions: vec![sug],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « ou » vs « où »
            if lower == "ou" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if ["sont", "est", "va", "vont", "aller", "trouve", "trouvent"].contains(&suiv.as_str()) {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion ou / où".into(),
                        explication: "Pour marquer le lieu ou l'interrogation, employez « où » avec un accent.".into(),
                        suggestions: vec![adapte_casse(t.mot, "où")],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « leur » vs « leurs » : invariable devant verbe, accordé devant nom pluriel
            if lower == "leurs" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                let est_verbe = est_infinitif(&suiv) || [
                    "disent", "donnent", "parlent", "expliquent", "ont", "avaient", "auront", "font", "demandent"
                ].contains(&suiv.as_str());

                if est_verbe {
                    let sug = adapte_casse(t.mot, "leur");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Invariabilité de leur devant un verbe".into(),
                        explication: "Devant un verbe, « leur » est pronom personnel COI et est toujours invariable (jamais de « s »).".into(),
                        suggestions: vec![sug],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « leur » singulier devant nom pluriel
            if lower == "leur" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if (suiv.ends_with('s') || suiv.ends_with('x')) && suiv.len() > 3 {
                    if !["fois", "temps", "corps", "cours", "cas", "prix", "choix", "mois"].contains(&suiv.as_str()) && !est_infinitif(&suiv) {
                        let sug = adapte_casse(t.mot, "leurs");
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Accord du possessif leur(s)".into(),
                            explication: format!(
                                "Devant le nom pluriel « {} », l'adjectif possessif prend un « s » (« leurs »).",
                                tokens[i + 1].mot
                            ),
                            suggestions: vec![sug],
                            genre: GenreErreur::Accord,
                        });
                        continue;
                    }
                }
            }

            // « quelque soit » -> « quel que soit »
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
            }

            // « sans doutes » -> « sans doute »
            if lower == "doutes" && i > 0 && tokens[i - 1].mot.to_lowercase() == "sans" {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Locution sans doute".into(),
                    explication: "Dans la locution « sans doute », « doute » reste au singulier.".into(),
                    suggestions: vec!["doute".into()],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // « tous le » -> « tout le »
            if lower == "tous" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "le" {
                let sug = adapte_casse(t.mot, "tout");
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Confusion tous le / tout le".into(),
                    explication: "Devant le masculin singulier « le », écrivez « tout » (avec un « t »).".into(),
                    suggestions: vec![sug],
                    genre: GenreErreur::Accord,
                });
                continue;
            }

            // « je vous envoit » -> « envoie »
            if lower == "envoit" && i > 0 && tokens[i - 1].mot.to_lowercase() == "vous" {
                if i >= 2 && tokens[i - 2].mot.to_lowercase() == "je" {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Conjugaison verbe envoyer".into(),
                        explication: "Avec le sujet « je », le verbe envoyer s'écrit « envoie » (au présent de l'indicatif).".into(),
                        suggestions: vec!["envoie".into()],
                        genre: GenreErreur::Grammaire,
                    });
                    continue;
                }
            }
        }
    }