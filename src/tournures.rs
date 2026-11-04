//! Règles précises issues de la relecture d'un livre réel : tournures fautives que les règles
//! générales et Grammalecte laissent passer, cohérence des noms propres d'un bout à l'autre du
//! document et concordance des sinogrammes avec le pinyin qui les accompagne.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::grammaire::{AlerteGrammaticale, GenreErreur};
use crate::masque::est_cjk;

pub const REGLE_SINOGRAMME: &str = "Sinogramme différent du pinyin";
pub const REGLE_ESPACE_NOMBRE: &str = "Espace manquante avant un nombre";
pub const REGLE_ANNEE_L: &str = "Lettre à la place du chiffre 1";
pub const REGLE_DATE_REGNE: &str = "Année incohérente avec le règne";
pub const REGLE_DECIMALE: &str = "Virgule décimale";
/// Règles dont le texte fautif contient des chiffres et qu'il faut garder malgré tout.
pub const REGLES_AVEC_CHIFFRES: &[&str] = &[REGLE_ESPACE_NOMBRE, REGLE_ANNEE_L, REGLE_DATE_REGNE, REGLE_DECIMALE];

struct Mot {
    t: String,
    bas: String,
    d: usize,
    f: usize,
}

fn est_lettre(c: char) -> bool {
    c.is_alphanumeric() && !est_cjk(c)
}

/// Mots du texte (indices en caractères) ; « d'échanges » donne « d' » + « échanges »,
/// mais « Xin'an » reste entier.
fn mots(car: &[char]) -> Vec<Mot> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < car.len() {
        if !est_lettre(car[i]) {
            i += 1;
            continue;
        }
        let d = i;
        while i < car.len()
            && (est_lettre(car[i])
                || (matches!(car[i], '-' | '\'' | '’') && i + 1 < car.len() && est_lettre(car[i + 1])))
        {
            i += 1;
        }
        let mut debut = d;
        for k in d..i {
            if matches!(car[k], '\'' | '’') {
                let avant: String = car[debut..k].iter().collect::<String>().to_lowercase();
                if ["c", "d", "j", "l", "m", "n", "s", "t", "qu", "jusqu", "puisqu", "lorsqu"].contains(&avant.as_str()) {
                    out.push(mot(car, debut, k + 1));
                    debut = k + 1;
                }
            }
        }
        out.push(mot(car, debut, i));
    }
    out
}

fn mot(car: &[char], d: usize, f: usize) -> Mot {
    let t: String = car[d..f].iter().collect();
    let bas = t.to_lowercase().replace('’', "'");
    Mot { t, bas, d, f }
}

fn alerte(d: usize, f: usize, fautif: String, regle: &str, explication: String, sugg: Vec<String>) -> AlerteGrammaticale {
    AlerteGrammaticale { debut: d, fin: f, texte_fautif: fautif, regle: regle.into(), explication, suggestions: sugg, genre: GenreErreur::Syntaxe }
}

/// Toutes les alertes de ce module, en indices de caractères dans `texte`.
pub fn analyse(texte: &str) -> Vec<AlerteGrammaticale> {
    let car: Vec<char> = texte.chars().collect();
    let m = mots(&car);
    let mut a = Vec::new();
    tournures(&car, &m, &mut a);
    tournures_bis(&car, &m, &mut a);
    dates_de_regne(&car, &m, &mut a);
    typographie(&car, &mut a);
    guillemets(&car, &mut a);
    noms_propres(&car, &m, &mut a);
    sinogrammes(&car, &mut a);
    a.sort_by_key(|x| (x.debut, x.fin));
    a
}

// ------------------------------------------------------------------ Tournures

const NEGATIONS: &[&str] = &["pas", "plus", "jamais", "guère", "point"];
const PAS_INFINITIFS: &[&str] = &[
    "premier", "dernier", "entier", "hier", "cher", "fier", "frère", "père", "mère", "derrière", "contre", "entre",
    "autre", "notre", "votre", "être", "lettre", "titre", "nombre", "ordre", "encore", "alors", "leur", "plusieurs",
];
const NOMS_EN_E: &[&str] = &[
    "été", "côté", "pitié", "amitié", "moitié", "santé", "vérité", "volonté", "liberté", "beauté", "charité", "société",
    "qualité", "quantité", "autorité", "priorité", "unité", "majorité", "minorité", "nécessité", "difficulté", "faculté",
    "propriété", "piété", "fierté", "bonté", "clarté", "cité", "communauté", "université", "activité", "curiosité",
    "sécurité", "identité", "réalité", "capacité", "fidélité", "marché", "café", "thé", "défilé", "degré", "gré", "pré",
    "blé", "clé", "idée", "année", "journée", "entrée", "armée", "lycée", "musée", "fée", "allée", "durée", "pensée",
];
const NOMBRES: &[&str] = &[
    "un", "une", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze", "douze", "quinze",
    "vingt", "trente", "quarante", "cinquante", "soixante", "cent", "mille",
];

fn infinitif_probable(m: &str) -> bool {
    m.chars().count() > 4
        && (m.ends_with("er") || m.ends_with("ir") || m.ends_with("re"))
        && !PAS_INFINITIFS.contains(&m)
        && m.chars().all(|c| c.is_lowercase() || c == '-')
}

fn est_nombre(m: &str) -> bool {
    m.chars().all(|c| c.is_ascii_digit()) || NOMBRES.contains(&m)
}

/// Infinitif d'un participe passé (« construit » → « construire »), s'il est sûr.
fn infinitif_du_participe(m: &str) -> Option<String> {
    if NOMS_EN_E.contains(&m) || ["ensuite", "suite", "fuite"].contains(&m) || m.chars().count() < 5 || !m.chars().all(|c| c.is_lowercase()) {
        return None;
    }
    for (fin, rempl) in [("ées", "er"), ("és", "er"), ("ée", "er"), ("é", "er"), ("uites", "uire"), ("uits", "uire"),
        ("uite", "uire"), ("uit", "uire"), ("crites", "crire"), ("crits", "crire"), ("crite", "crire"), ("crit", "crire")]
    {
        if let Some(r) = m.strip_suffix(fin) {
            return Some(format!("{r}{rempl}"));
        }
    }
    None
}

fn subjonctif_pouvoir(m: &str) -> Option<&'static str> {
    Some(match m {
        "peux" => "puisse",
        "peut" => "puisse",
        "pouvons" => "puissions",
        "pouvez" => "puissiez",
        "peuvent" => "puissent",
        _ => return None,
    })
}

fn singulier_verbe(m: &str) -> Option<String> {
    let s = match m {
        "doivent" => "doit", "peuvent" => "peut", "sont" => "est", "ont" => "a", "vont" => "va", "font" => "fait",
        "veulent" => "veut", "savent" => "sait", "seront" => "sera", "auront" => "aura", "devront" => "devra",
        "pourront" => "pourra", "furent" => "fut", "eurent" => "eut", "durent" => "dut", "purent" => "put",
        _ => return m.strip_suffix("aient").map(|r| format!("{r}ait")),
    };
    Some(s.into())
}

fn adapte_casse(modele: &str, s: &str) -> String {
    if modele.chars().next().is_some_and(|c| c.is_uppercase()) {
        let mut c = s.chars();
        c.next().map(|p| p.to_uppercase().chain(c).collect()).unwrap_or_default()
    } else {
        s.into()
    }
}

