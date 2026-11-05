#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Texteur : éditeur léger, ultra-rapide et 100 % local (sans Word).
//! - Ouvre et modifie les documents Word (.docx), .txt et .md sans ouvrir Word
//! - Détection des fautes d'orthographe (rouge) et de grammaire (vert)
//! - Mode Répétitions & Style avec formulations contextuelles de qualité (violet)
//! - Gestion des dictionnaires personnels et protection des termes spécialisés

mod dico;
mod docx;
mod ecriture;
mod editions;
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
use std::sync::atomic::{AtomicBool, Ordering};
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
    Editions,
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
    /// Fichier à ouvrir malgré un texte non enregistré (deuxième demande = confirmation)
    ouverture_en_attente: Option<PathBuf>,
    /// Titre affiché par la fenêtre (envoyé seulement quand il change)
    titre_fenetre: String,
    /// Volet ouvert avant le mode écriture, rouvert à la sortie
    panneau_avant_ecriture: PanneauLateral,
    nouveau_confirme: bool,
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
    /// Règles d'édition (maisons d'édition et leurs conventions)
    editions: editions::Editions,
    edition_ouverte: usize,
    nouvelle_edition: String,
    nouvelle_regle: editions::RegleEdition,
    collage_regles: Option<String>,
    filtre_regles: String,
    regle_ouverte: Option<usize>,
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

/// Mode clair doux (papier crème, encre brune) ou mode sombre ; lu par rgb() à chaque image.
static CLAIR: AtomicBool = AtomicBool::new(false);

fn clair() -> bool {
    CLAIR.load(Ordering::Relaxed)
}

/// Couleur de l'interface. Les couleurs sont pensées pour le mode sombre ; en mode clair elles sont traduites :
/// les fonds sombres deviennent un papier crème (jamais blanc pur), les textes clairs une encre brune,
/// les couleurs vives sont assombries pour rester lisibles sur le papier.
fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    if !clair() {
        return Color32::from_rgb(r, g, b);
    }
    let (h, s, l) = vers_tsl(r, g, b);
    let chroma = (r.max(g).max(b) - r.min(g).min(b)) as f32 / 255.0; // vivacité réelle (un blanc bleuté n'est pas vif)
    let (h2, s2, l2) = if chroma >= 0.25 && l >= 0.4 {
        (h, s * 0.85, (l * 0.62).clamp(0.26, 0.45)) // couleur vive : assombrie
    } else if l < 0.4 {
        if chroma >= 0.12 { (h, s.min(0.6) * 0.45, 0.93 - l * 0.4) } else { (40.0, 0.24, 0.945 - l * 0.45) } // fond : papier
    } else if l > 0.6 {
        (30.0, 0.12, 1.0 - l * 0.85) // texte clair : encre
    } else {
        (35.0, 0.08, 0.92 - l * 0.9) // gris moyen
    };
    depuis_tsl(h2, s2, l2)
}

fn vers_tsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let l = (max + min) / 2.0;
    let d = max - min;
    if d < 1e-6 {
        return (0.0, 0.0, l);
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r { ((g - b) / d).rem_euclid(6.0) } else if max == g { (b - r) / d + 2.0 } else { (r - g) / d + 4.0 };
    (h * 60.0, s.min(1.0), l)
}

