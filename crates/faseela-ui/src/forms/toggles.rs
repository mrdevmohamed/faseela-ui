//! Checkbox and switch values remain application-owned.
use super::common::support_text;
use super::options::{ControlKind, option_row, option_row_with_control};
use egui::{Id, Response, Ui, Vec2};

const CHECKBOX_SIZE: f32 = 18.0;
const SWITCH_SIZE: Vec2 = Vec2::new(38.0, 22.0);

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