fn tournures(car: &[char], m: &[Mot], a: &mut Vec<AlerteGrammaticale>) {
    // Texte entre deux mots : sert à repérer ponctuation et fins de phrase
    let entre = |i: usize| -> String { car[m[i - 1].f..m[i].d].iter().collect() };
    let rupture = |s: &str| s.chars().any(|c| ".;:!?«»()[]—–\n".contains(c));
    let colle = |i: usize| i > 0 && entre(i).trim().is_empty();
    for i in 0..m.len() {
        let w = m[i].bas.as_str();
        let suiv = |k: usize| m.get(i + k).filter(|_| (1..=k).all(|j| colle(i + j))).map(|x| x.bas.as_str());

        // « Chacun de mes enfants … doivent » → doit
        // (« Dans chacun des districts existaient… » : sujet inversé, on ne touche pas)
        let apres_preposition = i > 0 && ["dans", "à", "pour", "de", "sur", "en", "par", "avec", "chez", "entre"].contains(&m[i - 1].bas.as_str());
        if matches!(w, "chacun" | "chacune") && !apres_preposition && matches!(suiv(1), Some("de" | "des" | "d'" | "du")) {
            for j in i + 2..(i + 14).min(m.len()) {
                let e = entre(j);
                if rupture(&e) || e.contains(',') || ["qui", "que", "qu'", "dont", "où"].contains(&m[j].bas.as_str()) {
                    break;
                }
                if let Some(s) = singulier_verbe(&m[j].bas) {
                    a.push(alerte(m[j].d, m[j].f, m[j].t.clone(), "Accord avec « chacun »",
                        format!("Le sujet est « {} » (singulier) : le verbe reste au singulier (« {s} »).", m[i].t), vec![s]));
                    break;
                }
            }
        }
        // « dans intérêt de » → dans l'intérêt de
        if matches!(w, "dans" | "pour") && suiv(1) == Some("intérêt") {
            a.push(alerte(m[i].d, m[i + 1].f, format!("{} {}", m[i].t, m[i + 1].t), "Article manquant",
                "Il manque l'article : « dans l'intérêt de ».".into(), vec![format!("{} l'intérêt", m[i].t)]));
        }
        // « n'arrive pas trouver » → arriver à + infinitif
        if w.starts_with("arriv") && !w.starts_with("arrivé") && w != "arrivée" {
            let mut j = i + 1;
            while j < m.len() && colle(j) && NEGATIONS.contains(&m[j].bas.as_str()) {
                j += 1;
            }
            if j > i + 1 && j < m.len() && colle(j) && infinitif_probable(&m[j].bas) {
                let p = &m[j - 1];
                a.push(alerte(p.d, p.f, p.t.clone(), "Préposition « à » manquante",
                    format!("On dit « arriver à + infinitif » : « {} à {} ».", p.t, m[j].t), vec![format!("{} à", p.t)]));
            }
        }
        // « fit construit » → fit construire
        if matches!(w, "fit" | "firent" | "faisait" | "faisaient" | "fera" | "feront" | "ferait" | "feraient" | "fasse"
            | "fassent" | "faire" | "fis" | "faisant")
        {
            if let Some(s) = suiv(1).and_then(infinitif_du_participe) {
                let p = &m[i + 1];
                a.push(alerte(p.d, p.f, p.t.clone(), "Infinitif après « faire »",
                    format!("Après « {} », le verbe se met à l'infinitif : « {} {s} ».", m[i].t, m[i].t), vec![s]));
            }
        }
        // « la seule que nous ne pouvons adopter » → que nous puissions adopter
        if matches!(w, "seul" | "seule" | "seuls" | "seules" | "unique") {
            if let Some(q) = (i + 1..(i + 4).min(m.len())).find(|&j| matches!(m[j].bas.as_str(), "que" | "qu'")) {
                let mut j = q + 2;
                let ne = j < m.len() && matches!(m[j].bas.as_str(), "ne" | "n'");
                if ne {
                    j += 1;
                }
                if let Some(s) = m.get(j).and_then(|x| subjonctif_pouvoir(&x.bas)) {
                    let d = if ne { m[j - 1].d } else { m[j].d };
                    let fautif: String = car[d..m[j].f].iter().collect();
                    a.push(alerte(d, m[j].f, fautif, "Subjonctif après « le seul… que »",
                        format!("Après « {} … que », on emploie le subjonctif « {s} »{}.", m[i].t,
                            if ne { " (et sans « ne », qui inverse le sens)" } else { "" }), vec![s.into()]));
                }
            }
        }
        // « entre 15 à 19 ans » → entre 15 et 19 ans
        if w == "entre" && suiv(1).is_some_and(est_nombre) && suiv(2) == Some("à") && suiv(3).is_some_and(est_nombre) {
            let p = &m[i + 2];
            a.push(alerte(p.d, p.f, p.t.clone(), "« entre … et … »",
                "On écrit « entre … et … » (ou « de … à … »).".into(), vec!["et".into()]));
        }
        // « sans privilégier…, ni sans faire » → ni faire
        if w == "ni" && suiv(1) == Some("sans") && m[..i].iter().rev().take(40).any(|x| x.bas == "sans") {
            a.push(alerte(m[i].d, m[i + 1].f, format!("{} {}", m[i].t, m[i + 1].t), "« ni sans »",
                "Après un premier « sans », la suite se coordonne par « ni » seul.".into(), vec![m[i].t.clone()]));
        }
        // « que « un » → qu'« un »
        if matches!(w, "que" | "lorsque" | "puisque" | "jusque") && i + 1 < m.len() {
            let e = entre(i + 1);
            let voyelle = m[i + 1].bas.starts_with(|c: char| "aeiouyéèêàâîôû".contains(c));
            if e.contains('«') && !rupture(&e.replace('«', "")) && voyelle {
                let s = format!("{}'", &m[i].t[..m[i].t.len() - 1]);
                a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Élision devant voyelle",
                    format!("« {} » s'élide devant une voyelle, même après un guillemet : « {s}« ».", m[i].t), vec![s]));
            }
        }
        // « finit bar baisser » → par
        if w == "bar" && suiv(1).is_some_and(infinitif_probable) {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Faute de frappe probable",
                "« bar » devant un infinitif : il s'agit sans doute de « par ».".into(), vec![adapte_casse(&m[i].t, "par")]));
        }
        // Traits d'union fautifs
        let sans_tiret = match w {
            "quand-même" => Some("quand même"),
            "tout-à-fait" => Some("tout à fait"),
            "eux-aussi" | "elles-aussi" | "nous-aussi" | "vous-aussi" | "lui-aussi" | "moi-aussi" | "toi-aussi" => Some(""),
            _ => None,
        };
        if let Some(s) = sans_tiret {
            let s = if s.is_empty() { m[i].t.replacen('-', " ", 1) } else { adapte_casse(&m[i].t, s) };
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Trait d'union superflu",
                format!("« {s} » s'écrit sans trait d'union."), vec![s]));
        }
        // « d'une part … d'une autre part » → d'autre part
        if w == "d'" && suiv(1) == Some("une") && suiv(2) == Some("autre") && suiv(3) == Some("part")
            && m[..i].windows(3).any(|x| x[0].bas == "d'" && x[1].bas == "une" && x[2].bas == "part")
        {
            a.push(alerte(m[i].d, m[i + 3].f, car[m[i].d..m[i + 3].f].iter().collect(), "« d'une part… d'autre part »",
                "La locution est « d'une part… d'autre part ».".into(), vec!["d'autre part".into()]));
        }
        // « était comme étant » → était connu comme étant
        if matches!(w, "est" | "était" | "étaient" | "sont" | "fut" | "furent") && suiv(1) == Some("comme") && suiv(2) == Some("étant") {
            a.push(alerte(m[i + 1].d, m[i + 1].f, m[i + 1].t.clone(), "Mot manquant",
                format!("« {} comme étant » : il manque sans doute « connu ».", m[i].t), vec!["connu comme".into()]));
        }
        // « de12 femmes » → de 12
        if let Some(p) = m[i].t.find(|c: char| c.is_ascii_digit()).filter(|&p| p > 0) {
            let (debut, fin) = m[i].t.split_at(p);
            if ["de", "à", "et", "en", "le", "la", "les", "des", "du", "sur", "par"].contains(&debut.to_lowercase().as_str())
                && fin.chars().all(|c| c.is_ascii_digit())
            {
                a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), REGLE_ESPACE_NOMBRE,
                    "Il manque une espace entre le mot et le nombre.".into(), vec![format!("{debut} {fin}")]));
            }
        }
    }
}

