# Texteur

**Correcteur de français et éditeur de texte, rapide, fiable et simple : grammaire, style, syntaxe, ponctuation.**

Texteur est un éditeur de bureau (Rust + [egui](https://github.com/emilk/egui)) qui relit un texte français comme le
ferait un correcteur professionnel. Il souligne les fautes, explique la règle et propose la correction. Il a été conçu
pour relire des ouvrages longs, notamment des textes d'histoire chinoise qui mêlent français, pinyin et caractères
chinois.

![Texteur : le texte souligné à gauche, les explications et corrections à droite](capture.png)

## Ce qu'il repère

- **Grammaire :** accords (sujet-verbe, participes avec « être » et « avoir », épithètes, attributs), homophones,
  conjugaison, subjonctif après « bien que », « pour que »…
- **Syntaxe :** prépositions imposées par le verbe (« dépendre de »), articles oubliés, mots en double, « où … y ».
- **Style :** répétitions, tournures lourdes, expressions figées mal employées (« aller de l'avant »).
- **Typographie :** espaces, majuscules en début de phrase, ponctuation française.
- **Textes sinisants :** le dictionnaire chinois (caractères, pinyin, sens) protège les noms propres pour qu'ils ne
  soient jamais « corrigés ».

## Comment il fonctionne

Trois moteurs se complètent. Chaque alerte n'est affichée qu'une fois :

| Moteur | Rôle |
|---|---|
| **Règles Rust** (`src/grammaire.rs`, `src/tournures.rs`) | Règles écrites à la main : rapides, sûres, expliquées |
| **Serveur Python** (optionnel, `--ia`) | [Grammalecte](https://grammalecte.net) + analyse syntaxique spaCy (accords à distance, subjonctif, prépositions) |
| **Détecteur CamemBERT** (optionnel) | Modèle entraîné à repérer les fautes que les règles ne savent pas décrire |

Le serveur Python et le détecteur tournent en local : le texte ne quitte jamais le poste.

## Installation

```bash
cargo build --release
```

L'exécutable se trouve dans `target/release/texteur.exe`. Le menu **« ⬆ Mise à jour »** installe une nouvelle version.

En ligne de commande :

```bash
texteur fichier.txt --ia --json --sortie alertes.json
```

## Suivi des progrès

![Progrès de Texteur jour par jour](progres.svg)

La fiabilité est mesurée sur un rapport de relecture professionnelle : 345 vraies fautes de langue relevées dans un
livre. Le texte du livre n'est pas publié, pour des raisons de droits d'auteur. Le détail des mesures est dans
[PROGRES.md](PROGRES.md).
La courbe se met à jour à chaque mesure : `ressources/mesure_rapport.py` ajoute une ligne à `ressources/progres.csv`
puis redessine `progres.svg`.

## Licence

[MIT](LICENSE). Grammalecte, utilisé comme serveur séparé, est sous licence GPL v3.
