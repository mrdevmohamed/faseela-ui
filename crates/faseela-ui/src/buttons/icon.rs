//! Icon interaction adapted from Craft (MIT OR Apache-2.0), modified for Faseela.
//! Upstream: <https://github.com/storytold/craft-libs>.
use super::{ButtonVisuals, IconButtonStyle, style::bounded_stroke};
use egui::{Color32, Id, Rect, Response, Role, Sense, StrokeKind, Ui, Vec2, WidgetInfo};

/// An icon-only button. The accessible label is independent of the icon and widget ID.
///
/// The surrounding UI scope/position supplies the ID; use `Ui::push_id` or [`Self::id`] for
/// application-specific identity. The caller paints the icon and dispatches commands from the
/// returned response. Selection is caller-owned and is never toggled implicitly.
pub struct IconButton<'a> {
    label: &'a str,
    size: Vec2,
    selected: bool,
    enabled: bool,
    id: Option<Id>,
    style: &'a IconButtonStyle,
    single_shape_frame: bool,
}

impl<'a> IconButton<'a> {
    /// Create an icon-only button with an accessible label, size, and caller-owned style.
    pub fn new(label: &'a str, size: Vec2, style: &'a IconButtonStyle) -> Self {
        Self {
            label,
            size,
            selected: false,
            enabled: true,
            id: None,
            style,
            single_shape_frame: false,
        }
    }

    /// Mark the button as selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disable the button while keeping its accessible role and label.
    ///
    /// `ui.add_enabled(false, ...)` also disables it; this builder is a convenience for
    /// application-owned state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign an explicit stable widget ID.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Paint fill and border as one egui rectangle instead of separate shapes.
    pub fn single_shape_frame(mut self) -> Self {
        self.single_shape_frame = true;
        self
    }

    /// Paint one button and return its mouse, keyboard and accessibility response.
    ///
    /// `paint_icon` receives the full button rectangle and resolved icon tint. Invalid,
    /// non-positive or excessively large dimensions produce an inert zero-size response.
    pub fn show(self, ui: &mut Ui, paint_icon: impl FnOnce(&Ui, Rect, Color32)) -> Response {
        if self.enabled {
            self.show_in(ui, paint_icon)
        } else {
            ui.add_enabled_ui(false, |ui| self.show_in(ui, paint_icon))
                .inner
        }
    }

    fn show_in(self, ui: &mut Ui, paint_icon: impl FnOnce(&Ui, Rect, Color32)) -> Response {
        let enabled = ui.is_enabled();
        let valid = self.size.is_finite()
            && self.size.x > 0.0
            && self.size.y > 0.0
            && self.size.x <= 4096.0
            && self.size.y <= 4096.0;
        let sense = if valid && enabled {
            Sense::click()
        } else {
            Sense::hover()
        };

        let response = if let Some(id) = self.id {
            let (_, rect) = ui.allocate_space(if valid { self.size } else { Vec2::ZERO });
            ui.interact(rect, id, sense)
        } else {
            ui.allocate_exact_size(if valid { self.size } else { Vec2::ZERO }, sense)
                .1
        };
        response.widget_info(|| {
            WidgetInfo::selected(
                Role::Button,
                valid && enabled,
                self.selected && enabled,
                self.label,
            )
        });
        if !valid {
            return response;
        }

        let rect = response.rect;
        let visuals = if enabled {
            self.style.visuals(
                self.selected,
                response.hovered(),
                response.is_pointer_button_down_on(),
            )
        } else {
            self.style.disabled.unwrap_or(ButtonVisuals {
                fill: ui.visuals().widgets.noninteractive.bg_fill,
                stroke: ui.visuals().widgets.noninteractive.bg_stroke,
                icon: ui.visuals().widgets.noninteractive.fg_stroke.color,
            })
        };
        let stroke = bounded_stroke(visuals.stroke, rect);
        if self.single_shape_frame {
            ui.painter().rect(
                rect,
                self.style.corner_radius,
                visuals.fill,
                stroke,
                StrokeKind::Inside,
            );
        } else {
            if visuals.fill != Color32::TRANSPARENT {
                ui.painter()
                    .rect_filled(rect, self.style.corner_radius, visuals.fill);
            }
            if !stroke.is_empty() {
                ui.painter().rect_stroke(
                    rect,
                    self.style.corner_radius,
                    stroke,
                    StrokeKind::Inside,
                );
            }
        }
        paint_icon(ui, rect, visuals.icon);
        if enabled && response.has_focus() && rect.width() > 4.0 && rect.height() > 4.0 {
            let focus_rect = rect.shrink(2.0);
            let stroke = bounded_stroke(self.style.focus_stroke, focus_rect);
            if !stroke.is_empty() {
                ui.painter().rect_stroke(
                    focus_rect,
                    self.style.corner_radius,
                    stroke,
                    StrokeKind::Inside,
                );
            }
        }
        response
    }
}