const ADJ_ANTEPOSES: &[&str] = &[
    "grand", "grands", "grande", "grandes", "petits", "petites", "belles", "beaux", "bons", "bonnes", "vieux", "vieilles",
    "nouveaux", "nouvelles", "longues", "longs", "hauts", "hautes", "jolies", "jolis", "mauvaises",
];
/// Noms qui forment avec l'adjectif une unité (« des petits pois ») : « des » y est admis.
const NOMS_SOUDES: &[&str] = &["filles", "gens", "enfants", "pois", "écoles", "magasins", "annonces", "surfaces", "villes", "parents", "pains", "personnes", "hommes", "femmes"];
/// Abréviations suivies d'un point où la phrase continue en minuscule (« cf. la note », « etc. »).
const ABREVIATIONS: &[&str] = &[
    "etc", "cf", "env", "av", "apr", "vol", "chap", "art", "éd", "ed", "trad", "fig", "ill", "op", "cit", "ibid", "id", "ex",
    "pp", "sq", "coll", "dir", "réf", "suiv", "hab", "max", "min", "mm", "mme", "mlle", "dr", "pr", "st", "ste", "vs", "no",
    "nos", "tél", "lat", "anc", "resp", "ca", "sqq", "var", "trim", "km", "cm", "kg",
];
const H_MUETS: &[&str] = &[
    "hôte", "homme", "hiver", "honneur", "histoire", "héritage", "hôpital", "habitant", "habitude", "habit", "horizon",
    "hommage", "hôtel", "humeur", "huile", "herbe", "heure", "héritier", "hospitalité", "harmonie", "humanité", "habitation",
];
const DETERMINANTS: &[&str] = &[
    "le", "la", "les", "l'", "un", "une", "des", "du", "au", "aux", "son", "sa", "ses", "leur", "leurs", "ce", "cet",
    "cette", "ces", "mon", "ma", "mes", "notre", "nos", "de", "d'",
];
const SUBJONCTIFS: &[&str] = &[
    "soit", "soient", "ait", "aient", "fût", "fussent", "eût", "eussent", "puisse", "puissent", "fasse", "fassent", "aille",
    "aillent", "sache", "sachent", "veuille", "veuillent", "vienne", "viennent", "prenne", "prennent", "doive", "doivent",
];
const RELATIFS: &[&str] = &["qui", "que", "qu'", "dont", "où", "lequel", "laquelle", "lesquels", "lesquelles"];

/// Subjonctif d'une forme d'indicatif courante (« fut » → « fût »).
fn subjonctif_de(m: &str) -> Option<&'static str> {
    Some(match m {
        "est" => "soit", "sont" => "soient", "a" => "ait", "ont" => "aient", "fut" => "fût", "furent" => "fussent",
        "était" => "fût", "étaient" => "fussent", "avait" => "eût", "avaient" => "eussent", "peut" => "puisse",
        "peuvent" => "puissent", "pouvait" => "pût", "pouvaient" => "pussent",
        _ => return None,
    })
}

fn imparfait_probable(m: &str) -> bool {
    m.chars().count() > 5 && (m.ends_with("ait") || m.ends_with("aient")) && m.chars().all(|c| c.is_lowercase())
}

