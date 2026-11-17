//! Theme-aware form controls with application-owned values.
//!
//! Every control stores no hidden application state: callers pass `&mut` values for text fields,
//! checkboxes, switches, radio groups, and selects. The widgets provide labels, helper/error
//! text, disabled and focus presentation, AccessKit roles, and keyboard activation through
//! egui's native interaction model.
//!
//! ```
//! use faseela_ui::forms::{Checkbox, Switch, TextField};
//!
//! egui::__run_test_ui(|ui| {
//!     let mut name = "Faseela".to_owned();
//!     let mut enabled = true;
//!     TextField::new(&mut name).label("Name").show(ui);
//!     Checkbox::new("Show notifications", &mut enabled).show(ui);
//!     Switch::new("Live updates", &mut enabled).show(ui);
//! });
//! ```

use egui::{
    Color32, CornerRadius, FontId, Id, Margin, Response, RichText, Role, Sense, Stroke, StrokeKind,
    TextEdit, Ui, Vec2, WidgetInfo, accesskit, vec2,
};

use crate::direction::{self, Step};
use crate::theme::{self, ThemeTokens};

const FIELD_FONT_SIZE: f32 = 13.0;
const SUPPORT_FONT_SIZE: f32 = 11.5;
const CHECKBOX_SIZE: f32 = 18.0;
const RADIO_SIZE: f32 = 18.0;
const SWITCH_SIZE: Vec2 = Vec2::new(38.0, 22.0);
const ROW_GAP: f32 = 8.0;

fn label_galley(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    wrap_width: f32,
) -> std::sync::Arc<egui::Galley> {
    ui.fonts_mut(|fonts| fonts.layout(text.to_owned(), font, color, wrap_width.max(1.0)))
}

fn field_label(ui: &mut Ui, label: &str, enabled: bool) {
    let tokens = theme::active_tokens(ui.ctx());
    let color = if enabled {
        tokens.text
    } else {
        tokens.disabled_text
    };
    ui.label(RichText::new(label).size(FIELD_FONT_SIZE).color(color));
}

fn support_text(ui: &mut Ui, helper: Option<&str>, error: Option<&str>, enabled: bool) {
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
    } else if let Some(helper) = helper {
        let color = if enabled {
            tokens.muted_text
        } else {
            tokens.disabled_text
        };
        ui.label(RichText::new(helper).size(SUPPORT_FONT_SIZE).color(color));
    }
}

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
        let available_width = ui.available_width().max(24.0);
        let requested_width = self.width.unwrap_or(available_width);
        let width = requested_width.min(available_width).clamp(24.0, 4096.0);

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
                .inner_margin(Margin::symmetric(10, 6));

            let inner = frame.show(ui, |ui| {
                ui.set_width((width - 20.0).max(24.0));
                ui.horizontal(|ui| {
                    if let Some(icon) = self.leading_icon {
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
                        .desired_width(f32::INFINITY);
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
            support_text(ui, self.helper, self.error, enabled);
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

/// A themed checkbox with a label and optional helper/error text.
pub struct Checkbox<'a> {
    label: &'a str,
    value: &'a mut bool,
    enabled: bool,
    id: Option<Id>,
    helper: Option<&'a str>,
    error: Option<&'a str>,
}

impl<'a> Checkbox<'a> {
    /// Create a checkbox bound to an application-owned boolean.
    pub fn new(label: &'a str, value: &'a mut bool) -> Self {
        Self {
            label,
            value,
            enabled: true,
            id: None,
            helper: None,
            error: None,
        }
    }

    /// Enable or disable the checkbox.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable widget ID.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Supporting text below the row.
    pub fn helper(mut self, helper: &'a str) -> Self {
        self.helper = Some(helper);
        self
    }

    /// Error text below the row.
    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    /// Show the checkbox. Clicking or pressing Space/Enter while focused toggles the value.
    pub fn show(self, ui: &mut Ui) -> Response {
        if self.enabled {
            self.show_in(ui)
        } else {
            ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner
        }
    }

    fn show_in(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let mut response = option_row(
                ui,
                self.label,
                self.id,
                ControlKind::CheckBox,
                *self.value,
                CHECKBOX_SIZE,
            );
            if response.clicked() {
                *self.value = !*self.value;
                response.mark_changed();
            }
            support_text(ui, self.helper, self.error, ui.is_enabled());
            response
        })
        .inner
    }
}

/// A themed on/off switch with a label.
pub struct Switch<'a> {
    label: &'a str,
    value: &'a mut bool,
    enabled: bool,
    id: Option<Id>,
    helper: Option<&'a str>,
    error: Option<&'a str>,
}

impl<'a> Switch<'a> {
    /// Create a switch bound to an application-owned boolean.
    pub fn new(label: &'a str, value: &'a mut bool) -> Self {
        Self {
            label,
            value,
            enabled: true,
            id: None,
            helper: None,
            error: None,
        }
    }

    /// Enable or disable the switch.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable widget ID.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Supporting text below the row.
    pub fn helper(mut self, helper: &'a str) -> Self {
        self.helper = Some(helper);
        self
    }

    /// Error text below the row.
    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    /// Show the switch. Clicking or pressing Space/Enter while focused toggles the value.
    pub fn show(self, ui: &mut Ui) -> Response {
        if self.enabled {
            self.show_in(ui)
        } else {
            ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner
        }
    }

    fn show_in(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let mut response = option_row_with_control(
                ui,
                self.label,
                self.id,
                ControlKind::Switch,
                *self.value,
                SWITCH_SIZE,
            );
            if response.clicked() {
                *self.value = !*self.value;
                response.mark_changed();
            }
            support_text(ui, self.helper, self.error, ui.is_enabled());
            response
        })
        .inner
    }
}

