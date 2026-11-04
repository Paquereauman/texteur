//! Correcteur orthographique local : API de vérification orthographique de
//! Windows (hors ligne, dictionnaires des langues installées dans Windows).
//! Les paragraphes sont vérifiés dans un fil dédié et mis en cache par texte de paragraphe.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use windows::Win32::Foundation::S_OK;
use windows::Win32::Globalization::{
    CORRECTIVE_ACTION_DELETE, CORRECTIVE_ACTION_GET_SUGGESTIONS, CORRECTIVE_ACTION_REPLACE, ISpellChecker,
    ISpellCheckerFactory, ISpellingError, SpellCheckerFactory,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree, IEnumString,
};
use windows::core::{HSTRING, PWSTR};

/// Faute dans un paragraphe (indices de caractères dans le paragraphe).
pub struct Faute {
    pub debut: usize,
    pub fin: usize,
    pub suggestions: Vec<String>,
    /// Mot répété (« le le »).
    pub doublon: bool,
}

#[derive(Default)]
pub struct EtatOrtho {
    pub langues: Vec<String>,
    pub langue: String,
    pub cache: HashMap<String, Arc<Vec<Faute>>>,
    en_cours: HashSet<String>,
    pub erreur: Option<String>,
}

pub struct Orthographe {
    tx: Sender<(String, Vec<String>)>,
    pub etat: Arc<Mutex<EtatOrtho>>,
}

impl Orthographe {
    pub fn new(ctx: egui::Context, langue_voulue: &str) -> Self {
        let etat = Arc::new(Mutex::new(EtatOrtho::default()));
        let (tx, rx) = channel();
        let (e, voulue) = (etat.clone(), langue_voulue.to_string());
        std::thread::spawn(move || {
            if let Err(err) = unsafe { travail(&e, &ctx, rx, &voulue) } {
                e.lock().unwrap().erreur = Some(format!("correcteur Windows indisponible ({})", err.message()));
                ctx.request_repaint();
            }
        });
        Orthographe { tx, etat }
    }

    pub fn change_langue(&self, langue: &str) {
        let mut e = self.etat.lock().unwrap();
        if e.langue != langue {
            e.langue = langue.to_string();
            e.cache.clear();
            e.en_cours.clear();
        }
    }

    /// Envoie au correcteur les paragraphes pas encore vérifiés.
    pub fn verifie(&self, texte: &str) {
        let mut e = self.etat.lock().unwrap();
        if e.langue.is_empty() {
            return;
        }
        if e.cache.len() > 2000 {
            let actuels: HashSet<&str> = texte.split('\n').collect();
            e.cache.retain(|k, _| actuels.contains(k.as_str()));
        }
        let neufs: HashSet<String> = texte
            .split('\n')
            .filter(|p| !p.trim().is_empty() && !e.cache.contains_key(*p) && !e.en_cours.contains(*p))
            .map(String::from)
            .collect();
        if neufs.is_empty() {
            return;
        }
        e.en_cours.extend(neufs.iter().cloned());
        let _ = self.tx.send((e.langue.clone(), neufs.into_iter().collect()));
    }
}

unsafe fn lis_enum(en: &IEnumString) -> Vec<String> {
    let mut v = Vec::new();
    loop {
        let mut s = [PWSTR::null()];
        let mut n = 0u32;
        if unsafe { en.Next(&mut s, Some(&mut n)) } != S_OK || n == 0 {
            break;
        }
        v.push(unsafe { s[0].to_string() }.unwrap_or_default());
        unsafe { CoTaskMemFree(Some(s[0].0 as _)) };
    }
    v
}

unsafe fn travail(
    etat: &Mutex<EtatOrtho>,
    ctx: &egui::Context,
    rx: Receiver<(String, Vec<String>)>,
    voulue: &str,
) -> windows::core::Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let fabrique: ISpellCheckerFactory = CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_INPROC_SERVER)?;
        let langues = lis_enum(&fabrique.SupportedLanguages()?);
        {
            let mut e = etat.lock().unwrap();
            e.langue = if langues.iter().any(|l| l == voulue) {
                voulue.to_string()
            } else {
                langues.iter().find(|l| l.starts_with("fr")).or(langues.first()).cloned().unwrap_or_default()
            };
            e.langues = langues;
        }
        ctx.request_repaint();

        let mut correcteurs: HashMap<String, ISpellChecker> = HashMap::new();
        for (langue, paras) in rx {
            let correcteur = match correcteurs.get(&langue) {
                Some(c) => Some(c.clone()),
                None => match fabrique.CreateSpellChecker(&HSTRING::from(&langue)) {
                    Ok(c) => {
                        correcteurs.insert(langue.clone(), c.clone());
                        Some(c)
                    }
                    Err(err) => {
                        etat.lock().unwrap().erreur = Some(format!("langue {langue} : {}", err.message()));
                        None
                    }
                },
            };
            for p in paras {
                let fautes = correcteur.as_ref().and_then(|c| verifie_paragraphe(c, &p).ok()).unwrap_or_default();
                let mut e = etat.lock().unwrap();
                e.en_cours.remove(&p);
                if e.langue == langue {
                    e.cache.insert(p, Arc::new(fautes));
                }
            }
            ctx.request_repaint();
        }
    }
    Ok(())
}

