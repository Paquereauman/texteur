path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\main.rs"
with open(path, "r", encoding="utf-8") as f:
    text = f.read()

# 1. Update colore function so Chinese / protected text is uncolored and not underlined
old_block = """    let protege = TextFormat {
        color: if sombre { Color32::from_rgb(120, 215, 240) } else { Color32::from_rgb(0, 110, 140) },
        background: if sombre { Color32::from_rgba_unmultiplied(20, 56, 72, 210) } else { Color32::from_rgb(225, 245, 250) },
        ..normal.clone()
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

    let mut morceaux: Vec<(usize, usize, TextFormat)> = Vec::new();
    for &(a, b) in plages_protegees {
        morceaux.push((a, b, protege.clone()));
    }
    for &(a, b) in repetition_active_plages {
        if !plages_protegees.iter().any(|&p| chevauche(p, (a, b))) {
            morceaux.push((a, b, repet_active.clone()));
        }
    }
    for &(a, b) in plages_repetitions {
        if !plages_protegees.iter().any(|&p| chevauche(p, (a, b)))
            && !repetition_active_plages.iter().any(|&p| chevauche(p, (a, b)))
        {
            morceaux.push((a, b, repet_douce.clone()));
        }
    }"""

new_block = """    // Surlignage doux et reposant pour les répétitions (fond pastel lavande clair)
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

    let mut morceaux: Vec<(usize, usize, TextFormat)> = Vec::new();
    // Les caractères chinois et termes protégés ne sont ni soulignés ni colorés (rendu textuel naturel et épuré)
    for &(a, b) in repetition_active_plages {
        if !plages_protegees.iter().any(|&p| chevauche(p, (a, b))) {
            morceaux.push((a, b, repet_active.clone()));
        }
    }
    for &(a, b) in plages_repetitions {
        if !plages_protegees.iter().any(|&p| chevauche(p, (a, b)))
            && !repetition_active_plages.iter().any(|&p| chevauche(p, (a, b)))
        {
            morceaux.push((a, b, repet_douce.clone()));
        }
    }"""

# Normalize \r\n to \n
text_norm = text.replace('\r\n', '\n')
if old_block in text_norm:
    text_norm = text_norm.replace(old_block, new_block, 1)
    print("Replaced colore block successfully")
else:
    print("WARNING: colore block not found")

# 2. Update status bar
old_status = """                    ui.colored_label(Color32::from_rgb(56, 189, 248), format!("🛡️ {proteges} protégé(s)"));
                    ui.separator();"""

new_status = """                    let nb_cjk = texte.chars().filter(|&c| masque::est_cjk(c)).count();
                    if nb_cjk > 0 {
                        ui.colored_label(Color32::from_rgb(56, 189, 248), format!("🇨🇳 {nb_cjk} car. chinois reconnus"));
                        ui.separator();
                    } else if proteges > 0 {
                        ui.colored_label(Color32::from_rgb(56, 189, 248), format!("🛡️ {proteges} protégé(s)"));
                        ui.separator();
                    }"""

if old_status in text_norm:
    text_norm = text_norm.replace(old_status, new_status, 1)
    print("Replaced status bar successfully")
else:
    print("WARNING: status bar block not found")

with open(path, "w", encoding="utf-8") as f:
    f.write(text_norm)
print("Updated main.rs successfully")
