use super::*;
use eframe::egui::{self, Align, Id, RichText, Ui};
use faseela_ui::{
    forms::TextField,
    navigation::ButtonGroup,
    text, theme,
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
                .caption("The display helper applies Arabic shaping for painting only.")
                .show(ui);
            ui.add_space(8.0);
            let headline = text::display_text("فسيلة: واجهة عربية أنيقة وسريعة");
            ui.add(
                egui::Label::new(RichText::new(headline).size(18.0).color(tokens.text))
                    .halign(if self.rtl { Align::RIGHT } else { Align::LEFT })
                    .wrap(),
            );
            let body = text::display_text(
                "تعرض هذه الصفحة نصًا عربيًا متصل الحروف، مع أرقام لاتينية مثل 2026 و3.5.",
            );
            ui.add(
                egui::Label::new(RichText::new(body).size(12.0).color(tokens.muted_text))
                    .halign(if self.rtl { Align::RIGHT } else { Align::LEFT })
                    .wrap(),
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
                .helper("Stored as typed; display shaping is a separate step.")
                .id(Id::unique("showcase-arabic-input"))
                .show(ui);
            ui.add_space(6.0);
            let shaped = text::display_text(&self.arabic_input);
            ui.add(
                egui::Label::new(RichText::new(shaped).size(13.0).color(tokens.secondary))
                    .halign(if self.rtl { Align::RIGHT } else { Align::LEFT })
                    .wrap(),
            );
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
