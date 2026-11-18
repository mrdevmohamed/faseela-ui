//! Themed text buttons with caller-owned actions and selection.
use super::style::{InteractionState, bounded_stroke, focus_color, variant_visuals};
use super::{ButtonVariant, ControlSize};
use crate::theme;
use egui::{
    Color32, FontId, Id, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo,
};
use std::borrow::Cow;

/// A themed text button with variants, sizes, selected and disabled states.
///
/// The returned [`Response`] is a proposal: applications own the resulting state change or
/// command. Keyboard focus, Space/Enter activation, focus indicators, and AccessKit semantics
/// are provided by egui's click interaction and this widget's [`WidgetInfo`].
///
/// ```
/// use faseela_ui::buttons::{Button, ButtonVariant, ControlSize};
///
/// egui::__run_test_ui(|ui| {
///     let response = Button::new("Save")
///         .variant(ButtonVariant::Primary)
///         .size(ControlSize::Medium)
///         .show(ui);
///     let _clicked = response.clicked();
/// });
/// ```
#[must_use = "add the button to a Ui with `.show(ui)` or `ui.add(button)`"]
pub struct Button<'a> {
    text: Cow<'a, str>,
    icon: Option<Cow<'a, str>>,
    variant: ButtonVariant,
    size: ControlSize,
    selected: bool,
    enabled: bool,
    id: Option<Id>,
    min_width: f32,
    full_width: bool,
}

impl<'a> Button<'a> {
    /// Create a button with a visible and accessible label.
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            variant: ButtonVariant::Primary,
            size: ControlSize::Medium,
            selected: false,
            enabled: true,
            id: None,
            min_width: 0.0,
            full_width: false,
        }
    }

    /// Choose the visual treatment. Defaults to [`ButtonVariant::Primary`].
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Choose the control size. Defaults to [`ControlSize::Medium`].
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Add a leading icon glyph. Applications may use any font glyph or short symbol.
    ///
    /// For image assets, use egui's `Image` buttons directly or an [`super::IconButton`] with a custom
    /// painter.
    pub fn icon(mut self, icon: impl Into<Cow<'a, str>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Paint the button as selected. Selection is caller-owned and never toggled implicitly.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disable the button while keeping its accessible label and role.
    ///
    /// `ui.add_enabled(false, ...)` has the same effect and composes with this builder.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable widget ID, useful for tests and keyboard focus restoration.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Request a minimum width in logical points.
    pub fn min_width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.min_width = width.min(4096.0);
        }
        self
    }

    /// Expand the button to the available width.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    /// Show the button.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        if self.enabled {
            self.show_in(ui)
        } else {
            ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner
        }
    }
}

impl Button<'_> {
    fn show_in(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let enabled = ui.is_enabled();
        let metrics = self.size;
        let font = FontId::proportional(metrics.font_size());
        let icon_font = FontId::proportional((metrics.font_size() + 1.0).min(18.0));

        let text_galley = ui.fonts_mut(|fonts| {
            fonts.layout_no_wrap(self.text.to_string(), font.clone(), tokens.text)
        });
        let icon_galley = self.icon.as_ref().map(|icon| {
            ui.fonts_mut(|fonts| {
                fonts.layout_no_wrap(icon.to_string(), icon_font.clone(), tokens.text)
            })
        });

        let gap = icon_galley.as_ref().map_or(0.0, |_| 6.0);
        let icon_width = icon_galley
            .as_ref()
            .map_or(0.0, |galley| galley.rect.width());
        let padding = metrics.padding();
        let content_width = icon_width + gap + text_galley.rect.width();
        let desired_width = content_width + padding.x * 2.0;
        let available = ui.available_width().max(0.0);
        let width = if self.full_width {
            available
        } else {
            desired_width
                .max(self.min_width)
                .min(ui.max_rect().width().max(0.0))
        };
        let icon_height = icon_galley
            .as_ref()
            .map_or(0.0, |galley| galley.rect.height());
        let height = metrics
            .height()
            .max(text_galley.rect.height().max(icon_height) + padding.y * 2.0);
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };

        let (rect, response) = if let Some(id) = self.id {
            let (_, rect) = ui.allocate_space(Vec2::new(width, height));
            let response = ui.interact(rect, id, sense);
            (rect, response)
        } else {
            ui.allocate_exact_size(Vec2::new(width, height), sense)
        };

        let pressed = enabled && response.is_pointer_button_down_on();
        let state = InteractionState {
            hovered: enabled && response.hovered(),
            pressed,
            selected: self.selected && enabled,
            disabled: !enabled,
        };
        let visuals = variant_visuals(tokens, self.variant, state);
        response
            .widget_info(|| WidgetInfo::selected(Role::Button, enabled, self.selected, &self.text));
        if enabled && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let corner_radius = metrics.corner_radius();
        let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
        if visuals.fill != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, corner_radius, visuals.fill);
        }
        if !visuals.stroke.is_empty() {
            ui.painter().rect_stroke(
                rect,
                corner_radius,
                bounded_stroke(visuals.stroke, rect),
                StrokeKind::Inside,
            );
        }

        let mut x = rect.left() + ((rect.width() - content_width) * 0.5).max(0.0);
        let center_y = rect.center().y;
        let rtl = crate::direction::is_rtl(ui);
        let icon_rect = icon_galley.as_ref().map(|galley| {
            let size = galley.rect.size();
            (size, galley.clone())
        });
        if rtl {
            // Leading content is on the right in an RTL interface.
            let text_pos = egui::pos2(x, center_y - text_galley.rect.height() * 0.5);
            painter.galley_with_override_text_color(text_pos, text_galley.clone(), visuals.icon);
            x += text_galley.rect.width() + gap;
            if let Some((size, galley)) = icon_rect {
                painter.galley_with_override_text_color(
                    egui::pos2(x, center_y - size.y * 0.5),
                    galley,
                    visuals.icon,
                );
            }
        } else {
            if let Some((size, galley)) = icon_rect {
                painter.galley_with_override_text_color(
                    egui::pos2(x, center_y - size.y * 0.5),
                    galley,
                    visuals.icon,
                );
                x += size.x + gap;
            }
            painter.galley_with_override_text_color(
                egui::pos2(x, center_y - text_galley.rect.height() * 0.5),
                text_galley,
                visuals.icon,
            );
        }

        if enabled && response.has_focus() && rect.width() > 4.0 && rect.height() > 4.0 {
            let focus_rect = rect.shrink(2.0);
            let ring = bounded_stroke(
                Stroke::new(
                    2.0,
                    if self.selected
                        || (pressed
                            && matches!(
                                self.variant,
                                ButtonVariant::Outline | ButtonVariant::Ghost
                            ))
                    {
                        tokens.on_primary
                    } else {
                        focus_color(tokens, self.variant)
                    },
                ),
                focus_rect,
            );
            if !ring.is_empty() {
                ui.painter().rect_stroke(
                    focus_rect,
                    metrics.corner_radius(),
                    ring,
                    StrokeKind::Inside,
                );
            }
        }

        response
    }
}
