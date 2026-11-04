#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Texteur : éditeur léger, ultra-rapide et 100 % local (sans Word).
//! - Ouvre et modifie les documents Word (.docx), .txt et .md sans ouvrir Word
//! - Détection des fautes d'orthographe (rouge) et de grammaire (vert)
//! - Mode Répétitions & Style avec formulations contextuelles de qualité (violet)
//! - Gestion des dictionnaires personnels et protection des termes spécialisés

mod dico;
mod docx;
mod ecriture;
mod grammaire;
mod ia;
mod maj;
mod masque;
mod orthographe;
mod style;
mod tournures;
mod pedagogie;
mod regles_dynamiques;
use pedagogie::explication_pedagogique_approfondie;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use egui::text::{CCursor, CCursorRange, LayoutJob};
use egui::{Color32, FontFamily, FontId, Key, KeyboardShortcut, Modifiers, Stroke, TextFormat};

use dico::{Bibliotheque, Terme, analyse_ligne};
use grammaire::{CorrecteurGrammaire, GenreErreur};
use ia::RapportIA;
use masque::{Masque, chevauche};
use orthographe::{EtatOrtho, Orthographe, nom_langue};
use style::{AnalyseurStyle, Repetition};

const TEXTE_ESSAI: &str = "Bonjour, je vous envoie le document concernant le projet 長城 (Chángchéng).\nLes fournisseur de Shenzhen 深圳 ont confirmer la livraison du projet.\nNous devons faire une réunion pour faire le point sur ce problème et régler ce problème rapidement.";

const OUVRIR: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const ENREGISTRER: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
const AJOUT_SELECTION: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::D);
const AJOUT_PRESSE_PAPIERS: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::D);
const FAUTE_SUIVANTE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F8);
const DELAI_VERIF: Duration = Duration::from_millis(300);
const REGLES_COUVERTES_PAR_GRAMMALECTE: &[&str] = &[
    "Accord de l'adjectif au pluriel",
    "Accord du pluriel",
    "Infinitif après préposition",
    "Accord du singulier",
    "Confusion on / ont (pronom vs auxiliaire avoir)",
    "Élision obligatoire devant voyelle",
    "Accord en genre du déterminant (féminin requis)",
    "Confusion se / ce (pronom vs déterminant démonstratif)",
    "Adjectif de couleur invariable",
    "Confusion a / à",
    "Article contracté devant un nom féminin",
    "Confusion ce / se",
];

fn hash_texte(t: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

type CacheGL = std::sync::Arc<Mutex<std::collections::HashMap<String, Vec<crate::ia::ErreurGL>>>>;

fn libelle_grammalecte(type_erreur: &str) -> &'static str {
    match type_erreur {
        "gn" => "accord",
        "conj" | "vmode" | "infi" | "ppas" | "imp" | "inte" => "verbe",
        "conf" => "confusion",
        "neg" => "négation",
        "pleo" => "pléonasme",
        "loc" => "locution",
        "bs" => "style",
        "date" => "date",
        _ => "grammaire",
    }
}

