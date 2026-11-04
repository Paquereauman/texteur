path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs"
with open(path, "r", encoding="utf-8") as f:
    code = f.read()

new_test = """
    #[test]
    fn test_user_exact_screenshot_oct31() {
        let g = CorrecteurGrammaire::new();
        let texte = "Bonjour, je te envoie le document concernent le projet 長城 (Chángchéng).\\nLes fournisseur du Shenzhen 深圳 ont confirmer la livraison du projet.\\nNous devons faire un réunion pour faire la point sur se problème et raigler ce problème rapidement.";
        let alertes = g.analyse(texte);
        println!("=== ALERTES DU TEST EXACT OCT 31 ===");
        for a in &alertes {
            println!("  [{}] '{}' -> {:?}", a.regle, a.texte_fautif, a.suggestions);
        }
    }
"""

if "fn test_user_exact_screenshot_oct31" not in code:
    code = code[:-2] + new_test + "\n}\n"
    with open(path, "w", encoding="utf-8") as f:
        f.write(code)
    print("Added test_user_exact_screenshot_oct31")
