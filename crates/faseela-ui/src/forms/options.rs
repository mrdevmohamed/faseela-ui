//! Shared option-row measurement and painting for checkboxes, switches, and radios.
use super::common::{FIELD_FONT_SIZE, label_galley};
use crate::{
    direction,
    theme::{self, ThemeTokens},
};
use egui::{
    CornerRadius, FontId, Id, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, vec2,
};

const ROW_GAP: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ControlKind {
    CheckBox,
    Radio,
    Switch,
}

type OptionRow = (egui::Rect, Response);

pub(super) fn allocate_option_row(
    ui: &mut Ui,
    label: &str,
    id: Id,
    control_size: f32,
) -> OptionRow {
    allocate_row(ui, label, id, Vec2::splat(control_size))
}

fn allocate_row(ui: &mut Ui, label: &str, id: Id, control_size: Vec2) -> OptionRow {
    let available = ui.available_width().max(0.0);
    let tokens = theme::active_tokens(ui.ctx());
    let width = (available - control_size.x - ROW_GAP).max(1.0);
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
    (rect, response)
}

pub(super) fn option_row(
    ui: &mut Ui,
    label: &str,
    id: Option<Id>,
    kind: ControlKind,
    selected: bool,
    control_size: f32,
) -> Response {
    option_row_with_control(
        ui,
        label,
        id,
        kind,
        selected,
        Vec2::splat(control_size),
        false,
    )
}

pub(super) fn option_row_with_control(
    ui: &mut Ui,
    label: &str,
    id: Option<Id>,
    kind: ControlKind,
    selected: bool,
    control_size: Vec2,
    invalid: bool,
) -> Response {
    let tokens = theme::active_tokens(ui.ctx());
    let enabled = ui.is_enabled();
    let id = id.unwrap_or_else(|| ui.next_auto_id());
    let (rect, response) = allocate_row(ui, label, id, control_size);
    let selected = selected ^ (kind != ControlKind::Radio && response.clicked());
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
            hovered: response.hovered(),
            pressed: response.is_pointer_button_down_on(),
            invalid,
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

pub(super) struct OptionPaint<'a> {
    pub(super) label: &'a str,
    pub(super) kind: ControlKind,
    pub(super) selected: bool,
    pub(super) enabled: bool,
    pub(super) focused: bool,
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) invalid: bool,
    pub(super) tokens: ThemeTokens,
    pub(super) control_size: Vec2,
}

pub(super) fn paint_option_rect(ui: &Ui, rect: egui::Rect, paint: OptionPaint<'_>) {
    let OptionPaint {
        label,
        kind,
        selected,
        enabled,
        focused,
        hovered,
        pressed,
        invalid,
        tokens,
        control_size,
    } = paint;
    // A tiny parent may clip the control, but must never paint outside its own allocation.
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
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
    painter.galley_with_override_text_color(text_pos, galley, color);

    let idle_fill = if !enabled {
        tokens.disabled
    } else if pressed {
        tokens.active
    } else if hovered {
        tokens.hover
    } else {
        tokens.surface
    };

    match kind {
        ControlKind::CheckBox => {
            let fill = if selected {
                if enabled {
                    tokens.primary
                } else {
                    tokens.disabled
                }
            } else {
                idle_fill
            };
            let border = if !enabled {
                tokens.border
            } else if invalid {
                tokens.error
            } else if focused {
                tokens.focus
            } else {
                tokens.control_border
            };
            painter.rect(
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
                painter.line_segment([a, b], Stroke::new(2.0, mark));
                painter.line_segment([b, c], Stroke::new(2.0, mark));
            }
        }
        ControlKind::Radio => {
            let center = control_rect.center();
            let radius = control_rect.width() * 0.5;
            let fill = idle_fill;
            let border = if !enabled {
                tokens.border
            } else if invalid {
                tokens.error
            } else if focused {
                tokens.focus
            } else {
                tokens.control_border
            };
            painter.circle_filled(center, radius, fill);
            painter.circle_stroke(
                center,
                radius - 0.5,
                Stroke::new(if focused { 2.0 } else { 1.5 }, border),
            );
            if selected {
                painter.circle_filled(
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
            } else {
                idle_fill
            };
            let border = if !enabled {
                tokens.border
            } else if invalid {
                tokens.error
            } else if focused {
                tokens.focus
            } else {
                tokens.control_border
            };
            painter.rect(
                track,
                CornerRadius::same((track.height() * 0.5) as u8),
                fill,
                Stroke::new(if focused { 2.0 } else { 1.0 }, border),
                StrokeKind::Inside,
            );
            let knob_radius = track.height() * 0.5 - 4.0;
            let knob_x = if selected != rtl {
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
            painter.circle_filled(
                egui::pos2(knob_x, track.center().y),
                knob_radius,
                knob_color,
            );
        }
    }

    if focused {
        let focus_rect = rect.shrink(0.75);
        painter.rect_stroke(
            focus_rect,
            CornerRadius::same(5),
            Stroke::new(1.5, tokens.focus),
            StrokeKind::Inside,
        );
    }
}