/// Fautes Grammalecte de tout le texte ; les paragraphes déjà vérifiés viennent du cache.
fn alertes_grammalecte(texte: &str, port: u16, cache: &CacheGL) -> Result<Vec<AlerteG>, String> {
    let mut paragraphes = Vec::new();
    let mut debut = 0;
    for ligne in texte.split('\n') {
        if ligne.chars().any(char::is_alphabetic) {
            paragraphes.push((debut, ligne));
        }
        debut += ligne.chars().count() + 1;
    }
    let a_faire: Vec<&str> = {
        let c = cache.lock().map_err(|e| e.to_string())?;
        let mut vus = HashSet::new();
        paragraphes.iter().map(|(_, p)| *p).filter(|p| !c.contains_key(*p) && vus.insert(*p)).collect()
    };
    let mut partiels = std::collections::HashMap::new();
    for bloc in a_faire.chunks(50) {
        let erreurs = crate::ia::verifier_paragraphes(bloc, port)?;
        // spaCy et le détecteur encore en chargement : résultat partiel, gardé hors du cache
        if crate::ia::analyse_incomplete() {
            partiels.extend(bloc.iter().map(|p| p.to_string()).zip(erreurs));
            continue;
        }
        let mut c = cache.lock().map_err(|e| e.to_string())?;
        if c.len() > 20_000 {
            c.clear();
        }
        for (p, e) in bloc.iter().zip(erreurs) {
            c.insert(p.to_string(), e);
        }
    }
    let c = cache.lock().map_err(|e| e.to_string())?;
    let mut alertes = Vec::new();
    for (debut, p) in paragraphes {
        let car: Vec<char> = p.chars().collect();
        for crate::ia::ErreurGL(d, f, type_erreur, _, message, suggestions) in c.get(p).or_else(|| partiels.get(p)).into_iter().flatten() {
            if d < f && *f <= car.len() {
                alertes.push(AlerteG {
                    debut: debut + d,
                    fin: debut + f,
                    mot: car[*d..*f].iter().collect(),
                    suggestions: suggestions.clone(),
                    regle: format!("Grammalecte · {}", libelle_grammalecte(type_erreur)),
                    explication: message.clone(),
                    type_alerte: TypeAlerte::Grammaire,
                });
            }
        }
    }
    Ok(alertes)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeAlerte {
    Orthographe,
    Grammaire,
    Doublon,
    Repetition,
}

#[derive(Clone, Debug)]
pub struct AlerteG {
    pub debut: usize,
    pub fin: usize,
    pub mot: String,
    pub suggestions: Vec<String>,
    pub regle: String,
    pub explication: String,
    pub type_alerte: TypeAlerte,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OngletActif {
    Correction,
    Repetitions,
    /// Plan des chapitres, concentration, objectif de mots
    Ecriture,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanneauLateral {
    Aucun,
    Detections,
    Dictionnaires,
}

struct App {
    texte: String,
    titre: String,
    modifie: bool,
    selection_demandee: Option<(usize, usize)>,
    ortho: Orthographe,
    grammaire: CorrecteurGrammaire,
    style: AnalyseurStyle,
    biblio: Bibliotheque,
    masque: Arc<Masque>,
    ignores: HashSet<String>,
    fichier: Option<PathBuf>,
    onglet: OngletActif,
    surligner_repetitions: bool,
    repetition_selectionnee: Option<String>,
    panneau_lateral: PanneauLateral,
    edite: usize,
    recherche: String,
    collage: String,
    nouveau_nom: String,
    selection: (usize, usize),
    derniere_frappe: Instant,
    saisie_mot_rapide: String,
    message: String,
    fermeture_confirmee: bool,
    /// Règles Grammalecte, appliquées en direct : seuls les paragraphes modifiés repartent au serveur
    alertes_gl: Vec<AlerteG>,
    rx_gl: Option<std::sync::mpsc::Receiver<Result<Vec<AlerteG>, String>>>,
    cache_gl: CacheGL,
    hash_gl: u64,
    relance_gl: Instant,
    gl_indisponible: bool,
    /// Cache de la dernière analyse : évite de tout recalculer à chaque image
    cache_analyse: Option<(u64, (Vec<(usize, usize)>, Vec<AlerteG>, Vec<Repetition>))>,
    alertes_plus_infos: HashSet<usize>,
    plus_infos_barre: bool,
    /// Mode écriture : position (caractères) à amener en haut de l'éditeur, texte hors de la section estompé,
    /// soulignements des fautes, mots au début de la séance
    defiler_vers: Option<usize>,
    concentration: bool,
    soulignements_ecriture: bool,
    mots_ouverture: usize,
    /// Mises à jour : Texteur recompilé (chemin, âge en minutes), modèle du détecteur plus récent que celui chargé
    maj_texteur: Option<(PathBuf, u64)>,
    maj_modele: bool,
    rx_maj: Option<std::sync::mpsc::Receiver<bool>>,
    maj_signalee: bool,
    prochaine_verif_maj: Instant,
}

/// Action demandée depuis le plan (appliquée après l'affichage du panneau).
enum ActionPlan {
    Aller(usize),
    Deplacer(usize, bool),
    Niveau(usize, i8),
    RetirerTitre(usize),
    Selectionner(usize),
    SousPartie(usize),
    NouveauChapitre,
}

fn nb_mots(texte: &str) -> usize {
    ecriture::sans_marques(texte).split_whitespace().count()
}

/// Au plus `n` caractères.
fn abrege(t: &str, n: usize) -> String {
    if t.chars().count() <= n {
        t.to_string()
    } else {
        format!("{}…", t.chars().take(n.saturating_sub(1)).collect::<String>().trim_end())
    }
}

fn polices(ctx: &egui::Context) {
    let mut f = egui::FontDefinitions::default();
    let polices = [
        ("segoe", r"C:\Windows\Fonts\segoeui.ttf", true),
        ("segoe_sym", r"C:\Windows\Fonts\seguisym.ttf", false),
        ("cjk", r"C:\Windows\Fonts\msyh.ttc", false),
        ("cjk2", r"C:\Windows\Fonts\simsun.ttc", false),
    ];
    for (nom, chemin, premiere) in polices {
        if f.font_data.contains_key(nom) {
            continue;
        }
        let Ok(octets) = std::fs::read(chemin) else { continue };
        f.font_data.insert(nom.into(), Arc::new(egui::FontData::from_owned(octets)));
        for fam in [FontFamily::Proportional, FontFamily::Monospace] {
            let liste = f.families.get_mut(&fam).unwrap();
            if premiere && fam == FontFamily::Proportional {
                liste.insert(0, nom.into());
            } else {
                liste.push(nom.into());
            }
        }
    }
    ctx.set_fonts(f);
}

fn chemin_ignores() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("Texteur").join("ignores.json")
}

fn charge_ignores() -> HashSet<String> {
    std::fs::read_to_string(chemin_ignores()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn sauve_ignores(ignores: &HashSet<String>) {
    let f = chemin_ignores();
    if let Some(p) = f.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    if let Ok(json) = serde_json::to_string_pretty(ignores) {
        let _ = std::fs::write(f, json);
    }
}

/// Voyelles pinyin à ton (macron/caron) : absentes du français, donc signe sûr de romanisation chinoise.
fn est_voyelle_pinyin(c: char) -> bool {
    "āēīōūǖǎěǐǒǔǚǘǜĀĒĪŌŪǍĚǏǑǓ".contains(c)
}

/// Vrai si la zone [debut, fin) relève de l'écriture chinoise : caractères CJK dans le mot,
/// mot collé à des caractères CJK (ex. « Huawei华为 »), ou pinyin avec tons (« nǐ hǎo »).
/// Une telle zone n'est jamais une faute de français.
fn zone_chinoise(car: &[char], debut: usize, fin: usize) -> bool {
    let fin = fin.min(car.len());
    if debut >= fin {
        return false;
    }
    // Étendre au mot complet
    let mut d = debut;
    while d > 0 && (car[d - 1].is_alphanumeric() || car[d - 1] == '-') {
        d -= 1;
    }
    let mut f = fin;
    while f < car.len() && (car[f].is_alphanumeric() || car[f] == '-') {
        f += 1;
    }
    car[d..f].iter().any(|&c| masque::est_cjk(c) || est_voyelle_pinyin(c))
}

/// Analyse unifiée : combine les dictionnaires personnalisés, l'orthographe Windows,
/// les règles de grammaire françaises et l'analyse de répétitions.
fn analyse_unifiee(
    texte: &str,
    masque: &Masque,
    ortho: &Mutex<EtatOrtho>,
    grammaire: &CorrecteurGrammaire,
    style: &AnalyseurStyle,
    ignores: &HashSet<String>,
    onglet: OngletActif,
    repetition_cible: Option<&str>,
) -> (Vec<(usize, usize)>, Vec<AlerteG>, Vec<Repetition>) {
    let plages = masque.plages(texte);
    let car: Vec<char> = texte.chars().collect();
    let mut alertes = Vec::new();

    // 1. Alertes de grammaire (priorité haute pour les accords)
    let alertes_gram = grammaire.analyse(texte).into_iter().chain(tournures::analyse(texte));
    for ag in alertes_gram {
        // les sinogrammes sont protégés, sauf du contrôle qui les compare à leur pinyin
        let sino = ag.regle == tournures::REGLE_SINOGRAMME;
        if !sino && plages.iter().any(|&p| chevauche(p, (ag.debut, ag.fin))) {
            continue;
        }
        if !sino && zone_chinoise(&car, ag.debut, ag.fin) {
            continue;
        }
        if ignores.contains(&ag.texte_fautif) {
            continue;
        }
        let t_type = if ag.genre == GenreErreur::Doublon {
            TypeAlerte::Doublon
        } else {
            TypeAlerte::Grammaire
        };
        alertes.push(AlerteG {
            debut: ag.debut,
            fin: ag.fin,
            mot: ag.texte_fautif,
            suggestions: ag.suggestions,
            regle: ag.regle,
            explication: ag.explication,
            type_alerte: t_type,
        });
    }

    // 2. Alertes d'orthographe (mots inconnus / fautes de frappe)
    let e = ortho.lock().unwrap();
    let mut decalage = 0;
    for para in texte.split('\n') {
        if let Some(fs) = e.cache.get(para) {
            for f in fs.iter() {
                let (a, b) = (decalage + f.debut, decalage + f.fin);
                if plages.iter().any(|&p| chevauche(p, (a, b))) {
                    continue;
                }
                if zone_chinoise(&car, a, b) {
                    continue;
                }
                let mot: String = para.chars().skip(f.debut).take(f.fin - f.debut).collect();
                if ignores.contains(&mot) {
                    continue;
                }
                // Si la zone est déjà couverte par une alerte de grammaire plus précise, ne pas doubler
                if alertes.iter().any(|al| chevauche((al.debut, al.fin), (a, b))) {
                    continue;
                }
                alertes.push(AlerteG {
                    debut: a,
                    fin: b,
                    mot,
                    suggestions: f.suggestions.clone(),
                    regle: if f.doublon { "Mot répété".into() } else { "Orthographe lexicale".into() },
                    explication: if f.doublon { "Ce mot est présent deux fois de suite.".into() } else { "Ce mot n'est pas dans le dictionnaire usuel.".into() },
                    type_alerte: if f.doublon { TypeAlerte::Doublon } else { TypeAlerte::Orthographe },
                });
            }
        }
        decalage += para.chars().count() + 1;
    }
    drop(e);

    // 3. Détection des répétitions de style
    let repetitions = style.analyse(texte);

    // Si le mode Répétitions est actif, marquer les occurrences répétées
    if onglet == OngletActif::Repetitions {
        for rep in &repetitions {
            let correspond_cible = repetition_cible.map_or(true, |c| c.eq_ignore_ascii_case(&rep.racine));
            if correspond_cible {
                for &(deb, fin) in &rep.occurrences {
                    if plages.iter().any(|&p| chevauche(p, (deb, fin))) {
                        continue;
                    }
                    let mot_occ: String = texte.chars().skip(deb).take(fin - deb).collect();
                    alertes.push(AlerteG {
                        debut: deb,
                        fin,
                        mot: mot_occ,
                        suggestions: rep.alternatives.clone(),
                        regle: format!("Répétition : {} fois", rep.nombre),
                        explication: format!("Le terme « {} » est répété {} fois dans le document.", rep.mot, rep.nombre),
                        type_alerte: TypeAlerte::Repetition,
                    });
                }
            }
        }
    }

    alertes.sort_by_key(|a| a.debut);
    (plages, alertes, repetitions)
}

fn colore(
    ui: &egui::Ui,
    plages_protegees: &[(usize, usize)],
    plages_repetitions: &[(usize, usize)],
    repetition_active_plages: &[(usize, usize)],
    focus: Option<(usize, usize)>,
    texte: &str,
    largeur: f32,
) -> Arc<egui::Galley> {
    let sombre = ui.visuals().dark_mode;
    let normal = TextFormat {
        font_id: FontId::proportional(17.0),
        color: ui.visuals().text_color(),
        ..Default::default()
    };
    // Surlignage doux et reposant pour les répétitions (fond pastel lavande clair)
    let repet_douce = TextFormat {
        color: if sombre { Color32::from_rgb(222, 214, 250) } else { Color32::from_rgb(90, 65, 140) },
        background: if sombre { Color32::from_rgba_unmultiplied(167, 139, 250, 48) } else { Color32::from_rgba_unmultiplied(167, 139, 250, 55) },
        ..normal.clone()
    };
    // Surlignage accentué pour la répétition activement sélectionnée
    let repet_active = TextFormat {
        color: if sombre { Color32::from_rgb(245, 240, 255) } else { Color32::from_rgb(75, 45, 145) },
        background: if sombre { Color32::from_rgba_unmultiplied(167, 139, 250, 95) } else { Color32::from_rgba_unmultiplied(167, 139, 250, 110) },
        ..normal.clone()
    };

    // Titres « # » : plus grands, la marque discrète
    let mut titres: Vec<(usize, usize)> = Vec::new();
    let mut morceaux: Vec<(usize, usize, TextFormat)> = Vec::new();
    let mut c = 0;
    for l in texte.split('\n') {
        let n = l.chars().count();
        if let Some((niv, _)) = ecriture::titre(l) {
            let m = niv as usize + 1;
            let taille = [24.0, 21.0, 19.0].get(niv as usize - 1).copied().unwrap_or(18.0);
            let couleur = if sombre { Color32::from_rgb(240, 244, 255) } else { Color32::BLACK };
            let marque = TextFormat { font_id: FontId::proportional(taille), color: Color32::from_rgb(90, 100, 125), ..normal.clone() };
            let fmt = TextFormat { font_id: FontId::proportional(taille), color: couleur, ..normal.clone() };
            morceaux.push((c, c + m, marque));
            morceaux.push((c + m, c + n, fmt));
            titres.push((c, c + n));
        }
        c += n + 1;
    }
    // Les caractères chinois et termes protégés ne sont ni soulignés ni colorés (rendu textuel naturel et épuré)
    let libre = |a: usize, b: usize| !plages_protegees.iter().chain(&titres).any(|&p| chevauche(p, (a, b)));
    for &(a, b) in repetition_active_plages {
        if libre(a, b) {
            morceaux.push((a, b, repet_active.clone()));
        }
    }
    for &(a, b) in plages_repetitions {
        if libre(a, b) && !repetition_active_plages.iter().any(|&p| chevauche(p, (a, b))) {
            morceaux.push((a, b, repet_douce.clone()));
        }
    }
    morceaux.sort_by_key(|m| m.0);

    let octets: Vec<usize> = texte.char_indices().map(|(o, _)| o).chain([texte.len()]).collect();
    let n = octets.len().saturating_sub(1);
    let mut job = LayoutJob::default();
    // Mode concentration : le texte hors de la section en cours est estompé
    let ajoute = |job: &mut LayoutJob, a: usize, b: usize, fmt: &TextFormat| {
        let mut coupes = vec![a, b];
        if let Some((fa, fb)) = focus {
            coupes.extend([fa, fb].into_iter().filter(|&x| a < x && x < b));
            coupes.sort_unstable();
        }
        for w in coupes.windows(2) {
            let dedans = focus.is_none_or(|(fa, fb)| fa <= w[0] && w[1] <= fb);
            let f = if dedans { fmt.clone() } else { TextFormat { color: fmt.color.gamma_multiply(0.3), ..fmt.clone() } };
            job.append(&texte[octets[w[0]]..octets[w[1]]], 0.0, f);
        }
    };
    let mut pos = 0;
    for (a, b, fmt) in morceaux {
        let (a, b) = (a.clamp(pos, n), b.min(n));
        if a >= b {
            continue;
        }
        if pos < a {
            ajoute(&mut job, pos, a, &normal);
        }
        ajoute(&mut job, a, b, &fmt);
        pos = b;
    }
    if pos < n {
        ajoute(&mut job, pos, n, &normal);
    }
    job.wrap.max_width = largeur;
    ui.fonts(|f| f.layout_job(job))
}

/// Soulignement ondulé avec couleur dédiée selon le type de faute :
/// - Rouge pour l'orthographe
/// - Vert pour la grammaire / accords
/// - Orange pour les doublons
/// - Violet pour les répétitions de style
fn vagues(peintre: &egui::Painter, galley: &egui::Galley, origine: egui::Pos2, alertes: &[AlerteG]) {
    for a in alertes {
        let couleur = match a.type_alerte {
            TypeAlerte::Orthographe => Color32::from_rgb(235, 65, 65), // Rouge franc
            TypeAlerte::Grammaire => Color32::from_rgb(45, 185, 130),   // Vert émeraude
            TypeAlerte::Doublon => Color32::from_rgb(245, 160, 30),     // Orange vif
            TypeAlerte::Repetition => Color32::from_rgba_unmultiplied(155, 145, 205, 180), // Lavande doux et reposant
        };

        let mut lignes: Vec<(f32, f32, f32)> = Vec::new();
        for k in a.debut..a.fin {
            let r = galley.pos_from_ccursor(CCursor::new(k));
            let x_fin = galley.pos_from_ccursor(CCursor::new(k + 1)).min.x.max(r.min.x);
            match lignes.last_mut() {
                Some(l) if l.2 == r.max.y => l.1 = l.1.max(x_fin),
                _ => lignes.push((r.min.x, x_fin, r.max.y)),
            }
        }
        for (x0, x1, y) in lignes {
            let y = origine.y + y - 1.0;
            let (x0, x1) = (origine.x + x0, origine.x + x1);
            let mut points = Vec::new();
            let mut x = x0;
            let mut haut = true;
            while x < x1 {
                points.push(egui::pos2(x, if haut { y - 1.5 } else { y + 1.5 }));
                x += 2.5;
                haut = !haut;
            }
            points.push(egui::pos2(x1, y));
            let epaisseur = if a.type_alerte == TypeAlerte::Repetition { 1.1_f32 } else { 1.3_f32 };
            peintre.add(egui::Shape::line(points, Stroke::new(epaisseur, couleur)));
        }
    }
}

fn octet(s: &str, i: usize) -> usize {
    s.char_indices().nth(i).map_or(s.len(), |(o, _)| o)
}

impl App {
    /// Grammalecte relit en tâche de fond (le serveur démarre tout seul, en ~1 s, sans carte graphique).
    /// `relancer` : arrête d'abord le serveur, qui redémarre avec le dernier modèle du détecteur.
    fn lancer_grammalecte(&mut self, relancer: bool) {
        let texte = ecriture::sans_marques(&self.texte);
        self.hash_gl = hash_texte(&texte);
        let cache = self.cache_gl.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.rx_gl = Some(rx);
        std::thread::spawn(move || {
            if relancer {
                crate::ia::arreter_grammalecte();
            }
            tx.send(crate::ia::assurer_grammalecte().and_then(|port| alertes_grammalecte(&texte, port, &cache))).ok();
        });
    }

    /// Cherche un Texteur recompilé (tout de suite) et un nouveau modèle du détecteur (en tâche de fond).
    fn verifier_maj(&mut self, signaler: bool) {
        self.maj_texteur = maj::texteur_plus_recent();
        self.maj_signalee = signaler;
        self.prochaine_verif_maj = Instant::now() + Duration::from_secs(600);
        let (tx, rx) = std::sync::mpsc::channel();
        self.rx_maj = Some(rx);
        std::thread::spawn(move || tx.send(crate::ia::nouveau_modele()).ok());
    }

    fn installer_texteur(&mut self, ctx: &egui::Context) {
        let Some((src, _)) = self.maj_texteur.clone() else { return };
        if self.modifie {
            self.message = "Enregistrez d'abord le document (Ctrl+S), puis relancez la mise à jour.".into();
            return;
        }
        match maj::installe(&src) {
            Ok(exe) => {
                let mut cmd = std::process::Command::new(exe);
                cmd.args(self.fichier.iter());
                match cmd.spawn() {
                    Ok(_) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                    Err(e) => self.message = format!("Texteur mis à jour ; relancez-le vous-même ({e})."),
                }
            }
            Err(e) => self.message = format!("⚠ {e}"),
        }
    }

    fn recharger_modele(&mut self) {
        self.cache_gl.lock().unwrap().clear();
        self.maj_modele = false;
        self.gl_indisponible = false;
        self.lancer_grammalecte(true);
        self.message = "⟳ Rechargement du détecteur avec le nouveau modèle (environ une minute)…".into();
    }


    fn new(cc: &eframe::CreationContext) -> Self {
        polices(&cc.egui_ctx);
        cc.egui_ctx.style_mut(|s| {
            s.text_styles.insert(egui::TextStyle::Heading, FontId::proportional(17.5));
            s.text_styles.insert(egui::TextStyle::Body, FontId::proportional(15.5));
            s.text_styles.insert(egui::TextStyle::Button, FontId::proportional(13.5));
            s.text_styles.insert(egui::TextStyle::Small, FontId::proportional(12.5));

            s.spacing.item_spacing = egui::vec2(8.0, 6.0);
            s.spacing.button_padding = egui::vec2(10.0, 5.0);

            s.visuals.dark_mode = true;
            s.visuals.panel_fill = Color32::from_rgb(16, 18, 24); // #101218 ardoise sombre
            s.visuals.window_fill = Color32::from_rgb(22, 26, 35); // #161A23 cartes de surface
            s.visuals.extreme_bg_color = Color32::from_rgb(13, 15, 20);

            // Widgets inactifs
            s.visuals.widgets.inactive.bg_fill = Color32::from_rgb(26, 31, 41);
            s.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(22, 26, 35);
            s.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(40, 48, 63));
            s.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
            s.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(215, 222, 235));

            // Widgets survolés
            s.visuals.widgets.hovered.bg_fill = Color32::from_rgb(38, 45, 60);
            s.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(32, 38, 51);
            s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(65, 77, 102));
            s.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
            s.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(245, 248, 255));

            // Widgets actifs / cliqués
            s.visuals.widgets.active.bg_fill = Color32::from_rgb(48, 58, 78);
            s.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(42, 51, 68);
            s.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(99, 102, 241));
            s.visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
            s.visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

            // Widgets déroulés
            s.visuals.widgets.open.bg_fill = Color32::from_rgb(32, 38, 51);
            s.visuals.widgets.open.corner_radius = egui::CornerRadius::same(6);

            // Sélection de texte moderne
            s.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(79, 70, 229, 130);
            s.visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(129, 140, 248));
        });
        let biblio = Bibliotheque::charge();
        let masque = Arc::new(Masque::new(biblio.formes_actives(), biblio.reglages.proteger_ecritures));
        let mut app = App {
            texte: TEXTE_ESSAI.into(),
            titre: "Sans titre".into(),
            modifie: false,
            selection_demandee: None,
            ortho: Orthographe::new(cc.egui_ctx.clone(), &biblio.reglages.langue),
            grammaire: CorrecteurGrammaire::new(),
            style: AnalyseurStyle::new(),
            biblio,
            masque,
            ignores: charge_ignores(),
            fichier: None,
            onglet: OngletActif::Correction,
            surligner_repetitions: true,
            repetition_selectionnee: None,
            panneau_lateral: PanneauLateral::Detections,
            edite: 0,
            recherche: String::new(),
            collage: String::new(),
            nouveau_nom: String::new(),
            selection: (0, 0),
            derniere_frappe: Instant::now(),
            saisie_mot_rapide: String::new(),
            message: String::new(),
            fermeture_confirmee: false,
            alertes_gl: Vec::new(),
            rx_gl: None,
            cache_gl: Default::default(),
            hash_gl: 0,
            relance_gl: Instant::now(),
            gl_indisponible: false,
            cache_analyse: None,
            alertes_plus_infos: HashSet::new(),
            plus_infos_barre: false,
            defiler_vers: None,
            concentration: false,
            soulignements_ecriture: false,
            mots_ouverture: 0,
            maj_texteur: None,
            maj_modele: false,
            rx_maj: None,
            maj_signalee: false,
            prochaine_verif_maj: Instant::now(),
        };
        app.mots_ouverture = nb_mots(&app.texte);
        app.edite = app.biblio.cible().unwrap_or(0);
        if let Some(chemin) = std::env::args().nth(1) {
            app.ouvre(PathBuf::from(chemin));
        }
        app
    }

    fn ouvre(&mut self, chemin: PathBuf) {
        let est_docx = chemin.extension().is_some_and(|e| e.eq_ignore_ascii_case("docx"));
        let lu = if est_docx {
            docx::lis_plan(&chemin)
        } else {
            std::fs::read(&chemin)
                .map(|o| String::from_utf8_lossy(&o).trim_start_matches('\u{feff}').replace("\r\n", "\n"))
                .map_err(|e| e.to_string())
        };
        match lu {
            Ok(texte) => {
                self.texte = texte;
                self.titre = chemin.file_name().unwrap_or_default().to_string_lossy().into();
                self.modifie = false;
                self.message = format!("Ouvert sans Word : {}", self.titre);
                self.fichier = Some(chemin);
                self.mots_ouverture = nb_mots(&self.texte);
            }
            Err(e) => self.message = format!("Ouverture impossible : {e}"),
        }
    }

    fn enregistre(&mut self, sous: bool) {
        let source_docx = self.fichier.clone().filter(|f| f.extension().is_some_and(|e| e.eq_ignore_ascii_case("docx")));
        let dest = match (&self.fichier, sous) {
            (Some(f), false) => Some(f.clone()),
            _ => {
                let mut d = rfd::FileDialog::new();
                d = if source_docx.is_some() {
                    d.add_filter("Document Word", &["docx"]).add_filter("Texte", &["txt", "md"])
                } else {
                    d.add_filter("Texte", &["txt", "md"])
                };
                d.save_file()
            }
        };
        let Some(dest) = dest else { return };
        let texte = &self.texte;
        let res = match (&source_docx, dest.extension().is_some_and(|e| e.eq_ignore_ascii_case("docx"))) {
            (Some(src), true) => docx::ecris(src, &dest, texte),
            (None, true) => Err("Un .docx ne peut être créé qu'à partir d'un .docx ouvert.".into()),
            _ => std::fs::write(&dest, texte.replace('\n', "\r\n")).map_err(|e| e.to_string()),
        };
        match res {
            Ok(()) => {
                self.modifie = false;
                self.titre = dest.file_name().unwrap_or_default().to_string_lossy().into();
                self.message = format!("Enregistré avec succès : {}", self.titre);
                self.fichier = Some(dest);
            }
            Err(e) => self.message = format!("Enregistrement impossible : {e}"),
        }
    }

    fn biblio_modifiee(&mut self, dico: Option<usize>) {
        let res = dico.map_or(Ok(()), |i| self.biblio.sauve_dico(i)).and(self.biblio.sauve_reglages());
        if let Err(e) = res {
            self.message = format!("Dictionnaire non enregistré : {e}");
        }
        self.masque = Arc::new(Masque::new(self.biblio.formes_actives(), self.biblio.reglages.proteger_ecritures));
        self.ortho.etat.lock().unwrap().cache.clear();
    }

    fn ajoute_lignes(&mut self, texte: &str) -> usize {
        if self.biblio.dicos.is_empty() {
            let _ = self.biblio.cree("Personnel");
        }
        let Some(i) = self.biblio.cible() else {
            self.message = "Créez d'abord un dictionnaire cible.".into();
            return 0;
        };
        let termes: Vec<Terme> = texte.lines().filter_map(analyse_ligne).collect();
        let n = termes.into_iter().filter(|t| self.biblio.dicos[i].ajoute(t.clone())).count();
        if n > 0 {
            let nom = self.biblio.dicos[i].nom.clone();
            self.biblio.active(&nom, true);
            self.biblio_modifiee(Some(i));
        }
        n
    }

    fn nom_cible(&self) -> String {
        self.biblio.cible().map(|i| self.biblio.dicos[i].nom.clone()).unwrap_or_default()
    }

    /// Ignore un mot (mémorisé entre les sessions) et pénalise les règles apprises qui l'ont signalé.
    fn ignorer(&mut self, mot: &str) {
        self.ignores.insert(mot.to_string());
        self.grammaire.banque_regles.signaler_rejet(mot);
        sauve_ignores(&self.ignores);
    }

    fn ajoute_mot(&mut self, mot: &str) {
        let n = self.ajoute_lignes(mot);
        self.message = if n > 0 {
            format!("✔ « {} » ajouté à « {} » (protégé de la correction).", mot.trim(), self.nom_cible())
        } else {
            format!("« {} » est déjà dans « {} ».", mot.trim(), self.nom_cible())
        };
    }

    fn ajoute_selection(&mut self) {
        let (a, b) = self.selection;
        let sel: String = self.texte.chars().skip(a).take(b - a).collect();
        if sel.trim().is_empty() {
            self.message = "Sélectionnez d'abord un terme dans le texte.".into();
            return;
        }
        self.ajoute_mot(&sel);
    }

    fn ajoute_presse_papiers(&mut self) {
        match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
            Ok(t) => {
                let n = self.ajoute_lignes(&t);
                self.message = format!("{n} terme(s) ajouté(s) à « {} ».", self.nom_cible());
            }
            Err(_) => self.message = "Presse-papiers vide.".into(),
        }
    }

    fn remplace(&mut self, debut: usize, fin: usize, par: &str) {
        let (a, b) = (octet(&self.texte, debut), octet(&self.texte, fin));
        self.texte.replace_range(a..b, par);
        self.modifie = true;
        let curseur = debut + par.chars().count();
        self.selection_demandee = Some((curseur, curseur));
    }

    /// Génère et copie dans le presse-papiers le rapport d'anomalies avec leurs phrases de contexte
    fn copier_rapport_ia(&mut self, format_json: bool) {
        let (_plages, alertes, repetitions) = analyse_unifiee(
            &self.texte,
            &self.masque,
            &self.ortho.etat,
            &self.grammaire,
            &self.style,
            &self.ignores,
            OngletActif::Correction,
            None,
        );
        let rapport = RapportIA::generer(&self.texte, &alertes, &repetitions, &self.titre);
        let contenu = if format_json {
            rapport.vers_json()
        } else {
            rapport.vers_markdown()
        };
        match arboard::Clipboard::new().and_then(|mut c| c.set_text(contenu)) {
            Ok(()) => {
                let fmt = if format_json { "JSON" } else { "Markdown" };
                self.message = format!("✔ Rapport IA ({fmt}) copié avec phrases de contexte !");
            }
            Err(e) => {
                self.message = format!("Erreur presse-papiers : {e}");
            }
        }
    }

    /// Exporte le rapport IA dans un fichier sur le disque (.md ou .json)
    fn exporter_rapport_ia_fichier(&mut self) {
        let (_plages, alertes, repetitions) = analyse_unifiee(
            &self.texte,
            &self.masque,
            &self.ortho.etat,
            &self.grammaire,
            &self.style,
            &self.ignores,
            OngletActif::Correction,
            None,
        );
        let rapport = RapportIA::generer(&self.texte, &alertes, &repetitions, &self.titre);
        let nom_base = self.titre.trim_end_matches(".docx").trim_end_matches(".txt").trim_end_matches(".md");
        if let Some(dest) = rfd::FileDialog::new()
            .add_filter("Rapport Markdown pour IA", &["md"])
            .add_filter("Rapport JSON pour IA", &["json"])
            .set_file_name(&format!("{nom_base}_rapport_ia.md"))
            .save_file()
        {
            let est_json = dest.extension().is_some_and(|e| e.eq_ignore_ascii_case("json"));
            let contenu = if est_json { rapport.vers_json() } else { rapport.vers_markdown() };
            match std::fs::write(&dest, contenu) {
                Ok(()) => {
                    self.message = format!("✔ Rapport IA enregistré : {}", dest.file_name().unwrap_or_default().to_string_lossy());
                }
                Err(e) => {
                    self.message = format!("Erreur enregistrement : {e}");
                }
            }
        }
    }

    /// Barre d'outils : une seule ligne. À gauche le document et le mode, à droite les volets ;
    /// le reste (fichiers, rapport, langue, surlignage) est rangé dans des menus.
    fn barre(&mut self, ui: &mut egui::Ui, nb_fautes: usize, nb_repet: usize) {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, egui::CornerRadius::same(5), Color32::from_rgb(16, 185, 129));
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "✍", egui::FontId::proportional(15.0), Color32::WHITE);
            ui.add_space(4.0);

            ui.menu_button("📂 Fichier", |ui| {
                if ui.button("Ouvrir…                Ctrl+O").clicked() {
                    ui.close_menu();
                    if let Some(f) = rfd::FileDialog::new().add_filter("Documents", &["docx", "txt", "md"]).pick_file() {
                        self.ouvre(f);
                    }
                }
                if ui.button("Enregistrer           Ctrl+S").clicked() {
                    ui.close_menu();
                    self.enregistre(false);
                }
                if ui.button("Enregistrer sous…").clicked() {
                    ui.close_menu();
                    self.enregistre(true);
                }
                ui.separator();
                if ui.button("🤖 Copier le rapport pour une IA")
                    .on_hover_text("Fautes + phrases de contexte, prêtes à coller dans ChatGPT, Claude, Gemini…")
                    .clicked()
                {
                    ui.close_menu();
                    self.copier_rapport_ia(false);
                }
            });
            ui.separator();

            // Mode : correction ou style (le surlignage ne concerne que le style)
            if ui.selectable_label(self.onglet == OngletActif::Correction, format!("✓ Correction  {nb_fautes}")).clicked() {
                self.onglet = OngletActif::Correction;
            }
            if ui.selectable_label(self.onglet == OngletActif::Repetitions, format!("◈ Style  {nb_repet}")).clicked() {
                self.onglet = OngletActif::Repetitions;
            }
            if ui.selectable_label(self.onglet == OngletActif::Ecriture, "✎ Écriture")
                .on_hover_text("Plan des chapitres, déplacement des sections, concentration, objectif de mots")
                .clicked()
            {
                self.onglet = OngletActif::Ecriture;
            }
            if self.onglet == OngletActif::Repetitions {
                ui.checkbox(&mut self.surligner_repetitions, "surligner")
                    .on_hover_text("Surligner les répétitions dans le texte");
            }
            ui.separator();

            // À droite : les volets, puis les réglages
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (langues, mut langue) = {
                    let e = self.ortho.etat.lock().unwrap();
                    (e.langues.clone(), e.langue.clone())
                };
                ui.menu_button("⚙", |ui| {
                    if ui.button("⟳ Vérifier les mises à jour").on_hover_text("Texteur recompilé, nouveau modèle du détecteur").clicked() {
                        ui.close_menu();
                        self.verifier_maj(true);
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Langue du correcteur").weak());
                    for l in &langues {
                        if ui.selectable_value(&mut langue, l.clone(), nom_langue(l)).clicked() {
                            ui.close_menu();
                        }
                    }
                })
                .response
                .on_hover_text(format!("Réglages — langue : {}", nom_langue(&langue)));
                if langue != self.biblio.reglages.langue {
                    self.ortho.change_langue(&langue);
                    self.biblio.reglages.langue = langue;
                    self.biblio_modifiee(None);
                }

                if self.maj_texteur.is_some() || self.maj_modele {
                    let bouton = egui::RichText::new("⬆ Mise à jour").strong().color(Color32::from_rgb(52, 211, 153));
                    ui.menu_button(bouton, |ui| {
                        if let Some((_, age)) = self.maj_texteur {
                            let quand = if age < 120 { format!("il y a {age} min") } else { format!("il y a {} h", age / 60) };
                            let r = ui.add_enabled(!self.modifie, egui::Button::new(format!("Installer le nouveau Texteur (compilé {quand})")))
                                .on_hover_text("Texteur se ferme et se rouvre sur le même document")
                                .on_disabled_hover_text("Enregistrez d'abord le document (Ctrl+S)");
                            if r.clicked() {
                                ui.close_menu();
                                self.installer_texteur(ui.ctx());
                            }
                        }
                        if self.maj_modele && ui.button("Charger le nouveau modèle du détecteur")
                            .on_hover_text("Le serveur de correction redémarre (environ une minute, Grammalecte reste disponible)")
                            .clicked()
                        {
                            ui.close_menu();
                            self.recharger_modele();
                        }
                    });
                }
                let mut volet = |ui: &mut egui::Ui, panneau: PanneauLateral, libelle: String, aide: &str| {
                    let actif = self.panneau_lateral == panneau;
                    if ui.selectable_label(actif, libelle).on_hover_text(aide).clicked() {
                        self.panneau_lateral = if actif { PanneauLateral::Aucun } else { panneau };
                    }
                };
                volet(ui, PanneauLateral::Dictionnaires, "📖 Dico".into(), "Dictionnaires personnels, mots et caractères chinois protégés (Ctrl+D : ajouter la sélection)");
                volet(ui, PanneauLateral::Detections, format!("🎯 Fautes  {nb_fautes}"), "F9 — liste des fautes (Ctrl+B)");
            });
        });
    }

    /// Barre de suggestions interactive et explicative sous la barre d'outils
    fn barre_suggestions(&mut self, ui: &mut egui::Ui, alertes: &[AlerteG], repetitions: &[Repetition]) {
        let curseur = self.selection.0;

        // Bouton d'ajout rapide si l'utilisateur a sélectionné un terme dans le document (ex: sinogrammes, pinyin...)
        let a_selection = self.selection.1 > self.selection.0;
        let texte_selectionne = if a_selection {
            let s: String = self.texte.chars().skip(self.selection.0).take(self.selection.1 - self.selection.0).collect();
            let t = s.trim().to_string();
            if !t.is_empty() && t.chars().count() <= 35 {
                Some(t)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(ref sel) = texte_selectionne {
            ui.horizontal(|ui| {
                let btn_sel = egui::Button::new(egui::RichText::new(format!("✔ Protéger « {} » au dictionnaire (Ctrl+D)", sel)).strong().color(Color32::from_rgb(56, 189, 248)))
                    .fill(Color32::from_rgb(20, 36, 48))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(14, 165, 233)));
                if ui.add(btn_sel).on_hover_text("Ajouter au dictionnaire personnel pour ne plus le compter comme faute").clicked() {
                    self.ajoute_selection();
                }
                ui.label(egui::RichText::new("— sélection active dans l'éditeur").weak().small());
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
        }

        if self.onglet == OngletActif::Repetitions {
            // MODE RÉPÉTITIONS & STYLE (Lavande doux, reposant et élégant)
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(Color32::from_rgb(196, 181, 253), egui::RichText::new("◈ Style & Répétitions").strong());

                if repetitions.is_empty() {
                    ui.label(egui::RichText::new("✨ Vocabulaire riche et varié — aucune répétition lourde détectée.").color(Color32::from_rgb(52, 211, 153)));
                    return;
                }

                // Trouver la répétition active (soit sélectionnée, soit sous le curseur)
                let rep_active = if let Some(cible) = &self.repetition_selectionnee {
                    repetitions.iter().find(|r| r.racine.eq_ignore_ascii_case(cible))
                } else {
                    repetitions.iter().find(|r| r.occurrences.iter().any(|&(d, f)| d <= curseur && curseur <= f))
                };

                // Liste stable des puces de répétitions
                ui.label(egui::RichText::new("Mots fréquents :").weak());
                for rep in repetitions.iter().take(6) {
                    let est_choisi = rep_active.map_or(false, |ra| ra.racine == rep.racine);
                    let label = format!("{} ({})", rep.mot, rep.nombre);
                    let btn = if est_choisi {
                        egui::Button::new(egui::RichText::new(label).strong().color(Color32::from_rgb(196, 181, 253)))
                            .fill(Color32::from_rgb(44, 38, 64))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(139, 120, 195)))
                    } else {
                        egui::Button::new(label)
                    };
                    if ui.add(btn).clicked() {
                        self.repetition_selectionnee = if est_choisi { None } else { Some(rep.racine.clone()) };
                        if let Some(&(deb, fin)) = rep.occurrences.first() {
                            self.selection_demandee = Some((deb, fin));
                        }
                    }
                }

                if let Some(rep) = rep_active {
                    ui.separator();
                    ui.label(egui::RichText::new(format!("« {} »", rep.mot)).strong().color(Color32::from_rgb(196, 181, 253)));

                    // Navigation entre les occurrences
                    let occ_idx = rep.occurrences.iter().position(|&(d, f)| d <= curseur && curseur <= f);
                    let idx_affiche = occ_idx.map_or(1, |i| i + 1);
                    ui.label(egui::RichText::new(format!("({}/{})", idx_affiche, rep.occurrences.len())).weak().small());

                    if ui.button("▶ Suiv.").on_hover_text("Sauter à l'occurrence suivante").clicked() {
                        let proch = occ_idx.map_or(0, |i| (i + 1) % rep.occurrences.len());
                        let (d, f) = rep.occurrences[proch];
                        self.selection_demandee = Some((d, f));
                    }

                    if !rep.alternatives.is_empty() {
                        ui.label(egui::RichText::new("Remplacer par :").weak().small());
                        for alt in &rep.alternatives {
                            let btn_alt = egui::Button::new(egui::RichText::new(alt).color(Color32::from_rgb(216, 206, 253)))
                                .fill(Color32::from_rgb(36, 32, 50))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(90, 80, 125)));
                            if ui.add(btn_alt).clicked() {
                                let occ_actuelle = occ_idx.map_or_else(|| rep.occurrences.first().copied(), |i| Some(rep.occurrences[i]));
                                if let Some((deb, fin)) = occ_actuelle {
                                    self.remplace(deb, fin, alt);
                                }
                            }
                        }
                    }
                } else {
                    ui.separator();
                    ui.label(egui::RichText::new("Cliquez sur un mot ci-dessus pour naviguer et remplacer.").weak());
                }
            });
            return;
        }

        // MODE CORRECTION (Orthographe, Grammaire, Doublons)
        let sous_curseur = alertes.iter().find(|a| a.debut <= curseur && curseur <= a.fin).cloned();
        ui.horizontal_wrapped(|ui| {
            if let Some(ref a) = sous_curseur {
                // Badge de couleur selon le type
                let (couleur_badge, titre_badge) = match a.type_alerte {
                    TypeAlerte::Orthographe => (Color32::from_rgb(248, 113, 113), "Orthographe"),
                    TypeAlerte::Grammaire => (Color32::from_rgb(52, 211, 153), "Grammaire"),
                    TypeAlerte::Doublon => (Color32::from_rgb(245, 158, 11), "Doublon"),
                    TypeAlerte::Repetition => (Color32::from_rgb(196, 181, 253), "Style"),
                };

                ui.colored_label(couleur_badge, egui::RichText::new(format!("● {titre_badge}")).strong());
                ui.label(egui::RichText::new(format!("« {} » :", a.mot)).strong());
                ui.label(egui::RichText::new(&a.explication).weak());

                if a.type_alerte == TypeAlerte::Doublon {
                    if ui.button(egui::RichText::new("🗑 Supprimer le doublon").strong().color(Color32::from_rgb(245, 158, 11))).clicked() {
                        let car: Vec<char> = self.texte.chars().collect();
                        let mut deb = a.debut;
                        while deb > 0 && car[deb - 1].is_whitespace() {
                            deb -= 1;
                        }
                        self.remplace(deb, a.fin, "");
                    }
                } else {
                    for s in &a.suggestions {
                        let btn_sugg = egui::Button::new(egui::RichText::new(s).strong().color(Color32::from_rgb(56, 189, 248)))
                            .fill(Color32::from_rgb(22, 34, 46))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 80, 110)));
                        if ui.add(btn_sugg).clicked() {
                            self.remplace(a.debut, a.fin, s);
                        }
                    }
                }

                ui.separator();
                let btn_info = egui::Button::new(egui::RichText::new(if self.plus_infos_barre { "▲ Moins d'infos" } else { "ℹ Plus d'infos" }).small().color(Color32::from_rgb(147, 197, 253)))
                    .fill(Color32::from_rgb(25, 30, 45));
                if ui.add(btn_info).clicked() {
                    self.plus_infos_barre = !self.plus_infos_barre;
                }

                if ui.button("Ignorer").clicked() {
                    self.ignorer(&a.mot);
                }
                if a.type_alerte == TypeAlerte::Orthographe {
                    let nom_c = self.nom_cible();
                    if ui.button(format!("+ « {} » au dictionnaire", nom_c)).clicked() {
                        self.ajoute_mot(&a.mot);
                    }
                }
            } else if let Some(rep) = repetitions.iter().find(|r| r.occurrences.iter().any(|&(d, f)| d <= curseur && curseur <= f)) {
                ui.colored_label(Color32::from_rgb(196, 181, 253), egui::RichText::new("◈ Répétition").strong());
                ui.label(egui::RichText::new(format!("« {} » ({} fois) :", rep.mot, rep.nombre)).strong());
                let occ_idx = rep.occurrences.iter().position(|&(d, f)| d <= curseur && curseur <= f);
                if !rep.alternatives.is_empty() {
                    ui.label(egui::RichText::new("Remplacer par :").weak().small());
                    for alt in &rep.alternatives {
                        let btn_alt = egui::Button::new(egui::RichText::new(alt).color(Color32::from_rgb(216, 206, 253)))
                            .fill(Color32::from_rgb(36, 32, 50))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(90, 80, 125)));
                        if ui.add(btn_alt).clicked() {
                            let occ_actuelle = occ_idx.map_or_else(|| rep.occurrences.first().copied(), |i| Some(rep.occurrences[i]));
                            if let Some((deb, fin)) = occ_actuelle {
                                self.remplace(deb, fin, alt);
                            }
                        }
                    }
                }
            } else {
                let n = alertes.len();
                if n == 0 {
                    ui.colored_label(Color32::from_rgb(52, 211, 153), egui::RichText::new("✨ Document impeccable — aucune anomalie détectée.").strong());
                } else {
                    ui.label(egui::RichText::new(format!("💡 {n} alertes détectées — vert pour grammaire, rouge pour orthographe (F8 pour naviguer).")).weak());
                }
            }
        });

        // Volet pédagogique dépliable « Plus d'infos » dans la barre inférieure
        if self.plus_infos_barre {
            if let Some(ref a) = sous_curseur {
                let fiche = explication_pedagogique_approfondie(&a.regle, &a.mot, &a.explication);
                egui::Frame::NONE
                    .fill(Color32::from_rgb(18, 22, 32))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 75, 110)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new(format!("📘 {}", fiche.titre)).strong().color(Color32::from_rgb(147, 197, 253)));
                            ui.label(egui::RichText::new(format!("— {}", fiche.regle_complete)).weak().small());
                        });
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            egui::Frame::NONE
                                .fill(Color32::from_rgb(22, 34, 28))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 197, 94)))
                                .corner_radius(egui::CornerRadius::same(5))
                                .inner_margin(egui::Margin::symmetric(8, 4))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(format!("💡 Astuce mnémotechnique :
{}", fiche.astuce)).strong().color(Color32::from_rgb(134, 239, 172)).small());
                                });
                            ui.label(egui::RichText::new(format!("Exemple : {}", fiche.exemple_correct)).small());
                        });
                    });
            }
        }
    }

        fn panneau_dictionnaires(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.label("Les termes cochés sont acceptés par le correcteur et protégés de toute modification.");
            ui.add_space(4.0);

            let mut changement = false;
            for i in 0..self.biblio.dicos.len() {
                let (nom, n) = (self.biblio.dicos[i].nom.clone(), self.biblio.dicos[i].termes.len());
                let mut actif = self.biblio.est_actif(&nom);
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut actif, format!("{nom} ({n} termes)")).changed() {
                        self.biblio.active(&nom, actif);
                        changement = true;
                    }
                });
            }
            if ui
                .checkbox(&mut self.biblio.reglages.proteger_ecritures, "Protéger automatiquement les caractères CJK (汉字 / かな / 한글)")
                .changed()
            {
                changement = true;
            }
            if changement {
                self.biblio_modifiee(None);
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.nouveau_nom).hint_text("Nouveau dictionnaire").desired_width(180.0));
                if ui.button("Créer").clicked() && !self.nouveau_nom.trim().is_empty() {
                    let nom = self.nouveau_nom.clone();
                    match self.biblio.cree(&nom) {
                        Ok(i) => {
                            self.edite = i;
                            self.nouveau_nom.clear();
                            self.biblio_modifiee(Some(i));
                        }
                        Err(e) => self.message = format!("Erreur : {e}"),
                    }
                }
            });

            ui.separator();
            let modifiables: Vec<usize> = (0..self.biblio.dicos.len()).collect();
            if modifiables.is_empty() {
                return;
            }
            if !modifiables.contains(&self.edite) {
                self.edite = modifiables[0];
            }
            let i = self.edite;

            ui.horizontal(|ui| {
                ui.strong("Dictionnaire :");
                egui::ComboBox::from_id_salt("edite").selected_text(self.biblio.dicos[i].nom.clone()).show_ui(ui, |ui| {
                    for &idx in &modifiables {
                        ui.selectable_value(&mut self.edite, idx, self.biblio.dicos[idx].nom.clone());
                    }
                });
                let est_cible = self.biblio.cible() == Some(i);
                if ui.add_enabled(!est_cible, egui::Button::new(if est_cible { "Cible ✔" } else { "Choisir comme cible" })).clicked() {
                    self.biblio.reglages.cible = self.biblio.dicos[i].nom.clone();
                    self.biblio_modifiee(None);
                }
            });

            ui.add_space(6.0);
            ui.strong("Ajout rapide d'un terme ou sinogrammes :");
            ui.horizontal(|ui| {
                let rep_rap = ui.add(
                    egui::TextEdit::singleline(&mut self.saisie_mot_rapide)
                        .hint_text("Ex: 深圳, pinyin, jargon…")
                        .desired_width(190.0),
                );
                let valider = ui.button("+ Ajouter").on_hover_text("Ajouter au dictionnaire cible (Entrée)").clicked()
                    || (rep_rap.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                if valider && !self.saisie_mot_rapide.trim().is_empty() {
                    let mot = std::mem::take(&mut self.saisie_mot_rapide);
                    self.ajoute_mot(&mot);
                }
            });
            ui.add_space(6.0);
            ui.strong("Coller une liste de mots");
            ui.label(egui::RichText::new("Une ligne par mot ou expression :").small());
            ui.add(egui::TextEdit::multiline(&mut self.collage).desired_rows(3).desired_width(f32::INFINITY));
            if ui.button("Importer les termes").clicked() {
                let termes: Vec<Terme> = std::mem::take(&mut self.collage).lines().filter_map(analyse_ligne).collect();
                let n = termes.into_iter().filter(|t| self.biblio.dicos[i].ajoute(t.clone())).count();
                self.biblio_modifiee(Some(i));
                self.message = format!("{n} terme(s) ajouté(s).");
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Rechercher :");
                ui.text_edit_singleline(&mut self.recherche);
            });
            let q = dico::sans_accents(&self.recherche).to_lowercase();
            let mut a_supprimer = None;
            egui::Grid::new("termes").striped(true).num_columns(3).show(ui, |ui| {
                for (j, t) in self.biblio.dicos[i].termes.iter().enumerate() {
                    let tout = dico::sans_accents(&format!("{} {} {}", t.caracteres, t.mot, t.sens)).to_lowercase();
                    if !q.is_empty() && !tout.contains(&q) {
                        continue;
                    }
                    ui.label(&t.caracteres);
                    ui.label(&t.mot);
                    if ui.small_button("✕").clicked() {
                        a_supprimer = Some(j);
                    }
                    ui.end_row();
                }
            });
            if let Some(j) = a_supprimer {
                self.biblio.dicos[i].termes.remove(j);
                self.biblio_modifiee(Some(i));
            }
        });
    }

    /// Panneau latéral droit catégorisé (Grammaire, Orthographe, Style, Doublons)
    /// Remplace tout le texte (déplacement de section, niveau de titre…) et amène le curseur en `sel`.
    fn applique_plan(&mut self, texte: String, sel: (usize, usize)) {
        self.texte = texte;
        self.modifie = true;
        self.fermeture_confirmee = false;
        self.derniere_frappe = Instant::now();
        self.va_a(sel);
    }

    fn va_a(&mut self, sel: (usize, usize)) {
        self.selection_demandee = Some(sel);
        self.defiler_vers = Some(sel.0);
    }

    fn action_plan(&mut self, action: ActionPlan) {
        let plan = ecriture::plan(&self.texte);
        let titre = |s: &ecriture::Section| (s.debut + s.niveau as usize + 1, s.debut + s.niveau as usize + 1);
        match action {
            ActionPlan::Aller(i) => self.va_a(titre(&plan[i])),
            ActionPlan::Selectionner(i) => self.va_a((plan[i].debut, plan[i].fin)),
            ActionPlan::Deplacer(i, haut) => match ecriture::deplace(&self.texte, &plan, i, haut) {
                Some((t, pos)) => {
                    self.applique_plan(t, (pos, pos));
                    self.message = format!("✔ « {} » déplacé {}.", abrege(&plan[i].titre, 40), if haut { "vers le haut" } else { "vers le bas" });
                }
                None => {
                    self.message = format!("Pas de section de même niveau {} « {} ».", if haut { "avant" } else { "après" }, abrege(&plan[i].titre, 40))
                }
            },
            ActionPlan::Niveau(i, d) => {
                let n = (plan[i].niveau as i8 + d).clamp(1, ecriture::MAX_NIVEAU as i8) as u8;
                let t = ecriture::change_niveau(&ecriture::marque_tout(&self.texte, &plan), plan[i].ligne, n);
                let p = plan[i].debut + n as usize + 1;
                self.applique_plan(t, (p, p));
            }
            ActionPlan::RetirerTitre(i) => {
                let t = ecriture::change_niveau(&ecriture::marque_tout(&self.texte, &plan), plan[i].ligne, 0);
                self.applique_plan(t, (plan[i].debut, plan[i].debut));
            }
            ActionPlan::SousPartie(i) => {
                let base = ecriture::marque_tout(&self.texte, &plan);
                let (t, sel) = ecriture::insere_titre(&base, plan[i].fin_ligne, plan[i].niveau + 1, "Nouvelle partie");
                self.applique_plan(t, sel);
            }
            ActionPlan::NouveauChapitre => {
                // après le chapitre en cours (le plus haut niveau du plan), sinon à la fin
                let haut = plan.iter().map(|s| s.niveau).min().unwrap_or(1);
                let cour = ecriture::courante(&plan, self.selection.0);
                let fin = cour
                    .and_then(|c| plan[..=c].iter().rev().find(|s| s.niveau == haut))
                    .map_or(self.texte.split('\n').count(), |s| s.fin_ligne);
                let base = ecriture::marque_tout(&self.texte, &plan);
                let (t, sel) = ecriture::insere_titre(&base, fin, haut, "Nouveau chapitre");
                self.applique_plan(t, sel);
            }
        }
    }

    /// Mode écriture : plan cliquable (mots et fautes par section), gestion des sections, séance d'écriture.
    fn panneau_plan(&mut self, ui: &mut egui::Ui, alertes: &[AlerteG]) {
        let plan = ecriture::plan(&self.texte);
        let cour = ecriture::courante(&plan, self.selection.0);
        let mut action = None;
        ui.label(egui::RichText::new("✎ Plan").size(17.0).strong());
        ui.horizontal_wrapped(|ui| {
            let mut bouton = |ui: &mut egui::Ui, texte: &str, aide: &str, a: Option<ActionPlan>| {
                if ui.add_enabled(a.is_some(), egui::Button::new(texte)).on_hover_text(aide).clicked() {
                    action = a;
                }
            };
            bouton(ui, "▲", "Monter la section en cours, avec ses sous-parties (Alt+↑)", cour.map(|c| ActionPlan::Deplacer(c, true)));
            bouton(ui, "▼", "Descendre la section en cours (Alt+↓)", cour.map(|c| ActionPlan::Deplacer(c, false)));
            bouton(ui, "◀", "Titre de niveau supérieur (## → #)", cour.filter(|&c| plan[c].niveau > 1).map(|c| ActionPlan::Niveau(c, -1)));
            bouton(ui, "▶", "Titre de niveau inférieur (# → ##)", cour.map(|c| ActionPlan::Niveau(c, 1)));
            bouton(ui, "➕ Chapitre", "Nouveau chapitre après celui en cours", Some(ActionPlan::NouveauChapitre));
            bouton(ui, "➕ Partie", "Nouvelle sous-partie à la fin de la section en cours", cour.map(ActionPlan::SousPartie));
        });
        ui.separator();
        if plan.is_empty() {
            ui.label(egui::RichText::new(
                "Aucun titre. Commencez une ligne par « # » pour un chapitre, « ## » pour une partie, ou cliquez « ➕ Chapitre ». \
                 Dans un document Word, les styles de titre sont repris automatiquement.",
            ).weak());
        }
        let fautes: Vec<usize> = alertes.iter().filter(|a| a.type_alerte != TypeAlerte::Repetition).map(|a| a.debut).collect();
        let hauteur = (ui.available_height() - 190.0).max(120.0);
        egui::ScrollArea::vertical().max_height(hauteur).auto_shrink([false, true]).show(ui, |ui| {
            for (i, s) in plan.iter().enumerate() {
                let n = fautes.partition_point(|&d| d < s.fin) - fautes.partition_point(|&d| d < s.debut);
                ui.horizontal(|ui| {
                    ui.add_space(14.0 * (s.niveau.saturating_sub(1)) as f32);
                    let mut libelle = egui::RichText::new(abrege(&s.titre, 42usize.saturating_sub(3 * s.niveau as usize)));
                    if s.niveau == 1 {
                        libelle = libelle.strong();
                    }
                    let r = ui.selectable_label(cour == Some(i), libelle).on_hover_text(&s.titre);
                    if r.clicked() {
                        action = Some(ActionPlan::Aller(i));
                    }
                    r.context_menu(|ui| {
                        let mut choix = |ui: &mut egui::Ui, t: &str, a: ActionPlan| {
                            if ui.button(t).clicked() {
                                action = Some(a);
                                ui.close_menu();
                            }
                        };
                        choix(ui, "▲ Monter", ActionPlan::Deplacer(i, true));
                        choix(ui, "▼ Descendre", ActionPlan::Deplacer(i, false));
                        if s.niveau > 1 {
                            choix(ui, "◀ Niveau supérieur", ActionPlan::Niveau(i, -1));
                        }
                        choix(ui, "▶ Niveau inférieur", ActionPlan::Niveau(i, 1));
                        choix(ui, "➕ Ajouter une sous-partie", ActionPlan::SousPartie(i));
                        choix(ui, "▭ Sélectionner toute la section", ActionPlan::Selectionner(i));
                        choix(ui, "✕ Retirer le titre (devient du texte)", ActionPlan::RetirerTitre(i));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{}", s.mots)).small().weak())
                            .on_hover_text("mots de la section, sous-parties comprises");
                        if n > 0 {
                            ui.label(egui::RichText::new(format!("● {n}")).small().color(Color32::from_rgb(248, 113, 113)))
                                .on_hover_text("fautes signalées dans la section");
                        }
                    });
                });
            }
        });
        ui.separator();
        // Séance d'écriture
        let mots = nb_mots(&self.texte);
        let seance = mots as i64 - self.mots_ouverture as i64;
        let lecture = mots / 230;
        ui.label(format!("📄 {mots} mots · ≈ {} h {:02} min de lecture", lecture / 60, lecture % 60));
        if let Some(c) = cour {
            ui.label(egui::RichText::new(format!("Section en cours : {} mots", plan[c].mots)).weak());
        }
        let objectif = self.biblio.reglages.objectif_mots.max(1);
        ui.add(
            egui::ProgressBar::new((seance.max(0) as f32 / objectif as f32).min(1.0))
                .text(format!("Séance : {seance:+} mots / objectif {objectif}")),
        );
        ui.horizontal(|ui| {
            ui.label("Objectif de la séance");
            if ui.add(egui::DragValue::new(&mut self.biblio.reglages.objectif_mots).range(50..=20000).speed(10.0).suffix(" mots")).changed() {
                let _ = self.biblio.sauve_reglages();
            }
        });
        ui.checkbox(&mut self.concentration, "Concentration : estomper le reste du texte")
            .on_hover_text("Seule la section où se trouve le curseur reste nette");
        ui.checkbox(&mut self.soulignements_ecriture, "Souligner les fautes pendant l'écriture");
        if let Some(a) = action {
            self.action_plan(a);
            ui.ctx().memory_mut(|m| m.request_focus(egui::Id::new("editeur")));
        }
    }

    fn panneau_liste_fautes(
        &mut self,
        ui: &mut egui::Ui,
        alertes: &[AlerteG],
        repetitions: &[Repetition],
        id_editeur: egui::Id,
    ) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🎯 Détections & Analyse").size(17.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").on_hover_text("Fermer ce panneau latéral").clicked() {
                    self.panneau_lateral = PanneauLateral::Aucun;
                }
            });
        });
        ui.add_space(6.0);

        // Carte d'action d'export automatique pour l'IA
        egui::Frame::NONE
            .fill(Color32::from_rgb(24, 28, 40))
            .corner_radius(egui::CornerRadius::same(7))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 56, 82)))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🤖 Export IA :").strong().color(Color32::from_rgb(165, 180, 252)));
                    if ui.small_button("📋 Markdown").on_hover_text("Copier les anomalies avec phrases de contexte pour ChatGPT, Claude, Gemini...").clicked() {
                        self.copier_rapport_ia(false);
                    }
                    if ui.small_button("💾 JSON").on_hover_text("Copier les anomalies en JSON").clicked() {
                        self.copier_rapport_ia(true);
                    }
                    if ui.small_button("📁 Fichier…").on_hover_text("Enregistrer le rapport dans un fichier .md ou .json").clicked() {
                        self.exporter_rapport_ia_fichier();
                    }
                });
            });
        ui.add_space(6.0);

        let grammaire_alertes: Vec<&AlerteG> = alertes.iter().filter(|a| a.type_alerte == TypeAlerte::Grammaire).collect();
        let ortho_alertes: Vec<&AlerteG> = alertes.iter().filter(|a| a.type_alerte == TypeAlerte::Orthographe).collect();
        let doublon_alertes: Vec<&AlerteG> = alertes.iter().filter(|a| a.type_alerte == TypeAlerte::Doublon).collect();

        let total_alertes = grammaire_alertes.len() + ortho_alertes.len() + doublon_alertes.len();

        if total_alertes == 0 && repetitions.is_empty() {
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("✨ Aucune anomalie détectée").color(Color32::from_rgb(52, 211, 153)).size(16.0).strong());
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Votre texte est parfaitement rédigé.").weak());
            });
            return;
        }

        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            // SECTION 1 : GRAMMAIRE & ACCORDS
            let label_gram = format!("● Grammaire & Accords ({})", grammaire_alertes.len());
            egui::CollapsingHeader::new(egui::RichText::new(label_gram).strong().color(Color32::from_rgb(52, 211, 153)))
                .default_open(true)
                .show(ui, |ui| {
                    if grammaire_alertes.is_empty() {
                        ui.label(egui::RichText::new("Aucune faute de grammaire.").weak().small());
                    }
                    for (idx, a) in grammaire_alertes.iter().enumerate() {
                        ui.push_id(("gram", idx), |ui| {
                            egui::Frame::NONE
                                .fill(Color32::from_rgb(22, 26, 35))
                                .corner_radius(egui::CornerRadius::same(7))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 44, 58)))
                                .inner_margin(egui::Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.link(egui::RichText::new(&a.mot).strong().size(15.0).color(Color32::from_rgb(52, 211, 153))).on_hover_text("Cliquer pour localiser dans le texte").clicked() {
                                            self.selection_demandee = Some((a.debut, a.fin));
                                            ui.ctx().memory_mut(|m| m.request_focus(id_editeur));
                                        }
                                        ui.label(egui::RichText::new(&a.regle).small().weak());
                                    });
                                    ui.label(egui::RichText::new(&a.explication).small());
                                    if !a.suggestions.is_empty() {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(egui::RichText::new("Corriger :").small().weak());
                                            for s in &a.suggestions {
                                                if ui.small_button(egui::RichText::new(s).strong().color(Color32::from_rgb(56, 189, 248))).clicked() {
                                                    self.remplace(a.debut, a.fin, s);
                                                }
                                            }
                                        });
                                    }

                                    let est_ouvert = self.alertes_plus_infos.contains(&a.debut);
                                    ui.horizontal(|ui| {
                                        let texte_btn = if est_ouvert { "▲ Masquer l'explication" } else { "ℹ Plus d'infos" };
                                        if ui.button(egui::RichText::new(texte_btn).small().color(Color32::from_rgb(147, 197, 253)))
                                            .on_hover_text("Afficher la règle complète, l'explication académique et l'astuce mnémotechnique")
                                            .clicked()
                                        {
                                            if est_ouvert {
                                                self.alertes_plus_infos.remove(&a.debut);
                                            } else {
                                                self.alertes_plus_infos.insert(a.debut);
                                            }
                                        }
                                    });

                                    if est_ouvert {
                                        let fiche = explication_pedagogique_approfondie(&a.regle, &a.mot, &a.explication);
                                        egui::Frame::NONE
                                            .fill(Color32::from_rgb(16, 20, 28))
                                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)))
                                             .corner_radius(egui::CornerRadius::same(6))
                                             .inner_margin(egui::Margin::symmetric(8, 6))
                                             .show(ui, |ui| {
                                                 ui.label(egui::RichText::new(format!("📘 {}", fiche.titre)).strong().color(Color32::from_rgb(147, 197, 253)).small());
                                                 ui.add_space(2.0);
                                                 ui.label(egui::RichText::new(fiche.regle_complete).small().color(Color32::from_rgb(220, 228, 240)));
                                                 ui.add_space(4.0);
                                                 egui::Frame::NONE
                                                     .fill(Color32::from_rgb(22, 32, 28))
                                                     .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 197, 94)))
                                                     .corner_radius(egui::CornerRadius::same(5))
                                                     .inner_margin(egui::Margin::symmetric(8, 5))
                                                     .show(ui, |ui| {
                                                         ui.label(egui::RichText::new(format!("💡 Astuce mnémotechnique :
{}", fiche.astuce)).strong().color(Color32::from_rgb(134, 239, 172)).small());
                                                     });
                                                 ui.add_space(3.0);
                                                 ui.label(egui::RichText::new(format!("Exemple : {}", fiche.exemple_correct)).weak().small());
                                             });
                                     }
                                });
                            ui.add_space(4.0);
                        });
                    }
                });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // SECTION 2 : ORTHOGRAPHE & LEXIQUE
            let label_ortho = format!("● Orthographe & Vocabulaire ({})", ortho_alertes.len());
            egui::CollapsingHeader::new(egui::RichText::new(label_ortho).strong().color(Color32::from_rgb(248, 113, 113)))
                .default_open(true)
                .show(ui, |ui| {
                    if ortho_alertes.is_empty() {
                        ui.label(egui::RichText::new("Aucune faute d'orthographe.").weak().small());
                    }
                    for (idx, a) in ortho_alertes.iter().enumerate() {
                        ui.push_id(("ortho", idx), |ui| {
                            egui::Frame::NONE
                                .fill(Color32::from_rgb(22, 26, 35))
                                .corner_radius(egui::CornerRadius::same(7))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 44, 58)))
                                .inner_margin(egui::Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.link(egui::RichText::new(&a.mot).strong().size(15.0).color(Color32::from_rgb(248, 113, 113))).on_hover_text("Cliquer pour localiser").clicked() {
                                            self.selection_demandee = Some((a.debut, a.fin));
                                            ui.ctx().memory_mut(|m| m.request_focus(id_editeur));
                                        }
                                        ui.label(egui::RichText::new(&a.regle).small().weak());
                                    });
                                    if !a.suggestions.is_empty() {
                                        ui.horizontal_wrapped(|ui| {
                                            for s in a.suggestions.iter().take(3) {
                                                if ui.small_button(egui::RichText::new(s).strong().color(Color32::from_rgb(56, 189, 248))).clicked() {
                                                    self.remplace(a.debut, a.fin, s);
                                                }
                                            }
                                        });
                                    }
                                    ui.horizontal(|ui| {
                                        if ui.small_button("Ignorer").clicked() {
                                            self.ignorer(&a.mot);
                                        }
                                        if ui.small_button(format!("+ « {} »", self.nom_cible())).clicked() {
                                            self.ajoute_mot(&a.mot);
                                        }
                                        let est_ouvert = self.alertes_plus_infos.contains(&a.debut);
                                        let texte_btn = if est_ouvert { "▲ Moins d'infos" } else { "ℹ Plus d'infos" };
                                        if ui.button(egui::RichText::new(texte_btn).small().color(Color32::from_rgb(147, 197, 253))).clicked() {
                                            if est_ouvert {
                                                self.alertes_plus_infos.remove(&a.debut);
                                            } else {
                                                self.alertes_plus_infos.insert(a.debut);
                                            }
                                        }
                                    });

                                    if self.alertes_plus_infos.contains(&a.debut) {
                                        let fiche = explication_pedagogique_approfondie(&a.regle, &a.mot, &a.explication);
                                        egui::Frame::NONE
                                            .fill(Color32::from_rgb(16, 20, 28))
                                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)))
                                            .corner_radius(egui::CornerRadius::same(6))
                                            .inner_margin(egui::Margin::symmetric(8, 6))
                                            .show(ui, |ui| {
                                                ui.label(egui::RichText::new(format!("📘 {}", fiche.titre)).strong().color(Color32::from_rgb(147, 197, 253)).small());
                                                ui.add_space(2.0);
                                                ui.label(egui::RichText::new(fiche.regle_complete).small().color(Color32::from_rgb(220, 228, 240)));
                                                ui.add_space(4.0);
                                                egui::Frame::NONE
                                                    .fill(Color32::from_rgb(22, 32, 28))
                                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 197, 94)))
                                                    .corner_radius(egui::CornerRadius::same(5))
                                                    .inner_margin(egui::Margin::symmetric(8, 5))
                                                    .show(ui, |ui| {
                                                         ui.label(egui::RichText::new(format!("💡 Conseil :
{}", fiche.astuce)).strong().color(Color32::from_rgb(134, 239, 172)).small());
                                                    });
                                                ui.add_space(3.0);
                                                ui.label(egui::RichText::new(format!("Exemple : {}", fiche.exemple_correct)).weak().small());
                                            });
                                    }
                                });
                            ui.add_space(4.0);
                        });
                    }
                });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // SECTION 3 : RÉPÉTITIONS DE STYLE
            let label_style = format!("● Répétitions & Style ({})", repetitions.len());
            egui::CollapsingHeader::new(egui::RichText::new(label_style).strong().color(Color32::from_rgb(196, 181, 253)))
                .default_open(true)
                .show(ui, |ui| {
                    if repetitions.is_empty() {
                        ui.label(egui::RichText::new("Aucune répétition de mot plein.").weak().small());
                    }
                    for (idx, rep) in repetitions.iter().enumerate() {
                        let est_sel = self.repetition_selectionnee.as_deref() == Some(&rep.racine);
                        ui.push_id(("rep", idx), |ui| {
                            egui::Frame::NONE
                                .fill(Color32::from_rgb(22, 26, 35))
                                .corner_radius(egui::CornerRadius::same(7))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 44, 58)))
                                .inner_margin(egui::Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.selectable_label(est_sel, egui::RichText::new(format!("« {} »", rep.mot)).strong().color(Color32::from_rgb(196, 181, 253))).clicked() {
                                            self.repetition_selectionnee = if est_sel { None } else { Some(rep.racine.clone()) };
                                            if let Some(&(deb, fin)) = rep.occurrences.first() {
                                                self.selection_demandee = Some((deb, fin));
                                                ui.ctx().memory_mut(|m| m.request_focus(id_editeur));
                                            }
                                        }
                                        ui.label(egui::RichText::new(format!("{} fois", rep.nombre)).small().weak());

                                        if rep.occurrences.len() > 1 {
                                            if ui.small_button("▶ Suiv.").on_hover_text("Sauter à l'occurrence suivante").clicked() {
                                                let curseur = self.selection.0;
                                                let cur_idx = rep.occurrences.iter().position(|&(d, f)| d <= curseur && curseur <= f);
                                                let proch = cur_idx.map_or(0, |i| (i + 1) % rep.occurrences.len());
                                                let (d, f) = rep.occurrences[proch];
                                                self.selection_demandee = Some((d, f));
                                                ui.ctx().memory_mut(|m| m.request_focus(id_editeur));
                                            }
                                        }
                                    });
                                    if !rep.alternatives.is_empty() {
                                        ui.label(egui::RichText::new("Alternatives :").small().weak());
                                        ui.horizontal_wrapped(|ui| {
                                            for alt in &rep.alternatives {
                                                if ui.small_button(egui::RichText::new(alt).color(Color32::from_rgb(196, 181, 253))).clicked() {
                                                    let curseur = self.selection.0;
                                                    let occ_cible = rep.occurrences.iter().find(|&&(d, f)| d <= curseur && curseur <= f)
                                                        .or(rep.occurrences.first())
                                                        .copied();
                                                    if let Some((deb, fin)) = occ_cible {
                                                        self.remplace(deb, fin, alt);
                                                    }
                                                }
                                            }
                                        });
                                    }
                                });
                            ui.add_space(4.0);
                        });
                    }
                });

            // SECTION 4 : DOUBLONS
            if !doublon_alertes.is_empty() {
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);
                let label_doublons = format!("● Mots doublons ({})", doublon_alertes.len());
                egui::CollapsingHeader::new(egui::RichText::new(label_doublons).strong().color(Color32::from_rgb(245, 158, 11)))
                    .default_open(true)
                    .show(ui, |ui| {
                        for (idx, a) in doublon_alertes.iter().enumerate() {
                            ui.push_id(("doublon", idx), |ui| {
                                egui::Frame::NONE
                                    .fill(Color32::from_rgb(22, 26, 35))
                                    .corner_radius(egui::CornerRadius::same(7))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 44, 58)))
                                    .inner_margin(egui::Margin::symmetric(10, 8))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui.link(egui::RichText::new(&a.mot).strong().color(Color32::from_rgb(245, 158, 11))).clicked() {
                                                self.selection_demandee = Some((a.debut, a.fin));
                                                ui.ctx().memory_mut(|m| m.request_focus(id_editeur));
                                            }
                                        });
                                        if ui.small_button("Supprimer le doublon").clicked() {
                                            let car: Vec<char> = self.texte.chars().collect();
                                            let mut deb = a.debut;
                                            while deb > 0 && car[deb - 1].is_whitespace() {
                                                deb -= 1;
                                            }
                                            self.remplace(deb, a.fin, "");
                                        }
                                    });
                                ui.add_space(4.0);
                            });
                        }
                    });
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let id_editeur = egui::Id::new("editeur");

        // Raccourcis clavier
        // Basculer le panneau latéral droit (F9 ou Ctrl+B)
        if ctx.input(|i| i.key_pressed(egui::Key::F9) || (i.modifiers.ctrl && i.key_pressed(egui::Key::B))) {
            self.panneau_lateral = if self.panneau_lateral == PanneauLateral::Detections {
                PanneauLateral::Aucun
            } else {
                PanneauLateral::Detections
            };
        }

        if ctx.input_mut(|i| i.consume_shortcut(&OUVRIR)) {
            if let Some(f) = rfd::FileDialog::new().add_filter("Documents", &["docx", "txt", "md"]).pick_file() {
                self.ouvre(f);
            }
        }
        if ctx.input_mut(|i| i.consume_shortcut(&ENREGISTRER)) {
            self.enregistre(false);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&AJOUT_PRESSE_PAPIERS)) {
            self.ajoute_presse_papiers();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&AJOUT_SELECTION)) {
            self.ajoute_selection();
        }
        if let Some(f) = ctx.input(|i| i.raw.dropped_files.first().and_then(|f| f.path.clone())) {
            self.ouvre(f);
        }

        // les correcteurs lisent le texte sans les marques de titre (« # »), aux mêmes positions
        let texte = ecriture::sans_marques(&self.texte);
        let marque = if self.modifie { "● " } else { "" };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("{marque}{} — Texteur", self.titre)));

        if ctx.input(|i| i.viewport().close_requested()) && self.modifie && !self.fermeture_confirmee {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.fermeture_confirmee = true;
            self.message = "Modifications non enregistrées : fermez à nouveau pour quitter.".into();
        }

        // Vérification orthographique Windows en tâche de fond
        if self.derniere_frappe.elapsed() >= DELAI_VERIF {
            self.ortho.verifie(&texte);
        } else {
            ctx.request_repaint_after(DELAI_VERIF);
        }

        // Grammalecte en direct, après la même courte pause de frappe
        if let Some(Ok(res)) = self.rx_gl.as_ref().map(|rx| rx.try_recv()) {
            self.rx_gl = None;
            match res {
                Ok(alertes) => {
                    self.alertes_gl = alertes;
                    if crate::ia::analyse_incomplete() {
                        // relecture complète dès que spaCy et le détecteur sont chargés
                        self.hash_gl = 0;
                        self.relance_gl = Instant::now() + Duration::from_secs(5);
                    }
                }
                Err(e) => {
                    self.gl_indisponible = true;
                    self.message = format!("⚠ Grammalecte indisponible : {e}");
                }
            }
            self.cache_analyse = None;
        }
        if self.rx_gl.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else if !self.gl_indisponible && self.derniere_frappe.elapsed() >= DELAI_VERIF && Instant::now() >= self.relance_gl
            && hash_texte(&texte) != self.hash_gl
        {
            self.lancer_grammalecte(false);
        }

        // Mises à jour : au démarrage puis toutes les 10 minutes (l'entraînement peut livrer un modèle pendant la séance)
        if let Some(Ok(nouveau)) = self.rx_maj.as_ref().map(|rx| rx.try_recv()) {
            self.rx_maj = None;
            self.maj_modele = nouveau;
            if std::mem::take(&mut self.maj_signalee) {
                self.message = match (&self.maj_texteur, nouveau) {
                    (None, false) => "✔ Texteur et le modèle du détecteur sont à jour.".into(),
                    _ => "⬆ Mise à jour disponible : voir le bouton « ⬆ Mise à jour » en haut.".into(),
                };
            }
        }
        if self.rx_maj.is_none() && Instant::now() >= self.prochaine_verif_maj {
            self.verifier_maj(false);
        }

        // Analyse unifiée (Orthographe + Grammaire + Style), recalculée seulement si quelque chose a changé
        let cle_analyse = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            texte.hash(&mut h);
            self.masque.plages(&texte).hash(&mut h);
            let mut ign: Vec<&String> = self.ignores.iter().collect();
            ign.sort();
            ign.hash(&mut h);
            format!("{:?}", self.onglet).hash(&mut h);
            self.repetition_selectionnee.hash(&mut h);
            for r in &self.grammaire.banque_regles.regles {
                (&r.id, r.est_applicable()).hash(&mut h);
            }
            if let Ok(e) = self.ortho.etat.lock() {
                (e.cache.len(), &e.langue).hash(&mut h);
            }
            for a in &self.alertes_gl {
                (a.debut, a.fin).hash(&mut h);
            }
            h.finish()
        };
        let (plages, alertes, repetitions) = match &self.cache_analyse {
            Some((k, r)) if *k == cle_analyse => r.clone(),
            _ => {
                let (plages, mut alertes, repetitions) = analyse_unifiee(
                    &texte,
                    &self.masque,
                    &self.ortho.etat,
                    &self.grammaire,
                    &self.style,
                    &self.ignores,
                    self.onglet,
                    self.repetition_selectionnee.as_deref(),
                );
                // Relecture fusionnée (Grammalecte jugé par l'IA) d'abord, puis Grammalecte en direct pour les
                // phrases pas encore relues : gardées tant que le mot n'a pas changé, jamais en zone chinoise
                // Règles Rust trop grossières (sur un livre de 390 pages : ~3 600 alertes, presque toutes fausses) :
                // Grammalecte couvre les mêmes fautes avec précision, on ne les garde que s'il est absent
                alertes.retain(|a| {
                    a.type_alerte != TypeAlerte::Grammaire
                        || !(a.mot.chars().any(|c| c.is_ascii_digit()) && !tournures::REGLES_AVEC_CHIFFRES.contains(&a.regle.as_str())
                            || !self.gl_indisponible && REGLES_COUVERTES_PAR_GRAMMALECTE.contains(&a.regle.as_str()))
                });
                let car: Vec<char> = texte.chars().collect();
                let valide = |a: &AlerteG| a.fin <= car.len() && car[a.debut..a.fin].iter().collect::<String>() == a.mot;
                let gl = self.alertes_gl.iter().filter(|a| valide(a) && !zone_chinoise(&car, a.debut, a.fin));
                for a in gl {
                    if !self.ignores.contains(&a.mot) && !alertes.iter().any(|x| chevauche((x.debut, x.fin), (a.debut, a.fin))) {
                        alertes.push(a.clone());
                    }
                }
                alertes.sort_by_key(|a| (a.debut, a.fin));
                let r = (plages, alertes, repetitions);
                self.cache_analyse = Some((cle_analyse, r.clone()));
                r
            }
        };

        // Mode écriture : Alt+↑ / Alt+↓ déplacent la section en cours (avant que l'éditeur ne lise ces touches)
        if self.onglet == OngletActif::Ecriture {
            let (haut, bas) = ctx.input_mut(|i| (i.consume_key(Modifiers::ALT, Key::ArrowUp), i.consume_key(Modifiers::ALT, Key::ArrowDown)));
            if haut || bas {
                if let Some(c) = ecriture::courante(&ecriture::plan(&self.texte), self.selection.0) {
                    self.action_plan(ActionPlan::Deplacer(c, haut));
                }
            }
        }

        // F8 : Sauter à la faute suivante
        if ctx.input_mut(|i| i.consume_shortcut(&FAUTE_SUIVANTE)) {
            let apres = self.selection.1;
            if let Some(a) = alertes.iter().find(|al| al.debut >= apres).or(alertes.first()) {
                self.selection_demandee = Some((a.debut, a.fin));
                ctx.memory_mut(|m| m.request_focus(id_editeur));
            }
        }

        // Revue rapide au clavier : la faute sous le curseur se corrige sans souris,
        // puis on enchaîne automatiquement sur la suivante.
        let (sel_d, sel_f) = self.selection;
        if let Some(a) = alertes.iter().find(|a| a.debut <= sel_d && sel_f <= a.fin && a.type_alerte != TypeAlerte::Repetition).cloned() {
            let (choix, ignorer) = ctx.input_mut(|i| {
                let mut c = None;
                for (k, n) in [(Key::Enter, 0), (Key::Num1, 0), (Key::Num2, 1), (Key::Num3, 2)] {
                    if i.consume_key(Modifiers::ALT, k) {
                        c = Some(n);
                    }
                }
                (c, i.consume_key(Modifiers::ALT, Key::Delete) || i.consume_key(Modifiers::ALT, Key::Backspace))
            });
            let suivante = alertes.iter().find(|x| x.debut >= a.fin && x.type_alerte != TypeAlerte::Repetition).cloned();
            if let Some(s) = choix.and_then(|n| a.suggestions.get(n)).cloned() {
                self.remplace(a.debut, a.fin, &s);
                let delta = s.chars().count() as isize - (a.fin - a.debut) as isize;
                if let Some(n) = suivante {
                    let dec = |p: usize| (p as isize + delta).max(0) as usize;
                    self.selection_demandee = Some((dec(n.debut), dec(n.fin)));
                }
                self.message = format!("✓ « {} » → « {} »", a.mot, s);
                ctx.memory_mut(|m| m.request_focus(id_editeur));
            } else if ignorer {
                self.ignorer(&a.mot);
                if let Some(n) = suivante {
                    self.selection_demandee = Some((n.debut, n.fin));
                }
                ctx.memory_mut(|m| m.request_focus(id_editeur));
            }

            // Carte de revue flottante
            egui::Area::new(egui::Id::new("carte_revue"))
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -34.0))
                .interactable(false)
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(22, 27, 40))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)))
                        .inner_margin(egui::Margin::symmetric(12, 6))
                        .corner_radius(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("« {} »", a.mot)).strong().color(Color32::from_rgb(252, 165, 165)));
                                for (n, s) in a.suggestions.iter().take(3).enumerate() {
                                    ui.label(egui::RichText::new(format!("Alt+{} {}", n + 1, s)).color(Color32::from_rgb(134, 239, 172)));
                                }
                                ui.label(egui::RichText::new("· Alt+Entrée accepter · Alt+Suppr ignorer · F8 suivante").small().color(Color32::GRAY));
                            });
                        });
                });
        }

        // Barre d'outils supérieure stylisée
        egui::TopBottomPanel::top("barre")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(16, 18, 25))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 38, 52)))
                    .inner_margin(egui::Margin::symmetric(14, 8)),
            )
            .show(ctx, |ui| {
                self.barre(ui, alertes.iter().filter(|a| a.type_alerte != TypeAlerte::Repetition).count(), repetitions.len());
            });

        // Barre d'état inférieure moderne avec puces colorées
        egui::TopBottomPanel::bottom("statut")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(14, 16, 22))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 33, 46)))
                    .inner_margin(egui::Margin::symmetric(14, 7)),
            )
            .show(ctx, |ui| {
                let mots = texte.split_whitespace().count();
                let proteges = plages.len();
                let nb_gram = alertes.iter().filter(|a| a.type_alerte == TypeAlerte::Grammaire).count();
                let nb_ortho = alertes.iter().filter(|a| a.type_alerte == TypeAlerte::Orthographe).count();
                let nb_rep = repetitions.len();

                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(format!("📄 {mots} mots")).strong());
                    ui.separator();
                    if nb_gram > 0 {
                        ui.colored_label(Color32::from_rgb(52, 211, 153), format!("● {nb_gram} grammaire"));
                        ui.separator();
                    }
                    if nb_ortho > 0 {
                        ui.colored_label(Color32::from_rgb(248, 113, 113), format!("● {nb_ortho} orthographe"));
                        ui.separator();
                    }
                    if nb_rep > 0 {
                        ui.colored_label(Color32::from_rgb(196, 181, 253), format!("● {nb_rep} répétitions"));
                        ui.separator();
                    }
                    let nb_cjk = texte.chars().filter(|&c| masque::est_cjk(c)).count();
                    if nb_cjk > 0 {
                        ui.colored_label(Color32::from_rgb(56, 189, 248), format!("🇨🇳 {nb_cjk} car. chinois reconnus"));
                        ui.separator();
                    } else if proteges > 0 {
                        ui.colored_label(Color32::from_rgb(56, 189, 248), format!("🛡️ {proteges} protégé(s)"));
                        ui.separator();
                    }
                    ui.label(egui::RichText::new(&self.message).weak().italics());
                });
            });

        // Panneau latéral DROIT : Détections ou Dictionnaires
        match self.panneau_lateral {
            PanneauLateral::Detections => {
                egui::SidePanel::right("panneau_lateral_droit")
                    .default_width(340.0)
                    .min_width(280.0)
                    .max_width(520.0)
                    .frame(
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(16, 19, 26))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 38, 52)))
                            .inner_margin(egui::Margin::symmetric(14, 12)),
                    )
                    .show(ctx, |ui| {
                        self.panneau_liste_fautes(ui, &alertes, &repetitions, id_editeur);
                    });
            }
            PanneauLateral::Dictionnaires => {
                egui::SidePanel::right("panneau_lateral_droit")
                    .default_width(380.0)
                    .min_width(300.0)
                    .max_width(550.0)
                    .frame(
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(16, 19, 26))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 38, 52)))
                            .inner_margin(egui::Margin::symmetric(14, 12)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📖 Dictionnaires").size(17.0).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("✕").on_hover_text("Fermer le panneau").clicked() {
                                    self.panneau_lateral = PanneauLateral::Aucun;
                                }
                            });
                        });
                        ui.separator();
                        self.panneau_dictionnaires(ui);
                    });
            }
            PanneauLateral::Aucun => {
                let nb_alertes = alertes.len();
                let nb_rep = repetitions.len();
                egui::SidePanel::right("volet_replie_droit")
                    .resizable(false)
                    .exact_width(42.0)
                    .frame(
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(18, 22, 30))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)))
                            .inner_margin(egui::Margin::symmetric(4, 12))
                    )
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            let btn_reouvrir = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("◀")
                                        .size(16.0)
                                        .color(Color32::from_rgb(52, 211, 153))
                                        .strong()
                                )
                                .fill(Color32::from_rgb(26, 36, 48))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(52, 211, 153)))
                            ).on_hover_text("Ouvrir le panneau latéral des alertes (F9 ou Ctrl+B)");

                            if btn_reouvrir.clicked() {
                                self.panneau_lateral = PanneauLateral::Detections;
                            }

                            ui.add_space(8.0);
                            if nb_alertes > 0 {
                                if ui.button(egui::RichText::new(format!("⚠️\n{nb_alertes}")).color(Color32::from_rgb(248, 113, 113)).size(12.0).strong())
                                    .on_hover_text("Ouvrir les alertes de grammaire et d'orthographe (F9)").clicked() {
                                    self.panneau_lateral = PanneauLateral::Detections;
                                }
                            } else if nb_rep > 0 {
                                if ui.button(egui::RichText::new(format!("◈\n{nb_rep}")).color(Color32::from_rgb(167, 139, 250)).size(12.0).strong())
                                    .on_hover_text("Ouvrir les répétitions de style (F9)").clicked() {
                                    self.panneau_lateral = PanneauLateral::Detections;
                                }
                            } else {
                                ui.label(egui::RichText::new("✨\nOK").color(Color32::from_rgb(52, 211, 153)).size(11.0));
                            }
                        });
                    });
            }
        }

        // Mode écriture : plan à gauche
        if self.onglet == OngletActif::Ecriture {
            egui::SidePanel::left("plan")
                .default_width(300.0)
                .min_width(230.0)
                .max_width(480.0)
                .frame(
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(16, 19, 26))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 38, 52)))
                        .inner_margin(egui::Margin::symmetric(14, 12)),
                )
                .show(ctx, |ui| self.panneau_plan(ui, &alertes));
        }
        let ecriture = self.onglet == OngletActif::Ecriture;
        let focus = if ecriture && self.concentration {
            let plan = ecriture::plan(&self.texte);
            ecriture::courante(&plan, self.selection.0).map(|c| (plan[c].debut, plan[c].fin))
        } else {
            None
        };
        let souligner = !ecriture || self.soulignements_ecriture;

        // Panneau central de texte avec canvas papier épuré
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(13, 15, 20))
                    .inner_margin(egui::Margin::symmetric(16, 12)),
            )
            .show(ctx, |ui| {
                // Ruban de suggestions et actions contextuelles stylisé en carte flottante
                egui::Frame::NONE
                    .fill(Color32::from_rgb(20, 24, 33))
                    .corner_radius(egui::CornerRadius::same(8))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 43, 58)))
                    .inner_margin(egui::Margin::symmetric(14, 9))
                    .show(ui, |ui| {
                        if souligner {
                            self.barre_suggestions(ui, &alertes, &repetitions);
                        } else {
                            ui.label(egui::RichText::new("✎ Écriture libre — fautes masquées (cochez « Souligner les fautes » dans le plan)").weak());
                        }
                    });

                ui.add_space(8.0);

                let masque = self.masque.clone();
                let plages_repetitions: Vec<(usize, usize)> = if !ecriture && self.surligner_repetitions || self.onglet == OngletActif::Repetitions {
                    repetitions.iter().flat_map(|r| r.occurrences.iter().copied()).collect()
                } else {
                    Vec::new()
                };

                let repetition_active_plages: Vec<(usize, usize)> = if let Some(ref cible) = self.repetition_selectionnee {
                    repetitions.iter().filter(|r| r.racine.eq_ignore_ascii_case(cible)).flat_map(|r| r.occurrences.iter().copied()).collect()
                } else {
                    Vec::new()
                };

                let mut layouter = |ui: &egui::Ui, texte: &str, largeur: f32| {
                    colore(ui, &masque.plages(texte), &plages_repetitions, &repetition_active_plages, focus, texte, largeur)
                };

                if let Some((a, b)) = self.selection_demandee.take() {
                    let mut st = egui::TextEdit::load_state(ctx, id_editeur).unwrap_or_default();
                    st.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(a), CCursor::new(b))));
                    st.store(ctx, id_editeur);
                    self.selection = (a.min(b), a.max(b));
                }

                // Feuille / Document paper card avec marges aérées
                egui::Frame::NONE
                    .fill(Color32::from_rgb(18, 21, 28))
                    .corner_radius(egui::CornerRadius::same(8))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 37, 50)))
                    .inner_margin(egui::Margin::symmetric(24, 18))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                            let sortie = egui::TextEdit::multiline(&mut self.texte)
                                .id(id_editeur)
                                .desired_width(f32::INFINITY)
                                .desired_rows(28)
                                .lock_focus(true)
                                .frame(false)
                                .layouter(&mut layouter)
                                .show(ui);

                            if sortie.response.changed() {
                                self.modifie = true;
                                self.fermeture_confirmee = false;
                                self.derniere_frappe = Instant::now();
                            }

                            if let Some(c) = sortie.cursor_range {
                                let (a, b) = (c.primary.ccursor.index, c.secondary.ccursor.index);
                                self.selection = (a.min(b), a.max(b));
                            }

                            // Affichage des vagues colorées (vert, rouge, lavande)
                            if souligner {
                                vagues(ui.painter(), &sortie.galley, sortie.galley_pos, &alertes);
                            }
                            // titre choisi dans le plan : amené en haut de l'éditeur
                            if let Some(p) = self.defiler_vers.take() {
                                let r = sortie.galley.pos_from_ccursor(CCursor::new(p)).translate(sortie.galley_pos.to_vec2());
                                ui.scroll_to_rect(r.expand2(egui::vec2(0.0, 40.0)), Some(egui::Align::TOP));
                            }
                        });
                    });
            });
    }
}