/// Tournures relevées lors de la relecture complète du livre (chapitres 00 à 09).
fn tournures_bis(car: &[char], m: &[Mot], a: &mut Vec<AlerteGrammaticale>) {
    let entre = |i: usize| -> String { car[m[i - 1].f..m[i].d].iter().collect() };
    let rupture = |s: &str| s.chars().any(|c| ".;:!?«»()[]—–\n".contains(c));
    let colle = |i: usize| i > 0 && i < m.len() && entre(i).trim().is_empty();
    let debut_de_phrase = |i: usize| i == 0 || rupture(&entre(i));
    for i in 0..m.len() {
        let w = m[i].bas.as_str();
        let suiv = |k: usize| m.get(i + k).filter(|_| (1..=k).all(|j| colle(i + j))).map(|x| x.bas.as_str());
        let prec = (i > 0 && colle(i)).then(|| m[i - 1].bas.as_str());

        // « (l393) » → 1393
        let t: Vec<char> = m[i].t.chars().collect();
        if t.len() == 4 && matches!(t[0], 'l' | 'I') && t[1..].iter().all(|c| c.is_ascii_digit()) {
            let s: String = std::iter::once('1').chain(t[1..].iter().copied()).collect();
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), REGLE_ANNEE_L,
                format!("« {} » commence par la lettre « {} » au lieu du chiffre 1.", m[i].t, t[0]), vec![s]));
        }
        // « amÉnagé » : capitale accentuée au milieu d'un mot
        if let Some(k) = (1..t.len()).find(|&k| t[k - 1].is_lowercase() && t[k].is_uppercase() && !t[k].is_ascii()) {
            let s: String = t[..k].iter().copied().chain(t[k..].iter().flat_map(|c| c.to_lowercase())).collect();
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Majuscule au milieu d'un mot",
                format!("« {} » ne prend pas de majuscule au milieu du mot.", t[k]), vec![s]));
        }
        // « presqu'aucun » → presque aucun (seul « presqu'île » s'élide)
        if let Some(r) = w.strip_prefix("presqu'").filter(|r| !r.is_empty() && !r.starts_with("île")) {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Élision de « presque »",
                "« presque » ne s'élide que dans « presqu'île ».".into(), vec![adapte_casse(&m[i].t, &format!("presque {r}"))]));
        }
        // « pour des grandes familles » → de grandes familles
        let indefini = i == 0 || (rupture(&entre(i)) && !entre(i).contains(':'))
            || prec.is_some_and(|p| ["pour", "avec", "par", "sur", "dans", "chez", "sans", "comme", "contre", "vers", "étaient",
                "sont", "était", "est", "avait", "avaient", "a", "ont", "également", "aussi", "voici"].contains(&p));
        if w == "des" && indefini && suiv(1).is_some_and(|x| ADJ_ANTEPOSES.contains(&x))
            && suiv(2).is_some_and(|x| x.chars().all(|c| c.is_lowercase()) && !NOMS_SOUDES.contains(&x)
                && !["et", "ou", "de", "des", "du", "d'", "à", "au", "aux", "que", "qu'", "comme", "mais"].contains(&x))
        {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "« de » devant un adjectif",
                format!("Devant un adjectif placé avant le nom, « des » devient « de » : « de {} {} ».", m[i + 1].t, m[i + 2].t),
                vec![adapte_casse(&m[i].t, "de")]));
        }
        // « du hôte » → de l'hôte
        if let Some(n) = suiv(1).filter(|x| H_MUETS.contains(x)) {
            let s = match w { "du" => Some("de l'"), "le" | "la" => Some("l'"), "de" => Some("d'"), _ => None };
            if let Some(s) = s {
                a.push(alerte(m[i].d, m[i + 1].f, car[m[i].d..m[i + 1].f].iter().collect(), "Élision devant h muet",
                    format!("« {n} » commence par un h muet : on élide."), vec![format!("{}{}", adapte_casse(&m[i].t, s), m[i + 1].t)]));
            }
        }
        // « Veuillez-vous référer » → Veuillez vous référer
        if w == "veuillez-vous" {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Trait d'union superflu",
                "« vous » est complément de l'infinitif qui suit : pas de trait d'union.".into(), vec![m[i].t.replacen('-', " ", 1)]));
        }
        // « s'explique parce que » → par le fait que
        if w.starts_with("expliqu") && suiv(1) == Some("parce") && matches!(suiv(2), Some("que" | "qu'")) {
            a.push(alerte(m[i + 1].d, m[i + 2].f, car[m[i + 1].d..m[i + 2].f].iter().collect(), "« s'expliquer par »",
                "On dit « s'expliquer par le fait que ».".into(), vec![format!("par le fait {}", m[i + 2].t)]));
        }
        // « en terme de / en terme général » → en termes
        if w == "en" && suiv(1) == Some("terme") && matches!(suiv(2), Some("de" | "d'" | "général" | "généraux")) {
            a.push(alerte(m[i + 1].d, m[i + 1].f, m[i + 1].t.clone(), "« en termes de »",
                "La locution s'écrit au pluriel : « en termes de », « en termes généraux ».".into(), vec!["termes".into()]));
        }
        // « en rapport à » → par rapport à
        if w == "en" && suiv(1) == Some("rapport") && matches!(suiv(2), Some("à" | "au" | "aux")) {
            a.push(alerte(m[i].d, m[i + 1].f, car[m[i].d..m[i + 1].f].iter().collect(), "« par rapport à »",
                "On dit « par rapport à » (ou « en rapport avec »).".into(), vec![adapte_casse(&m[i].t, "par rapport")]));
        }
        // « insister de faire » → insister pour faire
        if w.starts_with("insist") && matches!(suiv(1), Some("de" | "d'")) && suiv(2).is_some_and(infinitif_probable) {
            a.push(alerte(m[i + 1].d, m[i + 1].f, m[i + 1].t.clone(), "« insister pour »",
                "On dit « insister pour + infinitif ».".into(), vec!["pour".into()]));
        }
        // « S'il agit d'un… » → s'il s'agit
        if w == "agit" && prec == Some("il") && i >= 2 && m[i - 2].bas == "s'" && matches!(suiv(1), Some("de" | "d'")) {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "« il s'agit de »",
                "La tournure impersonnelle est « il s'agit de ».".into(), vec!["s'agit".into()]));
        }
        // pinyin « nvzi » → nüzi (v pour ü)
        if t.len() > 2 && matches!(t[0], 'n' | 'l' | 'N' | 'L') && t[1] == 'v' && t[2].is_alphabetic() {
            let s: String = t.iter().enumerate().map(|(k, &c)| if k == 1 { 'ü' } else { c }).collect();
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Pinyin : ü",
                "En pinyin, « v » note la voyelle ü.".into(), vec![s]));
        }
        // « etc » sans point
        if w == "etc" && car.get(m[i].f) != Some(&'.') {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "« etc. »", "« etc. » prend toujours un point.".into(), vec!["etc.".into()]));
        }
        // « pourcent » → pour cent
        if matches!(w, "pourcent" | "pourcents") {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "« pour cent »", "On écrit « pour cent » (ou %).".into(), vec!["pour cent".into()]));
        }
        // « d'lettrés » : élision devant consonne
        if matches!(w, "d'" | "l'" | "qu'" | "n'" | "s'" | "c'" | "j'" | "m'" | "t'")
            && suiv(1).is_some_and(|x| x.starts_with(|c: char| c.is_alphabetic() && !"aeiouyhéèêëàâäîïôöûüùœæ".contains(c)))
        {
            let plein: String = match w { "qu'" => "que".into(), "c'" => "ce".into(), x => format!("{}e", &x[..1]) };
            a.push(alerte(m[i].d, m[i + 1].f, car[m[i].d..m[i + 1].f].iter().collect(), "Élision devant consonne",
                format!("On n'élide pas devant une consonne : « {plein} {} ».", m[i + 1].t),
                vec![format!("{} {}", adapte_casse(&m[i].t, &plein), m[i + 1].t)]));
        }
        // « du le règne » → du règne
        if w == "du" && matches!(suiv(1), Some("le" | "la" | "les")) {
            a.push(alerte(m[i].d, m[i + 1].f, car[m[i].d..m[i + 1].f].iter().collect(), "Article en trop",
                "« du » contient déjà l'article.".into(), vec![m[i].t.clone()]));
        }
        // « a lui seul » → à lui seul
        if w == "a" && matches!(suiv(1), Some("lui" | "elle" | "eux" | "elles")) && suiv(2).is_some_and(|x| x.starts_with("seul")) {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "« à lui seul »", "Locution : « à lui seul » (préposition).".into(),
                vec![adapte_casse(&m[i].t, "à")]));
        }
        // « à côte de » → à côté de
        if w == "côte" && prec == Some("à") && matches!(suiv(1), Some("de" | "du" | "des" | "d'")) {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "« à côté de »", "Locution : « à côté de ».".into(), vec!["côté".into()]));
        }
        // « des commençants de Huizhou » → commerçants
        if w == "commençants" {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Confusion probable",
                "« commençants » désigne des débutants : il s'agit sans doute de « commerçants ».".into(), vec!["commerçants".into()]));
        }
        // « aller-retours » → allers-retours
        if w == "aller-retours" {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Pluriel de « aller-retour »",
                "Au pluriel : « allers-retours ».".into(), vec![adapte_casse(&m[i].t, "allers-retours")]));
        }
        // « des vielles mœurs » → vieilles
        if matches!(w, "vielle" | "vielles") && prec.is_some_and(|p| DETERMINANTS.contains(&p))
            && suiv(1).is_some_and(|x| !["à", "et", "de", "du", "des", "d'"].contains(&x))
        {
            a.push(alerte(m[i].d, m[i].f, m[i].t.clone(), "Confusion vielle / vieille",
                "La « vielle » est un instrument de musique ; l'adjectif est « vieille ».".into(),
                vec![m[i].t.replace("vielle", "vieille")]));
        }
        // subjonctif après « bien que, pour que… » ; indicatif après « après que »
        let conj = matches!(suiv(1), Some("que" | "qu'"))
            && (matches!(w, "pour" | "afin" | "avant" | "sans" | "après")
                || w == "bien" && (debut_de_phrase(i) || entre(i).contains(',') || prec.is_some_and(|p| ["et", "mais", "or"].contains(&p))));
        if conj || matches!(w, "quoique" | "quoiqu'") {
            let premier = if conj { i + 2 } else { i + 1 };
            for j in premier..(premier + 12).min(m.len()) {
                let e = entre(j);
                let v = m[j].bas.as_str();
                if rupture(&e) || e.contains(',') || RELATIFS.contains(&v) {
                    break;
                }
                let apres_det = DETERMINANTS.contains(&m[j - 1].bas.as_str());
                if w == "après" {
                    let ind = match v { "ait" => "eut", "aient" => "eurent", "soit" => "fut", "soient" => "furent", _ => "" };
                    if !ind.is_empty() {
                        a.push(alerte(m[j].d, m[j].f, m[j].t.clone(), "Indicatif après « après que »",
                            "« après que » exprime un fait accompli : il se construit avec l'indicatif.".into(), vec![ind.into()]));
                        break;
                    }
                } else if SUBJONCTIFS.contains(&v) {
                    break;
                } else if let Some(s) = subjonctif_de(v).filter(|_| !apres_det) {
                    a.push(alerte(m[j].d, m[j].f, m[j].t.clone(), "Subjonctif attendu",
                        format!("« {} {} » se construit avec le subjonctif : « {s} ».", m[i].t, m[i + 1].t), vec![s.into()]));
                    break;
                } else if imparfait_probable(v) && !apres_det && j > premier {
                    a.push(alerte(m[j].d, m[j].f, m[j].t.clone(), "Subjonctif attendu",
                        format!("« {} {} » se construit avec le subjonctif, pas avec l'imparfait de l'indicatif.", m[i].t, m[i + 1].t),
                        vec![]));
                    break;
                }
            }
        }
        // « Bien que …, cependant … » : double opposition
        if w == "bien" && suiv(1).is_some_and(|x| x == "que" || x == "qu'") && debut_de_phrase(i) {
            for j in i + 2..(i + 60).min(m.len()) {
                let e = entre(j);
                if rupture(&e) {
                    break;
                }
                if e.contains(',') && matches!(m[j].bas.as_str(), "cependant" | "mais" | "toutefois" | "néanmoins") {
                    a.push(alerte(m[j].d, m[j].f, m[j].t.clone(), "« Bien que … cependant »",
                        format!("« {} {} » marque déjà l'opposition : « {} » fait double emploi.", m[i].t, m[i + 1].t, m[j].t), vec![]));
                    break;
                }
            }
        }
    }
}

