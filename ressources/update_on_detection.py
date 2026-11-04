with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "r", encoding="utf-8") as f:
    text = f.read()

target_on = r'''            // « on » vs « ont » (homophone pronom indéfini vs verbe avoir pluriel)
            if lower == "on" {
                let mut sujet_pluriel = false;
                if i > 0 {
                    let mut p = i - 1;
                    while p > 0 && (tokens[p].mot.chars().all(est_cjk) || ["du", "de", "d'", "dans", "en", "pour", "à", "sur"].contains(&tokens[p].mot.to_lowercase().as_str())) {
                        p -= 1;
                    }
                    let p_mot = tokens[p].mot.to_lowercase();
                    if p_mot == "ils" || p_mot == "elles" || p_mot == "fournisseur" || p_mot == "fournisseurs" || p_mot.ends_with('s') || p_mot.ends_with('x') {
                        if !["nous", "vous", "mais", "dans", "sans", "toujours", "jamais", "parfois", "alors", "après", "apres"].contains(&p_mot.as_str()) {
                            sujet_pluriel = true;
                        }
                    }
                    if p > 0 {
                        let pp_mot = tokens[p - 1].mot.to_lowercase();
                        if self.determinants_pluriel.contains(pp_mot.as_str()) {
                            sujet_pluriel = true;
                        }
                    }
                }'''

replacement_on = r'''            // « on » vs « ont » (homophone pronom indéfini vs verbe avoir pluriel)
            if lower == "on" {
                let mut sujet_pluriel = false;
                // Scanner en arrière dans la proposition pour repérer un sujet pluriel (ex: « Les fournisseur ... on »)
                let mut p = i;
                while p > 0 {
                    p -= 1;
                    let w = tokens[p].mot.to_lowercase();
                    if w == "." || w == ";" || w == "!" || w == "?" || w == "," {
                        break;
                    }
                    if w == "ils" || w == "elles" || self.determinants_pluriel.contains(w.as_str()) {
                        sujet_pluriel = true;
                        break;
                    }
                }'''

assert target_on in text, "target_on not found"
text = text.replace(target_on, replacement_on)

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "w", encoding="utf-8", newline="\r\n") as f:
    f.write(text)

print("Updated on_ont detection!")
