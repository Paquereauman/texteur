"""
mesure_rapport.py - Taux de détection de Texteur sur les fautes du rapport Suhou.

Pour chaque faute du rapport (hors erreurs historiques, calculs et « à vérifier ») :
la phrase passe dans Texteur (règles Rust via la CLI, Grammalecte + spaCy, orthographe) ;
la faute compte comme trouvée si une alerte recouvre le mot fautif (le mot de la phrase le plus
proche de la correction).

Usage : python -X utf8 mesure_rapport.py   (Python 3.12, celui de Grammalecte)
"""
import difflib
import json
import os
import re
import subprocess
import sys

ICI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.environ["USERPROFILE"], r"Desktop\Projets\AI"))
import grammalecte_serveur as gs  # noqa: E402

EXE = os.path.join(ICI, r"..\target\release\texteur.exe")
# Fautes de fond, hors du champ d'un correcteur de langue
HORS_LANGUE = re.compile(r"calcul|=|règne|dynastie|date incohérente|nom :|poète|province|empereur|graveur|studio|"
                         r"tableau|contresens|à vérifier|entrée à retirer|numéro|chiffres|caractères|pinyin|se lit",
                         re.I)
# Fautes qu'aucun correcteur ne peut voir dans la phrase seule (sens, faits, genre d'une personne réelle,
# caractères chinois, graphie des noms propres) : numéros du rapport, hors de la mesure
HORS_PORTEE = {67, 165, 183, 196, 219, 243, 253, 316, 325, 335, 408, 434, 448}
AVEC_CHIFFRES = {"Espace manquante avant un nombre", "Lettre à la place du chiffre 1", "Année incohérente avec le règne",
                 "Virgule décimale"}  # comme l'éditeur : les autres alertes portant sur des chiffres sont écartées
COUVERTES_GL = re.findall(r'"([^"]+)"', open(os.path.join(ICI, r"..\src\main.rs"), encoding="utf-8").read()
                          .split("REGLES_COUVERTES_PAR_GRAMMALECTE: &[&str] = &[")[1].split("];")[0])


def fautes():
    for l in open(os.path.join(ICI, "rapport_complet.tsv"), encoding="utf-8"):
        c = l.rstrip("\n").split(" | ")
        if len(c) >= 5 and c[0].isdigit() and c[2].strip() and not HORS_LANGUE.search(c[3] + " " + c[4])                 and int(c[0]) not in HORS_PORTEE:
            yield int(c[0]), c[2].strip(), c[3].strip(), c[4].strip()


def zone_fautive(phrase, correction):
    """(début, fin) du passage de la phrase le plus proche de la correction."""
    mots = [(m.start(), m.end()) for m in re.finditer(r"[\w’'-]+", phrase)]
    n = max(1, len(correction.split()))
    meilleur, score = None, 0
    for k in range(len(mots)):
        for t in (n - 1, n, n + 1):
            if t < 1 or k + t > len(mots):
                continue
            d, f = mots[k][0], mots[k + t - 1][1]
            # avec la ponctuation qui suit (« aînés, » → « aînés » : l'alerte porte sur la virgule)
            p = re.match(r"[ \u00a0\u202f]*[,.;:!?»)）]+", phrase[f:])
            for f2 in (f, f + p.end()) if p else (f,):
                r = difflib.SequenceMatcher(None, phrase[d:f2].lower(), correction.lower()).ratio()
                if r > score and phrase[d:f2].lower() != correction.lower():
                    meilleur, score = (d, f2), r
    return meilleur if score >= 0.5 else None


def alertes_rust(phrases):
    chemin = os.path.join(ICI, "_mesure.txt")
    open(chemin, "w", encoding="utf-8").write("\n".join(phrases))
    sortie = os.path.join(ICI, "_mesure.json")
    subprocess.run([EXE, chemin, "--ia", "--json", "--sortie", sortie], capture_output=True)
    rapport = json.load(open(sortie, encoding="utf-8"))
    debuts, k = [], 0
    for p in phrases:
        debuts.append(k)
        k += len(p) + 1
    res = [[] for _ in phrases]
    for a in rapport["alertes"]:
        if a["categorie"] == "Répétition" or a["regle"] in COUVERTES_GL                 or any(c.isdigit() for c in a["terme"]) and a["regle"] not in AVEC_CHIFFRES:
            continue
        i = max(j for j, d in enumerate(debuts) if d <= a["debut"])
        res[i].append((a["debut"] - debuts[i], a["fin"] - debuts[i], a["regle"]))
    return res


def alertes_grammalecte(p):
    res = [(e[0], e[1], e[3]) for e in gs.verifier(p)]
    for m in re.finditer(r"[a-zàâäéèêëïîôöùûüçœæ’'-]+", p):  # orthographe (mots en minuscules)
        if not gs.ORTHO.isValid(m.group().replace("’", "'")):
            res.append((m.start(), m.end(), "orthographe"))
    return res


def main():
    liste = list(fautes())
    phrases = [p for _, p, _, _ in liste]
    rust = alertes_rust(phrases)
    trouvees, ratees, sans_zone = 0, [], 0
    impaires = [0, 0]  # lignes jamais vues par le détecteur : [trouvées, total]
    for (n, p, corr, why), ar in zip(liste, rust):
        z = zone_fautive(p, corr)
        doublon = re.search(r"\b(\w+) \1\b", p, re.I) if re.search(r"répét|doublon", why, re.I) else None
        if doublon:  # « de de » : la faute est le mot répété, pas le mot le plus ressemblant ailleurs dans la phrase
            z = doublon.span()
        if z is None:
            sans_zone += 1
            continue
        al = ar + alertes_grammalecte(p)
        ok = any(d < z[1] and z[0] < f for d, f, _ in al)
        if n % 2:
            impaires[0] += ok; impaires[1] += 1
        if ok:
            trouvees += 1
        else:
            ratees.append((n, p[max(0, z[0] - 40):z[1] + 20], corr, why[:60]))
    total = trouvees + len(ratees)
    print(f"Fautes de langue mesurées : {total} (non localisables : {sans_zone})")
    print(f"TROUVÉES : {trouvees}/{total} = {100 * trouvees / total:.0f} %")
    print(f"Lignes impaires (non vues à l'entraînement) : {impaires[0]}/{impaires[1]} = {100 * impaires[0] / max(1, impaires[1]):.0f} %")
    with open(os.path.join(ICI, "_ratees.txt"), "w", encoding="utf-8") as f:
        for r in ratees:
            f.write(" | ".join(map(str, r)) + "\n")


if __name__ == "__main__":
    main()