/// Premier année grégorienne de chaque règne (l'an I).
const REGNES: &[(&str, i32)] = &[
    ("hongwu", 1368), ("jianwen", 1399), ("yongle", 1403), ("hongxi", 1425), ("xuande", 1426), ("zhengtong", 1436),
    ("jingtai", 1450), ("tianshun", 1457), ("chenghua", 1465), ("hongzhi", 1488), ("zhengde", 1506), ("jiajing", 1522),
    ("longqing", 1567), ("wanli", 1573), ("taichang", 1620), ("tianqi", 1621), ("chongzhen", 1628), ("shunzhi", 1644),
    ("kangxi", 1662), ("yongzheng", 1723), ("qianlong", 1736), ("jiaqing", 1796), ("daoguang", 1821), ("xianfeng", 1851),
    ("tongzhi", 1862), ("guangxu", 1875), ("xuantong", 1909), ("chunhua", 990), ("xuanhe", 1119), ("chunxi", 1174),
    ("jingding", 1260), ("xianchun", 1265), ("zhizheng", 1341), ("guangqi", 885),
];

fn romain(s: &str) -> Option<i32> {
    if s.is_empty() || !s.chars().all(|c| "IVXLC".contains(c)) {
        return None;
    }
    let v = |c| match c { 'I' => 1, 'V' => 5, 'X' => 10, 'L' => 50, _ => 100 };
    let c: Vec<i32> = s.chars().map(v).collect();
    Some((0..c.len()).map(|k| if c.get(k + 1).is_some_and(|&n| n > c[k]) { -c[k] } else { c[k] }).sum())
}

/// « en l'an XXVI du règne de Kangxi (1683) » : Kangxi commence en 1662, l'an XXVI est 1687.
fn dates_de_regne(car: &[char], m: &[Mot], a: &mut Vec<AlerteGrammaticale>) {
    for i in 1..m.len().saturating_sub(4) {
        if m[i].bas != "an" || m[i - 1].bas != "l'" {
            continue;
        }
        let Some(n) = romain(&m[i + 1].t) else { continue };
        let mut j = i + 2;
        let republique = m[j].bas == "de" && m.get(j + 2).is_some_and(|x| x.bas == "république");
        let (nom, debut) = if republique {
            (j + 2, 1912)
        } else {
            if m[j].bas != "du" || m.get(j + 1).map(|x| x.bas.as_str()) != Some("règne") || m.get(j + 2).map(|x| x.bas.as_str()) != Some("de") {
                continue;
            }
            j += 3;
            while m.get(j).is_some_and(|x| matches!(x.bas.as_str(), "l'" | "empereur")) {
                j += 1;
            }
            let Some(&(_, d)) = m.get(j).and_then(|x| REGNES.iter().find(|r| r.0 == x.bas)) else { continue };
            (j, d)
        };
        // l'année entre parenthèses ou crochets, peu après le nom du règne
        let fin = (m[nom].f + 60).min(car.len());
        let Some(p) = (m[nom].f..fin).find(|&k| matches!(car[k], '(' | '[')) else { continue };
        if car[m[nom].f..p].iter().any(|c| !(c.is_alphabetic() || matches!(c, ' ' | '\'' | '’' | '-' | '\u{a0}'))) {
            continue;
        }
        let mut q = p + 1;
        while q < car.len() && (car[q].is_ascii_digit() || q == p + 1 && car[q] == 'l') {
            q += 1;
        }
        let annee: String = car[p + 1..q].iter().map(|&c| if c == 'l' { '1' } else { c }).collect();
        let Ok(annee) = annee.parse::<i32>() else { continue };
        if !(3..=4).contains(&(q - p - 1)) || !matches!(car.get(q), Some(')' | ']' | ' ')) {
            continue;
        }
        let attendu = debut + n - 1;
        if annee != attendu {
            a.push(alerte(p + 1, q, car[p + 1..q].iter().collect(), REGLE_DATE_REGNE,
                format!("« {} » commence en {debut} : l'an {} correspond à {attendu}. Soit l'année, soit le numéro de l'an est faux.",
                    m[nom].t, m[i + 1].t), vec![attendu.to_string()]));
        }
    }
}

