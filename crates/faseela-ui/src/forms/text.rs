//! Logical Unicode text input and search presentation using egui's editor.
use super::common::{FIELD_FONT_SIZE, field_label, support_text};
use crate::theme;
use egui::RichText;
use egui::{
    CornerRadius, Id, Margin, Response, Role, Sense, Stroke, StrokeKind, TextEdit, Ui, WidgetInfo,
    accesskit, vec2,
};

/// A single-line text field with a label, placeholder, helper text, and error state.
///
/// The application owns the string. `show` returns the underlying [`Response`], including
/// `changed()` and focus information. Use [`Self::id`] for a stable identity when the field may
/// move in the UI tree.
pub struct TextField<'a> {
    value: &'a mut String,
    label: Option<&'a str>,
    hint: &'a str,
    helper: Option<&'a str>,
    error: Option<&'a str>,
    enabled: bool,
    id: Option<Id>,
    width: Option<f32>,
    leading_icon: Option<&'a str>,
    role: Option<Role>,
}

impl<'a> TextField<'a> {
    /// Create a field bound to an application-owned string.
    pub fn new(value: &'a mut String) -> Self {
        Self {
            value,
            label: None,
            hint: "",
            helper: None,
            error: None,
            enabled: true,
            id: None,
            width: None,
            leading_icon: None,
            role: None,
        }
    }

    /// Visible and accessible field label.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Placeholder shown when the field is empty.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    /// Supporting text shown below the field.
    pub fn helper(mut self, helper: &'a str) -> Self {
        self.helper = Some(helper);
        self
    }

    /// Error message. Also switches the control border to the error color.
    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    /// Enable or disable the field.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable ID to the text edit.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Request a width in logical points. The field still respects available width.
    pub fn width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.width = Some(width.min(4096.0));
        }
        self
    }

    fn leading_icon(mut self, icon: &'a str) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Show the field.
    pub fn show(self, ui: &mut Ui) -> Response {
        if self.enabled {
            self.show_in(ui)
        } else {
            ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner
        }
    }

    fn show_in(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let enabled = ui.is_enabled();
        let error_color = self.error.map(|_| tokens.error);
        let available_width = ui.available_width().max(0.0);
        let requested_width = self.width.unwrap_or(available_width);
        let width = requested_width.min(available_width).min(4096.0);
        if width < 24.0 {
            let id = self.id.unwrap_or_else(|| ui.next_auto_id());
            return ui
                .add_enabled_ui(false, |ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(width, 0.0), Sense::hover());
                    let response = ui.interact(rect, id, Sense::hover());
                    response.widget_info(|| {
                        WidgetInfo::labeled(
                            Role::TextInput,
                            false,
                            self.label.unwrap_or("Text input"),
                        )
                    });
                    ui.ctx().accesskit_node_builder(id, |node| {
                        if let Some(label) = self.label {
                            node.set_label(label);
                        }
                        node.set_disabled();
                        if self.error.is_some() {
                            node.set_invalid(accesskit::Invalid::True);
                        }
                    });
                    support_text(ui, self.helper, self.error, false, &response);
                    response
                })
                .inner;
        }
        // Relax decoration before shrinking the editor in very narrow containers.
        let horizontal_padding = ((width - 24.0) * 0.5).clamp(0.0, 10.0) as i8;
        let content_width = (width - f32::from(horizontal_padding) * 2.0).max(0.0);

        ui.vertical(|ui| {
            if let Some(label) = self.label {
                field_label(ui, label, enabled);
            }

            let border = error_color.unwrap_or(tokens.control_border);
            // Paint the outline inside the measured rect below. Letting `Frame` paint it
            // would add the stroke width to the outer size and overflow the requested width.
            let frame = egui::Frame::new()
                .fill(if enabled {
                    tokens.surface
                } else {
                    tokens.disabled
                })
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(horizontal_padding, 6));

            let inner = frame.show(ui, |ui| {
                ui.set_width(content_width);
                crate::direction::horizontal(ui, |ui| {
                    if let Some(icon) = self.leading_icon.filter(|_| content_width >= 64.0) {
                        ui.label(
                            RichText::new(icon)
                                .size(FIELD_FONT_SIZE + 1.0)
                                .color(tokens.muted_text),
                        );
                    }
                    let mut edit = TextEdit::singleline(self.value)
                        .hint_text(self.hint)
                        .frame(egui::Frame::NONE)
                        .margin(Margin::ZERO)
                        .text_color(if enabled {
                            tokens.text
                        } else {
                            tokens.disabled_text
                        })
                        .desired_width(ui.available_width().max(0.0));
                    if let Some(id) = self.id {
                        edit = edit.id(id);
                    }
                    ui.add(edit)
                })
                .inner
            });
            let response = inner.inner;
            let rect = inner.response.rect;

            if response.has_focus() {
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(8),
                    Stroke::new(2.0, tokens.focus),
                    StrokeKind::Inside,
                );
            } else if error_color.is_some() {
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(8),
                    Stroke::new(1.5, tokens.error),
                    StrokeKind::Inside,
                );
            } else {
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(8),
                    Stroke::new(1.0, border),
                    StrokeKind::Inside,
                );
            }

            let invalid = self.error.is_some();
            if let Some(label) = self.label {
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_label(label);
                    if invalid {
                        node.set_invalid(accesskit::Invalid::True);
                    }
                });
            } else if invalid {
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_invalid(accesskit::Invalid::True);
                });
            }
            if let Some(role) = self.role {
                ui.ctx()
                    .accesskit_node_builder(response.id, |node| node.set_role(role));
            }
            support_text(ui, self.helper, self.error, enabled, &response);
            response
        })
        .inner
    }
}

/// A search field with a leading search glyph.
///
/// It is a [`TextField`] preset: the application still owns the query string and receives the
/// text-edit response.
pub struct SearchField<'a> {
    value: &'a mut String,
    label: Option<&'a str>,
    hint: &'a str,
    enabled: bool,
    id: Option<Id>,
    width: Option<f32>,
}

impl<'a> SearchField<'a> {
    /// Create a search field bound to an application-owned query.
    pub fn new(value: &'a mut String) -> Self {
        Self {
            value,
            label: None,
            hint: "Search…",
            enabled: true,
            id: None,
            width: None,
        }
    }

    /// Visible and accessible label.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Placeholder text. Defaults to `Search…`.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    /// Enable or disable the field.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable ID.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Request a width in logical points.
    pub fn width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.width = Some(width.min(4096.0));
        }
        self
    }

    /// Show the search field.
    pub fn show(self, ui: &mut Ui) -> Response {
        let mut field = TextField::new(self.value)
            .hint(self.hint)
            .enabled(self.enabled)
            .leading_icon("⌕")
            .role(Role::SearchInput);
        if let Some(label) = self.label {
            field = field.label(label);
        }
        if let Some(id) = self.id {
            field = field.id(id);
        }
        if let Some(width) = self.width {
            field = field.width(width);
        }
        field.show(ui)
    }
}
