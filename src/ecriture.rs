//! Mode écriture : plan du document et gestion des chapitres.
//! Les titres sont des lignes du texte marquées « # », « ## »… (comme en Markdown ; les styles de titre d'un .docx
//! deviennent ces marques à l'ouverture et redeviennent des styles à l'enregistrement). Sans aucune marque, les lignes
//! « Chapitre… », « Introduction »… servent de plan. Tout se fait donc sur le texte : rien d'autre à synchroniser.

pub const MAX_NIVEAU: u8 = 9;

/// Niveau et texte d'une ligne de titre : « ## Partie » -> (2, "Partie").
pub fn titre(ligne: &str) -> Option<(u8, &str)> {
    let n = ligne.bytes().take_while(|&b| b == b'#').count();
    if n == 0 || n > MAX_NIVEAU as usize {
        return None;
    }
    ligne[n..].strip_prefix(' ').map(|t| (n as u8, t))
}

/// Le texte tel que les correcteurs le lisent : les marques de titre deviennent des espaces (mêmes positions).
pub fn sans_marques(texte: &str) -> String {
    if !texte.contains("# ") {
        return texte.to_string();
    }
    let mut s = String::with_capacity(texte.len());
    for (k, l) in texte.split('\n').enumerate() {
        if k > 0 {
            s.push('\n');
        }
        match titre(l) {
            Some((n, t)) => {
                s.extend(std::iter::repeat_n(' ', n as usize + 1));
                s.push_str(t);
            }
            None => s.push_str(l),
        }
    }
    s
}

/// Ligne qui sert de titre de chapitre quand le texte n'a aucune marque « # ».
fn titre_implicite(ligne: &str) -> bool {
    const MOTS: &[&str] = &[
        "chapitre", "partie", "livre", "prologue", "épilogue", "introduction", "conclusion", "annexe", "préface", "avant-propos",
    ];
    let l = ligne.trim();
    let premier = l.split_whitespace().next().unwrap_or("").trim_end_matches([':', '.', ',']).to_lowercase();
    l.chars().count() <= 100 && !l.ends_with(['.', ',', ';']) && MOTS.contains(&premier.as_str())
}

/// Part idéale de chaque chapitre (titres de niveau 1) dans le livre : l'introduction ≈ 10 %, la conclusion ≈ 8 %,
/// le reste partagé à parts égales entre les autres chapitres. Renvoie une part (0 à 1) par section du plan (0 hors niveau 1).
pub fn parts_ideales(plan: &[Section]) -> Vec<f32> {
    let est = |s: &Section, mots: &[&str]| {
        let t = s.titre.to_lowercase();
        mots.iter().any(|m| t.contains(m))
    };
    let chap: Vec<usize> = (0..plan.len()).filter(|&i| plan[i].niveau == 1).collect();
    let intro = chap.iter().filter(|&&i| est(&plan[i], &["introduction", "avant-propos", "préface", "prologue"])).count();
    let concl = chap.iter().filter(|&&i| est(&plan[i], &["conclusion", "épilogue", "postface"])).count();
    let autres = chap.len() - intro - concl;
    let (pi, pc) = (if intro > 0 { 0.10 } else { 0.0 }, if concl > 0 { 0.08 } else { 0.0 });
    let reste = if autres > 0 { (1.0 - pi - pc) / autres as f32 } else { 0.0 };
    plan.iter()
        .map(|s| match () {
            _ if s.niveau != 1 => 0.0,
            _ if est(s, &["introduction", "avant-propos", "préface", "prologue"]) => pi / intro as f32,
            _ if est(s, &["conclusion", "épilogue", "postface"]) => pc / concl as f32,
            _ if autres > 0 => reste,
            _ => (1.0 - pi - pc) / chap.len().max(1) as f32,
        })
        .collect()
}

/// Plan type d'un livre : introduction, chapitres, conclusion.
pub fn modele_livre(chapitres: usize) -> String {
    let mut t = String::from("# Introduction\n\n");
    for i in 1..=chapitres {
        t.push_str(&format!("# Chapitre {i}\n\n## Partie 1\n\n## Partie 2\n\n"));
    }
    t.push_str("# Conclusion\n\n");
    t
}

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub niveau: u8,
    pub titre: String,
    /// lignes [ligne, fin_ligne) : le titre, son texte et ses sous-sections
    pub ligne: usize,
    pub fin_ligne: usize,
    /// mêmes bornes en caractères
    pub debut: usize,
    pub fin: usize,
    /// mots de la section, sous-sections comprises (titres exclus)
    pub mots: usize,
    /// titre marqué « # » (sinon repéré d'après ses mots)
    pub marque: bool,
}

