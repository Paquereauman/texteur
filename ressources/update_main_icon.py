import re

path = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\main.rs"
with open(path, "r", encoding="utf-8") as f:
    text = f.read()

# Replace charger_icone_texteur
pat = r"fn charger_icone_texteur\(\) -> egui::IconData \{[\s\S]*?\n\}"
replacement = """fn charger_icone_texteur() -> egui::IconData {
    let raw = include_bytes!("../ressources/texteur_64.rgba");
    egui::IconData {
        rgba: raw.to_vec(),
        width: 64,
        height: 64,
    }
}"""

text, count = re.subn(pat, replacement, text, count=1)
print(f"Replaced charger_icone_texteur: {count}")

# Add the brand badge in the toolbar (fn barre)
# Look for: ui.horizontal_wrapped(|ui| {
#               // Groupe Fichiers
toolbar_pat = r"(fn barre\(&mut self, ui: &mut egui::Ui, nb_fautes: usize, nb_repet: usize\) \{\s+ui\.horizontal_wrapped\(\|ui\| \{\s+)(// Groupe Fichiers)"

toolbar_repl = r"""\1// Identité visuelle & Logo TEXTEUR
            ui.horizontal(|ui| {
                let (rect, _response) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, egui::CornerRadius::same(5), Color32::from_rgb(16, 185, 129));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✍",
                    egui::FontId::proportional(15.0),
                    Color32::WHITE,
                );
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new("TEXTEUR")
                        .strong()
                        .color(Color32::from_rgb(241, 245, 249))
                        .size(15.0),
                );
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            \2"""

text, count2 = re.subn(toolbar_pat, toolbar_repl, text, count=1)
print(f"Replaced toolbar: {count2}")

with open(path, "w", encoding="utf-8") as f:
    f.write(text)
print("Updated main.rs successfully")