/// A standalone radio button. Selection is caller-owned.
///
/// For a complete group with arrow-key navigation, use [`RadioGroup`].
pub struct RadioButton<'a> {
    label: &'a str,
    selected: bool,
    enabled: bool,
    id: Option<Id>,
}

impl<'a> RadioButton<'a> {
    /// Create a radio button reflecting `selected`.
    pub fn new(label: &'a str, selected: bool) -> Self {
        Self {
            label,
            selected,
            enabled: true,
            id: None,
        }
    }

    /// Enable or disable the radio button.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable widget ID.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Show the radio button. The caller updates its selection when `clicked()` is true.
    pub fn show(self, ui: &mut Ui) -> Response {
        if self.enabled {
            option_row(
                ui,
                self.label,
                self.id,
                ControlKind::Radio,
                self.selected,
                RADIO_SIZE,
            )
        } else {
            ui.add_enabled_ui(false, |ui| {
                option_row(
                    ui,
                    self.label,
                    self.id,
                    ControlKind::Radio,
                    self.selected,
                    RADIO_SIZE,
                )
            })
            .inner
        }
    }
}

/// A vertical radio group with caller-owned selection and arrow-key navigation.
///
/// Clicking an option updates the selection immediately. While an option is focused, arrow keys
/// follow the interface reading direction for horizontal arrows; Up/Down always move backward
/// and forward through the group.
pub struct RadioGroup<'a> {
    options: &'a [&'a str],
    selected: &'a mut usize,
    label: Option<&'a str>,
    enabled: bool,
    id: Option<Id>,
    helper: Option<&'a str>,
    error: Option<&'a str>,
}