pub fn plan(texte: &str) -> Vec<Section> {
    let lignes: Vec<&str> = texte.split('\n').collect();
    let mut debuts = Vec::with_capacity(lignes.len() + 1);
    let mut cumul_mots = Vec::with_capacity(lignes.len() + 1);
    let (mut c, mut m) = (0usize, 0usize);
    let mut titres: Vec<(usize, u8, &str, bool)> = Vec::new();
    for (k, l) in lignes.iter().enumerate() {
        debuts.push(c);
        cumul_mots.push(m);
        c += l.chars().count() + 1;
        match titre(l) {
            Some((n, t)) => titres.push((k, n, t, true)),
            None => m += l.split_whitespace().count(),
        }
    }
    debuts.push(c.saturating_sub(1));
    cumul_mots.push(m);
    if titres.is_empty() {
        titres = lignes.iter().enumerate().filter(|(_, l)| titre_implicite(l)).map(|(k, l)| (k, 1, l.trim(), false)).collect();
    }
    titres
        .iter()
        .enumerate()
        .map(|(i, &(k, n, t, marque))| {
            let fin_ligne = titres[i + 1..].iter().find(|s| s.1 <= n).map_or(lignes.len(), |s| s.0);
            let mots = cumul_mots[fin_ligne] - cumul_mots[k] - if marque { 0 } else { lignes[k].split_whitespace().count() };
            let titre = t.trim();
            Section {
                niveau: n,
                titre: if titre.is_empty() { "(sans titre)".into() } else { titre.into() },
                ligne: k,
                fin_ligne,
                debut: debuts[k],
                fin: debuts[fin_ligne],
                mots,
                marque,
            }
        })
        .collect()
}

/// Section la plus profonde qui contient la position `pos` (en caractères).
pub fn courante(plan: &[Section], pos: usize) -> Option<usize> {
    plan.iter().rposition(|s| s.debut <= pos && pos < s.fin.max(s.debut + 1))
}

fn joint(lignes: &[&str]) -> String {
    lignes.join("\n")
}

/// Position (en caractères) du début de la ligne `k`.
fn debut_ligne(lignes: &[&str], k: usize) -> usize {
    lignes[..k].iter().map(|l| l.chars().count() + 1).sum()
}

/// Échange la section `i` (avec ses sous-sections) avec sa voisine de même niveau, au-dessus ou au-dessous.
/// Renvoie le nouveau texte et la nouvelle position du titre déplacé.
pub fn deplace(texte: &str, plan: &[Section], i: usize, haut: bool) -> Option<(String, usize)> {
    let s = &plan[i];
    let voisine = if haut {
        plan[..i].iter().rev().take_while(|v| v.niveau >= s.niveau).find(|v| v.niveau == s.niveau)
    } else {
        plan[i + 1..].iter().take_while(|v| v.niveau >= s.niveau).find(|v| v.niveau == s.niveau)
    }?;
    let (a, b) = if haut { (voisine, s) } else { (s, voisine) };
    let lignes: Vec<&str> = texte.split('\n').collect();
    let mut nouv: Vec<&str> = lignes[..a.ligne].to_vec();
    nouv.extend(&lignes[b.ligne..b.fin_ligne]);
    nouv.extend(&lignes[a.ligne..a.fin_ligne]);
    nouv.extend(&lignes[b.fin_ligne..]);
    let ligne_titre = if haut { a.ligne } else { a.ligne + b.fin_ligne - b.ligne };
    Some((joint(&nouv), debut_ligne(&nouv, ligne_titre)))
}

/// Le texte où toutes les sections du plan sont marquées « # » à leur niveau (titres repérés d'après leurs mots).
pub fn marque_tout(texte: &str, plan: &[Section]) -> String {
    if plan.iter().all(|s| s.marque) {
        return texte.to_string();
    }
    let mut lignes: Vec<String> = texte.split('\n').map(String::from).collect();
    for s in plan.iter().filter(|s| !s.marque) {
        lignes[s.ligne] = format!("{} {}", "#".repeat(s.niveau as usize), lignes[s.ligne].trim());
    }
    lignes.join("\n")
}

