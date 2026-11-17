//! Reusable button primitives: text buttons, icon buttons, variants, and sizes.
//!
//! The original icon-only primitive is adapted from Craft's reusable egui controls.
//!
//! Upstream: <https://github.com/storytold/craft-libs>
//! The application owns icon assets, commands and selection state.

use std::borrow::Cow;

use egui::{
    Color32, CornerRadius, FontId, Id, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2,
    Widget, WidgetInfo,
};

use crate::theme::{self, ThemeTokens, blend};

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
    fn visuals(&self, selected: bool, hovered: bool, pressed: bool) -> ButtonVisuals {
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
    /// This keeps icon-only controls visually consistent with [`Button`] variants and sizes.
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

/// An icon-only button. The accessible label is independent of the icon and widget ID.
///
/// The surrounding UI scope/position supplies the ID; use `Ui::push_id` or [`Self::id`] for
/// application-specific identity. The caller paints the icon and dispatches commands from the
/// returned response. Selection is caller-owned and is never toggled implicitly.
pub struct IconButton<'a> {
    label: &'a str,
    size: Vec2,
    selected: bool,
    enabled: bool,
    id: Option<Id>,
    style: &'a IconButtonStyle,
    single_shape_frame: bool,
}

impl<'a> IconButton<'a> {
    /// Create an icon-only button with an accessible label, size, and caller-owned style.
    pub fn new(label: &'a str, size: Vec2, style: &'a IconButtonStyle) -> Self {
        Self {
            label,
            size,
            selected: false,
            enabled: true,
            id: None,
            style,
            single_shape_frame: false,
        }
    }

    /// Mark the button as selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disable the button while keeping its accessible role and label.
    ///
    /// `ui.add_enabled(false, ...)` also disables it; this builder is a convenience for
    /// application-owned state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign an explicit stable widget ID.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Paint fill and border as one egui rectangle instead of separate shapes.
    pub fn single_shape_frame(mut self) -> Self {
        self.single_shape_frame = true;
        self
    }

    /// Paint one button and return its mouse, keyboard and accessibility response.
    ///
    /// `paint_icon` receives the full button rectangle and resolved icon tint. Invalid,
    /// non-positive or excessively large dimensions produce an inert zero-size response.
    pub fn show(self, ui: &mut Ui, paint_icon: impl FnOnce(&Ui, Rect, Color32)) -> Response {
        if self.enabled {
            self.show_in(ui, paint_icon)
        } else {
            ui.add_enabled_ui(false, |ui| self.show_in(ui, paint_icon))
                .inner
        }
    }

    fn show_in(self, ui: &mut Ui, paint_icon: impl FnOnce(&Ui, Rect, Color32)) -> Response {
        let enabled = ui.is_enabled();
        let valid = self.size.is_finite()
            && self.size.x > 0.0
            && self.size.y > 0.0
            && self.size.x <= 4096.0
            && self.size.y <= 4096.0;
        let sense = if valid && enabled {
            Sense::click()
        } else {
            Sense::hover()
        };

        let response = if let Some(id) = self.id {
            let (_, rect) = ui.allocate_space(if valid { self.size } else { Vec2::ZERO });
            ui.interact(rect, id, sense)
        } else {
            ui.allocate_exact_size(if valid { self.size } else { Vec2::ZERO }, sense)
                .1
        };
        response.widget_info(|| {
            WidgetInfo::selected(
                Role::Button,
                valid && enabled,
                self.selected && enabled,
                self.label,
            )
        });
        if !valid {
            return response;
        }

        let rect = response.rect;
        let visuals = if enabled {
            self.style.visuals(
                self.selected,
                response.hovered(),
                response.is_pointer_button_down_on(),
            )
        } else {
            self.style.disabled.unwrap_or(ButtonVisuals {
                fill: ui.visuals().widgets.noninteractive.bg_fill,
                stroke: ui.visuals().widgets.noninteractive.bg_stroke,
                icon: ui.visuals().widgets.noninteractive.fg_stroke.color,
            })
        };
        let stroke = bounded_stroke(visuals.stroke, rect);
        if self.single_shape_frame {
            ui.painter().rect(
                rect,
                self.style.corner_radius,
                visuals.fill,
                stroke,
                StrokeKind::Inside,
            );
        } else {
            if visuals.fill != Color32::TRANSPARENT {
                ui.painter()
                    .rect_filled(rect, self.style.corner_radius, visuals.fill);
            }
            if !stroke.is_empty() {
                ui.painter().rect_stroke(
                    rect,
                    self.style.corner_radius,
                    stroke,
                    StrokeKind::Inside,
                );
            }
        }
        paint_icon(ui, rect, visuals.icon);
        if enabled && response.has_focus() && rect.width() > 4.0 && rect.height() > 4.0 {
            let focus_rect = rect.shrink(2.0);
            let stroke = bounded_stroke(self.style.focus_stroke, focus_rect);
            if !stroke.is_empty() {
                ui.painter().rect_stroke(
                    focus_rect,
                    self.style.corner_radius,
                    stroke,
                    StrokeKind::Inside,
                );
            }
        }
        response
    }
}

