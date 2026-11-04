# -*- coding: utf-8 -*-
import json
import os

termes = []

def add(hanzi, pinyin, sens, trad=""):
    termes.append({
        "hanzi": hanzi,
        "trad": trad if trad else "",
        "pinyin": pinyin,
        "sens": sens
    })

# ==========================================
# 1. VILLES, MÉTROPOLES ET RÉGIONS URBAINES
# ==========================================
add("北京", "Běijīng", "Pékin (capitale de la Chine)")
add("上海", "Shànghǎi", "Shanghai (centre économique et financier)")
add("深圳", "Shēnzhèn", "Shenzhen (métropole technologique du Guangdong)")
add("广州", "Guǎngzhōu", "Canton / Guangzhou (capitale du Guangdong)", "廣州")
add("天津", "Tiānjīn", "Tianjin (municipalité autonome du nord)")
add("重庆", "Chóngqìng", "Chongqing (mégalopole du sud-ouest)", "重慶")
add("成都", "Chéngdū", "Chengdu (capitale du Sichuan)")
add("杭州", "Hángzhōu", "Hangzhou (capitale du Zhejiang, siège d'Alibaba)")
add("武汉", "Wǔhàn", "Wuhan (carrefour industriel du Hubei)", "武漢")
add("西安", "Xī'ān", "Xi'an (capitale historique du Shaanxi, Guerriers de terre cuite)")
add("南京", "Nánjīng", "Nankin / Nanjing (capitale du Jiangsu)")
add("苏州", "Sūzhōu", "Suzhou (centre manufacturier et parcs technologiques du Jiangsu)", "蘇州")
add("东莞", "Dōngguǎn", "Dongguan (pôle manufacturier mondial du Guangdong)", "東莞")
add("佛山", "Fóshān", "Foshan (pôle industriel du Guangdong)")
add("宁波", "Níngbō", "Ningbo (grand port maritime du Zhejiang)", "寧波")
add("无锡", "Wúxī", "Wuxi (centre technologique du Jiangsu)", "無錫")
add("青岛", "Qīngdǎo", "Qingdao (grand port et pôle industriel du Shandong)", "青島")
add("长沙", "Chángshā", "Changsha (capitale du Hunan)", "長沙")
add("郑州", "Zhèngzhōu", "Zhengzhou (carrefour logistique et industriel du Henan)", "鄭州")
add("合肥", "Héféi", "Hefei (pôle d'innovation et batteries de l'Anhui)")
add("沈阳", "Shěnyáng", "Shenyang (capitale industrielle du Liaoning)", "瀋陽")
add("大连", "Dàlián", "Dalian (grand port du Liaoning)", "大連")
add("济南", "Jǐnán", "Jinan (capitale du Shandong)", "濟南")
add("厦门", "Xiàmén", "Xiamen (zone économique spéciale du Fujian)", "廈門")
add("福州", "Fúzhōu", "Fuzhou (capitale du Fujian)")
add("昆明", "Kūnmíng", "Kunming (capitale du Yunnan, ville du printemps)")
add("哈尔滨", "Hā'ěrbīn", "Harbin (capitale du Heilongjiang, festival de glace)", "哈爾濱")
add("长春", "Chángchūn", "Changchun (capitale automobile du Jilin)", "長春")
add("南昌", "Nánchāng", "Nanchang (capitale du Jiangxi)")
add("贵阳", "Guìyáng", "Guiyang (centre national de mégadonnées du Guizhou)", "貴陽")
add("南宁", "Nánníng", "Nanning (capitale du Guangxi, porte de l'ASEAN)", "南寧")
add("太原", "Tàiyuán", "Taiyuan (capitale du Shanxi)")
add("石家庄", "Shíjiāzhuāng", "Shijiazhuang (capitale du Hebei)", "石家莊")
add("兰州", "Lánzhōu", "Lanzhou (capitale du Gansu, carrefour de la Route de la Soie)", "蘭州")
add("乌鲁木齐", "Wūlǔmùqí", "Urumqi (capitale de la région du Xinjiang)", "烏魯木齊")
add("呼和浩特", "Hūhéhàotè", "Hohhot (capitale de la Mongolie-Intérieure)")
add("银川", "Yínchuān", "Yinchuan (capitale du Ningxia)")
add("西宁", "Xīníng", "Xining (capitale du Qinghai)", "西寧")
add("拉萨", "Lāsà", "Lhassa (capitale de la région autonome du Tibet)")
add("海口", "Hǎikǒu", "Haikou (capitale de Hainan, zone franche portuaire)")
add("三亚", "Sānyà", "Sanya (station balnéaire de Hainan)", "三亞")
add("香港", "Xiānggǎng", "Hong Kong (région administrative spéciale)")
add("澳门", "Àomén", "Macao (région administrative spéciale)", "澳門")
add("台北", "Táiběi", "Taipei (capitale de Taïwan)", "臺北")
add("高雄", "Gāoxióng", "Kaohsiung (grand port du sud de Taïwan)")
add("新竹", "Xīnzhú", "Hsinchu (parc scientifique et pôle semi-conducteurs de Taïwan)")
add("珠海", "Zhūhǎi", "Zhuhai (zone économique spéciale voisine de Macao)")
add("中山", "Zhōngshān", "Zhongshan (ville manufacturière du Guangdong)")
add("惠州", "Huìzhōu", "Huizhou (pôle électronique du Guangdong)")
add("汕头", "Shàntóu", "Shantou (zone économique spéciale du Guangdong)", "汕頭")
add("义乌", "Yìwū", "Yiwu (capitale mondiale des petites marchandises, Zhejiang)", "義烏")
add("温州", "Wēnzhōu", "Wenzhou (berceau de l'entrepreneuriat privé du Zhejiang)")
add("绍兴", "Shàoxīng", "Shaoxing (pôle textile du Zhejiang)", "紹興")
add("嘉兴", "Jiāxīng", "Jiaxing (pôle industriel du delta du Yangtsé)", "嘉興")
add("金华", "Jīnhuá", "Jinhua (pôle commercial et logistique du Zhejiang)", "金華")
add("常州", "Chángzhōu", "Changzhou (pôle véhicules électriques et batteries du Jiangsu)")
add("南通", "Nántōng", "Nantong (port maritime et construction navale du Jiangsu)")
add("徐州", "Xúzhōu", "Xuzhou (pôle de machinerie lourde du Jiangsu)")
add("烟台", "Yāntái", "Yantai (port et industrie chimique du Shandong)", "煙臺")
add("潍坊", "Wéifāng", "Weifang (pôle agricole et mécanique du Shandong)", "濰坊")
add("威海", "Wēihǎi", "Weihai (port maritime du Shandong)")
add("泉州", "Quánzhōu", "Quanzhou (départ de la Route maritime de la Soie, Fujian)")
add("洛阳", "Luòyáng", "Luoyang (capitale impériale historique du Henan)", "洛陽")
add("桂林", "Guìlín", "Guilin (paysages karstiques renommés du Guangxi)")
add("大理", "Dàlǐ", "Dali (ville historique et touristique du Yunnan)")
add("丽江", "Lìjiāng", "Lijiang (cité ancienne UNESCO du Yunnan)", "麗江")

