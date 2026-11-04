import os
import sys

print("Building advanced detection for se/ce, on/ont, -ai/-e, and du/de villes...")

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "r", encoding="utf-8") as f:
    text = f.read()

# Let's inspect where verifier_homophones is in text
pos_homo = text.find("fn verifier_homophones")
assert pos_homo != -1, "verifier_homophones not found"

# 1. Add known cities to CorrecteurGrammaire struct and new()
cities_hashset = r'''        let mut villes = HashSet::new();
        for v in &[
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
            "montréal", "genève", "geneve", "bruxelles"
        ] {
            villes.insert(*v);
        }'''

# In CorrecteurGrammaire struct:
struct_target = "    pub verbes_etre: HashSet<&'static str>,"
struct_replace = "    pub verbes_etre: HashSet<&'static str>,\n    pub villes_sans_article: HashSet<&'static str>,"
if "pub villes_sans_article" not in text:
    text = text.replace(struct_target, struct_replace)

new_target = "            verbes_etre: etre,\n        }"
new_replace = "            verbes_etre: etre,\n            villes_sans_article: villes,\n        }"
if "villes_sans_article: villes" not in text:
    text = text.replace("Self {", cities_hashset + "\n\n        Self {")
    text = text.replace(new_target, new_replace)

# 2. Add rule for « se » devant nom (ex: « se problème » -> « ce problème »)
se_nom_check = r'''            // « se » vs « ce » (devant un nom) : ex. « se problème » -> « ce problème »
            if lower == "se" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                let est_nom = self.est_nom_masculin(&suiv) || self.est_nom_feminin(&suiv) || [
                    "problème", "probleme", "projet", "document", "dossier", "contrat", "rapport", "matin", "soir", "jour",
                    "client", "type", "cas", "modèle", "modele", "système", "systeme", "rôle", "role", "service", "point",
                    "monde", "temps", "groupe", "texte", "sens", "prix", "fait", "moment", "cours", "sujet", "livraison"
                ].contains(&suiv.as_str());

                let est_verbe = est_infinitif(&suiv) || [
                    "passe", "rend", "souvient", "trompe", "trouve", "font", "sont", "déroule", "deroule", "pose", "met", "situe", "veut", "doit", "peut"
                ].contains(&suiv.as_str());

                if est_nom && !est_verbe {
                    let sug = if self.est_nom_feminin(&suiv) {
                        "cette"
                    } else {
                        let commence_voyelle = suiv.chars().next().map_or(false, |c| {
                            ['a', 'e', 'i', 'o', 'u', 'y', 'é', 'è', 'ê', 'à', 'â', 'î', 'ô', 'û', 'h'].contains(&c)
                        });
                        if commence_voyelle { "cet" } else { "ce" }
                    };
                    let sug_casse = adapte_casse(t.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion se / ce (pronom vs déterminant démonstratif)".into(),
                        explication: format!(
                            "Devant le nom « {} », on emploie le déterminant démonstratif « {} » et non le pronom réfléchi « se ».",
                            tokens[i + 1].mot, sug_casse
                        ),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }'''

# 3. Add rule for « on » vs « ont »
on_ont_check = r'''            // « on » vs « ont » (homophone pronom indéfini vs verbe avoir pluriel)
            if lower == "on" {
                let mut sujet_pluriel = false;
                if i > 0 {
                    let mut p = i - 1;
                    while p > 0 && (tokens[p].mot.chars().all(est_cjk) || ["du", "de", "d'", "dans", "en", "pour", "à", "sur"].contains(&tokens[p].mot.to_lowercase().as_str())) {
                        p -= 1;
                    }
                    let p_mot = tokens[p].mot.to_lowercase();
                    if p_mot == "ils" || p_mot == "elles" || p_mot == "fournisseur" || p_mot == "fournisseurs" || p_mot.ends_with('s') || p_mot.ends_with('x') {
                        if !["nous", "vous", "mais", "dans", "sans", "toujours", "jamais", "parfois", "alors", "après", "apres"].contains(&p_mot.as_str()) {
                            sujet_pluriel = true;
                        }
                    }
                    if p > 0 {
                        let pp_mot = tokens[p - 1].mot.to_lowercase();
                        if self.determinants_pluriel.contains(pp_mot.as_str()) {
                            sujet_pluriel = true;
                        }
                    }
                }

                if sujet_pluriel {
                    let sug = adapte_casse(t.mot, "ont");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion on / ont (pronom vs auxiliaire avoir pluriel)".into(),
                        explication: "Le sujet qui précède est pluriel : employez la forme verbale « ont » (du verbe avoir) et non le pronom indéfini « on » (test : on peut dire « avaient »).".into(),
                        suggestions: vec![sug],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }'''