fn executer_cli_ia(args: &[String]) {
    let chemin_fichier = args.iter().skip(1).find(|a| !a.starts_with('-'));
    let Some(chemin_str) = chemin_fichier else {
        println!("Texteur CLI IA — Usage : Texteur.exe <document.docx|txt|md> --ia [--json] [--sortie <rapport>]");
        return;
    };
    let chemin = PathBuf::from(chemin_str);
    let est_docx = chemin.extension().is_some_and(|e| e.eq_ignore_ascii_case("docx"));
    let texte = if est_docx {
        docx::lis(&chemin).unwrap_or_default()
    } else {
        std::fs::read_to_string(&chemin).unwrap_or_default().trim_start_matches('\u{feff}').replace("\r\n", "\n")
    };
    if texte.trim().is_empty() {
        println!("Document vide ou illisible : {}", chemin.display());
        return;
    }

    let biblio = Bibliotheque::charge();
    let masque = Masque::new(biblio.formes_actives(), biblio.reglages.proteger_ecritures);
    let grammaire = CorrecteurGrammaire::new();
    let style = AnalyseurStyle::new();

    let plages = masque.plages(&texte);
    let mut alertes = Vec::new();

    // 1. Alertes de grammaire et accords (mêmes règles que l'éditeur)
    for ag in grammaire.analyse(&texte).into_iter().chain(tournures::analyse(&texte)) {
        if plages.iter().any(|&p| chevauche(p, (ag.debut, ag.fin))) {
            continue;
        }
        let t_type = if ag.genre == GenreErreur::Doublon {
            TypeAlerte::Doublon
        } else {
            TypeAlerte::Grammaire
        };
        alertes.push(AlerteG {
            debut: ag.debut,
            fin: ag.fin,
            mot: ag.texte_fautif,
            suggestions: ag.suggestions,
            regle: ag.regle,
            explication: ag.explication,
            type_alerte: t_type,
        });
    }

    // 2. Répétitions de style
    let repetitions = style.analyse(&texte);

    let format_json = args.iter().any(|a| a == "--json");
    let nom_doc = chemin.file_name().unwrap_or_default().to_string_lossy().to_string();
    let rapport = RapportIA::generer(&texte, &alertes, &repetitions, &nom_doc);

    let contenu = if format_json {
        rapport.vers_json()
    } else {
        rapport.vers_markdown()
    };

    // Chercher argument --sortie ou sauvegarder à côté du fichier
    let idx_sortie = args.iter().position(|a| a == "--sortie" || a == "-o");
    let chemin_sortie = if let Some(idx) = idx_sortie {
        args.get(idx + 1).map(PathBuf::from)
    } else {
        let ext = if format_json { "json" } else { "md" };
        let tige = chemin.file_stem().unwrap_or_default().to_string_lossy();
        Some(chemin.with_file_name(format!("{tige}_rapport_ia.{ext}")))
    };

    if let Some(dest) = &chemin_sortie {
        let _ = std::fs::write(dest, &contenu);
        println!("Rapport IA généré : {}", dest.display());
    }
    print!("{contenu}");
}


