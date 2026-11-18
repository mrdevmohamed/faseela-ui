//! Shared button metrics, state resolution, and styles.
use crate::theme::{self, ThemeTokens, blend};
use egui::{Color32, CornerRadius, Rect, Stroke, Vec2};

/// Visual treatment of a text button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// Filled with [`ThemeTokens::primary`]. Use for the single most important action.
    #[default]
    Primary,
    /// Filled with [`ThemeTokens::secondary`]. Use for supporting emphasis.
    Secondary,
    /// Transparent with a visible border and primary-colored text.
    Outline,
    /// Transparent until hovered. Use for low-emphasis actions.
    Ghost,
    /// Filled with [`ThemeTokens::error`]. Use for destructive actions such as delete.
    Destructive,
}

/// Control size shared by buttons and form controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ControlSize {
    /// Compact, for dense toolbars. Intended for pointer use with a visual focus ring.
    Small,
    /// Default size for ordinary forms and actions.
    #[default]
    Medium,
    /// Prominent size for primary calls to action.
    Large,
}

impl ControlSize {
    /// Height in logical points.
    pub const fn height(self) -> f32 {
        match self {
            Self::Small => 26.0,
            Self::Medium => 32.0,
            Self::Large => 42.0,
        }
    }

    /// Horizontal and vertical content padding in logical points.
    pub const fn padding(self) -> Vec2 {
        match self {
            Self::Small => Vec2::new(8.0, 3.0),
            Self::Medium => Vec2::new(12.0, 6.0),
            Self::Large => Vec2::new(16.0, 9.0),
        }
    }

    /// Label font size in logical points.
    pub const fn font_size(self) -> f32 {
        match self {
            Self::Small => 12.0,
            Self::Medium => 13.0,
            Self::Large => 15.0,
        }
    }

    /// Corner radius in logical points.
    pub const fn corner_radius(self) -> CornerRadius {
        match self {
            Self::Small => CornerRadius::same(7),
            Self::Medium => CornerRadius::same(8),
            Self::Large => CornerRadius::same(9),
        }
    }
}

/// The background, border and icon/text color for one interaction state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonVisuals {
    /// Background fill.
    pub fill: Color32,
    /// Border stroke.
    pub stroke: Stroke,
    /// Icon and label color.
    pub icon: Color32,
}

impl ButtonVisuals {
    /// Text color used by this state.
    pub const fn foreground(self) -> Color32 {
        self.icon
    }
}

/// A small style adapter, independent of an application's complete theme.
#[derive(Clone, Copy, Debug)]
pub struct IconButtonStyle {
    /// Ordinary, unfocused state.
    pub normal: ButtonVisuals,
    /// Pointer-hover state.
    pub hovered: ButtonVisuals,
    /// `None` keeps the ordinary hover/normal appearance while pressed.
    pub pressed: Option<ButtonVisuals>,
    /// Selection takes precedence over press and hover.
    pub selected: ButtonVisuals,
    /// Disabled state. `None` derives a muted state from the tokens.
    pub disabled: Option<ButtonVisuals>,
    /// Corner rounding.
    pub corner_radius: CornerRadius,
    /// An inset outline while the button has keyboard focus; `Stroke::NONE` opts out.
    pub focus_stroke: Stroke,
}

impl IconButtonStyle {
    pub(super) fn visuals(&self, selected: bool, hovered: bool, pressed: bool) -> ButtonVisuals {
        if selected {
            self.selected
        } else if let Some(visuals) = self.pressed.filter(|_| pressed) {
            visuals
        } else if hovered {
            self.hovered
        } else {
            self.normal
        }
    }

