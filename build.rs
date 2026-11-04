fn main() {
    // Dictionnaire chinois personnel : facultatif, absent du dépôt public
    let perso = "ressources/base_chinois_perso.json";
    println!("cargo:rerun-if-changed={perso}");
    let sortie = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("base_chinois_perso.json");
    std::fs::write(&sortie, std::fs::read_to_string(perso).unwrap_or_else(|_| "[]".into())).unwrap();
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("ressources/texteur.ico");
        res.set("FileDescription", "Texteur - Correcteur de Français & Écriture");
        res.set("ProductName", "Texteur");
        res.set("OriginalFilename", "Texteur.exe");
        res.set("LegalCopyright", "Copyright © 2026");
        if let Err(e) = res.compile() {
            eprintln!("Erreur lors de la compilation des ressources Windows : {}", e);
        }
    }
}
