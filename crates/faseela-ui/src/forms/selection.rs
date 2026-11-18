//! Standalone radio choices, keyboard radio groups, and native combo-box adapters.
use super::common::{FIELD_FONT_SIZE, field_label, support_text};
use super::options::{
    ControlKind, OptionPaint, allocate_option_row, option_row, paint_option_rect,
};
use crate::{
    direction::{self, Step},
    theme,
};
use egui::{Id, Response, RichText, Role, Ui, Vec2, WidgetInfo, accesskit};

const RADIO_SIZE: f32 = 18.0;

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
