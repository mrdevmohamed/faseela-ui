//! Shared labels and helper/error presentation.
use crate::theme;
use egui::{Color32, FontId, Response, RichText, Role, Ui, accesskit};

pub(super) const FIELD_FONT_SIZE: f32 = 13.0;
pub(super) const SUPPORT_FONT_SIZE: f32 = 11.5;
pub(super) fn label_galley(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    wrap_width: f32,
) -> std::sync::Arc<egui::Galley> {
    ui.fonts_mut(|fonts| fonts.layout(text.to_owned(), font, color, wrap_width.max(1.0)))
}

pub(super) fn field_label(ui: &mut Ui, label: &str, enabled: bool) {
    let tokens = theme::active_tokens(ui.ctx());
    let color = if enabled {
        tokens.text
    } else {
        tokens.disabled_text
    };
    ui.label(RichText::new(label).size(FIELD_FONT_SIZE).color(color));
}

pub(super) fn support_text(
    ui: &mut Ui,
    helper: Option<&str>,
    error: Option<&str>,
    enabled: bool,
    control: &Response,
) {
    let tokens = theme::active_tokens(ui.ctx());
    if let Some(error) = error {
        let color = if enabled {
            tokens.error
        } else {
            tokens.disabled_text
        };
        let response = ui.label(RichText::new(error).size(SUPPORT_FONT_SIZE).color(color));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(Role::Alert);
            node.set_label(error);
        });
        ui.ctx().accesskit_node_builder(control.id, |node| {
            node.set_invalid(accesskit::Invalid::True);
            node.push_described_by(response.id.accesskit_id());
        });
    } else if let Some(helper) = helper {
        let color = if enabled {
            tokens.muted_text
        } else {
            tokens.disabled_text
        };
        let response = ui.label(RichText::new(helper).size(SUPPORT_FONT_SIZE).color(color));
        ui.ctx().accesskit_node_builder(control.id, |node| {
            node.push_described_by(response.id.accesskit_id());
        });
    }
}
