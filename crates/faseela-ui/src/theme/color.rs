//! WCAG contrast calculations and bounded sRGB interpolation.
use egui::Color32;

/// Linearize one sRGB channel component in `0..=1`.
fn linearize(channel: u8) -> f32 {
    let channel = f32::from(channel) / 255.0;
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2.x relative luminance for an sRGB color.
pub fn relative_luminance(color: Color32) -> f32 {
    0.2126 * linearize(color.r()) + 0.7152 * linearize(color.g()) + 0.0722 * linearize(color.b())
}

/// WCAG 2.x contrast ratio between two opaque sRGB colors.
///
/// The ratio is symmetric and always at least 1.0. Normal text needs 4.5:1, large text and
/// meaningful non-text indicators need 3:1.
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let (lighter, darker) = {
        let a = relative_luminance(a);
        let b = relative_luminance(b);
        if a >= b { (a, b) } else { (b, a) }
    };
    (lighter + 0.05) / (darker + 0.05)
}

/// Mix two colors in sRGB space. `amount` is clamped to `0.0..=1.0`.
///
/// This is intended for subtle hover and pressed tints, not for perceptual palette generation.
pub fn blend(from: Color32, to: Color32, amount: f32) -> Color32 {
    let amount = if amount.is_finite() {
        amount.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * amount).round() as u8;
    Color32::from_rgba_premultiplied(
        mix(from.r(), to.r()),
        mix(from.g(), to.g()),
        mix(from.b(), to.b()),
        mix(from.a(), to.a()),
    )
}
