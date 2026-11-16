//! Faseela design tokens and runtime theme presets.

use egui::{
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke,
    Style, Theme, Visuals,
};
use std::sync::Arc;

/// Official Faseela palette.
pub mod palette {
    use egui::Color32;

    pub const BACKGROUND: Color32 = Color32::from_rgb(0x0A, 0x1D, 0x1A);
    pub const PRIMARY: Color32 = Color32::from_rgb(0x2F, 0xA3, 0x6B);
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(0x9B, 0xEB, 0xB8);
    pub const ACCENT: Color32 = Color32::from_rgb(0xF2, 0xC1, 0x4E);
    pub const SURFACE: Color32 = Color32::from_rgb(0xE8, 0xF7, 0xEE);
}

/// Built-in appearance presets exposed by the shared UI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemePreset {
    #[default]
    FaseelaDark,
    Midnight,
    Forest,
    Light,
}

impl ThemePreset {
    pub const ALL: [Self; 4] = [Self::FaseelaDark, Self::Midnight, Self::Forest, Self::Light];

    pub const fn label(self) -> &'static str {
        match self {
            Self::FaseelaDark => "Faseela Dark",
            Self::Midnight => "Midnight",
            Self::Forest => "Forest",
            Self::Light => "Light",
        }
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Midnight,
            2 => Self::Forest,
            3 => Self::Light,
            _ => Self::FaseelaDark,
        }
    }

    /// Secondary text color with sufficient contrast for this preset.
    pub const fn muted_text(self) -> Color32 {
        match self {
            Self::FaseelaDark => Color32::from_rgb(0xA9, 0xC1, 0xB7),
            Self::Midnight => Color32::from_rgb(0xA9, 0xB2, 0xD0),
            Self::Forest => Color32::from_rgb(0xAF, 0xC8, 0xB7),
            Self::Light => Color32::from_rgb(0x62, 0x73, 0x6A),
        }
    }
}

/// Install bundled Arabic and broad-Unicode fallback fonts for UI text.
pub fn install_arabic_font(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    let font_name = "faseela-arabic-fallback".to_owned();
    let arabic_name = "faseela-noto-sans-arabic".to_owned();

    fonts.font_data.insert(
        arabic_name.clone(),
        Arc::new(FontData::from_static(uzor_fonts::NOTO_SANS_ARABIC)),
    );
    fonts.font_data.insert(
        font_name.clone(),
        Arc::new(FontData::from_static(dejavu::sans::regular())),
    );

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family.clone())
            .or_default()
            .push(arabic_name.clone());
        fonts
            .families
            .entry(family)
            .or_default()
            .push(font_name.clone());
    }

    ctx.set_fonts(fonts);
}

