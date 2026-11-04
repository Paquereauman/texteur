//! Module d'exportation et d'extraction de contexte pour l'IA.
//! Collecte automatiquement toutes les erreurs (grammaire, lexique, style/répétitions, doublons)
//! avec leurs phrases de contexte exactes, afin de permettre à une IA de traiter le texte
//! sans avoir à ingérer ni relire l'intégralité du document.

use serde::{Deserialize, Serialize};

use crate::TypeAlerte;
use crate::AlerteG;
use crate::style::Repetition;

/// Extrait la phrase ou le fragment de contexte englobant l'intervalle [debut..fin],
/// en mettant en valeur le terme fautif entre « guillemets ».
pub fn contexte_phrase(texte: &str, debut: usize, fin: usize) -> String {
    let car: Vec<char> = texte.chars().collect();
    let n = car.len();
    if n == 0 || debut >= n {
        return String::new();
    }
    let fin = fin.min(n).max(debut);

    // Remonter pour trouver le début de la phrase
    let mut d = debut;
    while d > 0 {
        let prev = car[d - 1];
        if (prev == '.' || prev == '?' || prev == '!' || prev == '\n') && d < debut {
            break;
        }
        // Limite maximale en amont : 130 caractères
        if debut - d > 130 {
            break;
        }
        d -= 1;
    }
    // Nettoyer les espaces et retours chariot au début
    while d < debut && (car[d].is_whitespace() || car[d] == '\n') {
        d += 1;
    }

    // Descendre pour trouver la fin de la phrase
    let mut f = fin;
    while f < n {
        let cur = car[f];
        if cur == '.' || cur == '?' || cur == '!' || cur == '\n' {
            f += 1;
            break;
        }
        // Limite maximale en aval : 130 caractères
        if f - fin > 130 {
            break;
        }
        f += 1;
    }

    let mut res = String::new();
    if d > 0 && !car[d - 1].is_whitespace() && car[d - 1] != '\n' && car[d - 1] != '.' {
        res.push('…');
    }
    res.push_str(&car[d..debut].iter().collect::<String>());
    res.push_str("« ");
    res.push_str(&car[debut..fin].iter().collect::<String>());
    res.push_str(" »");
    res.push_str(&car[fin..f].iter().collect::<String>());
    if f < n && car[f - 1] != '.' && car[f - 1] != '?' && car[f - 1] != '!' && car[f - 1] != '\n' {
        res.push('…');
    }
    res.trim().replace('\n', " ")
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AlerteIA {
    pub categorie: String,
    pub terme: String,
    pub suggestions: Vec<String>,
    pub regle: String,
    pub explication: String,
    pub phrase_contexte: String,
    pub debut: usize,
    pub fin: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepetitionIA {
    pub mot: String,
    pub nombre_occurrences: usize,
    pub alternatives: Vec<String>,
    pub phrases_contextes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RapportIA {
    pub document: String,
    pub total_mots: usize,
    pub total_alertes: usize,
    pub nb_grammaire: usize,
    pub nb_orthographe: usize,
    pub nb_doublons: usize,
    pub nb_repetitions: usize,
    pub alertes: Vec<AlerteIA>,
    pub repetitions: Vec<RepetitionIA>,
}

impl RapportIA {
    pub fn generer(
        texte: &str,
        alertes: &[AlerteG],
        repetitions: &[Repetition],
        nom_document: &str,
    ) -> Self {
        let total_mots = texte.split_whitespace().count();
        let mut alertes_ia = Vec::new();

        let mut nb_gram = 0;
        let mut nb_ortho = 0;
        let mut nb_doublon = 0;

        for a in alertes {
            if a.type_alerte == TypeAlerte::Repetition {
                continue;
            }
            let cat = match a.type_alerte {
                TypeAlerte::Grammaire => {
                    nb_gram += 1;
                    "Grammaire"
                }
                TypeAlerte::Orthographe => {
                    nb_ortho += 1;
                    "Orthographe"
                }
                TypeAlerte::Doublon => {
                    nb_doublon += 1;
                    "Doublon"
                }
                _ => "Autre",
            };

            let ctx = contexte_phrase(texte, a.debut, a.fin);
            alertes_ia.push(AlerteIA {
                categorie: cat.to_string(),
                terme: a.mot.clone(),
                suggestions: a.suggestions.clone(),
                regle: a.regle.clone(),
                explication: a.explication.clone(),
                phrase_contexte: ctx,
                debut: a.debut,
                fin: a.fin,
            });
        }

        let mut repetitions_ia = Vec::new();
        for rep in repetitions {
            let mut ctxs = Vec::new();
            for &(d, f) in &rep.occurrences {
                ctxs.push(contexte_phrase(texte, d, f));
            }
            repetitions_ia.push(RepetitionIA {
                mot: rep.mot.clone(),
                nombre_occurrences: rep.nombre,
                alternatives: rep.alternatives.clone(),
                phrases_contextes: ctxs,
            });
        }

        let nb_rep = repetitions_ia.len();
        let total = alertes_ia.len() + nb_rep;

        RapportIA {
            document: nom_document.to_string(),
            total_mots,
            total_alertes: total,
            nb_grammaire: nb_gram,
            nb_orthographe: nb_ortho,
            nb_doublons: nb_doublon,
            nb_repetitions: nb_rep,
            alertes: alertes_ia,
            repetitions: repetitions_ia,
        }
    }

    /// Génère un rapport Markdown concis, optimisé pour les prompts d'IA (ChatGPT, Gemini, Claude...).
    pub fn vers_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# Rapport d'analyse Texteur (optimisé pour IA)\n");
        md.push_str(&format!(
            "- **Document** : `{}`\n- **Volume** : {} mots\n- **Alertes** : {} ({} grammaire, {} orthographe, {} doublons, {} termes répétés)\n\n",
            self.document, self.total_mots, self.total_alertes, self.nb_grammaire, self.nb_orthographe, self.nb_doublons, self.nb_repetitions
        ));

        if self.alertes.is_empty() && self.repetitions.is_empty() {
            md.push_str("✨ Aucune anomalie détectée dans ce texte.\n");
            return md;
        }

        if !self.alertes.is_empty() {
            md.push_str("## 1. Fautes de grammaire et d'orthographe (avec contextes)\n\n");
            for (i, a) in self.alertes.iter().enumerate() {
                let sugg = if a.suggestions.is_empty() {
                    "aucune".to_string()
                } else {
                    a.suggestions.join(" / ")
                };
                md.push_str(&format!(
                    "{}. **[{}]** `{}` ➔ Suggestions : `{}`\n   - *Règle* : {}\n   - *Explication* : {}\n   - *Contexte* : « {} »\n\n",
                    i + 1, a.categorie, a.terme, sugg, a.regle, a.explication, a.phrase_contexte
                ));
            }
        }

        if !self.repetitions.is_empty() {
            md.push_str("## 2. Répétitions et propositions d'enrichissement de style\n\n");
            for (i, r) in self.repetitions.iter().enumerate() {
                let alts = if r.alternatives.is_empty() {
                    "aucune".to_string()
                } else {
                    r.alternatives.join(", ")
                };
                md.push_str(&format!(
                    "{}. **Terme répété** : « `{}` » ({} fois)\n   - *Alternatives élégantes* : {}\n   - *Phrases cibles* :\n",
                    i + 1, r.mot, r.nombre_occurrences, alts
                ));
                for (j, ctx) in r.phrases_contextes.iter().enumerate() {
                    md.push_str(&format!("     {}. « {} »\n", j + 1, ctx));
                }
                md.push('\n');
            }
        }

        md.push_str("---\n*Instruction IA* : Corrigez ou reformulez uniquement les phrases ciblées en préservant le sens initial et les termes techniques protégés.\n");
        md
    }

    /// Génère un JSON formaté et standardisé pour intégration programmatique.
    pub fn vers_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn serveur_repond(port: u16) -> bool {
    let Ok(mut stream) = TcpStream::connect_timeout(&std::net::SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_millis(350)) else {
        return false;
    };
    stream.set_read_timeout(Some(Duration::from_millis(400))).ok();
    stream.set_write_timeout(Some(Duration::from_millis(400))).ok();
    let req = format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port);
    let mut buf = [0u8; 128];
    stream.write_all(req.as_bytes()).is_ok() && stream.read(&mut buf).is_ok_and(|n| n > 0)
}

/// Lance un script Python en arrière-plan, sans fenêtre (version de Python préférée, puis celle du PATH).
fn lancer_script(version: &str, script: &std::path::Path, port: u16) -> Result<(), String> {
    if !script.exists() {
        return Err(format!("Script introuvable : {}", script.display()));
    }
    let python_local = std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
        .join(format!(r"Programs\Python\{version}\python.exe"));
    let candidats: Vec<std::ffi::OsString> = [python_local.into_os_string(), "python".into(), "py".into()].into();
    for py in candidats {
        let mut cmd = std::process::Command::new(&py);
        cmd.arg("-X").arg("utf8").arg(script).arg(port.to_string())
            .current_dir(script.parent().unwrap_or(std::path::Path::new(".")))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        if cmd.spawn().is_ok() {
            return Ok(());
        }
    }
    Err(format!("Python introuvable : impossible de démarrer {}.", script.display()))
}

// ---------------------------------------------------------------- Grammalecte (règles, processeur seul)
const PORT_GRAMMALECTE: u16 = 11435;

/// Erreur Grammalecte dans un paragraphe : début, fin (caractères), type, règle, message, suggestions.
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)] // l'identifiant de règle sert au débogage
pub struct ErreurGL(pub usize, pub usize, pub String, pub String, pub String, pub Vec<String>);

