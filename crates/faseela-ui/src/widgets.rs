//! Reusable, theme-aware presentation widgets.
//!
//! Every widget in this module reads its colors from [`crate::theme::active_tokens`], so a theme
//! switch takes effect on the next frame without callers rebuilding their styles. Widgets return
//! proposals (for example which toolbar item was clicked); applications keep ownership of state,
//! commands, and selection.

use crate::{
    direction::HasDirection,
    layout::{ToolbarFit, fit_toolbar},
    theme::{self, ThemeTokens},
};
use egui::{
    Align2, Button, Color32, CornerRadius, FontId, Id, InnerResponse, Layout, Margin, Response,
    RichText, Sense, Stroke, TextStyle, Ui, Vec2, Widget, WidgetInfo,
};

/// A themed surface container used to group related content.
///
/// The ordinary fill is [`ThemeTokens::surface`]; [`Card::elevated`] uses
/// [`ThemeTokens::elevated_surface`]. Both use [`ThemeTokens::border`] for the outline unless
/// overridden.
///
/// ```
/// use faseela_ui::widgets::Card;
///
/// egui::__run_test_ui(|ui| {
///     Card::new().show(ui, |ui| {
///         ui.label("Grouped content inherits the active theme.");
///     });
/// });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Card {
    elevated: bool,
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    corner_radius: CornerRadius,
    padding: Margin,
    outer_margin: Margin,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            elevated: false,
            fill: None,
            stroke: None,
            corner_radius: CornerRadius::same(10),
            padding: Margin::same(14),
            outer_margin: Margin::same(0),
        }
    }
}

impl Card {
    /// A standard card with the active theme's surface color.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use the raised surface color, for example for a selected card.
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    /// Override the computed fill color.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Override the computed border stroke.
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Override the corner rounding of the card.
    pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    /// Padding between the card border and its contents.
    pub fn padding(mut self, padding: Margin) -> Self {
        self.padding = padding;
        self
    }

    /// Margin outside the card border.
    pub fn outer_margin(mut self, outer_margin: Margin) -> Self {
        self.outer_margin = outer_margin;
        self
    }

    /// Fill used before any override, for tests and callers matching custom painting.
    pub fn computed_fill(self, tokens: ThemeTokens) -> Color32 {
        self.fill.unwrap_or(if self.elevated {
            tokens.elevated_surface
        } else {
            tokens.surface
        })
    }

    /// Show the card and its contents.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let tokens = theme::active_tokens(ui.ctx());
        egui::Frame {
            inner_margin: self.padding,
            outer_margin: self.outer_margin,
            fill: self.computed_fill(tokens),
            stroke: self.stroke.unwrap_or(Stroke::new(1.0, tokens.border)),
            corner_radius: self.corner_radius,
            shadow: egui::Shadow::NONE,
        }
        .show(ui, add)
    }
}

/// A section title with an optional de-emphasized caption.
pub struct SectionHeader<'a> {
    title: &'a str,
    caption: Option<&'a str>,
}

impl<'a> SectionHeader<'a> {
    /// Create a header with `title` and no caption.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            caption: None,
        }
    }

    /// Add a caption rendered in the theme's muted text color.
    pub fn caption(mut self, caption: &'a str) -> Self {
        self.caption = Some(caption);
        self
    }

    /// Show the header. Both texts are ordinary egui labels and are accessible by default.
    pub fn show(self, ui: &mut Ui) -> InnerResponse<()> {
        let tokens = theme::active_tokens(ui.ctx());
        ui.vertical(|ui| {
            ui.label(RichText::new(self.title).strong().color(tokens.text));
            if let Some(caption) = self.caption {
                ui.label(RichText::new(caption).small().color(tokens.muted_text));
            }
        })
    }
}

/// Semantic tone of a [`Badge`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeTone {
    /// Muted, informational state using `elevated_surface`.
    Neutral,
    /// Emphasis using `primary`.
    Primary,
    /// Successful outcome using `success`.
    Success,
    /// Caution using `warning`.
    Warning,
    /// Failure using `error`.
    Error,
}

impl BadgeTone {
    /// Token color associated with this tone.
    pub fn color(self, tokens: ThemeTokens) -> Color32 {
        match self {
            Self::Neutral => tokens.elevated_surface,
            Self::Primary => tokens.primary,
            Self::Success => tokens.success,
            Self::Warning => tokens.warning,
            Self::Error => tokens.error,
        }
    }
}