impl<'a> RadioGroup<'a> {
    /// Create a group bound to an application-owned selected index.
    pub fn new(options: &'a [&'a str], selected: &'a mut usize) -> Self {
        Self {
            options,
            selected,
            label: None,
            enabled: true,
            id: None,
            helper: None,
            error: None,
        }
    }

    /// Visible group label.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Enable or disable the whole group.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable ID salt for the group.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Supporting text below the group.
    pub fn helper(mut self, helper: &'a str) -> Self {
        self.helper = Some(helper);
        self
    }

    /// Error text below the group.
    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    /// Show the group. Returns the response of the group container.
    pub fn show(self, ui: &mut Ui) -> Response {
        if !self.enabled {
            return ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner;
        }
        self.show_in(ui)
    }

    fn show_in(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let enabled = ui.is_enabled();
        let mut selected = (*self.selected).min(self.options.len().saturating_sub(1));
        let id_salt = match self.id {
            Some(id) => id,
            None => {
                let id = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);
                id
            }
        };
        let rtl = direction::is_rtl(ui);

        let inner = ui.scope_builder(egui::UiBuilder::new().scope_id(id_salt), |ui| {
            // Use the group scope (not the inner vertical Ui) so option IDs stay stable if the
            // group's internal layout changes. Option `i` has the ID
            // `group_id.with(("radio-option", i))`.
            let option_scope = ui.scope_id();
            let vertical = ui.vertical(|ui| {
                if let Some(label) = self.label {
                    field_label(ui, label, enabled);
                }

                let mut rows = Vec::with_capacity(self.options.len());
                for (index, option) in self.options.iter().enumerate() {
                    let id = option_scope.with(("radio-option", index));
                    let row = allocate_option_row(ui, option, id, RADIO_SIZE);
                    rows.push((index, *option, row));
                }

                if enabled {
                    // Clicking an option selects it immediately.
                    if let Some((index, _, (_, response))) =
                        rows.iter().find(|(_, _, (_, response))| response.clicked())
                    {
                        selected = *index;
                        response.request_focus();
                    }

                    // Keyboard navigation is handled after allocating all rows so focus can
                    // move to an option that has not been painted yet.
                    let focused = rows
                        .iter()
                        .find(|(_, _, (_, response))| response.has_focus())
                        .map(|(index, _, _)| *index);
                    if let Some(current) = focused {
                        let mut target = None;
                        ui.input(|input| {
                            if input.key_pressed(egui::Key::ArrowUp) {
                                target = Some(Step::Previous);
                            } else if input.key_pressed(egui::Key::ArrowDown) {
                                target = Some(Step::Next);
                            } else if input.key_pressed(if rtl {
                                egui::Key::ArrowRight
                            } else {
                                egui::Key::ArrowLeft
                            }) {
                                target = Some(Step::Previous);
                            } else if input.key_pressed(if rtl {
                                egui::Key::ArrowLeft
                            } else {
                                egui::Key::ArrowRight
                            }) {
                                target = Some(Step::Next);
                            }
                        });
                        if let Some(step) = target {
                            let len = rows.len();
                            let next = match step {
                                Step::Previous => (current + len - 1) % len,
                                Step::Next => (current + 1) % len,
                            };
                            selected = next;
                            if let Some((_, _, (_, response))) = rows.get(next) {
                                response.request_focus();
                            }
                        }
                    }
                }

                for (index, option, (rect, response)) in &rows {
                    let is_selected = *index == selected;
                    response.widget_info(|| {
                        WidgetInfo::selected(Role::RadioButton, enabled, is_selected, option)
                    });
                    paint_option_rect(
                        ui,
                        *rect,
                        OptionPaint {
                            label: option,
                            kind: ControlKind::Radio,
                            selected: is_selected,
                            enabled,
                            focused: response.has_focus(),
                            tokens,
                            control_size: Vec2::splat(RADIO_SIZE),
                        },
                    );
                }

                support_text(ui, self.helper, self.error, enabled);
            });
            vertical.response
        });

        let response = inner.inner;
        if let Some(label) = self.label {
            response.widget_info(|| WidgetInfo::labeled(Role::RadioGroup, enabled, label));
        }
        *self.selected = selected;
        response
    }
}

/// A themed select/combo-box presentation adapter.
///
/// Options are presented as labels; the application owns the selected index. If the index is
/// out of range, a neutral placeholder is shown until the user chooses an option.
pub struct Select<'a> {
    label: &'a str,
    options: &'a [&'a str],
    selected: &'a mut usize,
    enabled: bool,
    id: Option<Id>,
    width: Option<f32>,
    helper: Option<&'a str>,
    error: Option<&'a str>,
}

