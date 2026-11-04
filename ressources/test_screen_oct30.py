import subprocess

text = """Bonjour, je LE vous envoieS le document concernant le projet 長城 (Chángchéng).
Les fournisseur de Shenzhen 深圳 ont confirmer la livraison du projet.
Nous devons faire un réunion pour faire sa point sur ce problème et régler ce problème rapidement."""

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\ressources\test_screen_oct30.txt", "w", encoding="utf-8") as f:
    f.write(text)

print("Saved test text")
