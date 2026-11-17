//! Faseela design tokens and runtime theme presets.
//!
//! [`ThemeTokens`] is the single source of truth for every semantic color used by the shared
//! widgets. [`apply_preset`] installs those tokens into egui's [`Visuals`], while
//! [`active_tokens`] lets caller-painted widgets read the same values without being handed the
//! current preset. This keeps custom painting in sync with theme switches.

use egui::{
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Id, Margin,
    Stroke, Style, Theme, Visuals,
};
use std::sync::Arc;

/// Official Faseela identity palette (dark foundation).
pub mod palette {
    use egui::Color32;

    /// Deep brand background.
    pub const BACKGROUND: Color32 = Color32::from_rgb(0x0A, 0x1D, 0x1A);
    /// Brand primary green.
    pub const PRIMARY: Color32 = Color32::from_rgb(0x2F, 0xA3, 0x6B);
    /// Light brand green, used as a secondary highlight and focus color.
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(0x9B, 0xEB, 0xB8);
    /// Brand accent gold.
    pub const ACCENT: Color32 = Color32::from_rgb(0xF2, 0xC1, 0x4E);
    /// Light brand surface.
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
    /// All presets in the order shown by the showcase.
    pub const ALL: [Self; 4] = [Self::FaseelaDark, Self::Midnight, Self::Forest, Self::Light];

    /// Stable, human-readable name.
    pub const fn label(self) -> &'static str {
        match self {
            Self::FaseelaDark => "Faseela Dark",
            Self::Midnight => "Midnight",
            Self::Forest => "Forest",
            Self::Light => "Light",
        }
    }

    /// Preset for a stable index (unknown indices fall back to the default preset).
    pub const fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Midnight,
            2 => Self::Forest,
            3 => Self::Light,
            _ => Self::FaseelaDark,
        }
    }

    /// Secondary text color with sufficient contrast for this preset.
    ///
    /// Equivalent to [`ThemeTokens::muted_text`] for the same preset.
    pub const fn muted_text(self) -> Color32 {
        self.tokens().muted_text
    }

    /// Complete semantic token set for this preset.
    pub const fn tokens(self) -> ThemeTokens {
        match self {
            Self::FaseelaDark => ThemeTokens {
                background: Color32::from_rgb(0x0A, 0x1D, 0x1A),
                surface: Color32::from_rgb(0x10, 0x2A, 0x25),
                elevated_surface: Color32::from_rgb(0x17, 0x35, 0x2D),
                primary: Color32::from_rgb(0x2F, 0xA3, 0x6B),
                secondary: Color32::from_rgb(0x9B, 0xEB, 0xB8),
                accent: Color32::from_rgb(0xF2, 0xC1, 0x4E),
                text: Color32::from_rgb(0xE8, 0xF7, 0xEE),
                muted_text: Color32::from_rgb(0xA9, 0xC1, 0xB7),
                border: Color32::from_rgb(0x2D, 0x51, 0x45),
                selection: Color32::from_rgb(0x2F, 0xA3, 0x6B),
                hover: Color32::from_rgb(0x20, 0x46, 0x3A),
                active: Color32::from_rgb(0x2F, 0xA3, 0x6B),
                disabled: Color32::from_rgb(0x2E, 0x47, 0x3C),
                focus: Color32::from_rgb(0x9B, 0xEB, 0xB8),
                error: Color32::from_rgb(0xFF, 0x7B, 0x7B),
                warning: Color32::from_rgb(0xF2, 0xC1, 0x4E),
                success: Color32::from_rgb(0x4F, 0xC1, 0x84),
            },
            Self::Midnight => ThemeTokens {
                background: Color32::from_rgb(0x10, 0x14, 0x26),
                surface: Color32::from_rgb(0x19, 0x20, 0x38),
                elevated_surface: Color32::from_rgb(0x25, 0x2E, 0x4A),
                primary: Color32::from_rgb(0x68, 0x75, 0xD9),
                secondary: Color32::from_rgb(0xC0, 0xC7, 0xFF),
                accent: Color32::from_rgb(0xE7, 0xB9, 0x6A),
                text: Color32::from_rgb(0xEA, 0xED, 0xFF),
                muted_text: Color32::from_rgb(0xA9, 0xB2, 0xD0),
                border: Color32::from_rgb(0x3A, 0x45, 0x67),
                selection: Color32::from_rgb(0x68, 0x75, 0xD9),
                hover: Color32::from_rgb(0x31, 0x3C, 0x5A),
                active: Color32::from_rgb(0x68, 0x75, 0xD9),
                disabled: Color32::from_rgb(0x2E, 0x36, 0x52),
                focus: Color32::from_rgb(0xC0, 0xC7, 0xFF),
                error: Color32::from_rgb(0xFF, 0x8A, 0x97),
                warning: Color32::from_rgb(0xE7, 0xB9, 0x6A),
                success: Color32::from_rgb(0x63, 0xC9, 0x94),
            },
            Self::Forest => ThemeTokens {
                background: Color32::from_rgb(0x10, 0x24, 0x1C),
                surface: Color32::from_rgb(0x18, 0x32, 0x26),
                elevated_surface: Color32::from_rgb(0x24, 0x43, 0x31),
                primary: Color32::from_rgb(0x3D, 0x8B, 0x62),
                secondary: Color32::from_rgb(0xB6, 0xE3, 0xC3),
                accent: Color32::from_rgb(0xE6, 0xC4, 0x6A),
                text: Color32::from_rgb(0xE6, 0xF2, 0xE9),
                muted_text: Color32::from_rgb(0xAF, 0xC8, 0xB7),
                border: Color32::from_rgb(0x35, 0x5A, 0x43),
                selection: Color32::from_rgb(0x3D, 0x8B, 0x62),
                hover: Color32::from_rgb(0x2D, 0x50, 0x39),
                active: Color32::from_rgb(0x3D, 0x8B, 0x62),
                disabled: Color32::from_rgb(0x2E, 0x4C, 0x39),
                focus: Color32::from_rgb(0xB6, 0xE3, 0xC3),
                error: Color32::from_rgb(0xF3, 0x8B, 0x8B),
                warning: Color32::from_rgb(0xE6, 0xC4, 0x6A),
                success: Color32::from_rgb(0x5A, 0xAE, 0x7E),
            },
            Self::Light => ThemeTokens {
                background: Color32::from_rgb(0xF4, 0xF7, 0xF5),
                surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                elevated_surface: Color32::from_rgb(0xE8, 0xF0, 0xEB),
                primary: Color32::from_rgb(0x23, 0x7A, 0x50),
                secondary: Color32::from_rgb(0x3E, 0x8E, 0x64),
                accent: Color32::from_rgb(0xA6, 0x6B, 0x00),
                text: Color32::from_rgb(0x1D, 0x2B, 0x23),
                muted_text: Color32::from_rgb(0x62, 0x73, 0x6A),
                border: Color32::from_rgb(0xD6, 0xE1, 0xD9),
                selection: Color32::from_rgb(0x23, 0x7A, 0x50),
                hover: Color32::from_rgb(0xDF, 0xEB, 0xE3),
                active: Color32::from_rgb(0x23, 0x7A, 0x50),
                disabled: Color32::from_rgb(0xE4, 0xEB, 0xE6),
                focus: Color32::from_rgb(0x23, 0x7A, 0x50),
                error: Color32::from_rgb(0xB0, 0x00, 0x20),
                warning: Color32::from_rgb(0xA6, 0x6B, 0x00),
                success: Color32::from_rgb(0x23, 0x7A, 0x50),
            },
        }
    }
}