/// A themed text button with variants, sizes, selected and disabled states.
///
/// The returned [`Response`] is a proposal: applications own the resulting state change or
/// command. Keyboard focus, Space/Enter activation, focus indicators, and AccessKit semantics
/// are provided by egui's click interaction and this widget's [`WidgetInfo`].
///
/// ```
/// use faseela_ui::buttons::{Button, ButtonVariant, ControlSize};
///
/// egui::__run_test_ui(|ui| {
///     let response = Button::new("Save")
///         .variant(ButtonVariant::Primary)
///         .size(ControlSize::Medium)
///         .show(ui);
///     let _clicked = response.clicked();
/// });
/// ```
#[must_use = "add the button to a Ui with `.show(ui)` or `ui.add(button)`"]
pub struct Button<'a> {
    text: Cow<'a, str>,
    icon: Option<Cow<'a, str>>,
    variant: ButtonVariant,
    size: ControlSize,
    selected: bool,
    enabled: bool,
    id: Option<Id>,
    min_width: f32,
    full_width: bool,
}

impl<'a> Button<'a> {
    /// Create a button with a visible and accessible label.
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            variant: ButtonVariant::Primary,
            size: ControlSize::Medium,
            selected: false,
            enabled: true,
            id: None,
            min_width: 0.0,
            full_width: false,
        }
    }

    /// Choose the visual treatment. Defaults to [`ButtonVariant::Primary`].
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Choose the control size. Defaults to [`ControlSize::Medium`].
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Add a leading icon glyph. Applications may use any font glyph or short symbol.
    ///
    /// For image assets, use egui's `Image` buttons directly or an [`IconButton`] with a custom
    /// painter.
    pub fn icon(mut self, icon: impl Into<Cow<'a, str>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Paint the button as selected. Selection is caller-owned and never toggled implicitly.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disable the button while keeping its accessible label and role.
    ///
    /// `ui.add_enabled(false, ...)` has the same effect and composes with this builder.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable widget ID, useful for tests and keyboard focus restoration.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Request a minimum width in logical points.
    pub fn min_width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.min_width = width.min(4096.0);
        }
        self
    }

    /// Expand the button to the available width.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    /// Show the button.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        if self.enabled {
            self.show_in(ui)
        } else {
            ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner
        }
    }
}