# ==========================================
# 2. PROVINCES ET RÉGIONS ADMINISTRATIVES
# ==========================================
add("广东", "Guǎngdōng", "Guangdong (province manufacturière leader)", "廣東")
add("浙江", "Zhèjiāng", "Zhejiang (province d'innovation et d'e-commerce)")
add("江苏", "Jiāngsū", "Jiangsu (province industrielle majeure)", "江蘇")
add("山东", "Shāndōng", "Shandong (province côtière et agricole)", "山東")
add("四川", "Sìchuān", "Sichuan (province dynamique du sud-ouest)")
add("湖北", "Húběi", "Hubei (province centrale, pôle automobile et optique)")
add("湖南", "Húnán", "Hunan (province centrale, machines lourdes et médias)")
add("河南", "Hénán", "Henan (province centrale très peuplée)")
add("河北", "Héběi", "Hebei (province entourant Pékin)")
add("福建", "Fújiàn", "Fujian (province côtière face à Taïwan)")
add("安徽", "Ānhuī", "Anhui (pôle émergent de hautes technologies)")
add("江西", "Jiāngxī", "Jiangxi (province centrale, céramique et métaux rares)")
add("陕西", "Shǎnxī", "Shaanxi (province historique, aéronautique et énergie)", "陝西")
add("山西", "Shānxī", "Shanxi (province du charbon et de l'énergie)")
add("辽宁", "Liáoníng", "Liaoning (province du nord-est, industrie lourde)", "遼寧")
add("吉林", "Jílín", "Jilin (province du nord-est, automobile et chimie)")
add("黑龙江", "Hēilóngjiāng", "Heilongjiang (province septentrionale)", "黑龍江")
add("云南", "Yúnnán", "Yunnan (province frontalière du sud-ouest)", "雲南")
add("贵州", "Guìzhōu", "Guizhou (province du sud-ouest, mégadonnées et alcool)", "貴州")
add("海南", "Hǎinán", "Hainan (province insulaire tropicale et port franc)")
add("甘肃", "Gānsù", "Gansu (province du couloir du Hexi)", "甘肅")
add("青海", "Qīnghǎi", "Qinghai (province du plateau tibétain)")
add("新疆", "Xīnjiāng", "Xinjiang (région autonome ouïghoure)")
add("西藏", "Xīzàng", "Tibet / Xizang (région autonome du plateau tibétain)")
add("内蒙古", "Nèi Měnggǔ", "Mongolie-Intérieure (région autonome)")
add("广西", "Guǎngxī", "Guangxi (région autonome zhuang du sud)", "廣西")
add("宁夏", "Níngxià", "Ningxia (région autonome hui)", "寧夏")
add("台湾", "Táiwān", "Taïwan (île et province insulaire)", "臺灣")