/// Semantic colors shared by every Faseela widget.
///
/// Each field describes a role rather than a fixed color, so components can remain theme-agnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeTokens {
    /// Application background, behind panels and windows.
    pub background: Color32,
    /// Ordinary surface, such as a panel or card.
    pub surface: Color32,
    /// Raised surface, such as a hovered card or menu.
    pub elevated_surface: Color32,
    /// Main action and emphasis color.
    pub primary: Color32,
    /// Secondary emphasis, links, and highlights.
    pub secondary: Color32,
    /// Decorative accent, such as a status highlight.
    pub accent: Color32,
    /// Primary body text.
    pub text: Color32,
    /// De-emphasized text and captions.
    pub muted_text: Color32,
    /// Separators and control borders.
    pub border: Color32,
    /// Selected rows, tabs, and text selection.
    pub selection: Color32,
    /// Pointer-hover fill.
    pub hover: Color32,
    /// Pressed or active fill.
    pub active: Color32,
    /// Fill for disabled controls.
    pub disabled: Color32,
    /// Keyboard focus indicator.
    pub focus: Color32,
    /// Error status.
    pub error: Color32,
    /// Warning status.
    pub warning: Color32,
    /// Success status.
    pub success: Color32,
}

impl ThemeTokens {
    /// A readable foreground for text placed on `color`.
    ///
    /// Uses perceptual luminance so a light theme's dark green primary still gets white text,
    /// while a light badge fill gets dark text.
    pub fn on_color(color: Color32) -> Color32 {
        let luminance = 0.2126 * f32::from(color.r())
            + 0.7152 * f32::from(color.g())
            + 0.0722 * f32::from(color.b());
        if luminance > 150.0 {
            Color32::from_rgb(0x1D, 0x2B, 0x23)
        } else {
            Color32::WHITE
        }
    }
}

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
    let focus = Stroke::new(1.5, tokens.focus);
    let primary_stroke = Stroke::new(1.0, tokens.primary);
    let text_stroke = Stroke::new(1.0, tokens.text);
    let on_primary = Stroke::new(1.0, ThemeTokens::on_color(tokens.active));

    visuals.panel_fill = tokens.background;
    visuals.window_fill = tokens.surface;
    visuals.extreme_bg_color = if preset == ThemePreset::Light {
        tokens.elevated_surface
    } else {
        tokens.background.gamma_multiply(0.65)
    };
    visuals.faint_bg_color = tokens.elevated_surface;
    visuals.selection.bg_fill = tokens.selection;
    visuals.selection.stroke = focus;
    visuals.hyperlink_color = tokens.secondary;
    visuals.warn_fg_color = tokens.warning;
    visuals.error_fg_color = tokens.error;
    visuals.widgets.noninteractive.bg_fill = tokens.surface;
    visuals.widgets.noninteractive.bg_stroke = border;
    // egui reuses `noninteractive` for disabled controls: muted foreground signals that state.
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, tokens.muted_text);
    visuals.widgets.inactive.bg_fill = tokens.surface;
    visuals.widgets.inactive.bg_stroke = border;
    visuals.widgets.inactive.fg_stroke = text_stroke;
    visuals.widgets.hovered.bg_fill = tokens.hover;
    visuals.widgets.hovered.bg_stroke = primary_stroke;
    visuals.widgets.hovered.fg_stroke = text_stroke;
    visuals.widgets.active.bg_fill = tokens.active;
    visuals.widgets.active.bg_stroke = focus;
    visuals.widgets.active.fg_stroke = on_primary;
    visuals.widgets.open.bg_fill = tokens.elevated_surface;
    visuals.widgets.open.bg_stroke = primary_stroke;
    visuals.widgets.open.fg_stroke = text_stroke;
    visuals.window_stroke = border;
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
    ctx.data_mut(|data| data.insert_temp(preset_id(), preset));
}

