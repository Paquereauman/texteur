//! Lecture/écriture minimale des .docx sans Word : un paragraphe Word = une ligne.
//! À l'enregistrement, seuls les paragraphes modifiés sont réécrits (leur texte va dans le premier run ; la mise en
//! forme des autres est conservée intacte). Les paragraphes ajoutés reprennent la mise en forme de leur voisin, les
//! paragraphes déplacés gardent la leur ; les titres (« # », « ## »… en début de ligne) suivent les styles de titre.

use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Range;
use std::path::Path;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use zip::write::SimpleFileOptions;

use crate::ecriture::titre;

const DOC: &str = "word/document.xml";
const STYLES: &str = "word/styles.xml";

fn lis_entree(chemin: &Path, nom: &str) -> Result<String, String> {
    let mut za = zip::ZipArchive::new(File::open(chemin).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let mut xml = String::new();
    za.by_name(nom).map_err(|e| e.to_string())?.read_to_string(&mut xml).map_err(|e| e.to_string())?;
    Ok(xml)
}

fn lis_document(chemin: &Path) -> Result<String, String> {
    lis_entree(chemin, DOC)
}

fn attribut(e: &BytesStart, nom: &str) -> Option<String> {
    e.try_get_attribute(nom).ok().flatten().and_then(|a| a.unescape_value().ok()).map(|v| v.into_owned())
}

fn val(e: &BytesStart) -> Option<String> {
    attribut(e, "w:val")
}

/// Niveau de titre (1 à 9) des styles de paragraphe qui en ont un : niveau de plan du style, sinon son nom
/// (« heading 2 », « Title », « Titre 3 lettres »), sinon celui du style dont il hérite. Aussi, pour chaque niveau,
/// le style de titre standard de Word (« heading N », « Title »), s'il est défini.
fn niveaux_styles(xml: &str) -> (HashMap<String, u8>, HashMap<u8, String>) {
    #[derive(Default)]
    struct S {
        nom: String,
        base: Option<String>,
        plan: Option<u8>,
    }
    let mut styles: HashMap<String, S> = HashMap::new();
    let mut courant: Option<String> = None;
    let mut r = Reader::from_str(xml);
    loop {
        let (e, debut) = match r.read_event() {
            Ok(Event::Start(e)) => (e, true),
            Ok(Event::Empty(e)) => (e, false),
            Ok(Event::End(e)) if e.name().as_ref() == b"w:style" => {
                courant = None;
                continue;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => continue,
        };
        if e.name().as_ref() == b"w:style" {
            courant = attribut(&e, "w:styleId").filter(|_| debut && attribut(&e, "w:type").as_deref() == Some("paragraph"));
            if let Some(id) = &courant {
                styles.entry(id.clone()).or_default();
            }
            continue;
        }
        let Some(s) = courant.as_ref().and_then(|id| styles.get_mut(id)) else { continue };
        match e.name().as_ref() {
            b"w:name" => s.nom = val(&e).unwrap_or_default().to_lowercase(),
            b"w:basedOn" => s.base = val(&e),
            b"w:outlineLvl" => s.plan = val(&e).and_then(|v| v.parse::<u8>().ok()).map(|n| if n < 9 { n + 1 } else { 0 }),
            _ => {}
        }
    }
    fn niveau(styles: &HashMap<String, S>, id: &str, prof: u8) -> u8 {
        let Some(s) = styles.get(id) else { return 0 };
        let chiffre = |reste: &str| reste.trim_start().chars().next().and_then(|c| c.to_digit(10)).filter(|&d| d >= 1).map(|d| d as u8);
        s.plan
            .or_else(|| (s.nom == "title").then_some(1))
            .or_else(|| s.nom.strip_prefix("heading").or_else(|| s.nom.strip_prefix("titre")).and_then(chiffre))
            .unwrap_or_else(|| match &s.base {
                Some(b) if prof < 8 => niveau(styles, b, prof + 1),
                _ => 0,
            })
    }
    let niveaux: HashMap<String, u8> = styles.keys().map(|id| (id.clone(), niveau(&styles, id, 0))).filter(|(_, n)| *n > 0).collect();
    let mut standards = HashMap::new();
    for (id, s) in &styles {
        let n = if s.nom == "title" { 1 } else { s.nom.strip_prefix("heading ").and_then(|c| c.parse().ok()).unwrap_or(0) };
        if n > 0 && niveaux.get(id) == Some(&n) && (s.nom != "title" || !standards.contains_key(&1)) {
            standards.insert(n, id.clone());
        }
    }
    (niveaux, standards)
}

struct Para {
    texte: String,
    /// 0 = texte courant ; seuls les paragraphes non vides du corps (hors tableaux et zones de texte) sont des titres
    niveau: u8,
    style: Option<String>,
}

fn analyse(xml: &str, styles: &HashMap<String, u8>) -> Result<Vec<Para>, String> {
    let mut paras: Vec<Para> = Vec::new();
    let mut eligibles: Vec<bool> = Vec::new();
    let mut pile: Vec<usize> = Vec::new();
    let (mut dans_r, mut dans_t, mut tableaux) = (0usize, false, 0usize);
    let mut r = Reader::from_str(xml);
    loop {
        let ev = r.read_event().map_err(|e| e.to_string())?;
        if let Event::Start(e) | Event::Empty(e) = &ev {
            let debut = matches!(ev, Event::Start(_));
            match (e.name().as_ref(), pile.last().copied()) {
                (b"w:p", _) => {
                    eligibles.push(pile.is_empty() && tableaux == 0);
                    if debut {
                        pile.push(paras.len());
                    }
                    paras.push(Para { texte: String::new(), niveau: 0, style: None });
                }
                (b"w:tbl", _) if debut => tableaux += 1,
                (b"w:r", _) if debut => dans_r += 1,
                (b"w:t", _) if debut => dans_t = true,
                (b"w:tab", Some(i)) if !debut && dans_r > 0 => paras[i].texte.push('\t'),
                (b"w:br" | b"w:cr", Some(i)) if !debut && dans_r > 0 => paras[i].texte.push(' '),
                (b"w:pStyle", Some(i)) if eligibles[i] => {
                    paras[i].style = val(e);
                    paras[i].niveau = paras[i].style.as_ref().and_then(|s| styles.get(s)).copied().unwrap_or(0);
                }
                (b"w:outlineLvl", Some(i)) if eligibles[i] => {
                    let n = val(e).and_then(|v| v.parse::<u8>().ok()).unwrap_or(9);
                    paras[i].niveau = if n < 9 { n + 1 } else { 0 };
                }
                _ => {}
            }
            continue;
        }
        match ev {
            Event::End(e) => match e.name().as_ref() {
                b"w:p" => {
                    pile.pop();
                }
                b"w:tbl" => tableaux = tableaux.saturating_sub(1),
                b"w:r" => dans_r = dans_r.saturating_sub(1),
                b"w:t" => dans_t = false,
                _ => {}
            },
            Event::Text(t) if dans_t => {
                if let Some(&i) = pile.last() {
                    paras[i].texte.push_str(&t.unescape().map_err(|e| e.to_string())?);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    for p in paras.iter_mut().filter(|p| p.texte.trim().is_empty()) {
        p.niveau = 0;
    }
    Ok(paras)
}

fn paragraphes(xml: &str) -> Result<Vec<String>, String> {
    Ok(analyse(xml, &HashMap::new())?.into_iter().map(|p| p.texte).collect())
}

pub fn lis(chemin: &Path) -> Result<String, String> {
    Ok(paragraphes(&lis_document(chemin)?)?.join("\n"))
}

/// Comme `lis`, les titres précédés de leur niveau (« # Chapitre », « ## Partie ») pour le mode écriture.
pub fn lis_plan(chemin: &Path) -> Result<String, String> {
    let (styles, _) = niveaux_styles(&lis_entree(chemin, STYLES).unwrap_or_default());
    let paras = analyse(&lis_document(chemin)?, &styles)?;
    Ok(paras
        .iter()
        .map(|p| if p.niveau > 0 { format!("{} {}", "#".repeat(p.niveau as usize), p.texte) } else { p.texte.clone() })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Écrit le texte d'un paragraphe : les tabulations redeviennent des `<w:tab/>`.
fn ecris_texte(w: &mut Writer<Vec<u8>>, texte: &str) -> std::io::Result<()> {
    for (n, morceau) in texte.split('\t').enumerate() {
        if n > 0 {
            w.write_event(Event::End(BytesEnd::new("w:t")))?;
            w.write_event(Event::Empty(BytesStart::new("w:tab")))?;
            w.write_event(Event::Start(BytesStart::new("w:t").with_attributes([("xml:space", "preserve")])))?;
        }
        w.write_event(Event::Text(BytesText::new(morceau)))?;
    }
    Ok(())
}

/// Un run de texte, avec la mise en forme `rpr` (XML brut, éventuellement vide).
fn ecris_run(w: &mut Writer<Vec<u8>>, rpr: &str, texte: &str) -> std::io::Result<()> {
    w.write_event(Event::Start(BytesStart::new("w:r")))?;
    w.write_event(Event::Text(BytesText::from_escaped(rpr)))?;
    w.write_event(Event::Start(BytesStart::new("w:t").with_attributes([("xml:space", "preserve")])))?;
    ecris_texte(w, texte)?;
    w.write_event(Event::End(BytesEnd::new("w:t")))?;
    w.write_event(Event::End(BytesEnd::new("w:r")))
}

/// Même nombre de paragraphes : réécrit en place le texte de ceux qui ont changé.
fn reecris(xml: &str, lignes: &[&str]) -> Result<Vec<u8>, String> {
    let orig = paragraphes(xml)?;
    if orig.len() != lignes.len() {
        return Err(format!("{} paragraphes pour {} lignes.", orig.len(), lignes.len()));
    }
    let change: Vec<bool> = orig.iter().zip(lignes).map(|(a, b)| a != b).collect();
    let mut ecrit = vec![false; orig.len()];
    let mut r = Reader::from_str(xml);
    let mut w = Writer::new(Vec::new());
    let (mut idx, mut pile, mut dans_r) = (0usize, Vec::<usize>::new(), 0usize);
    let mut saute_texte = false;
    let err = |e: std::io::Error| e.to_string();
    loop {
        let ev = r.read_event().map_err(|e| e.to_string())?;
        let courant = pile.last().copied().filter(|&i| change[i]);
        match &ev {
            Event::Start(e) => match e.name().as_ref() {
                b"w:p" => {
                    pile.push(idx);
                    idx += 1;
                }
                b"w:r" => dans_r += 1,
                b"w:t" if courant.is_some() => {
                    let i = courant.unwrap();
                    let mut b = BytesStart::new("w:t");
                    b.push_attribute(("xml:space", "preserve"));
                    w.write_event(Event::Start(b)).map_err(err)?;
                    if !ecrit[i] {
                        ecrit[i] = true;
                        ecris_texte(&mut w, lignes[i]).map_err(err)?;
                    }
                    saute_texte = true;
                    continue;
                }
                _ => {}
            },
            Event::Empty(e) => match e.name().as_ref() {
                b"w:p" => {
                    idx += 1;
                    if change[idx - 1] && !lignes[idx - 1].is_empty() {
                        w.write_event(Event::Start(e.to_owned())).map_err(err)?;
                        ecris_run(&mut w, "", lignes[idx - 1]).map_err(err)?;
                        w.write_event(Event::End(BytesEnd::new("w:p"))).map_err(err)?;
                        continue;
                    }
                }
                b"w:tab" | b"w:br" | b"w:cr" if dans_r > 0 && courant.is_some() => continue,
                _ => {}
            },
            Event::End(e) => match e.name().as_ref() {
                b"w:p" => {
                    if let Some(i) = courant.filter(|&i| !ecrit[i] && !lignes[i].is_empty()) {
                        // paragraphe vide devenu non vide : on lui ajoute un run
                        ecrit[i] = true;
                        ecris_run(&mut w, "", lignes[i]).map_err(err)?;
                    }
                    pile.pop();
                }
                b"w:r" => dans_r = dans_r.saturating_sub(1),
                b"w:t" => saute_texte = false,
                _ => {}
            },
            Event::Text(_) | Event::CData(_) if saute_texte => continue,
            Event::Eof => break,
            _ => {}
        }
        w.write_event(ev).map_err(err)?;
    }
    Ok(w.into_inner())
}

/// Un enfant direct du corps (`w:body`) : sa plage dans le XML (espaces qui le précèdent compris) et ses lignes.
struct Bloc {
    plage: Range<usize>,
    lignes: Range<usize>,
    /// paragraphe ordinaire (une seule ligne) : peut être déplacé, supprimé, servir de modèle
    simple: bool,
}

/// Les blocs du corps, la position du début du corps et celle de la fin du dernier bloc.
fn blocs(xml: &str) -> Result<(Vec<Bloc>, usize, usize), String> {
    let mut r = Reader::from_str(xml);
    let (mut prof, mut corps, mut debut_corps) = (0usize, None::<usize>, 0usize);
    let (mut lignes, mut debut_bloc, mut ligne_bloc, mut nom_bloc) = (0usize, 0usize, 0usize, Vec::new());
    let mut res = Vec::new();
    loop {
        let ev = r.read_event().map_err(|e| e.to_string())?;
        let pos = r.buffer_position() as usize;
        match &ev {
            Event::Start(e) => {
                let avant = lignes;
                lignes += (e.name().as_ref() == b"w:p") as usize;
                if corps == Some(prof) {
                    (ligne_bloc, nom_bloc) = (avant, e.name().as_ref().to_vec());
                }
                prof += 1;
                if e.name().as_ref() == b"w:body" && corps.is_none() {
                    (corps, debut_corps, debut_bloc) = (Some(prof), pos, pos);
                }
            }
            Event::Empty(e) => {
                let avant = lignes;
                let p = e.name().as_ref() == b"w:p";
                lignes += p as usize;
                if corps == Some(prof) {
                    res.push(Bloc { plage: debut_bloc..pos, lignes: avant..lignes, simple: p });
                    debut_bloc = pos;
                }
            }
            Event::End(e) => {
                prof = prof.saturating_sub(1);
                if corps == Some(prof + 1) && e.name().as_ref() == b"w:body" {
                    return Ok((res, debut_corps, debut_bloc));
                }
                if corps == Some(prof) {
                    let simple = nom_bloc == b"w:p" && lignes - ligne_bloc == 1;
                    res.push(Bloc { plage: debut_bloc..pos, lignes: ligne_bloc..lignes, simple });
                    debut_bloc = pos;
                }
            }
            Event::Eof => return Err("Document Word sans corps (w:body).".into()),
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    /// ancienne ligne, nouvelle ligne (identique ou modifiée)
    Garde(usize, usize),
    Suppr(usize),
    Ajout(usize),
}

/// Au-delà (plus de lignes ajoutées et supprimées), l'alignement fin coûterait trop de mémoire (~D² entiers).
const MAX_DIFFERENCES: isize = 3000;

/// Alignement minimal ligne à ligne (diff de Myers). Une ligne supprimée suivie d'une ligne ajoutée devient une ligne
/// modifiée, sauf si l'une des deux a été déplacée (son texte existe ailleurs dans l'autre version).
fn aligne<T: Eq + std::hash::Hash>(a: &[T], b: &[T]) -> Vec<Op> {
    let pre = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suf = a[pre..].iter().rev().zip(b[pre..].iter().rev()).take_while(|(x, y)| x == y).count();
    let (x, y) = (&a[pre..a.len() - suf], &b[pre..b.len() - suf]);
    let (n, m) = (x.len() as isize, y.len() as isize);
    let max = (n + m).min(MAX_DIFFERENCES);
    let off = max + 1;
    let mut v = vec![0isize; 2 * max as usize + 3];
    let mut traces: Vec<Vec<i32>> = Vec::new();
    let mut trouve = false;
    'd: for d in 0..=max {
        traces.push(v[(off - d) as usize..=(off + d) as usize].iter().map(|&x| x as i32).collect());
        for k in (-d..=d).step_by(2) {
            let mut i = if k == -d || (k != d && v[(off + k - 1) as usize] < v[(off + k + 1) as usize]) {
                v[(off + k + 1) as usize]
            } else {
                v[(off + k - 1) as usize] + 1
            };
            let mut j = i - k;
            while i < n && j < m && x[i as usize] == y[j as usize] {
                (i, j) = (i + 1, j + 1);
            }
            v[(off + k) as usize] = i;
            if i >= n && j >= m {
                trouve = true;
                break 'd;
            }
        }
    }
    let mut ops: Vec<Op> = (0..pre).map(|i| Op::Garde(i, i)).collect();
    let fin: Vec<Op> = (0..suf).map(|s| Op::Garde(a.len() - suf + s, b.len() - suf + s)).collect();
    if !trouve {
        // très gros remaniement : tout le milieu supprimé puis ajouté (les déplacements restent reconnus)
        ops.extend((pre..pre + n as usize).map(Op::Suppr));
        ops.extend((pre..pre + m as usize).map(Op::Ajout));
        ops.extend(fin);
        return ops;
    }
    let mut milieu = Vec::new();
    {
        let (mut i, mut j) = (n, m);
        for (d, t) in traces.iter().enumerate().rev() {
            let d = d as isize;
            let (pi, pj) = if d == 0 {
                (0, 0)
            } else {
                let k = i - j;
                let at = |k: isize| t[(k + d) as usize] as isize;
                let pk = if k == -d || (k != d && at(k - 1) < at(k + 1)) { k + 1 } else { k - 1 };
                (at(pk), at(pk) - pk)
            };
            while i > pi && j > pj {
                (i, j) = (i - 1, j - 1);
                milieu.push(Op::Garde(i as usize, j as usize));
            }
            if d > 0 {
                milieu.push(if i == pi { Op::Ajout((j - 1) as usize) } else { Op::Suppr((i - 1) as usize) });
            }
            (i, j) = (pi, pj);
        }
        milieu.reverse();
    }
    let ajoutes: std::collections::HashSet<&T> = milieu.iter().filter_map(|o| if let Op::Ajout(j) = o { Some(&y[*j]) } else { None }).collect();
    let supprimes: std::collections::HashSet<&T> = milieu.iter().filter_map(|o| if let Op::Suppr(i) = o { Some(&x[*i]) } else { None }).collect();
    // dans une suite de suppressions et d'ajouts, les premiers de chaque se correspondent (paragraphes modifiés)
    let mut k = 0;
    while k < milieu.len() {
        if let Op::Garde(i, j) = milieu[k] {
            ops.push(Op::Garde(i + pre, j + pre));
            k += 1;
            continue;
        }
        let fin = milieu[k..].iter().position(|o| matches!(o, Op::Garde(..))).map_or(milieu.len(), |p| k + p);
        let run = &milieu[k..fin];
        let sup: Vec<usize> = run.iter().filter_map(|o| if let Op::Suppr(i) = o { Some(*i) } else { None }).collect();
        let aj: Vec<usize> = run.iter().filter_map(|o| if let Op::Ajout(j) = o { Some(*j) } else { None }).collect();
        let mut libres = sup.iter().copied().filter(|&i| !ajoutes.contains(&x[i])).peekable();
        ops.extend(sup.iter().filter(|&&i| ajoutes.contains(&x[i])).map(|&i| Op::Suppr(i + pre)));
        let mut paires = Vec::new();
        for &j in &aj {
            match libres.peek() {
                Some(&i) if !supprimes.contains(&y[j]) => {
                    libres.next();
                    paires.push(Op::Garde(i + pre, j + pre));
                }
                _ => paires.push(Op::Ajout(j + pre)),
            }
        }
        ops.extend(libres.map(|i| Op::Suppr(i + pre)));
        ops.extend(paires);
        k = fin;
    }
    ops.extend(fin);
    ops
}

/// Ce qui fait d'un paragraphe un titre de tel niveau.
enum Marque {
    Aucune,
    Style(String),
    Plan(u8),
}

/// Réécrit les propriétés d'un paragraphe (`w:pPr`) : retire son style de titre et son niveau de plan, puis pose `m`
/// (un style en tête de `w:pPr`, un niveau de plan avant `w:rPr`/`w:sectPr`, comme le veut le schéma Word).
fn avec_marque(p: &str, m: &Marque, titres: &HashMap<String, u8>) -> Result<String, String> {
    let err = |e: std::io::Error| e.to_string();
    let ecris_marque = |w: &mut Writer<Vec<u8>>| -> std::io::Result<()> {
        match m {
            Marque::Style(s) => w.write_event(Event::Empty(BytesStart::new("w:pStyle").with_attributes([("w:val", s.as_str())]))),
            Marque::Plan(n) => {
                let v = (n - 1).to_string();
                w.write_event(Event::Empty(BytesStart::new("w:outlineLvl").with_attributes([("w:val", v.as_str())])))
            }
            Marque::Aucune => Ok(()),
        }
    };
    let ppr_complet = |w: &mut Writer<Vec<u8>>| -> std::io::Result<()> {
        if matches!(m, Marque::Aucune) {
            return Ok(());
        }
        w.write_event(Event::Start(BytesStart::new("w:pPr")))?;
        ecris_marque(w)?;
        w.write_event(Event::End(BytesEnd::new("w:pPr")))
    };
    let plan = matches!(m, Marque::Plan(_));
    let mut r = Reader::from_str(p);
    let mut w = Writer::new(Vec::new());
    // prof : éléments ouverts (1 = dans w:p, 2 = dans w:pPr)
    let (mut prof, mut dans_ppr, mut fait, mut saute) = (0usize, false, false, 0usize);
    loop {
        let ev = r.read_event().map_err(|e| e.to_string())?;
        if saute > 0 {
            match ev {
                Event::Start(_) => saute += 1,
                Event::End(_) => saute -= 1,
                _ => {}
            }
            continue;
        }
        match &ev {
            Event::Start(e) | Event::Empty(e) => {
                let vide = matches!(ev, Event::Empty(_));
                let nom = e.name().as_ref().to_vec();
                if prof == 0 && nom == b"w:p" && vide {
                    w.write_event(Event::Start(e.to_owned())).map_err(err)?;
                    ppr_complet(&mut w).map_err(err)?;
                    w.write_event(Event::End(BytesEnd::new("w:p"))).map_err(err)?;
                    continue;
                }
                if prof == 1 && !fait {
                    fait = true;
                    if nom == b"w:pPr" {
                        if vide {
                            ppr_complet(&mut w).map_err(err)?;
                            continue;
                        }
                        w.write_event(ev.clone()).map_err(err)?;
                        if matches!(m, Marque::Style(_)) {
                            ecris_marque(&mut w).map_err(err)?;
                        }
                        dans_ppr = true;
                        prof += 1;
                        continue;
                    }
                    ppr_complet(&mut w).map_err(err)?;
                }
                if dans_ppr && prof == 2 {
                    let style = nom == b"w:pStyle";
                    let style_titre = style && val(e).is_some_and(|s| titres.contains_key(&s));
                    if nom == b"w:outlineLvl" || style_titre || style && matches!(m, Marque::Style(_)) {
                        saute = (!vide) as usize;
                        continue;
                    }
                    if plan && matches!(nom.as_slice(), b"w:rPr" | b"w:sectPr" | b"w:pPrChange") {
                        ecris_marque(&mut w).map_err(err)?;
                        dans_ppr = false;
                    }
                }
                if !vide {
                    prof += 1;
                }
            }
            Event::End(e) => {
                if prof == 2 && dans_ppr && e.name().as_ref() == b"w:pPr" {
                    if plan {
                        ecris_marque(&mut w).map_err(err)?;
                    }
                    dans_ppr = false;
                }
                if prof == 1 && !fait {
                    fait = true;
                    ppr_complet(&mut w).map_err(err)?;
                }
                prof = prof.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
        w.write_event(ev).map_err(err)?;
    }
    String::from_utf8(w.into_inner()).map_err(|e| e.to_string())
}

/// Propriétés (`w:pPr`) et mise en forme du premier run (`w:rPr`) d'un paragraphe modèle, sans saut de section ni
/// révisions : de quoi créer un paragraphe voisin qui lui ressemble.
fn proprietes(p: &str) -> (String, String) {
    let mut r = Reader::from_str(p);
    let (mut ppr, mut rpr) = (Writer::new(Vec::new()), Writer::new(Vec::new()));
    // dans : 1 = pPr du paragraphe, 2 = rPr du premier run
    let (mut prof, mut saute, mut dans, mut run_vu, mut ppr_vu, mut rpr_vu) = (0usize, 0usize, 0u8, false, false, false);
    loop {
        let Ok(ev) = r.read_event() else { break };
        if saute > 0 {
            match ev {
                Event::Start(_) => saute += 1,
                Event::End(_) => saute -= 1,
                _ => {}
            }
            continue;
        }
        let nom = match &ev {
            Event::Start(e) | Event::Empty(e) => e.name().as_ref().to_vec(),
            Event::End(e) => e.name().as_ref().to_vec(),
            Event::Eof => break,
            _ => Vec::new(),
        };
        if dans > 0 && matches!(nom.as_slice(), b"w:sectPr" | b"w:pPrChange" | b"w:rPrChange") {
            saute = matches!(ev, Event::Start(_)) as usize;
            continue;
        }
        match &ev {
            Event::Start(_) => {
                if prof == 1 && nom == b"w:pPr" && !ppr_vu {
                    dans = 1;
                } else if nom == b"w:r" && dans == 0 {
                    run_vu = true;
                } else if nom == b"w:rPr" && run_vu && dans == 0 && !rpr_vu {
                    dans = 2;
                }
                prof += 1;
            }
            Event::End(_) => prof = prof.saturating_sub(1),
            _ => {}
        }
        let _ = match dans {
            1 => ppr.write_event(ev.clone()),
            2 => rpr.write_event(ev.clone()),
            _ => Ok(()),
        };
        if matches!(ev, Event::End(_)) && (dans == 1 && nom == b"w:pPr" || dans == 2 && nom == b"w:rPr") {
            ppr_vu |= dans == 1;
            rpr_vu |= dans == 2;
            dans = 0;
        }
    }
    let s = |w: Writer<Vec<u8>>| String::from_utf8(w.into_inner()).unwrap_or_default();
    (s(ppr), s(rpr))
}

fn nouveau_paragraphe(ppr: &str, rpr: &str, texte: &str) -> Result<String, String> {
    let err = |e: std::io::Error| e.to_string();
    let mut w = Writer::new(Vec::new());
    w.write_event(Event::Start(BytesStart::new("w:p"))).map_err(err)?;
    w.write_event(Event::Text(BytesText::from_escaped(ppr))).map_err(err)?;
    if !texte.is_empty() {
        ecris_run(&mut w, rpr, texte).map_err(err)?;
    }
    w.write_event(Event::End(BytesEnd::new("w:p"))).map_err(err)?;
    String::from_utf8(w.into_inner()).map_err(|e| e.to_string())
}

/// Nouveau document.xml pour `lignes` (texte de l'éditeur, titres marqués « # ») : paragraphes gardés, modifiés,
/// ajoutés, supprimés ou déplacés. Les tableaux et zones de texte peuvent être modifiés, pas gagner ou perdre de lignes.
fn restructure(xml: &str, styles_xml: &str, lignes: &[&str]) -> Result<Vec<u8>, String> {
    let (titres, standards) = niveaux_styles(styles_xml);
    let orig = analyse(xml, &titres)?;
    let anc: Vec<(u8, &str)> = orig.iter().map(|p| (p.niveau, p.texte.as_str())).collect();
    let nouv: Vec<(u8, &str)> = lignes.iter().map(|l| titre(l).unwrap_or((0, l))).collect();
    let (blocs, debut_corps, fin_blocs) = blocs(xml)?;
    if blocs.last().map_or(0, |b| b.lignes.end) != orig.len() {
        return Err("Structure du document Word non reconnue : enregistrez sous un autre format.".into());
    }
    let mut bloc_de = vec![0usize; orig.len()];
    for (k, b) in blocs.iter().enumerate() {
        bloc_de[b.lignes.clone()].fill(k);
    }
    // style donné à chaque niveau de titre : le plus employé dans ce document, sinon un style de titre défini
    let mut usages: HashMap<(u8, &str), usize> = HashMap::new();
    for p in orig.iter().filter(|p| p.niveau > 0) {
        if let Some(s) = p.style.as_deref().filter(|s| titres.get(*s) == Some(&p.niveau)) {
            *usages.entry((p.niveau, s)).or_default() += 1;
        }
    }
    let marque = |n: u8| -> Marque {
        if n == 0 {
            return Marque::Aucune;
        }
        let doc = usages.iter().filter(|((m, _), _)| *m == n).max_by_key(|((_, s), c)| (**c, std::cmp::Reverse(*s))).map(|((_, s), _)| *s);
        let defini = || titres.iter().filter(|(_, m)| **m == n).map(|(s, _)| s.as_str()).min();
        match doc.or_else(|| standards.get(&n).map(String::as_str)).or_else(defini) {
            Some(s) => Marque::Style(s.to_string()),
            None => Marque::Plan(n),
        }
    };
    let ops = aligne(&anc, &nouv);
    let mut nouvelle_de: Vec<Option<usize>> = vec![None; orig.len()];
    let mut deplaces: HashMap<(u8, &str), VecDeque<usize>> = HashMap::new();
    for op in &ops {
        match *op {
            Op::Garde(i, j) => nouvelle_de[i] = Some(j),
            Op::Suppr(i) if blocs[bloc_de[i]].simple => deplaces.entry(anc[i]).or_default().push_back(i),
            _ => {}
        }
    }
    // tableaux et zones de texte entièrement supprimés puis réinsérés à l'identique ailleurs : déplacés tels quels
    let mut figes: HashMap<(u8, &str), VecDeque<usize>> = HashMap::new();
    for (k, b) in blocs.iter().enumerate() {
        if !b.simple && !b.lignes.is_empty() && b.lignes.clone().all(|l| nouvelle_de[l].is_none()) {
            figes.entry(anc[b.lignes.start]).or_default().push_back(k);
        }
    }
    let mut repris: HashMap<usize, usize> = HashMap::new(); // position dans ops -> bloc repris
    let mut x = 0;
    while x < ops.len() {
        let Op::Ajout(j) = ops[x] else {
            x += 1;
            continue;
        };
        let convient = |k: usize| {
            let l = &blocs[k].lignes;
            (0..l.len()).all(|t| ops.get(x + t) == Some(&Op::Ajout(j + t)) && nouv.get(j + t) == Some(&anc[l.start + t]))
        };
        match figes.get_mut(&nouv[j]).and_then(|q| q.iter().position(|&k| convient(k)).and_then(|p| q.remove(p))) {
            Some(k) => {
                repris.insert(x, k);
                x += blocs[k].lignes.len();
            }
            None => x += 1,
        }
    }
    let deplace: std::collections::HashSet<usize> = repris.values().copied().collect();
    let fige = "Les tableaux et zones de texte du document Word ne peuvent ni gagner ni perdre de lignes : \
                annulez l'ajout ou la suppression de ligne à cet endroit.";
    let frag = |k: usize| &xml[blocs[k].plage.clone()];
    let vers_texte = |o: Vec<u8>| String::from_utf8(o).map_err(|e| e.to_string());
    let mut sortie = String::with_capacity(xml.len() + 4096);
    sortie.push_str(&xml[..debut_corps]);
    let mut prochain = 0usize;
    let mut ancre: Option<usize> = None;
    // dernier paragraphe émis de chaque niveau : modèle des paragraphes ajoutés
    let mut modeles: HashMap<u8, usize> = HashMap::new();
    let mut dernier: Option<(usize, u8)> = None;
    let emet_vides = |sortie: &mut String, prochain: &mut usize, jusqua: usize| {
        while *prochain < jusqua {
            if blocs[*prochain].lignes.is_empty() {
                sortie.push_str(frag(*prochain));
            }
            *prochain += 1;
        }
    };
    let mut x = 0;
    while x < ops.len() {
        let op = ops[x];
        x += 1;
        if let Some(&k) = repris.get(&(x - 1)) {
            sortie.push_str(frag(k));
            x += blocs[k].lignes.len() - 1;
            ancre = Some(blocs[k].lignes.end - 1);
            continue;
        }
        match op {
            Op::Garde(i, j) => {
                let k = bloc_de[i];
                let b = &blocs[k];
                if b.simple {
                    emet_vides(&mut sortie, &mut prochain, k);
                    let mut p = if anc[i].1 == nouv[j].1 { frag(k).to_string() } else { vers_texte(reecris(frag(k), &[nouv[j].1])?)? };
                    if anc[i].0 != nouv[j].0 {
                        p = avec_marque(&p, &marque(nouv[j].0), &titres)?;
                    }
                    sortie.push_str(&p);
                    prochain = prochain.max(k + 1);
                    modeles.insert(nouv[j].0, k);
                    dernier = Some((k, nouv[j].0));
                } else if i == b.lignes.start {
                    emet_vides(&mut sortie, &mut prochain, k);
                    let js = b.lignes.clone().map(|l| nouvelle_de[l]).collect::<Option<Vec<usize>>>().ok_or(fige)?;
                    if js.windows(2).any(|w| w[1] != w[0] + 1) {
                        return Err(fige.into());
                    }
                    if b.lignes.clone().zip(&js).all(|(l, &j)| anc[l].1 == nouv[j].1) {
                        sortie.push_str(frag(k));
                    } else {
                        let textes: Vec<&str> = js.iter().map(|&j| nouv[j].1).collect();
                        sortie.push_str(&vers_texte(reecris(frag(k), &textes)?)?);
                    }
                    prochain = prochain.max(k + 1);
                }
                ancre = Some(i);
            }
            Op::Suppr(i) => {
                let k = bloc_de[i];
                if !blocs[k].simple && !deplace.contains(&k) {
                    return Err(fige.into());
                }
                emet_vides(&mut sortie, &mut prochain, k);
                prochain = prochain.max(k + 1);
                ancre = Some(i);
            }
            Op::Ajout(j) => {
                if let Some(a) = ancre {
                    let b = &blocs[bloc_de[a]];
                    if !b.simple && a + 1 != b.lignes.end {
                        return Err(fige.into());
                    }
                }
                let n = nouv[j].0;
                if let Some(i) = deplaces.get_mut(&nouv[j]).and_then(|q| q.pop_front()) {
                    // paragraphe déplacé : repris tel quel, mise en forme comprise
                    sortie.push_str(frag(bloc_de[i]));
                    modeles.insert(n, bloc_de[i]);
                    dernier = Some((bloc_de[i], n));
                    continue;
                }
                let premier = blocs.iter().position(|b| b.simple).map(|k| (k, anc[blocs[k].lignes.start].0));
                let (k, niveau_modele) =
                    modeles.get(&n).map(|&k| (k, n)).or(dernier).or(premier).ok_or("Document Word sans paragraphe.")?;
                let (ppr, rpr) = proprietes(frag(k));
                // un titre créé d'après du texte courant (ou l'inverse) prend la mise en forme de son style, pas du voisin
                let mut p = nouveau_paragraphe(&ppr, if niveau_modele == n { &rpr } else { "" }, nouv[j].1)?;
                if niveau_modele != n {
                    p = avec_marque(&p, &marque(n), &titres)?;
                }
                sortie.push_str(&p);
            }
        }
    }
    emet_vides(&mut sortie, &mut prochain, blocs.len());
    sortie.push_str(&xml[fin_blocs..]);
    // garde-fou : le document produit doit se relire exactement comme le texte de l'éditeur
    let relu = analyse(&sortie, &titres)?;
    let attendu = nouv.iter().map(|&(n, t)| (if t.trim().is_empty() { 0 } else { n }, t));
    if relu.len() != nouv.len() || relu.iter().zip(attendu).any(|(p, (n, t))| p.texte != t || p.niveau != n) {
        return Err("Enregistrement .docx annulé (rien n'a été modifié) : le document produit ne correspondait pas au texte.".into());
    }
    Ok(sortie.into_bytes())
}

/// Enregistre `texte` dans `dest`, en partant du .docx `source` (peut être le même fichier).
pub fn ecris(source: &Path, dest: &Path, texte: &str) -> Result<(), String> {
    let lignes: Vec<&str> = texte.split('\n').collect();
    let nouveau = restructure(&lis_document(source)?, &lis_entree(source, STYLES).unwrap_or_default(), &lignes)?;
    let tmp = dest.with_extension("docx.tmp");
    {
        let mut za = zip::ZipArchive::new(File::open(source).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let mut zw = zip::ZipWriter::new(File::create(&tmp).map_err(|e| e.to_string())?);
        for i in 0..za.len() {
            let nom = za.by_index_raw(i).map_err(|e| e.to_string())?.name().to_string();
            if nom == DOC {
                zw.start_file(DOC, SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated))
                    .map_err(|e| e.to_string())?;
                zw.write_all(&nouveau).map_err(|e| e.to_string())?;
            } else {
                zw.raw_copy_file(za.by_index_raw(i).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            }
        }
        zw.finish().map_err(|e| e.to_string())?;
    }
    std::fs::rename(&tmp, dest).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{e} (le fichier est-il ouvert dans Word ?)")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<w:document><w:body><w:p><w:pPr><w:tabs><w:tab w:val="left"/></w:tabs></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Je vous </w:t></w:r><w:r><w:t>envoit</w:t><w:tab/><w:t>長城</w:t></w:r></w:p><w:p/><w:p><w:r><w:t>Intact &amp; gras</w:t></w:r></w:p></w:body></w:document>"#;

    const STY: &str = r#"<w:styles><w:style w:type="paragraph" w:styleId="Titre"><w:name w:val="Title"/></w:style><w:style w:type="paragraph" w:styleId="T2"><w:name w:val="Titre 2 Romain"/><w:basedOn w:val="Titre2"/></w:style><w:style w:type="paragraph" w:styleId="Titre2"><w:name w:val="heading 2"/><w:pPr><w:outlineLvl w:val="1"/></w:pPr></w:style><w:style w:type="paragraph" w:styleId="Corps"><w:name w:val="Body Text"/></w:style></w:styles>"#;

    const LIVRE: &str = r#"<w:document><w:body><w:p><w:pPr><w:pStyle w:val="Titre"/></w:pPr><w:r><w:t>Chapitre I</w:t></w:r></w:p><w:p><w:pPr><w:jc w:val="both"/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>Texte un.</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Case</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:pPr><w:pStyle w:val="Titre"/></w:pPr><w:r><w:t>Chapitre II</w:t></w:r></w:p><w:p><w:r><w:t>Texte deux.</w:t></w:r></w:p><w:sectPr/></w:body></w:document>"#;

    fn plan(xml: &str) -> Vec<String> {
        analyse(xml, &niveaux_styles(STY).0)
            .unwrap()
            .iter()
            .map(|p| if p.niveau > 0 { format!("{} {}", "#".repeat(p.niveau as usize), p.texte) } else { p.texte.clone() })
            .collect()
    }

    fn apres(lignes: &[&str]) -> String {
        String::from_utf8(restructure(LIVRE, STY, lignes).unwrap()).unwrap()
    }

    #[test]
    fn lit_paragraphes() {
        assert_eq!(paragraphes(XML).unwrap(), ["Je vous envoit\t長城", "", "Intact & gras"]);
    }

    #[test]
    fn reecrit_seulement_les_paragraphes_modifies() {
        let x = String::from_utf8(restructure(XML, "", &["Je vous envoie\t長城", "Nouveau", "Intact & gras"]).unwrap()).unwrap();
        assert_eq!(paragraphes(&x).unwrap(), ["Je vous envoie\t長城", "Nouveau", "Intact & gras"]);
        assert!(x.contains("<w:rPr><w:b/></w:rPr>"));
        assert!(x.contains(r#"<w:tabs><w:tab w:val="left"/></w:tabs>"#));
        assert!(x.contains("<w:p><w:r><w:t>Intact &amp; gras</w:t></w:r></w:p>"));
    }

    #[test]
    fn niveaux_des_styles() {
        let (n, standards) = niveaux_styles(STY);
        assert_eq!(standards[&2], "Titre2");
        assert_eq!((n["Titre"], n["T2"], n["Titre2"]), (1, 2, 2));
        assert!(!n.contains_key("Corps"));
        assert_eq!(plan(LIVRE), ["# Chapitre I", "Texte un.", "Case", "# Chapitre II", "Texte deux."]);
    }

    #[test]
    fn ajoute_et_supprime_des_paragraphes() {
        let x = apres(&["# Chapitre I", "Texte un.", "Suite ajoutée.", "Case", "# Chapitre II"]);
        assert_eq!(plan(&x), ["# Chapitre I", "Texte un.", "Suite ajoutée.", "Case", "# Chapitre II"]);
        // le paragraphe ajouté reprend la mise en forme de son voisin
        assert!(x.contains(r#"<w:p><w:pPr><w:jc w:val="both"/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t xml:space="preserve">Suite ajoutée.</w:t>"#));
        assert!(x.ends_with("<w:sectPr/></w:body></w:document>"));
    }

    #[test]
    fn deplace_un_chapitre_avec_sa_mise_en_forme() {
        let x = apres(&["# Chapitre II", "Texte deux.", "# Chapitre I", "Texte un.", "Case"]);
        assert_eq!(plan(&x), ["# Chapitre II", "Texte deux.", "# Chapitre I", "Texte un.", "Case"]);
        assert!(x.contains(r#"<w:r><w:rPr><w:i/></w:rPr><w:t>Texte un.</w:t>"#));
    }

    #[test]
    fn change_et_cree_des_titres() {
        let x = apres(&["## Chapitre I", "Texte un.", "Case", "# Chapitre II", "# Nouveau", "Texte deux."]);
        assert_eq!(plan(&x), ["## Chapitre I", "Texte un.", "Case", "# Chapitre II", "# Nouveau", "Texte deux."]);
        assert!(x.contains(r#"<w:pPr><w:pStyle w:val="Titre2"/></w:pPr><w:r><w:t>Chapitre I</w:t>"#));
        let x = apres(&["Chapitre I", "Texte un.", "Case", "# Chapitre II", "Texte deux."]);
        assert_eq!(plan(&x)[0], "Chapitre I");
    }

    #[test]
    fn deplace_un_chapitre_avec_son_tableau() {
        // le tableau ne peut qu'être déplacé : supprimé ici, réinséré là
        let x = apres(&["# Chapitre II", "Texte deux.", "Case", "# Chapitre I", "Texte un."]);
        assert_eq!(plan(&x), ["# Chapitre II", "Texte deux.", "Case", "# Chapitre I", "Texte un."]);
        let x = apres(&["Texte deux.", "# Chapitre I", "Texte un.", "Case", "# Chapitre II"]);
        assert_eq!(plan(&x), ["Texte deux.", "# Chapitre I", "Texte un.", "Case", "# Chapitre II"]);
        let x = apres(&["# Chapitre II", "# Chapitre I", "Texte un.", "Case", "Texte deux."]);
        assert_eq!(plan(&x), ["# Chapitre II", "# Chapitre I", "Texte un.", "Case", "Texte deux."]);
        assert!(x.contains("<w:tbl>"));
    }

    #[test]
    fn protege_les_tableaux() {
        assert!(restructure(LIVRE, STY, &["# Chapitre I", "Texte un.", "# Chapitre II", "Texte deux."]).is_err());
        let x = apres(&["# Chapitre I", "Texte un.", "Case modifiée", "# Chapitre II", "Texte deux."]);
        assert_eq!(plan(&x)[2], "Case modifiée");
    }

    #[test]
    fn alignement_minimal() {
        let ops = aligne(&["a", "b", "c", "d"], &["a", "x", "c", "d", "e"]);
        assert_eq!(ops, [Op::Garde(0, 0), Op::Garde(1, 1), Op::Garde(2, 2), Op::Garde(3, 3), Op::Ajout(4)]);
        let ops = aligne(&["a", "b", "c"], &["c", "a", "b"]);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Garde(i, j) if i != j)).count(), 2);
        // « b » déplacé à côté d'une ligne modifiée : pas confondu avec la modification
        let ops = aligne(&["a", "b", "c", "d"], &["a", "x", "c", "d", "b"]);
        assert!(ops.contains(&Op::Suppr(1)) && ops.contains(&Op::Ajout(1)) && ops.contains(&Op::Ajout(4)));
    }

    /// `DOCX_ESSAI=in.docx DOCX_SORTIE=out.docx cargo test -- --ignored vrai_docx --nocapture`
    #[test]
    #[ignore]
    fn vrai_docx() {
        let (src, dst) = (std::env::var("DOCX_ESSAI").unwrap(), std::env::var("DOCX_SORTIE").unwrap());
        let texte = lis_plan(Path::new(&src)).unwrap();
        let mut lignes: Vec<String> = texte.split('\n').map(String::from).collect();
        // une ligne modifiée, une ajoutée, et le dernier chapitre remonté en tête
        let i = lignes.iter().position(|l| l.len() > 20 && !l.starts_with('#')).unwrap();
        lignes[i] = format!("[MODIFIÉ 長城] {}", lignes[i]);
        lignes.insert(i + 1, "Paragraphe ajouté par le test.".into());
        if let Some(c) = lignes.iter().rposition(|l| l.starts_with("# ")) {
            let fin = lignes.len() - 1;
            let chapitre: Vec<String> = lignes.drain(c..fin).collect();
            let premier = lignes.iter().position(|l| l.starts_with("# ")).unwrap();
            lignes.splice(premier..premier, chapitre);
        }
        ecris(Path::new(&src), Path::new(&dst), &lignes.join("\n")).unwrap();
        let relu = lis_plan(Path::new(&dst)).unwrap();
        println!("{} paragraphes, {} titres", lignes.len(), lignes.iter().filter(|l| l.starts_with('#')).count());
        assert_eq!(relu, lignes.join("\n"));
    }
}