/// A compact themed status badge with a semantic tone.
///
/// ```
/// use faseela_ui::widgets::{Badge, BadgeTone};
///
/// egui::__run_test_ui(|ui| {
///     ui.add(Badge::new("Build passed", BadgeTone::Success));
/// });
/// ```
pub struct Badge<'a> {
    text: &'a str,
    tone: BadgeTone,
}

impl<'a> Badge<'a> {
    /// Create a badge with `text` and a semantic `tone`.
    pub fn new(text: &'a str, tone: BadgeTone) -> Self {
        Self { text, tone }
    }

    /// Show the badge. The text is registered as an accessible label.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for Badge<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let font_id = TextStyle::Body.resolve(ui.style());
        let text_width = ui
            .fonts_mut(|fonts| fonts.layout_no_wrap(self.text.to_owned(), font_id, tokens.text))
            .rect
            .width();
        let padding = ui.spacing().button_padding;
        let size = Vec2::new(
            text_width + padding.x * 2.0,
            ui.spacing().interact_size.y * 0.7,
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let fill = self.tone.color(tokens);
        ui.painter().rect_filled(rect, CornerRadius::same(6), fill);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            self.text,
            FontId::proportional(11.0),
            ThemeTokens::on_color(fill),
        );
        response.widget_info(|| WidgetInfo::labeled(egui::Role::Label, true, self.text));
        response
    }
}

/// A centered message for an empty list or view, with an optional action.
///
/// ```
/// use faseela_ui::widgets::EmptyState;
///
/// egui::__run_test_ui(|ui| {
///     let response = EmptyState::new("No items").action("Add").show(ui);
///     let _clicked = response.inner.action_clicked;
/// });
/// ```
pub struct EmptyState<'a> {
    title: &'a str,
    description: Option<&'a str>,
    action: Option<&'a str>,
    action_enabled: bool,
}

/// Result of showing an [`EmptyState`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EmptyStateResponse {
    /// Whether the optional action button was activated this frame.
    pub action_clicked: bool,
    /// Whether the empty state's action was enabled.
    pub action_enabled: bool,
}

impl<'a> EmptyState<'a> {
    /// Create an empty state with `title` and no description or action.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: None,
            action: None,
            action_enabled: true,
        }
    }

    /// Add a short explanation below the title.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Add an action proposal button below the message.
    pub fn action(mut self, action: &'a str) -> Self {
        self.action = Some(action);
        self
    }

    /// Disable the action button while keeping it visible and accessible.
    pub fn action_enabled(mut self, enabled: bool) -> Self {
        self.action_enabled = enabled;
        self
    }

    /// Show the empty state, filling the available width.
    pub fn show(self, ui: &mut Ui) -> InnerResponse<EmptyStateResponse> {
        let tokens = theme::active_tokens(ui.ctx());
        ui.vertical_centered(|ui| {
            ui.set_width(ui.available_width());
            ui.add_space(12.0);
            ui.label(RichText::new(self.title).strong().color(tokens.text));
            if let Some(description) = self.description {
                ui.label(RichText::new(description).small().color(tokens.muted_text));
            }
            let mut response = EmptyStateResponse::default();
            if let Some(action) = self.action {
                let primary = tokens.primary;
                let button =
                    Button::new(RichText::new(action).color(ThemeTokens::on_color(primary)))
                        .fill(primary)
                        .stroke(Stroke::NONE);
                response.action_enabled = self.action_enabled;
                response.action_clicked = ui.add_enabled(self.action_enabled, button).clicked();
            }
            ui.add_space(12.0);
            response
        })
    }
}

/// One command shown in a [`toolbar`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolbarItem<'a> {
    /// Accessible and visible label. Use stable labels; identity comes from the item's index.
    pub label: &'a str,
    /// Whether the command can be activated.
    pub enabled: bool,
}

impl<'a> ToolbarItem<'a> {
    /// Create an enabled toolbar command.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            enabled: true,
        }
    }

    /// Mark the command as disabled.
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Compute the rectangles for a fitted toolbar, from the reading start to the reading end.
fn layout_items(
    rect: egui::Rect,
    fit: &ToolbarFit,
    spacing: f32,
    rtl: bool,
) -> (Vec<(usize, egui::Rect)>, Option<egui::Rect>) {
    let height = rect.height();
    let mut cursor = if rtl { rect.right() } else { rect.left() };
    let mut placed = Vec::with_capacity(fit.shown.len());

    for &(index, width) in &fit.shown {
        if rtl {
            cursor -= width;
            placed.push((
                index,
                egui::Rect::from_min_size(egui::pos2(cursor, rect.top()), Vec2::new(width, height)),
            ));
            cursor -= spacing;
        } else {
            placed.push((
                index,
                egui::Rect::from_min_size(egui::pos2(cursor, rect.top()), Vec2::new(width, height)),
            ));
            cursor += width + spacing;
        }
    }

    let overflow = if fit.overflow.is_empty() {
        None
    } else if rtl {
        cursor -= fit.overflow_width;
        Some(egui::Rect::from_min_size(
            egui::pos2(cursor, rect.top()),
            Vec2::new(fit.overflow_width, height),
        ))
    } else {
        Some(egui::Rect::from_min_size(
            egui::pos2(cursor, rect.top()),
            Vec2::new(fit.overflow_width, height),
        ))
    };

    (placed, overflow)
}