/// Donne le niveau `n` (0 = texte courant) au titre de la ligne `k`.
pub fn change_niveau(texte: &str, k: usize, n: u8) -> String {
    let mut lignes: Vec<&str> = texte.split('\n').collect();
    let corps = titre(lignes[k]).map_or(lignes[k], |(_, t)| t);
    let nouvelle = if n == 0 { corps.to_string() } else { format!("{} {corps}", "#".repeat(n.min(MAX_NIVEAU) as usize)) };
    lignes[k] = &nouvelle;
    joint(&lignes)
}

/// Insère un titre de niveau `n` (et une ligne vide pour écrire) avant la ligne `k` ; renvoie le texte et les bornes
/// (en caractères) du titre, à sélectionner pour le renommer aussitôt.
pub fn insere_titre(texte: &str, k: usize, n: u8, nom: &str) -> (String, (usize, usize)) {
    let mut lignes: Vec<&str> = texte.split('\n').collect();
    let k = k.min(lignes.len());
    let ligne = format!("{} {nom}", "#".repeat(n.clamp(1, MAX_NIVEAU) as usize));
    lignes.splice(k..k, [ligne.as_str(), ""]);
    let debut = debut_ligne(&lignes, k) + n as usize + 1;
    (joint(&lignes), (debut, debut + nom.chars().count()))
}

fn est_conclusion(t: &str) -> bool {
    let t = t.to_lowercase();
    ["conclusion", "épilogue", "postface"].iter().any(|m| t.contains(m))
}

/// Numérote dans l'ordre les titres de plus haut niveau « Chapitre N… » (les autres titres ne bougent pas).
pub fn renumerote(texte: &str) -> String {
    let p = plan(texte);
    let Some(haut) = p.iter().map(|s| s.niveau).min() else { return texte.to_string() };
    let mut lignes: Vec<String> = texte.split('\n').map(String::from).collect();
    let mut n = 0;
    for s in p.iter().filter(|s| s.niveau == haut && s.marque) {
        let nouveau = {
            let Some((_, t)) = titre(&lignes[s.ligne]) else { continue };
            let Some(reste) = t.strip_prefix("Chapitre ") else { continue };
            let chiffres = reste.bytes().take_while(u8::is_ascii_digit).count();
            if chiffres == 0 {
                continue;
            }
            n += 1;
            format!("{} Chapitre {n}{}", "#".repeat(haut as usize), &reste[chiffres..])
        };
        lignes[s.ligne] = nouveau;
    }
    lignes.join("\n")
}

/// Ajoute un chapitre après celui du curseur (sinon avant la conclusion, sinon à la fin), numéroté à la suite ;
/// les « Chapitre N » suivants sont renumérotés. Renvoie le texte et le titre, à sélectionner pour le renommer.
pub fn ajoute_chapitre(texte: &str, pos: usize) -> (String, (usize, usize)) {
    let p = plan(texte);
    let base = marque_tout(texte, &p);
    let haut = p.iter().map(|s| s.niveau).min().unwrap_or(1);
    let chap = courante(&p, pos).and_then(|c| (0..=c).rev().find(|&k| p[k].niveau == haut));
    let concl = p.iter().find(|s| s.niveau == haut && est_conclusion(&s.titre));
    let k = match (chap, concl) {
        (Some(c), _) if !est_conclusion(&p[c].titre) => p[c].fin_ligne,
        (_, Some(s)) => s.ligne,
        _ => texte.split('\n').count(),
    };
    let t = renumerote(&insere_titre(&base, k, haut, "Chapitre 0").0);
    let sel = {
        let lignes: Vec<&str> = t.split('\n').collect();
        let d = debut_ligne(&lignes, k);
        (d + haut as usize + 1, d + lignes[k].chars().count())
    };
    (t, sel)
}

