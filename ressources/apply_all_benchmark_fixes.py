import re

path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs"
with open(path, "r", encoding="utf-8") as f:
    text = f.read()

# 1. Add estimer_participe_infinitif if not exists
if "fn estimer_participe_infinitif" not in text:
    target = "fn est_infinitif(mot: &str) -> bool {"
    repl = """fn estimer_participe_infinitif(inf: &str) -> Option<String> {
    if inf.ends_with("er") && inf.len() > 2 {
        Some(format!("{}é", &inf[..inf.len() - 2]))
    } else {
        None
    }
}

fn est_infinitif(mot: &str) -> bool {"""
    text = text.replace(target, repl)

# 2. In verifier_transitivite_et_pleonasmes:
# Add "Par contre" and "Comme par exemple"
pleonasme_anchor = "fn verifier_transitivite_et_pleonasmes(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {"
pleonasme_additions = """fn verifier_transitivite_et_pleonasmes(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        // Connecteur Par contre (déconseillé en style soigné)
        for i in 0..tokens.len().saturating_sub(1) {
            let t1 = &tokens[i];
            let t2 = &tokens[i + 1];
            let w1 = t1.mot.to_lowercase();
            let w2 = t2.mot.to_lowercase();
            if w1 == "par" && w2 == "contre" {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t1.deb,
                    fin: decalage + t2.fin,
                    texte_fautif: format!("{} {}", t1.mot, t2.mot),
                    regle: "Connecteur par contre".into(),
                    explication: "« Par contre » est souvent déconseillé dans un style soigné. Préférez « en revanche » ou « d'un autre côté ».".into(),
                    suggestions: vec![adapte_casse(t1.mot, "en revanche"), adapte_casse(t1.mot, "d'un autre côté")],
                    genre: GenreErreur::Syntaxe,
                });
            }

            // Pléonasme « comme par exemple »
            if w1 == "comme" && w2 == "par" && i + 2 < tokens.len() && tokens[i + 2].mot.to_lowercase() == "exemple" {
                alertes.push(AlerteGrammaticale {
                    debut: decalage + t1.deb,
                    fin: decalage + tokens[i + 2].fin,
                    texte_fautif: format!("{} {} {}", t1.mot, t2.mot, tokens[i + 2].mot),
                    regle: "Pléonasme comme par exemple".into(),
                    explication: "« Comme par exemple » est un pléonasme. Employez soit « comme », soit « par exemple ».".into(),
                    suggestions: vec![adapte_casse(t1.mot, "comme"), adapte_casse(t1.mot, "par exemple")],
                    genre: GenreErreur::Syntaxe,
                });
            }
        }
"""
text = text.replace(pleonasme_anchor, pleonasme_additions, 1)

# 3. In verifier_accord_sujet_verbe:
# Add Vous faisez / Vous disez / Vous refaisez / Vous redisez
# And add modal verb + past participle (-é) -> infinitive (-er)
sujet_verbe_anchor = "fn verifier_accord_sujet_verbe(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {"
sujet_verbe_additions = """fn verifier_accord_sujet_verbe(&self, tokens: &[Token], decalage: usize, alertes: &mut Vec<AlerteGrammaticale>) {
        // Formes verbales irrégulières fréquentes (vous faisez -> vous faites, vous disez -> vous dites)
        for i in 0..tokens.len().saturating_sub(1) {
            let t_suj = &tokens[i];
            let s_lower = t_suj.mot.to_lowercase();
            if s_lower == "vous" {
                let t_v = &tokens[i + 1];
                let v_lower = t_v.mot.to_lowercase();
                let corr = match v_lower.as_str() {
                    "faisez" => Some("faites"),
                    "refaisez" => Some("refaites"),
                    "disez" => Some("dites"),
                    "redisez" => Some("redites"),
                    "croivrent" => Some("croient"),
                    "voyent" => Some("voient"),
                    "prendent" => Some("prennent"),
                    _ => None,
                };
                if let Some(sug) = corr {
                    let sug_casse = adapte_casse(t_v.mot, sug);
                    alertes.push(AlerteGrammaticale {
                        debut: decalage + t_v.deb,
                        fin: decalage + t_v.fin,
                        texte_fautif: t_v.mot.to_string(),
                        regle: "Conjugaison irrégulière".into(),
                        explication: format!("La forme correcte est « {} » (et non « {} »).", sug_casse, t_v.mot),
                        suggestions: vec![sug_casse],
                        genre: GenreErreur::Grammaire,
                    });
                }
            }

            // Verbes modaux suivis d'un participe passé (-é) au lieu de l'infinitif (-er)
            // ex: "il faut mangé" -> "manger"
            if ["faut", "fallait", "faudra", "dois", "doit", "devons", "devez", "doivent", "peux", "peut", "pouvons", "pouvez", "peuvent", "veux", "veut", "voulons", "voulez", "veulent", "va", "vont", "allons", "allez", "sait", "savent"].contains(&s_lower.as_str()) {
                let t_v = &tokens[i + 1];
                let v_lower = t_v.mot.to_lowercase();
                if (v_lower.ends_with('é') || v_lower.ends_with("ée") || v_lower.ends_with("és") || v_lower.ends_with("ées")) && v_lower.len() > 3 {
                    if let Some(inf) = estimer_infinitif_participe(&v_lower) {
                        let sug_casse = adapte_casse(t_v.mot, &inf);
                        alertes.push(AlerteGrammaticale {
                            debut: decalage + t_v.deb,
                            fin: decalage + t_v.fin,
                            texte_fautif: t_v.mot.to_string(),
                            regle: "Infinitif requis après verbe semi-auxiliaire".into(),
                            explication: format!("Après le verbe « {} », le verbe qui suit se met à l'infinitif (« {} »).", t_suj.mot, sug_casse),
                            suggestions: vec![sug_casse],
                            genre: GenreErreur::Infinitif,
                        });
                    }
                }
            }
        }
"""
text = text.replace(sujet_verbe_anchor, sujet_verbe_additions, 1)

with open(path, "w", encoding="utf-8") as f:
    f.write(text)

print("Applied part 1: pleonasme + sujet_verbe")