unsafe fn verifie_paragraphe(c: &ISpellChecker, p: &str) -> windows::core::Result<Vec<Faute>> {
    let u16s: Vec<u16> = p.encode_utf16().collect();
    // index UTF-16 (Windows) -> index de caractère (Texteur)
    let mut vers_car = Vec::with_capacity(u16s.len() + 1);
    for (i, ch) in p.chars().enumerate() {
        vers_car.extend(std::iter::repeat_n(i, ch.len_utf16()));
    }
    vers_car.push(p.chars().count());
    let borne = |i: usize| i.min(u16s.len());

    let mut res = Vec::new();
    unsafe {
        let erreurs = c.Check(&HSTRING::from(p))?;
        loop {
            let mut e: Option<ISpellingError> = None;
            if erreurs.Next(&mut e) != S_OK {
                break;
            }
            let Some(e) = e else { break };
            let (d, l) = (e.StartIndex()? as usize, e.Length()? as usize);
            let (d, f) = (borne(d), borne(d + l));
            let action = e.CorrectiveAction()?;
            let suggestions = if action == CORRECTIVE_ACTION_REPLACE {
                let r = e.Replacement()?;
                let s = r.to_string().unwrap_or_default();
                CoTaskMemFree(Some(r.0 as _));
                vec![s]
            } else if action == CORRECTIVE_ACTION_GET_SUGGESTIONS {
                let mot = String::from_utf16_lossy(&u16s[d..f]);
                lis_enum(&c.Suggest(&HSTRING::from(mot))?).into_iter().take(6).collect()
            } else {
                Vec::new()
            };
            res.push(Faute { debut: vers_car[d], fin: vers_car[f], suggestions, doublon: action == CORRECTIVE_ACTION_DELETE });
        }
    }
    Ok(res)
}

pub fn nom_langue(code: &str) -> String {
    let nom = match code {
        "fr-FR" => "Français (France)",
        "fr-CA" => "Français (Canada)",
        "fr-BE" => "Français (Belgique)",
        "fr-CH" => "Français (Suisse)",
        "en-US" => "Anglais (États-Unis)",
        "en-GB" => "Anglais (Royaume-Uni)",
        "en-AU" => "Anglais (Australie)",
        "en-CA" => "Anglais (Canada)",
        "en-IN" => "Anglais (Inde)",
        "es-ES" => "Espagnol (Espagne)",
        "es-MX" => "Espagnol (Mexique)",
        "de-DE" => "Allemand",
        "it-IT" => "Italien",
        "pt-BR" => "Portugais (Brésil)",
        "pt-PT" => "Portugais (Portugal)",
        "nl-NL" => "Néerlandais",
        "ru-RU" => "Russe",
        _ => return code.to_string(),
    };
    nom.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// `cargo test -- --ignored windows_reel --nocapture`
    #[test]
    #[ignore]
    fn windows_reel() {
        let o = Orthographe::new(egui::Context::default(), "fr-FR");
        std::thread::sleep(Duration::from_millis(1500));
        let p = "Les fournisseur ont confirmer la la livraison du projet 長城, je vous envoit.";
        o.verifie(p);
        std::thread::sleep(Duration::from_millis(2000));
        let e = o.etat.lock().unwrap();
        println!("langues={:?} langue={} erreur={:?}", e.langues, e.langue, e.erreur);
        for f in e.cache.get(p).expect("pas de résultat").iter() {
            let mot: String = p.chars().skip(f.debut).take(f.fin - f.debut).collect();
            println!("  « {mot} » doublon={} -> {:?}", f.doublon, f.suggestions);
        }
    }
}
