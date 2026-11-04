with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "r", encoding="utf-8") as f:
    text = f.read()

# Locate from 'impl Default for CorrecteurGrammaire' to 'let mut preps = HashSet::new();'
idx_default = text.find("impl Default for CorrecteurGrammaire")
idx_preps = text.find("let mut preps = HashSet::new();")

clean_header = """impl Default for CorrecteurGrammaire {
    fn default() -> Self {
        Self::new()
    }
}

impl CorrecteurGrammaire {
    pub fn new() -> Self {
        let mut villes = HashSet::new();
        for v in &[
            "shenzhen", "shanghai", "beijing", "pekin", "pékin", "canton", "guangzhou", "chengdu",
            "wuhan", "tianjin", "hangzhou", "nankin", "nanjing", "chongqing", "xian", "xi'an",
            "suzhou", "zhengzhou", "dongguan", "foshan", "hong kong", "hongkong", "taipei",
            "paris", "lyon", "marseille", "toulouse", "nice", "nantes", "strasbourg", "montpellier",
            "bordeaux", "lille", "rennes", "reims", "toulon", "saint-étienne", "le havre", "grenoble",
            "dijon", "angers", "nîmes", "villeurbanne", "clermont-ferrand", "le mans", "aix-en-provence",
            "brest", "tours", "amiens", "limoges", "annecy", "perpignan", "boulogne-billancourt",
            "metz", "besançon", "orléans", "saint-denis", "argenteuil", "rouen", "mulhouse", "montreuil",
            "caen", "nancy", "londres", "berlin", "madrid", "rome", "tokyo", "seoul", "séoul",
            "singapour", "new york", "los angeles", "chicago", "san francisco", "toronto", "montreal",
            "montréal", "genève", "geneve", "bruxelles"
        ] {
            villes.insert(*v);
        }

        """

text = text[:idx_default] + clean_header + text[idx_preps:]

with open(r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\grammaire.rs", "w", encoding="utf-8", newline="\r\n") as f:
    f.write(text)

print("Cleaned up default and new headers successfully!")
