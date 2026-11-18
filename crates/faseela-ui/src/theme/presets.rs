//! Built-in Faseela palettes; official brand colors remain stable.
use super::ThemeTokens;
use egui::Color32;

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
                primary: palette::PRIMARY,
                secondary: palette::PRIMARY_LIGHT,
                accent: palette::ACCENT,
                text: Color32::from_rgb(0xE8, 0xF7, 0xEE),
                muted_text: Color32::from_rgb(0xA9, 0xC1, 0xB7),
                border: Color32::from_rgb(0x2D, 0x51, 0x45),
                control_border: Color32::from_rgb(0x6B, 0x91, 0x80),
                selection: palette::PRIMARY,
                hover: Color32::from_rgb(0x20, 0x46, 0x3A),
                active: palette::PRIMARY,
                disabled: Color32::from_rgb(0x2E, 0x47, 0x3C),
                disabled_text: Color32::from_rgb(0x91, 0xA8, 0x9E),
                focus: palette::PRIMARY_LIGHT,
                error: Color32::from_rgb(0xFF, 0x7B, 0x7B),
                warning: palette::ACCENT,
                success: Color32::from_rgb(0x4F, 0xC1, 0x84),
                // Near-black brand green: 5.46:1 on the official primary green.
                on_primary: Color32::from_rgb(0x0A, 0x1D, 0x1A),
                on_secondary: Color32::from_rgb(0x0A, 0x1D, 0x1A),
                on_accent: Color32::from_rgb(0x1D, 0x2B, 0x23),
                on_success: Color32::from_rgb(0x06, 0x17, 0x0E),
                on_warning: Color32::from_rgb(0x1D, 0x2B, 0x23),
                on_error: Color32::from_rgb(0x26, 0x09, 0x0A),
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
                control_border: Color32::from_rgb(0x74, 0x80, 0xA8),
                selection: Color32::from_rgb(0x68, 0x75, 0xD9),
                hover: Color32::from_rgb(0x31, 0x3C, 0x5A),
                active: Color32::from_rgb(0x68, 0x75, 0xD9),
                disabled: Color32::from_rgb(0x2E, 0x36, 0x52),
                disabled_text: Color32::from_rgb(0x8E, 0x98, 0xB8),
                focus: Color32::from_rgb(0xC0, 0xC7, 0xFF),
                error: Color32::from_rgb(0xFF, 0x8A, 0x97),
                warning: Color32::from_rgb(0xE7, 0xB9, 0x6A),
                success: Color32::from_rgb(0x63, 0xC9, 0x94),
                on_primary: Color32::from_rgb(0x0A, 0x0D, 0x18),
                on_secondary: Color32::from_rgb(0x10, 0x14, 0x26),
                on_accent: Color32::from_rgb(0x1D, 0x20, 0x30),
                on_success: Color32::from_rgb(0x06, 0x17, 0x0E),
                on_warning: Color32::from_rgb(0x1D, 0x20, 0x30),
                on_error: Color32::from_rgb(0x26, 0x09, 0x0C),
            },
            Self::Forest => ThemeTokens {
                background: Color32::from_rgb(0x10, 0x24, 0x1C),
                surface: Color32::from_rgb(0x18, 0x32, 0x26),
                elevated_surface: Color32::from_rgb(0x24, 0x43, 0x31),
                // Slightly lifted from #3D8B62 so indicators reach 3:1 on elevated surfaces.
                primary: Color32::from_rgb(0x46, 0x99, 0x6C),
                secondary: Color32::from_rgb(0xB6, 0xE3, 0xC3),
                accent: Color32::from_rgb(0xE6, 0xC4, 0x6A),
                text: Color32::from_rgb(0xE6, 0xF2, 0xE9),
                muted_text: Color32::from_rgb(0xAF, 0xC8, 0xB7),
                border: Color32::from_rgb(0x35, 0x5A, 0x43),
                control_border: Color32::from_rgb(0x6E, 0x9A, 0x7E),
                selection: Color32::from_rgb(0x46, 0x99, 0x6C),
                hover: Color32::from_rgb(0x2D, 0x50, 0x39),
                active: Color32::from_rgb(0x46, 0x99, 0x6C),
                disabled: Color32::from_rgb(0x2E, 0x4C, 0x39),
                disabled_text: Color32::from_rgb(0x91, 0xAC, 0x9B),
                focus: Color32::from_rgb(0xB6, 0xE3, 0xC3),
                error: Color32::from_rgb(0xF3, 0x8B, 0x8B),
                warning: Color32::from_rgb(0xE6, 0xC4, 0x6A),
                success: Color32::from_rgb(0x5A, 0xAE, 0x7E),
                on_primary: Color32::from_rgb(0x06, 0x12, 0x0E),
                on_secondary: Color32::from_rgb(0x10, 0x24, 0x1C),
                on_accent: Color32::from_rgb(0x1D, 0x2B, 0x23),
                on_success: Color32::from_rgb(0x06, 0x12, 0x0E),
                on_warning: Color32::from_rgb(0x1D, 0x2B, 0x23),
                on_error: Color32::from_rgb(0x26, 0x09, 0x0C),
            },
            Self::Light => ThemeTokens {
                background: Color32::from_rgb(0xF4, 0xF7, 0xF5),
                surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                elevated_surface: Color32::from_rgb(0xE8, 0xF0, 0xEB),
                primary: Color32::from_rgb(0x23, 0x7A, 0x50),
                // Darkened from #3E8E64 to keep links and secondary text above 4.5:1.
                secondary: Color32::from_rgb(0x2B, 0x6B, 0x49),
                // Darkened from #A66B00 so normal-size white text reaches 5.68:1.
                accent: Color32::from_rgb(0x8F, 0x5C, 0x00),
                text: Color32::from_rgb(0x1D, 0x2B, 0x23),
                muted_text: Color32::from_rgb(0x55, 0x66, 0x5D),
                border: Color32::from_rgb(0xD6, 0xE1, 0xD9),
                control_border: Color32::from_rgb(0x6B, 0x7F, 0x73),
                selection: Color32::from_rgb(0x23, 0x7A, 0x50),
                hover: Color32::from_rgb(0xDF, 0xEB, 0xE3),
                active: Color32::from_rgb(0x23, 0x7A, 0x50),
                disabled: Color32::from_rgb(0xE4, 0xEB, 0xE6),
                disabled_text: Color32::from_rgb(0x5F, 0x6F, 0x66),
                focus: Color32::from_rgb(0x23, 0x7A, 0x50),
                error: Color32::from_rgb(0xB0, 0x00, 0x20),
                warning: Color32::from_rgb(0x8F, 0x5C, 0x00),
                success: Color32::from_rgb(0x23, 0x7A, 0x50),
                on_primary: Color32::WHITE,
                on_secondary: Color32::WHITE,
                on_accent: Color32::WHITE,
                on_success: Color32::WHITE,
                on_warning: Color32::WHITE,
                on_error: Color32::WHITE,
            },
        }
    }
}