/// Apply a complete theme preset. Call again whenever the selected preset changes.
pub fn apply_preset(ctx: &Context, preset: ThemePreset) {
    let mut style = Style::default();
    let mut visuals = match preset {
        ThemePreset::Light => Visuals::light(),
        _ => Visuals::dark(),
    };

    let (background, panel, surface, primary, accent, text_accent) = match preset {
        ThemePreset::FaseelaDark => (
            Color32::from_rgb(0x0A, 0x1D, 0x1A),
            Color32::from_rgb(0x10, 0x2A, 0x25),
            Color32::from_rgb(0x17, 0x35, 0x2D),
            palette::PRIMARY,
            palette::ACCENT,
            palette::PRIMARY_LIGHT,
        ),
        ThemePreset::Midnight => (
            Color32::from_rgb(0x10, 0x14, 0x26),
            Color32::from_rgb(0x19, 0x20, 0x38),
            Color32::from_rgb(0x25, 0x2E, 0x4A),
            Color32::from_rgb(0x68, 0x75, 0xD9),
            Color32::from_rgb(0xE7, 0xB9, 0x6A),
            Color32::from_rgb(0xC0, 0xC7, 0xFF),
        ),
        ThemePreset::Forest => (
            Color32::from_rgb(0x10, 0x24, 0x1C),
            Color32::from_rgb(0x18, 0x32, 0x26),
            Color32::from_rgb(0x24, 0x43, 0x31),
            Color32::from_rgb(0x3D, 0x8B, 0x62),
            Color32::from_rgb(0xE6, 0xC4, 0x6A),
            Color32::from_rgb(0xB6, 0xE3, 0xC3),
        ),
        ThemePreset::Light => (
            Color32::from_rgb(0xF4, 0xF7, 0xF5),
            Color32::from_rgb(0xFF, 0xFF, 0xFF),
            Color32::from_rgb(0xE8, 0xF0, 0xEB),
            Color32::from_rgb(0x23, 0x7A, 0x50),
            Color32::from_rgb(0xA6, 0x6B, 0x00),
            Color32::from_rgb(0x27, 0x67, 0x49),
        ),
    };

    let (text_color, border_color, hover_color) = match preset {
        ThemePreset::FaseelaDark => (
            Color32::from_rgb(0xE8, 0xF7, 0xEE),
            Color32::from_rgb(0x2D, 0x51, 0x45),
            Color32::from_rgb(0x20, 0x46, 0x3A),
        ),
        ThemePreset::Midnight => (
            Color32::from_rgb(0xEA, 0xED, 0xFF),
            Color32::from_rgb(0x3A, 0x45, 0x67),
            Color32::from_rgb(0x31, 0x3C, 0x5A),
        ),
        ThemePreset::Forest => (
            Color32::from_rgb(0xE6, 0xF2, 0xE9),
            Color32::from_rgb(0x35, 0x5A, 0x43),
            Color32::from_rgb(0x2D, 0x50, 0x39),
        ),
        ThemePreset::Light => (
            Color32::from_rgb(0x1D, 0x2B, 0x23),
            Color32::from_rgb(0xD6, 0xE1, 0xD9),
            Color32::from_rgb(0xDF, 0xEB, 0xE3),
        ),
    };
    let text_stroke = Stroke::new(1.0, text_color);
    let border_stroke = Stroke::new(1.0, border_color);
    let hover_stroke = Stroke::new(1.0, primary);
    let on_primary_stroke = Stroke::new(1.0, Color32::WHITE);

    visuals.panel_fill = background;
    visuals.window_fill = panel;
    visuals.extreme_bg_color = if preset == ThemePreset::Light {
        Color32::from_rgb(0xE2, 0xE9, 0xE4)
    } else {
        background.gamma_multiply(0.65)
    };
    visuals.faint_bg_color = surface;
    visuals.selection.bg_fill = primary;
    visuals.selection.stroke = Stroke::new(1.0, text_accent);
    visuals.hyperlink_color = text_accent;
    visuals.warn_fg_color = accent;
    visuals.widgets.noninteractive.bg_fill = panel;
    visuals.widgets.noninteractive.bg_stroke = border_stroke;
    visuals.widgets.noninteractive.fg_stroke = text_stroke;
    visuals.widgets.inactive.bg_fill = surface;
    visuals.widgets.inactive.bg_stroke = border_stroke;
    visuals.widgets.inactive.fg_stroke = text_stroke;
    visuals.widgets.hovered.bg_fill = hover_color;
    visuals.widgets.hovered.bg_stroke = hover_stroke;
    visuals.widgets.hovered.fg_stroke = text_stroke;
    visuals.widgets.active.bg_fill = primary;
    visuals.widgets.active.bg_stroke = hover_stroke;
    visuals.widgets.active.fg_stroke = on_primary_stroke;
    visuals.widgets.open.bg_fill = surface;
    visuals.widgets.open.bg_stroke = hover_stroke;
    visuals.widgets.open.fg_stroke = text_stroke;
    visuals.window_stroke = border_stroke;
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);

    style.visuals = visuals;
    style.spacing.window_margin = Margin::same(16);
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
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
}

/// Backwards-compatible shortcut for the default Faseela dark theme.
pub fn apply_dark(ctx: &Context) {
    apply_preset(ctx, ThemePreset::FaseelaDark);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_faseela_palette() {
        let ctx = Context::default();
        apply_dark(&ctx);
        let visuals = ctx.style_of(Theme::Dark).visuals.clone();
        assert_eq!(visuals.panel_fill, palette::BACKGROUND);
        assert_eq!(visuals.selection.bg_fill, palette::PRIMARY);
        assert_eq!(visuals.widgets.active.fg_stroke.color, Color32::WHITE);
    }

    #[test]
    fn light_preset_uses_light_visuals_and_background() {
        let ctx = Context::default();
        apply_preset(&ctx, ThemePreset::Light);
        let visuals = ctx.style_of(Theme::Light).visuals.clone();
        assert_eq!(visuals.panel_fill, Color32::from_rgb(0xF4, 0xF7, 0xF5));
        assert_eq!(visuals.widgets.noninteractive.bg_fill, Color32::WHITE);
        assert_eq!(
            visuals.widgets.inactive.fg_stroke.color,
            Color32::from_rgb(0x1D, 0x2B, 0x23)
        );
    }

    #[test]
    fn every_preset_sets_widget_foregrounds_and_borders() {
        let ctx = Context::default();

        for preset in ThemePreset::ALL {
            apply_preset(&ctx, preset);
            let egui_theme = if preset == ThemePreset::Light {
                Theme::Light
            } else {
                Theme::Dark
            };
            let visuals = ctx.style_of(egui_theme).visuals.clone();
            let expected_text = match preset {
                ThemePreset::FaseelaDark => Color32::from_rgb(0xE8, 0xF7, 0xEE),
                ThemePreset::Midnight => Color32::from_rgb(0xEA, 0xED, 0xFF),
                ThemePreset::Forest => Color32::from_rgb(0xE6, 0xF2, 0xE9),
                ThemePreset::Light => Color32::from_rgb(0x1D, 0x2B, 0x23),
            };

            assert_eq!(visuals.widgets.inactive.fg_stroke.color, expected_text);
            assert_eq!(visuals.widgets.active.fg_stroke.color, Color32::WHITE);
            assert!(visuals.window_stroke.width > 0.0);
        }
    }

    #[test]
    fn every_preset_has_a_stable_label_and_index() {
        assert_eq!(ThemePreset::from_index(3), ThemePreset::Light);
        assert_eq!(ThemePreset::Light.label(), "Light");
        assert_eq!(ThemePreset::ALL.len(), 4);
    }
}
