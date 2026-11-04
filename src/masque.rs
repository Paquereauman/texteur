//! Repérage des passages à protéger de la correction : écritures chinoise/japonaise/coréenne
//! (automatique, désactivable) et termes des dictionnaires actifs (pinyin, noms propres…).

use std::collections::HashMap;

pub fn est_cjk(c: char) -> bool {
    matches!(c as u32,
        0x2E80..=0x2FDF      // radicaux
        | 0x3000..=0x303F    // ponctuation CJK
        | 0x3040..=0x30FF    // kana japonais
        | 0x31C0..=0x31EF    // traits
        | 0x3400..=0x4DBF    // extension A
        | 0x4E00..=0x9FFF    // idéogrammes unifiés
        | 0xAC00..=0xD7AF    // hangul
        | 0xF900..=0xFAFF    // compatibilité
        | 0xFF00..=0xFFEF    // pleine chasse
        | 0x20000..=0x2EBEF) // extensions B à F
}

/// `(a, b)` et `(c, d)` se chevauchent ; une insertion (c == d) strictement dans `(a, b)` compte.
pub fn chevauche((a, b): (usize, usize), (c, d): (usize, usize)) -> bool {
    if c == d { a < c && c < b } else { c < b && a < d }
}

fn min(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

#[derive(Default)]
pub struct Masque {
    par_initiale: HashMap<char, Vec<Vec<char>>>,
    ecritures: bool,
}

impl Masque {
    /// `ecritures` : protéger automatiquement sinogrammes, kana et hangul.
    pub fn new<S: AsRef<str>>(termes: impl IntoIterator<Item = S>, ecritures: bool) -> Self {
        let mut m: HashMap<char, Vec<Vec<char>>> = HashMap::new();
        // Mots grammaticaux et fonctionnels français à ne JAMAIS masquer
        // (pour éviter qu'un pinyin sans ton comme "Du", "Ma", "Si" masque les fautes de français)
        const MOTS_FONCTIONNELS_FR: &[&str] = &[
            "du", "de", "des", "le", "la", "les", "un", "une", "ce", "cet", "cette", "ces",
            "se", "sa", "son", "ses", "mon", "ton", "ma", "ta", "si", "en", "ou", "où",
            "a", "à", "te", "me", "tu", "il", "ils", "on", "au", "aux", "par", "pour", "dans", "sur"
        ];

        for t in termes {
            let s_brut = t.as_ref().trim();
            let v: Vec<char> = s_brut.chars().map(min).collect();
            // un nombre (« 10 », « 10e ») n'est jamais un terme à protéger : il masquerait « 10.7 % », « 10 000 »…
            if v.is_empty() || (v.len() < 2 && !est_cjk(v[0])) || v[0].is_ascii_digit() {
                continue;
            }
            // Si le terme est entièrement latin et correspond à un mot grammatical français, ne pas le masquer
            if !v.iter().any(|&c| est_cjk(c)) {
                let mot_latin: String = v.iter().collect();
                if MOTS_FONCTIONNELS_FR.contains(&mot_latin.as_str()) {
                    continue;
                }
            }
            m.entry(v[0]).or_default().push(v);
        }
        for l in m.values_mut() {
            l.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
            l.dedup();
        }
        Masque { par_initiale: m, ecritures }
    }

    /// Plages protégées, en indices de caractères, triées et disjointes.
    pub fn plages(&self, texte: &str) -> Vec<(usize, usize)> {
        let ch: Vec<char> = texte.chars().collect();
        let bas: Vec<char> = ch.iter().map(|c| min(*c)).collect();
        let mut res = Vec::new();
        let mut i = 0;
        while i < ch.len() {
            if self.ecritures && est_cjk(ch[i]) {
                let d = i;
                while i < ch.len() && est_cjk(ch[i]) {
                    i += 1;
                }
                res.push((d, i));
                continue;
            }
            if i == 0 || !ch[i - 1].is_alphanumeric() || est_cjk(ch[i]) {
                let trouve = self.par_initiale.get(&bas[i]).and_then(|l| {
                    l.iter().find(|t| {
                        bas[i..].starts_with(t)
                            && (est_cjk(t[0]) || ch.get(i + t.len()).is_none_or(|c| !c.is_alphanumeric()))
                    })
                });
                if let Some(t) = trouve {
                    res.push((i, i + t.len()));
                    i += t.len();
                    continue;
                }
            }
            i += 1;
        }

        // Si l'auto-protection globale des écritures est désactivée, protéger tout de même
        // les noms propres chinois détectés (toponymes, entreprises, personnes)
        if !self.ecritures {
            for (d, f, _) in detecte_noms_propres_cjk(texte) {
                if !res.iter().any(|&p| chevauche(p, (d, f))) {
                    res.push((d, f));
                }
            }
            res.sort_by_key(|p| p.0);
        }

        res
    }
}

/// Détecte les structures de noms propres chinois (personnes avec titres ou patronymes,
/// toponymes et organisations) dans une séquence de sinogrammes.
pub fn detecte_noms_propres_cjk(texte: &str) -> Vec<(usize, usize, &'static str)> {
    let ch: Vec<char> = texte.chars().collect();
    let n = ch.len();
    let mut entites = Vec::new();

    // 100 patronymes chinois majeurs
    const PATRONYMES: &[char] = &[
        '王', '李', '张', '刘', '陈', '杨', '黄', '赵', '吴', '周',
        '徐', '孙', '马', '朱', '胡', '郭', '何', '高', '林', '郑',
        '谢', '罗', '梁', '宋', '唐', '许', '韩', '冯', '邓', '曹',
        '彭', '曾', '肖', '田', '董', '袁', '潘', '于', '蒋', '蔡',
        '余', '杜', '叶', '程', '苏', '魏', '吕', '丁', '任', '沈',
        '姚', '卢', '姜', '崔', '钟', '谭', '陆', '汪', '范', '金',
        '石', '廖', '贾', '夏', '韦', '付', '方', '白', '邹', '孟',
        '熊', '秦', '邱', '江', '尹', '毛', '段', '雷', '钱', '汤',
        '黎', '易', '常', '武', '乔', '贺', '赖', '龚', '文',
    ];

    // Titres de personnes honorifiques ou professionnels
    const TITRES_PERSONNES: &[&str] = &[
        "总", "经理", "董事长", "总裁", "主席", "主任", "厂长", "工程师",
        "老师", "师傅", "医生", "律师", "教授", "先生", "女士", "小姐",
    ];

    // Suffixes d'organisations
    const SUFFIXES_ORGA: &[&str] = &[
        "公司", "有限公司", "集团", "工厂", "制造厂", "局", "院", "银行",
        "大学", "学院", "事务所", "中心", "协会",
    ];

    // Suffixes de toponymes et géographie
    const SUFFIXES_LIEUX: &[&str] = &[
        "省", "市", "区", "县", "镇", "村", "路", "街", "大道", "桥",
        "机场", "港", "站", "山", "江", "河", "湖", "海", "湾", "关",
    ];

    let mut i = 0;
    while i < n {
        if est_cjk(ch[i]) {
            let debut = i;
            while i < n && est_cjk(ch[i]) {
                i += 1;
            }
            let fin = i;
            let longueur = fin - debut;
            let segment: String = ch[debut..fin].iter().collect();

            // 1. Détection organisation
            if longueur >= 3 && SUFFIXES_ORGA.iter().any(|suf| segment.ends_with(suf)) {
                entites.push((debut, fin, "Entreprise / Organisation"));
                continue;
            }

            // 2. Détection toponyme / lieu
            if longueur >= 2 && SUFFIXES_LIEUX.iter().any(|suf| segment.ends_with(suf)) {
                entites.push((debut, fin, "Lieu / Toponyme"));
                continue;
            }

            // 3. Détection personne (patronyme + titre ou prénom)
            let premier = ch[debut];
            if PATRONYMES.contains(&premier) {
                if longueur >= 2 && TITRES_PERSONNES.iter().any(|titre| segment[premier.len_utf8()..].ends_with(titre)) {
                    entites.push((debut, fin, "Personne (avec titre)"));
                    continue;
                }
                if longueur == 2 || longueur == 3 {
                    entites.push((debut, fin, "Nom de personne"));
                    continue;
                }
            }
        } else {
            i += 1;
        }
    }

    entites
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protege_sinogrammes_et_termes() {
        let m = Masque::new(["Chángchéng", "Shenzhen"], true);
        let t = "Le 长城 (Chángchéng) près de shenzhen, pas Shenzhenois.";
        let p: Vec<String> = m.plages(t).iter().map(|&(a, b)| t.chars().skip(a).take(b - a).collect()).collect();
        assert_eq!(p, ["长城", "Chángchéng", "shenzhen"]);
    }

    #[test]
    fn sinogrammes_du_dictionnaire_sans_protection_auto() {
        let m = Masque::new(["长城"], false);
        assert_eq!(m.plages("le 长城 et 深圳"), [(3, 5)]);
    }

    #[test]
    fn chevauchement() {
        assert!(chevauche((3, 6), (5, 8)));
        assert!(!chevauche((3, 6), (6, 8)));
        assert!(chevauche((3, 6), (4, 4)));
        assert!(!chevauche((3, 6), (3, 3)));
    }

    #[test]
    fn test_detection_noms_propres_cjk() {
        let texte = "Nous rencontrons 王总 à 深圳市 au siège de 比亚迪集团.";
        let entites = detecte_noms_propres_cjk(texte);
        assert!(entites.iter().any(|&(_, _, cat)| cat.contains("Personne")));
        assert!(entites.iter().any(|&(_, _, cat)| cat.contains("Lieu")));
        assert!(entites.iter().any(|&(_, _, cat)| cat.contains("Organisation")));
    }
}