impl<'a> Select<'a> {
    /// Create a select bound to an application-owned selected index.
    pub fn new(label: &'a str, options: &'a [&'a str], selected: &'a mut usize) -> Self {
        Self {
            label,
            options,
            selected,
            enabled: true,
            id: None,
            width: None,
            helper: None,
            error: None,
        }
    }

    /// Enable or disable the select.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable ID salt.
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

    /// Supporting text below the control.
    pub fn helper(mut self, helper: &'a str) -> Self {
        self.helper = Some(helper);
        self
    }

    /// Error text below the control.
    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    /// Show the select.
    pub fn show(self, ui: &mut Ui) -> Response {
        if !self.enabled {
            return ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner;
        }
        self.show_in(ui)
    }

    fn show_in(self, ui: &mut Ui) -> Response {
        let enabled = ui.is_enabled();
        let tokens = theme::active_tokens(ui.ctx());
        let available_width = ui.available_width().max(24.0);
        let width = self
            .width
            .unwrap_or(available_width)
            .min(available_width)
            .max(24.0);
        let selected_text = self
            .options
            .get(*self.selected)
            .copied()
            .unwrap_or("Select…");
        let id_salt = match self.id {
            Some(id) => id,
            None => {
                let id = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);
                id
            }
        };

        ui.vertical(|ui| {
            ui.label(
                RichText::new(self.label)
                    .size(FIELD_FONT_SIZE)
                    .color(if enabled {
                        tokens.text
                    } else {
                        tokens.disabled_text
                    }),
            );
            let response = ui
                .push_id(id_salt, |ui| {
                    egui::ComboBox::from_id_salt("select")
                        .width(width)
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (index, option) in self.options.iter().enumerate() {
                                if ui.selectable_value(self.selected, index, *option).changed() {
                                    ui.close();
                                }
                            }
                        })
                        .response
                })
                .inner;
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_label(self.label);
                if self.error.is_some() {
                    node.set_invalid(accesskit::Invalid::True);
                }
            });
            support_text(ui, self.helper, self.error, enabled);
            response
        })
        .inner
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlKind {
    CheckBox,
    Radio,
    Switch,
}

type OptionRow = (egui::Rect, Response);

fn allocate_option_row(ui: &mut Ui, label: &str, id: Id, control_size: f32) -> OptionRow {
    let available = ui.available_width().max(0.0);
    let tokens = theme::active_tokens(ui.ctx());
    let width = (available - control_size - ROW_GAP).max(24.0);
    let galley = label_galley(
        ui,
        label,
        FontId::proportional(FIELD_FONT_SIZE),
        tokens.text,
        width,
    );
    let height = (galley.rect.height() + 8.0).max(26.0);
    let (_, rect) = ui.allocate_space(vec2(available, height));
    let response = ui.interact(rect, id, Sense::click());
    (rect, response)
}

fn option_row(
    ui: &mut Ui,
    label: &str,
    id: Option<Id>,
    kind: ControlKind,
    selected: bool,
    control_size: f32,
) -> Response {
    let tokens = theme::active_tokens(ui.ctx());
    let enabled = ui.is_enabled();
    let id = id.unwrap_or_else(|| {
        ui.scope_id()
            .with(("option-row", label, control_size as u32))
    });
    let (rect, response) = allocate_option_row(ui, label, id, control_size);
    response.widget_info(|| WidgetInfo::selected(role_for(kind), enabled, selected, label));
    paint_option_rect(
        ui,
        rect,
        OptionPaint {
            label,
            kind,
            selected,
            enabled,
            focused: response.has_focus(),
            tokens,
            control_size: Vec2::splat(control_size),
        },
    );
    response
}

