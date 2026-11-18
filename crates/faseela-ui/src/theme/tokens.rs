//! Semantic colors for shared widgets and application painting.
use super::{contrast_ratio, relative_luminance};
use egui::Color32;

/// Semantic colors shared by every Faseela widget.
///
/// Each field describes a role rather than a fixed color, so components can remain theme-agnostic.
/// Foreground tokens (`on_*`) are chosen and tested per preset instead of being derived from a
/// luminance threshold at paint time. Applications that build custom presets should keep the
/// documented contrast guarantees; see [`contrast_ratio`] and the crate's theme tests.
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
    /// Decorative separators and card outlines.
    ///
    /// This token is intentionally subtle. Use [`Self::control_border`] for interactive control
    /// boundaries that should meet the WCAG 3:1 non-text contrast requirement.
    pub border: Color32,
    /// Boundary color for checkboxes, switches, inputs, and other interactive controls.
    pub control_border: Color32,
    /// Selected rows, tabs, and text selection.
    pub selection: Color32,
    /// Pointer-hover fill.
    pub hover: Color32,
    /// Pressed or active fill.
    pub active: Color32,
    /// Fill for disabled controls.
    pub disabled: Color32,
    /// Text and icon color for disabled controls.
    pub disabled_text: Color32,
    /// Keyboard focus indicator.
    pub focus: Color32,
    /// Error status.
    pub error: Color32,
    /// Warning status.
    pub warning: Color32,
    /// Success status.
    pub success: Color32,
    /// Foreground on [`Self::primary`] fills and selected states.
    pub on_primary: Color32,
    /// Foreground on [`Self::secondary`] fills.
    pub on_secondary: Color32,
    /// Foreground on [`Self::accent`] fills.
    pub on_accent: Color32,
    /// Foreground on [`Self::success`] fills.
    pub on_success: Color32,
    /// Foreground on [`Self::warning`] fills.
    pub on_warning: Color32,
    /// Foreground on [`Self::error`] fills.
    pub on_error: Color32,
}

impl ThemeTokens {
    /// WCAG 2.x relative luminance of an opaque sRGB color.
    ///
    /// Alpha is ignored. The result is in `0.0..=1.0`; invalid or transparent colors are still
    /// evaluated by their RGB channels.
    pub fn relative_luminance(color: Color32) -> f32 {
        relative_luminance(color)
    }

    /// WCAG 2.x contrast ratio between two colors, in `1.0..=21.0`.
    pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
        contrast_ratio(a, b)
    }

    /// The higher-contrast of white and Faseela's dark ink for `color`.
    ///
    /// Uses real WCAG relative luminance. This is a best-effort helper for arbitrary
    /// application colors: mid-tone fills can still be below 4.5:1 with both candidates, so
    /// built-in presets use explicit `on_*` tokens that are tested for WCAG AA.
    pub fn on_color(color: Color32) -> Color32 {
        let dark = Color32::from_rgb(0x1D, 0x2B, 0x23);
        if contrast_ratio(Color32::WHITE, color) >= contrast_ratio(dark, color) {
            Color32::WHITE
        } else {
            dark
        }
    }

    /// Foreground token for a semantic [`BadgeTone`](crate::widgets::BadgeTone) fill.
    pub fn on_badge(self, tone: crate::widgets::BadgeTone) -> Color32 {
        use crate::widgets::BadgeTone;
        match tone {
            BadgeTone::Neutral => self.text,
            BadgeTone::Primary => self.on_primary,
            BadgeTone::Success => self.on_success,
            BadgeTone::Warning => self.on_warning,
            BadgeTone::Error => self.on_error,
        }
    }
}
