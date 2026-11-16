//! Faseela's initial design tokens and egui style integration.

use egui::{Color32, Context, CornerRadius, FontId, Margin, Stroke, Style, Visuals};

/// Official Faseela palette.
pub mod palette {
    use egui::Color32;

    pub const BACKGROUND: Color32 = Color32::from_rgb(0x0A, 0x1D, 0x1A);
    pub const PRIMARY: Color32 = Color32::from_rgb(0x2F, 0xA3, 0x6B);
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(0x9B, 0xEB, 0xB8);
    pub const ACCENT: Color32 = Color32::from_rgb(0xF2, 0xC1, 0x4E);
    pub const SURFACE: Color32 = Color32::from_rgb(0xE8, 0xF7, 0xEE);
}

/// Applies the initial dark Faseela visual language to an egui context.
pub fn apply_dark(ctx: &Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::dark();

    visuals.panel_fill = palette::BACKGROUND;
    visuals.window_fill = Color32::from_rgb(0x10, 0x2A, 0x25);
    visuals.extreme_bg_color = Color32::from_rgb(0x06, 0x14, 0x12);
    visuals.faint_bg_color = Color32::from_rgb(0x17, 0x35, 0x2D);
    visuals.selection.bg_fill = palette::PRIMARY;
    visuals.selection.stroke = Stroke::new(1.0, palette::PRIMARY_LIGHT);
    visuals.hyperlink_color = palette::PRIMARY_LIGHT;
    visuals.warn_fg_color = palette::ACCENT;
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0x10, 0x2A, 0x25);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x17, 0x35, 0x2D);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x20, 0x4B, 0x3B);
    visuals.widgets.active.bg_fill = palette::PRIMARY;
    visuals.widgets.open.bg_fill = Color32::from_rgb(0x17, 0x35, 0x2D);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);

    style.visuals = visuals;
    style.spacing.window_margin = Margin::same(16);
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.text_styles.insert(egui::TextStyle::Heading, FontId::proportional(24.0));
    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_faseela_palette() {
        let ctx = Context::default();
        apply_dark(&ctx);
        let visuals = ctx.style().visuals.clone();
        assert_eq!(visuals.panel_fill, palette::BACKGROUND);
        assert_eq!(visuals.selection.bg_fill, palette::PRIMARY);
    }
}