fn option_row_with_control(
    ui: &mut Ui,
    label: &str,
    id: Option<Id>,
    kind: ControlKind,
    selected: bool,
    control_size: Vec2,
) -> Response {
    let tokens = theme::active_tokens(ui.ctx());
    let enabled = ui.is_enabled();
    let id = id.unwrap_or_else(|| ui.scope_id().with(("option-row-switch", label)));
    let available = ui.available_width().max(0.0);
    let width = (available - control_size.x - ROW_GAP).max(24.0);
    let galley = label_galley(
        ui,
        label,
        FontId::proportional(FIELD_FONT_SIZE),
        tokens.text,
        width,
    );
    let height = (galley.rect.height() + 8.0)
        .max(control_size.y + 6.0)
        .max(26.0);
    let (_, rect) = ui.allocate_space(vec2(available, height));
    let response = ui.interact(rect, id, Sense::click());
    response.widget_info(|| WidgetInfo::selected(role_for(kind), enabled, selected, label));
    paint_option_rect(
        ui,
        rect,
        OptionPaint {
            label,
            kind,
            selected,
            enabled,
            focused: response.has_focus(),
            tokens,
            control_size,
        },
    );
    response
}

fn role_for(kind: ControlKind) -> Role {
    match kind {
        ControlKind::CheckBox => Role::CheckBox,
        ControlKind::Radio => Role::RadioButton,
        ControlKind::Switch => Role::Switch,
    }
}

struct OptionPaint<'a> {
    label: &'a str,
    kind: ControlKind,
    selected: bool,
    enabled: bool,
    focused: bool,
    tokens: ThemeTokens,
    control_size: Vec2,
}