    /// Build a complete icon-button style from the active design tokens.
    ///
    /// This keeps icon-only controls visually consistent with [`super::Button`] variants and sizes.
    pub fn themed(tokens: ThemeTokens, variant: ButtonVariant, size: ControlSize) -> Self {
        let normal = variant_visuals(tokens, variant, InteractionState::normal(size));
        let hovered = variant_visuals(tokens, variant, InteractionState::hovered(size));
        let pressed = variant_visuals(tokens, variant, InteractionState::pressed(size));
        let selected = variant_visuals(tokens, variant, InteractionState::selected(size));
        let disabled = variant_visuals(tokens, variant, InteractionState::disabled(size));
        Self {
            normal,
            hovered,
            pressed: Some(pressed),
            selected,
            disabled: Some(disabled),
            corner_radius: size.corner_radius(),
            focus_stroke: Stroke::new(2.0, focus_color(tokens, variant)),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct InteractionState {
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) selected: bool,
    pub(super) disabled: bool,
    pub(super) size: ControlSize,
}

impl InteractionState {
    pub(super) const fn normal(size: ControlSize) -> Self {
        Self {
            hovered: false,
            pressed: false,
            selected: false,
            disabled: false,
            size,
        }
    }

    pub(super) const fn hovered(size: ControlSize) -> Self {
        Self {
            hovered: true,
            ..Self::normal(size)
        }
    }

    pub(super) const fn pressed(size: ControlSize) -> Self {
        Self {
            hovered: true,
            pressed: true,
            ..Self::normal(size)
        }
    }

    pub(super) const fn selected(size: ControlSize) -> Self {
        Self {
            selected: true,
            ..Self::normal(size)
        }
    }

    pub(super) const fn disabled(size: ControlSize) -> Self {
        Self {
            disabled: true,
            ..Self::normal(size)
        }
    }
}

/// Resolve a variant's fill, border, and foreground for one state.
pub(super) fn variant_visuals(
    tokens: ThemeTokens,
    variant: ButtonVariant,
    state: InteractionState,
) -> ButtonVisuals {
    let _ = state.size;
    if state.disabled {
        return ButtonVisuals {
            fill: tokens.disabled,
            stroke: Stroke::new(1.0, tokens.border),
            icon: tokens.disabled_text,
        };
    }

    if state.selected {
        return ButtonVisuals {
            fill: tokens.selection,
            stroke: Stroke::new(1.0, tokens.focus),
            icon: tokens.on_primary,
        };
    }

    let base = match variant {
        ButtonVariant::Primary => (tokens.primary, tokens.on_primary, tokens.primary),
        ButtonVariant::Secondary => (tokens.secondary, tokens.on_secondary, tokens.secondary),
        ButtonVariant::Destructive => (tokens.error, tokens.on_error, tokens.error),
        ButtonVariant::Outline => (Color32::TRANSPARENT, tokens.text, tokens.control_border),
        ButtonVariant::Ghost => (Color32::TRANSPARENT, tokens.text, Color32::TRANSPARENT),
    };

    if state.pressed {
        if matches!(variant, ButtonVariant::Outline | ButtonVariant::Ghost) {
            return ButtonVisuals {
                fill: tokens.active,
                stroke: Stroke::new(1.0, tokens.focus),
                icon: tokens.on_primary,
            };
        }
        // Press toward whichever direction increases contrast with the foreground. Dark text
        // needs a lighter fill; light text needs a darker fill.
        let foreground_is_darker =
            theme::relative_luminance(base.1) < theme::relative_luminance(base.0);
        return ButtonVisuals {
            fill: blend(
                base.0,
                if foreground_is_darker {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                },
                0.13,
            ),
            stroke: Stroke::new(1.0, base.2),
            icon: base.1,
        };
    }

    if state.hovered {
        if matches!(variant, ButtonVariant::Outline | ButtonVariant::Ghost) {
            return ButtonVisuals {
                fill: tokens.hover,
                stroke: Stroke::new(1.0, tokens.primary),
                icon: tokens.text,
            };
        }
        return ButtonVisuals {
            fill: blend(base.0, Color32::WHITE, 0.06),
            stroke: Stroke::new(1.0, tokens.focus),
            icon: base.1,
        };
    }

    ButtonVisuals {
        fill: base.0,
        stroke: Stroke::new(1.0, base.2),
        icon: base.1,
    }
}

pub(super) fn focus_color(tokens: ThemeTokens, variant: ButtonVariant) -> Color32 {
    match variant {
        ButtonVariant::Primary => tokens.on_primary,
        ButtonVariant::Secondary => tokens.on_secondary,
        ButtonVariant::Destructive => tokens.on_error,
        ButtonVariant::Outline | ButtonVariant::Ghost => tokens.focus,
    }
}

pub(super) fn bounded_stroke(stroke: Stroke, rect: Rect) -> Stroke {
    if stroke.width.is_finite() && stroke.width > 0.0 {
        Stroke::new(
            stroke.width.min(rect.width().min(rect.height()) * 0.5),
            stroke.color,
        )
    } else {
        Stroke::NONE
    }
}