/// Génère l'icône officielle de Texteur (Plume émeraude & Monogramme 'T' sur fond sombre)
fn charger_icone_texteur() -> egui::IconData {
    let raw = include_bytes!("../ressources/texteur_64.rgba");
    egui::IconData {
        rgba: raw.to_vec(),
        width: 64,
        height: 64,
    }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--ia" || a == "--rapport-ia" || a == "-ia") {
        executer_cli_ia(&args);
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 720.0])
            .with_drag_and_drop(true)
            .with_icon(charger_icone_texteur()),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native("Texteur", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostique_analyse_unifiee() {
        let biblio = Bibliotheque::charge();
        let masque = Masque::new(biblio.formes_actives(), biblio.reglages.proteger_ecritures);
        let grammaire = CorrecteurGrammaire::new();
        let style = AnalyseurStyle::new();
        let ortho = Mutex::new(EtatOrtho::default());
        let ignores = HashSet::new();

        let texte = "Bonjour, je te envoie le document concernent le projet 長城 (Chángchéng).\nLes fournisseur du Shenzhen 深圳 ont confirmer la livraison du projet.\nNous devons faire un réunion pour faire la point sur se problème et raigler ce problème rapidement.";

        let (plages, alertes, _repetitions) = analyse_unifiee(
            texte,
            &masque,
            &ortho,
            &grammaire,
            &style,
            &ignores,
            OngletActif::Correction,
            None,
        );

        println!("=== PLAGES DU MASQUE ===");
        let chars: Vec<char> = texte.chars().collect();
        for (a, b) in &plages {
            let s: String = chars[*a..*b].iter().collect();
            println!("  Plage [{}..{}] = '{}'", a, b, s);
        }

        let texte2 = "Bonjour, je LE vous envoieS le document concernant le projet 長城 (Chángchéng).\nLes fournisseur de Shenzhen 深圳 ont confirmer la livraison du projet.\nNous devons faire un réunion pour faire sa point sur ce problème et régler ce problème rapidement.";

        let (_plages2, alertes2, _repetitions2) = analyse_unifiee(
            texte2,
            &masque,
            &ortho,
            &grammaire,
            &style,
            &ignores,
            OngletActif::Correction,
            None,
        );

        println!("=== ALERTES IMAGE 1 ===");
        for a in &alertes2 {
            println!("  [{:?}] [{}] '{}' ({}-{}) -> {:?}", a.type_alerte, a.regle, a.mot, a.debut, a.fin, a.suggestions);
        }

        println!("=== ALERTES DE L'ANALYSE UNIFIÉE ===");
        for a in &alertes {
            println!("  [{:?}] [{}] '{}' ({}-{}) -> {:?}", a.type_alerte, a.regle, a.mot, a.debut, a.fin, a.suggestions);
        }

        // Assertions strictes garantissant 100 % de détection sur les images 1 et 2
        let fautes_img2: Vec<String> = alertes.iter().map(|a| a.mot.clone()).collect();
        assert!(fautes_img2.contains(&"te envoie".to_string()), "Élision 'te envoie' manquante");
        assert!(fautes_img2.contains(&"concernent".to_string()), "'concernent' manquant");
        assert!(fautes_img2.contains(&"fournisseur".to_string()), "'fournisseur' manquant");
        assert!(fautes_img2.contains(&"du".to_string()), "'du' devant Shenzhen manquant");
        assert!(fautes_img2.contains(&"confirmer".to_string()), "'confirmer' manquant");
        assert!(fautes_img2.contains(&"un".to_string()), "'un réunion' manquant");
        assert!(fautes_img2.contains(&"la".to_string()), "'la point' manquant");
        assert!(fautes_img2.contains(&"se".to_string()), "'se problème' manquant");

        let fautes_img1: Vec<String> = alertes2.iter().map(|a| a.mot.clone()).collect();
        assert!(fautes_img1.contains(&"LE vous".to_string()), "Ordre 'LE vous' manquant");
        assert!(fautes_img1.contains(&"fournisseur".to_string()), "'fournisseur' manquant");
        assert!(fautes_img1.contains(&"confirmer".to_string()), "'confirmer' manquant");
        assert!(fautes_img1.contains(&"un".to_string()), "'un réunion' manquant");
        assert!(fautes_img1.contains(&"sa".to_string()), "'sa point' manquant");
    }
}