/// Démarre le serveur Grammalecte s'il ne répond pas (prêt en ~1 s, pas de carte graphique).
pub fn assurer_grammalecte() -> Result<u16, String> {
    use std::sync::Mutex;
    static DEMARRAGE: Mutex<()> = Mutex::new(());
    if serveur_repond(PORT_GRAMMALECTE) {
        return Ok(PORT_GRAMMALECTE);
    }
    let _verrou = DEMARRAGE.lock().unwrap_or_else(|e| e.into_inner());
    if serveur_repond(PORT_GRAMMALECTE) {
        return Ok(PORT_GRAMMALECTE);
    }
    lancer_script("Python312", &script_serveur().with_file_name("grammalecte_serveur.py"), PORT_GRAMMALECTE)?;
    let debut = std::time::Instant::now();
    while debut.elapsed() < Duration::from_secs(120) {
        std::thread::sleep(Duration::from_millis(300));
        if serveur_repond(PORT_GRAMMALECTE) {
            return Ok(PORT_GRAMMALECTE);
        }
    }
    Err("Grammalecte n'a pas démarré (grammalecte-fr est-il installé pour Python 3.12 ?)".into())
}

/// Corps de la réponse à une requête GET locale (délai court : sert aux vérifications de fond).
fn get_json(port: u16, chemin: &str) -> Option<serde_json::Value> {
    let mut stream = TcpStream::connect_timeout(&std::net::SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_millis(350)).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    stream.write_all(format!("GET {chemin} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n").as_bytes()).ok()?;
    let mut brut = Vec::new();
    stream.read_to_end(&mut brut).ok()?;
    let s = String::from_utf8_lossy(&brut);
    serde_json::from_str(&s[s.find("\r\n\r\n")? + 4..]).ok()
}

