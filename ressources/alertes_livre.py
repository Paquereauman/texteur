"""
alertes_livre.py - Alertes de chaque source du serveur sur la version finale corrigée du livre Suhou.

Le texte est déjà corrigé : presque toute alerte y est une fausse alerte. Donne, par source
(Grammalecte, spaCy, détecteur), le nombre d'alertes pour 1 000 mots, la durée, et écrit
des exemples dans _alertes_livre.txt pour juger à la main.
Usage : python -X utf8 alertes_livre.py [docx]
"""
import collections
import os
import re
import sys
import time
import zipfile

sys.path.insert(0, os.path.join(os.environ["USERPROFILE"], r"Desktop\Projets\AI"))
import grammalecte_serveur as gs  # noqa: E402

ICI = os.path.dirname(os.path.abspath(__file__))
LIVRE = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.environ["USERPROFILE"], r"Desktop\Livre correction\Fichier finaux livre Suhou\Backup",
    "Tang Lixing et al (Version finale révisée et corrigée).docx")


def paragraphes(chemin):
    xml = zipfile.ZipFile(chemin).read("word/document.xml").decode("utf-8")
    for p in re.findall(r"<w:p[ >].*?</w:p>", xml, re.S):
        t = "".join(re.findall(r"<w:t[^>]*>([^<]*)</w:t>", p))
        t = t.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", '"').replace("&apos;", "'")
        if sum(c.isalpha() for c in t) > 20:
            yield t


def source(e):
    return "détecteur" if e[3] == "detecteur" else "spaCy" if e[3].startswith("spacy") else "Grammalecte"


def main():
    paras = list(paragraphes(LIVRE))
    mots = sum(len(re.findall(r"\w+", p)) for p in paras)
    t = time.time()
    res = []
    for a in range(0, len(paras), 50):
        res += gs.verifier_lot(paras[a:a + 50])
    duree = time.time() - t
    compte, exemples = collections.Counter(), collections.defaultdict(list)
    for p, erreurs in zip(paras, res):
        for e in erreurs:
            s = source(e)
            compte[s] += 1
            exemples[s].append(f"{p[max(0, e[0] - 50):e[0]]}[[{p[e[0]:e[1]]}]]{p[e[1]:e[1] + 30]}  → {e[5][:2]}  ({e[3]})")
    print(f"{len(paras)} paragraphes, {mots} mots, {duree:.0f} s ({mots / duree:.0f} mots/s)")
    for s, n in compte.most_common():
        print(f"  {s:12} {n:5} alertes  {1000 * n / mots:.2f} / 1000 mots")
    with open(os.path.join(ICI, "_alertes_livre.txt"), "w", encoding="utf-8") as f:
        for s, ex in exemples.items():
            f.write(f"=== {s} ({len(ex)})\n" + "\n".join(ex) + "\n")


if __name__ == "__main__":
    main()
