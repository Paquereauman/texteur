//! Règles d'édition : chaque maison d'édition (ou revue, collection) a ses conventions maison
//! (« c.-à-d. » plutôt que « c'est-à-dire », siècles en chiffres romains…). L'utilisateur crée les éditions
//! et leurs règles ; celles de l'édition choisie sont signalées comme des fautes.
//! Fichier : %APPDATA%\Texteur\editions.json

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::grammaire::{AlerteGrammaticale, GenreErreur, adapte_casse, chercher_motif};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct RegleEdition {
    /// Ce que l'édition refuse (« c'est-à-dire »).
    pub motif: String,
    /// Ce qu'elle veut à la place (« c.-à-d. ») ; vide : à retirer ou à reformuler.
    pub remplacement: String,
    /// Pourquoi, tel que l'affiche la fiche de la faute.
    pub explication: String,
    /// Le motif doit être un mot entier (« Mr » ne doit pas toucher « Mrs »).
    pub mot_entier: bool,
    /// Respecter les majuscules (« Première guerre mondiale » faux, « Première Guerre mondiale » juste).
    pub casse: bool,
    pub active: bool,
}

impl Default for RegleEdition {
    fn default() -> Self {
        RegleEdition { motif: String::new(), remplacement: String::new(), explication: String::new(), mot_entier: true, casse: false, active: true }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Edition {
    pub nom: String,
    pub regles: Vec<RegleEdition>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Editions {
    pub liste: Vec<Edition>,
    /// Nom de l'édition appliquée au texte ; vide : aucune.
    pub active: String,
}

impl Default for Editions {
    fn default() -> Self {
        let r = |motif: &str, remplacement: &str, explication: &str| RegleEdition {
            motif: motif.into(),
            remplacement: remplacement.into(),
            explication: explication.into(),
            ..Default::default()
        };
        Editions {
            liste: vec![peter_lang(), Edition {
                nom: "Exemple (à adapter)".into(),
                regles: vec![
                    r("c'est-à-dire", "c.-à-d.", "Abréviation demandée par l'éditeur."),
                    r("Mr", "M.", "En français, « Monsieur » s'abrège « M. »."),
                    r("etc...", "etc.", "« etc. » n'est jamais suivi de points de suspension."),
                    r("XXème", "XXe", "Les ordinaux s'abrègent « e » (XXe siècle)."),
                ],
            }],
            active: "Peter Lang".into(),
        }
    }
}

/// Guide stylistique français de Peter Lang : les règles qu'un correcteur peut vérifier mot à mot.
pub fn peter_lang() -> Edition {
    let r = |motif: &str, remplacement: &str, explication: &str, casse: bool| RegleEdition {
        motif: motif.into(),
        remplacement: remplacement.into(),
        explication: format!("Peter Lang : {explication}"),
        casse,
        ..Default::default()
    };
    let maj = "les majuscules sont accentuées.";
    let guerre = "seul le nom principal et les adjectifs qui le précèdent prennent la majuscule (Première Guerre mondiale).";
    let lieu = "pas de majuscule pour situer un lieu (le nord de la France, l'ouest de la Belgique).";
    let annees = "les années s'écrivent en entier (années 1950, et non années 50).";
    let etc = "une énumération inachevée se termine par « etc. », jamais par « … ».";
    let mut v = vec![
        r("Etat", "État", maj, true),
        r("Etats", "États", maj, true),
        r("Etats-Unis", "États-Unis", maj, true),
        r("Eglise", "Église", maj, true),
        r("Ecole", "École", maj, true),
        r("Etant donné", "Étant donné", maj, true),
        r("Egypte", "Égypte", maj, true),
        r("Equateur", "Équateur", maj, true),
        r("Ile", "Île", maj, true),
        r("Première guerre mondiale", "Première Guerre mondiale", guerre, true),
        r("première guerre mondiale", "Première Guerre mondiale", guerre, true),
        r("Première Guerre Mondiale", "Première Guerre mondiale", guerre, true),
        r("Seconde guerre mondiale", "Seconde Guerre mondiale", guerre, true),
        r("seconde guerre mondiale", "Seconde Guerre mondiale", guerre, true),
        r("Seconde Guerre Mondiale", "Seconde Guerre mondiale", guerre, true),
        r("Deuxième guerre mondiale", "Deuxième Guerre mondiale", guerre, true),
        r("deuxième guerre mondiale", "Deuxième Guerre mondiale", guerre, true),
        r("Organisation Mondiale de la Santé", "Organisation mondiale de la santé",
          "seul le premier mot du développé d'un sigle prend la majuscule.", true),
        r("Nord de la", "nord de la", lieu, true),
        r("Sud de la", "sud de la", lieu, true),
        r("Est de la", "est de la", lieu, true),
        r("Ouest de la", "ouest de la", lieu, true),
        r("etc...", "etc.", etc, false),
        r("etc…", "etc.", etc, false),
        r("op.cit.", "op. cit.", "abréviation avec un point et une espace.", false),
        r("\"", "« »", "citations entre guillemets français « », avec espaces insécables ; les guillemets anglais ne servent qu'à l'intérieur d'une citation.", false),
    ];
    for d in ["20", "30", "40", "50", "60", "70", "80", "90"] {
        v.push(r(&format!("années {d}"), &format!("années 19{d}"), annees, false));
    }
    let mut tiret = r(" - ", " – ", "le tiret moyen (–), entouré d'espaces, sert de parenthèse ; le trait d'union (-) ne sert qu'aux mots composés.", false);
    tiret.mot_entier = false;
    v.push(tiret);
    Edition { nom: "Peter Lang".into(), regles: v }
}

/// Recherche sensible à la casse (seule l'apostrophe ’/' est assimilée), en indices de caractères.
fn chercher_exact(car: &[char], motif: &str, mot_entier: bool) -> Vec<(usize, usize)> {
    let n = |c: char| if c == '’' { '\'' } else { c };
    let m: Vec<char> = motif.chars().map(n).collect();
    let mut res = Vec::new();
    let mut i = 0;
    while !m.is_empty() && i + m.len() <= car.len() {
        let f = i + m.len();
        if (0..m.len()).all(|k| n(car[i + k]) == m[k])
            && (!mot_entier || !(i > 0 && car[i - 1].is_alphanumeric()) && !car.get(f).is_some_and(|c| c.is_alphanumeric()))
        {
            res.push((i, f));
            i = f;
        } else {
            i += 1;
        }
    }
    res
}

fn chemin() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("Texteur").join("editions.json")
}

impl Editions {
    pub fn charge() -> Self {
        std::fs::read_to_string(chemin()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    pub fn sauve(&self) -> Result<(), String> {
        let f = chemin();
        if let Some(p) = f.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        std::fs::write(&f, serde_json::to_string_pretty(self).unwrap()).map_err(|e| e.to_string())
    }

    pub fn edition_active(&self) -> Option<&Edition> {
        self.liste.iter().find(|e| !self.active.is_empty() && e.nom == self.active)
    }
}

/// Alertes des règles de l'édition sur tout le texte (positions en caractères).
pub fn alertes(edition: &Edition, texte: &str) -> Vec<AlerteGrammaticale> {
    let car: Vec<char> = texte.chars().collect();
    let mut res = Vec::new();
    for r in edition.regles.iter().filter(|r| r.active && !r.motif.trim().is_empty()) {
        // un motif fait d'espaces (« - » entouré d'espaces) est gardé tel quel
        let motif = if r.motif.trim().chars().count() > 1 { r.motif.trim() } else { r.motif.as_str() };
        let trouves = if r.casse { chercher_exact(&car, motif, r.mot_entier) } else { chercher_motif(&car, motif, r.mot_entier) };
        for (d, f) in trouves {
            let fautif: String = car[d..f].iter().collect();
            let explication = if r.explication.trim().is_empty() {
                format!("Convention de l'édition « {} ».", edition.nom)
            } else {
                r.explication.clone()
            };
            res.push(AlerteGrammaticale {
                debut: d,
                fin: f,
                suggestions: match (r.remplacement.is_empty(), r.casse) {
                    (true, _) => vec![],
                    (false, true) => vec![r.remplacement.clone()],
                    (false, false) => vec![adapte_casse(&fautif, &r.remplacement)],
                },
                texte_fautif: fautif,
                regle: format!("📘 {}", edition.nom),
                explication,
                genre: GenreErreur::Syntaxe,
            });
        }
    }
    res
}

/// Cellules d'une ligne : séparateur d'un caractère (guillemets « " » de CSV compris) ou de plusieurs (« → »).
fn cellules(ligne: &str, sep: &str) -> Vec<String> {
    let mut s = sep.chars();
    let (Some(c0), None) = (s.next(), s.next()) else { return ligne.split(sep).map(String::from).collect() };
    let car: Vec<char> = ligne.chars().collect();
    let mut res = vec![String::new()];
    let mut guil = false;
    let mut j = 0;
    while j < car.len() {
        let c = car[j];
        let cour = res.last_mut().unwrap();
        if c == '"' && guil && car.get(j + 1) == Some(&'"') {
            cour.push('"');
            j += 1;
        } else if c == '"' && (guil || cour.trim().is_empty()) {
            guil = !guil;
        } else if c == c0 && !guil {
            res.push(String::new());
        } else {
            cour.push(c);
        }
        j += 1;
    }
    res
}

/// Lignes d'un tableau en texte (copié d'Excel ou de Word, fichier CSV) ; chaque ligne prend le premier séparateur
/// de `seps` qu'elle contient.
fn lignes_texte(texte: &str, seps: &[&str]) -> Vec<Vec<String>> {
    texte
        .trim_start_matches('\u{feff}')
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| match seps.iter().find(|s| l.contains(*s)) {
            Some(s) => cellules(l, s),
            None => vec![l.to_string()],
        })
        .collect()
}

/// Texte collé : colonnes séparées par tabulation (Excel), « → », « => », « = », « ; » ou « | ».
pub fn lire_texte(texte: &str) -> Vec<Vec<String>> {
    lignes_texte(texte, &["\t", "→", "=>", " = ", ";", "|"])
}

/// Fichier .xlsx (première feuille), .csv, .tsv ou .txt.
pub fn lire_tableau(chemin: &std::path::Path) -> Result<Vec<Vec<String>>, String> {
    let ext = chemin.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "xlsx" {
        return lire_xlsx(chemin);
    }
    if ext == "xls" {
        return Err("ancien format .xls : enregistrez le classeur en .xlsx ou .csv".into());
    }
    let octets = std::fs::read(chemin).map_err(|e| e.to_string())?;
    // Excel enregistre souvent ses CSV en Windows-1252
    let texte = String::from_utf8(octets).unwrap_or_else(|e| {
        e.into_bytes()
            .iter()
            .map(|&b| match b {
                0x80 => '€', 0x85 => '…', 0x8C => 'Œ', 0x91 | 0x92 => '’', 0x93 => '“', 0x94 => '”',
                0x96 => '–', 0x97 => '—', 0x9C => 'œ', _ => b as char,
            })
            .collect()
    });
    Ok(lignes_texte(&texte, &["\t", ";", "→", "=>", "|", ","]))
}

fn lire_xlsx(chemin: &std::path::Path) -> Result<Vec<Vec<String>>, String> {
    use quick_xml::events::Event;
    use std::io::Read;
    let err = |e: &dyn std::fmt::Display| e.to_string();
    let mut za = zip::ZipArchive::new(std::fs::File::open(chemin).map_err(|e| err(&e))?).map_err(|e| err(&e))?;
    let feuille = za
        .file_names()
        .filter(|n| n.starts_with("xl/worksheets/sheet") && n.ends_with(".xml"))
        .min_by_key(|n| (n.len(), n.to_string()))
        .map(String::from)
        .ok_or("classeur sans feuille")?;
    let mut lis = |nom: &str| -> Option<String> {
        let mut s = String::new();
        za.by_name(nom).ok()?.read_to_string(&mut s).ok()?;
        Some(s)
    };
    let partages = lis("xl/sharedStrings.xml").unwrap_or_default();
    let xml = lis(&feuille).ok_or("feuille illisible")?;

    // textes partagés : un <si> par texte, fait d'un ou plusieurs <t>
    let mut textes: Vec<String> = Vec::new();
    let mut r = quick_xml::Reader::from_str(&partages);
    let mut dans_t = false;
    loop {
        match r.read_event().map_err(|e| err(&e))? {
            Event::Start(e) if e.local_name().as_ref() == b"si" => textes.push(String::new()),
            Event::Start(e) if e.local_name().as_ref() == b"t" => dans_t = true,
            Event::End(e) if e.local_name().as_ref() == b"t" => dans_t = false,
            Event::Text(t) if dans_t => {
                if let Some(d) = textes.last_mut() {
                    d.push_str(&t.unescape().map_err(|e| err(&e))?);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    let mut lignes: Vec<Vec<String>> = Vec::new();
    let mut r = quick_xml::Reader::from_str(&xml);
    let (mut col, mut genre, mut val, mut dans_v) = (0usize, String::new(), String::new(), false);
    loop {
        match r.read_event().map_err(|e| err(&e))? {
            Event::Start(e) if e.local_name().as_ref() == b"row" => lignes.push(Vec::new()),
            Event::Start(e) if e.local_name().as_ref() == b"c" => {
                let attr = |n: &str| e.try_get_attribute(n).ok().flatten().and_then(|a| a.unescape_value().ok()).map(|v| v.into_owned());
                let reference = attr("r").unwrap_or_default();
                let lettres = reference.bytes().take_while(u8::is_ascii_alphabetic);
                col = lettres.fold(0, |n, b| n * 26 + (b.to_ascii_uppercase() - b'A') as usize + 1).saturating_sub(1);
                genre = attr("t").unwrap_or_default();
                val.clear();
            }
            Event::Start(e) if matches!(e.local_name().as_ref(), b"v" | b"t") => dans_v = true,
            Event::End(e) if matches!(e.local_name().as_ref(), b"v" | b"t") => dans_v = false,
            Event::Text(t) if dans_v => val.push_str(&t.unescape().map_err(|e| err(&e))?),
            Event::End(e) if e.local_name().as_ref() == b"c" => {
                let v = if genre == "s" { val.trim().parse::<usize>().ok().and_then(|k| textes.get(k).cloned()).unwrap_or_default() } else { val.clone() };
                if let Some(l) = lignes.last_mut() {
                    if l.len() <= col {
                        l.resize(col + 1, String::new());
                    }
                    l[col] = v;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    lignes.retain(|l| l.iter().any(|c| !c.trim().is_empty()));
    Ok(lignes)
}

/// Règles tirées d'un tableau : A à éviter, B à écrire, C explication, D « oui » pour respecter les majuscules.
/// Une première ligne d'en-tête (« à éviter », « motif »…) est sautée.
pub fn regles_depuis(lignes: Vec<Vec<String>>) -> Vec<RegleEdition> {
    let entete = ["motif", "à éviter", "a eviter", "éviter", "fautif", "forme fautive", "incorrect", "erreur", "au lieu de", "ne pas écrire"];
    let oui = |c: Option<&String>| c.is_some_and(|c| matches!(c.trim().to_lowercase().as_str(), "oui" | "x" | "1" | "vrai" | "yes" | "true"));
    lignes
        .into_iter()
        .enumerate()
        .filter_map(|(k, l)| {
            let brut = l.first()?;
            // un motif fait d'espaces autour d'un signe (« - ») est gardé tel quel
            let garde = |b: &'_ str| if b.trim().chars().count() > 1 { b.trim().to_string() } else { b.to_string() };
            let motif = garde(brut);
            if motif.trim().is_empty() || k == 0 && entete.contains(&motif.trim().to_lowercase().as_str()) {
                return None;
            }
            let bord = |c: Option<char>| c.is_some_and(char::is_alphanumeric);
            Some(RegleEdition {
                remplacement: l.get(1).map_or(String::new(), |s| garde(s)),
                explication: l.get(2).map_or(String::new(), |s| s.trim().into()),
                mot_entier: bord(motif.chars().next()) && bord(motif.chars().last()),
                casse: oui(l.get(3)),
                active: true,
                motif,
            })
        })
        .collect()
}

/// Ajoute les règles ; une règle au motif déjà connu remplace l'ancienne (son explication gardée si la nouvelle n'en a pas).
/// Renvoie (ajoutées, mises à jour).
pub fn fusionne(e: &mut Edition, regles: Vec<RegleEdition>) -> (usize, usize) {
    let (mut aj, mut maj) = (0, 0);
    for mut r in regles {
        match e.regles.iter_mut().find(|x| x.motif == r.motif) {
            Some(x) => {
                if r.explication.is_empty() {
                    r.explication = x.explication.clone();
                }
                if *x != r {
                    *x = r;
                    maj += 1;
                }
            }
            None => {
                e.regles.push(r);
                aj += 1;
            }
        }
    }
    (aj, maj)
}

/// Règles en CSV (« ; », UTF-8), à modifier dans Excel puis réimporter.
pub fn vers_csv(e: &Edition) -> String {
    let q = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
    let mut t = String::from("\u{feff}à éviter;à écrire;explication;majuscules\r\n");
    for r in &e.regles {
        t += &format!("{};{};{};{}\r\n", q(&r.motif), q(&r.remplacement), q(&r.explication), if r.casse { "oui" } else { "non" });
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_et_export() {
        // collage depuis Excel (tabulations) ou à la main (→)
        let r = regles_depuis(lire_texte("À éviter\tÀ écrire\nc'est-à-dire\tc.-à-d.\tAbréviation\nMr → M.\n\n - \t – "));
        let vus: Vec<(&str, &str, &str, bool)> = r.iter().map(|x| (x.motif.as_str(), x.remplacement.as_str(), x.explication.as_str(), x.mot_entier)).collect();
        assert_eq!(vus, vec![("c'est-à-dire", "c.-à-d.", "Abréviation", true), ("Mr", "M.", "", true), (" - ", " – ", "", false)]);
        // export CSV puis réimport : mêmes règles
        let pl = peter_lang();
        let f = std::env::temp_dir().join("texteur_regles.test_tmp.csv");
        std::fs::write(&f, vers_csv(&pl)).unwrap();
        let r = regles_depuis(lire_tableau(&f).unwrap());
        let _ = std::fs::remove_file(&f);
        let cle = |v: &[RegleEdition]| v.iter().map(|x| (x.motif.clone(), x.remplacement.clone(), x.casse)).collect::<Vec<_>>();
        assert_eq!(cle(&r), cle(&pl.regles));
        // fusion : motif connu mis à jour, nouveau ajouté
        let mut e = Edition { nom: "E".into(), regles: r[..2].to_vec() };
        let mut nouvelle = r[0].clone();
        nouvelle.remplacement = "X".into();
        assert_eq!(fusionne(&mut e, vec![nouvelle, r[5].clone()]), (1, 1));
        assert_eq!(e.regles[0].remplacement, "X");
    }

    #[test]
    fn import_xlsx() {
        use std::io::Write;
        let f = std::env::temp_dir().join("texteur_regles.test_tmp.xlsx");
        let mut z = zip::ZipWriter::new(std::fs::File::create(&f).unwrap());
        let o = zip::write::SimpleFileOptions::default();
        z.start_file("xl/sharedStrings.xml", o).unwrap();
        z.write_all(r#"<sst><si><t>à éviter</t></si><si><t>c'est-à-dire</t></si><si><r><t>c.-à-</t></r><r><t>d.</t></r></si></sst>"#.as_bytes()).unwrap();
        z.start_file("xl/worksheets/sheet1.xml", o).unwrap();
        z.write_all(r#"<worksheet><sheetData><row r="1"><c r="A1" t="s"><v>0</v></c></row><row r="2"><c r="A2" t="s"><v>1</v></c><c r="B2" t="s"><v>2</v></c><c r="D2" t="inlineStr"><is><t>oui</t></is></c></row><row r="3"><c r="A3" t="inlineStr"><is><t>Mr &amp; Mme</t></is></c><c r="B3" t="inlineStr"><is><t>M. et Mme</t></is></c></row></sheetData></worksheet>"#.as_bytes()).unwrap();
        z.finish().unwrap();
        let r = regles_depuis(lire_tableau(&f).unwrap());
        let _ = std::fs::remove_file(&f);
        let vus: Vec<(&str, &str, bool)> = r.iter().map(|x| (x.motif.as_str(), x.remplacement.as_str(), x.casse)).collect();
        assert_eq!(vus, vec![("c'est-à-dire", "c.-à-d.", true), ("Mr & Mme", "M. et Mme", false)]);
    }

    #[test]
    fn regles_de_l_edition() {
        let pl = peter_lang();
        let a = alertes(&pl, "Pendant la Première guerre mondiale, l'Etat et l'État du nord de la Chine, etc... dans les années 50 - enfin.");
        let mut vus: Vec<&str> = a.iter().map(|x| x.texte_fautif.as_str()).collect();
        vus.sort();
        assert_eq!(vus, vec![" - ", "Etat", "Première guerre mondiale", "années 50", "etc..."]);
        let e = &Editions::default().liste[1];
        let a = alertes(e, "C'est-à-dire que Mr Wang et Mrs Li, etc... au XXème siècle.");
        let vus: Vec<(&str, &str)> = a.iter().map(|x| (x.texte_fautif.as_str(), x.suggestions[0].as_str())).collect();
        assert_eq!(vus, vec![("C'est-à-dire", "C.-à-d."), ("Mr", "M."), ("etc...", "etc."), ("XXème", "XXe")]);
        let mut aucune = Editions::default();
        assert_eq!(aucune.edition_active().map(|e| e.nom.as_str()), Some("Peter Lang"));
        aucune.active.clear();
        assert!(aucune.edition_active().is_none());
        aucune.active = "Exemple (à adapter)".into();
        assert!(aucune.edition_active().is_some());
    }
}