/// Ponctuation et typographie, au caractère près.
fn typographie(car: &[char], a: &mut Vec<AlerteGrammaticale>) {
    let lettre = |k: usize| car.get(k).is_some_and(|c| c.is_alphabetic() && !est_cjk(*c));
    let txt = |d: usize, f: usize| -> String { car[d..f].iter().collect() };
    for k in 0..car.len() {
        let c = car[k];
        let n1 = car.get(k + 1).copied();
        let n2 = car.get(k + 2).copied();
        // « ?. » ; « .. » (hors points de suspension)
        if matches!(c, '?' | '!') && n1 == Some('.') && n2 != Some('.') {
            a.push(alerte(k, k + 2, txt(k, k + 2), "Double ponctuation", format!("« {c} » termine déjà la phrase."), vec![c.to_string()]));
        }
        if c == '.' && n1 == Some('.') && n2 != Some('.') && (k == 0 || car[k - 1] != '.') {
            a.push(alerte(k, k + 2, txt(k, k + 2), "Double ponctuation", "Point répété.".into(), vec![".".into()]));
        }
        // « enfant . » : espace avant le point
        if c == ' ' && n1 == Some('.') && n2 != Some('.') && k > 0 && (lettre(k - 1) || matches!(car[k - 1], ')' | '»'))
            && n2.is_none_or(|x| x.is_whitespace())
        {
            a.push(alerte(k, k + 2, txt(k, k + 2), "Espace avant le point", "Pas d'espace avant un point.".into(), vec![".".into()]));
        }
        // « d'argent.Sous » : espace manquante après le point
        if c == '.' && k >= 2 && car[k - 1].is_lowercase() && car[k - 2].is_lowercase() && lettre(k - 1)
            && n1.is_some_and(|x| x.is_uppercase() && !est_cjk(x)) && n2.is_some_and(|x| x.is_lowercase())
        {
            a.push(alerte(k, k + 2, txt(k, k + 2), "Espace après le point", "Il manque une espace après le point.".into(),
                vec![format!(". {}", n1.unwrap())]));
        }
        // « … du district. le chiffre » : majuscule après un point (hors abréviations, initiales, points de suspension)
        if c == '.' && n1 == Some(' ') && n2.is_some_and(|x| x.is_lowercase()) && lettre(k + 2) && (k == 0 || car[k - 1] != '.') {
            let mut d = k;
            while d > 0 && lettre(d - 1) {
                d -= 1;
            }
            let avant: String = car[d..k].iter().collect::<String>().to_lowercase();
            if avant.chars().count() > 1 && !ABREVIATIONS.contains(&avant.as_str()) && (d == 0 || !matches!(car[d - 1], '.' | '-')) {
                let f = (k + 2..car.len()).find(|&x| !lettre(x)).unwrap_or(car.len());
                let mot = txt(k + 2, f);
                let maj: String = mot.chars().take(1).flat_map(char::to_uppercase).chain(mot.chars().skip(1)).collect();
                a.push(alerte(k + 2, f, mot, "Majuscule en début de phrase", "Après un point, la phrase commence par une majuscule.".into(),
                    vec![maj]));
            }
        }
        // « l' enregistrement » : espace après une apostrophe d'élision
        if "ldnmstcjLDNMSTCJ".contains(c) && (k == 0 || !lettre(k - 1)) && matches!(n1, Some('\'' | '’')) && n2 == Some(' ')
            && lettre(k + 3)
        {
            a.push(alerte(k, k + 3, txt(k, k + 3), "Espace après l'apostrophe", "Pas d'espace après une apostrophe d'élision.".into(),
                vec![txt(k, k + 2)]));
        }
        // « 10.7 % » → 10,7 %
        if c == '.' && k > 0 && car[k - 1].is_ascii_digit() && n1.is_some_and(|x| x.is_ascii_digit()) {
            let mut d = k;
            while d > 0 && car[d - 1].is_ascii_digit() {
                d -= 1;
            }
            let mut f = k + 1;
            while f < car.len() && car[f].is_ascii_digit() {
                f += 1;
            }
            let pourcent = car.get(f) == Some(&'%') || matches!(car.get(f), Some(' ' | '\u{a0}' | '\u{202f}')) && car.get(f + 1) == Some(&'%');
            if pourcent && (d == 0 || !matches!(car[d - 1], '.' | ',')) {
                a.push(alerte(d, f, txt(d, f), REGLE_DECIMALE, "En français, la partie décimale se sépare par une virgule.".into(),
                    vec![txt(d, f).replace('.', ",")]));
            }
        }
        // parenthèses chinoises （ ） autour d'un texte français
        if c == '（' {
            if let Some(f) = (k + 1..(k + 80).min(car.len())).find(|&f| car[f] == '）') {
                if (k + 1..f).any(|x| car[x].is_ascii_alphabetic()) {
                    a.push(alerte(k, f + 1, txt(k, f + 1), "Parenthèses chinoises",
                        "Dans un texte français, on emploie les parenthèses ( ).".into(), vec![format!("({})", txt(k + 1, f))]));
                }
            }
        }
    }
}

/// Guillemets mal appariés, paragraphe par paragraphe (citations imbriquées permises) :
/// « » » sans ouvrant, ou deux « encore ouverts en fin de paragraphe. Un seul « ouvert est permis :
/// la citation continue au paragraphe suivant, qui peut alors fermer sans rouvrir.
fn guillemets(car: &[char], a: &mut Vec<AlerteGrammaticale>) {
    let mut reste = false;
    for (debut, para) in paragraphes(car) {
        if para.iter().all(|c| c.is_whitespace()) {
            continue;
        }
        let mut pile = Vec::new();
        for (k, &c) in para.iter().enumerate() {
            match c {
                '«' => pile.push(debut + k),
                '»' if pile.pop().is_none() => {
                    if reste {
                        reste = false;
                    } else {
                        a.push(alerte(debut + k, debut + k + 1, "»".into(), "Guillemet fermant orphelin",
                            "Ce guillemet fermant n'a pas de guillemet ouvrant.".into(), vec![]));
                    }
                }
                _ => {}
            }
        }
        if pile.len() >= 2 {
            a.push(alerte(pile[0], pile[0] + 1, "«".into(), "Guillemet non fermé",
                "Ce guillemet ouvrant n'est jamais fermé dans le paragraphe.".into(), vec![]));
        }
        reste = !pile.is_empty() || reste && !para.contains(&'«') && !para.contains(&'»');
    }
}

/// Paragraphes du texte : (indice du premier caractère, caractères).
fn paragraphes(car: &[char]) -> impl Iterator<Item = (usize, &[char])> {
    let mut debut = 0;
    car.split(|&c| c == '\n').map(move |p| {
        let d = debut;
        debut += p.len() + 1;
        (d, p)
    })
}

// ------------------------------------------------------------------ Noms propres

/// Distance d'édition 1 (substitution, insertion ou inversion de deux lettres voisines) ;
/// renvoie aussi si c'est une substitution.
fn distance_un(a: &[char], b: &[char]) -> Option<bool> {
    if a.len() == b.len() {
        let diff: Vec<usize> = (0..a.len()).filter(|&k| a[k] != b[k]).collect();
        return match diff.as_slice() {
            [_] => Some(true),
            [x, y] if *y == x + 1 && a[*x] == b[*y] && a[*y] == b[*x] => Some(false),
            _ => None,
        };
    }
    let (court, long) = if a.len() < b.len() { (a, b) } else { (b, a) };
    if long.len() != court.len() + 1 {
        return None;
    }
    let k = (0..court.len()).find(|&k| court[k] != long[k]).unwrap_or(court.len());
    (court[k..] == long[k + 1..]).then_some(false)
}

