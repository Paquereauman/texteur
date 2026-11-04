import subprocess
import json

test_sentences = [
    # 1. User screenshot cases
    "Les fournisseur du Shenzhen 深圳 on confirmai les livraison du projet.",
    "Nous devons faire une réunion pour faire le point sur ce problème et régler se problème rapidement.",
    
    # 2. Syntax & Pronoun order & Subordination
    "Malgré que le projet avance, il reste des blocages.",
    "Après qu'il soit venu, nous avons discuté.",
    "Où que tu vas pour les vacances ?",
    "C'est de cela dont je vous parlais.",
    "Par contre nous devons agir vite.",
    "Quelque soit la décision, nous sommes prêts.",
    "Quelques soient les difficultés rencontrées.",
    "Comme par exemple lors de la dernière réunion.",
    "Il a réussi voire même dépassé les objectifs.",
    "Je veux donner le à lui.",
    
    # 3. Determiner + Noun agreements & Euphony
    "Il a rencontré un difficulté majeure.",
    "C'est une grand problème pour l'équipe.",
    "Ce arbre est très vieux.",
    "Cette accord a été signé hier.",
    "Ma amie est venue me voir.",
    "Sa entreprise recrute actuellement.",
    "Des bal ont été organisés dans la ville.",
    "Les chovals courent dans le pré.",
    "Toutes les document sont archivés.",
    "Chaque jours nous faisons une vérification.",
    
    # 4. Homophones & Phonetics
    "Sa va très bien aujourd'hui.",
    "Machine a café en panne.",
    "La ville ou il habite est agréable.",
    "Ces arrivé très vite sans prévenir.",
    "Il ces trompé dans son calcul.",
    "Ils son très contents du résultat.",
    "Je lui donne leurs adresse.",
    "Ils leur ont donné leurs livres.",
    "Il a mangé un peut de pain.",
    "Je peu le faire sans problème.",
    "Il peux venir demain matin.",
    "Tout les jours nous apprenons.",
    "Tous le monde est invité.",
    "Quand à moi, je préfère attendre.",
    
    # 5. Verbs & Participe passé
    "Si j'aurais su, je ne serais pas venu.",
    "Si il viendrait, dis-le moi.",
    "Elle a manger à midi.",
    "Il faut mangé pour avoir des forces.",
    "Sans hésité, il a accepté l'offre.",
    "Pour validé ce document, il faut signer.",
    "Les lettres que tu as envoyé ce matin.",
    "La réunion que nous avons organisé hier.",
    "Elles se sont parlées pendant une heure.",
    "Ils se sont téléphonés hier soir.",
    "Des vestes marrons et des jupes bleu marine.",
    "Tu mange trop vite.",
    "Ils mange ensemble chaque jour.",
    "Vous faisez fausse route.",
    "Vous disez n'importe quoi.",
]

print(f"Total test sentences to evaluate: {len(test_sentences)}")