#[cfg(test)]
mod tests_grammalecte {
    use super::*;

    #[test]
    #[ignore = "démarre le serveur Grammalecte (Python 3.12)"]
    fn grammalecte_de_bout_en_bout() {
        let texte = "Titre\nLes fournisseur de 深圳 (Shēnzhèn) ont envoyer les pièce.\n\nOn est pas sûr ; il faut que tu viens.";
        let cache = CacheGL::default();
        let t = std::time::Instant::now();
        let port = crate::ia::assurer_grammalecte().expect("démarrage");
        let alertes = alertes_grammalecte(texte, port, &cache).expect("vérification");
        let car: Vec<char> = texte.chars().collect();
        for a in &alertes {
            assert_eq!(car[a.debut..a.fin].iter().collect::<String>(), a.mot);
            println!("{} -> {:?} [{}]", a.mot, a.suggestions, a.regle);
        }
        let mots: Vec<&str> = alertes.iter().map(|a| a.mot.as_str()).collect();
        for m in ["fournisseur", "envoyer", "pièce", "est", "viens"] {
            assert!(mots.contains(&m), "{m} manquant : {mots:?}");
        }
        println!("démarrage + vérification : {:?}", t.elapsed());
        let t = std::time::Instant::now();
        alertes_grammalecte(&format!("{texte}\nUne nouvelle phrase."), port, &cache).unwrap();
        println!("après modification (cache) : {:?}", t.elapsed());
    }
}