/// Vrai si l'entraînement a produit un modèle du détecteur plus récent que celui que le serveur a chargé.
pub fn nouveau_modele() -> bool {
    let charge = get_json(PORT_GRAMMALECTE, "/health").and_then(|v| v.get("modele")?.as_f64());
    let dossier = script_serveur().with_file_name("detecteur").join("modele_retenu");
    let disque = std::fs::read_dir(dossier).ok().and_then(|d| {
        d.filter_map(|e| e.ok()?.metadata().ok()?.modified().ok()).max()?.duration_since(std::time::UNIX_EPOCH).ok()
    });
    matches!((charge, disque), (Some(c), Some(d)) if d.as_secs_f64() > c + 1.0)
}

/// Arrête le serveur Grammalecte et attend qu'il ait fermé : `assurer_grammalecte` le relancera avec le dernier modèle.
pub fn arreter_grammalecte() {
    let _ = post_json(PORT_GRAMMALECTE, "/arret", &serde_json::json!({}), 5);
    let debut = std::time::Instant::now();
    while serveur_repond(PORT_GRAMMALECTE) && debut.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Vrai tant que le serveur charge encore spaCy et le détecteur : ses réponses sont alors partielles.
static INCOMPLET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn analyse_incomplete() -> bool {
    INCOMPLET.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn verifier_paragraphes(paragraphes: &[&str], port: u16) -> Result<Vec<Vec<ErreurGL>>, String> {
    let (code, corps) = post_json(port, "/api/grammalecte", &serde_json::json!({ "paragraphes": paragraphes }), 300)?;
    if code != 200 {
        return Err(format!("Grammalecte a répondu {code} : {corps}"));
    }
    #[derive(Deserialize)]
    struct Rep {
        erreurs: Vec<Vec<ErreurGL>>,
        #[serde(default)]
        incomplet: bool,
    }
    serde_json::from_str::<Rep>(&corps)
        .map(|r| {
            INCOMPLET.store(r.incomplet, std::sync::atomic::Ordering::Relaxed);
            r.erreurs
        })
        .map_err(|e| format!("Réponse Grammalecte invalide : {e}"))
}

/// Dossier des scripts Python (modifiable via la variable d'environnement TEXTEUR_SERVEUR_IA).
fn script_serveur() -> std::path::PathBuf {
    std::env::var("TEXTEUR_SERVEUR_IA").map(Into::into).unwrap_or_else(|_| {
        std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default()).join(r"Desktop\Projets\AI\qwen_serveur.py")
    })
}

/// POST JSON générique vers le serveur IA local ; renvoie (code HTTP, corps).
fn post_json(port: u16, chemin: &str, payload: &serde_json::Value, delai_s: u64) -> Result<(u16, String), String> {
    let mut stream = TcpStream::connect_timeout(&std::net::SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_secs(3))
        .map_err(|e| format!("Serveur IA non joignable sur 127.0.0.1:{} ({})", port, e))?;
    stream.set_read_timeout(Some(Duration::from_secs(delai_s))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();
    let body = payload.to_string();
    let req = format!(
        "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        chemin, port, body.len(), body
    );
    stream.write_all(req.as_bytes()).map_err(|e| format!("Erreur d'envoi : {}", e))?;
    let mut brut = Vec::new();
    stream.read_to_end(&mut brut).map_err(|e| format!("Erreur de lecture : {}", e))?;
    let s = String::from_utf8_lossy(&brut);
    let pos = s.find("\r\n\r\n").ok_or("Réponse HTTP malformée")?;
    let code = s.split_whitespace().nth(1).and_then(|c| c.parse().ok()).unwrap_or(0);
    Ok((code, s[pos + 4..].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contexte_phrase() {
        let texte = "Bonjour à tous. Les fournisseur de Shenzhen ont confirmé la commande. Tout va bien.";
        let byte_pos = texte.find("fournisseur").unwrap();
        let deb = texte[..byte_pos].chars().count();
        let fin = deb + "fournisseur".chars().count();
        let ctx = contexte_phrase(texte, deb, fin);
        assert!(ctx.contains("Les « fournisseur » de Shenzhen"));
        assert!(!ctx.contains("Bonjour à tous"));
        assert!(!ctx.contains("Tout va bien"));
    }
}