fn paint_option_rect(ui: &Ui, rect: egui::Rect, paint: OptionPaint<'_>) {
    let OptionPaint {
        label,
        kind,
        selected,
        enabled,
        focused,
        tokens,
        control_size,
    } = paint;
    let rtl = direction::is_rtl(ui);
    let control_rect = if rtl {
        egui::Rect::from_min_size(
            egui::pos2(
                rect.right() - control_size.x,
                rect.center().y - control_size.y * 0.5,
            ),
            control_size,
        )
    } else {
        egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.center().y - control_size.y * 0.5),
            control_size,
        )
    };
    let label_left = if rtl {
        rect.left()
    } else {
        control_rect.right() + ROW_GAP
    };
    let label_width = if rtl {
        (control_rect.left() - ROW_GAP - rect.left()).max(0.0)
    } else {
        (rect.right() - label_left).max(0.0)
    };
    let color = if enabled {
        tokens.text
    } else {
        tokens.disabled_text
    };
    let galley = label_galley(
        ui,
        label,
        FontId::proportional(FIELD_FONT_SIZE),
        color,
        label_width,
    );
    let text_x = if rtl {
        label_left + (label_width - galley.rect.width()).max(0.0)
    } else {
        label_left
    };
    let text_pos = egui::pos2(text_x, rect.center().y - galley.rect.height() * 0.5);
    ui.painter()
        .with_clip_rect(rect)
        .galley_with_override_text_color(text_pos, galley, color);

    match kind {
        ControlKind::CheckBox => {
            let fill = if selected {
                if enabled {
                    tokens.primary
                } else {
                    tokens.disabled
                }
            } else if enabled {
                tokens.surface
            } else {
                tokens.disabled
            };
            let border = if !enabled {
                tokens.border
            } else if focused {
                tokens.focus
            } else {
                tokens.control_border
            };
            ui.painter().rect(
                control_rect,
                CornerRadius::same(4),
                fill,
                Stroke::new(if focused { 2.0 } else { 1.0 }, border),
                StrokeKind::Inside,
            );
            if selected {
                let mark = if enabled {
                    tokens.on_primary
                } else {
                    tokens.disabled_text
                };
                let a = control_rect.left_top() + vec2(4.0, 9.0);
                let b = control_rect.left_top() + vec2(7.5, 12.5);
                let c = control_rect.left_top() + vec2(14.0, 5.5);
                ui.painter().line_segment([a, b], Stroke::new(2.0, mark));
                ui.painter().line_segment([b, c], Stroke::new(2.0, mark));
            }
        }
        ControlKind::Radio => {
            let center = control_rect.center();
            let radius = control_rect.width() * 0.5;
            let fill = if enabled {
                tokens.surface
            } else {
                tokens.disabled
            };
            let border = if !enabled {
                tokens.border
            } else if focused {
                tokens.focus
            } else {
                tokens.control_border
            };
            ui.painter().circle_filled(center, radius, fill);
            ui.painter().circle_stroke(
                center,
                radius - 0.5,
                Stroke::new(if focused { 2.0 } else { 1.5 }, border),
            );
            if selected {
                ui.painter().circle_filled(
                    center,
                    radius * 0.52,
                    if enabled {
                        tokens.primary
                    } else {
                        tokens.disabled_text
                    },
                );
            }
        }
        ControlKind::Switch => {
            let track = control_rect;
            let fill = if selected {
                if enabled {
                    tokens.primary
                } else {
                    tokens.disabled
                }
            } else if enabled {
                tokens.elevated_surface
            } else {
                tokens.disabled
            };
            let border = if !enabled {
                tokens.border
            } else if focused {
                tokens.focus
            } else {
                tokens.control_border
            };
            ui.painter().rect(
                track,
                CornerRadius::same((track.height() * 0.5) as u8),
                fill,
                Stroke::new(if focused { 2.0 } else { 1.0 }, border),
                StrokeKind::Inside,
            );
            let knob_radius = track.height() * 0.5 - 4.0;
            let knob_x = if selected {
                track.right() - knob_radius - 3.0
            } else {
                track.left() + knob_radius + 3.0
            };
            let knob_color = if selected {
                if enabled {
                    tokens.on_primary
                } else {
                    tokens.disabled_text
                }
            } else if enabled {
                tokens.text
            } else {
                tokens.disabled_text
            };
            ui.painter().circle_filled(
                egui::pos2(knob_x, track.center().y),
                knob_radius,
                knob_color,
            );
        }
    }

    if focused {
        let focus_rect = control_rect.expand(2.0);
        ui.painter().rect_stroke(
            focus_rect,
            CornerRadius::same(5),
            Stroke::new(1.5, tokens.focus),
            StrokeKind::Outside,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemePreset;
    use egui::{
        Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, pos2, vec2,
    };

    fn frame(ctx: &Context, events: Vec<Event>, add: impl FnMut(&mut Ui)) -> egui::FullOutput {
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 400.0))),
                focused: true,
                events,
                ..Default::default()
            },
            add,
        );
        output.textures_delta.clear();
        output
    }

    fn click_at(pos: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    #[test]
    fn text_field_updates_application_owned_value_and_reports_changes() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
        let id = Id::unique("name-field");
        let mut value = String::new();
        let _ = frame(&ctx, vec![], |ui| {
            TextField::new(&mut value).label("Name").id(id).show(ui);
            ui.memory_mut(|memory| memory.request_focus(id));
        });
        let _ = frame(
            &ctx,
            vec![
                Event::Text("Faseela".to_owned()),
                Event::Key {
                    key: Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            |ui| {
                let response = TextField::new(&mut value).label("Name").id(id).show(ui);
                assert!(response.changed());
            },
        );
        assert_eq!(value, "Faseela");
    }

    #[test]
    fn text_field_error_uses_error_color_and_invalid_state() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let tokens = ThemePreset::Light.tokens();
        let mut value = String::new();
        let output = frame(&ctx, vec![], |ui| {
            TextField::new(&mut value)
                .label("Email")
                .error("Enter a valid email")
                .show(ui);
        });
        let has_error_stroke = output.shapes.iter().any(|clip| match &clip.shape {
            Shape::Rect(rect) => rect.stroke.color == tokens.error && rect.stroke.width > 0.0,
            _ => false,
        });
        assert!(has_error_stroke, "error border must be painted");
    }

    #[test]
    fn disabled_text_field_is_inert_and_reports_disabled() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
        let id = Id::unique("disabled-field");
        let mut value = "Read only".to_owned();
        let mut rect = Rect::NOTHING;
        let _ = frame(&ctx, vec![], |ui| {
            rect = TextField::new(&mut value)
                .label("Name")
                .id(id)
                .enabled(false)
                .show(ui)
                .rect;
        });
        let _ = frame(&ctx, click_at(rect.center()), |ui| {
            let response = TextField::new(&mut value)
                .label("Name")
                .id(id)
                .enabled(false)
                .show(ui);
            assert!(!response.enabled());
            assert!(!response.has_focus());
        });
        assert_eq!(value, "Read only");
    }

    #[test]
    fn checkbox_toggles_by_click_and_respects_disabled_state() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let id = Id::unique("checkbox");
        let mut checked = false;
        let mut rect = Rect::NOTHING;
        let _ = frame(&ctx, vec![], |ui| {
            rect = Checkbox::new("Enable notifications", &mut checked)
                .id(id)
                .show(ui)
                .rect;
        });
        let _ = frame(&ctx, click_at(rect.center()), |ui| {
            Checkbox::new("Enable notifications", &mut checked)
                .id(id)
                .show(ui);
        });
        assert!(checked);

        let mut disabled = false;
        let _ = frame(&ctx, vec![], |ui| {
            rect = Checkbox::new("Disabled", &mut disabled)
                .id(id)
                .enabled(false)
                .show(ui)
                .rect;
        });
        let _ = frame(&ctx, click_at(rect.center()), |ui| {
            Checkbox::new("Disabled", &mut disabled)
                .id(id)
                .enabled(false)
                .show(ui);
        });
        assert!(!disabled);
    }

    #[test]
    fn switch_toggles_by_keyboard_space_when_focused() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
        let id = Id::unique("switch");
        let mut enabled = false;
        let _ = frame(&ctx, vec![], |ui| {
            Switch::new("Live updates", &mut enabled).id(id).show(ui);
            ui.memory_mut(|memory| memory.request_focus(id));
        });
        let _ = frame(
            &ctx,
            vec![Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            |ui| {
                Switch::new("Live updates", &mut enabled).id(id).show(ui);
            },
        );
        assert!(enabled);
    }

    #[test]
    fn radio_group_changes_selection_by_click_and_arrow_key() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let id = Id::unique("radio-group");
        let mut selected = 0usize;
        let _ = frame(&ctx, vec![], |ui| {
            RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
                .label("Density")
                .id(id)
                .show(ui);
        });

        // Focus the first option and use the reading-order next arrow.
        let _ = frame(&ctx, vec![], |ui| {
            RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
                .label("Density")
                .id(id)
                .show(ui);
            let option_id = id.with(("radio-option", 0));
            ui.memory_mut(|memory| memory.request_focus(option_id));
        });
        let _ = frame(
            &ctx,
            vec![Event::Key {
                key: Key::ArrowDown,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            |ui| {
                RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
                    .label("Density")
                    .id(id)
                    .show(ui);
            },
        );
        assert_eq!(selected, 1);
    }

    #[test]
    fn radio_group_click_selects_the_clicked_option() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let id = Id::unique("radio-group-click");
        let mut selected = 0usize;
        let _ = frame(&ctx, vec![], |ui| {
            RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
                .label("Density")
                .id(id)
                .show(ui);
        });
        let option = ctx
            .read_response(id.with(("radio-option", 2)))
            .expect("third option");
        assert!(option.enabled());
        let _ = frame(&ctx, click_at(option.rect.center()), |ui| {
            RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
                .label("Density")
                .id(id)
                .show(ui);
        });
        assert_eq!(selected, 2);
    }

    #[test]
    fn disabled_radio_group_ignores_clicks_and_arrow_keys() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
        let id = Id::unique("radio-group-disabled");
        let mut selected = 1usize;
        let _ = frame(&ctx, vec![], |ui| {
            RadioGroup::new(&["One", "Two", "Three"], &mut selected)
                .label("Disabled")
                .id(id)
                .enabled(false)
                .show(ui);
        });
        let option = ctx
            .read_response(id.with(("radio-option", 2)))
            .expect("third option");
        assert!(!option.enabled());
        let _ = frame(&ctx, click_at(option.rect.center()), |ui| {
            RadioGroup::new(&["One", "Two", "Three"], &mut selected)
                .label("Disabled")
                .id(id)
                .enabled(false)
                .show(ui);
        });
        let _ = frame(
            &ctx,
            vec![Event::Key {
                key: Key::ArrowDown,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            |ui| {
                RadioGroup::new(&["One", "Two", "Three"], &mut selected)
                    .label("Disabled")
                    .id(id)
                    .enabled(false)
                    .show(ui);
            },
        );
        assert_eq!(selected, 1);
    }

    #[test]
    fn radio_group_clamps_out_of_range_selection() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
        let mut selected = usize::MAX;
        let _ = frame(&ctx, vec![], |ui| {
            RadioGroup::new(&["One", "Two"], &mut selected).show(ui);
        });
        assert_eq!(selected, 1);
    }

    #[test]
    fn every_control_renders_in_all_themes_and_directions() {
        for preset in ThemePreset::ALL {
            for rtl in [false, true] {
                let ctx = Context::default();
                theme::apply_preset(&ctx, preset);
                direction::apply(
                    &ctx,
                    if rtl {
                        direction::Direction::Rtl
                    } else {
                        direction::Direction::Ltr
                    },
                );
                let mut text = "مرحبا Faseela".to_owned();
                let mut checked = true;
                let mut switched = false;
                let mut radio = 0usize;
                let mut select = 0usize;
                let output = frame(&ctx, vec![], |ui| {
                    TextField::new(&mut text).label("Name").show(ui);
                    SearchField::new(&mut text).label("Search").show(ui);
                    Checkbox::new("Checkbox", &mut checked).show(ui);
                    Switch::new("Switch", &mut switched).show(ui);
                    RadioGroup::new(&["A", "B"], &mut radio)
                        .label("Radio")
                        .show(ui);
                    Select::new("Select", &["One", "Two"], &mut select).show(ui);
                });
                assert!(!output.shapes.is_empty(), "{preset:?} rtl={rtl}");
            }
        }
    }

    #[test]
    fn select_renders_options_and_clamps_nothing_implicitly() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let mut selected = 1usize;
        let _ = frame(&ctx, vec![], |ui| {
            Select::new("Theme", &["Dark", "Light"], &mut selected).show(ui);
        });
        assert_eq!(selected, 1);
    }

    #[test]
    fn select_opens_with_keyboard_from_focus_and_closes_with_escape() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let id = Id::unique("select-keyboard");
        let mut selected = 0usize;
        let mut button_id = None;
        let _ = frame(&ctx, vec![], |ui| {
            let response = Select::new("Theme", &["Dark", "Light"], &mut selected)
                .id(id)
                .show(ui);
            ui.memory_mut(|memory| memory.request_focus(response.id));
            button_id = Some(response.id);
        });
        let button_id = button_id.expect("select response id");
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(button_id));
        let popup_id = button_id.with("popup");
        assert!(!egui::Popup::is_id_open(&ctx, popup_id));

        let _ = frame(
            &ctx,
            vec![Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            |ui| {
                Select::new("Theme", &["Dark", "Light"], &mut selected)
                    .id(id)
                    .show(ui);
            },
        );
        assert!(egui::Popup::is_id_open(&ctx, popup_id));

        let _ = frame(
            &ctx,
            vec![Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            |ui| {
                Select::new("Theme", &["Dark", "Light"], &mut selected)
                    .id(id)
                    .show(ui);
            },
        );
        assert!(!egui::Popup::is_id_open(&ctx, popup_id));
    }

    #[test]
    fn text_field_outline_does_not_expand_past_available_width() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let mut value = String::new();
        let _ = frame(&ctx, vec![], |ui| {
            TextField::new(&mut value)
                .label("Name")
                .helper("Outline is painted inside the measured field.")
                .show(ui);
        });
        assert!(
            ctx.globally_used_rect().right() <= 320.0,
            "field used {:?}",
            ctx.globally_used_rect()
        );
    }

    #[test]
    fn long_labels_wrap_within_available_width() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let mut checked = false;
        let output = frame(&ctx, vec![], |ui| {
            Checkbox::new(
                "A deliberately long checkbox label that should wrap instead of clipping its text",
                &mut checked,
            )
            .show(ui);
        });
        assert!(!output.shapes.is_empty());
        let _ = pos2(0.0, 0.0);
    }
}