impl Button<'_> {
    fn show_in(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let enabled = ui.is_enabled();
        let metrics = self.size;
        let font = FontId::proportional(metrics.font_size());
        let icon_font = FontId::proportional((metrics.font_size() + 1.0).min(18.0));

        let text_galley = ui.fonts_mut(|fonts| {
            fonts.layout_no_wrap(self.text.to_string(), font.clone(), tokens.text)
        });
        let icon_galley = self.icon.as_ref().map(|icon| {
            ui.fonts_mut(|fonts| {
                fonts.layout_no_wrap(icon.to_string(), icon_font.clone(), tokens.text)
            })
        });

        let gap = icon_galley.as_ref().map_or(0.0, |_| 6.0);
        let icon_width = icon_galley
            .as_ref()
            .map_or(0.0, |galley| galley.rect.width());
        let padding = metrics.padding();
        let content_width = icon_width + gap + text_galley.rect.width();
        let desired_width = content_width + padding.x * 2.0;
        let width = if self.full_width {
            ui.available_width().max(desired_width.max(self.min_width))
        } else {
            desired_width.max(self.min_width)
        };
        let height = metrics
            .height()
            .max(text_galley.rect.height().max(icon_width) + padding.y * 2.0);
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };

        let (rect, response) = if let Some(id) = self.id {
            let (_, rect) = ui.allocate_space(Vec2::new(width, height));
            let response = ui.interact(rect, id, sense);
            (rect, response)
        } else {
            ui.allocate_exact_size(Vec2::new(width, height), sense)
        };

        let pressed = enabled && response.is_pointer_button_down_on();
        let state = InteractionState {
            hovered: enabled && response.hovered(),
            pressed,
            selected: self.selected && enabled,
            disabled: !enabled,
            size: metrics,
        };
        let visuals = variant_visuals(tokens, self.variant, state);
        response
            .widget_info(|| WidgetInfo::selected(Role::Button, enabled, self.selected, &self.text));
        if enabled && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let corner_radius = metrics.corner_radius();
        if visuals.fill != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, corner_radius, visuals.fill);
        }
        if !visuals.stroke.is_empty() {
            ui.painter().rect_stroke(
                rect,
                corner_radius,
                bounded_stroke(visuals.stroke, rect),
                StrokeKind::Inside,
            );
        }

        let mut x = rect.center().x - content_width * 0.5;
        let center_y = rect.center().y;
        let rtl = crate::direction::is_rtl(ui);
        let icon_rect = icon_galley.as_ref().map(|galley| {
            let size = galley.rect.size();
            (size, galley.clone())
        });
        if rtl {
            // Leading content is on the right in an RTL interface.
            let text_pos = egui::pos2(x, center_y - text_galley.rect.height() * 0.5);
            ui.painter().galley_with_override_text_color(
                text_pos,
                text_galley.clone(),
                visuals.icon,
            );
            x += text_galley.rect.width() + gap;
            if let Some((size, galley)) = icon_rect {
                ui.painter().galley_with_override_text_color(
                    egui::pos2(x, center_y - size.y * 0.5),
                    galley,
                    visuals.icon,
                );
            }
        } else {
            if let Some((size, galley)) = icon_rect {
                ui.painter().galley_with_override_text_color(
                    egui::pos2(x, center_y - size.y * 0.5),
                    galley,
                    visuals.icon,
                );
                x += size.x + gap;
            }
            ui.painter().galley_with_override_text_color(
                egui::pos2(x, center_y - text_galley.rect.height() * 0.5),
                text_galley,
                visuals.icon,
            );
        }

        if enabled && response.has_focus() && rect.width() > 4.0 && rect.height() > 4.0 {
            let focus_rect = rect.shrink(2.0);
            let ring = bounded_stroke(
                Stroke::new(2.0, focus_color(tokens, self.variant)),
                focus_rect,
            );
            if !ring.is_empty() {
                ui.painter().rect_stroke(
                    focus_rect,
                    metrics.corner_radius(),
                    ring,
                    StrokeKind::Inside,
                );
            }
        }

        response
    }
}

#[derive(Clone, Copy, Debug)]
struct InteractionState {
    hovered: bool,
    pressed: bool,
    selected: bool,
    disabled: bool,
    size: ControlSize,
}

impl InteractionState {
    const fn normal(size: ControlSize) -> Self {
        Self {
            hovered: false,
            pressed: false,
            selected: false,
            disabled: false,
            size,
        }
    }

    const fn hovered(size: ControlSize) -> Self {
        Self {
            hovered: true,
            ..Self::normal(size)
        }
    }