# ==========================================
# 3. GÉOGRAPHIE, FLEUVES, MONTAGNES, SITES
# ==========================================
add("长城", "Chángchéng", "Grande Muraille de Chine", "長城")
add("故宫", "Gùgōng", "Cité interdite (Palais impérial de Pékin)", "故宮")
add("天安门", "Tiān'ānmén", "Place Tiananmen (Pékin)", "天安門")
add("天坛", "Tiāntán", "Temple du Ciel (Pékin)", "天壇")
add("颐和园", "Yíhéyuán", "Palais d'Été (Pékin)", "頤和園")
add("兵马俑", "Bīngmǎyǒng", "Guerriers et chevaux de terre cuite (Xi'an)", "兵馬俑")
add("长江", "Chángjiāng", "Fleuve Yangzi Jiang / Fleuve Bleu", "長江")
add("黄河", "Huánghé", "Fleuve Jaune (berceau de la civilisation chinoise)", "黃河")
add("珠江", "Zhūjiāng", "Rivière des Perles (delta du Guangdong)")
add("淮河", "Huáihé", "Rivière Huai (ligne de démarcation Nord/Sud)")
add("泰山", "Tàishān", "Mont Tai (montagne sacrée du Shandong)")
add("黄山", "Huángshān", "Monts Huangshan (montagnes Jaunes de l'Anhui)", "黃山")
add("华山", "Huàshān", "Mont Hua (montagne sacrée du Shaanxi)", "華山")
add("恒山", "Héngshān", "Mont Heng du Nord (Shanxi)")
add("衡山", "Héngshān", "Mont Heng du Sud (Hunan)")
add("嵩山", "Sōngshān", "Mont Song (Henan, temple Shaolin)")
add("峨眉山", "Éméishān", "Mont Emei (Sichuan, montagne sacrée bouddhique)")
add("五台山", "Wǔtáishān", "Mont Wutai (Shanxi, montagne sacrée bouddhique)", "五臺山")
add("九华山", "Jiǔhuáshān", "Mont Jiuhua (Anhui, montagne sacrée bouddhique)", "九華山")
add("普陀山", "Pǔtuóshān", "Mont Putuo (Zhejiang, montagne sacrée de Guanyin)")
add("珠穆朗玛峰", "Zhūmùlǎngmǎ Fēng", "Mont Everest / Qomolangma", "珠穆朗瑪峰")
add("青藏高原", "Qīng-Zàng Gāoyuán", "Plateau tibétain (toit du monde)")
add("三峡大坝", "Sānxiá Dàbà", "Barrage des Trois-Gorges (Hubei)", "三峽大壩")
add("东方明珠", "Dōngfāng Míngzhū", "Tour Perle de l'Orient (Shanghai)", "東方明珠")
add("莫高窟", "Mògāo Kū", "Grottes de Mogao (Dunhuang, Gansu)")
add("布达拉宫", "Bùdálǎ Gōng", "Palais du Potala (Lhassa, Tibet)", "布達拉宮")
add("西湖", "Xī Hú", "Lac de l'Ouest (Hangzhou)")

# ==========================================
# 4. ENTREPRISES, MARQUES, GÉANTS TECH ET INDUSTRIE
# ==========================================
add("华为", "Huáwèi", "Huawei Technologies (télécoms, smartphones, cloud)", "華為")
add("腾讯", "Téngxùn", "Tencent Holdings (WeChat, jeux vidéo, fintech)", "騰訊")
add("阿里巴巴", "Ālǐbābā", "Alibaba Group (e-commerce, cloud, logistique)")
add("字节跳动", "Zìjié Tiàodòng", "ByteDance (TikTok, Douyin)", "字節跳動")
add("小米", "Xiǎomǐ", "Xiaomi Corporation (smartphones, IoT, véhicules électriques)")
add("百度", "Bǎidù", "Baidu (moteur de recherche, IA, conduite autonome)")
add("美团", "Měituán", "Meituan (livraison, services locaux, e-commerce)", "美團")
add("京东", "Jīngdōng", "JD.com (e-commerce, logistique intégrée)", "京東")
add("拼多多", "Pīnduōduō", "Pinduoduo / PDD Holdings (e-commerce, Temu)")
add("网易", "WǎngYì", "NetEase (jeux en ligne, services internet)", "網易")
add("快手", "Kuàishǒu", "Kuaishou (vidéos courtes et e-commerce)")
add("哔哩哔哩", "Bīlībīlī", "Bilibili (plateforme vidéo et animation)")
add("大疆", "Dàjiāng", "DJI Innovations (drones civils et stabilisateurs)")
add("联想", "Liánxiǎng", "Lenovo Group (ordinateurs personnels, serveurs)", "聯想")
add("中芯国际", "Zhōngxīn Guójì", "SMIC (fonderie de semi-conducteurs chinoise)", "中芯國際")
add("台积电", "Tái-Jī-Diàn", "TSMC (leader mondial des semi-conducteurs)", "台積電")
add("京东方", "Jīngdōngfāng", "BOE Technology (écrans OLED et LCD)", "京東方")
add("比亚迪", "Bǐyàdí", "BYD Company (véhicules électriques et batteries)", "比亞迪")
add("宁德时代", "Níngdé Shídài", "CATL (leader mondial des batteries pour VE)", "寧德時代")
add("吉利", "Jílì", "Geely Auto (groupe automobile, Volvo, Zeekr)")
add("蔚来", "Wèilái", "NIO (constructeur de véhicules électriques haut de gamme)", "蔚來")
add("小鹏", "Xiǎopéng", "XPeng Motors (véhicules électriques intelligents)", "小鵬")
add("理想汽车", "Lǐxiǎng Qìchē", "Li Auto (véhicules hybrides et électriques)", "理想汽車")
add("长城汽车", "Chángchéng Qìchē", "Great Wall Motor / GWM (SUV et pick-up)", "長城汽車")
add("奇瑞", "Qíruì", "Chery Automobile (constructeur et exportateur automobile)")
add("上汽集团", "Shàngqì Jítuán", "SAIC Motor (premier groupe automobile chinois)", "上汽集團")
add("海尔", "Hǎi'ěr", "Haier Smart Home (électroménager mondial)", "海爾")
add("美的", "Měidì", "Midea Group (électroménager, robotique KUKA)")
add("格力", "Gèlì", "Gree Electric (climatiseurs et équipement ménager)")
add("富士康", "Fùshìkāng", "Foxconn / Hon Hai Precision (assemblage électronique)")
add("中国移动", "Zhōngguó Yídòng", "China Mobile (premier opérateur télécom mondial)", "中國移動")
add("中国电信", "Zhōngguó Diànxìn", "China Telecom (opérateur de télécommunications)", "中國電信")
add("中国联通", "Zhōngguó Liántōng", "China Unicom (opérateur télécom)", "中國聯通")
add("工商银行", "Gōngshāng Yínháng", "ICBC (Banque industrielle et commerciale de Chine)", "工商銀行")
add("建设银行", "Jiànshè Yínháng", "China Construction Bank / CCB", "建設銀行")
add("农业银行", "Nóngyè Yínháng", "Agricultural Bank of China / ABC", "農業銀行")
add("中国银行", "Zhōngguó Yínháng", "Bank of China / BOC", "中國銀行")
add("招商银行", "Zhāoshāng Yínháng", "China Merchants Bank / CMB", "招商銀行")
add("中国平安", "Zhōngguó Píng'ān", "Ping An Insurance (groupe d'assurance et fintech)", "中國平安")
add("中石化", "Zhōngshíhuà", "Sinopec (groupe pétrochimique d'État)")
add("中石油", "Zhōngshíyóu", "PetroChina / CNPC (compagnie pétrolière nationale)")
add("国家电网", "Guójiā Diànwǎng", "State Grid Corporation of China (réseau électrique)", "國家電網")
add("中国中车", "Zhōngguó Zhōngchē", "CRRC (constructeur ferroviaire et TGV mondial)", "中國中車")
add("中远海运", "Zhōngyuǎn Hǎiyùn", "COSCO Shipping (armateur et transport de conteneurs)", "中遠海運")
add("贵州茅台", "Guìzhōu Máotái", "Kweichow Moutai (spiritueux haut de gamme baijiu)", "貴州茅臺")
add("五粮液", "Wǔliángyè", "Wuliangye (célèbre distillerie de baijiu)", "五糧液")
add("青岛啤酒", "Qīngdǎo Píjiǔ", "Bière Tsingtao (brasserie historique de Qingdao)", "青島啤酒")
add("安踏", "Āntà", "Anta Sports (équipements sportifs, Fila, Amer Sports)")
add("李宁", "Lǐ Níng", "Li-Ning (marque d'articles de sport)", "李寧")
add("希音", "Xīyīn", "Shein (géant mondial de la mode en ligne)")
add("顺丰速运", "Shùnfēng Sùyùn", "SF Express (leader de la livraison express en Chine)", "順豐速運")
add("菜鸟网络", "Càiniǎo Wǎngluò", "Cainiao Network (plateforme logistique mondiale d'Alibaba)", "菜鳥網絡")
add("微信", "Wēixìn", "WeChat (messagerie et écosystème mobile leader)")
add("支付宝", "Zhīfùbǎo", "Alipay (plateforme de paiement mobile leader)", "支付寶")
add("淘宝", "Táobǎo", "Taobao (plateforme d'e-commerce C2C leader)", "淘寶")
add("天猫", "Tiānmāo", "Tmall (plateforme d'e-commerce B2C pour grandes marques)", "天貓")
add("小红书", "Xiǎohóngshū", "Xiaohongshu / RED (réseau social et lifestyle)")
add("抖音", "Dǒuyīn", "Douyin (version chinoise de TikTok)")

