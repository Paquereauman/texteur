path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs"
with open(path, "r", encoding="utf-8") as f:
    code = f.read()

new_test = """
    #[test]
    fn test_capture_ecran_oct30() {
        let g = CorrecteurGrammaire::new();
        let texte = "Bonjour, je LE vous envoieS le document concernant le projet 長城 (Chángchéng).\\nLes fournisseur de Shenzhen 深圳 ont confirmer la livraison du projet.\\nNous devons faire un réunion pour faire sa point sur ce problème et régler ce problème rapidement.";
        let alertes = g.analyse(texte);
        println!("=== ALERTES OBTENUES ===");
        for a in &alertes {
            println!("  [{}] '{}' -> {:?}", a.regle, a.texte_fautif, a.suggestions);
        }
    }
"""

if "fn test_capture_ecran_oct30" not in code:
    code = code[:-2] + new_test + "\n}\n"
    with open(path, "w", encoding="utf-8") as f:
        f.write(code)
    print("Added test_capture_ecran_oct30")