    const fn pressed(size: ControlSize) -> Self {
        Self {
            hovered: true,
            pressed: true,
            ..Self::normal(size)
        }
    }

    const fn selected(size: ControlSize) -> Self {
        Self {
            selected: true,
            ..Self::normal(size)
        }
    }

    const fn disabled(size: ControlSize) -> Self {
        Self {
            disabled: true,
            ..Self::normal(size)
        }
    }
}

/// Resolve a variant's fill, border, and foreground for one state.
fn variant_visuals(
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

fn focus_color(tokens: ThemeTokens, variant: ButtonVariant) -> Color32 {
    match variant {
        ButtonVariant::Primary => tokens.on_primary,
        ButtonVariant::Secondary => tokens.on_secondary,
        ButtonVariant::Destructive => tokens.on_error,
        ButtonVariant::Outline | ButtonVariant::Ghost => tokens.focus,
    }
}

fn bounded_stroke(stroke: Stroke, rect: Rect) -> Stroke {
    if stroke.width.is_finite() && stroke.width > 0.0 {
        Stroke::new(
            stroke.width.min(rect.width().min(rect.height()) * 0.5),
            stroke.color,
        )
    } else {
        Stroke::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemePreset, contrast_ratio};
    use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2, vec2};

    fn visuals(fill: Color32) -> ButtonVisuals {
        ButtonVisuals {
            fill,
            stroke: Stroke::NONE,
            icon: fill,
        }
    }

    fn style() -> IconButtonStyle {
        IconButtonStyle {
            normal: visuals(Color32::BLACK),
            hovered: visuals(Color32::GREEN),
            pressed: Some(visuals(Color32::BLUE)),
            selected: visuals(Color32::YELLOW),
            disabled: Some(visuals(Color32::GRAY)),
            corner_radius: CornerRadius::same(4),
            focus_stroke: Stroke::NONE,
        }
    }

    fn frame(ctx: &Context, events: Vec<Event>, add: impl FnMut(&mut Ui)) -> egui::FullOutput {
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 200.0))),
                events,
                ..Default::default()
            },
            add,
        );
        output.textures_delta.clear();
        output
    }

    fn click_at(pos: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    #[test]
    fn selected_visuals_take_precedence_over_hover_and_press() {
        assert_eq!(style().visuals(true, true, true), visuals(Color32::YELLOW));
    }

    #[test]
    fn pressed_and_hovered_visuals_are_resolved() {
        let style = style();
        assert_eq!(style.visuals(false, true, true), visuals(Color32::BLUE));
        assert_eq!(style.visuals(false, true, false), visuals(Color32::GREEN));
        assert_eq!(style.visuals(false, false, false), visuals(Color32::BLACK));
    }

    #[test]
    fn invalid_strokes_are_ignored_and_large_strokes_are_bounded() {
        let rect = Rect::from_min_size(egui::Pos2::ZERO, Vec2::splat(20.0));
        assert_eq!(
            bounded_stroke(Stroke::new(f32::NAN, Color32::WHITE), rect),
            Stroke::NONE
        );
        assert_eq!(
            bounded_stroke(Stroke::new(f32::MAX, Color32::WHITE), rect).width,
            10.0
        );
    }

    #[test]
    fn control_sizes_have_predictable_order() {
        assert!(ControlSize::Small.height() < ControlSize::Medium.height());
        assert!(ControlSize::Medium.height() < ControlSize::Large.height());
        assert!(ControlSize::Small.font_size() < ControlSize::Large.font_size());
    }

    #[test]
    fn all_variants_and_states_keep_foreground_contrast() {
        for preset in ThemePreset::ALL {
            let tokens = preset.tokens();
            for variant in [
                ButtonVariant::Primary,
                ButtonVariant::Secondary,
                ButtonVariant::Outline,
                ButtonVariant::Ghost,
                ButtonVariant::Destructive,
            ] {
                for state in [
                    InteractionState::normal(ControlSize::Medium),
                    InteractionState::hovered(ControlSize::Medium),
                    InteractionState::pressed(ControlSize::Medium),
                    InteractionState::selected(ControlSize::Medium),
                ] {
                    let visuals = variant_visuals(tokens, variant, state);
                    if visuals.fill != Color32::TRANSPARENT {
                        let ratio = contrast_ratio(visuals.icon, visuals.fill);
                        assert!(
                            ratio >= 4.5,
                            "{preset:?} {variant:?} {state:?} has {ratio:.2}:1 text contrast"
                        );
                    }
                }
                let disabled = variant_visuals(
                    tokens,
                    variant,
                    InteractionState::disabled(ControlSize::Medium),
                );
                assert!(
                    contrast_ratio(disabled.icon, disabled.fill) >= 3.0,
                    "{preset:?} {variant:?} disabled text must remain legible"
                );
            }
        }
    }

    #[test]
    fn disabled_text_button_cannot_be_clicked() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
        let mut rect = Rect::NOTHING;
        let output = frame(&ctx, vec![], |ui| {
            let response = Button::new("Disabled").enabled(false).show(ui);
            rect = response.rect;
            assert!(!response.enabled());
        });
        assert!(!output.shapes.is_empty());
        let _ = frame(&ctx, click_at(rect.center()), |ui| {
            let response = Button::new("Disabled").enabled(false).show(ui);
            assert!(!response.clicked());
            assert!(!response.enabled());
        });
    }

    #[test]
    fn pointer_click_activates_enabled_button_once() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let mut rect = Rect::NOTHING;
        let _ = frame(&ctx, vec![], |ui| {
            rect = Button::new("Save").show(ui).rect;
        });
        let mut clicks = 0;
        let _ = frame(&ctx, click_at(rect.center()), |ui| {
            if Button::new("Save").show(ui).clicked() {
                clicks += 1;
            }
        });
        assert_eq!(clicks, 1);
    }

    #[test]
    fn keyboard_space_and_enter_activate_a_focused_button() {
        for key in [egui::Key::Space, egui::Key::Enter] {
            let ctx = Context::default();
            theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
            let id = Id::unique("keyboard-button");
            let mut rect = Rect::NOTHING;
            let _ = frame(&ctx, vec![], |ui| {
                rect = Button::new("Activate").id(id).show(ui).rect;
                ui.memory_mut(|memory| memory.request_focus(id));
            });
            let mut clicks = 0;
            let _ = frame(
                &ctx,
                vec![Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }],
                |ui| {
                    if Button::new("Activate").id(id).show(ui).clicked() {
                        clicks += 1;
                    }
                },
            );
            assert_eq!(clicks, 1, "{key:?} should activate the focused button");
            let _ = rect;
        }
    }

    #[test]
    fn icon_button_reports_selected_state_and_disabled_state() {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let style = IconButtonStyle::themed(
            ThemePreset::Light.tokens(),
            ButtonVariant::Outline,
            ControlSize::Medium,
        );
        let mut output = frame(&ctx, vec![], |ui| {
            let response = IconButton::new("Toggle panel", vec2(32.0, 32.0), &style)
                .selected(true)
                .show(ui, |_, _, _| {});
            assert!(response.enabled());
            let disabled = IconButton::new("Unavailable", vec2(32.0, 32.0), &style)
                .enabled(false)
                .show(ui, |_, _, _| {});
            assert!(!disabled.enabled());
            assert!(!disabled.clicked());
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn themed_icon_style_matches_text_button_variant() {
        for preset in ThemePreset::ALL {
            let tokens = preset.tokens();
            let style =
                IconButtonStyle::themed(tokens, ButtonVariant::Destructive, ControlSize::Large);
            assert_eq!(style.normal.fill, tokens.error);
            assert_eq!(style.normal.icon, tokens.on_error);
            assert_eq!(
                style.disabled.expect("disabled style").fill,
                tokens.disabled
            );
            assert_eq!(style.corner_radius, ControlSize::Large.corner_radius());
        }
    }
}
