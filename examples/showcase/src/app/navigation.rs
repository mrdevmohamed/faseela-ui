//! Section metadata and responsive navigation presentation.
use super::{Showcase, helpers::card};
use eframe::egui::{self, Id, Margin, RichText, Ui};
use faseela_ui::{
    buttons::{Button, ButtonVariant, ControlSize},
    direction, theme,
    widgets::{Badge, BadgeTone, Card},
};

/// Navigation entries for the ten showcase sections.
pub(super) const SECTIONS: [(&str, &str); 10] = [
    (
        "Overview",
        "A live application built from the shared Faseela UI crate.",
    ),
    (
        "Foundations",
        "Semantic color, typography, spacing, and shape tokens.",
    ),
    (
        "Buttons",
        "Every variant, size, icon, selection, and disabled state.",
    ),
    (
        "Form Controls",
        "Application-owned values with labels, help, errors, and keyboard input.",
    ),
    (
        "Navigation",
        "Breadcrumbs, segmented controls, button groups, and tabs.",
    ),
    (
        "Layout",
        "Responsive panes, resize handles, and toolbar fitting.",
    ),
    (
        "Feedback & Status",
        "Progress, loading, status, badges, empty states, and tooltips.",
    ),
    (
        "Appearance",
        "Switch themes and inspect the measured contrast of their tokens.",
    ),
    (
        "Accessibility",
        "Contrast evidence, focus behavior, disabled controls, and keyboard review.",
    ),
    (
        "RTL & Arabic",
        "Right-to-left layout, mirrored controls, and Arabic display text.",
    ),
];

impl Showcase {
    pub(super) fn mobile_header(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        direction::horizontal(ui, |ui| {
            ui.heading(RichText::new("Faseela").size(24.0).color(tokens.text));
            ui.with_layout(direction::trailing(ui), |ui| {
                let label = if self.mobile_nav_open {
                    "Close"
                } else {
                    "Sections"
                };
                let response = Button::new(label)
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small)
                    .id(Id::unique("showcase-mobile-menu"))
                    .show(ui);
                if response.clicked() {
                    self.mobile_nav_open = !self.mobile_nav_open;
                }
            });
        });
        ui.label(
            RichText::new(format!(
                "{} · {} · {}",
                SECTIONS[self.selected_section].0,
                self.selected_theme.label(),
                if self.rtl { "RTL" } else { "LTR" }
            ))
            .size(11.0)
            .color(tokens.muted_text),
        );
    }

    pub(super) fn mobile_navigation(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            ui.label(
                RichText::new("SHOWCASE SECTIONS")
                    .size(9.5)
                    .strong()
                    .color(tokens.secondary),
            );
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .id_salt("showcase-mobile-nav-scroll")
                .max_height(238.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (index, (title, _)) in SECTIONS.iter().enumerate() {
                        let selected = self.selected_section == index;
                        let response = Button::new(*title)
                            .variant(if selected {
                                ButtonVariant::Primary
                            } else {
                                ButtonVariant::Ghost
                            })
                            .selected(selected)
                            .size(ControlSize::Small)
                            .full_width()
                            .id(Id::unique("showcase-mobile-nav").with(index))
                            .show(ui);
                        if response.clicked() {
                            self.select_section(index);
                        }
                    }
                });
        });
    }

    pub(super) fn sidebar(&mut self, ui: &mut Ui, height: f32) {
        let tokens = theme::active_tokens(ui.ctx());
        Card::new()
            .outer_margin(Margin {
                left: 8,
                right: 8,
                top: 8,
                bottom: 8,
            })
            .padding(Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height((height - 44.0).max(0.0));
                ui.label(
                    RichText::new("Faseela")
                        .size(24.0)
                        .strong()
                        .color(tokens.text),
                );
                ui.label(
                    RichText::new("UI COMPONENT LIBRARY")
                        .size(9.0)
                        .strong()
                        .color(tokens.secondary),
                );
                ui.add_space(18.0);

                for (index, (title, _)) in SECTIONS.iter().enumerate() {
                    let selected = self.selected_section == index;
                    let response = Button::new(*title)
                        .variant(if selected {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Ghost
                        })
                        .selected(selected)
                        .size(ControlSize::Small)
                        .full_width()
                        .id(Id::unique("showcase-nav").with(index))
                        .show(ui);
                    if response.clicked() {
                        self.select_section(index);
                    }
                    ui.add_space(2.0);
                }

                ui.with_layout(egui::Layout::bottom_up(direction::start_align(ui)), |ui| {
                    ui.label(
                        RichText::new("Rust · egui · WebAssembly")
                            .size(9.5)
                            .color(tokens.muted_text),
                    );
                    ui.label(
                        RichText::new("Faseela UI 0.1")
                            .size(9.5)
                            .color(tokens.secondary),
                    );
                });
            });
    }

    pub(super) fn section_intro(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        let index = self.selected_section.min(SECTIONS.len() - 1);
        let (title, caption) = SECTIONS[index];
        let section_number = format!("{:02} / 10", index + 1);
        let direction_label = if self.rtl { "RTL" } else { "LTR" };
        ui.horizontal_wrapped(|ui| {
            ui.heading(RichText::new(title).size(25.0).color(tokens.text));
            ui.add(Badge::new(&section_number, BadgeTone::Neutral));
            ui.add(Badge::new(self.selected_theme.label(), BadgeTone::Primary));
            ui.add(Badge::new(direction_label, BadgeTone::Neutral));
        });
        ui.label(RichText::new(caption).size(12.0).color(tokens.muted_text));
    }
}
