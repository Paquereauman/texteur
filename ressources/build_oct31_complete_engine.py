# -*- coding: utf-8 -*-
with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# 1. Add villes_sans_article to struct
struct_needle = "    couleurs_variables_exceptions: HashSet<&'static str>,\n}"
struct_replace = "    couleurs_variables_exceptions: HashSet<&'static str>,\n    pub villes_sans_article: HashSet<&'static str>,\n}"
if "pub villes_sans_article" not in text:
    if struct_needle in text:
        text = text.replace(struct_needle, struct_replace)

# 2. Add villes to new() and Self
new_decl = "    pub fn new() -> Self {"
cities_code = r'''    pub fn new() -> Self {
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
        ]);'''
if "villes.extend" not in text:
    if new_decl in text:
        text = text.replace(new_decl, cities_code, 1)

self_needle = "            couleurs_variables_exceptions: coul_var,\n        }"
self_replace = "            couleurs_variables_exceptions: coul_var,\n            villes_sans_article: villes,\n        }"
if "villes_sans_article: villes" not in text:
    if self_needle in text:
        text = text.replace(self_needle, self_replace, 1)

# 3. Replace verifier_elisions_obligatoires
old_elision_start = text.find("fn verifier_elisions_obligatoires(")
old_elision_end = text.find("fn verifier_pronoms_relatifs_et_syntaxe(")
if old_elision_start != -1 and old_elision_end != -1:
    new_elision_code = r'''    fn verifier_elisions_obligatoires(
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

'''
    text = text[:old_elision_start] + new_elision_code + text[old_elision_end:]

# 4. Add rules in verifier_homophones loop:
# - du / au + ville
# - concernent -> concernant
homo_loop_start = "for (i, t) in tokens.iter().enumerate() {\n            let lower = t.mot.to_lowercase();"
if "Confusion concernent / concernant" not in text and homo_loop_start in text:
    homo_additions = r'''            // « concernent » -> « concernant » (ou « concerne »)
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
            }'''
    text = text.replace(homo_loop_start, homo_loop_start + "\n" + homo_additions, 1)

# 5. Add test_user_exact_screenshot_oct31 to tests if missing
if "fn test_user_exact_screenshot_oct31" not in text:
    test_oct31_code = r'''
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
'''
    last_brace = text.rfind('}')
    text = text[:last_brace] + test_oct31_code + "\n}\n"

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Oct 31 Complete Engine updated successfully!")
