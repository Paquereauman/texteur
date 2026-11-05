//! Module pédagogique complet pour Texteur
//! Fournit les explications académiques approfondies, les règles officielles
//! et les astuces mnémotechniques pratiques lorsque l'utilisateur clique sur « Plus d'infos ».

#[derive(Clone, Debug)]
pub struct FichePedagogique {
    pub titre: &'static str,
    pub regle_complete: &'static str,
    pub astuce: &'static str,
    pub exemple_correct: &'static str,
}

impl FichePedagogique {
    /// Une vraie fiche existe pour cette faute (sinon, pas de bouton « Plus d'infos »).
    pub fn utile(&self) -> bool {
        !self.titre.is_empty()
    }
}

/// Analyse la règle et le contexte de la faute pour retourner la fiche pédagogique idoine.
pub fn explication_pedagogique_approfondie(
    regle: &str,
    texte_fautif: &str,
    explication: &str,
) -> FichePedagogique {
    let r_lower = regle.to_lowercase();
    let mot_lower = texte_fautif.to_lowercase();
    let exp_lower = explication.to_lowercase();

    // 1. PARTICIPE PASSÉ APRÈS AVOIR / ÊTRE (ex: « ont confirmer » -> « confirmé »)
    if r_lower.contains("participe passé") || r_lower.contains("auxiliaire avoir") || exp_lower.contains("auxiliaire avoir") {
        return FichePedagogique {
            titre: "Participe passé employé avec l'auxiliaire avoir ou être",
            regle_complete: "Après un auxiliaire (« a », « ont », « avait », « sera »...), le verbe qui suit doit être sous sa forme de participe passé (terminaison en -é, -i, -u, -is, -it) pour former un temps composé. Écrire un infinitif en -er après « ont » ou « a » constitue une faute majeure de conjugaison.",
            astuce: "Remplacez mentalement par un verbe du 3ᵉ groupe comme « vendre » ou « prendre » :\nOn dit « ils ont vendu » (et JAMAIS « ils ont vendre »), donc il faut impérativement la terminaison du participe passé : « ont confirmé » !",
            exemple_correct: "« Les fournisseurs ont confirmé (vendu) la livraison. »",
        };
    }

    // 2. ÉLISION OBLIGATOIRE DEVANT VOYELLE (ex: « te envoie » -> « t'envoie », « le accord » -> « l'accord »)
    if r_lower.contains("élision") || r_lower.contains("elision") {
        if mot_lower.contains("si") {
            return FichePedagogique {
                titre: "Élision obligatoire de la conjonction « si »",
                regle_complete: "En français, la conjonction de subordination « si » s'élide obligatoirement devant les pronoms masculins « il » et « ils » (« s'il », « s'ils »). En revanche, elle ne s'élide jamais devant « elle », « elles » ou « on » (« si elle », « si on »).",
                astuce: "Retenez la règle galante : « si » s'élide pour les messieurs (s'il / s'ils), mais reste tout entier pour les dames (si elle) !",
                exemple_correct: "« S'il vous plaît » / « S'ils confirment leur présence ». ",
            };
        }
        return FichePedagogique {
            titre: "Règle générale de l'élision euphonique obligatoire",
            regle_complete: "Les petits mots monosyllabiques se terminant par un 'e' caduc (« je », « me », « te », « se », « de », « le », « la », « ne », « que », « jusque ») s'élident obligatoirement avec une apostrophe lorsqu'ils précèdent un mot débutant par une voyelle ou un 'h' muet. Cette règle évite le hiatus (heurt désagréable de deux voyelles consécutives à l'oral).",
            astuce: "Prononcez la suite à voix haute d'un seul souffle :\nOn ne dit pas « je te envoie », mais « je t'envoie » directement lié avec l'apostrophe !",
            exemple_correct: "« Je t'envoie le document concernant le projet. »",
        };
    }

    // 3. ORDRE DES PRONOMS PERSONNELS CONJOINTS (ex: « le vous » -> « vous l' »)
    if r_lower.contains("ordre des pronoms") || r_lower.contains("pronom personnel conjoint") {
        return FichePedagogique {
            titre: "Ordre obligatoire des pronoms personnels conjoints antéposés",
            regle_complete: "Lorsque deux pronoms compléments précèdent le verbe en français moderne, les pronoms de 1ʳᵉ et 2ᵉ personnes (« me », « te », « nous », « vous ») se placent TOUJOURS avant les pronoms de 3ᵉ personne compléments d'objet direct (« le », « la », « les »). L'ordre inverse (« je le vous envoie ») est calqué sur l'ancien français et est fautif aujourd'hui.",
            astuce: "Pensez à la politesse : la personne à qui vous vous adressez (« vous ») passe d'abord !\nÉcrivez : « je vous le transmets » ou devant voyelle « je vous l'envoie ».",
            exemple_correct: "« Je vous l'envoie immédiatement. »",
        };
    }

    // 4. TOPONYMES DE VILLES SANS ARTICLE (ex: « du Shenzhen » -> « de Shenzhen »)
    if r_lower.contains("nom de ville") || r_lower.contains("toponyme") || r_lower.contains("ville") {
        return FichePedagogique {
            titre: "Syntaxe des noms de villes et agglomérations",
            regle_complete: "En français, les noms de villes ne prennent pas d'article défini (sauf rares exceptions historiques figées comme Le Havre, Le Caire ou La Rochelle). Pour marquer la provenance ou l'appartenance, on emploie la préposition simple « de » (ou « d' » devant voyelle), et jamais l'article contracté « du » ou « des ».",
            astuce: "Shenzhen, Suzhou, Paris, Lyon sont des villes sans article :\nOn écrit « les fournisseurs de Shenzhen », « le bureau de Suzhou », « l'aéroport de Paris ».",
            exemple_correct: "« Les fournisseurs de Shenzhen et de Suzhou ont confirmé. »",
        };
    }

    // 5. ACCORD DU DÉTERMINANT EN GENRE (ex: « un réunion » -> « une réunion », « la point » -> « le point », « sa point » -> « le/son point »)
    if r_lower.contains("accord en genre") || r_lower.contains("genre du déterminant") || r_lower.contains("féminin requis") || r_lower.contains("masculin requis") {
        return FichePedagogique {
            titre: "Accord en genre du déterminant avec le nom substantif",
            regle_complete: "Le déterminant (article défini, indéfini, possessif ou démonstratif) reçoit obligatoirement le genre (masculin ou féminin) du nom qu'il introduit. En français, chaque nom a un genre intrinsèque : « réunion » et « solution » sont féminins, alors que « point » et « problème » sont masculins.",
            astuce: "Pour tester le genre sans hésiter, ajoutez un adjectif parlant :\n- « Une grande réunion » -> féminin : « une réunion »\n- « Un bon point » -> masculin : « faire le point » (ou « son point »)\n- « Un grand problème » -> masculin : « ce problème » !",
            exemple_correct: "« Nous devons faire une réunion pour faire le point. »",
        };
    }

    // 6. HOMOPHONES SE / CE (ex: « se problème » -> « ce problème »)
    if r_lower.contains("se / ce") || (mot_lower == "se" && exp_lower.contains("ce")) || (mot_lower == "ce" && exp_lower.contains("se")) {
        return FichePedagogique {
            titre: "Distinction fondamentale entre « ce » (démonstratif) et « se » (réfléchi)",
            regle_complete: "« Ce » (avec un C) est un déterminant démonstratif qui accompagne un nom pour le désigner (« ce problème », « ce document ») ou le pronom neutre sujet du verbe être (« c'est »). « Se » (avec un S) est un pronom réfléchi de 3ᵉ personne qui accompagne toujours un verbe pronominal (« il se lève », « ils se réunissent »).",
            astuce: "Remplacez par « celui-ci » ou « celle-ci » :\nSi vous pouvez dire « sur celui-ci problème », il s'agit du démonstratif : écrivez « ce » avec un C comme dans « Celui-ci » !",
            exemple_correct: "« Régler ce problème » / « L'équipe se réunit ». ",
        };
    }

    // 7. CONFUSION CONCERNENT / CONCERNANT
    if r_lower.contains("concernent") || mot_lower == "concernent" {
        return FichePedagogique {
            titre: "Distinction entre la préposition « concernant » et le verbe « concernent »",
            regle_complete: "« Concernant » (avec -ant) est une préposition invariable (issue du participe présent) équivalente à « au sujet de » ou « à propos de ». « Concernent » (avec -ent) est la forme conjuguée du verbe concerner au présent à la 3ᵉ personne du pluriel (sujet pluriel : « ils / elles concernent »).",
            astuce: "Faites la substitution par « au sujet de » :\n« Le document au sujet du projet » est parfaitement correct, donc écrivez la préposition invariable en -ant : « concernant le projet » !",
            exemple_correct: "« Le document concernant le projet 長城. »",
        };
    }

    // 8. ACCORD DU PLURIEL DES NOMS ET ADJECTIFS (ex: « Les fournisseur » -> « fournisseurs »)
    if r_lower.contains("pluriel") || r_lower.contains("accord du pluriel") {
        return FichePedagogique {
            titre: "Accord en nombre dans le groupe nominal au pluriel",
            regle_complete: "Lorsque le déterminant est au pluriel (« les », « des », « ces », « plusieurs », « quelques »), le nom et les adjectifs qui le qualifient prennent obligatoirement la marque du pluriel (habituellement la lettre 's', ou 'x' pour les mots en -au, -eau, -eu).",
            astuce: "Le déterminant est le phare du groupe nominal : dès que vous lisez « Les », préparez-vous à mettre un « s » au nom qui suit : « Les fournisseurs » !",
            exemple_correct: "« Les fournisseurs de Shenzhen ont confirmé la livraison. »",
        };
    }

    // 9. HOMOPHONES C'EST / S'EST
    if r_lower.contains("c'est") || r_lower.contains("s'est") {
        return FichePedagogique {
            titre: "Distinction entre le présentatif « c'est » et le verbe pronominal « s'est »",
            regle_complete: "« C'est » (C apostrophe) est la contraction du pronom démonstratif « ce » et du verbe « est » (signifie « cela est »). « S'est » (S apostrophe) est le pronom réfléchi « se » associé à l'auxiliaire « est » dans la conjugaison d'un verbe pronominal au passé composé.",
            astuce: "Transposez à l'imparfait :\n- Si vous pouvez dire « c'était », écrivez « c'est » avec un C !\n- Si vous dites « il s'était », écrivez « s'est » avec un S !",
            exemple_correct: "« C'est un point crucial » / « Le fournisseur s'est engagé. »",
        };
    }

    // 10. HOMOPHONES A / À
    if r_lower.contains("a / à") || r_lower.contains("a et à") {
        return FichePedagogique {
            titre: "Distinction entre le verbe avoir « a » et la préposition « à »",
            regle_complete: "« a » sans accent est le verbe avoir conjugué à la 3ᵉ personne du singulier au présent de l'indicatif. « à » avec accent grave est une préposition invariable introduisant un complément de lieu, de temps ou de manière.",
            astuce: "Remplacez par l'imparfait « avait » :\nSi la phrase reste correcte (« il avait réussi »), écrivez « a » sans accent. Si c'est impossible (« aller avait Shenzhen » est faux), mettez l'accent grave : « à » !",
            exemple_correct: "« Il a réussi à négocier le contrat. »",
        };
    }

    // 11. HOMOPHONES LEUR / LEURS
    if r_lower.contains("leur") {
        return FichePedagogique {
            titre: "Règle d'invariabilité de « leur » pronom vs « leurs » déterminant",
            regle_complete: "Placé devant un verbe, « leur » est un pronom personnel complément indirect (pluriel de « lui ») : il est STRICTEMENT INVARIABLE et ne prend jamais de 's'. Devant un nom au pluriel, « leur » est un déterminant possessif et s'accorde avec le nom en prenant un 's' (« leurs bureaux »).",
            astuce: "Devant un verbe, remplacez par « lui » au singulier :\n« Je lui confirme » -> « Je leur confirme » (impossible de mettre un 's' !).",
            exemple_correct: "« Je leur ai transmis leurs documents. »",
        };
    }

    // 12. ORTHOGRAPHE LEXICALE (ex: « raigler » -> « régler »)
    if r_lower.contains("orthographe") || r_lower.contains("lexicale") || r_lower.contains("dictionnaire") {
        return FichePedagogique {
            titre: "Orthographe lexicale et dictionnaire d'usage",
            regle_complete: "Cette graphie n'apparaît pas dans le lexique officiel de la langue française. Il s'agit généralement d'une coquille de frappe, d'une graphie phonétique erronée ou d'une erreur d'accentuation.",
            astuce: "Consultez les suggestions proposées : le mot correct respecte l'étymologie et les conventions de l'Académie française. Par exemple, « régler » vient du latin « regulare » et s'écrit avec un accent aigu sur le premier 'e'.",
            exemple_correct: "« Régler ce problème avec rigueur. »",
        };
    }

    // 13. DOUBLON (MOT RÉPÉTÉ CONSÉCUTIVEMENT)
    if r_lower.contains("doublon") || r_lower.contains("répété") {
        return FichePedagogique {
            titre: "Suppression du mot répété consécutivement (bégaiement typographique)",
            regle_complete: "La répétition immédiate d'un même mot (« le le », « de de », « que que ») est une inadvertance fréquente lors de la frappe ou d'un copier-coller qui alourdit inutilement la phrase.",
            astuce: "Supprimez simplement l'un des deux exemplaires pour rétablir une syntaxe fluide et professionnelle.",
            exemple_correct: "« Faire le point sur ce problème. »",
        };
    }

    // 14. PLÉONASME
    if r_lower.contains("pléonasme") {
        return FichePedagogique {
            titre: "Pléonasme : deux mots qui disent la même chose",
            regle_complete: "Un pléonasme répète une idée déjà contenue dans un autre mot (« monter en haut », « prévoir à l'avance », « mais cependant »). Il alourdit la phrase sans rien apporter ; il n'est admis que pour un effet d'insistance voulu.",
            astuce: "Cherchez le sens du verbe ou du mot principal : s'il contient déjà l'idée ajoutée (« pré- » = avant, « col- » = ensemble), supprimez l'ajout.",
            exemple_correct: "« Nous avons prévu la réunion ; puis nous sommes partis. »",
        };
    }

    // 15. STYLE : TOURNURE LOURDE OU CRITIQUÉE
    if r_lower.starts_with("style") {
        return FichePedagogique {
            titre: "Tournure lourde ou critiquée",
            regle_complete: "Cette tournure n'est pas une faute de grammaire, mais elle est jugée lourde, administrative ou critiquée par les ouvrages de référence. Une formulation plus directe rend le texte plus clair et plus élégant.",
            astuce: "Lisez la phrase à voix haute avec la suggestion : si elle dit la même chose en moins de mots, gardez-la.",
            exemple_correct: "« À la suite de votre demande, nous avons modifié le texte. »",
        };
    }

    // 16. EXPRESSION FIGÉE / LOCUTION
    if r_lower.contains("expression figée") || r_lower.contains("locution") {
        return FichePedagogique {
            titre: "Expression figée : sa forme ne se modifie pas",
            regle_complete: "Une locution figée (« prêt sur gages », « risquer gros », « sur un pied d'égalité », « prendre en compte ») a une forme fixée par l'usage : on n'en change ni les mots, ni le nombre, ni la préposition, sous peine de produire une forme fautive ou un croisement de deux expressions.",
            astuce: "En cas de doute, cherchez l'expression entière dans un dictionnaire (à l'article du nom principal) : la forme consacrée y figure.",
            exemple_correct: "« Il tenait une boutique de prêt sur gages et risquait gros. »",
        };
    }

    // 17. CONSTRUCTION VERBALE (préposition imposée par le verbe)
    if r_lower.contains("construction verbale") || r_lower.contains("préposition") {
        return FichePedagogique {
            titre: "Construction du verbe : la bonne préposition",
            regle_complete: "Chaque verbe impose sa construction : avec ou sans préposition, et laquelle. On se passionne « pour », on insiste « pour », on renchérit « sur », on joue « d' » un instrument, on conseille « à » quelqu'un « de » faire. Une préposition en double (« à aux ») est toujours fautive.",
            astuce: "Remplacez le complément par « quelque chose » : « se passionner pour quelque chose », « renchérir sur quelque chose ». La préposition qui sonne juste est celle du dictionnaire.",
            exemple_correct: "« Il se passionnait pour la poésie et renchérissait sur chaque argument. »",
        };
    }

    // 18. BARBARISME (mot qui n'existe pas)
    if r_lower.contains("barbarisme") {
        return FichePedagogique {
            titre: "Barbarisme : ce mot n'existe pas",
            regle_complete: "Un barbarisme est un mot inventé ou déformé, souvent construit par analogie avec un mot voisin (« inconvenable » sur « convenable », « indivisaire » sur « indivis »). Il faut le remplacer par le mot existant.",
            astuce: "Si un mot vous semble « logique » mais rare, vérifiez-le : la langue a souvent déjà un autre mot pour cette idée (« inconvenant », « indivis », « piégé »).",
            exemple_correct: "« Il a jugé la proposition inconvenante. »",
        };
    }

    // 19. GENRE D'UN NOM
    if r_lower == "genre" || r_lower.contains("genre du nom") {
        return FichePedagogique {
            titre: "Genre du nom",
            regle_complete: "Certains noms ont un genre trompeur : « abîme », « jade », « effluve », « tentacule », « astérisque » sont masculins ; « oasis », « octave », « espèce » sont féminins. Les fractions (« un dixième », « un vingtième ») sont masculines. Déterminants et adjectifs s'accordent avec ce genre.",
            astuce: "Retenez une phrase-témoin : « un abîme profond », « du jade précieux », « un vingtième de la surface ».",
            exemple_correct: "« Une parure de jade précieux, au bord d'un abîme. »",
        };
    }

    // 20. ACCORD À DISTANCE (sujet éloigné du verbe ou de l'attribut)
    if exp_lower.contains("doit s'accorder avec") || r_lower.contains("accord") && exp_lower.contains("sujet") {
        return FichePedagogique {
            titre: "Accord avec un sujet éloigné",
            regle_complete: "Le verbe s'accorde avec son sujet, et le participe passé employé avec « être » (ou l'adjectif attribut) s'accorde en genre et en nombre avec ce même sujet, même lorsque des compléments s'intercalent entre eux (« le centre de recherche fondé par mon père a été créé »).",
            astuce: "Supprimez mentalement tout ce qui se trouve entre le sujet et le verbe : « Le centre … a été créé », « Leur vie … était pleine ».",
            exemple_correct: "« Les enfants de ma voisine jouent dehors ; leur vie était pleine de joie. »",
        };
    }

    // 21. NOM PROPRE INCOHÉRENT
    if r_lower.contains("nom propre") {
        return FichePedagogique {
            titre: "Cohérence des noms propres",
            regle_complete: "Un même nom propre doit s'écrire de la même façon dans tout le document. Une graphie qui n'apparaît qu'une fois, à une lettre près d'une graphie fréquente, est presque toujours une faute de frappe (lettre inversée, oubliée ou ajoutée).",
            astuce: "Faites Ctrl+F sur les deux graphies : celle qui revient le plus souvent (et qui correspond aux caractères chinois) est la bonne.",
            exemple_correct: "« Changshu » partout, et non « Chansghu » une fois.",
        };
    }

    // PAR DÉFAUT : pas de fiche (l'explication de la faute suffit ; un texte passe-partout n'apprend rien)
    FichePedagogique { titre: "", regle_complete: "", astuce: "", exemple_correct: "" }
}
