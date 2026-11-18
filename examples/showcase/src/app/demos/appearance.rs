use super::*;
use eframe::egui::{Id, RichText, Ui};
use faseela_ui::{
    buttons::{Button, ButtonVariant},
    theme::{self, ThemePreset, palette},
    widgets::{Badge, BadgeTone, SectionHeader},
};

impl Showcase {
    pub(super) fn appearance(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Theme presets")
                .caption("Switching a preset re-applies semantic tokens on the next frame.")
                .show(ui);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                for (index, preset) in ThemePreset::ALL.iter().enumerate() {
                    let selected = *preset == self.selected_theme;
                    let response = Button::new(preset.label())
                        .variant(if selected {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Outline
                        })
                        .selected(selected)
                        .id(Id::unique("showcase-theme").with(index))
                        .show(ui);
                    if response.clicked() {
                        self.selected_theme = *preset;
                        self.last_action = format!("Theme set to {}.", preset.label());
                        ui.ctx().request_repaint();
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!(
                    "Selected: {} · painted: {}",
                    self.selected_theme.label(),
                    theme::active_preset(ui.ctx()).label()
                ))
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Faseela identity")
                .caption("The official palette anchors the default preset.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                for (name, color) in [
                    ("Deep", palette::BACKGROUND),
                    ("Green", palette::PRIMARY),
                    ("Mint", palette::PRIMARY_LIGHT),
                    ("Gold", palette::ACCENT),
                ] {
                    palette_swatch(ui, name, color, tokens);
                }
            });
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Measured contrast")
                .caption("Computed with WCAG relative luminance from the selected preset.")
                .show(ui);
            ui.add_space(8.0);
            let checks = contrast_checks(self.selected_theme.tokens());
            for check in checks.iter().take(7) {
                contrast_row(ui, check);
            }
            let failures = checks.iter().filter(|check| !check.passes()).count();
            ui.add_space(6.0);
            ui.add(Badge::new(
                if failures == 0 {
                    "All sampled pairs pass"
                } else {
                    "Some sampled pairs fail"
                },
                if failures == 0 {
                    BadgeTone::Success
                } else {
                    BadgeTone::Error
                },
            ));
        });
    }
}
