import re

path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs"
with open(path, "r", encoding="utf-8") as f:
    code = f.read()

benchmark_code = """
    #[test]
    fn test_benchmark_exhaustif_diagnostique() {
        let g = CorrecteurGrammaire::new();
        let tests = vec![
            ("Les fournisseur du Shenzhen on confirmai les livraison du projet.", vec!["fournisseur", "du", "on", "confirmai", "livraison"]),
            ("Nous devons faire une réunion pour faire le point sur ce problème et régler se problème rapidement.", vec!["se"]),
            ("Malgré que le projet avance, il reste des blocages.", vec!["Malgré que"]),
            ("Après qu'il soit venu, nous avons discuté.", vec!["soit"]),
            ("Où que tu vas pour les vacances ?", vec!["que"]),
            ("Par contre nous devons agir vite.", vec!["Par contre"]),
            ("Quelque soit la décision, nous sommes prêts.", vec!["Quelque soit"]),
            ("Comme par exemple lors de la réunion.", vec!["Comme par exemple"]),
            ("Il a réussi voire même dépassé les objectifs.", vec!["voire même"]),
            ("Il a rencontré un difficulté majeure.", vec!["un"]),
            ("C'est une grand problème pour l'équipe.", vec!["une"]),
            ("Ce arbre est très vieux.", vec!["Ce"]),
            ("Cette accord a été signé hier.", vec!["Cette"]),
            ("Ma amie est venue me voir.", vec!["Ma"]),
            ("Sa entreprise recrute actuellement.", vec!["Sa"]),
            ("Des bal ont été organisés.", vec!["bal"]),
            ("Les chovals courent dans le pré.", vec!["chovals"]),
            ("Toutes les document sont archivés.", vec!["document"]),
            ("Chaque jours nous faisons une vérification.", vec!["jours"]),
            ("Sa va très bien aujourd'hui.", vec!["Sa"]),
            ("Machine a café en panne.", vec!["a"]),
            ("La ville ou il habite est agréable.", vec!["ou"]),
            ("Ces arrivé très vite sans prévenir.", vec!["Ces"]),
            ("Il ces trompé dans son calcul.", vec!["ces"]),
            ("Ils son très contents du résultat.", vec!["son"]),
            ("Je lui donne leurs adresse.", vec!["leurs"]),
            ("Il a mangé un peut de pain.", vec!["peut"]),
            ("Je peu le faire sans problème.", vec!["peu"]),
            ("Il peux venir demain matin.", vec!["peux"]),
            ("Tout les jours nous apprenons.", vec!["Tout"]),
            ("Tous le monde est invité.", vec!["Tous"]),
            ("Quand à moi, je préfère attendre.", vec!["Quand à"]),
            ("Si j'aurais su, je ne serais pas venu.", vec!["aurais"]),
            ("Elle a manger à midi.", vec!["manger"]),
            ("Il faut mangé pour avoir des forces.", vec!["mangé"]),
            ("Sans hésité, il a accepté l'offre.", vec!["hésité"]),
            ("Pour validé ce document, il faut signer.", vec!["validé"]),
            ("La réunion que nous avons organisé hier.", vec!["organisé"]),
            ("Elles se sont parlées pendant une heure.", vec!["parlées"]),
            ("Ils se sont téléphonés hier soir.", vec!["téléphonés"]),
            ("Des vestes marrons.", vec!["marrons"]),
            ("Tu mange trop vite.", vec!["mange"]),
            ("Ils mange ensemble chaque jour.", vec!["mange"]),
            ("Vous faisez fausse route.", vec!["faisez"]),
            ("Vous disez n'importe quoi.", vec!["disez"]),
        ];

        let mut reussis = 0;
        let mut echecs = 0;
        for (phrase, cibles) in &tests {
            let alertes = g.analyse(phrase);
            let alertes_str: Vec<String> = alertes.iter().map(|a| a.texte_fautif.clone()).collect();
            let mut manque = Vec::new();
            for c in cibles {
                if !alertes_str.iter().any(|a| a.eq_ignore_ascii_case(c) || a.to_lowercase().contains(&c.to_lowercase())) {
                    manque.push(*c);
                }
            }
            if manque.is_empty() {
                reussis += 1;
            } else {
                echecs += 1;
                println!("MANQUE dans '{}' : {:?}", phrase, manque);
                println!("  Alertes obtenues : {:?}", alertes_str);
            }
        }
        println!("BILAN DIAGNOSTIQUE : {} réussis, {} avec éléments manquants sur {} tests.", reussis, echecs, tests.len());
    }
}
"""

# Replace the last `}` of the file (closing mod tests)
last_brace_idx = code.rfind('}')
new_code = code[:last_brace_idx] + benchmark_code

with open(path, "w", encoding="utf-8") as f:
    f.write(new_code)
print("Added benchmark test to grammaire.rs")
