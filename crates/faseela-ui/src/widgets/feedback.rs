//! Determinate and indeterminate progress, status rows, and empty states.
use super::BadgeTone;
use crate::{
    buttons::{Button as ThemedButton, ButtonVariant},
    direction, theme,
};
use egui::{CornerRadius, InnerResponse, Response, RichText, Sense, Stroke, Ui, WidgetInfo, vec2};

/// A determinate progress bar with an optional visible and accessible label.
///
/// ```
/// use faseela_ui::widgets::ProgressBar;
///
/// egui::__run_test_ui(|ui| {
///     ProgressBar::new(0.65).label("Uploading").show_value(true).show(ui);
/// });
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressBar<'a> {
    value: f32,
    label: Option<&'a str>,
    show_value: bool,
    width: Option<f32>,
}

impl<'a> ProgressBar<'a> {
    /// Create a bar for `value` in `0.0..=1.0`. Non-finite values become zero.
    pub fn new(value: f32) -> Self {
        Self {
            value: if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            },
            label: None,
            show_value: false,
            width: None,
        }
    }

    /// Visible and accessible label.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Show the rounded percentage beside the bar.
    pub fn show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }

    /// Request a width in logical points, clamped to the available width.
    pub fn width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.width = Some(width.min(4096.0));
        }
        self
    }

    /// Show the bar and return its response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let percent = (self.value * 100.0).round() as u32;
        let label = self
            .label
            .map(|label| format!("{label}: {percent}%"))
            .unwrap_or_else(|| format!("{percent}%"));
        ui.vertical(|ui| {
            if self.label.is_some() || self.show_value {
                direction::horizontal(ui, |ui| {
                    if let Some(label) = self.label {
                        ui.label(RichText::new(label).size(12.0).color(tokens.text));
                    }
                    if self.show_value {
                        ui.with_layout(direction::trailing(ui), |ui| {
                            ui.label(
                                RichText::new(format!("{percent}%"))
                                    .size(12.0)
                                    .color(tokens.muted_text),
                            );
                        });
                    }
                });
            }
            let available = ui.available_width().max(0.0);
            let width = self.width.unwrap_or(available).min(available);
            let (rect, response) = ui.allocate_exact_size(vec2(width, 8.0), Sense::hover());
            let track = tokens.elevated_surface;
            let radius = CornerRadius::same(4);
            ui.painter().rect_filled(rect, radius, track);
            ui.painter().rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, tokens.control_border),
                egui::StrokeKind::Inside,
            );
            let fill_width = (rect.width() * self.value).max(0.0);
            if fill_width > 0.0 {
                let start = if direction::is_rtl(ui) {
                    egui::pos2(rect.right() - fill_width, rect.top())
                } else {
                    rect.min
                };
                let fill = egui::Rect::from_min_size(start, vec2(fill_width, rect.height()))
                    .intersect(rect);
                ui.painter().rect_filled(fill, radius, tokens.primary);
            }
            response
                .widget_info(|| WidgetInfo::labeled(egui::Role::ProgressIndicator, true, &label));
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_numeric_value(f64::from(percent));
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value(100.0);
            });
            response
        })
        .inner
    }
}

/// An indeterminate loading indicator (spinner) with a visible accessible label.
///
/// ```
/// use faseela_ui::widgets::LoadingIndicator;
///
/// egui::__run_test_ui(|ui| {
///     LoadingIndicator::new().label("Loading projects").show(ui);
/// });
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoadingIndicator<'a> {
    label: Option<&'a str>,
    size: f32,
}

impl<'a> Default for LoadingIndicator<'a> {
    fn default() -> Self {
        Self {
            label: None,
            size: 18.0,
        }
    }
}

impl<'a> LoadingIndicator<'a> {
    /// Create a loading indicator without a visible label.
    pub fn new() -> Self {
        Self::default()
    }

    /// Visible and accessible label shown to the reading end of the spinner.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Diameter in logical points, clamped to `8.0..=96.0`.
    pub fn size(mut self, size: f32) -> Self {
        if size.is_finite() {
            self.size = size.clamp(8.0, 96.0);
        }
        self
    }