# ==========================================
# 5. PERSONNALITÉS, HISTOIRE, PHILOSOPHES
# ==========================================
add("孔子", "Kǒngzǐ", "Confucius (philosophe et éducateur fondateur de la pensée chinoise)")
add("老子", "Lǎozǐ", "Laozi / Lao-tseu (fondateur du taoïsme, auteur du Daodejing)")
add("孙子", "Sūnzǐ", "Sunzi / Sun Tzu (stratège militaire, auteur de L'Art de la guerre)", "孫子")
add("孟子", "Mèngzǐ", "Mencius (philosophe confucéen)")
add("庄子", "Zhuāngzǐ", "Zhuangzi (grand penseur et maître taoïste)", "莊子")
add("司马迁", "Sīmǎ Qiān", "Sima Qian (père de l'historiographie chinoise, Shiji)", "司馬遷")
add("秦始皇", "Qín Shǐhuáng", "Qin Shi Huang (premier empereur de Chine unifiée, dynastie Qin)")
add("汉武帝", "Hàn Wǔdì", "Empereur Wu des Han (ouverture de la Route de la Soie)", "漢武帝")
add("唐太宗", "Táng Tàizōng", "Empereur Taizong des Tang (âge d'or de la dynastie Tang)")
add("武则天", "Wǔ Zétiān", "Wu Zetian (unique impératrice régnante de l'histoire chinoise)", "武則天")
add("康熙", "Kāngxī", "Empereur Kangxi (dynastie Qing, plus long règne impérial)")
add("乾隆", "Qiánlóng", "Empereur Qianlong (dynastie Qing, expansion de l'empire)")
add("孙中山", "Sūn Zhōngshān", "Sun Yat-sen (père de la République de Chine)", "孫中山")
add("毛泽东", "Máo Zédōng", "Mao Zedong (fondateur de la République populaire de Chine)", "毛澤東")
add("周恩来", "Zhōu Ēnlái", "Zhou Enlai (premier Premier ministre de la RPC, diplomate)", "周恩來")
add("邓小平", "Dèng Xiǎopíng", "Deng Xiaoping (artisan des réformes et de l'ouverture économique)", "鄧小平")
add("习近平", "Xí Jìnpíng", "Xi Jinping (président de la République populaire de Chine)", "習近平")
add("李强", "Lǐ Qiáng", "Li Qiang (Premier ministre de la RPC)", "李強")
add("李白", "Lǐ Bái", "Li Bai / Li Po (illustre poète romantique de la dynastie Tang)")
add("杜甫", "Dù Fǔ", "Du Fu (grand poète réaliste de la dynastie Tang)")
add("苏轼", "Sū Shì", "Su Dongpo / Su Shi (maître de la littérature et de la poésie Song)", "蘇軾")
add("鲁迅", "Lǔ Xùn", "Lu Xun (père de la littérature chinoise moderne)", "魯迅")
add("莫言", "Mò Yán", "Mo Yan (écrivain, prix Nobel de littérature 2012)")
add("刘慈欣", "Liú Cíxīn", "Liu Cixin (auteur de science-fiction, Le Problème à trois corps)", "劉慈欣")
add("马云", "Mǎ Yún", "Jack Ma (fondateur d'Alibaba Group)", "馬雲")
add("马化腾", "Mǎ Huàténg", "Pony Ma (fondateur et président de Tencent)", "馬化騰")
add("任正非", "Rén Zhèngfēi", "Ren Zhengfei (fondateur et PDG de Huawei)")
add("雷军", "Léi Jūn", "Lei Jun (fondateur et PDG de Xiaomi)", "雷軍")
add("王传福", "Wáng Chuánfú", "Wang Chuanfu (fondateur et président de BYD)", "王傳福")
add("张一鸣", "Zhāng Yīmíng", "Zhang Yiming (fondateur de ByteDance / TikTok)", "張一鳴")

