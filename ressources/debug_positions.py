texte = """Bonjour, je te envoie le document concernent le projet 長城 (Chángchéng).
Les fournisseur du Shenzhen 深圳 ont confirmer la livraison du projet.
Nous devons faire un réunion pour faire la point sur se problème et raigler ce problème rapidement."""

for i, l in enumerate(texte.splitlines()):
    print(f"Ligne {i}")
    for word in ["je", "te", "envoie", "concernent", "fournisseur", "du", "Shenzhen", "ont", "confirmer", "un", "réunion", "la", "point", "se", "problème", "raigler"]:
        if word in l:
            idx = l.find(word)
            print(f"  '{word}' in line {i} at char {idx}..{idx+len(word)}")
