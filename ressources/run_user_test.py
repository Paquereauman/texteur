import subprocess

test_code = r'''
    #[test]
    fn test_texte_complet_utilisateur() {
        let g = CorrecteurGrammaire::new();
        let texte = "Bonjour, je vous envoie le document concernant le projet 长城 (Chángchéng).\nLes fournisseur de Shenzhen 深圳 ont confirmer la livraison du projet.\nNous devons faire une réunion pour faire le point sur ce problème et régler ce problème rapidement.\n\nLa fille que tu as vu\n\nLes lunettes marrons.";
        let alertes = g.analyse(texte);
        println!("TOTAL ALERTES GRAM: {}", alertes.len());
        for a in &alertes {
            println!(" - '{}' -> {:?} ({})", a.texte_fautif, a.suggestions, a.regle);
        }
        assert!(alertes.iter().any(|a| a.texte_fautif == "fournisseur"));
        assert!(alertes.iter().any(|a| a.texte_fautif == "confirmer"));
        assert!(alertes.iter().any(|a| a.texte_fautif == "vu"));
        assert!(alertes.iter().any(|a| a.texte_fautif == "marrons"));
    }
'''

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'r', encoding='utf-8') as f:
    content = f.read()

if 'test_texte_complet_utilisateur' not in content:
    idx = content.rfind('}')
    new_content = content[:idx] + test_code + '\n}\n'
    with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)

res = subprocess.run(['cargo', 'test', '--', 'test_texte_complet_utilisateur', '--nocapture'], cwd=r'c:\Users\Bpaquereau\Desktop\Antidote\texteur', capture_output=True, text=True)
print(res.stdout)