/// Ajoute « Partie N » à la fin de la section `i` (N : nombre de ses parties + 1).
pub fn ajoute_partie(texte: &str, i: usize) -> (String, (usize, usize)) {
    let p = plan(texte);
    let base = marque_tout(texte, &p);
    let n = p[i].niveau + 1;
    let nb = p[i + 1..].iter().take_while(|s| s.niveau > p[i].niveau).filter(|s| s.niveau == n).count();
    insere_titre(&base, p[i].fin_ligne, n, &format!("Partie {}", nb + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ajout_de_chapitres() {
        let t = "# Introduction\nA\n# Chapitre 1\nB\n# Chapitre 2\nC\n# Conclusion\nD";
        let sel = |r: &str, (d, f): (usize, usize)| r.chars().skip(d).take(f - d).collect::<String>();
        let (r, s) = ajoute_chapitre(t, t.find('B').unwrap());
        assert_eq!(r, "# Introduction\nA\n# Chapitre 1\nB\n# Chapitre 2\n\n# Chapitre 3\nC\n# Conclusion\nD");
        assert_eq!(sel(&r, s), "Chapitre 2");
        let (r, s) = ajoute_chapitre(t, t.len() - 1);
        assert!(r.ends_with("# Chapitre 3\n\n# Conclusion\nD"), "{r}");
        assert_eq!(sel(&r, s), "Chapitre 3");
        let (r, s) = ajoute_partie(t, 1);
        assert!(r.contains("# Chapitre 1\nB\n## Partie 1\n\n# Chapitre 2"), "{r}");
        assert_eq!(sel(&r, s), "Partie 1");
        let (r, s) = ajoute_chapitre("", 0);
        assert_eq!(sel(&r, s), "Chapitre 1");
    }

    const LIVRE: &str = "Avant-texte\n# Chapitre I\nUn deux trois.\n## Partie A\nQuatre.\n# Chapitre II\nCinq six.";

    #[test]
    fn plan_et_mots() {
        let p = plan(LIVRE);
        let resume: Vec<(u8, &str, usize, usize)> = p.iter().map(|s| (s.niveau, s.titre.as_str(), s.ligne, s.mots)).collect();
        assert_eq!(resume, [(1, "Chapitre I", 1, 4), (2, "Partie A", 3, 1), (1, "Chapitre II", 5, 2)]);
        assert_eq!((p[0].fin_ligne, p[1].fin_ligne, p[2].fin_ligne), (5, 5, 7));
        assert_eq!(&LIVRE.chars().skip(p[2].debut).collect::<String>(), "# Chapitre II\nCinq six.");
        assert_eq!(courante(&p, p[1].debut + 3), Some(1));
        assert_eq!(courante(&p, 2), None);
    }

    #[test]
    fn marques_masquees_sans_decalage() {
        let s = sans_marques(LIVRE);
        assert_eq!(s.chars().count(), LIVRE.chars().count());
        assert!(s.contains("\n  Chapitre I\n") && s.contains("\n   Partie A\n"));
        assert_eq!(titre("#hashtag"), None);
    }

    #[test]
    fn deplace_les_chapitres() {
        let p = plan(LIVRE);
        let (t, pos) = deplace(LIVRE, &p, 2, true).unwrap();
        assert_eq!(t, "Avant-texte\n# Chapitre II\nCinq six.\n# Chapitre I\nUn deux trois.\n## Partie A\nQuatre.");
        assert_eq!(pos, "Avant-texte\n".chars().count());
        assert_eq!(deplace(LIVRE, &p, 0, false).unwrap().0, t);
        assert!(deplace(LIVRE, &p, 1, true).is_none()); // pas de partie sœur avant « Partie A »
    }

    #[test]
    fn titres_implicites_et_niveaux() {
        let texte = "CHAPITRE I Les familles\nTexte.\nChapitre 2 : Les migrations\nSuite.";
        let p = plan(texte);
        assert_eq!(p.len(), 2);
        assert!(!p[0].marque && p[0].mots == 1);
        let m = marque_tout(texte, &p);
        assert!(m.starts_with("# CHAPITRE I") && m.contains("\n# Chapitre 2"));
        assert_eq!(change_niveau(&m, 0, 2).lines().next(), Some("## CHAPITRE I Les familles"));
        assert_eq!(change_niveau(&m, 0, 0).lines().next(), Some("CHAPITRE I Les familles"));
        let (t, (a, b)) = insere_titre(&m, 2, 1, "Nouveau chapitre");
        assert_eq!(t.chars().skip(a).take(b - a).collect::<String>(), "Nouveau chapitre");
        assert_eq!(plan(&t).len(), 3);
    }
}