fn measure_text(ui: &Ui, text: &str, style: TextStyle) -> f32 {
    let font_id = style.resolve(ui.style());
    ui.fonts_mut(|fonts| fonts.layout_no_wrap(text.to_owned(), font_id, Color32::WHITE))
        .rect
        .width()
}

/// Show a responsive command bar built on [`fit_toolbar`].
///
/// Items keep their natural widths and order. When they no longer fit, the trailing commands
/// move into an overflow menu that remains keyboard reachable. Layout follows the reading
/// direction stored on the egui context. The returned index is a proposal: the application owns
/// command dispatch. `None` is returned when the available width cannot fit even the overflow
/// control; callers should then provide an explicit fallback such as a wrapping row.
///
/// ```
/// use faseela_ui::widgets::{ToolbarItem, toolbar};
///
/// egui::__run_test_ui(|ui| {
///     let items = [ToolbarItem::new("Open"), ToolbarItem::new("Save").disabled()];
///     let clicked = toolbar(ui, egui::Id::unique("demo-toolbar"), &items);
///     // The index is a proposal; the application dispatches the matching command.
///     let _ = clicked;
/// });
/// ```
pub fn toolbar(ui: &mut Ui, id: Id, items: &[ToolbarItem<'_>]) -> Option<usize> {
    if items.is_empty() {
        return None;
    }

    let tokens = theme::active_tokens(ui.ctx());
    let spacing = ui.spacing().item_spacing.x.max(0.0);
    let padding = ui.spacing().button_padding;
    let widths: Vec<f32> = items
        .iter()
        .map(|item| measure_text(ui, item.label, TextStyle::Button) + padding.x * 2.0 + 2.0)
        .collect();
    let overflow_width =
        (measure_text(ui, "+99", TextStyle::Button) + padding.x * 2.0 + 2.0).max(36.0);
    let available = ui.available_width().max(0.0);
    let height = ui.spacing().interact_size.y;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(available, height), Sense::hover());
    let fit = fit_toolbar(&widths, available, spacing, overflow_width)?;

    let rtl = ui.direction() == crate::direction::Direction::Rtl;
    let (placed, overflow) = layout_items(rect, &fit, spacing, rtl);
    let mut clicked = None;

    for (index, item_rect) in placed {
        let item = items[index];
        let button = Button::new(RichText::new(item.label).color(tokens.text))
            .fill(tokens.surface)
            .stroke(Stroke::new(1.0, tokens.border))
            .min_size(item_rect.size());
        if ui.put(item_rect, button).clicked() {
            clicked = Some(index);
        }
    }

    if let Some(menu_rect) = overflow {
        let count = fit.overflow.len();
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(menu_rect)
                .layout(Layout::centered_and_justified(egui::Direction::TopDown)),
            |ui| {
                let button = Button::new(
                    RichText::new(format!("+{count}")).color(ThemeTokens::on_color(tokens.primary)),
                )
                .fill(tokens.primary)
                .stroke(Stroke::NONE)
                .min_size(menu_rect.size());
                let menu = egui::containers::menu::MenuButton::from_button(button);
                ui.push_id(id.with("overflow"), |ui| {
                    menu.ui(ui, |ui| {
                        for &index in &fit.overflow {
                            let item = items[index];
                            if ui
                                .add_enabled(item.enabled, Button::new(item.label))
                                .clicked()
                            {
                                clicked = Some(index);
                                ui.close();
                            }
                        }
                    });
                });
            },
        );
    }

    clicked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemePreset;
    use egui::{Context, FullOutput, Pos2, RawInput, Shape, pos2, vec2};

    fn render(preset: ThemePreset, rtl: bool, mut add: impl FnMut(&mut Ui)) -> FullOutput {
        let ctx = Context::default();
        theme::apply_preset(&ctx, preset);
        crate::direction::apply(
            &ctx,
            if rtl {
                crate::direction::Direction::Rtl
            } else {
                crate::direction::Direction::Ltr
            },
        );
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, vec2(500.0, 300.0))),
                ..Default::default()
            },
            |ui| add(ui),
        );
        output.textures_delta.clear();
        output
    }

    #[test]
    fn card_and_badge_colors_follow_active_tokens() {
        for preset in ThemePreset::ALL {
            let tokens = preset.tokens();
            assert_eq!(Card::new().computed_fill(tokens), tokens.surface);
            assert_eq!(
                Card::new().elevated(true).computed_fill(tokens),
                tokens.elevated_surface
            );
            assert_eq!(
                Card::new().fill(Color32::RED).computed_fill(tokens),
                Color32::RED
            );
            assert_eq!(BadgeTone::Primary.color(tokens), tokens.primary);
            assert_eq!(BadgeTone::Success.color(tokens), tokens.success);
            assert_eq!(BadgeTone::Warning.color(tokens), tokens.warning);
            assert_eq!(BadgeTone::Error.color(tokens), tokens.error);
            assert_eq!(BadgeTone::Neutral.color(tokens), tokens.elevated_surface);
        }
    }

    #[test]
    fn all_widgets_render_for_every_preset_and_direction() {
        for preset in ThemePreset::ALL {
            for rtl in [false, true] {
                let output = render(preset, rtl, |ui| {
                    Card::new().show(ui, |ui| {
                        SectionHeader::new("Title").caption("Caption").show(ui);
                        ui.horizontal(|ui| {
                            ui.add(Badge::new("OK", BadgeTone::Success));
                            ui.add(Badge::new("Late", BadgeTone::Warning));
                            ui.add(Badge::new("Failed", BadgeTone::Error));
                        });
                        EmptyState::new("Nothing here")
                            .description("Try again")
                            .action("Refresh")
                            .show(ui);
                        toolbar(
                            ui,
                            Id::unique("widgets-toolbar"),
                            &[
                                ToolbarItem::new("File"),
                                ToolbarItem::new("Edit"),
                                ToolbarItem::new("Run").disabled(),
                            ],
                        );
                    });
                });
                let has_rect = output
                    .shapes
                    .iter()
                    .any(|clip| matches!(clip.shape, Shape::Rect(_)));
                assert!(has_rect, "{preset:?} rtl={rtl} must paint rect shapes");
                for clip in &output.shapes {
                    if let Shape::Rect(rect) = &clip.shape {
                        assert!(rect.fill.a() > 0 || rect.stroke.width > 0.0);
                    }
                }
            }
        }
    }

    #[test]
    fn toolbar_layout_places_items_in_reading_order() {
        let rect = egui::Rect::from_min_size(pos2(10.0, 20.0), vec2(200.0, 30.0));
        let fit = ToolbarFit {
            shown: vec![(0, 40.0), (1, 50.0)],
            overflow: vec![2],
            overflow_width: 30.0,
        };
        let (ltr_items, ltr_overflow) = layout_items(rect, &fit, 6.0, false);
        assert_eq!(ltr_items[0].1.left(), 10.0);
        assert_eq!(ltr_items[1].1.left(), 56.0);
        assert_eq!(ltr_overflow.expect("overflow").left(), 112.0);

        let (rtl_items, rtl_overflow) = layout_items(rect, &fit, 6.0, true);
        assert_eq!(rtl_items[0].1.right(), 210.0);
        assert_eq!(rtl_items[1].1.right(), 164.0);
        assert_eq!(rtl_overflow.expect("overflow").right(), 108.0);
    }

    #[test]
    fn empty_state_action_can_be_disabled() {
        let output = render(ThemePreset::Light, false, |ui| {
            let enabled = EmptyState::new("One").action("Go").show(ui).inner;
            assert!(enabled.action_enabled);
            let disabled = EmptyState::new("Two")
                .action("Go")
                .action_enabled(false)
                .show(ui)
                .inner;
            assert!(!disabled.action_enabled);
            assert!(!disabled.action_clicked);
        });
        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn empty_toolbar_and_impossible_width_are_safe() {
        let output = render(ThemePreset::FaseelaDark, false, |ui| {
            assert_eq!(toolbar(ui, Id::unique("empty"), &[]), None);
            assert_eq!(
                toolbar(ui, Id::unique("too-narrow"), &[ToolbarItem::new("File")]),
                None
            );
        });
        assert!(!output.shapes.is_empty());
    }
}
