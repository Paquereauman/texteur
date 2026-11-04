"""
graphique_progres.py - Courbe des progrès de Texteur jour par jour (progres.svg, affichée dans le README).
Données : ressources/progres.csv (date ; % des fautes trouvées ; % sur les lignes jamais vues ; étape).
mesure_rapport.py y ajoute une ligne à chaque mesure ; la dernière mesure de chaque jour fait foi.
Usage : python -X utf8 ressources/graphique_progres.py
"""
import csv
import os

ICI = os.path.dirname(os.path.abspath(__file__))
CSV = os.path.join(ICI, "progres.csv")
SVG = os.path.join(ICI, "..", "progres.svg")
L, H, G, D, HT, B = 760, 340, 56, 150, 54, 44  # largeur, hauteur, marges gauche/droite/haut/bas
BAS, HAUT = 50, 100  # axe des % (de 50 à 100)
SERIES = [("trouvees", "Toutes les fautes", "s1"), ("impaires", "Jamais vues", "s2")]


def jours():
    par_jour = {}
    for r in csv.DictReader(open(CSV, encoding="utf-8"), delimiter=";"):
        par_jour[r["date"]] = r  # la dernière mesure du jour l'emporte
    return [par_jour[d] for d in sorted(par_jour)]


def ajouter(trouvees, impaires, etape=""):
    """Appelé par mesure_rapport.py après chaque mesure."""
    import datetime
    neuf = not os.path.exists(CSV)
    with open(CSV, "a", encoding="utf-8", newline="") as f:
        if neuf:
            f.write("date;trouvees;impaires;etape\n")
        f.write(f"{datetime.date.today()};{trouvees:.1f};{impaires:.1f};{etape}\n")
    dessiner()


def dessiner():
    rs = jours()
    n = len(rs)
    x = lambda i: G + (L - G - D) * (i / (n - 1) if n > 1 else 0.5)
    y = lambda v: HT + (H - HT - B) * (HAUT - v) / (HAUT - BAS)
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {L} {H}" font-family="system-ui,Segoe UI,sans-serif" '
         f'role="img" aria-label="Progrès de Texteur : part des fautes du rapport trouvées, jour par jour">',
         "<style>.fond{fill:#ffffff}.t1{fill:#1f2328}.t2{fill:#59636e}.grille{stroke:#d8dee4}.s1{stroke:#2563c9;fill:#2563c9}"
         ".s2{stroke:#d4660f;fill:#d4660f}.anneau{stroke:#ffffff}"
         "@media (prefers-color-scheme:dark){.fond{fill:#0d1117}.t1{fill:#e6edf3}.t2{fill:#9198a1}.grille{stroke:#30363d}"
         ".s1{stroke:#4f86e0;fill:#4f86e0}.s2{stroke:#dd7426;fill:#dd7426}.anneau{stroke:#0d1117}}</style>",
         f'<rect class="fond" width="{L}" height="{H}" rx="8"/>',
         f'<text class="t1" x="{G}" y="24" font-size="15" font-weight="600">Fautes du rapport trouvées par Texteur</text>']
    lx = G
    for _, nom, c in SERIES:  # légende
        o.append(f'<line class="{c}" x1="{lx}" y1="40" x2="{lx + 18}" y2="40" stroke-width="2"/>'
                 f'<text class="t2" x="{lx + 24}" y="44" font-size="12">{nom}</text>')
        lx += 150
    for v in range(BAS, HAUT + 1, 10):
        o.append(f'<line class="grille" x1="{G}" y1="{y(v):.1f}" x2="{L - D + 20}" y2="{y(v):.1f}" stroke-width="1"/>'
                 f'<text class="t2" x="{G - 8}" y="{y(v) + 4:.1f}" font-size="11" text-anchor="end">{v} %</text>')
    pas = max(1, n // 8)
    for i, r in enumerate(rs):
        if i % pas == 0 or i == n - 1:
            j = r["date"][8:10] + "/" + r["date"][5:7]
            o.append(f'<text class="t2" x="{x(i):.1f}" y="{H - B + 18}" font-size="11" text-anchor="middle">{j}</text>')
    for cle, nom, c in SERIES:
        pts = [(x(i), y(float(r[cle])), float(r[cle]), r) for i, r in enumerate(rs) if r[cle]]
        if len(pts) > 1:
            o.append(f'<polyline class="{c}" fill="none" stroke-width="2" stroke-linejoin="round" '
                     f'points="{" ".join(f"{a:.1f},{b:.1f}" for a, b, _, _ in pts)}"/>')
        for a, b, v, r in pts:
            o.append(f'<circle class="{c} anneau" cx="{a:.1f}" cy="{b:.1f}" r="4" stroke-width="2" style="stroke-width:2">'
                     f'<title>{r["date"]} : {nom.lower()} {v:.0f} %{" — " + r["etape"] if r["etape"] else ""}</title></circle>')
        if pts:
            a, b, v, _ = pts[-1]
            o.append(f'<text class="t1" x="{a + 10:.1f}" y="{b + 4:.1f}" font-size="12">{nom} : {v:.0f} %</text>')
    o.append("</svg>")
    open(SVG, "w", encoding="utf-8").write("\n".join(o))


if __name__ == "__main__":
    dessiner()
