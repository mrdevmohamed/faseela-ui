use super::*;
use eframe::egui::{Id, RichText, Ui};
use faseela_ui::{
    buttons::{Button, ButtonVariant, ControlSize},
    theme,
    widgets::{
        Badge, BadgeTone, EmptyState, LoadingIndicator, ProgressBar, SectionHeader,
        StatusIndicator, Tooltip,
    },
};

impl Showcase {
    pub(super) fn feedback(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Progress")
                .caption("A determinate bar with visible and accessible percent value.")
                .show(ui);
            ui.add_space(8.0);
            ProgressBar::new(self.progress)
                .label("Release upload")
                .show_value(true)
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if Button::new("Advance 10%")
                    .variant(ButtonVariant::Secondary)
                    .size(ControlSize::Small)
                    .id(Id::unique("showcase-progress-advance"))
                    .show(ui)
                    .clicked()
                {
                    self.progress = (self.progress + 0.1).min(1.0);
                }
                if Button::new("Reset")
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small)
                    .id(Id::unique("showcase-progress-reset"))
                    .show(ui)
                    .clicked()
                {
                    self.progress = 0.0;
                }
                ui.label(
                    RichText::new(format!("{:.0}% complete", self.progress * 100.0))
                        .size(10.5)
                        .color(tokens.muted_text),
                );
            });
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Loading & status")
                .caption("Spinners and status rows use semantic tones from the active theme.")
                .show(ui);
            ui.add_space(8.0);
            if Button::new(if self.show_loading {
                "Stop loading"
            } else {
                "Start loading"
            })
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small)
            .id(Id::unique("showcase-loading-toggle"))
            .show(ui)
            .clicked()
            {
                self.show_loading = !self.show_loading;
            }
            ui.add_space(8.0);
            if self.show_loading {
                LoadingIndicator::new().label("Syncing workspace…").show(ui);
            } else {
                StatusIndicator::new("Idle", BadgeTone::Neutral)
                    .detail("No background work")
                    .show(ui);
            }
            ui.add_space(8.0);
            StatusIndicator::new("API connected", BadgeTone::Success)
                .detail("42 ms")
                .show(ui);
            StatusIndicator::new("Token expires soon", BadgeTone::Warning)
                .detail("in 2 days")
                .show(ui);
            StatusIndicator::new("Sync failed", BadgeTone::Error)
                .detail("retry queued")
                .show(ui);
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Badges, empty state & tooltip")
                .caption("Tone foregrounds and measurements come from the same tokens.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.add(Badge::new("Neutral", BadgeTone::Neutral));
                ui.add(Badge::new("Primary", BadgeTone::Primary));
                ui.add(Badge::new("Success", BadgeTone::Success));
                ui.add(Badge::new("Warning", BadgeTone::Warning));
                ui.add(Badge::new("Error", BadgeTone::Error));
            });
            ui.add_space(10.0);
            if self.notifications {
                for (title, tone, detail) in [
                    ("Build finished", BadgeTone::Success, "2 minutes ago"),
                    ("Update available", BadgeTone::Warning, "version 0.2"),
                    ("Sync failed", BadgeTone::Error, "retry scheduled"),
                ] {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(title).size(11.5).color(tokens.text));
                        ui.add(Badge::new(detail, tone));
                    });
                }
                if Button::new("Clear notifications")
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small)
                    .id(Id::unique("showcase-clear-notifications"))
                    .show(ui)
                    .clicked()
                {
                    self.notifications = false;
                }
            } else {
                let empty = EmptyState::new("Inbox is clear")
                    .description("Turn on notifications to see the populated state.")
                    .action("Show sample notifications")
                    .show(ui);
                if empty.inner.action_clicked {
                    self.notifications = true;
                }
            }
            ui.add_space(8.0);
            let tooltip_response = Button::new("Hover for tooltip")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .id(Id::unique("showcase-tooltip"))
                .show(ui);
            let _ = Tooltip::new(
                "Tooltips are keyboard-adjacent supporting text; the visible label still names the control.",
            )
            .show(tooltip_response);
        });
    }
}