# 4. Add rule for verb in -ai after on/ont/auxiliary -> -é (ex: on confirmai -> confirmé)
confirmai_check = r'''            // Confusion désinence -ai (passé simple 1re pers) vs participe passé en -é (ex: on confirmai -> confirmé)
            if (lower.ends_with("ai") || lower.ends_with("ais")) && lower.len() > 4 {
                if !["délai", "balai", "essai", "vrai", "relais", "jamais", "palais", "rabais", "marais"].contains(&lower.as_str()) {
                    if i > 0 {
                        let prev_w = tokens[i - 1].mot.to_lowercase();
                        if ["on", "ont", "a", "ai", "as", "avons", "avez", "il", "elle", "ils", "elles"].contains(&prev_w.as_str()) {
                            let racine = if lower.ends_with("ais") {
                                &t.mot[..t.mot.len() - 3]
                            } else {
                                &t.mot[..t.mot.len() - 2]
                            };
                            let suggestion = format!("{}é", racine);
                            alertes.push(AlerteGrammaticale {
                                debut: decalage + t.deb,
                                fin: decalage + t.fin,
                                texte_fautif: t.mot.to_string(),
                                regle: "Participe passé en -é requis (non -ai)".into(),
                                explication: format!(
                                    "Après « {} », le verbe doit être au participe passé (« {} ») et non au passé simple en -ai (qui est réservé au sujet « je »).",
                                    tokens[i - 1].mot, suggestion
                                ),
                                suggestions: vec![suggestion],
                                genre: GenreErreur::Grammaire,
                            });
                            continue;
                        }
                    }
                }
            }'''

# 5. Add rule for « du » + ville -> « de » (ex: « du Shenzhen » -> « de Shenzhen »)
du_ville_check = r'''            // « du » + nom de ville -> « de [ville] » (ex: « du Shenzhen » -> « de Shenzhen »)
            if lower == "du" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if self.villes_sans_article.contains(suiv.as_str()) {
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Préposition de devant nom de ville".into(),
                        explication: format!(
                            "Les noms de ville ne prennent pas d'article contracté : employez la préposition « de » (« de {} ») et non « du ».",
                            tokens[i + 1].mot
                        ),
                        suggestions: vec!["de".into()],
                        genre: GenreErreur::Grammaire,
                    });
                    continue;
                }
            }'''

# 6. Add « sa » vs « ça » (ex: « sa va » -> « ça va », « sa marche » -> « ça marche »)
sa_ca_check = r'''            // « sa » vs « ça » (adjectif possessif vs pronom démonstratif)
            if lower == "sa" && i + 1 < tokens.len() {
                let suiv = tokens[i + 1].mot.to_lowercase();
                if ["va", "marche", "dépend", "depend", "fait", "sera", "serait", "peut", "pourrait", "suffit", "semble", "donne"].contains(&suiv.as_str()) {
                    let sug = adapte_casse(t.mot, "ça");
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t.deb,
                        fin: decalage + t.fin,
                        texte_fautif: t.mot.to_string(),
                        regle: "Confusion sa / ça".into(),
                        explication: format!(
                            "Devant le verbe « {} », employez le pronom démonstratif « {} » (avec une cédille) et non le possessif « sa ».",
                            tokens[i + 1].mot, sug
                        ),
                        suggestions: vec![sug],
                        genre: GenreErreur::Homophone,
                    });
                    continue;
                }
            }'''

# Insert all these checks at the beginning of the loop in verifier_homophones
loop_start = "for (i, t) in tokens.iter().enumerate() {\n            let lower = t.mot.to_lowercase();"
assert loop_start in text, "loop_start not found in verifier_homophones"

combined_checks = f"""{loop_start}

{se_nom_check}

{on_ont_check}

{confirmai_check}

{du_ville_check}

{sa_ca_check}
"""

text = text.replace(loop_start, combined_checks, 1)

# Add a unit test specifically verifying the user's exact screen text
test_ecran_code = r'''
    #[test]
    fn test_ecran_utilisateur_toutes_erreurs_detectees() {
        let g = CorrecteurGrammaire::new();
        let texte = "Les fournisseur du Shenzhen on confirmai les livraison du projet.\net régler se problème rapidement.";
        let alertes = g.analyse(texte);

        // Doit détecter :
        // 1. fournisseur -> fournisseurs
        // 2. du Shenzhen -> de
        // 3. on -> ont
        // 4. confirmai -> confirmé
        // 5. livraison -> livraisons
        // 6. se problème -> ce
        let fautes: Vec<String> = alertes.iter().map(|a| a.texte_fautif.clone()).collect();
        println!("Fautes détectées sur le texte utilisateur: {:?}", fautes);

        assert!(fautes.contains(&"fournisseur".to_string()), "fournisseur manquant");
        assert!(fautes.contains(&"du".to_string()), "du manquant");
        assert!(fautes.contains(&"on".to_string()), "on manquant");
        assert!(fautes.contains(&"confirmai".to_string()), "confirmai manquant");
        assert!(fautes.contains(&"livraison".to_string()), "livraison manquant");
        assert!(fautes.contains(&"se".to_string()), "se manquant");
    }
'''

target_tests = "    #[test]\n    fn test_pleonasmes() {"
assert target_tests in text, "target_tests not found"
text = text.replace(target_tests, test_ecran_code + "\n" + target_tests)

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "w", encoding="utf-8", newline="\r\n") as f:
    f.write(text)

print("Added all advanced detections and unit test to grammaire.rs!")