# ==========================================
# 6. COMMERCE, AFFAIRES, INDUSTRIE ET CONTRATS
# ==========================================
add("合同", "hétong", "contrat officiel")
add("协议", "xiéyì", "accord, convention, protocole", "協議")
add("采购", "cǎigòu", "achats, approvisionnement", "採購")
add("订单", "dìngdān", "bon de commande, commande", "訂單")
add("报价单", "bàojiàdān", "devis, offre de prix, cotation", "報價單")
add("发票", "fāpiào", "facture officielle fiscale chinoise", "發票")
add("形式发票", "xíngshì fāpiào", "facture pro forma")
add("装箱单", "zhuāngxiāngdān", "liste de colisage (packing list)", "裝箱單")
add("提单", "tídān", "connaissement maritime (Bill of Lading / B/L)", "提單")
add("原产地证书", "yuánchǎndì zhèngshū", "certificat d'origine", "原產地證書")
add("信用证", "xìnyòngzhèng", "lettre de crédit bancaire (L/C)", "信用證")
add("电汇", "diànhuì", "virement télégraphique / bancaire (T/T)", "電匯")
add("定金", "dìngjīn", "acompte, arrhes de réservation")
add("尾款", "wěikuǎn", "solde final de paiement")
add("单价", "dānjià", "prix unitaire", "單價")
add("总额", "zǒng'é", "montant total, somme globale", "總額")
add("含税", "hánshuì", "toutes taxes comprises (TTC)")
add("不含税", "bù hánshuì", "hors taxes (HT)")
add("汇率", "huìlǜ", "taux de change monétaire", "匯率")
add("人民币", "rénmínbì", "monnaie chinoise (RMB / Yuan)", "人民幣")
add("美元", "měiyuán", "dollar américain (USD)")
add("欧元", "ōuyuán", "euro (EUR)", "歐元")
add("样品", "yàngpǐn", "échantillon de produit", "樣品")
add("打样", "dǎyàng", "prototypage, réalisation d'échantillons", "打樣")
add("模具", "mújù", "moule industriel, matrice")
add("生产线", "shēngchǎnxiàn", "ligne de production / d'assemblage", "生產線")
add("工厂", "gōngchǎng", "usine, manufacture", "工廠")
add("交期", "jiāoqī", "délai de livraison, date d'échéance")
add("交货期", "jiāohuòqī", "délai de livraison de la marchandise", "交貨期")
add("起订量", "qǐdìngliàng", "quantité minimale de commande (MOQ)", "起訂量")
add("规格", "guīgé", "spécifications techniques, gabarit", "規格")
add("原材料", "yuáncáiliào", "matières premières")
add("质量控制", "zhìliàng kòngzhì", "contrôle qualité (QC)", "質量控制")
add("品质", "pǐnzhì", "qualité d'un produit ou d'un matériau", "品質")
add("不良率", "bùliánglǜ", "taux de rebut, taux de défectuosité")
add("检验报告", "jiǎnyàn bàogào", "rapport d'inspection / de test", "檢驗報告")
add("出货检验", "chūhuò jiǎnyàn", "inspection avant expédition", "出貨檢驗")
add("海运", "hǎiyùn", "transport maritime, fret maritime", "海運")
add("空运", "kōngyùn", "transport aérien, fret aérien", "空運")
add("清关", "qīngguān", "dédouanement à l'arrivée", "清關")
add("报关", "bàoguān", "déclaration en douane", "報關")
add("关税", "guānshuì", "droits de douane, tarif douanier", "關稅")
add("集装箱", "jízhuāngxiāng", "conteneur maritime", "集裝箱")
add("整箱", "zhěngxiāng", "conteneur complet (FCL)")
add("拼箱", "pīnxiāng", "groupage de conteneur (LCL)")
add("仓储", "cāngchǔ", "entreposage, stockage", "倉儲")
add("供应链", "gōngyìngliàn", "chaîne d'approvisionnement (supply chain)", "供應鏈")
add("离岸价", "lí'ànjià", "prix FOB (franco à bord)", "離岸價")
add("到岸价", "dào'ànjià", "prix CIF (coût, assurance et fret)", "到岸價")
add("出厂价", "chūchǎngjià", "prix départ usine (EXW)", "出廠價")
add("商标", "shāngbiāo", "marque commerciale déposée", "商標")
add("专利", "zhuānlì", "brevet d'invention ou modèle", "專利")
add("版权", "bǎnquán", "droit d'auteur, copyright", "版權")
add("保密协议", "bǎomì xiéyì", "accord de non-divulgation / confidentialité (NDA)", "保密協議")
add("违约金", "wéiyuējīn", "indemnité de rupture / pénalité contractuelle", "違約金")
add("不可抗力", "bùkěkànglì", "force majeure (clause contractuelle)")
add("仲裁", "zhòngcái", "arbitrage commercial")
add("客户", "kèhù", "client, compte commercial", "客戶")
add("供应商", "gōngyìngshāng", "fournisseur officiel", "供應商")
add("合作伙伴", "hézuò huǒbàn", "partenaire commercial ou stratégique", "合作夥伴")
add("代理商", "dàilǐshāng", "agent commercial, distributeur agréé")
add("董事长", "dǒngshìzhǎng", "président du conseil d'administration", "董事長")
add("总经理", "zǒngjīnglǐ", "directeur général (DG / CEO)", "總經理")
add("经理", "jīnglǐ", "directeur, manager, responsable", "經理")
add("工程师", "gōngchéngshī", "ingénieur qualifié", "工程師")
add("车间主任", "chējiān zhǔrèn", "chef d'atelier de fabrication", "車間主任")
add("谈判", "tánpàn", "négociation commerciale", "談判")
add("让步", "ràngbù", "concession, compromis", "讓步")
add("折扣", "zhékòu", "remise commerciale, rabais")
add("退款", "tuìkuǎn", "remboursement, avoir financier")
add("保修期", "bǎoxiūqī", "période de garantie contractuelle")
add("售后服务", "shòuhòu fúwù", "service après-vente (SAV)", "售後服務")

