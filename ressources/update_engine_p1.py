import re

path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs"
with open(path, "r", encoding="utf-8") as f:
    code = f.read()

# Helper function to estimate infinitive from past participle if not present
if "fn estimer_participe_infinitif" not in code:
    inf_pp_helper = """
fn estimer_participe_infinitif(inf: &str) -> Option<String> {
    if inf.ends_with("er") && inf.len() > 2 {
        Some(format!("{}é", &inf[..inf.len() - 2]))
    } else {
        None
    }
}
"""
    # Insert before fn est_infinitif
    code = code.replace("fn est_infinitif(mot: &str) -> bool {", inf_pp_helper + "\nfn est_infinitif(mot: &str) -> bool {")

print("Added estimer_participe_infinitif")

# Let's inspect verifier_accord_determinant to support adjectives between determiner and noun
# Specifically, we want:
# 1. CAS 3: Déterminant masculin devant nom féminin
# 2. CAS 4: Déterminant féminin devant nom masculin
# With optional intervening adjective (grand, petite, nouveau, etc.)

# Let's check how verifier_accord_determinant is structured
with open(path, "w", encoding="utf-8") as f:
    f.write(code)
print("Updated base helpers")
