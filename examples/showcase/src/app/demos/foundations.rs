use super::*;
use eframe::egui::{
    self, Align2, CornerRadius, FontId, Id, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2,
    vec2,
};
use faseela_ui::{
    buttons::{Button, ButtonVariant},
    text,
    theme::{self, ThemePreset},
    widgets::SectionHeader,
};

impl Showcase {
    pub(super) fn overview(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            ui.label(
                RichText::new("WELCOME BACK")
                    .size(9.5)
                    .strong()
                    .color(tokens.secondary),
            );
            ui.add_space(4.0);
            ui.heading(
                RichText::new("A production-minded egui foundation")
                    .size(23.0)
                    .color(tokens.text),
            );
            ui.label(
                RichText::new(
                    "Every section is this application: the same library widgets painted in a \
                     responsive shell with real state, keyboard input, and four themes.",
                )
                .color(tokens.muted_text),
            );
            ui.add_space(8.0);
            let arabic = text::display_text("مرحبًا بك في مكتبة فسيلة للواجهات.");
            ui.add(
                egui::Label::new(RichText::new(arabic).size(13.0).color(tokens.secondary)).wrap(),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Quick actions")
                .caption("These buttons change the selected section, theme, or direction.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let open_buttons = Button::new("Browse buttons")
                    .variant(ButtonVariant::Primary)
                    .id(Id::unique("showcase-overview-buttons"))
                    .show(ui);
                if open_buttons.clicked() {
                    self.select_section(2);
                    self.last_action = "Opened the Buttons section.".to_owned();
                }

                let open_forms = Button::new("Try forms")
                    .variant(ButtonVariant::Secondary)
                    .id(Id::unique("showcase-overview-forms"))
                    .show(ui);
                if open_forms.clicked() {
                    self.select_section(3);
                    self.last_action = "Opened the Form Controls section.".to_owned();
                }

                let cycle_theme = Button::new("Cycle theme")
                    .variant(ButtonVariant::Outline)
                    .id(Id::unique("showcase-overview-theme"))
                    .show(ui);
                if cycle_theme.clicked() {
                    let index = ThemePreset::ALL
                        .iter()
                        .position(|preset| *preset == self.selected_theme)
                        .unwrap_or(0);
                    self.selected_theme = ThemePreset::ALL[(index + 1) % ThemePreset::ALL.len()];
                    self.last_action = format!("Theme set to {}.", self.selected_theme.label());
                }

                let toggle_direction = Button::new("Toggle direction")
                    .variant(ButtonVariant::Ghost)
                    .id(Id::unique("showcase-overview-direction"))
                    .show(ui);
                if toggle_direction.clicked() {
                    self.rtl = !self.rtl;
                    self.last_action = format!(
                        "Reading direction set to {}.",
                        if self.rtl { "RTL" } else { "LTR" }
                    );
                }
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new(&self.last_action)
                    .size(11.0)
                    .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Live state")
                .caption("Values reported by this running instance.")
                .show(ui);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                stat_tile(ui, "SECTIONS", "10");
                stat_tile(ui, "THEMES", "4");
                stat_tile(ui, "DIRECTION", if self.rtl { "RTL" } else { "LTR" });
                stat_tile(ui, "FORM PLAN", plan_label(self.plan));
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!(
                    "Theme {} · direction {} · notifications {} · filter “{}”",
                    self.selected_theme.label(),
                    if self.rtl {
                        "right-to-left"
                    } else {
                        "left-to-right"
                    },
                    if self.notifications { "on" } else { "off" },
                    if self.search.trim().is_empty() {
                        "none"
                    } else {
                        self.search.trim()
                    }
                ))
                .size(11.0)
                .color(tokens.muted_text),
            );
        });
    }

    pub(super) fn foundations(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Semantic color tokens")
                .caption("Widgets paint from roles, not fixed colors, so presets switch cleanly.")
                .show(ui);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                for (name, color) in [
                    ("background", tokens.background),
                    ("surface", tokens.surface),
                    ("elevated_surface", tokens.elevated_surface),
                    ("primary", tokens.primary),
                    ("secondary", tokens.secondary),
                    ("accent", tokens.accent),
                    ("text", tokens.text),
                    ("muted_text", tokens.muted_text),
                    ("border", tokens.border),
                    ("control_border", tokens.control_border),
                    ("focus", tokens.focus),
                    ("error", tokens.error),
                    ("warning", tokens.warning),
                    ("success", tokens.success),
                ] {
                    token_swatch(ui, name, color, tokens);
                }
            });
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Typography")
                .caption("The crate uses egui's proportional family and application-scaled sizes.")
                .show(ui);
            ui.add_space(8.0);
            ui.label(
                RichText::new("Display heading")
                    .size(23.0)
                    .strong()
                    .color(tokens.text),
            );
            ui.label(
                RichText::new("Body copy supports the theme's primary text token.")
                    .size(13.0)
                    .color(tokens.text),
            );
            ui.label(
                RichText::new("Caption and supporting text use the muted token.")
                    .size(10.5)
                    .color(tokens.muted_text),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new("0123456789 · ABCDEFGHIJKLM · مرحبًا")
                    .size(12.0)
                    .color(tokens.secondary),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Spacing & shape")
                .caption("Shared widgets use a compact 4-point rhythm and 4–10 point radii.")
                .show(ui);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                for (label, width) in [("4", 4.0), ("8", 8.0), ("12", 12.0), ("20", 20.0)] {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(width + 44.0, 26.0), Sense::hover());
                    let bar =
                        Rect::from_min_size(rect.left_center() - vec2(0.0, 4.0), vec2(width, 8.0));
                    ui.painter()
                        .rect_filled(bar, CornerRadius::same(2), tokens.primary);
                    ui.painter().text(
                        pos2(bar.right() + 6.0, bar.center().y),
                        Align2::LEFT_CENTER,
                        format!("{label} pt"),
                        FontId::proportional(9.5),
                        tokens.muted_text,
                    );
                }
            });
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for radius in [2.0_f32, 4.0, 7.0, 10.0] {
                    let (rect, _) = ui.allocate_exact_size(vec2(58.0, 36.0), Sense::hover());
                    let shape = Rect::from_min_size(rect.min, vec2(42.0, 30.0));
                    ui.painter().rect(
                        shape,
                        CornerRadius::same(radius as u8),
                        tokens.elevated_surface,
                        Stroke::new(1.0, tokens.control_border),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        pos2(rect.right() - 10.0, rect.center().y),
                        Align2::RIGHT_CENTER,
                        format!("{}", radius as u8),
                        FontId::proportional(9.5),
                        tokens.muted_text,
                    );
                }
            });
        });
    }
}