# ==========================================
# 7. CULTURE, RELATIONS, VIE QUOTIDIENNE & HSK
# ==========================================
add("关系", "guānxi", "réseau de relations interpersonnelles", "關係")
add("面子", "miànzi", "prestige personnel, réputation, 'la face'")
add("客气", "kèqi", "poli, courtois, cérémonieux", "客氣")
add("你好", "nǐ hǎo", "bonjour, salut")
add("谢谢", "xièxie", "merci beaucoup", "謝謝")
add("不客气", "bú kèqi", "de rien, je vous en prie", "不客氣")
add("再见", "zàijiàn", "au revoir", "再見")
add("对不起", "duìbuqǐ", "pardon, désolé, excusez-moi", "對不起")
add("没关系", "méi guānxi", "ce n'est rien, sans importance", "沒關係")
add("请问", "qǐngwèn", "puis-je vous demander, s'il vous plaît", "請問")
add("没问题", "méi wèntí", "pas de problème, aucun souci", "沒問題")
add("加油", "jiāyóu", "courage !, allez !, persévérez !")
add("干杯", "gānbēi", "cul sec !, santé ! (toast)", "乾杯")
add("恭喜", "gōngxǐ", "félicitations, tous mes vœux")
add("春节", "Chūnjié", "Fête du printemps (Nouvel An chinois)", "春節")
add("中秋节", "Zhōngqiūjié", "Fête de la mi-automne", "中秋節")
add("端午节", "Duānwǔjié", "Fête des bateaux-dragons", "端午節")
add("国庆节", "Guóqìngjié", "Fête nationale chinoise (1er octobre)", "國慶節")
add("清明节", "Qīngmíngjié", "Fête de Qingming (hommage aux ancêtres)", "清明節")
add("红包", "hóngbāo", "enveloppe rouge avec étrennes monétaires", "紅包")
add("功夫", "gōngfu", "kung-fu, art martial traditionnel")
add("太极拳", "tàijíquán", "tai-chi-chuan, gymnastique énergétique", "太極拳")
add("风水", "fēngshuǐ", "feng shui (art d'harmonisation spatiale)", "風水")
add("阴阳", "yīnyáng", "yin et yang (dualité fondamentale)", "陰陽")
add("道", "dào", "la Voie, le Principe suprême (taoïsme)")
add("气", "qì", "énergie vitale, souffle vital (MTC)", "氣")
add("中医", "zhōngyī", "médecine traditionnelle chinoise (MTC)", "中醫")
add("针灸", "zhēnjiǔ", "acupuncture et moxibustion", "針灸")
add("普通话", "Pǔtōnghuà", "mandarin standard national", "普通話")
add("汉字", "hànzì", "sinogramme, caractère d'écriture chinoise", "漢字")
add("拼音", "pīnyīn", "pinyin (romanisation phonétique officielle)")
add("中文", "Zhōngwén", "langue chinoise écrite et parlée")
add("汉语", "Hànyǔ", "langue chinoise des Hans", "漢語")
add("饺子", "jiǎozi", "raviolis traditionnels du nord", "餃子")
add("包子", "bāozi", "brioche farcie cuite à la vapeur")
add("火锅", "huǒguō", "fondue chinoise traditionnelle", "火鍋")
add("北京烤鸭", "Běijīng kǎoyā", "canard laqué de Pékin", "北京烤鴨")
add("麻婆豆腐", "mápó dòufu", "tofu épicé au porc du Sichuan")
add("茶", "chá", "thé chinois (vert, noir, oolong, pu'er)")
add("白酒", "báijiǔ", "eau-de-vie traditionnelle de céréales")
add("筷子", "kuàizi", "baguettes pour manger")
add("老师", "lǎoshī", "enseignant, maître, professeur", "老師")
add("老板", "lǎobǎn", "patron, dirigeant d'entreprise", "老闆")
add("师傅", "shīfu", "maître artisan, chauffeur, professionnel", "師傅")
add("朋友", "péngyou", "ami, camarade")
add("同事", "tóngshì", "collègue de travail")

# Sauvegarde dans le fichier base_chinois.json
chemin_sortie = os.path.join(os.path.dirname(__file__), "base_chinois.json")
with open(chemin_sortie, "w", encoding="utf-8") as f:
    json.dump(termes, f, ensure_ascii=False, indent=2)

print(f"Base de vocabulaire et noms propres générée : {len(termes)} termes enregistrés dans {chemin_sortie}")

