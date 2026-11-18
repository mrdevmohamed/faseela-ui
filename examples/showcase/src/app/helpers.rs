//! Showcase-only layout, swatches, labels, and measured-contrast presentation.
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Margin, Rect, RichText, Sense, Stroke, StrokeKind,
    Ui, vec2,
};
use faseela_ui::{
    buttons::{ButtonVariant, ControlSize, IconButtonStyle},
    theme::{self, ThemeTokens},
    widgets::{Badge, BadgeTone, Card},
};

pub(super) fn padded<R>(
    ui: &mut Ui,
    horizontal: i8,
    vertical: i8,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    egui::Frame::new()
        .inner_margin(Margin::symmetric(horizontal, vertical))
        .show(ui, add)
        .inner
}

pub(super) fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    Card::new()
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

pub(super) fn stat_tile(ui: &mut Ui, label: &str, value: &str) {
    let tokens = theme::active_tokens(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(116.0, 58.0), Sense::hover());
    ui.painter().rect(
        rect,
        CornerRadius::same(8),
        tokens.elevated_surface,
        Stroke::new(1.0, tokens.border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.left_top() + vec2(9.0, 8.0),
        Align2::LEFT_TOP,
        label,
        FontId::proportional(8.5),
        tokens.muted_text,
    );
    ui.painter().text(
        rect.left_top() + vec2(9.0, 24.0),
        Align2::LEFT_TOP,
        value,
        FontId::proportional(15.0),
        tokens.text,
    );
}

pub(super) fn token_swatch(ui: &mut Ui, name: &str, color: Color32, tokens: ThemeTokens) {
    let (rect, _) = ui.allocate_exact_size(vec2(112.0, 58.0), Sense::hover());
    let swatch = Rect::from_min_size(rect.min, vec2(112.0, 29.0));
    ui.painter().rect(
        swatch,
        CornerRadius::same(5),
        color,
        Stroke::new(1.0, tokens.control_border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        swatch.left_bottom() + vec2(0.0, 3.0),
        Align2::LEFT_TOP,
        name,
        FontId::proportional(9.5),
        tokens.text,
    );
    ui.painter().text(
        swatch.left_bottom() + vec2(0.0, 14.0),
        Align2::LEFT_TOP,
        hex(color),
        FontId::proportional(8.5),
        tokens.muted_text,
    );
}

pub(super) fn palette_swatch(ui: &mut Ui, name: &str, color: Color32, tokens: ThemeTokens) {
    let (rect, _) = ui.allocate_exact_size(vec2(64.0, 52.0), Sense::hover());
    let swatch = Rect::from_min_size(rect.min, vec2(64.0, 31.0));
    ui.painter().rect(
        swatch,
        CornerRadius::same(6),
        color,
        Stroke::new(1.0, tokens.control_border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        swatch.left_bottom() + vec2(0.0, 2.0),
        Align2::LEFT_TOP,
        name,
        FontId::proportional(9.5),
        tokens.text,
    );
    ui.painter().text(
        swatch.left_bottom() + vec2(0.0, 13.0),
        Align2::LEFT_TOP,
        hex(color),
        FontId::proportional(8.0),
        tokens.muted_text,
    );
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ContrastCheck {
    pub(super) label: &'static str,
    foreground: Color32,
    background: Color32,
    pub(super) minimum: f32,
}

impl ContrastCheck {
    pub(super) fn ratio(self) -> f32 {
        theme::contrast_ratio(self.foreground, self.background)
    }

    pub(super) fn passes(self) -> bool {
        self.ratio() + f32::EPSILON >= self.minimum
    }
}

pub(super) fn contrast_checks(tokens: ThemeTokens) -> Vec<ContrastCheck> {
    vec![
        ContrastCheck {
            label: "text / background",
            foreground: tokens.text,
            background: tokens.background,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "text / surface",
            foreground: tokens.text,
            background: tokens.surface,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "muted text / background",
            foreground: tokens.muted_text,
            background: tokens.background,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "secondary / background",
            foreground: tokens.secondary,
            background: tokens.background,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "on primary / primary",
            foreground: tokens.on_primary,
            background: tokens.primary,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "on warning / warning",
            foreground: tokens.on_warning,
            background: tokens.warning,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "on error / error",
            foreground: tokens.on_error,
            background: tokens.error,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "focus / background",
            foreground: tokens.focus,
            background: tokens.background,
            minimum: 3.0,
        },
        ContrastCheck {
            label: "control border / surface",
            foreground: tokens.control_border,
            background: tokens.surface,
            minimum: 3.0,
        },
    ]
}

pub(super) fn contrast_row(ui: &mut Ui, check: &ContrastCheck) {
    let tokens = theme::active_tokens(ui.ctx());
    let ratio = check.ratio();
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(check.label).size(10.5).color(tokens.text));
        ui.add(Badge::new(
            &format!("{ratio:.2}:1"),
            if check.passes() {
                BadgeTone::Success
            } else {
                BadgeTone::Error
            },
        ));
        ui.label(
            RichText::new(format!("minimum {:.1}:1", check.minimum))
                .size(9.5)
                .color(tokens.muted_text),
        );
    });
}

pub(super) fn icon_style(tokens: ThemeTokens) -> IconButtonStyle {
    IconButtonStyle::themed(tokens, ButtonVariant::Outline, ControlSize::Medium)
}

pub(super) fn plan_label(index: usize) -> &'static str {
    match index {
        1 => "Team",
        2 => "Enterprise",
        _ => "Starter",
    }
}

pub(super) fn density_label(index: usize) -> &'static str {
    match index {
        1 => "Comfortable",
        2 => "Spacious",
        _ => "Compact",
    }
}

pub(super) fn hex(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}