fn noms_propres(car: &[char], m: &[Mot], a: &mut Vec<AlerteGrammaticale>) {
    let est_nom = |t: &str| {
        let mut c = t.chars();
        c.next().is_some_and(|p| p.is_uppercase() && p.is_alphabetic())
            && c.all(|x| x.is_lowercase() || matches!(x, '\'' | '’'))
            && t.chars().count() >= 4
    };
    let mut nb: HashMap<&str, usize> = HashMap::new();
    let mut paires: HashMap<String, usize> = HashMap::new();
    let minuscules: HashSet<&str> = m.iter().map(|x| x.t.as_str()).filter(|t| t.chars().next().is_some_and(char::is_lowercase)).collect();
    for (i, x) in m.iter().enumerate() {
        if est_nom(&x.t) {
            *nb.entry(&x.t).or_default() += 1;
        }
        if i > 0 && car[m[i - 1].f..x.d].iter().collect::<String>() == " " && est_nom(&m[i - 1].t) {
            *paires.entry(format!("{} {}", m[i - 1].t, x.t)).or_default() += 1;
        }
    }
    let frequents: Vec<(Vec<char>, &str, usize)> = nb.iter().filter(|e| *e.1 >= 2).map(|(t, n)| (t.chars().collect(), *t, *n)).collect();
    for x in m {
        if nb.get(x.t.as_str()) != Some(&1) || minuscules.contains(x.t.to_lowercase().as_str()) || x.t.ends_with(['\'', '’'])
            || MOTS_COURANTS.contains(&x.bas.as_str())
        {
            continue;
        }
        let c: Vec<char> = x.t.chars().collect();
        let syllabes = segmente(&normalise(&x.t));
        if syllabes.is_some() && annote_par_sinogrammes(car, x.f) {
            continue; // l'auteur a donné les caractères : nom voulu
        }
        let proche = frequents.iter().filter(|(f, _, _)| f[0] == c[0]).find_map(|(f, t, n)| {
            distance_un(&c, f)?;
            let ok = match (syllabes, segmente(&normalise(t))) {
                // deux pinyin : seul l'oubli d'un n, g ou h trahit une faute (Zenwei / Zengwei, Chu'an / Chun'an) ;
                // une lettre changée ou ajoutée donne souvent un autre nom (Liang / Jiang, Longtan / Longtang)
                (Some(s), Some(_)) => s >= 2 && *n >= 3 && oubli(&c, f).is_some_and(|l| matches!(l, 'n' | 'g' | 'h')),
                // nom chinois d'une syllabe près d'un mot français (Quan / Quand) : rien à signaler
                (Some(s), None) => s >= 2 && !pluriel(&c, f),
                // mot ordinaire : pas un simple pluriel (Tableaux / Tableau)
                (None, _) => !pluriel(&c, f),
            };
            ok.then_some((*t, *n))
        });
        let colle = paires.iter().find(|(p, n)| **n >= 2 && p.replace(' ', "").to_lowercase() == x.bas).map(|(p, n)| (p.as_str(), *n));
        if let Some((t, n)) = proche.or(colle) {
            a.push(alerte(x.d, x.f, x.t.clone(), "Nom propre incohérent",
                format!("« {} » n'apparaît qu'une fois, alors que « {t} » apparaît {n} fois : faute de frappe ?", x.t), vec![t.into()]));
        }
    }
}

/// Mots français à majuscule qu'une lettre sépare d'un autre mot courant (Quant / Quand, État / Étant)
const MOTS_COURANTS: &[&str] = &["quant", "dieux", "état", "états", "note", "notes", "étant", "deux", "quand"];

/// Lettre de `long` absente de `court` quand `court` est `long` amputé d'une lettre.
fn oubli(court: &[char], long: &[char]) -> Option<char> {
    if long.len() != court.len() + 1 {
        return None;
    }
    let k = (0..court.len()).find(|&k| court[k] != long[k]).unwrap_or(court.len());
    (court[k..] == long[k + 1..]).then(|| long[k])
}

fn pluriel(a: &[char], b: &[char]) -> bool {
    let (court, long) = if a.len() < b.len() { (a, b) } else { (b, a) };
    long.len() == court.len() + 1 && long.starts_with(court) && matches!(long[court.len()], 's' | 'x')
}

/// Le mot est suivi de « (汉字) » ou « 汉字 » qui correspondent à son pinyin.
fn annote_par_sinogrammes(car: &[char], mut i: usize) -> bool {
    while i < car.len() && matches!(car[i], ' ' | '(' | '\u{a0}') {
        i += 1;
    }
    i < car.len() && est_cjk(car[i])
}

// ------------------------------------------------------------------ Sinogrammes / pinyin

fn lectures() -> &'static (HashMap<char, Vec<&'static str>>, HashSet<&'static str>) {
    static T: OnceLock<(HashMap<char, Vec<&'static str>>, HashSet<&'static str>)> = OnceLock::new();
    T.get_or_init(|| {
        let mut l = HashMap::new();
        let mut syll = HashSet::new();
        for ligne in include_str!("../ressources/pinyin_caracteres.txt").lines() {
            let mut c = ligne.chars();
            if let Some(h) = c.next() {
                let v: Vec<&'static str> = ligne[h.len_utf8()..].split(' ').filter(|s| !s.is_empty()).collect();
                syll.extend(v.iter().copied());
                l.insert(h, v);
            }
        }
        (l, syll)
    })
}

/// Minuscules sans accents ni séparateurs (« Xin’an » → « xinan », « Lü » → « lu »).
fn normalise(s: &str) -> String {
    s.nfd().filter(|c| !is_combining_mark(*c)).filter(|c| c.is_ascii_alphabetic()).collect::<String>().to_lowercase()
}

/// Nombre de syllabes pinyin si `s` se découpe entièrement en syllabes.
fn segmente(s: &str) -> Option<usize> {
    let syll = &lectures().1;
    let b = s.as_bytes();
    let mut meilleur: Vec<Option<usize>> = vec![None; b.len() + 1];
    meilleur[0] = Some(0);
    for i in 0..b.len() {
        let Some(n) = meilleur[i] else { continue };
        for l in 1..=6.min(b.len() - i) {
            if syll.contains(&s[i..i + l]) && meilleur[i + l].is_none_or(|x| x > n + 1) {
                meilleur[i + l] = Some(n + 1);
            }
        }
    }
    meilleur[b.len()]
}

/// Aligne le pinyin `s` sur les caractères `h` ; renvoie les positions des caractères qui ne
/// correspondent pas (au plus `max`), avec la syllabe lue à leur place.
fn aligne(s: &str, h: &[char], max: usize) -> Option<Vec<(usize, String)>> {
    let (l, syll) = lectures();
    fn rec(s: &str, h: &[char], j: usize, max: usize, l: &HashMap<char, Vec<&str>>, syll: &HashSet<&str>) -> Option<Vec<(usize, String)>> {
        if j == h.len() {
            return s.is_empty().then(Vec::new);
        }
        let lec = l.get(&h[j])?;
        for r in lec {
            if let Some(reste) = s.strip_prefix(r) {
                if let Some(v) = rec(reste, h, j + 1, max, l, syll) {
                    return Some(v);
                }
            }
        }
        if max == 0 {
            return None;
        }
        for k in 1..=6.min(s.len()) {
            if syll.contains(&s[..k]) {
                if let Some(mut v) = rec(&s[k..], h, j + 1, max - 1, l, syll) {
                    v.insert(0, (j, s[..k].to_string()));
                    return Some(v);
                }
            }
        }
        None
    }
    rec(s, h, 0, max, l, syll)
}

