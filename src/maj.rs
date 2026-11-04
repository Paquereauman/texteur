//! Mises à jour sur ce PC : un Texteur recompilé dans le dossier des sources remplace celui qui tourne ; un modèle
//! plus récent du détecteur (entraînement) est chargé en relançant le serveur (voir `ia::arreter_grammalecte`).

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Texteur compilé dans le dossier des sources (modifiable par la variable d'environnement TEXTEUR_SOURCE).
fn compile() -> PathBuf {
    std::env::var("TEXTEUR_SOURCE").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default()).join(r"Desktop\Antidote\texteur\target\release\texteur.exe")
    })
}

fn date(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).ok()?.modified().ok()
}

/// Le Texteur compilé et son âge en minutes, s'il est plus récent que celui qui tourne (et n'est pas lui).
pub fn texteur_plus_recent() -> Option<(PathBuf, u64)> {
    let actuel = std::env::current_exe().ok()?;
    let src = compile();
    if src.canonicalize().ok()? == actuel.canonicalize().ok()? {
        return None;
    }
    let s = date(&src)?;
    if s.duration_since(date(&actuel)?).ok()?.as_secs() < 2 {
        return None;
    }
    Some((src, SystemTime::now().duration_since(s).map_or(0, |d| d.as_secs() / 60)))
}

/// Remplace le programme en cours par `src` (Windows permet de renommer un programme qui tourne : l'ancien est gardé
/// à côté, « .ancien.exe ») ; renvoie le chemin à relancer.
pub fn installe(src: &Path) -> Result<PathBuf, String> {
    let actuel = std::env::current_exe().map_err(|e| e.to_string())?;
    let nom = actuel.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let ancien = actuel.with_file_name(format!("{nom}.ancien.exe"));
    let _ = std::fs::remove_file(&ancien);
    if std::fs::rename(&actuel, &ancien).is_err() {
        // l'ancien est peut-être encore ouvert : autre nom
        let t = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs());
        std::fs::rename(&actuel, actuel.with_file_name(format!("{nom}.ancien-{t}.exe")))
            .map_err(|e| format!("Remplacement impossible : {e}"))?;
    }
    if let Err(e) = std::fs::copy(src, &actuel) {
        let _ = std::fs::rename(&ancien, &actuel);
        return Err(format!("Copie du nouveau Texteur impossible : {e}"));
    }
    Ok(actuel)
}
