//! Wrapping related buttons with caller-owned selection.
use super::SelectionResponse;
use crate::{
    buttons::{Button, ButtonVariant, ControlSize},
    direction,
};
use egui::{Id, Role, Ui, WidgetInfo};

/// A horizontal group of related action buttons with caller-owned selection.
///
/// Clicking a button updates `selected` and returns its index in
/// [`SelectionResponse::activated`]. When the interface is right-to-left the group is laid out
/// from right to left and wraps from the right edge. An out-of-range selection paints no button
/// as selected and is not silently changed; clicking selects a valid index again.
///
/// ```
/// use faseela_ui::navigation::ButtonGroup;
///
/// egui::__run_test_ui(|ui| {
///     let mut selected = 0;
///     let result = ButtonGroup::new(&["Day", "Week", "Month"], &mut selected).show(ui);
///     let _ = result.activated;
/// });
/// ```
pub struct ButtonGroup<'a> {
    labels: &'a [&'a str],
    selected: &'a mut usize,
    enabled: bool,
    id: Option<Id>,
    size: ControlSize,
    variant: ButtonVariant,
    accessible_label: &'a str,
}

impl<'a> ButtonGroup<'a> {
    /// Create a group bound to an application-owned selected index.
    pub fn new(labels: &'a [&'a str], selected: &'a mut usize) -> Self {
        Self {
            labels,
            selected,
            enabled: true,
            id: None,
            size: ControlSize::Medium,
            variant: ButtonVariant::Outline,
            accessible_label: "Button group",
        }
    }

    /// Enable or disable the whole group.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable ID for the group and its buttons.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Control size. Defaults to [`ControlSize::Medium`].
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Button variant. Defaults to [`ButtonVariant::Outline`].
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Accessible name for the group container.
    pub fn accessible_label(mut self, label: &'a str) -> Self {
        self.accessible_label = label;
        self
    }

    /// Show the group.
    pub fn show(self, ui: &mut Ui) -> SelectionResponse {
        if !self.enabled {
            return ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner;
        }
        self.show_in(ui)
    }

    fn show_in(self, ui: &mut Ui) -> SelectionResponse {
        let enabled = ui.is_enabled();
        let id = match self.id {
            Some(id) => id,
            None => {
                let id = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);
                id
            }
        };
        let labels = self.labels;
        let size = self.size;
        let variant = self.variant;
        let mut selected = *self.selected;
        let mut activated = None;

        let group = ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
            ui.with_layout(direction::row(ui), |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (index, label) in labels.iter().enumerate() {
                        let is_selected = selected == index;
                        let response = Button::new(*label)
                            .variant(variant)
                            .size(size)
                            .selected(is_selected)
                            .id(id.with(("group-item", index)))
                            .show(ui);
                        if response.clicked() {
                            selected = index;
                            activated = Some(index);
                        }
                    }
                })
            })
        });
        let mut response = group.inner.inner.response;
        if *self.selected != selected {
            response.mark_changed();
        }
        response.widget_info(|| WidgetInfo::labeled(Role::Toolbar, enabled, self.accessible_label));
        *self.selected = selected;
        SelectionResponse {
            response,
            activated,
        }
    }
}
