use super::*;
use eframe::egui::{Id, RichText, Ui};
use faseela_ui::{
    buttons::{Button, ButtonVariant, ControlSize},
    forms::Checkbox,
    theme,
    widgets::{Badge, BadgeTone, SectionHeader},
};

impl Showcase {
    pub(super) fn accessibility(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Contrast evidence")
                .caption("WCAG AA: 4.5:1 for normal text and 3:1 for non-text indicators.")
                .show(ui);
            ui.add_space(8.0);
            let checks = contrast_checks(tokens);
            let text: Vec<ContrastCheck> = checks
                .iter()
                .copied()
                .filter(|check| check.minimum >= 4.5)
                .collect();
            let non_text: Vec<ContrastCheck> = checks
                .iter()
                .copied()
                .filter(|check| check.minimum < 4.5)
                .collect();
            let text_pass = text.iter().filter(|check| check.passes()).count();
            let non_text_pass = non_text.iter().filter(|check| check.passes()).count();
            ui.horizontal_wrapped(|ui| {
                ui.add(Badge::new(
                    &format!("Text {text_pass}/{}", text.len()),
                    if text_pass == text.len() {
                        BadgeTone::Success
                    } else {
                        BadgeTone::Error
                    },
                ));
                ui.add(Badge::new(
                    &format!("Non-text {non_text_pass}/{}", non_text.len()),
                    if non_text_pass == non_text.len() {
                        BadgeTone::Success
                    } else {
                        BadgeTone::Error
                    },
                ));
            });
            ui.add_space(8.0);
            for check in checks.iter().take(7) {
                contrast_row(ui, check);
            }
            ui.label(
                RichText::new(
                    "Ratios are sampled pairs, not a complete audit of every custom composition.",
                )
                .size(10.0)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Keyboard focus")
                .caption("Focus can be moved programmatically and is painted by the widgets.")
                .show(ui);
            ui.add_space(8.0);
            let trigger_id = Id::unique("showcase-a11y-focus-trigger");
            let target_id = Id::unique("showcase-a11y-target");
            ui.horizontal_wrapped(|ui| {
                if Button::new("Focus the target")
                    .variant(ButtonVariant::Secondary)
                    .size(ControlSize::Small)
                    .id(trigger_id)
                    .show(ui)
                    .clicked()
                {
                    ui.memory_mut(|memory| memory.request_focus(target_id));
                }
                if ui
                    .add(
                        Button::new("Focus target")
                            .variant(ButtonVariant::Outline)
                            .size(ControlSize::Small)
                            .id(target_id),
                    )
                    .clicked()
                {
                    self.last_action =
                        "Focus target activated with pointer or keyboard.".to_owned();
                }
            });
            let focused = ui.ctx().memory(|memory| memory.focused()) == Some(target_id);
            ui.add_space(4.0);
            ui.label(
                RichText::new(if focused {
                    "Focus is on the target. Space or Enter would activate it."
                } else {
                    "Focus is elsewhere. Use the button above or press Tab."
                })
                .size(10.5)
                .color(if focused {
                    tokens.secondary
                } else {
                    tokens.muted_text
                }),
            );
            ui.label(
                RichText::new(&self.last_action)
                    .small()
                    .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Disabled & reviewed")
                .caption("Disabled widgets stay named and visible without accepting activation.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    Button::new("Disabled action")
                        .variant(ButtonVariant::Primary)
                        .enabled(false),
                );
                let mut disabled_value = false;
                Checkbox::new("Disabled checkbox", &mut disabled_value)
                    .enabled(false)
                    .show(ui);
            });
            ui.add_space(8.0);
            Checkbox::new(
                "I can reach every interactive control with Tab",
                &mut self.keyboard_review,
            )
            .helper("A real checkbox, not an automated claim.")
            .id(Id::unique("showcase-keyboard-review"))
            .show(ui);
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Known limitation: eframe's web canvas does not expose per-widget AccessKit nodes \
                     to browsers; native AccessKit support depends on the platform backend.",
                )
                .size(10.0)
                .color(tokens.muted_text),
            );
        });
    }
}
