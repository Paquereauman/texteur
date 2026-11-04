import re

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Add F9 / Ctrl+B shortcut in update()
shortcut_target = r'''        // Raccourcis clavier
        if ctx.input_mut(|i| i.consume_shortcut(&OUVRIR)) {'''

shortcut_replacement = r'''        // Raccourcis clavier
        // Basculer le panneau latéral droit (F9 ou Ctrl+B)
        if ctx.input(|i| i.key_pressed(egui::Key::F9) || (i.modifiers.ctrl && i.key_pressed(egui::Key::B))) {
            self.panneau_lateral = if self.panneau_lateral == PanneauLateral::Detections {
                PanneauLateral::Aucun
            } else {
                PanneauLateral::Detections
            };
        }

        if ctx.input_mut(|i| i.consume_shortcut(&OUVRIR)) {'''

if shortcut_target in content:
    content = content.replace(shortcut_target, shortcut_replacement, 1)
    print("Added F9/Ctrl+B shortcut")
else:
    print("WARNING: shortcut_target not found!")

# 2. Refine the toolbar button for Détections
toolbar_target = r'''            // Panneaux latéraux DROIT : Détections & Dictionnaires
            let est_det = self.panneau_lateral == PanneauLateral::Detections;
            let btn_det = if est_det {
                egui::Button::new(egui::RichText::new("🎯 Détections").strong().color(Color32::from_rgb(96, 165, 250)))
                    .fill(Color32::from_rgb(26, 38, 56))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(59, 130, 246)))
            } else {
                egui::Button::new("🎯 Détections")
            };
            if ui.add(btn_det).on_hover_text("Ouvrir/fermer le panneau latéral des fautes (sur la droite)").clicked() {
                self.panneau_lateral = if est_det { PanneauLateral::Aucun } else { PanneauLateral::Detections };
            }'''

toolbar_replacement = r'''            // Panneaux latéraux DROIT : Volet Fautes & Dictionnaires
            let est_det = self.panneau_lateral == PanneauLateral::Detections;
            let libelle_det = if nb_fautes > 0 {
                format!("🎯 Volet Fautes ({nb_fautes}) [F9]")
            } else {
                "🎯 Volet Fautes [F9]".to_string()
            };
            let btn_det = if est_det {
                egui::Button::new(egui::RichText::new(&libelle_det).strong().color(Color32::from_rgb(96, 165, 250)))
                    .fill(Color32::from_rgb(26, 38, 56))
                    .stroke(Stroke::new(1.2_f32, Color32::from_rgb(59, 130, 246)))
            } else {
                egui::Button::new(egui::RichText::new(&libelle_det).strong().color(if nb_fautes > 0 { Color32::from_rgb(251, 191, 36) } else { Color32::from_rgb(203, 213, 225) }))
                    .fill(Color32::from_rgb(22, 26, 34))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(60, 70, 90)))
            };
            if ui.add(btn_det).on_hover_text("Ouvrir / fermer le volet latéral des fautes (Raccourci : touche F9 ou Ctrl+B)").clicked() {
                self.panneau_lateral = if est_det { PanneauLateral::Aucun } else { PanneauLateral::Detections };
            }'''

if toolbar_target in content:
    content = content.replace(toolbar_target, toolbar_replacement, 1)
    print("Refined toolbar button for Volet Fautes")
else:
    print("WARNING: toolbar_target not found!")

# 3. Add docked sidebar tab when PanneauLateral::Aucun
docked_target = r'''            PanneauLateral::Aucun => {}
        }

        // Panneau central de texte avec canvas papier épuré'''

