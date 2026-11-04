import os
import sys

print("Fixing CAS 6 for leurs + verb...")

# Read build_full_engine.py
with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\ressources\build_full_engine.py", "r", encoding="utf-8") as f:
    text = f.read()

target = '''            // CAS 6 : Déterminant pluriel suivi d'un nom singulier (« les document » -> « documents »)
            if self.determinants_pluriel.contains(det_lower.as_str()) {
                // Si « leurs » est suivi d'un verbe (« leurs parlent », « leurs donnent »), c'est le pronom COI leur traité dans verifier_homophones
                if det_lower == "leurs" && (est_infinitif(&nom_lower) || nom_lower.ends_with("nt") || ["a", "ont", "font", "vont", "sont"].contains(&nom_lower.as_str())) {
                    continue;
                }'''

replacement = '''            // CAS 6 : Déterminant pluriel suivi d'un nom singulier (« les document » -> « documents »)
            if self.determinants_pluriel.contains(det_lower.as_str()) {
                // Si « leurs » est suivi d'un verbe (« leurs parlent », « leurs donnent »), c'est le pronom COI leur traité dans verifier_homophones
                if det_lower == "leurs" && (est_infinitif(&nom_lower) || nom_lower.ends_with("nt") || ["a", "ont", "font", "vont", "sont"].contains(&nom_lower.as_str())) {
                    continue;
                }'''

assert target in text, "Target not found in build_full_engine.py"
text = text.replace(target, replacement, 1)

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\ressources\build_full_engine.py", "w", encoding="utf-8") as f:
    f.write(text)

print("Updated build_full_engine.py successfully.")
