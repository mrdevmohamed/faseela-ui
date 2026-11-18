//! Demonstrations call the actual public library APIs and mutate application state.
mod accessibility;
mod appearance;
mod arabic;
mod controls;
mod feedback;
mod foundations;
mod layout;
mod navigation;

use super::{Showcase, helpers::*, navigation::SECTIONS};
use eframe::egui::{RichText, Ui};
use faseela_ui::theme;

impl Showcase {
    pub(super) fn section_content(&mut self, ui: &mut Ui) {
        let index = self.selected_section.min(SECTIONS.len() - 1);
        match index {
            0 => self.overview(ui),
            1 => self.foundations(ui),
            2 => self.buttons(ui),
            3 => self.form_controls(ui),
            4 => self.navigation(ui),
            5 => self.layout(ui),
            6 => self.feedback(ui),
            7 => self.appearance(ui),
            8 => self.accessibility(ui),
            _ => self.rtl_arabic(ui),
        }

        let tokens = theme::active_tokens(ui.ctx());
        ui.add_space(18.0);
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Faseela UI").strong().color(tokens.secondary));
            ui.label(
                RichText::new(
                    "Shared components · application-owned state · native and WebAssembly",
                )
                .size(10.5)
                .color(tokens.muted_text),
            );
        });
        ui.add_space(16.0);
    }
}