docked_replacement = r'''            PanneauLateral::Aucun => {
                let nb_alertes = alertes.len();
                let nb_rep = repetitions.len();
                egui::SidePanel::right("volet_replie_droit")
                    .resizable(false)
                    .exact_width(42.0)
                    .frame(
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(18, 22, 30))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)))
                            .inner_margin(egui::Margin::symmetric(4, 12))
                    )
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            let btn_reouvrir = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("◀")
                                        .size(16.0)
                                        .color(Color32::from_rgb(52, 211, 153))
                                        .strong()
                                )
                                .fill(Color32::from_rgb(26, 36, 48))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
                            ).on_hover_text("Ouvrir le panneau latéral des alertes (F9 ou Ctrl+B)");

                            if btn_reouvrir.clicked() {
                                self.panneau_lateral = PanneauLateral::Detections;
                            }

                            ui.add_space(8.0);
                            if nb_alertes > 0 {
                                if ui.button(egui::RichText::new(format!("⚠️\n{nb_alertes}")).color(Color32::from_rgb(248, 113, 113)).size(12.0).strong())
                                    .on_hover_text("Ouvrir les alertes de grammaire et d'orthographe (F9)").clicked() {
                                    self.panneau_lateral = PanneauLateral::Detections;
                                }
                            } else if nb_rep > 0 {
                                if ui.button(egui::RichText::new(format!("◈\n{nb_rep}")).color(Color32::from_rgb(167, 139, 250)).size(12.0).strong())
                                    .on_hover_text("Ouvrir les répétitions de style (F9)").clicked() {
                                    self.panneau_lateral = PanneauLateral::Detections;
                                }
                            } else {
                                ui.label(egui::RichText::new("✨\nOK").color(Color32::from_rgb(52, 211, 153)).size(11.0));
                            }
                        });
                    });
            }
        }

        // Panneau central de texte avec canvas papier épuré'''

if docked_target in content:
    content = content.replace(docked_target, docked_replacement, 1)
    print("Added docked sidebar tab for PanneauLateral::Aucun")
else:
    print("WARNING: docked_target not found!")

# 4. Add charger_icone_texteur and with_icon in ViewportBuilder
icon_func = r'''
/// Génère l'icône officielle de Texteur (Plume émeraude & Monogramme 'T' sur fond sombre)
fn charger_icone_texteur() -> egui::IconData {
    let w = 32u32;
    let h = 32u32;
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);

    for y in 0..h {
        for x in 0..w {
            let dx = if x < 5 { 5 - x } else if x > 26 { x - 26 } else { 0 };
            let dy = if y < 5 { 5 - y } else if y > 26 { y - 26 } else { 0 };
            let dist_carre = dx * dx + dy * dy;

            if dist_carre > 25 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else if dist_carre >= 16 || x == 0 || x == 31 || y == 0 || y == 31 {
                // Bordure émeraude éclatante
                rgba.extend_from_slice(&[45, 185, 130, 255]);
            } else {
                // Fond sombre raffiné
                let mut r = 18u8;
                let mut g = 22u8;
                let mut b = 30u8;

                // Barre horizontale du T majuscule
                let est_barre_h = y >= 7 && y <= 9 && x >= 7 && x <= 24;
                // Barre verticale du T majuscule
                let est_barre_v = x >= 14 && x <= 17 && y >= 10 && y <= 24;
                // Plume dorée/émeraude sur le coin droit
                let est_plume = x >= 19 && x <= 25 && y >= 13 && y <= 23 && (x + y >= 33 && x + y <= 37);

                if est_barre_h || est_barre_v {
                    r = 250;
                    g = 252;
                    b = 255;
                } else if est_plume {
                    r = 52;
                    g = 211;
                    b = 153;
                }

                rgba.extend_from_slice(&[r, g, b, 255]);
            }
        }
    }

    egui::IconData {
        rgba,
        width: w,
        height: h,
    }
}
'''

icon_main_target = r'''    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 720.0])
            .with_drag_and_drop(true),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };'''

icon_main_replacement = r'''    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 720.0])
            .with_drag_and_drop(true)
            .with_icon(charger_icone_texteur()),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };'''

if icon_main_target in content:
    content = content.replace(icon_main_target, icon_main_replacement, 1)
    # Insert icon_func right before fn main()
    main_pos = content.rfind("fn main() -> eframe::Result {")
    content = content[:main_pos] + icon_func + "\n" + content[main_pos:]
    print("Added app icon to NativeOptions & ViewportBuilder")
else:
    print("WARNING: icon_main_target not found!")

with open(r'c:\Users\Bpaquereau\Desktop\Antidote\texteur\src\main.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("src/main.rs updated successfully!")
