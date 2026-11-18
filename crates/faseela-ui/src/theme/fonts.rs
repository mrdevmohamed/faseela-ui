//! Embedded Unicode fallback installation, independent of color theme.
use egui::{Context, FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

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