# ==========================================
# 8. INFRASTRUCTURES, AÉROPORTS, TGV ET PORTS
# ==========================================
add("高铁", "gāotiě", "train à grande vitesse chinois (TGV / CRH)", "高鐵")
add("动车", "dòngchē", "train rapide interurbain", "動車")
add("磁悬浮", "cíxuánfú", "train à sustentation magnétique (Maglev)", "磁懸浮")
add("首都机场", "Shǒudū Jīchǎng", "Aéroport international de Pékin-Capitale", "首都機場")
add("大兴机场", "Dàxīng Jīchǎng", "Aéroport international de Pékin-Daxing", "大興機場")
add("浦东机场", "Pǔdōng Jīchǎng", "Aéroport international de Shanghai-Pudong", "浦東機場")
add("虹桥机场", "Hóngqiáo Jīchǎng", "Aéroport international de Shanghai-Hongqiao", "虹橋機場")
add("白云机场", "Báiyún Jīchǎng", "Aéroport international de Canton-Baiyun", "白雲機場")
add("宝安机场", "Bǎo'ān Jīchǎng", "Aéroport international de Shenzhen-Bao'an", "寶安機場")
add("洋山港", "Yángshān Gǎng", "Port en eau profonde de Yangshan (Shanghai)")
add("盐田港", "Yántián Gǎng", "Port de conteneurs de Yantian (Shenzhen)", "鹽田港")
add("港珠澳大桥", "Gǎng-Zhū-Ào Dàqiáo", "Pont maritime Hong Kong-Zhuhai-Macao", "港珠澳大橋")
add("南水北调", "Nánshuǐ Běidiào", "Grand projet de dérivation des eaux Sud-Nord", "南水北調")
add("西气东输", "Xīqì Dōngshū", "Gazoducs Ouest-Est de transport énergétique", "西氣東輸")
add("一带一路", "Yīdài Yīlù", "Initiative 'la Ceinture et la Route' (Nouvelles routes de la soie)", "一帶一路")

# ==========================================
# 9. TECHNOLOGIE DE POINTE, IA ET SEMICONDUCTEURS
# ==========================================
add("人工智能", "réngōng zhìnéng", "intelligence artificielle (IA)")
add("大模型", "dà móxíng", "grand modèle de langage / modèle d'IA (LLM)")
add("机器学习", "jīqì xuéxí", "apprentissage automatique (machine learning)", "機器學習")
add("深度学习", "shēndù xuéxí", "apprentissage profond (deep learning)", "深度學習")
add("芯片", "xīnpiàn", "puce électronique, circuit intégré (chip)")
add("半导体", "bàndǎotǐ", "semi-conducteur", "半導體")
add("光刻机", "guāngkèjī", "machine de lithographie pour puces (DUV/EUV)", "光刻機")
add("晶圆", "jīngyuán", "wafer de silicium, tranche de semi-conducteur", "晶圓")
add("光伏", "guāngfú", "énergie solaire photovoltaïque")
add("储能", "chǔnéng", "stockage d'énergie par batteries", "儲能")
add("新能源", "xīn néngyuán", "énergies nouvelles, technologies vertes")
add("电动汽车", "diàndòng qìchē", "véhicule électrique (VE)", "電動汽車")
add("动力电池", "dònglì diànchí", "batterie de traction pour véhicules électriques", "動力電池")
add("自动驾驶", "zìdòng jiàshǐ", "conduite autonome, véhicule autonome", "自動駕駛")
add("云计算", "yúnsuàn", "informatique en nuage (cloud computing)", "雲計算")
add("大数据", "dàshùjù", "mégadonnées (big data)", "大數據")
add("物联网", "wùliánwǎng", "Internet des objets (IoT)", "物聯網")
add("量子计算", "liàngzǐ jìsuàn", "calcul et informatique quantique", "量子計算")

# ==========================================
# 10. DYNASTIES ET ÉPOQUES HISTORIQUES
# ==========================================
add("夏朝", "Xià Cháo", "dynastie Xia (première dynastie légendaire)")
add("商朝", "Shāng Cháo", "dynastie Shang (première dynastie historique)")
add("周朝", "Zhōu Cháo", "dynastie Zhou (Zhou occidentaux et orientaux)")
add("春秋", "Chūnqiū", "Période des Printemps et Automnes")
add("战国", "Zhànguó", "Période des Royaumes combattants", "戰國")
add("秦朝", "Qín Cháo", "dynastie Qin (unification de l'empire)")
add("汉朝", "Hàn Cháo", "dynastie Han (âge d'or impérial antique)", "漢朝")
add("三国", "Sānguó", "Période des Trois Royaumes (Wei, Shu, Wu)", "三國")
add("唐朝", "Táng Cháo", "dynastie Tang (apogée cosmopolite et poétique)")
add("宋朝", "Sòng Cháo", "dynastie Song (renaissance culturelle et scientifique)")
add("元朝", "Yuán Cháo", "dynastie Yuan (dynastie mongole, Kubilai Khan)")
add("明朝", "Míng Cháo", "dynastie Ming (construction de la Cité interdite)")
add("清朝", "Qīng Cháo", "dynastie Qing (dernière dynastie impériale mandchoue)")
add("中华民国", "Zhōnghuá Mínguó", "République de Chine (1912-1949, Taïwan)", "中華民國")
add("中华人民共和国", "Zhōnghuá Rénmín Gònghéguó", "République populaire de Chine (RPC, 1949)", "中華人民共和國")