fn depuis_tsl(h: f32, s: f32, l: f32) -> Color32 {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h / 60.0) as u32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let v = |t: f32| ((t + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(v(r), v(g), v(b))
}

fn appliquer_theme(ctx: &egui::Context) {
        ctx.style_mut(|s| {
            s.visuals = if clair() { egui::Visuals::light() } else { egui::Visuals::dark() };
            s.text_styles.insert(egui::TextStyle::Heading, FontId::proportional(17.5));
            s.text_styles.insert(egui::TextStyle::Body, FontId::proportional(15.5));
            s.text_styles.insert(egui::TextStyle::Button, FontId::proportional(13.5));
            s.text_styles.insert(egui::TextStyle::Small, FontId::proportional(12.5));

            s.spacing.item_spacing = egui::vec2(8.0, 6.0);
            s.spacing.button_padding = egui::vec2(10.0, 5.0);

            s.visuals.dark_mode = !clair();
            s.visuals.panel_fill = rgb(16, 18, 24); // #101218 ardoise sombre
            s.visuals.window_fill = rgb(22, 26, 35); // #161A23 cartes de surface
            s.visuals.extreme_bg_color = rgb(13, 15, 20);

            // Widgets inactifs
            s.visuals.widgets.inactive.bg_fill = rgb(26, 31, 41);
            s.visuals.widgets.inactive.weak_bg_fill = rgb(22, 26, 35);
            s.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, rgb(40, 48, 63));
            s.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
            s.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, rgb(215, 222, 235));

            // Widgets survolés
            s.visuals.widgets.hovered.bg_fill = rgb(38, 45, 60);
            s.visuals.widgets.hovered.weak_bg_fill = rgb(32, 38, 51);
            s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, rgb(65, 77, 102));
            s.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
            s.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, rgb(245, 248, 255));

            // Widgets actifs / cliqués
            s.visuals.widgets.active.bg_fill = rgb(48, 58, 78);
            s.visuals.widgets.active.weak_bg_fill = rgb(42, 51, 68);
            s.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, rgb(99, 102, 241));
            s.visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
            s.visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, rgb(255, 255, 255));

            // Widgets déroulés
            s.visuals.widgets.open.bg_fill = rgb(32, 38, 51);
            s.visuals.widgets.open.corner_radius = egui::CornerRadius::same(6);

            // Sélection de texte moderne
            s.visuals.selection.bg_fill = if clair() { Color32::from_rgba_unmultiplied(99, 102, 241, 70) } else { Color32::from_rgba_unmultiplied(79, 70, 229, 130) };
            s.visuals.selection.stroke = Stroke::new(1.0_f32, rgb(129, 140, 248));
            if clair() {
                // cases à cocher, champs et bordures bien visibles sur le papier
                s.visuals.widgets.inactive.bg_fill = Color32::from_rgb(252, 250, 245);
                s.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(196, 186, 168));
                s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(160, 146, 120));
                s.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(218, 209, 192));
                s.visuals.extreme_bg_color = Color32::from_rgb(252, 250, 245);
            }
        });
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
    // Répétitions : surligneur jaune fluo
    let repet_douce = TextFormat {
        color: if sombre { rgb(235, 228, 205) } else { Color32::from_rgb(40, 34, 20) },
        background: if sombre { Color32::from_rgba_unmultiplied(250, 204, 21, 40) } else { Color32::from_rgba_unmultiplied(255, 226, 70, 80) },
        ..normal.clone()
    };
    // Surlignage accentué pour la répétition activement sélectionnée
    let repet_active = TextFormat {
        color: if sombre { Color32::from_rgb(255, 245, 205) } else { Color32::from_rgb(40, 34, 20) },
        background: if sombre { Color32::from_rgba_unmultiplied(250, 204, 21, 95) } else { Color32::from_rgba_unmultiplied(255, 214, 40, 150) },
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
            let couleur = rgb(240, 244, 255);
            let marque = TextFormat { font_id: FontId::proportional(taille), color: rgb(90, 100, 125), ..normal.clone() };
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
/// - Orange pour la grammaire / accords
/// - Jaune pour les doublons et répétitions
/// - Violet pour les répétitions de style
/// Faute de typographie (espaces, guillemets, tirets, ponctuation) : soulignée en bleu.
fn est_typo(a: &AlerteG) -> bool {
    let r = a.regle.to_lowercase();
    ["espace", "guillemet", "apostrophe", "tiret", "ponctuation", "typo", "insécable", "nbsp", "esp_", "indent", "alinéa"]
        .iter()
        .any(|m| r.contains(m))
}

fn vagues(peintre: &egui::Painter, galley: &egui::Galley, origine: egui::Pos2, alertes: &[AlerteG]) {
    for a in alertes {
        let couleur = match a.type_alerte {
            TypeAlerte::Orthographe => rgb(235, 65, 65), // Rouge franc
            TypeAlerte::Grammaire if est_typo(a) => rgb(59, 130, 246), // Bleu : typographie
            TypeAlerte::Grammaire => rgb(240, 135, 45),   // Orange
            TypeAlerte::Doublon => rgb(234, 179, 8), // jaune, comme les répétitions
            TypeAlerte::Repetition => rgb(234, 179, 8), // jaune
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
        let biblio = Bibliotheque::charge();
        CLAIR.store(biblio.reglages.theme_clair || std::env::var("TEXTEUR_THEME").is_ok_and(|t| t == "clair"), Ordering::Relaxed);
        appliquer_theme(&cc.egui_ctx);
        let masque = Arc::new(Masque::new(biblio.formes_actives(), biblio.reglages.proteger_ecritures));
        let mut app = App {
            texte: TEXTE_ESSAI.into(),
            titre: "Sans titre".into(),
            modifie: false,
            selection_demandee: None,
            ortho: Orthographe::new(cc.egui_ctx.clone(), &biblio.reglages.langue),
            grammaire: CorrecteurGrammaire::new(),
            editions: editions::Editions::charge(),
            edition_ouverte: 0,
            nouvelle_edition: String::new(),
            nouvelle_regle: Default::default(),
            collage_regles: None,
            filtre_regles: String::new(),
            regle_ouverte: None,
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
            ouverture_en_attente: None,
            titre_fenetre: String::new(),
            panneau_avant_ecriture: PanneauLateral::Detections,
            nouveau_confirme: false,
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
        app.grammaire.edition = app.editions.edition_active().cloned();
        if let Some(chemin) = std::env::args().nth(1) {
            app.ouvre(PathBuf::from(chemin));
        }
        app
    }

    fn ouvre(&mut self, chemin: PathBuf) {
        if self.modifie && self.ouverture_en_attente.as_ref() != Some(&chemin) {
            self.message = format!("⚠ Modifications non enregistrées : Ctrl+S pour les garder, ou rouvrez « {} » pour les abandonner.",
                chemin.file_name().unwrap_or_default().to_string_lossy());
            self.ouverture_en_attente = Some(chemin);
            return;
        }
        self.ouverture_en_attente = None;
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
        self.fermeture_confirmee = false;
        let curseur = debut + par.chars().count();
        self.selection_demandee = Some((curseur, curseur));
    }

    /// Génère et copie dans le presse-papiers le rapport d'anomalies avec leurs phrases de contexte
    /// Copie les phrases fautives (avec une consigne) pour les faire corriger par une IA.
    fn copier_passages_ia(&mut self) {
        let texte = ecriture::sans_marques(&self.texte);
        // les alertes affichées (Grammalecte et détecteur compris) ; sinon une analyse locale
        let alertes = match &self.cache_analyse {
            Some((_, (_, a, _))) => a.clone(),
            None => analyse_unifiee(&texte, &self.masque, &self.ortho.etat, &self.grammaire, &self.style, &self.ignores,
                OngletActif::Correction, None).1,
        };
        let alertes: Vec<AlerteG> = alertes.into_iter().filter(|a| a.type_alerte != TypeAlerte::Repetition).collect();
        let n = alertes.len();
        let contenu = RapportIA::generer(&texte, &alertes, &[], &self.titre).vers_passages();
        self.message = match arboard::Clipboard::new().and_then(|mut c| c.set_text(contenu)) {
            Ok(()) => format!("✔ Passages à corriger copiés ({n} fautes) : collez-les dans ChatGPT, Claude, DeepSeek…"),
            Err(e) => format!("Erreur presse-papiers : {e}"),
        };
    }

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
            ui.menu_button("📂 Fichier", |ui| {
                if ui.button("Nouveau document     Ctrl+N").clicked() {
                    ui.close_menu();
                    self.nouveau(String::new(), "Sans titre");
                }
                if ui.button("Nouveau livre (plan type)").on_hover_text("Introduction, 5 chapitres de deux parties, conclusion ; mode écriture").clicked() {
                    ui.close_menu();
                    if self.nouveau(ecriture::modele_livre(5), "Nouveau livre") {
                        if self.onglet != OngletActif::Ecriture {
                            self.bascule_ecriture();
                        }
                        if self.biblio.reglages.pages_visees == 0 {
                            self.biblio.reglages.pages_visees = 200;
                            let _ = self.biblio.sauve_reglages();
                        }
                    }
                }
                ui.separator();
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
                if ui.button("📋 Copier les passages à corriger pour une IA")
                    .on_hover_text("Chaque phrase fautive avec ses fautes et une consigne : collez dans ChatGPT, Claude, DeepSeek…")
                    .clicked()
                {
                    ui.close_menu();
                    self.copier_passages_ia();
                }
                ui.menu_button("🤖 Rapport complet pour une IA", |ui| {
                    if ui.button("📋 Copier (Markdown)").on_hover_text("Fautes + phrases de contexte, prêtes à coller dans ChatGPT, Claude, Gemini…").clicked() {
                        ui.close_menu();
                        self.copier_rapport_ia(false);
                    }
                    if ui.button("📋 Copier (JSON)").clicked() {
                        ui.close_menu();
                        self.copier_rapport_ia(true);
                    }
                    if ui.button("💾 Enregistrer dans un fichier…").clicked() {
                        ui.close_menu();
                        self.exporter_rapport_ia_fichier();
                    }
                });
            });
            ui.separator();

            let ecriture = self.onglet == OngletActif::Ecriture;
            if ui.selectable_label(ecriture, "✎ Écriture")
                .on_hover_text("Mode écriture (Ctrl+E) : page centrée sans distraction, plan des chapitres, objectif de mots")
                .clicked()
            {
                self.bascule_ecriture();
            }

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
                    let mut c = self.biblio.reglages.theme_clair;
                    if ui.checkbox(&mut c, "☀ Mode clair (papier, doux pour les yeux)").changed() {
                        self.biblio.reglages.theme_clair = c;
                        CLAIR.store(c, Ordering::Relaxed);
                        appliquer_theme(ui.ctx());
                        if let Err(e) = self.biblio.sauve_reglages() {
                            self.message = format!("Réglage non enregistré : {e}");
                        }
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
                    let bouton = egui::RichText::new("⬆ Mise à jour").strong().color(rgb(52, 211, 153));
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
                        if self.onglet == OngletActif::Ecriture {
                            self.onglet = OngletActif::Correction; // un volet ouvert quitte le mode écriture
                        }
                    }
                };
                let ed = if self.editions.active.is_empty() { "📘 Règles éditeurs".to_string() } else { format!("📘 {}", abrege(&self.editions.active, 14)) };
                volet(ui, PanneauLateral::Editions, ed, "Règles éditeurs : les conventions de chaque maison d'édition");
                volet(ui, PanneauLateral::Dictionnaires, "📖 Dico".into(), "Dictionnaires personnels, mots et caractères chinois protégés (Ctrl+D : ajouter la sélection)");
                volet(ui, PanneauLateral::Detections, format!("🎯 Fautes  {}", nb_fautes + nb_repet), "F9 — fautes et répétitions (Ctrl+B)");
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
                let btn_sel = egui::Button::new(egui::RichText::new(format!("✔ Protéger « {} » au dictionnaire (Ctrl+D)", sel)).strong().color(rgb(56, 189, 248)))
                    .fill(rgb(20, 36, 48))
                    .stroke(Stroke::new(1.0_f32, rgb(14, 165, 233)));
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
                ui.colored_label(rgb(250, 204, 21), egui::RichText::new("◈ Style & Répétitions").strong());

                if repetitions.is_empty() {
                    ui.label(egui::RichText::new("✨ Vocabulaire riche et varié — aucune répétition lourde détectée.").color(rgb(52, 211, 153)));
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
                        egui::Button::new(egui::RichText::new(label).strong().color(rgb(250, 204, 21)))
                            .fill(rgb(44, 38, 64))
                            .stroke(Stroke::new(1.0_f32, rgb(139, 120, 195)))
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
                    ui.label(egui::RichText::new(format!("« {} »", rep.mot)).strong().color(rgb(250, 204, 21)));

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
                            let btn_alt = egui::Button::new(egui::RichText::new(alt).color(rgb(216, 206, 253)))
                                .fill(rgb(36, 32, 50))
                                .stroke(Stroke::new(1.0_f32, rgb(90, 80, 125)));
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
                    TypeAlerte::Orthographe => (rgb(248, 113, 113), "Orthographe"),
                    TypeAlerte::Grammaire if est_typo(a) => (rgb(96, 165, 250), "Typographie"),
                    TypeAlerte::Grammaire => (rgb(251, 146, 60), "Grammaire"),
                    TypeAlerte::Doublon => (rgb(250, 204, 21), "Doublon"),
                    TypeAlerte::Repetition => (rgb(250, 204, 21), "Style"),
                };

                ui.colored_label(couleur_badge, egui::RichText::new(format!("● {titre_badge}")).strong());
                ui.label(egui::RichText::new(format!("« {} » :", a.mot)).strong());
                ui.label(egui::RichText::new(&a.explication).weak());

                if a.type_alerte == TypeAlerte::Doublon {
                    if ui.button(egui::RichText::new("🗑 Supprimer le doublon").strong().color(rgb(250, 204, 21))).clicked() {
                        let car: Vec<char> = self.texte.chars().collect();
                        let mut deb = a.debut;
                        while deb > 0 && car[deb - 1].is_whitespace() {
                            deb -= 1;
                        }
                        self.remplace(deb, a.fin, "");
                    }
                } else {
                    for s in &a.suggestions {
                        let btn_sugg = egui::Button::new(egui::RichText::new(s).strong().color(rgb(56, 189, 248)))
                            .fill(rgb(22, 34, 46))
                            .stroke(Stroke::new(1.0_f32, rgb(40, 80, 110)));
                        if ui.add(btn_sugg).clicked() {
                            self.remplace(a.debut, a.fin, s);
                        }
                    }
                }

                ui.separator();
                if explication_pedagogique_approfondie(&a.regle, &a.mot, &a.explication).utile() {
                    let btn_info = egui::Button::new(egui::RichText::new(if self.plus_infos_barre { "▲ Moins d'infos" } else { "ℹ Plus d'infos" }).small().color(rgb(147, 197, 253)))
                        .fill(rgb(25, 30, 45));
                    if ui.add(btn_info).clicked() {
                        self.plus_infos_barre = !self.plus_infos_barre;
                    }
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
                ui.colored_label(rgb(250, 204, 21), egui::RichText::new("◈ Répétition").strong());
                ui.label(egui::RichText::new(format!("« {} » ({} fois) :", rep.mot, rep.nombre)).strong());
                let occ_idx = rep.occurrences.iter().position(|&(d, f)| d <= curseur && curseur <= f);
                if !rep.alternatives.is_empty() {
                    ui.label(egui::RichText::new("Remplacer par :").weak().small());
                    for alt in &rep.alternatives {
                        let btn_alt = egui::Button::new(egui::RichText::new(alt).color(rgb(216, 206, 253)))
                            .fill(rgb(36, 32, 50))
                            .stroke(Stroke::new(1.0_f32, rgb(90, 80, 125)));
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
                    ui.colored_label(rgb(52, 211, 153), egui::RichText::new("✨ Document impeccable — aucune anomalie détectée.").strong());
                } else {
                    ui.label(egui::RichText::new(format!("💡 Orange : grammaire · bleu : typographie · rouge : orthographe · jaune : répétitions · F8 : faute suivante")).weak());
                }
            }
        });

        // Volet pédagogique dépliable « Plus d'infos » dans la barre inférieure
        if self.plus_infos_barre {
            if let Some(fiche) = sous_curseur.as_ref().map(|a| explication_pedagogique_approfondie(&a.regle, &a.mot, &a.explication)).filter(|f| f.utile()) {
                egui::Frame::NONE
                    .fill(rgb(18, 22, 32))
                    .stroke(Stroke::new(1.0_f32, rgb(55, 75, 110)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new(format!("📘 {}", fiche.titre)).strong().color(rgb(147, 197, 253)));
                            ui.label(egui::RichText::new(format!("— {}", fiche.regle_complete)).weak().small());
                        });
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            egui::Frame::NONE
                                .fill(rgb(22, 34, 28))
                                .stroke(Stroke::new(1.0_f32, rgb(34, 197, 94)))
                                .corner_radius(egui::CornerRadius::same(5))
                                .inner_margin(egui::Margin::symmetric(8, 4))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(format!("💡 Astuce mnémotechnique :
{}", fiche.astuce)).strong().color(rgb(134, 239, 172)).small());
                                });
                            ui.label(egui::RichText::new(format!("Exemple : {}", fiche.exemple_correct)).small());
                        });
                    });
            }
        }
    }


    /// Volet « Éditions » : choisir l'édition appliquée, créer des éditions et leurs règles maison.
    fn panneau_editions(&mut self, ui: &mut egui::Ui) {
        let mut change = false;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📘 Règles éditeurs").size(17.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").on_hover_text("Fermer le panneau").clicked() {
                    self.panneau_lateral = PanneauLateral::Aucun;
                }
            });
        });
        ui.label(egui::RichText::new("Les conventions de chaque éditeur, signalées comme des fautes dans le texte.").weak().small());
        ui.add_space(6.0);

        // Édition appliquée au texte
        ui.horizontal(|ui| {
            ui.label("Appliquer :");
            let actuelle = if self.editions.active.is_empty() { "Aucune".to_string() } else { self.editions.active.clone() };
            egui::ComboBox::from_id_salt("edition_active").selected_text(actuelle).width(200.0).show_ui(ui, |ui| {
                change |= ui.selectable_value(&mut self.editions.active, String::new(), "Aucune").changed();
                for e in &self.editions.liste {
                    change |= ui.selectable_value(&mut self.editions.active, e.nom.clone(), &e.nom).changed();
                }
            });
        });
        ui.separator();

        // Liste des éditions
        ui.horizontal(|ui| {
            let champ = ui.add(egui::TextEdit::singleline(&mut self.nouvelle_edition).hint_text("Nouvel éditeur…").desired_width(190.0));
            let nom = self.nouvelle_edition.trim().to_string();
            let libre = !nom.is_empty() && !self.editions.liste.iter().any(|e| e.nom.eq_ignore_ascii_case(&nom));
            let entree = champ.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if (ui.add_enabled(libre, egui::Button::new("＋ Créer")).clicked() || entree && libre) {
                self.editions.liste.push(crate::editions::Edition { nom, regles: vec![] });
                self.edition_ouverte = self.editions.liste.len() - 1;
                self.nouvelle_edition.clear();
                change = true;
            }
        });
        if self.editions.liste.is_empty() {
            ui.label(egui::RichText::new("Aucun éditeur : créez-en un ci-dessus.").weak());
            if change {
                self.editions_modifiees();
            }
            return;
        }
        self.edition_ouverte = self.edition_ouverte.min(self.editions.liste.len() - 1);
        ui.horizontal_wrapped(|ui| {
            for (i, e) in self.editions.liste.iter().enumerate() {
                let n = e.regles.len();
                if ui.selectable_label(i == self.edition_ouverte, format!("{}  ({n})", e.nom)).clicked() {
                    self.edition_ouverte = i;
                }
            }
        });
        ui.add_space(4.0);

        let i = self.edition_ouverte;
        let mut supprimer_edition = false;
        ui.horizontal(|ui| {
            ui.label("Nom :");
            let ancien = self.editions.liste[i].nom.clone();
            let r = ui.add(egui::TextEdit::singleline(&mut self.editions.liste[i].nom).desired_width(170.0));
            if r.changed() {
                if self.editions.active == ancien {
                    self.editions.active = self.editions.liste[i].nom.clone();
                }
                change = true;
            }
            if ui.button("🗑").on_hover_text("Supprimer cet éditeur et ses règles").clicked() {
                supprimer_edition = true;
            }
        });
        if supprimer_edition {
            let nom = self.editions.liste.remove(i).nom;
            if self.editions.active == nom {
                self.editions.active.clear();
            }
            self.editions_modifiees();
            return;
        }

        // Ajouter une règle : une ligne, Entrée pour valider
        egui::Frame::NONE
            .fill(rgb(22, 26, 35))
            .stroke(Stroke::new(1.0_f32, rgb(40, 48, 63)))
            .corner_radius(8)
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let l = ((ui.available_width() - 60.0) / 2.0).max(80.0);
                    let a = ui.add(egui::TextEdit::singleline(&mut self.nouvelle_regle.motif).hint_text("à éviter (ex. Mr)").desired_width(l));
                    ui.label("→");
                    let b = ui.add(egui::TextEdit::singleline(&mut self.nouvelle_regle.remplacement).hint_text("à écrire (ex. M.)").desired_width(l));
                    let entree = (a.lost_focus() || b.lost_focus()) && ui.input(|i| i.key_pressed(Key::Enter));
                    let pret = !self.nouvelle_regle.motif.trim().is_empty();
                    if (ui.add_enabled(pret, egui::Button::new("＋")).on_hover_text("Ajouter la règle (Entrée)").clicked() || entree) && pret {
                        let r = std::mem::take(&mut self.nouvelle_regle);
                        self.nouvelle_regle.mot_entier = r.mot_entier;
                        self.nouvelle_regle.casse = r.casse;
                        let lu = editions::regles_depuis(vec![vec![r.motif, r.remplacement, r.explication]]);
                        let lu: Vec<_> = lu.into_iter().map(|x| editions::RegleEdition { casse: r.casse, ..x }).collect();
                        let (aj, _) = editions::fusionne(&mut self.editions.liste[i], lu);
                        self.message = if aj > 0 { "✔ Règle ajoutée.".into() } else { "✔ Règle mise à jour (motif déjà connu).".into() };
                        change = true;
                        a.request_focus();
                    }
                });
                ui.add(egui::TextEdit::singleline(&mut self.nouvelle_regle.explication).hint_text("Pourquoi ? (facultatif, affiché avec la faute)").desired_width(f32::INFINITY));
                ui.checkbox(&mut self.nouvelle_regle.casse, "Respecter les majuscules").on_hover_text("« Etat » est signalé, « état » non");
            });

        // Plusieurs règles d'un coup : collage ou fichier
        ui.horizontal(|ui| {
            if ui.button("📋 Coller une liste").on_hover_text("Copiez des lignes depuis Excel, Word ou un mail : une règle par ligne,\nà éviter → à écrire → explication (tabulation, →, = ou ; entre les colonnes)").clicked() {
                self.collage_regles = Some(String::new());
            }
            if ui.button("📂 Importer…").on_hover_text("Classeur Excel (.xlsx) ou fichier .csv / .txt :\ncolonne A à éviter, B à écrire, C explication, D « oui » pour respecter les majuscules").clicked() {
                if let Some(f) = rfd::FileDialog::new().add_filter("Excel, CSV", &["xlsx", "csv", "tsv", "txt"]).pick_file() {
                    match editions::lire_tableau(&f) {
                        Ok(l) => {
                            let (aj, maj) = editions::fusionne(&mut self.editions.liste[i], editions::regles_depuis(l));
                            self.message = format!("📘 Import : {aj} règle(s) ajoutée(s), {maj} mise(s) à jour.");
                            change = true;
                        }
                        Err(e) => self.message = format!("Import impossible : {e}"),
                    }
                }
            }
            if ui.button("💾 Exporter").on_hover_text("Enregistrer les règles en CSV : à modifier dans Excel, puis à réimporter").clicked() {
                let e = &self.editions.liste[i];
                if let Some(f) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).set_file_name(format!("{}.csv", e.nom)).save_file() {
                    self.message = match std::fs::write(&f, editions::vers_csv(e)) {
                        Ok(()) => format!("💾 {} règle(s) exportée(s).", e.regles.len()),
                        Err(x) => format!("Export impossible : {x}"),
                    };
                }
            }
        });
        let (mut valider, mut annuler) = (false, false);
        if let Some(txt) = &mut self.collage_regles {
            ui.add(egui::TextEdit::multiline(txt).hint_text("c'est-à-dire\tc.-à-d.\tAbréviation maison\nMr → M.").desired_rows(5).desired_width(f32::INFINITY));
            let n = editions::regles_depuis(editions::lire_texte(txt)).len();
            ui.horizontal(|ui| {
                valider = ui.add_enabled(n > 0, egui::Button::new(format!("＋ Ajouter {n} règle(s)"))).clicked();
                annuler = ui.button("Annuler").clicked();
            });
        }
        if valider {
            let t = self.collage_regles.take().unwrap_or_default();
            let (aj, maj) = editions::fusionne(&mut self.editions.liste[i], editions::regles_depuis(editions::lire_texte(&t)));
            self.message = format!("📘 {aj} règle(s) ajoutée(s), {maj} mise(s) à jour.");
            change = true;
        } else if annuler {
            self.collage_regles = None;
        }
        ui.add_space(4.0);

        // Règles de l'éditeur : une ligne chacune, détails à la demande
        if self.editions.liste[i].regles.len() > 6 {
            ui.add(egui::TextEdit::singleline(&mut self.filtre_regles).hint_text("🔍 Chercher une règle…").desired_width(f32::INFINITY));
        }
        let filtre = self.filtre_regles.to_lowercase();
        let ouverte = self.regle_ouverte;
        let (mut retirer, mut basculer) = (None, None);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (k, r) in self.editions.liste[i].regles.iter_mut().enumerate() {
                let cherche = |t: &str| t.to_lowercase().contains(&filtre);
                if !filtre.is_empty() && !(cherche(&r.motif) || cherche(&r.remplacement) || cherche(&r.explication)) {
                    continue;
                }
                ui.horizontal(|ui| {
                    change |= ui.checkbox(&mut r.active, "").on_hover_text("Règle active").changed();
                    let l = ((ui.available_width() - 70.0) / 2.0).max(70.0);
                    let m = ui.add(egui::TextEdit::singleline(&mut r.motif).desired_width(l));
                    change |= m.changed();
                    if !r.explication.is_empty() {
                        m.on_hover_text(&r.explication);
                    }
                    ui.label("→");
                    change |= ui.add(egui::TextEdit::singleline(&mut r.remplacement).hint_text("(à retirer)").desired_width(l)).changed();
                    if ui.add(egui::Button::new(if ouverte == Some(k) { "▴" } else { "⋯" }).frame(false)).on_hover_text("Explication et options").clicked() {
                        basculer = Some(k);
                    }
                    if ui.add(egui::Button::new("✕").frame(false)).on_hover_text("Supprimer la règle").clicked() {
                        retirer = Some(k);
                    }
                });
                if ouverte == Some(k) {
                    ui.indent(k, |ui| {
                        change |= ui.add(egui::TextEdit::singleline(&mut r.explication).hint_text("Explication (facultatif)").desired_width(f32::INFINITY)).changed();
                        ui.horizontal(|ui| {
                            change |= ui.checkbox(&mut r.mot_entier, "Mot entier").changed();
                            change |= ui.checkbox(&mut r.casse, "Respecter les majuscules").changed();
                        });
                    });
                }
            }
            if self.editions.liste[i].regles.is_empty() {
                ui.label(egui::RichText::new("Aucune règle : ajoutez-en une ci-dessus, collez une liste ou importez un classeur Excel.").weak());
            }
        });
        if let Some(k) = basculer {
            self.regle_ouverte = if ouverte == Some(k) { None } else { Some(k) };
        }
        if let Some(k) = retirer {
            self.editions.liste[i].regles.remove(k);
            self.regle_ouverte = None;
            change = true;
        }
        if change {
            self.editions_modifiees();
        }
    }

    /// Document vide ou plan type ; un texte non enregistré n'est remplacé qu'à la deuxième demande.
    fn nouveau(&mut self, texte: String, titre: &str) -> bool {
        if self.modifie && !self.nouveau_confirme {
            self.nouveau_confirme = true;
            self.message = "⚠ Modifications non enregistrées : Ctrl+S pour les garder, ou recommencez pour les abandonner.".into();
            return false;
        }
        self.nouveau_confirme = false;
        self.texte = texte;
        self.titre = titre.into();
        self.fichier = None;
        self.modifie = false;
        self.mots_ouverture = 0;
        self.selection_demandee = Some((0, 0));
        self.message = format!("Nouveau document : {titre}");
        true
    }

    /// Entre dans le mode écriture (volets fermés, gardés pour le retour) ou en sort.
    fn bascule_ecriture(&mut self) {
        if self.onglet == OngletActif::Ecriture {
            self.onglet = OngletActif::Correction;
            self.panneau_lateral = std::mem::replace(&mut self.panneau_avant_ecriture, PanneauLateral::Aucun);
        } else {
            self.onglet = OngletActif::Ecriture;
            self.panneau_avant_ecriture = std::mem::replace(&mut self.panneau_lateral, PanneauLateral::Aucun);
        }
    }

    fn editions_modifiees(&mut self) {
        if let Err(e) = self.editions.sauve() {
            self.message = format!("Règles éditeurs non enregistrées : {e}");
        }
        self.grammaire.edition = self.editions.edition_active().cloned();
        self.cache_analyse = None;
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
                let (t, sel) = ecriture::ajoute_partie(&self.texte, i);
                self.applique_plan(t, sel);
                self.message = "✔ Partie ajoutée : tapez son titre, puis ↓ pour écrire dessous.".into();
            }
            ActionPlan::NouveauChapitre => {
                let (t, sel) = ecriture::ajoute_chapitre(&self.texte, self.selection.0);
                self.applique_plan(t, sel);
                self.message = "✔ Chapitre ajouté : tapez son titre, puis ↓ pour écrire dessous.".into();
            }
        }
    }

    /// Mode écriture : plan cliquable (mots et fautes par section), gestion des sections, séance d'écriture.
    fn panneau_plan(&mut self, ui: &mut egui::Ui, alertes: &[AlerteG]) {
        let plan = ecriture::plan(&self.texte);
        let cour = ecriture::courante(&plan, self.selection.0);
        let mut action = None;

        // En-tête : titre et ajout ; les déplacements n'apparaissent que pour une section en cours
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Plan").size(17.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.toggle_value(&mut self.soulignements_ecriture, "〰").on_hover_text("Souligner les fautes pendant l'écriture");
                ui.toggle_value(&mut self.concentration, "◎").on_hover_text("Concentration : estomper tout sauf la section en cours");
            });
        });
        if let Some(c) = cour {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let mut outil = |ui: &mut egui::Ui, t: &str, aide: &str, a: Option<ActionPlan>| {
                    if ui.add_enabled(a.is_some(), egui::Button::new(t).frame(false)).on_hover_text(aide).clicked() {
                        action = a;
                    }
                };
                outil(ui, "↑", "Monter la section (Alt+↑)", Some(ActionPlan::Deplacer(c, true)));
                outil(ui, "↓", "Descendre la section (Alt+↓)", Some(ActionPlan::Deplacer(c, false)));
                outil(ui, "←", "Niveau supérieur (## → #)", (plan[c].niveau > 1).then_some(ActionPlan::Niveau(c, -1)));
                outil(ui, "→", "Niveau inférieur (# → ##)", Some(ActionPlan::Niveau(c, 1)));
                ui.label(egui::RichText::new(abrege(&plan[c].titre, 24)).small().weak());
            });
        }
        ui.add_space(4.0);
        if plan.is_empty() {
            ui.label(egui::RichText::new("Pas encore de titre.").weak());
            ui.label(egui::RichText::new("« # » en début de ligne : chapitre · « ## » : partie").small().weak());
            if ui.button("＋ Premier chapitre").clicked() {
                action = Some(ActionPlan::NouveauChapitre);
            }
        }
        let fautes: Vec<usize> = alertes.iter().filter(|a| a.type_alerte != TypeAlerte::Repetition).map(|a| a.debut).collect();
        let mots_vises = self.biblio.reglages.pages_visees * self.biblio.reglages.mots_par_page.max(50);
        let parts = ecriture::parts_ideales(&plan);
        let hauteur = (ui.available_height() - 290.0).max(120.0);
        egui::ScrollArea::vertical().max_height(hauteur).auto_shrink([false, true]).show(ui, |ui| {
            for (i, s) in plan.iter().enumerate() {
                let n = fautes.partition_point(|&d| d < s.fin) - fautes.partition_point(|&d| d < s.debut);
                ui.horizontal(|ui| {
                    let h = ui.spacing().interact_size.y;
                    let survol = ui.rect_contains_pointer(egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), h)));
                    ui.add_space(14.0 * (s.niveau.saturating_sub(1)) as f32);
                    let mut libelle = egui::RichText::new(abrege(&s.titre, 42usize.saturating_sub(3 * s.niveau as usize)));
                    if s.niveau == 1 {
                        libelle = libelle.strong();
                    }
                    let r = ui.selectable_label(cour == Some(i), libelle).on_hover_text(&s.titre);
                    if r.clicked() {
                        action = Some(ActionPlan::Aller(i));
                    }
                    // barre fine sous le chapitre : avancement vers sa taille idéale
                    let ideal = parts[i] * mots_vises as f32;
                    if s.niveau == 1 && ideal > 0.0 {
                        let f = (s.mots as f32 / ideal).min(1.0);
                        let b = egui::Rect::from_min_size(r.rect.left_bottom() + egui::vec2(0.0, 1.0), egui::vec2(r.rect.width() * f, 2.0));
                        let c = if s.mots as f32 > ideal * 1.15 { rgb(248, 113, 113) } else { rgb(52, 211, 153) };
                        ui.painter().rect_filled(b, 1.0, c);
                    }
                    r.context_menu(|ui| {
                        let mut choix = |ui: &mut egui::Ui, t: &str, a: ActionPlan| {
                            if ui.button(t).clicked() {
                                action = Some(a);
                                ui.close_menu();
                            }
                        };
                        choix(ui, "↑ Monter", ActionPlan::Deplacer(i, true));
                        choix(ui, "↓ Descendre", ActionPlan::Deplacer(i, false));
                        if s.niveau > 1 {
                            choix(ui, "← Niveau supérieur", ActionPlan::Niveau(i, -1));
                        }
                        choix(ui, "→ Niveau inférieur", ActionPlan::Niveau(i, 1));
                        choix(ui, "＋ Ajouter une sous-partie", ActionPlan::SousPartie(i));
                        choix(ui, "Sélectionner toute la section", ActionPlan::Selectionner(i));
                        choix(ui, "Retirer le titre (devient du texte)", ActionPlan::RetirerTitre(i));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if (survol || cour == Some(i)) && s.niveau < ecriture::MAX_NIVEAU
                            && ui.add(egui::Button::new(egui::RichText::new("＋").color(rgb(129, 140, 248))).frame(false))
                                .on_hover_text(format!("Ajouter une partie à la fin de « {} »", abrege(&s.titre, 30))).clicked()
                        {
                            action = Some(ActionPlan::SousPartie(i));
                        }
                        let ideal = (parts[i] * mots_vises as f32) as usize;
                        if s.niveau == 1 && ideal > 0 {
                            ui.label(egui::RichText::new(format!("{} / {} · {:.0} %", s.mots, ideal, parts[i] * 100.0)).small().weak())
                                .on_hover_text(format!("{} mots écrits sur {} visés (≈ {} pages, {:.0} % du livre)",
                                    s.mots, ideal, ideal / self.biblio.reglages.mots_par_page.max(50), parts[i] * 100.0));
                        } else {
                            ui.label(egui::RichText::new(format!("{}", s.mots)).small().weak())
                                .on_hover_text("mots de la section, sous-parties comprises");
                        }
                        if n > 0 {
                            ui.label(egui::RichText::new(format!("● {n}")).small().color(rgb(248, 113, 113)))
                                .on_hover_text("fautes signalées dans la section");
                        }
                    });
                });
            }
        });

        if !plan.is_empty() {
            ui.add_space(4.0);
            let b = egui::Button::new(egui::RichText::new("＋  Nouveau chapitre").color(rgb(129, 140, 248)))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0_f32, rgb(55, 65, 90)))
                .corner_radius(8);
            if ui.add_sized([ui.available_width(), 28.0], b)
                .on_hover_text("Après le chapitre en cours (avant la conclusion), numéroté à la suite — Ctrl+Maj+Entrée")
                .clicked()
            {
                action = Some(ActionPlan::NouveauChapitre);
            }
        }
        // Carte d'écriture, en bas : chapitre en cours, livre, séance
        let mots = nb_mots(&self.texte);
        let par_page = self.biblio.reglages.mots_par_page.max(50);
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            egui::Frame::NONE
                .fill(rgb(22, 26, 35))
                .stroke(Stroke::new(1.0_f32, rgb(40, 48, 63)))
                .corner_radius(12)
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 5.0;
                        let titre = |ui: &mut egui::Ui, t: &str| {
                            ui.label(egui::RichText::new(t.to_uppercase()).size(10.5).strong().color(rgb(120, 130, 155)));
                        };
                        let jauge = |ui: &mut egui::Ui, f: f32, c: Color32| {
                            let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), egui::Sense::hover());
                            ui.painter().rect_filled(r, 3.0, rgb(38, 45, 60));
                            if f > 0.0 {
                                ui.painter().rect_filled(egui::Rect::from_min_size(r.min, egui::vec2(r.width() * f.min(1.0), 6.0)), 3.0, c);
                            }
                        };
                        let ligne = |ui: &mut egui::Ui, gauche: String, droite: String| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(gauche).size(13.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(droite).size(12.0).weak());
                                });
                            });
                        };

                        // 1. Chapitre en cours
                        let chap = cour.map(|c| (0..=c).rev().find(|&k| plan[k].niveau == 1).unwrap_or(c));
                        let vises = self.biblio.reglages.pages_visees * par_page;
                        let parts = ecriture::parts_ideales(&plan);
                        if let Some(k) = chap {
                            titre(ui, "Chapitre en cours");
                            let ideal = (parts[k] * vises as f32) as usize;
                            ligne(ui, abrege(&plan[k].titre, 26), if ideal > 0 {
                                format!("{} / {} mots", plan[k].mots, ideal)
                            } else {
                                format!("{} mots", plan[k].mots)
                            });
                            if ideal > 0 {
                                let f = plan[k].mots as f32 / ideal as f32;
                                jauge(ui, f, if f > 1.15 { rgb(248, 113, 113) } else { rgb(52, 211, 153) });
                            }
                            ui.add_space(6.0);
                        }

                        // 2. Livre : pages visées, avancement, séances restantes
                        titre(ui, "Livre");
                        ui.horizontal(|ui| {
                            let mut pages = self.biblio.reglages.pages_visees;
                            if ui.add(egui::DragValue::new(&mut pages).range(0..=2000).speed(2.0).suffix(" pages visées"))
                                .on_hover_text("Taille visée du livre (glisser pour changer ; 0 : pas d'objectif)").changed()
                            {
                                self.biblio.reglages.pages_visees = pages;
                                let _ = self.biblio.sauve_reglages();
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("{} p. écrites", mots / par_page)).size(12.0).weak());
                            });
                        });
                        if vises > 0 {
                            let f = mots as f32 / vises as f32;
                            jauge(ui, f, rgb(129, 140, 248));
                            let seance = self.biblio.reglages.objectif_mots.max(1);
                            let reste = vises.saturating_sub(mots);
                            ligne(ui, format!("{:.0} %", 100.0 * f.min(1.0)), if reste == 0 {
                                "objectif atteint 🎉".into()
                            } else {
                                format!("encore ≈ {} séances", reste.div_ceil(seance))
                            });
                        }
                        ui.add_space(6.0);

                        // 3. Séance
                        titre(ui, "Séance");
                        let faits = mots.saturating_sub(self.mots_ouverture);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{faits}")).size(22.0).strong());
                            ui.label(egui::RichText::new("mots sur").weak());
                            if ui.add(egui::DragValue::new(&mut self.biblio.reglages.objectif_mots).range(50..=20000).speed(10.0))
                                .on_hover_text("Objectif de mots par séance (glisser pour changer)").changed()
                            {
                                let _ = self.biblio.sauve_reglages();
                            }
                        });
                        jauge(ui, faits as f32 / self.biblio.reglages.objectif_mots.max(1) as f32, rgb(250, 204, 21));
                        let lecture = mots / 230;
                        ui.label(egui::RichText::new(format!("{mots} mots au total · {} h {:02} de lecture", lecture / 60, lecture % 60)).size(11.5).weak());
                    });
                });
        });
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
            ui.label(egui::RichText::new("🎯 Fautes").size(17.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").on_hover_text("Fermer ce panneau latéral").clicked() {
                    self.panneau_lateral = PanneauLateral::Aucun;
                }
            });
        });
        let nb_corr = alertes.iter().filter(|a| a.type_alerte != TypeAlerte::Repetition).count();
        ui.horizontal(|ui| {
            if ui.selectable_label(self.onglet != OngletActif::Repetitions, format!("✓ Correction  {nb_corr}"))
                .on_hover_text("Grammaire, orthographe, règles éditeurs").clicked()
            {
                self.onglet = OngletActif::Correction;
            }
            if ui.selectable_label(self.onglet == OngletActif::Repetitions, format!("◈ Style  {}", repetitions.len()))
                .on_hover_text("Répétitions de mots").clicked()
            {
                self.onglet = OngletActif::Repetitions;
            }
            if self.onglet == OngletActif::Repetitions {
                ui.checkbox(&mut self.surligner_repetitions, "surligner").on_hover_text("Surligner les répétitions dans le texte");
            }
        });
        ui.add_space(6.0);

        let grammaire_alertes: Vec<&AlerteG> = alertes.iter().filter(|a| a.type_alerte == TypeAlerte::Grammaire).collect();
        let ortho_alertes: Vec<&AlerteG> = alertes.iter().filter(|a| a.type_alerte == TypeAlerte::Orthographe).collect();
        let doublon_alertes: Vec<&AlerteG> = alertes.iter().filter(|a| a.type_alerte == TypeAlerte::Doublon).collect();

        let total_alertes = grammaire_alertes.len() + ortho_alertes.len() + doublon_alertes.len();

        if total_alertes == 0 && repetitions.is_empty() {
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("✨ Aucune anomalie détectée").color(rgb(52, 211, 153)).size(16.0).strong());
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Votre texte est parfaitement rédigé.").weak());
            });
            return;
        }

        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            // SECTION 1 : GRAMMAIRE & ACCORDS
            let label_gram = format!("● Grammaire & Accords ({})", grammaire_alertes.len());
            egui::CollapsingHeader::new(egui::RichText::new(label_gram).strong().color(rgb(251, 146, 60)))
                .default_open(true)
                .show(ui, |ui| {
                    if grammaire_alertes.is_empty() {
                        ui.label(egui::RichText::new("Aucune faute de grammaire.").weak().small());
                    }
                    for (idx, a) in grammaire_alertes.iter().enumerate() {
                        ui.push_id(("gram", idx), |ui| {
                            egui::Frame::NONE
                                .fill(rgb(22, 26, 35))
                                .corner_radius(egui::CornerRadius::same(7))
                                .stroke(Stroke::new(1.0_f32, rgb(38, 44, 58)))
                                .inner_margin(egui::Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.link(egui::RichText::new(&a.mot).strong().size(15.0).color(rgb(251, 146, 60))).on_hover_text("Cliquer pour localiser dans le texte").clicked() {
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
                                                if ui.small_button(egui::RichText::new(s).strong().color(rgb(56, 189, 248))).clicked() {
                                                    self.remplace(a.debut, a.fin, s);
                                                }
                                            }
                                        });
                                    }

                                    let utile = explication_pedagogique_approfondie(&a.regle, &a.mot, &a.explication).utile();
                                    let est_ouvert = utile && self.alertes_plus_infos.contains(&a.debut);
                                    if utile { ui.horizontal(|ui| {
                                        let texte_btn = if est_ouvert { "▲ Masquer l'explication" } else { "ℹ Plus d'infos" };
                                        if ui.button(egui::RichText::new(texte_btn).small().color(rgb(147, 197, 253)))
                                            .on_hover_text("Afficher la règle complète, l'explication académique et l'astuce mnémotechnique")
                                            .clicked()
                                        {
                                            if est_ouvert {
                                                self.alertes_plus_infos.remove(&a.debut);
                                            } else {
                                                self.alertes_plus_infos.insert(a.debut);
                                            }
                                        }
                                    }); }

                                    if est_ouvert {
                                        let fiche = explication_pedagogique_approfondie(&a.regle, &a.mot, &a.explication);
                                        egui::Frame::NONE
                                            .fill(rgb(16, 20, 28))
                                            .stroke(Stroke::new(1.0_f32, rgb(45, 60, 85)))
                                             .corner_radius(egui::CornerRadius::same(6))
                                             .inner_margin(egui::Margin::symmetric(8, 6))
                                             .show(ui, |ui| {
                                                 ui.label(egui::RichText::new(format!("📘 {}", fiche.titre)).strong().color(rgb(147, 197, 253)).small());
                                                 ui.add_space(2.0);
                                                 ui.label(egui::RichText::new(fiche.regle_complete).small().color(rgb(220, 228, 240)));
                                                 ui.add_space(4.0);
                                                 egui::Frame::NONE
                                                     .fill(rgb(22, 32, 28))
                                                     .stroke(Stroke::new(1.0_f32, rgb(34, 197, 94)))
                                                     .corner_radius(egui::CornerRadius::same(5))
                                                     .inner_margin(egui::Margin::symmetric(8, 5))
                                                     .show(ui, |ui| {
                                                         ui.label(egui::RichText::new(format!("💡 Astuce mnémotechnique :
{}", fiche.astuce)).strong().color(rgb(134, 239, 172)).small());
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
            egui::CollapsingHeader::new(egui::RichText::new(label_ortho).strong().color(rgb(248, 113, 113)))
                .default_open(true)
                .show(ui, |ui| {
                    if ortho_alertes.is_empty() {
                        ui.label(egui::RichText::new("Aucune faute d'orthographe.").weak().small());
                    }
                    for (idx, a) in ortho_alertes.iter().enumerate() {
                        ui.push_id(("ortho", idx), |ui| {
                            egui::Frame::NONE
                                .fill(rgb(22, 26, 35))
                                .corner_radius(egui::CornerRadius::same(7))
                                .stroke(Stroke::new(1.0_f32, rgb(38, 44, 58)))
                                .inner_margin(egui::Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.link(egui::RichText::new(&a.mot).strong().size(15.0).color(rgb(248, 113, 113))).on_hover_text("Cliquer pour localiser").clicked() {
                                            self.selection_demandee = Some((a.debut, a.fin));
                                            ui.ctx().memory_mut(|m| m.request_focus(id_editeur));
                                        }
                                        ui.label(egui::RichText::new(&a.regle).small().weak());
                                    });
                                    if !a.suggestions.is_empty() {
                                        ui.horizontal_wrapped(|ui| {
                                            for s in a.suggestions.iter().take(3) {
                                                if ui.small_button(egui::RichText::new(s).strong().color(rgb(56, 189, 248))).clicked() {
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
                                        if explication_pedagogique_approfondie(&a.regle, &a.mot, &a.explication).utile() {
                                        let texte_btn = if est_ouvert { "▲ Moins d'infos" } else { "ℹ Plus d'infos" };
                                        if ui.button(egui::RichText::new(texte_btn).small().color(rgb(147, 197, 253))).clicked() {
                                            if est_ouvert {
                                                self.alertes_plus_infos.remove(&a.debut);
                                            } else {
                                                self.alertes_plus_infos.insert(a.debut);
                                            }
                                        }
                                        }
                                    });

                                    let fiche = explication_pedagogique_approfondie(&a.regle, &a.mot, &a.explication);
                                    if fiche.utile() && self.alertes_plus_infos.contains(&a.debut) {
                                        egui::Frame::NONE
                                            .fill(rgb(16, 20, 28))
                                            .stroke(Stroke::new(1.0_f32, rgb(45, 60, 85)))
                                            .corner_radius(egui::CornerRadius::same(6))
                                            .inner_margin(egui::Margin::symmetric(8, 6))
                                            .show(ui, |ui| {
                                                ui.label(egui::RichText::new(format!("📘 {}", fiche.titre)).strong().color(rgb(147, 197, 253)).small());
                                                ui.add_space(2.0);
                                                ui.label(egui::RichText::new(fiche.regle_complete).small().color(rgb(220, 228, 240)));
                                                ui.add_space(4.0);
                                                egui::Frame::NONE
                                                    .fill(rgb(22, 32, 28))
                                                    .stroke(Stroke::new(1.0_f32, rgb(34, 197, 94)))
                                                    .corner_radius(egui::CornerRadius::same(5))
                                                    .inner_margin(egui::Margin::symmetric(8, 5))
                                                    .show(ui, |ui| {
                                                         ui.label(egui::RichText::new(format!("💡 Conseil :
{}", fiche.astuce)).strong().color(rgb(134, 239, 172)).small());
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
            egui::CollapsingHeader::new(egui::RichText::new(label_style).strong().color(rgb(250, 204, 21)))
                .default_open(true)
                .show(ui, |ui| {
                    if repetitions.is_empty() {
                        ui.label(egui::RichText::new("Aucune répétition de mot plein.").weak().small());
                    }
                    for (idx, rep) in repetitions.iter().enumerate() {
                        let est_sel = self.repetition_selectionnee.as_deref() == Some(&rep.racine);
                        ui.push_id(("rep", idx), |ui| {
                            egui::Frame::NONE
                                .fill(rgb(22, 26, 35))
                                .corner_radius(egui::CornerRadius::same(7))
                                .stroke(Stroke::new(1.0_f32, rgb(38, 44, 58)))
                                .inner_margin(egui::Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.selectable_label(est_sel, egui::RichText::new(format!("« {} »", rep.mot)).strong().color(rgb(250, 204, 21))).clicked() {
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
                                                if ui.small_button(egui::RichText::new(alt).color(rgb(250, 204, 21))).clicked() {
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
                egui::CollapsingHeader::new(egui::RichText::new(label_doublons).strong().color(rgb(250, 204, 21)))
                    .default_open(true)
                    .show(ui, |ui| {
                        for (idx, a) in doublon_alertes.iter().enumerate() {
                            ui.push_id(("doublon", idx), |ui| {
                                egui::Frame::NONE
                                    .fill(rgb(22, 26, 35))
                                    .corner_radius(egui::CornerRadius::same(7))
                                    .stroke(Stroke::new(1.0_f32, rgb(38, 44, 58)))
                                    .inner_margin(egui::Margin::symmetric(10, 8))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui.link(egui::RichText::new(&a.mot).strong().color(rgb(250, 204, 21))).clicked() {
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

        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::N)) {
            self.nouveau(String::new(), "Sans titre");
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::Enter)) {
            self.action_plan(ActionPlan::NouveauChapitre);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::E)) {
            self.bascule_ecriture();
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
        let titre = format!("{marque}{} — Texteur", self.titre);
        if titre != self.titre_fenetre {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(titre.clone()));
            self.titre_fenetre = titre;
        }

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
            (Arc::as_ptr(&self.masque) as usize).hash(&mut h); // nouveau masque = dictionnaires modifiés
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
                let valide = |a: &AlerteG| a.debut <= a.fin && a.fin <= car.len() && car[a.debut..a.fin].iter().collect::<String>() == a.mot;
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
                        .fill(rgb(22, 27, 40))
                        .stroke(Stroke::new(1.0_f32, rgb(56, 189, 248)))
                        .inner_margin(egui::Margin::symmetric(12, 6))
                        .corner_radius(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("« {} »", a.mot)).strong().color(rgb(252, 165, 165)));
                                for (n, s) in a.suggestions.iter().take(3).enumerate() {
                                    ui.label(egui::RichText::new(format!("Alt+{} {}", n + 1, s)).color(rgb(134, 239, 172)));
                                }
                                ui.label(egui::RichText::new("· Alt+Entrée accepter · Alt+Suppr ignorer · F8 suivante").small().color(rgb(140, 148, 165)));
                            });
                        });
                });
        }

        // Barre d'outils supérieure stylisée
        egui::TopBottomPanel::top("barre")
            .frame(
                egui::Frame::NONE
                    .fill(rgb(16, 18, 25))
                    .stroke(Stroke::new(1.0_f32, rgb(32, 38, 52)))
                    .inner_margin(egui::Margin::symmetric(14, 8)),
            )
            .show(ctx, |ui| {
                self.barre(ui, alertes.iter().filter(|a| a.type_alerte != TypeAlerte::Repetition).count(), repetitions.len());
            });

        // Barre d'état inférieure moderne avec puces colorées
        egui::TopBottomPanel::bottom("statut")
            .frame(
                egui::Frame::NONE
                    .fill(rgb(14, 16, 22))
                    .stroke(Stroke::new(1.0_f32, rgb(28, 33, 46)))
                    .inner_margin(egui::Margin::symmetric(14, 7)),
            )
            .show(ctx, |ui| {
                let mots = texte.split_whitespace().count();
                let proteges = plages.len();

                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(format!("📄 {mots} mots")).strong());
                    ui.separator();
                    if self.onglet == OngletActif::Ecriture {
                        let faits = nb_mots(&self.texte).saturating_sub(self.mots_ouverture);
                        let but = self.biblio.reglages.objectif_mots.max(1);
                        ui.add(egui::ProgressBar::new((faits as f32 / but as f32).min(1.0)).desired_width(140.0)
                            .text(format!("séance : {faits} / {but} mots")));
                        ui.label(egui::RichText::new("Ctrl+E : quitter l'écriture").weak());
                        ui.separator();
                    }
                    let nb_cjk = texte.chars().filter(|&c| masque::est_cjk(c)).count();
                    if nb_cjk > 0 {
                        ui.colored_label(rgb(56, 189, 248), format!("🇨🇳 {nb_cjk} car. chinois reconnus"));
                        ui.separator();
                    } else if proteges > 0 {
                        ui.colored_label(rgb(56, 189, 248), format!("🛡️ {proteges} protégé(s)"));
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
                            .fill(rgb(16, 19, 26))
                            .stroke(Stroke::new(1.0_f32, rgb(32, 38, 52)))
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
                            .fill(rgb(16, 19, 26))
                            .stroke(Stroke::new(1.0_f32, rgb(32, 38, 52)))
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
            PanneauLateral::Editions => {
                egui::SidePanel::right("panneau_lateral_droit")
                    .default_width(400.0)
                    .min_width(320.0)
                    .max_width(560.0)
                    .frame(
                        egui::Frame::NONE
                            .fill(rgb(16, 19, 26))
                            .stroke(Stroke::new(1.0_f32, rgb(32, 38, 52)))
                            .inner_margin(egui::Margin::symmetric(14, 12)),
                    )
                    .show(ctx, |ui| self.panneau_editions(ui));
            }
            PanneauLateral::Aucun if self.onglet == OngletActif::Ecriture => {}
            PanneauLateral::Aucun => {
                let nb_alertes = alertes.len();
                let nb_rep = repetitions.len();
                egui::SidePanel::right("volet_replie_droit")
                    .resizable(false)
                    .exact_width(42.0)
                    .frame(
                        egui::Frame::NONE
                            .fill(rgb(18, 22, 30))
                            .stroke(Stroke::new(1.0_f32, rgb(45, 55, 75)))
                            .inner_margin(egui::Margin::symmetric(4, 12))
                    )
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            let btn_reouvrir = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("◀")
                                        .size(16.0)
                                        .color(rgb(52, 211, 153))
                                        .strong()
                                )
                                .fill(rgb(26, 36, 48))
                                .stroke(Stroke::new(1.0_f32, rgb(52, 211, 153)))
                            ).on_hover_text("Ouvrir le panneau latéral des alertes (F9 ou Ctrl+B)");

                            if btn_reouvrir.clicked() {
                                self.panneau_lateral = PanneauLateral::Detections;
                            }

                            ui.add_space(8.0);
                            if nb_alertes > 0 {
                                if ui.button(egui::RichText::new(format!("⚠️\n{nb_alertes}")).color(rgb(248, 113, 113)).size(12.0).strong())
                                    .on_hover_text("Ouvrir les alertes de grammaire et d'orthographe (F9)").clicked() {
                                    self.panneau_lateral = PanneauLateral::Detections;
                                }
                            } else if nb_rep > 0 {
                                if ui.button(egui::RichText::new(format!("◈\n{nb_rep}")).color(rgb(167, 139, 250)).size(12.0).strong())
                                    .on_hover_text("Ouvrir les répétitions de style (F9)").clicked() {
                                    self.panneau_lateral = PanneauLateral::Detections;
                                }
                            } else {
                                ui.label(egui::RichText::new("✨\nOK").color(rgb(52, 211, 153)).size(11.0));
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
                        .fill(rgb(16, 19, 26))
                        .stroke(Stroke::new(1.0_f32, rgb(32, 38, 52)))
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
                    .fill(rgb(13, 15, 20))
                    .inner_margin(egui::Margin::symmetric(16, 12)),
            )
            .show(ctx, |ui| {

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
                    .fill(rgb(18, 21, 28))
                    .corner_radius(egui::CornerRadius::same(8))
                    .stroke(Stroke::new(1.0_f32, rgb(32, 37, 50)))
                    .inner_margin(if ecriture { egui::Margin::symmetric(24, 28) } else { egui::Margin::symmetric(24, 18) })
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                            // écriture : colonne d'environ 72 caractères, centrée
                            ui.horizontal_top(|ui| {
                            let colonne = if ecriture { ui.available_width().min(760.0) } else { ui.available_width() };
                            ui.add_space((ui.available_width() - colonne) / 2.0);
                            let sortie = egui::TextEdit::multiline(&mut self.texte)
                                .id(id_editeur)
                                .desired_width(colonne)
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
                            // Bulle de correction, juste sous le mot où se trouve le curseur (ou sous la sélection)
                            let cur = self.selection.0;
                            let ancre = if self.selection.1 > self.selection.0 {
                                Some((self.selection.0, self.selection.1))
                            } else if self.onglet == OngletActif::Repetitions {
                                repetitions.iter().flat_map(|r| r.occurrences.iter().copied()).find(|&(d, f)| d <= cur && cur <= f)
                            } else if souligner {
                                alertes.iter().find(|a| a.debut <= cur && cur <= a.fin).map(|a| (a.debut, a.fin))
                            } else {
                                None
                            };
                            if let Some((d, f)) = ancre.filter(|&(_, f)| f <= self.texte.chars().count()) {
                                let g = |c| sortie.galley.pos_from_ccursor(CCursor::new(c)).translate(sortie.galley_pos.to_vec2());
                                let (rd, rf) = (g(d), g(f));
                                let bas = if (rf.top() - rd.top()).abs() < 1.0 { rd.bottom() } else { rf.bottom() };
                                if ui.clip_rect().contains(egui::pos2(rd.left(), bas - 2.0)) {
                                    egui::Area::new(egui::Id::new("bulle_correction"))
                                        .order(egui::Order::Foreground)
                                        .fixed_pos(egui::pos2(rd.left() - 10.0, bas + 6.0))
                                        .constrain(true)
                                        .show(ui.ctx(), |ui| {
                                            egui::Frame::popup(ui.style())
                                                .fill(rgb(22, 26, 35))
                                                .stroke(Stroke::new(1.0_f32, rgb(48, 56, 82)))
                                                .corner_radius(10)
                                                .inner_margin(egui::Margin::symmetric(12, 9))
                                                .show(ui, |ui| {
                                                    ui.set_max_width(560.0);
                                                    self.barre_suggestions(ui, &alertes, &repetitions);
                                                });
                                        });
                                }
                            }
                            // titre choisi dans le plan : amené en haut de l'éditeur
                            if let Some(p) = self.defiler_vers.take() {
                                let r = sortie.galley.pos_from_ccursor(CCursor::new(p)).translate(sortie.galley_pos.to_vec2());
                                ui.scroll_to_rect(r.expand2(egui::vec2(0.0, 40.0)), Some(egui::Align::TOP));
                            }
                            });
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
