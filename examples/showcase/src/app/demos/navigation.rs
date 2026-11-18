use super::*;
use eframe::egui::{Align2, CornerRadius, FontId, Id, Rect, RichText, Sense, Ui, pos2, vec2};
use faseela_ui::{
    navigation::{Breadcrumb, ButtonGroup, SegmentedControl},
    tabs::Tab,
    theme,
    widgets::SectionHeader,
};

impl Showcase {
    pub(super) fn navigation(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Breadcrumb")
                .caption("Earlier path entries stay interactive; the current page does not.")
                .show(ui);
            ui.add_space(8.0);
            let crumbs = ["Workspace", "Projects", "Faseela UI", "Navigation"];
            let result = Breadcrumb::new(&crumbs)
                .id(Id::unique("showcase-breadcrumb"))
                .show(ui);
            if let Some(index) = result.activated {
                self.breadcrumb_last = Some(index);
                self.last_action = format!("Breadcrumb opened “{}”.", crumbs[index]);
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(match self.breadcrumb_last {
                    Some(index) => format!("Last opened crumb: {}", crumbs[index]),
                    None => "No breadcrumb has been activated.".to_owned(),
                })
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Modes & views")
                .caption(
                    "Exclusive controls update application-owned indices and report activation.",
                )
                .show(ui);
            ui.add_space(8.0);
            let segmented =
                SegmentedControl::new(&["Design", "Develop", "Ship"], &mut self.nav_segment)
                    .label("Workspace view")
                    .full_width()
                    .id(Id::unique("showcase-segmented"))
                    .show(ui);
            if let Some(index) = segmented.activated {
                self.last_action = format!(
                    "Workspace view set to {}.",
                    ["Design", "Develop", "Ship"][index]
                );
            }
            ui.add_space(10.0);
            let group = ButtonGroup::new(&["Day", "Week", "Month"], &mut self.nav_group)
                .accessible_label("Reporting period")
                .id(Id::unique("showcase-period"))
                .show(ui);
            if let Some(index) = group.activated {
                self.last_action = format!(
                    "Reporting period set to {}.",
                    ["Day", "Week", "Month"][index]
                );
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!(
                    "View {} · period {}",
                    ["Design", "Develop", "Ship"][self.nav_segment.min(2)],
                    ["Day", "Week", "Month"][self.nav_group.min(2)]
                ))
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Tabs")
                .caption("The Tab primitive paints with caller-owned selection and focus.")
                .show(ui);
            ui.add_space(8.0);
            self.tab_strip(ui);
            ui.add_space(8.0);
            ui.label(
                RichText::new(match self.nav_tab {
                    0 => "Preview: responsive content area.",
                    _ => "Details: keyboard focus and selected state are managed by the caller.",
                })
                .size(11.0)
                .color(tokens.muted_text),
            );
        });
    }

    pub(super) fn tab_strip(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        let labels = ["Preview", "Details"];
        let width = ui.available_width().max(1.0);
        let gap = 4.0;
        let tab_width = ((width - gap) * 0.5).max(1.0);
        let (outer, _) = ui.allocate_exact_size(vec2(width, 34.0), Sense::hover());
        let rtl = faseela_ui::direction::is_rtl(ui);
        for (index, label) in labels.iter().enumerate() {
            let physical_index = if rtl { labels.len() - 1 - index } else { index };
            let rect = Rect::from_min_size(
                pos2(
                    outer.left() + physical_index as f32 * (tab_width + gap),
                    outer.top(),
                ),
                vec2(tab_width, 34.0),
            );
            let selected = self.nav_tab == index;
            let response = Tab::new(Id::unique("showcase-tabs").with(index), label, selected)
                .show_at(ui, rect, |ui, response| {
                    let fill = if selected {
                        tokens.selection
                    } else if response.hovered() {
                        tokens.hover
                    } else {
                        tokens.elevated_surface
                    };
                    let fore = if selected {
                        tokens.on_primary
                    } else {
                        tokens.text
                    };
                    ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        *label,
                        FontId::proportional(12.5),
                        fore,
                    );
                });
            if response.clicked() {
                self.nav_tab = index;
            }
        }
    }
}