/// Backwards-compatible shortcut for the default Faseela dark theme.
pub fn apply_dark(ctx: &Context) {
    apply_preset(ctx, ThemePreset::FaseelaDark);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(color: Color32) -> f32 {
        0.2126 * f32::from(color.r())
            + 0.7152 * f32::from(color.g())
            + 0.0722 * f32::from(color.b())
    }

    #[test]
    fn applies_faseela_palette() {
        let ctx = Context::default();
        apply_dark(&ctx);
        let visuals = ctx.style_of(Theme::Dark).visuals.clone();
        assert_eq!(visuals.panel_fill, palette::BACKGROUND);
        assert_eq!(visuals.selection.bg_fill, palette::PRIMARY);
        assert_eq!(visuals.widgets.active.fg_stroke.color, Color32::WHITE);
        assert_eq!(visuals.hyperlink_color, palette::PRIMARY_LIGHT);
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
        assert_eq!(
            visuals.widgets.hovered.bg_fill,
            ThemePreset::Light.tokens().hover
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
            let tokens = preset.tokens();

            assert_eq!(visuals.widgets.inactive.fg_stroke.color, tokens.text);
            assert_eq!(
                visuals.widgets.noninteractive.fg_stroke.color,
                tokens.muted_text
            );
            assert_eq!(visuals.widgets.active.fg_stroke.color, Color32::WHITE);
            assert_eq!(visuals.widgets.hovered.bg_fill, tokens.hover);
            assert_eq!(visuals.selection.bg_fill, tokens.selection);
            assert_eq!(visuals.error_fg_color, tokens.error);
            assert_eq!(visuals.warn_fg_color, tokens.warning);
            assert!(visuals.window_stroke.width > 0.0);
        }
    }

    #[test]
    fn all_tokens_are_opaque_and_have_readable_text() {
        for preset in ThemePreset::ALL {
            let tokens = preset.tokens();
            for color in [
                tokens.background,
                tokens.surface,
                tokens.elevated_surface,
                tokens.primary,
                tokens.secondary,
                tokens.accent,
                tokens.text,
                tokens.muted_text,
                tokens.border,
                tokens.selection,
                tokens.hover,
                tokens.active,
                tokens.disabled,
                tokens.focus,
                tokens.error,
                tokens.warning,
                tokens.success,
            ] {
                assert_eq!(color.a(), 255, "{preset:?} has a translucent token");
            }

            let contrast = |a: Color32, b: Color32| (luminance(a) - luminance(b)).abs();
            assert!(
                contrast(tokens.text, tokens.background) > 80.0,
                "{preset:?} text must contrast with its background"
            );
            assert!(
                contrast(tokens.text, tokens.surface) > 80.0,
                "{preset:?} text must contrast with its surface"
            );
            assert!(
                contrast(tokens.muted_text, tokens.background) > 40.0,
                "{preset:?} muted text must remain readable"
            );
        }
    }

    #[test]
    fn light_preset_is_a_light_palette_not_a_dark_background() {
        let light = ThemePreset::Light.tokens();
        let dark = ThemePreset::FaseelaDark.tokens();
        assert!(luminance(light.background) > luminance(light.text));
        assert!(luminance(light.surface) > luminance(light.text));
        assert!(luminance(dark.background) < luminance(dark.text));
        assert_ne!(light.primary, dark.primary);
    }

    #[test]
    fn active_tokens_follow_the_last_applied_preset() {
        let ctx = Context::default();
        assert_eq!(active_preset(&ctx), ThemePreset::FaseelaDark);
        assert_eq!(active_tokens(&ctx), ThemePreset::FaseelaDark.tokens());

        for preset in ThemePreset::ALL {
            apply_preset(&ctx, preset);
            assert_eq!(active_preset(&ctx), preset);
            assert_eq!(active_tokens(&ctx), preset.tokens());
        }
    }

    #[test]
    fn switching_theme_updates_the_current_ui_style() {
        let ctx = Context::default();
        let mut dark_fill = None;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            apply_preset(ui.ctx(), ThemePreset::FaseelaDark);
            ui.reset_style();
            dark_fill = Some(ui.visuals().panel_fill);

            apply_preset(ui.ctx(), ThemePreset::Light);
            // `Ui` caches its style at creation; `reset_style` makes the switch immediate.
            assert_eq!(ui.visuals().panel_fill, dark_fill.unwrap());
            ui.reset_style();
            assert_eq!(
                ui.visuals().panel_fill,
                ThemePreset::Light.tokens().background
            );
            assert_ne!(ui.visuals().panel_fill, dark_fill.unwrap());
        });
        output.textures_delta.clear();
    }

    #[test]
    fn contrast_text_is_chosen_from_luminance() {
        assert_eq!(
            ThemeTokens::on_color(Color32::from_rgb(0x23, 0x7A, 0x50)),
            Color32::WHITE
        );
        assert_eq!(
            ThemeTokens::on_color(Color32::from_rgb(0xF2, 0xC1, 0x4E)),
            Color32::from_rgb(0x1D, 0x2B, 0x23)
        );
    }

    fn missing_glyphs(
        font: &'static [u8],
        characters: impl IntoIterator<Item = char>,
    ) -> Vec<char> {
        let face = ttf_parser::Face::parse(font, 0).expect("bundled font must parse");
        characters
            .into_iter()
            .filter(|character| face.glyph_index(*character).is_none())
            .collect()
    }

    #[test]
    fn noto_sans_arabic_covers_core_arabic_ui_characters() {
        let required = ('\u{0621}'..='\u{063A}') // hamza..ghain
            .chain('\u{0641}'..='\u{064A}') // feh..yeh
            .chain('\u{064B}'..='\u{0652}') // harakat
            .chain('\u{0660}'..='\u{0669}') // Arabic-Indic digits
            .chain(['\u{060C}', '\u{061B}', '\u{061F}']); // comma, semicolon, question mark

        assert_eq!(
            missing_glyphs(uzor_fonts::NOTO_SANS_ARABIC, required),
            Vec::<char>::new()
        );
    }

    #[test]
    fn noto_sans_arabic_covers_persian_and_urdu_additions() {
        let required = [
            '\u{067E}', // peh
            '\u{0686}', // tcheh
            '\u{0698}', // jeh
            '\u{06A9}', // keheh
            '\u{06AF}', // gaf
            '\u{06CC}', // farsi yeh
            '\u{06F0}', '\u{06F9}', // extended Arabic-Indic digits
        ];

        assert_eq!(
            missing_glyphs(uzor_fonts::NOTO_SANS_ARABIC, required),
            Vec::<char>::new()
        );
    }

    #[test]
    fn dejavu_fallback_covers_printable_ascii() {
        assert_eq!(
            missing_glyphs(dejavu::sans::regular(), ' '..='~'),
            Vec::<char>::new()
        );
    }

    #[test]
    fn every_preset_has_a_stable_label_and_index() {
        assert_eq!(ThemePreset::from_index(3), ThemePreset::Light);
        assert_eq!(ThemePreset::Light.label(), "Light");
        assert_eq!(ThemePreset::ALL.len(), 4);
        for (index, preset) in ThemePreset::ALL.into_iter().enumerate() {
            assert_eq!(ThemePreset::from_index(index), preset);
        }
    }
}
