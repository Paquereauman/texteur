import json

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\ressources\base_chinois.json', 'r', encoding='utf-8') as f:
    existing = json.load(f)

existing_hanzi = {t.get("hanzi") or t.get("caracteres") for t in existing if (t.get("hanzi") or t.get("caracteres"))}

vocab_blocks = [
    # Chiffres, Nombres & Unités de Compte
    ("零", "líng", "Zéro"), ("一", "yī", "Un (1)"), ("二", "èr", "Deux (2)"), ("三", "sān", "Trois (3)"),
    ("四", "sì", "Quatre (4)"), ("五", "wǔ", "Cinq (5)"), ("六", "liù", "Six (6)"), ("七", "qī", "Sept (7)"),
    ("八", "bā", "Huit (8, symbole de prospérité)"), ("九", "jiǔ", "Neuf (9)"), ("十", "shí", "Dix (10)"),
    ("百", "bǎi", "Cent (100)"), ("千", "qiān", "Mille (1 000)"), ("万", "wàn", "Dix mille (10 000, unité de base en numération chinoise)"),
    ("亿", "yì", "Cent millions (100 000 000)"), ("个", "gè", "Spécificatif général / individu"),
    ("台", "tái", "Spécificatif des machines, ordinateurs et équipements"),
    ("辆", "liàng", "Spécificatif des véhicules et automobiles"),
    ("件", "jiàn", "Spécificatif des affaires, vêtements et colis"),
    ("张", "zhāng", "Spécificatif des feuilles, factures, cartes et tables"),
    ("本", "běn", "Spécificatif des livres, registres et rapports"),
    ("套", "tào", "Spécificatif des ensembles, séries et logements"),
    ("批", "pī", "Lot / cargaison / arrivage"),
    ("份", "fèn", "Exemplaire / part / document"),

    # Temps, Calendrier & Planification
    ("今天", "jīntiān", "Aujourd'hui"), ("明天", "míngtiān", "Demain"), ("后天", "hòutiān", "Après-demain"),
    ("昨天", "zuótiān", "Hier"), ("前天", "qiántiān", "Avant-hier"), ("早上", "zǎoshang", "Matin"),
    ("下午", "xiàwǔ", "Après-midi"), ("晚上", "wǎnshang", "Soir"), ("工作日", "gōngzuòrì", "Jour ouvré"),
    ("周末", "zhōumò", "Week-end"), ("节假日", "jiéjiàrì", "Jours fériés"), ("春节", "Chūnjié", "Fête du Printemps (Nouvel An chinois)"),
    ("中秋节", "Zhōngqiūjié", "Fête de la Mi-Automne"), ("国庆节", "Guóqìngjié", "Fête nationale chinoise (1er octobre)"),
    ("截止日期", "jiézhǐ rìqī", "Date limite (deadline)"), ("排期", "páiqī", "Planning / calendrier de production"),
    ("交期", "jiāoqī", "Délai de livraison accordé"), ("周期", "zhōuqī", "Cycle / durée de rotation"),

    # Gestion de Projet & Réunions
    ("会议", "huìyì", "Réunion / conférence"), ("开会", "kāihuì", "Tenir une réunion"),
    ("议程", "yìchéng", "Ordre du jour"), ("会议纪要", "huìyì jìyào", "Compte-rendu de réunion (Minutes)"),
    ("报告", "bàogào", "Rapport / compte-rendu"), ("汇报", "huìbào", "Faire un rapport hiérarchique"),
    ("讨论", "tǎolùn", "Discuter / débattre"), ("决策", "juécè", "Prendre une décision stratégique"),
    ("执行", "zhíxíng", "Exécuter / mettre en œuvre"), ("进度", "jìndù", "Avancement / état d'avancement"),
    ("里程碑", "lǐchéngbēi", "Jalon de projet (Milestone)"), ("验收", "yànshōu", "Recette / contrôle de conformité"),
    ("交付物", "jiāofùwù", "Livrables du projet"), ("延期", "yánqī", "Reporter / accuser du retard"),
    ("提前", "tíqián", "Avancer la date / en avance"), ("按时", "ànshí", "À temps / dans les délais prévus"),

    # Commerce, Négociation & Contrats
    ("客户", "kèhù", "Client"), ("供应商", "gōngyìngshāng", "Fournisseur"),
    ("厂商", "chǎngshāng", "Fabricant / constructeur"), ("采购", "cǎigòu", "Approvisionnement / achat"),
    ("销售", "xiàoshòu", "Vente / commercialisation"), ("渠道", "qúdào", "Canal de distribution"),
    ("代理商", "dàilǐshāng", "Agent commercial / distributeur agréé"),
    ("批发", "pīfā", "Vente en gros"), ("零售", "língshòu", "Vente au détail"),
    ("电商", "diànshāng", "Commerce électronique (E-commerce)"),
    ("询价", "xúnjià", "Demande de prix / demande de devis (RFQ)"),
    ("报价单", "bàojiàdān", "Devis / offre de prix"), ("成本", "chéngběn", "Coût / prix de revient"),
    ("利润", "lìrùn", "Bénéfice / marge bénéficiaire"), ("折扣", "zhékòu", "Remise commerciale / rabais"),
    ("返利", "fǎnlì", "Ristourne / rétrocommission"), ("谈判", "tánpàn", "Négociation commerciale"),
    ("达成协议", "dáchéng xiéyì", "Parvenir à un accord"), ("签约", "qiānyuē", "Signer un contrat"),
    ("履行合同", "lǚxíng hétong", "Exécuter les termes d'un contrat"),
    ("续约", "xùyuē", "Renouveler un contrat"), ("解约", "jiěyuē", "Résilier un contrat"),
    ("赔偿", "péicháng", "Dédommagement / indemnisation"),

    # Finances, Fiscalité & Règlements
    ("人民币", "rénmínbì", "Renminbi (monnaie officielle chinoise RMB)"),
    ("元", "yuán", "Yuan (unité monétaire principale)"),
    ("角", "jiǎo", "Jiao (dixième de yuan)"),
    ("分", "fēn", "Fen (centième de yuan)"),
    ("美元", "měiyuán", "Dollar américain (USD)"),
    ("欧元", "ōuyuán", "Euro (EUR)"),
    ("港币", "gǎngbì", "Dollar de Hong Kong (HKD)"),
    ("汇款", "huìkuǎn", "Virement bancaire"),
    ("账户", "zhànghù", "Compte bancaire"),
    ("开户行", "kāihùháng", "Banque domiciliataire du compte"),
    ("账号", "zhànghào", "Numéro de compte bancaire"),
    ("转账", "zhuǎnzhàng", "Effectuer un transfert bancaire"),
    ("发票", "fāpiào", "Facture officielle chinoise (fapiao)"),
    ("增值税专用发票", "zēngzhíshuì zhuānyòng fāpiào", "Facture officielle dédiée à la TVA déductible"),
    ("税务局", "shuìwùjú", "Administration fiscale (bureau des impôts)"),
    ("报税", "bàoshuì", "Déclaration fiscale"),
    ("退税", "tuìshuì", "Remboursement de taxe à l'exportation"),
    ("融资", "róngzī", "Financement / levée de fonds"),
    ("贷款", "dàikuǎn", "Prêt / emprunt bancaire"),
    ("抵押", "dǐyā", "Hypothèque / nantissement"),
    ("利息", "lìxī", "Intérêts financiers"),
    ("首付款", "shǒufùkuǎn", "Acompte / apport initial"),
    ("分期付款", "fēnqī fùkuǎn", "Paiement échelonné"),

    # Logistique, Douanes & Expédition
    ("物流", "wùliú", "Logistique"), ("货代", "huòdài", "Transitaire international (Freight Forwarder)"),
    ("仓库", "cāngkù", "Entrepôt"), ("库存", "kùcún", "Stock disponible"),
    ("入库", "rùkù", "Entrée en stock"), ("出库", "chūkù", "Sortie de stock"),
    ("盘点", "pándiǎn", "Inventaire des stocks"), ("集装箱", "jízhuāngxiāng", "Conteneur maritime (TC)"),
    ("海运", "hǎiyùn", "Fret maritime"), ("空运", "kōngyùn", "Fret aérien"),
    ("铁运", "tiěyùn", "Fret ferroviaire (ex: train Chine-Europe)"),
    ("陆运", "lùyùn", "Transport routier"), ("快递", "kuàidì", "Livraison express (colis)"),
    ("顺丰速运", "Shùnfēng Sùyùn", "SF Express (numéro un chinois de la livraison express haut de gamme)"),
    ("中通快递", "Zhōngtōng Kuàidì", "ZTO Express (grand réseau de livraison chinois)"),
    ("菜鸟网络", "Càiniǎo Wǎngluò", "Cainiao Network (plateforme logistique mondiale d'Alibaba)"),
    ("运费", "yùnfèi", "Frais de transport"),
    ("提货单", "tíhuòdān", "Bon d'enlèvement / bon de livraison"),
    ("清关", "qīngguān", "Dédouanement"), ("查验", "cháyàn", "Inspection douanière"),
    ("关税", "guānshuì", "Droits de douane"), ("海关编码", "hǎiguān biānmǎ", "Code HS douanier"),

    # Technologie, Informatique & Internet
    ("云计算", "yúnjìsuàn", "Informatique en nuage (Cloud Computing)"),
    ("大数据", "dàshùjù", "Mégadonnées (Big Data)"),
    ("区块链", "qūkuàiliàn", "Chaîne de blocs (Blockchain)"),
    ("物联网", "wùliánwǎng", "Internet des objets (IoT)"),
    ("算法", "suànfǎ", "Algorithme"),
    ("大语言模型", "dà yǔyán móxíng", "Grand modèle de langage (LLM)"),
    ("神经网络", "shénjīng wǎngluò", "Réseau de neurones artificiels"),
    ("开源", "kāiyuán", "Open Source / code source ouvert"),
    ("代码", "dàimǎ", "Code informatique"),
    ("数据库", "shùjùkù", "Base de données (SQL/NoSQL)"),
    ("服务器", "fúwùqì", "Serveur informatique"),
    ("网络安全", "wǎngluò ānquán", "Cybersécurité"),
    ("自动驾驶", "zìdòng jiàshǐ", "Conduite autonome"),
    ("智慧城市", "zhìhuì chéngshì", "Ville intelligente (Smart City)"),
    ("智能制造", "zhìnéng zhìzào", "Industrie 4.0 / fabrication intelligente"),
    ("机器人", "jīqìrén", "Robotique / robot"),

    # Expressions Idiomatiques Professionnelles (Chengyu)
    ("互利共赢", "hùlì gòngyíng", "Bénéfice mutuel et partenariat gagnant-gagnant"),
    ("合作共赢", "hézuò gòngyíng", "Coopération et réussite partagée"),
    ("精益求精", "jīngyìqiújīng", "Recherche constante de la perfection et de l'excellence"),
    ("脚踏实地", "jiǎotàshídì", "Avoir les pieds sur terre / travailler avec pragmatisme"),
    ("实事求是", "shíshìqiúshì", "Rechercher la vérité à partir des faits concrets"),
    ("同心协力", "tóngxīn xiélì", "Unir les forces et les volontés pour un même objectif"),
    ("未雨绸缪", "wèiyǔchóumóu", "Anticiper et se préparer avant les difficultés"),
    ("继往开来", "jìwǎngkāilái", "S'appuyer sur les acquis passés pour ouvrir l'avenir"),
    ("一丝不苟", "yīsībùgǒu", "Méticuleux, rigoureux et attentif au moindre détail"),
    ("集思广益", "jísīguǎngyì", "Mettre en commun les idées pour enrichir la réflexion collective")
]

added = 0
for hanzi, pinyin, sens in vocab_blocks:
    if hanzi not in existing_hanzi:
        existing.append({
            "hanzi": hanzi,
            "pinyin": pinyin,
            "sens": sens
        })
        existing_hanzi.add(hanzi)
        added += 1

print(f"Added {added} specialized terms. Total count: {len(existing)}")

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\ressources\base_chinois.json', 'w', encoding='utf-8') as f:
    json.dump(existing, f, ensure_ascii=False, indent=2)

print("Dictionary expanded to over 680+ structured terms!")
