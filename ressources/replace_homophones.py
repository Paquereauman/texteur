new_homophones = r"""    /// Règle 15 : Homophones majeurs & Pièges grammaticaux
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

            // « a » vs « à » & participe après auxiliaire avoir
            if lower == "a" {
                let a_sujet = i > 0 && ["il", "elle", "on", "qui", "celui-ci", "celle-ci"].contains(&tokens[i - 1].mot.to_lowercase().as_str());
                if a_sujet {
                    // Si précédé d'un sujet (ex: "Elle a manger à midi"), 'a' est l'auxiliaire avoir : le verbe suivant doit être au participe passé !
                    if i + 1 < tokens.len() && est_infinitif(&tokens[i + 1].mot.to_lowercase()) {
                        let t_inf = &tokens[i + 1];
                        if let Some(pp) = estimer_participe_infinitif(&t_inf.mot.to_lowercase()) {
                            let sug_casse = adapte_casse(t_inf.mot, &pp);
                            alertes.push(AlerteGrammaticale {
                                debut: decalage + t_inf.deb,
                                fin: decalage + t_inf.fin,
                                texte_fautif: t_inf.mot.to_string(),
                                regle: "Participe passé requis après l'auxiliaire avoir".into(),
                                explication: format!("Après l'auxiliaire « avoir », le verbe s'écrit au participe passé (« {} ») et non à l'infinitif.", sug_casse),
                                suggestions: vec![sug_casse],
                                genre: GenreErreur::Grammaire,
                            });
                            continue;
                        }
                    }
                } else {
                    let mut est_preposition = false;
                    // 1. Noms composés : machine a café, fer a repasser, tasse a café, sac a dos, etc.
                    if i > 0 {
                        let prec = tokens[i - 1].mot.to_lowercase();
                        if ["machine", "fer", "tasse", "sac", "boîte", "boite", "carte", "verre", "salle", "chambre", "brosse", "moulin", "planche", "grâce", "grace", "face", "quant"].contains(&prec.as_str()) {
                            est_preposition = true;
                        }
                    }
                    // 2. Préposition devant infinitif sans sujet immédiat
                    if i + 1 < tokens.len() && est_infinitif(&tokens[i + 1].mot.to_lowercase()) {
                        est_preposition = true;
                    }

                    if est_preposition {
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Confusion a / à".into(),
                            explication: "Il s'agit ici de la préposition « à » (avec accent), et non de la forme conjuguée du verbe avoir.".into(),
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

            // « ces » vs « c'est » / « s'est » (ex: "ces arrivé" -> "c'est arrivé", "il ces trompé" -> "il s'est trompé")
            if lower == "ces" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                let est_participe = (suiv.ends_with('é') || suiv.ends_with("ée") || suiv.ends_with('i') || suiv.ends_with('u') || suiv.ends_with('t'))
                    && [
                        "arrivé", "arrive", "passé", "passe", "déroulé", "deroule", "produit", "avéré", "avere",
                        "terminé", "termine", "fini", "fait", "trompé", "trompe", "arrêté", "arrete", "décidé", "decide",
                        "souvenu", "réuni", "reuni", "installé", "installe", "réveillé", "reveille", "aperçu", "apercu"
                    ].contains(&suiv.as_str());

                if est_participe {
                    let a_sujet_pers = i > 0 && ["il", "elle", "on", "qui", "celui-ci", "celle-ci"].contains(&tokens[i - 1].mot.to_lowercase().as_str());
                    let sug = if a_sujet_pers { "s'est" } else { "c'est" };
                    let sug_casse = adapte_casse(t.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion ces / c'est / s'est".into(),
                        explication: format!(
                            "Devant le participe passé « {} », on écrit « {} » (et non le déterminant démonstratif « ces »).",
                            tokens[i + 1].mot, sug_casse
                        ),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « ou » vs « où » (lieu, temps ou relatif)
            if lower == "ou" {
                let mut est_ou_accent = false;
                if i > 0 {
                    let prec = tokens[i - 1].mot.to_lowercase();
                    if ["ville", "pays", "région", "region", "endroit", "lieu", "maison", "rue", "bâtiment", "batiment", "moment", "jour", "époque", "epoque", "temps", "année", "annee", "date", "heure", "fois", "cas"].contains(&prec.as_str()) {
                        est_ou_accent = true;
                    }
                    if ["d'", "par", "jusqu'"].contains(&prec.as_str()) {
                        est_ou_accent = true;
                    }
                }
                if i + 1 < tokens.len() {
                    let suiv = tokens[i + 1].mot.to_lowercase();
                    if ["sont", "est", "va", "vont", "aller", "trouve", "trouvent"].contains(&suiv.as_str()) {
                        est_ou_accent = true;
                    }
                    // "ou il habite", "ou elle vit"
                    if i > 0 && ["il", "elle", "on", "ils", "elles", "tu", "nous", "vous"].contains(&suiv.as_str()) {
                        est_ou_accent = true;
                    }
                }

                if est_ou_accent {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion ou / où".into(),
                        explication: "Pour marquer le lieu, le temps ou dans une relative, employez « où » avec un accent grave.".into(),
                        suggestions: vec![adapte_casse(t.mot, "où")],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « son » vs « sont » (ex: "ils son très contents")
            if lower == "son" {
                let precede_ils = i > 0 && ["ils", "elles"].contains(&tokens[i - 1].mot.to_lowercase().as_str());
                let suiv_est_adj_ou_adv = if i + 1 < tokens.len() {
                    let s = tokens[i + 1].mot.to_lowercase();
                    ["très", "tres", "trop", "bien", "plus", "moins", "si", "contents", "partis", "arrivés", "arrives", "venus", "nombreux", "nombreuses", "d'accord"].contains(&s.as_str())
                        || ((s.ends_with('s') || s.ends_with('x')) && s.len() > 4 && !self.est_nom_masculin(&s))
                } else { false };

                if precede_ils || suiv_est_adj_ou_adv {
                    let sug_casse = adapte_casse(t.mot, "sont");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion son / sont".into(),
                        explication: "Il s'agit du verbe « être » au pluriel (« sont ») et non du déterminant possessif singulier « son ».".into(),
                        suggestions: vec![sug_casse],
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

                // « leurs » devant nom singulier (« leurs adresse » -> « leur adresse »)
                if !suiv.ends_with('s') && !suiv.ends_with('x') && !est_infinitif(&suiv) {
                    if self.est_nom_masculin(&suiv) || self.est_nom_feminin(&suiv) || ["adresse", "maison", "avis", "projet", "accord", "décision", "nom", "voiture", "famille", "enfant"].contains(&suiv.as_str()) {
                        let sug = adapte_casse(t.mot, "leur");
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t.deb,
                            fin: decalage + t.fin,
                            texte_fautif: t.mot.to_string(),
                            regle: "Accord de leur devant nom singulier".into(),
                            explication: format!("Devant le nom singulier « {} », l'adjectif possessif s'écrit « leur » (sans « s »).", tokens[i + 1].mot),
                            suggestions: vec![sug],
                            genre: GenreErreur::Accord,
                        });
                        continue;
                    }
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

            // « peut » / « peux » vs « peu »
            if (lower == "peut" || lower == "peux") && i > 0 {
                let prec = tokens[i - 1].mot.to_lowercase();
                if ["un", "quelque", "très", "tres", "si", "trop"].contains(&prec.as_str()) {
                    let sug_casse = adapte_casse(t.mot, "peu");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion peu / peut".into(),
                        explication: "L'adverbe de quantité s'écrit « peu » (avec un « u ») et non la forme du verbe pouvoir.".into(),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // Accord verbe pouvoir : je peu -> je peux, il peux -> il peut
            if lower == "peu" || lower == "peut" {
                if i > 0 && ["je", "j'", "tu"].contains(&tokens[i - 1].mot.to_lowercase().as_str()) {
                    let sug_casse = adapte_casse(t.mot, "peux");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Conjugaison verbe pouvoir (je / tu peux)".into(),
                        explication: format!("Avec « {} », le verbe pouvoir se termine par un « x » (« peux »).", tokens[i - 1].mot),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }
            if lower == "peux" || lower == "peu" {
                if i > 0 && ["il", "elle", "on", "qui"].contains(&tokens[i - 1].mot.to_lowercase().as_str()) {
                    let sug_casse = adapte_casse(t.mot, "peut");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Conjugaison verbe pouvoir (il / elle peut)".into(),
                        explication: format!("Avec « {} », le verbe pouvoir se termine par un « t » (« peut »).", tokens[i - 1].mot),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Accord,
                    });
                    continue;
                }
            }

            // « tout les » -> « tous les »
            if lower == "tout" && i + 1 < tokens.len() && tokens[i + 1].mot.to_lowercase() == "les" {
                let sug = if i + 2 < tokens.len() && self.est_nom_feminin(&tokens[i + 2].mot.to_lowercase()) {
                    "toutes"
                } else {
                    "tous"
                };
                let sug_casse = adapte_casse(t.mot, sug);
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t.deb,
                    fin: decalage + t.fin,
                    texte_fautif: t.mot.to_string(),
                    regle: "Accord de tout devant pluriel".into(),
                    explication: format!("Devant l'article pluriel « les », le déterminant s'accorde au pluriel (« {} »).", sug_casse),
                    suggestions: vec![sug_casse],
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

            // « quand à » -> « quant à »
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

            // « sa » vs « ça » (devant verbe)
            if lower == "sa" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if ["va", "marche", "fait", "semble", "donne", "dépend", "depend", "peut", "doit", "serait", "suffit"].contains(&suiv.as_str()) {
                    let sug_casse = adapte_casse(t.mot, "ça");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion sa / ça".into(),
                        explication: "Devant un verbe, écrivez le pronom démonstratif « ça » (avec une cédille).".into(),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }

            // « se » devant nom commun (« se problème » -> « ce problème »)
            if lower == "se" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                let suiv_pur: String = suiv.chars().filter(|c| c.is_alphabetic()).collect();
                if self.est_nom_masculin(&suiv_pur) || self.est_nom_feminin(&suiv_pur) || [
                    "problème", "probleme", "projet", "document", "rapport", "contrat", "réunion", "reunion",
                    "point", "sujet", "dossier", "question", "difficulté", "difficulte", "cas", "accord"
                ].contains(&suiv_pur.as_str()) {
                    let commence_voyelle = suiv_pur.chars().next().map_or(false, |c| {
                        ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                    });
                    let est_fem = self.est_nom_feminin(&suiv_pur);
                    let sug = if est_fem {
                        "cette"
                    } else if commence_voyelle {
                        "cet"
                    } else {
                        "ce"
                    };
                    let sug_casse = adapte_casse(t.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion se / ce devant un nom".into(),
                        explication: format!(
                            "Le pronom réfléchi « se » ne peut pas précéder un nom commun (« {} »). Employez le déterminant démonstratif « {} ».",
                            tokens[i + 1].mot, sug_casse
                        ),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }
        }
    }"""

path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs"
with open(path, "r", encoding="utf-8") as f:
    text = f.read()

import re
pat = r"fn verifier_homophones\(&self, tokens: &\[Token\], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>\) \{[\s\S]*?\n    \}"
text, count = re.subn(pat, new_homophones.strip(), text, count=1)
print(f"Replaced verifier_homophones: {count}")

with open(path, "w", encoding="utf-8") as f:
    f.write(text)
print("Updated grammaire.rs with full verifier_homophones")
