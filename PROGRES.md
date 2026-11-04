# Progrès de Texteur

**Mesure :** part des 345 fautes de langue (grammaire, style, syntaxe, ponctuation) d'un rapport de relecture
professionnelle que Texteur repère. Les erreurs hors de portée d'un correcteur ne sont pas comptées : sens, faits
historiques, caractères chinois. Script : `ressources/mesure_rapport.py`.

- **Lignes paires :** elles servent à concevoir les règles et à entraîner le détecteur.
- **Lignes impaires :** jamais vues. C'est le chiffre honnête.

**Fausses alertes :** alertes sur des phrases justes, comptées pour 1000 mots. Elles sont mesurées sur Wikipédia et
sur le livre corrigé.

| Date | Étape | Fautes trouvées | Lignes jamais vues | Fausses alertes (Wikipédia / livre) |
|---|---|---|---|---|
| 2026-11-03 | Règles Rust + Grammalecte + CamemBERT | 75 % | — | — |
| 2026-11-04 | Erreurs hors de portée exclues, détecteur avec contre-vérification | 73 % | 68 % | 0,74 / 0,49 |
| 2026-11-04 | Règles spaCy : « de » + adjectif, subjonctif, « où … y », « après êtres »… | 77 % | 70 % | 0,63 / 0,53 |
| 2026-11-04 | Majuscule après un point, nombres non protégés, participe avec « avoir », préposition du verbe, article oublié | 78 % | 71 % | 0,63 / 0,53 |

**Comparaison :** LanguageTool 6.6 seul trouve 34 % de ces fautes. Ajouté à Texteur, il n'apporterait que 2 points.

## Qui trouve quoi (2026-11-04)

Une même faute peut être trouvée par plusieurs moteurs.

| Moteur | Fautes trouvées | Trouvées par lui seul |
|---|---|---|
| Règles Rust | 91 | 47 |
| CamemBERT | 90 | 67 |
| Orthographe | 78 | 40 |
| Grammalecte | 36 | 22 |
| Règles spaCy | 39 | 30 |