fn sinogrammes(car: &[char], a: &mut Vec<AlerteGrammaticale>) {
    let (l, _) = lectures();
    // Couples pinyin → caractères vérifiés dans le document : ils servent de suggestions
    let mut connus: HashMap<String, String> = HashMap::new();
    let mut douteux = Vec::new();
    let mut i = 0;
    while i < car.len() {
        if !(est_cjk(car[i]) && l.contains_key(&car[i])) {
            i += 1;
            continue;
        }
        let d = i;
        while i < car.len() && est_cjk(car[i]) && l.contains_key(&car[i]) {
            i += 1;
        }
        let h = &car[d..i];
        if i < car.len() && est_cjk(car[i]) {
            continue; // caractère inconnu de la table : on ne juge pas
        }
        // Mots latins juste avant (« Taiping Tianguo 天平天国 », « Zou (邹) »)
        let mut k = d;
        while k > 0 && matches!(car[k - 1], ' ' | '(' | '\u{a0}') {
            k -= 1;
        }
        let mut latins: Vec<String> = Vec::new();
        while k > 0 && latins.len() < 14 {
            let f = k;
            while k > 0 && (car[k - 1].is_alphabetic() && !est_cjk(car[k - 1]) || matches!(car[k - 1], '\'' | '’' | '-')) {
                k -= 1;
            }
            if k == f {
                break;
            }
            let w = normalise(&car[k..f].iter().collect::<String>());
            if w.is_empty() || segmente(&w).is_none() {
                break;
            }
            latins.insert(0, w);
            while k > 0 && matches!(car[k - 1], ' ' | '–' | '·' | '\u{a0}') {
                k -= 1;
            }
        }
        if latins.is_empty() {
            continue;
        }
        let suffixe = |n: usize| latins[latins.len() - n..].concat();
        if let Some(n) = (1..=latins.len()).find(|&n| aligne(&suffixe(n), h, 0).is_some()) {
            connus.insert(suffixe(n), h.iter().collect());
            continue;
        }
        if let Some((n, v)) = (1..=latins.len()).rev().find_map(|n| aligne(&suffixe(n), h, 1).map(|v| (n, v))) {
            if h.len() >= 2 && v.len() == 1 {
                douteux.push((d, h.to_vec(), suffixe(n), v[0].clone()));
            }
        }
    }
    for (d, h, pinyin, (j, lu)) in douteux {
        let c = h[j];
        let lec = l.get(&c).map(|v| v.join(", ")).unwrap_or_default();
        // « taipingtianguo » vérifié ailleurs, ou en tête d'un terme plus long (« taipingtianguoyundong »)
        let sugg: Vec<String> = connus
            .iter()
            .filter(|(p, _)| p.starts_with(&pinyin))
            .map(|(_, hz)| hz.chars().take(h.len()).collect::<String>())
            .filter(|s| aligne(&pinyin, &s.chars().collect::<Vec<_>>(), 0).is_some())
            .take(1)
            .collect();
        a.push(alerte(d, d + h.len(), h.iter().collect(), REGLE_SINOGRAMME,
            format!("Le pinyin donne « {lu} » mais « {c} » se lit {lec} : soit le caractère est erroné,                      soit le pinyin doit porter « {lec} »."), sugg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regles(t: &str) -> Vec<(String, Vec<String>)> {
        analyse(t).into_iter().map(|a| (a.texte_fautif, a.suggestions)).collect()
    }

    #[test]
    fn tournures_du_livre() {
        let r = regles("Chacun de mes enfants et de mes petits-enfants doivent perpétuer la pratique.");
        assert_eq!(r, vec![("doivent".into(), vec!["doit".into()])]);
        assert_eq!(regles("une famille qui n’arrive pas trouver l’équilibre")[0].1, vec!["pas à"]);
        assert_eq!(regles("Huaxi fit construit à l’est la résidence")[0].1, vec!["construire"]);
        assert_eq!(regles("La méthode est la seule que nous ne pouvons adopter.")[0].1, vec!["puissions"]);
        assert_eq!(regles("une avait entre 15 à 19 ans")[0].1, vec!["et"]);
        assert_eq!(regles("sans privilégier son intérêt, ni sans faire des opérations")[0].1, vec!["ni"]);
        assert_eq!(regles("On dit que « un petit commerçant")[0].1, vec!["qu'"]);
        assert_eq!(regles("le préfet finit bar baisser les bras")[0].1, vec!["par"]);
        assert_eq!(regles("C’est quand-même épouvantable.")[0].1, vec!["quand même"]);
        assert_eq!(regles("de 23 hommes et de12 femmes")[0].1, vec!["de 12"]);
        assert_eq!(regles("dans intérêt de tout le lignage")[0].1, vec!["dans l'intérêt"]);
        assert_eq!(regles("Pan Shi’en était comme étant le Pan de la rue")[0].1, vec!["connu comme"]);
        assert!(regles("Il fit appel à eux ; ni avec ni sans toi ; il fit pitié ; il arrive demain.").is_empty());
    }

    #[test]
    fn relecture_complete() {
        let s = |t: &str| regles(t).into_iter().flat_map(|r| r.1).collect::<Vec<_>>();
        assert_eq!(s("en l’an XXVI du règne de Kangxi (l683), le nombre"), vec!["1683", "1687"]);
        assert_eq!(s("Il y avait presqu’aucun lettré."), vec!["presque aucun"]);
        assert_eq!(s("Il vivait avec des grandes familles riches."), vec!["de"]);
        assert_eq!(s("la porte du hôte"), vec!["de l'hôte"]);
        assert_eq!(s("Veuillez-vous référer au tableau."), vec!["Veuillez vous"]);
        assert_eq!(s("Il vit en terme de richesse."), vec!["termes"]);
        assert_eq!(s("la nvzi du village"), vec!["nüzi"]);
        assert_eq!(s("le chiffre du le règne"), vec!["du"]);
        assert_eq!(s("Il fallut attendre pour que cette contrainte fut brisée."), vec!["fût"]);
        assert_eq!(regles("Bien qu’il soit riche, cependant il travaille.").len(), 1);
        assert_eq!(s("Il gagnait 10.7 % de plus."), vec!["10,7"]);
        assert_eq!(s("Pourquoi ?. Il partit."), vec!["?"]);
        assert_eq!(s("Il partit au marché. le soir, il revint."), vec!["Le"]);
        assert!(s("Voir cf. la note, etc. puis av. la fin, en 200 av. J.-C. la dynastie… et M. Wu vint").is_empty());
        assert_eq!(regles("Il récitait des textes taoïstes. » Puis il partit.").len(), 1);
        assert!(regles("Il dit : « Le maître a écrit « Wu » sur la porte. » Puis :
« Suite.
Fin. »").is_empty());
        assert_eq!(s("l’ enregistrement"), vec!["l’"]);
        // faux positifs écartés
        assert!(regles("des bons et des mauvais ; des petits pains ; le tome LVIII ; Guangxu (1797-1876).").is_empty());
        assert!(regles("ceci atteste bien que cette préfecture n’a qu’un vingtième de terre.").is_empty());
        assert!(regles("en l’an XXXI du règne de Guangxu (1905).").is_empty());
    }

    #[test]
    fn noms_et_sinogrammes() {
        let t = "Les Zou à Changshu. Ils étaient à Chansghu. Puis Changshu encore. Wang Zhengke, Wang Zhengke et Zhengke, puis Zhenke.";
        let r = regles(t);
        assert!(r.contains(&("Chansghu".into(), vec!["Changshu".into()])), "{r:?}");
        assert!(r.contains(&("Zhenke".into(), vec!["Zhengke".into()])), "{r:?}");
        let t = "(Taiping tianguo yundong 太平天国运动) puis (Taiping Tianguo 天平天国), et Zou (邹).";
        let r = regles(t);
        assert_eq!(r, vec![("天平天国".into(), vec!["太平天国".into()])]);
        assert!(regles("Liang et Jiang, Jiang encore. Les Tableaux, le Tableau, un Tableau.").is_empty());
        assert!(regles("Dans chacun des six districts existaient des maisons ; il fit ensuite construire.").is_empty());
    }
}