#[cfg(test)]
mod tests_livre {
    use super::*;

    /// TEXTEUR_LIVRE=dossier : relit chaque NN.txt avec les règles Rust, écrit NN_rust.json
    #[test]
    #[ignore = "relecture d'un livre entier"]
    fn regles_rust_sur_un_livre() {
        let dossier = std::path::PathBuf::from(std::env::var("TEXTEUR_LIVRE").expect("TEXTEUR_LIVRE"));
        let biblio = Bibliotheque::charge();
        let masque = Masque::new(biblio.formes_actives(), biblio.reglages.proteger_ecritures);
        let (grammaire, style) = (CorrecteurGrammaire::new(), AnalyseurStyle::new());
        let (ortho, ignores) = (Mutex::new(EtatOrtho::default()), HashSet::new());
        for n in 0..11 {
            let Ok(texte) = std::fs::read_to_string(dossier.join(format!("{n:02}.txt"))) else { continue };
            let t = std::time::Instant::now();
            let (_, alertes, _) = analyse_unifiee(&texte, &masque, &ortho, &grammaire, &style, &ignores, OngletActif::Correction, None);
            let car: Vec<char> = texte.chars().collect();
            let sortie: Vec<serde_json::Value> = alertes
                .iter()
                .filter(|a| a.type_alerte == TypeAlerte::Grammaire)
                .map(|a| {
                    let (d, f) = (a.debut.saturating_sub(60), (a.fin + 60).min(car.len()));
                    serde_json::json!({"d": a.debut, "f": a.fin, "fautif": a.mot, "sugg": a.suggestions, "regle": a.regle,
                        "contexte": car[d..f].iter().collect::<String>().replace('\n', " ⏎ ")})
                })
                .collect();
            println!("{n:02} : {} alertes de grammaire Rust en {:?}", sortie.len(), t.elapsed());
            std::fs::write(dossier.join(format!("{n:02}_rust.json")), serde_json::to_string_pretty(&sortie).unwrap()).unwrap();
        }
    }
}