# ==========================================
# 11. PATRONYMES CHINOIS MAJEURS (百家姓)
# ==========================================
noms_famille = [
    ("王", "Wáng", "patronyme Wang (premier nom le plus porté en Chine)"),
    ("李", "Lǐ", "patronyme Li (nom impérial de la dynastie Tang)"),
    ("张", "Zhāng", "patronyme Zhang (troisième patronyme de Chine)", "張"),
    ("刘", "Liú", "patronyme Liu (nom impérial de la dynastie Han)", "劉"),
    ("陈", "Chén", "patronyme Chen (très répandu dans le sud et la diaspora)", "陳"),
    ("杨", "Yáng", "patronyme Yang", "楊"),
    ("黄", "Huáng", "patronyme Huang", "黃"),
    ("赵", "Zhào", "patronyme Zhao (nom impérial de la dynastie Song)", "趙"),
    ("吴", "Wú", "patronyme Wu", "吳"),
    ("周", "Zhōu", "patronyme Zhou"),
    ("徐", "Xú", "patronyme Xu"),
    ("孙", "Sūn", "patronyme Sun", "孫"),
    ("马", "Mǎ", "patronyme Ma", "馬"),
    ("朱", "Zhū", "patronyme Zhu (nom impérial de la dynastie Ming)"),
    ("胡", "Hú", "patronyme Hu"),
    ("郭", "Guō", "patronyme Guo"),
    ("何", "Hé", "patronyme He"),
    ("高", "Gāo", "patronyme Gao"),
    ("林", "Lín", "patronyme Lin (très fréquent au Fujian, Taïwan et Guangdong)"),
    ("郑", "Zhèng", "patronyme Zheng", "鄭"),
    ("谢", "Xiè", "patronyme Xie", "謝"),
    ("罗", "Luó", "patronyme Luo", "羅"),
    ("梁", "Liáng", "patronyme Liang"),
    ("宋", "Sòng", "patronyme Song"),
    ("唐", "Táng", "patronyme Tang"),
    ("许", "Xǔ", "patronyme Xu", "許"),
    ("韩", "Hán", "patronyme Han", "韓"),
    ("冯", "Féng", "patronyme Feng", "馮"),
    ("邓", "Dèng", "patronyme Deng", "鄧"),
    ("曹", "Cáo", "patronyme Cao"),
    ("彭", "Péng", "patronyme Peng"),
    ("曾", "Zēng", "patronyme Zeng"),
    ("肖", "Xiào", "patronyme Xiao / Qiao"),
    ("田", "Tián", "patronyme Tian"),
    ("董", "Dǒng", "patronyme Dong"),
    ("袁", "Yuán", "patronyme Yuan"),
    ("潘", "Pān", "patronyme Pan"),
    ("于", "Yú", "patronyme Yu"),
    ("蒋", "Jiǎng", "patronyme Jiang", "蔣"),
    ("蔡", "Cài", "patronyme Cai"),
    ("余", "Yú", "patronyme Yu"),
    ("杜", "Dù", "patronyme Du"),
    ("叶", "Yè", "patronyme Ye", "葉"),
    ("程", "Chéng", "patronyme Cheng"),
    ("苏", "Sū", "patronyme Su", "蘇"),
    ("魏", "Wèi", "patronyme Wei"),
    ("吕", "Lǚ", "patronyme Lu", "呂"),
    ("丁", "Dīng", "patronyme Ding"),
    ("任", "Rèn", "patronyme Ren"),
    ("沈", "Shěn", "patronyme Shen"),
    ("姚", "Yáo", "patronyme Yao"),
    ("卢", "Lú", "patronyme Lu", "盧"),
    ("姜", "Jiāng", "patronyme Jiang"),
    ("崔", "Cuī", "patronyme Cui"),
    ("钟", "Zhōng", "patronyme Zhong", "鍾"),
    ("谭", "Tán", "patronyme Tan", "譚"),
    ("陆", "Lù", "patronyme Lu", "陸"),
    ("汪", "Wāng", "patronyme Wang"),
    ("范", "Fàn", "patronyme Fan"),
    ("金", "Jīn", "patronyme Jin"),
    ("石", "Shí", "patronyme Shi"),
    ("廖", "Liào", "patronyme Liao"),
    ("贾", "Jiǎ", "patronyme Jia", "賈"),
    ("夏", "Xià", "patronyme Xia"),
    ("韦", "Wéi", "patronyme Wei", "韋"),
    ("付", "Fù", "patronyme Fu"),
    ("方", "Fāng", "patronyme Fang"),
    ("白", "Bái", "patronyme Bai"),
    ("邹", "Zōu", "patronyme Zou", "鄒"),
    ("孟", "Mèng", "patronyme Meng"),
    ("熊", "Xióng", "patronyme Xiong"),
    ("秦", "Qín", "patronyme Qin"),
    ("邱", "Qiū", "patronyme Qiu"),
    ("江", "Jiāng", "patronyme Jiang"),
    ("尹", "Yǐn", "patronyme Yin"),
    ("毛", "Máo", "patronyme Mao"),
    ("段", "Duàn", "patronyme Duan"),
    ("雷", "Léi", "patronyme Lei"),
    ("钱", "Qián", "patronyme Qian", "錢"),
    ("汤", "Tāng", "patronyme Tang", "湯"),
    ("黎", "Lí", "patronyme Li"),
    ("易", "Yì", "patronyme Yi"),
    ("常", "Cháng", "patronyme Chang"),
    ("武", "Wǔ", "patronyme Wu"),
    ("乔", "Qiáo", "patronyme Qiao", "喬"),
    ("贺", "Hè", "patronyme He", "賀"),
    ("赖", "Lài", "patronyme Lai", "賴"),
    ("龚", "Gōng", "patronyme Gong", "龔"),
    ("文", "Wén", "patronyme Wen")
]

for item in noms_famille:
    if len(item) == 4:
        add(item[0], item[1], item[2], item[3])
    else:
        add(item[0], item[1], item[2])

# Sauvegarde finale
with open(chemin_sortie, "w", encoding="utf-8") as f:
    json.dump(termes, f, ensure_ascii=False, indent=2)

print(f"Total enrichi final : {len(termes)} termes enregistrés dans {chemin_sortie}")
