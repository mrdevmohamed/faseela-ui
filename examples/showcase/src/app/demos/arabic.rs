use super::*;
use eframe::egui::{Id, RichText, Ui};
use faseela_ui::{
    forms::TextField,
    navigation::ButtonGroup,
    theme,
    widgets::{Badge, BadgeTone, SectionHeader},
};

impl Showcase {
    pub(super) fn rtl_arabic(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Reading direction")
                .caption("The shared layout helpers mirror navigation and pane order.")
                .show(ui);
            ui.add_space(8.0);
            let mut selected = usize::from(self.rtl);
            let result = ButtonGroup::new(&["LTR", "RTL"], &mut selected)
                .accessible_label("Reading direction")
                .id(Id::unique("showcase-rtl"))
                .show(ui);
            if result.activated.is_some() {
                self.rtl = selected == 1;
                self.last_action = format!(
                    "Reading direction set to {}.",
                    if self.rtl { "RTL" } else { "LTR" }
                );
            }
            ui.add_space(8.0);
            ui.add(Badge::new(
                if self.rtl {
                    "RIGHT TO LEFT"
                } else {
                    "LEFT TO RIGHT"
                },
                BadgeTone::Primary,
            ));
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!(
                    "Active direction: {} · segmented controls and split panes follow it.",
                    if self.rtl { "RTL" } else { "LTR" }
                ))
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Arabic display text")
                .caption("Short display labels are reordered; egui handles Arabic letter shaping.")
                .show(ui);
            ui.add_space(8.0);
            display_label(ui, "فسيلة: واجهة عربية أنيقة وسريعة", 18.0, tokens.text);
            display_label(ui, "الأرقام 2026 و3.5", 12.0, tokens.muted_text);
            ui.label(
                RichText::new("Display-only samples; mixed-text wrapping is approximate.")
                    .small()
                    .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Logical text stays logical")
                .caption("Edit the source below; the display line is generated separately.")
                .show(ui);
            ui.add_space(8.0);
            TextField::new(&mut self.arabic_input)
                .label("Arabic source")
                .hint("Type logical Unicode text")
                .helper("Stored as typed; display reordering is a separate step. Bidi editing remains limited.")
                .id(Id::unique("showcase-arabic-input"))
                .show(ui);
            ui.add_space(6.0);
            display_label(ui, &self.arabic_input, 13.0, tokens.secondary);
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Mirrored: navigation order, split panes, directional arrows, and control alignment. \
                     Logical: numbers, URLs, code, and stored text. Full bidi shaping in editable text \
                     is not provided; the helper is for display labels.",
                )
                .size(10.0)
                .color(tokens.muted_text),
            );
        });
    }
}
