use super::*;
use eframe::egui::{self, Id, Margin, RichText, Stroke, Ui, pos2, vec2};
use faseela_ui::{
    buttons::{Button, ButtonVariant, ControlSize, IconButton},
    forms::{Checkbox, RadioGroup, SearchField, Select, Switch, TextField},
    theme,
    widgets::{Card, SectionHeader},
};

const FILTERABLE_COMPONENTS: [&str; 4] = [
    "Buttons",
    "Form controls",
    "Navigation",
    "Feedback & status",
];

impl Showcase {
    pub(super) fn buttons(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Variants")
                .caption("Primary, secondary, outline, ghost, and destructive actions.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                for (label, variant, id) in [
                    ("Primary", ButtonVariant::Primary, "showcase-action-primary"),
                    (
                        "Secondary",
                        ButtonVariant::Secondary,
                        "showcase-action-secondary",
                    ),
                    ("Outline", ButtonVariant::Outline, "showcase-action-outline"),
                    ("Ghost", ButtonVariant::Ghost, "showcase-action-ghost"),
                    (
                        "Destructive",
                        ButtonVariant::Destructive,
                        "showcase-action-destructive",
                    ),
                ] {
                    let response = Button::new(label)
                        .variant(variant)
                        .id(Id::unique(id))
                        .show(ui);
                    if response.clicked() {
                        self.last_action = format!("{label} button activated.");
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new(&self.last_action)
                    .size(11.0)
                    .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Sizes & icons")
                .caption("Small, medium, and large controls; leading glyphs and icon-only actions.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    Button::new("Small")
                        .size(ControlSize::Small)
                        .variant(ButtonVariant::Secondary),
                );
                ui.add(
                    Button::new("Medium")
                        .size(ControlSize::Medium)
                        .variant(ButtonVariant::Secondary),
                );
                ui.add(
                    Button::new("Large")
                        .size(ControlSize::Large)
                        .variant(ButtonVariant::Secondary),
                );
                ui.add(Button::new("With icon").icon("+"));
            });
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                let selected = Button::new("Selected state")
                    .variant(ButtonVariant::Outline)
                    .selected(self.action_selected)
                    .id(Id::unique("showcase-action-selected"))
                    .show(ui);
                if selected.clicked() {
                    self.action_selected = !self.action_selected;
                    self.last_action = format!(
                        "Selected button is {}.",
                        if self.action_selected { "on" } else { "off" }
                    );
                }

                let disabled = Button::new("Disabled")
                    .variant(ButtonVariant::Primary)
                    .enabled(false)
                    .id(Id::unique("showcase-action-disabled"))
                    .show(ui);
                let _ = disabled;

                let icon_style = icon_style(tokens);
                let icon = IconButton::new("Toggle bookmark", vec2(38.0, 38.0), &icon_style)
                    .selected(self.bookmarked)
                    .id(Id::unique("showcase-icon-bookmark"))
                    .show(ui, |ui, rect, tint| {
                        let center = rect.center();
                        let points = vec![
                            pos2(center.x, center.y - 8.0),
                            pos2(center.x + 2.6, center.y - 2.4),
                            pos2(center.x + 8.4, center.y - 1.8),
                            pos2(center.x + 4.0, center.y + 2.0),
                            pos2(center.x + 5.4, center.y + 8.0),
                            pos2(center.x, center.y + 4.9),
                            pos2(center.x - 5.4, center.y + 8.0),
                            pos2(center.x - 4.0, center.y + 2.0),
                            pos2(center.x - 8.4, center.y - 1.8),
                            pos2(center.x - 2.6, center.y - 2.4),
                        ];
                        ui.painter()
                            .add(egui::Shape::convex_polygon(points, tint, Stroke::NONE));
                    });
                if icon.clicked() {
                    self.bookmarked = !self.bookmarked;
                    self.last_action = format!(
                        "Bookmark icon is {}.",
                        if self.bookmarked { "on" } else { "off" }
                    );
                }
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new("Disabled controls remain focusable to assistive technology but never activate.")
                    .size(10.5)
                    .color(tokens.muted_text),
            );
        });
    }

    pub(super) fn form_controls(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Text & search")
                .caption("The application owns every value; the widgets report changes.")
                .show(ui);
            ui.add_space(8.0);
            TextField::new(&mut self.name)
                .label("Full name")
                .hint("e.g. Faseela Rahman")
                .helper("Used only inside this demo.")
                .id(Id::unique("showcase-name"))
                .show(ui);
            ui.add_space(8.0);

            let invalid_email = !self.email.trim().is_empty() && !self.email.contains('@');
            let mut email = TextField::new(&mut self.email)
                .label("Email")
                .hint("name@example.com")
                .id(Id::unique("showcase-email"));
            email = if invalid_email {
                email.error("Include an @ sign to make this look like an email.")
            } else {
                email.helper("A small validation rule runs on the application side.")
            };
            email.show(ui);

            ui.add_space(8.0);
            SearchField::new(&mut self.search)
                .label("Filter components")
                .hint("Try “form” or “rtl”")
                .id(Id::unique("showcase-search"))
                .show(ui);
            let query = self.search.trim().to_lowercase();
            let matches: Vec<&str> = FILTERABLE_COMPONENTS
                .iter()
                .copied()
                .filter(|name| query.is_empty() || name.to_lowercase().contains(&query))
                .collect();
            ui.label(
                RichText::new(if query.is_empty() {
                    format!("{} components available.", FILTERABLE_COMPONENTS.len())
                } else {
                    format!(
                        "{} match{} for “{}”: {}",
                        matches.len(),
                        if matches.len() == 1 { "" } else { "es" },
                        self.search.trim(),
                        if matches.is_empty() {
                            "none".to_owned()
                        } else {
                            matches.join(", ")
                        }
                    )
                })
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Choices")
                .caption(
                    "Checkbox, switch, radio group, and select all share tokens and focus rules.",
                )
                .show(ui);
            ui.add_space(8.0);
            Checkbox::new("Product notifications", &mut self.notifications)
                .helper("Also updates the Feedback & Status section.")
                .id(Id::unique("showcase-notify"))
                .show(ui);
            Switch::new("Compact spacing", &mut self.compact_mode)
                .helper("Changes the preview padding below.")
                .id(Id::unique("showcase-compact"))
                .show(ui);
            ui.add_space(8.0);
            RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut self.density)
                .label("Density")
                .helper("Arrow keys follow the reading direction.")
                .id(Id::unique("showcase-density"))
                .show(ui);
            ui.add_space(8.0);
            Select::new("Plan", &["Starter", "Team", "Enterprise"], &mut self.plan)
                .helper("The selected index stays in application state.")
                .id(Id::unique("showcase-plan"))
                .show(ui);
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Application state")
                .caption("A snapshot of the values the library has proposed so far.")
                .show(ui);
            ui.add_space(8.0);
            let padding = if self.compact_mode { 5 } else { 11 };
            Card::new()
                .elevated(true)
                .padding(Margin::same(padding))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(if self.compact_mode {
                            "Compact preview"
                        } else {
                            "Comfortable preview"
                        })
                        .strong()
                        .color(tokens.text),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{} · {} plan · {} density",
                            if self.name.trim().is_empty() {
                                "Anonymous"
                            } else {
                                self.name.trim()
                            },
                            plan_label(self.plan),
                            density_label(self.density)
                        ))
                        .size(11.0)
                        .color(tokens.muted_text),
                    );
                });
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!(
                    "Notifications {} · search “{}”",
                    if self.notifications { "on" } else { "off" },
                    self.search.trim()
                ))
                .size(10.5)
                .color(tokens.muted_text),
            );
        });
    }
}
