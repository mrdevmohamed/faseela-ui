//! Faseela design tokens and runtime theme presets.
//!
//! [`ThemeTokens`] is the single source of truth for every semantic color used by the shared
//! widgets. [`apply_preset`] installs those tokens into egui's [`Visuals`], while
//! [`active_tokens`] lets caller-painted widgets read the same values without being handed the
//! current preset. This keeps custom painting in sync with theme switches.

mod color;
mod fonts;
mod presets;
mod tokens;

pub use color::{blend, contrast_ratio, relative_luminance};
pub use fonts::install_arabic_font;
pub use presets::{ThemePreset, palette};
pub use tokens::ThemeTokens;

use egui::{Context, CornerRadius, FontId, Id, Margin, Stroke, Style, Theme, Visuals};

fn preset_id() -> Id {
    Id::unique("faseela_ui::theme::preset")
}

/// The preset most recently installed by [`apply_preset`], or the default preset.
pub fn active_preset(ctx: &Context) -> ThemePreset {
    ctx.data(|data| data.get_temp::<ThemePreset>(preset_id()))
        .unwrap_or_default()
}

/// Semantic tokens of the currently active preset, for caller-painted widgets.
///
/// Returns the default preset's tokens until [`apply_preset`] (or [`apply_dark`]) is called.
///
/// ```
/// use faseela_ui::theme::{self, ThemePreset};
///
/// let ctx = egui::Context::default();
/// theme::apply_preset(&ctx, ThemePreset::Light);
/// assert_eq!(theme::active_tokens(&ctx), ThemePreset::Light.tokens());
/// ```
pub fn active_tokens(ctx: &Context) -> ThemeTokens {
    active_preset(ctx).tokens()
}

/// Apply a complete theme preset. Call again whenever the selected preset changes.
///
/// Every token is derived from [`ThemePreset::tokens`], so egui widgets and caller-painted
/// widgets read the same values. The chosen preset is also recorded for [`active_tokens`].
pub fn apply_preset(ctx: &Context, preset: ThemePreset) {
    let tokens = preset.tokens();
    let mut style = Style::default();
    let mut visuals = match preset {
        ThemePreset::Light => Visuals::light(),
        _ => Visuals::dark(),
    };

    let border = Stroke::new(1.0, tokens.border);
    let control_border = Stroke::new(1.0, tokens.control_border);
    let focus = Stroke::new(2.0, tokens.focus);
    let primary_stroke = Stroke::new(1.0, tokens.primary);
    let text_stroke = Stroke::new(1.0, tokens.text);
    let on_primary = Stroke::new(1.0, tokens.on_primary);

    visuals.panel_fill = tokens.background;
    visuals.window_fill = tokens.surface;
    visuals.extreme_bg_color = if preset == ThemePreset::Light {
        tokens.elevated_surface
    } else {
        tokens.background.gamma_multiply(0.65)
    };
    visuals.faint_bg_color = tokens.elevated_surface;
    visuals.selection.bg_fill = tokens.selection;
    // egui uses this stroke as the foreground of selected text and selectable labels.
    // Focus has its own token; using it here produces mint-on-green text in the dark preset.
    visuals.selection.stroke = on_primary;
    visuals.hyperlink_color = tokens.secondary;
    visuals.warn_fg_color = tokens.warning;
    visuals.error_fg_color = tokens.error;
    visuals.widgets.noninteractive.bg_fill = tokens.surface;
    visuals.widgets.noninteractive.weak_bg_fill = tokens.surface;
    visuals.widgets.noninteractive.bg_stroke = border;
    // egui reuses `noninteractive` for disabled controls: muted foreground signals that state.
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, tokens.muted_text);
    visuals.widgets.inactive.bg_fill = tokens.surface;
    visuals.widgets.inactive.weak_bg_fill = tokens.surface;
    visuals.widgets.inactive.bg_stroke = control_border;
    visuals.widgets.inactive.fg_stroke = text_stroke;
    visuals.widgets.hovered.bg_fill = tokens.hover;
    visuals.widgets.hovered.weak_bg_fill = tokens.hover;
    visuals.widgets.hovered.bg_stroke = primary_stroke;
    visuals.widgets.hovered.fg_stroke = text_stroke;
    visuals.widgets.active.bg_fill = tokens.active;
    visuals.widgets.active.weak_bg_fill = tokens.active;
    visuals.widgets.active.bg_stroke = focus;
    visuals.widgets.active.fg_stroke = on_primary;
    visuals.widgets.open.bg_fill = tokens.elevated_surface;
    visuals.widgets.open.weak_bg_fill = tokens.elevated_surface;
    visuals.widgets.open.bg_stroke = primary_stroke;
    visuals.widgets.open.fg_stroke = text_stroke;
    visuals.window_stroke = border;
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::same(8);
        // Prevent adjacent controls from painting over each other's bounds.
        widget.expansion = 0.0;
    }

    style.visuals = visuals;
    style.spacing.window_margin = Margin::same(16);
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.interact_size = egui::vec2(40.0, 30.0);
    style
        .text_styles
        .insert(egui::TextStyle::Heading, FontId::proportional(24.0));

    let egui_theme = if preset == ThemePreset::Light {
        Theme::Light
    } else {
        Theme::Dark
    };
    ctx.set_style_of(egui_theme, style);
    ctx.set_theme(egui_theme);
    ctx.data_mut(|data| data.insert_temp(preset_id(), preset));
}

/// Backwards-compatible shortcut for the default Faseela dark theme.
pub fn apply_dark(ctx: &Context) {
    apply_preset(ctx, ThemePreset::FaseelaDark);
}

#[cfg(test)]
mod tests;