    /// Show the indicator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let accessible = self.label.unwrap_or("Loading");
        let height = self.size.max(ui.spacing().interact_size.y);
        let width = ui.available_width().max(0.0);
        ui.allocate_ui_with_layout(vec2(width, height), direction::row(ui), |ui| {
            let response = ui.add(egui::Spinner::new().size(self.size).color(tokens.primary));
            response.widget_info(|| {
                WidgetInfo::labeled(egui::Role::ProgressIndicator, true, accessible)
            });
            if let Some(label) = self.label {
                ui.label(RichText::new(label).size(12.0).color(tokens.text));
            }
            response
        })
        .inner
    }
}

/// A compact status row: a semantic dot followed by readable text.
///
/// ```
/// use faseela_ui::widgets::{BadgeTone, StatusIndicator};
///
/// egui::__run_test_ui(|ui| {
///     StatusIndicator::new("Connected", BadgeTone::Success).show(ui);
/// });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatusIndicator<'a> {
    label: &'a str,
    tone: BadgeTone,
    detail: Option<&'a str>,
}

impl<'a> StatusIndicator<'a> {
    /// Create a status with `label` and semantic `tone`.
    pub fn new(label: &'a str, tone: BadgeTone) -> Self {
        Self {
            label,
            tone,
            detail: None,
        }
    }

    /// De-emphasized supporting text after the status label.
    pub fn detail(mut self, detail: &'a str) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Show the status indicator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let dot_color = self.tone.dot_color(tokens);
        let response = direction::horizontal(ui, |ui| {
            let (dot_rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
            ui.painter()
                .circle_filled(dot_rect.center(), 5.0, dot_color);
            let response = ui.label(RichText::new(self.label).size(12.5).color(tokens.text));
            if let Some(detail) = self.detail {
                ui.label(RichText::new(detail).size(11.5).color(tokens.muted_text));
            }
            response
        })
        .inner;
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::Role::Status);
            node.set_label(self.label);
        });
        response
    }
}

/// A centered message for an empty list or view, with an optional action.
///
/// ```
/// use faseela_ui::widgets::EmptyState;
///
/// egui::__run_test_ui(|ui| {
///     let response = EmptyState::new("No items").action("Add").show(ui);
///     let _clicked = response.inner.action_clicked;
/// });
/// ```
pub struct EmptyState<'a> {
    title: &'a str,
    description: Option<&'a str>,
    action: Option<&'a str>,
    action_enabled: bool,
}

/// Result of showing an [`EmptyState`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EmptyStateResponse {
    /// Whether the optional action button was activated this frame.
    pub action_clicked: bool,
    /// Whether the empty state's action was enabled.
    pub action_enabled: bool,
}

impl<'a> EmptyState<'a> {
    /// Create an empty state with `title` and no description or action.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: None,
            action: None,
            action_enabled: true,
        }
    }

    /// Add a short explanation below the title.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Add an action proposal button below the message.
    pub fn action(mut self, action: &'a str) -> Self {
        self.action = Some(action);
        self
    }

    /// Disable the action button while keeping it visible and accessible.
    pub fn action_enabled(mut self, enabled: bool) -> Self {
        self.action_enabled = enabled;
        self
    }

    /// Show the empty state, filling the available width.
    pub fn show(self, ui: &mut Ui) -> InnerResponse<EmptyStateResponse> {
        let tokens = theme::active_tokens(ui.ctx());
        ui.vertical_centered(|ui| {
            ui.set_width(ui.available_width());
            ui.add_space(12.0);
            ui.label(RichText::new(self.title).strong().color(tokens.text));
            if let Some(description) = self.description {
                ui.label(RichText::new(description).small().color(tokens.muted_text));
            }
            let mut response = EmptyStateResponse::default();
            if let Some(action) = self.action {
                let button = ThemedButton::new(action)
                    .variant(ButtonVariant::Primary)
                    .enabled(self.action_enabled);
                response.action_enabled = self.action_enabled && ui.is_enabled();
                response.action_clicked = button.show(ui).clicked();
            }
            ui.add_space(12.0);
            response
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_values_are_finite_and_bounded() {
        assert_eq!(ProgressBar::new(-1.0).value, 0.0);
        assert_eq!(ProgressBar::new(2.0).value, 1.0);
        assert_eq!(ProgressBar::new(f32::NAN).value, 0.0);
        assert_eq!(ProgressBar::new(f32::INFINITY).value, 0.0);
    }
}
