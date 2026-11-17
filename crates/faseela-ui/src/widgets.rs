//! Reusable, theme-aware presentation widgets.
//!
//! Every widget in this module reads its colors from [`crate::theme::active_tokens`], so a theme
//! switch takes effect on the next frame without callers rebuilding their styles. Widgets return
//! proposals (for example which toolbar item was clicked); applications keep ownership of state,
//! commands, and selection.

use crate::{
    buttons::{Button as ThemedButton, ButtonVariant, ControlSize},
    direction,
    layout::{ToolbarFit, fit_toolbar},
    theme::{self, ThemeTokens},
};
use egui::{
    Color32, CornerRadius, FontId, Id, InnerResponse, Layout, Margin, Response, RichText, Sense,
    Stroke, Ui, Vec2, Widget, WidgetInfo, vec2,
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
    /// Fill color associated with this tone.
    pub fn color(self, tokens: ThemeTokens) -> Color32 {
        match self {
            Self::Neutral => tokens.elevated_surface,
            Self::Primary => tokens.primary,
            Self::Success => tokens.success,
            Self::Warning => tokens.warning,
            Self::Error => tokens.error,
        }
    }

    /// Foreground color associated with this tone.
    ///
    /// Neutral badges use ordinary body text on [`ThemeTokens::elevated_surface`]. The remaining
    /// tones use their matching explicit `on_*` token.
    pub fn foreground(self, tokens: ThemeTokens) -> Color32 {
        tokens.on_badge(self)
    }

    /// Color of a status dot painted directly on an ordinary background.
    pub fn dot_color(self, tokens: ThemeTokens) -> Color32 {
        match self {
            Self::Neutral => tokens.muted_text,
            _ => self.color(tokens),
        }
    }
}

const BADGE_FONT_SIZE: f32 = 11.0;
const BADGE_PADDING: Vec2 = vec2(8.0, 3.0);

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
        let font_id = FontId::proportional(BADGE_FONT_SIZE);
        let galley =
            ui.fonts_mut(|fonts| fonts.layout_no_wrap(self.text.to_owned(), font_id, tokens.text));
        let size = vec2(
            galley.rect.width() + BADGE_PADDING.x * 2.0,
            galley.rect.height() + BADGE_PADDING.y * 2.0,
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let fill = self.tone.color(tokens);
        let foreground = self.tone.foreground(tokens);
        ui.painter().rect_filled(rect, CornerRadius::same(6), fill);
        ui.painter()
            .galley(rect.center() - galley.rect.size() * 0.5, galley, foreground);
        response.widget_info(|| WidgetInfo::labeled(egui::Role::Label, true, self.text));
        response
    }
}

/// A determinate progress bar with an optional visible and accessible label.
///
/// ```
/// use faseela_ui::widgets::ProgressBar;
///
/// egui::__run_test_ui(|ui| {
///     ProgressBar::new(0.65).label("Uploading").show_value(true).show(ui);
/// });
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressBar<'a> {
    value: f32,
    label: Option<&'a str>,
    show_value: bool,
    width: Option<f32>,
}

impl<'a> ProgressBar<'a> {
    /// Create a bar for `value` in `0.0..=1.0`. Non-finite values become zero.
    pub fn new(value: f32) -> Self {
        Self {
            value: if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            },
            label: None,
            show_value: false,
            width: None,
        }
    }

    /// Visible and accessible label.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Show the rounded percentage beside the bar.
    pub fn show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }

    /// Request a width in logical points, clamped to the available width.
    pub fn width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.width = Some(width.min(4096.0));
        }
        self
    }

    /// Show the bar and return its response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let percent = (self.value * 100.0).round() as u32;
        let label = self
            .label
            .map(|label| format!("{label}: {percent}%"))
            .unwrap_or_else(|| format!("{percent}%"));
        ui.vertical(|ui| {
            if self.label.is_some() || self.show_value {
                direction::horizontal(ui, |ui| {
                    if let Some(label) = self.label {
                        ui.label(RichText::new(label).size(12.0).color(tokens.text));
                    }
                    if self.show_value {
                        ui.with_layout(direction::trailing(ui), |ui| {
                            ui.label(
                                RichText::new(format!("{percent}%"))
                                    .size(12.0)
                                    .color(tokens.muted_text),
                            );
                        });
                    }
                });
            }
            let available = ui.available_width().max(1.0);
            let width = self.width.unwrap_or(available).min(available).max(1.0);
            let (rect, response) = ui.allocate_exact_size(vec2(width, 8.0), Sense::hover());
            let track = tokens.elevated_surface;
            let radius = CornerRadius::same(4);
            ui.painter().rect_filled(rect, radius, track);
            ui.painter().rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, tokens.control_border),
                egui::StrokeKind::Inside,
            );
            let fill_width = (rect.width() * self.value).max(0.0);
            if fill_width > 0.0 {
                let fill = egui::Rect::from_min_size(rect.min, vec2(fill_width, rect.height()))
                    .intersect(rect);
                ui.painter().rect_filled(fill, radius, tokens.primary);
            }
            response
                .widget_info(|| WidgetInfo::labeled(egui::Role::ProgressIndicator, true, &label));
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_numeric_value(f64::from(percent));
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value(100.0);
            });
            response
        })
        .inner
    }
}

/// An indeterminate loading indicator (spinner) with a visible accessible label.
///
/// ```
/// use faseela_ui::widgets::LoadingIndicator;
///
/// egui::__run_test_ui(|ui| {
///     LoadingIndicator::new().label("Loading projects").show(ui);
/// });
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoadingIndicator<'a> {
    label: Option<&'a str>,
    size: f32,
}

impl<'a> Default for LoadingIndicator<'a> {
    fn default() -> Self {
        Self {
            label: None,
            size: 18.0,
        }
    }
}

impl<'a> LoadingIndicator<'a> {
    /// Create a loading indicator without a visible label.
    pub fn new() -> Self {
        Self::default()
    }

    /// Visible and accessible label shown to the reading end of the spinner.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Diameter in logical points, clamped to `8.0..=96.0`.
    pub fn size(mut self, size: f32) -> Self {
        if size.is_finite() {
            self.size = size.clamp(8.0, 96.0);
        }
        self
    }

    /// Show the indicator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let accessible = self.label.unwrap_or("Loading");
        let height = self.size.max(ui.spacing().interact_size.y);
        let width = ui.available_width().max(0.0);
        ui.allocate_ui_with_layout(vec2(width, height), direction::row(ui), |ui| {
            let response = ui.add(egui::Spinner::new().size(self.size).color(tokens.primary));
            response.widget_info(|| {
                WidgetInfo::labeled(egui::Role::ProgressIndicator, true, accessible)
            });
            if let Some(label) = self.label {
                ui.label(RichText::new(label).size(12.0).color(tokens.text));
            }
            response
        })
        .inner
    }
}

/// A compact status row: a semantic dot followed by readable text.
///
/// ```
/// use faseela_ui::widgets::{BadgeTone, StatusIndicator};
///
/// egui::__run_test_ui(|ui| {
///     StatusIndicator::new("Connected", BadgeTone::Success).show(ui);
/// });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatusIndicator<'a> {
    label: &'a str,
    tone: BadgeTone,
    detail: Option<&'a str>,
}

impl<'a> StatusIndicator<'a> {
    /// Create a status with `label` and semantic `tone`.
    pub fn new(label: &'a str, tone: BadgeTone) -> Self {
        Self {
            label,
            tone,
            detail: None,
        }
    }

    /// De-emphasized supporting text after the status label.
    pub fn detail(mut self, detail: &'a str) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Show the status indicator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let dot_color = self.tone.dot_color(tokens);
        let response = direction::horizontal(ui, |ui| {
            let (dot_rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
            ui.painter()
                .circle_filled(dot_rect.center(), 5.0, dot_color);
            let response = ui.label(RichText::new(self.label).size(12.5).color(tokens.text));
            if let Some(detail) = self.detail {
                ui.label(RichText::new(detail).size(11.5).color(tokens.muted_text));
            }
            response
        })
        .inner;
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::Role::Status);
            node.set_label(self.label);
        });
        response
    }
}

/// A themed horizontal or vertical rule.
///
/// ```
/// use faseela_ui::widgets::Separator;
///
/// egui::__run_test_ui(|ui| {
///     Separator::horizontal().show(ui);
///     ui.horizontal(|ui| {
///         ui.label("Left");
///         Separator::vertical().length(40.0).show(ui);
///         ui.label("Right");
///     });
/// });
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Separator {
    orientation: SeparatorOrientation,
    length: Option<f32>,
    inset: f32,
}

/// Direction of a [`Separator`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl Default for Separator {
    fn default() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            length: None,
            inset: 0.0,
        }
    }
}

impl Separator {
    /// A full-width horizontal rule.
    pub fn horizontal() -> Self {
        Self::default()
    }

    /// A vertical rule. Set a length explicitly for a useful height.
    pub fn vertical() -> Self {
        Self {
            orientation: SeparatorOrientation::Vertical,
            ..Self::default()
        }
    }

    /// Explicit length in logical points. Defaults to the available width or a 20-point height.
    pub fn length(mut self, length: f32) -> Self {
        if length.is_finite() && length > 0.0 {
            self.length = Some(length.min(1_000_000.0));
        }
        self
    }

    /// Inset each painted end by this many logical points.
    pub fn inset(mut self, inset: f32) -> Self {
        if inset.is_finite() && inset > 0.0 {
            self.inset = inset.min(1_000_000.0);
        }
        self
    }

    /// Show the separator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let tokens = theme::active_tokens(ui.ctx());
        let size = match self.orientation {
            SeparatorOrientation::Horizontal => {
                let width = self.length.unwrap_or_else(|| ui.available_width().max(0.0));
                vec2(width.max(0.0), 1.0)
            }
            SeparatorOrientation::Vertical => vec2(1.0, self.length.unwrap_or(20.0)),
        };
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let line_rect = match self.orientation {
            SeparatorOrientation::Horizontal => egui::Rect::from_min_max(
                egui::pos2(rect.left() + self.inset, rect.center().y),
                egui::pos2(rect.right() - self.inset, rect.center().y),
            ),
            SeparatorOrientation::Vertical => egui::Rect::from_min_max(
                egui::pos2(rect.center().x, rect.top() + self.inset),
                egui::pos2(rect.center().x, rect.bottom() - self.inset),
            ),
        };
        if line_rect.width().is_finite() && line_rect.height().is_finite() {
            ui.painter().line_segment(
                [line_rect.left_top(), line_rect.right_bottom()],
                Stroke::new(1.0, tokens.border),
            );
        }
        response
    }
}

/// A small themed tooltip attached to an existing response.
///
/// ```
/// use faseela_ui::widgets::Tooltip;
///
/// egui::__run_test_ui(|ui| {
///     let response = ui.button("Save");
///     let response = Tooltip::new("Save the current document").show(response);
///     let _ = response;
/// });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tooltip<'a> {
    text: &'a str,
}

impl<'a> Tooltip<'a> {
    /// Create a tooltip with `text`.
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    /// Attach the tooltip to `response` and return it unchanged otherwise.
    pub fn show(self, response: Response) -> Response {
        let text = self.text;
        response.on_hover_ui(|ui| {
            ui.label(RichText::new(text).size(12.0));
        })
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
                let button = ThemedButton::new(action)
                    .variant(ButtonVariant::Primary)
                    .enabled(self.action_enabled);
                response.action_enabled = self.action_enabled;
                response.action_clicked = button.show(ui).clicked();
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

fn measure_text(ui: &Ui, text: &str, font_id: FontId) -> f32 {
    ui.fonts_mut(|fonts| fonts.layout_no_wrap(text.to_owned(), font_id, Color32::WHITE))
        .rect
        .width()
}

/// How a toolbar fitted into its available width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarState {
    /// There were no items to show.
    #[default]
    Empty,
    /// Every item is visible.
    Fits,
    /// Some items moved into the overflow menu.
    Overflow,
    /// Even the overflow control does not fit. Nothing was painted.
    NoSpace,
}

/// The full result of [`toolbar_response`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolbarResponse {
    /// Layout state for this frame.
    pub state: ToolbarState,
    /// The command activated this frame, if any.
    pub activated: Option<usize>,
    /// Visible item indices, in input order.
    pub shown: Vec<usize>,
    /// Item indices reachable through the overflow menu, in input order.
    pub overflow: Vec<usize>,
    /// Stable ID of the overflow menu button when one is shown.
    pub overflow_button_id: Option<Id>,
}

impl ToolbarResponse {
    /// Whether the command at `index` is currently reachable (visible or in the overflow menu).
    pub fn contains(&self, index: usize) -> bool {
        self.shown.contains(&index) || self.overflow.contains(&index)
    }
}

fn overflow_control_width(ui: &Ui, count: usize) -> f32 {
    let size = ControlSize::Small;
    let label = format!("+{count}");
    let text = measure_text(ui, &label, FontId::proportional(size.font_size()));
    (text + size.padding().x * 2.0 + 2.0).max(34.0)
}

/// Show a responsive command bar built on [`fit_toolbar`].
///
/// Items keep their natural widths and order. When they no longer fit, the trailing commands
/// move into an overflow menu that remains keyboard reachable. Layout follows the reading
/// direction stored on the egui context. Only `activated` is returned; use
/// [`toolbar_response`] when the application needs the layout state or the overflow contents.
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
    toolbar_response(ui, id, items).activated
}

/// Show a responsive command bar and report its layout state.
///
/// Disabled items stay visible and focusable in the accessibility tree but cannot be activated,
/// whether they are painted in the bar or listed in the overflow menu. Every item has a stable ID:
/// visible item `i` is `id.with(("toolbar-item", i))`, menu item `i` is
/// `id.with(("overflow-item", i))`, and [`ToolbarResponse::overflow_button_id`] identifies the
/// menu trigger.
///
/// ```
/// use faseela_ui::widgets::{ToolbarItem, ToolbarState, toolbar_response};
///
/// egui::__run_test_ui(|ui| {
///     ui.set_max_width(220.0);
///     let items: Vec<_> = (0..20)
///         .map(|index| ToolbarItem::new(if index % 2 == 0 { "Open" } else { "Save" }))
///         .collect();
///     let result = toolbar_response(ui, egui::Id::unique("large-toolbar"), &items);
///     assert_eq!(result.state, ToolbarState::Overflow);
///     assert_eq!(result.overflow.len() + result.shown.len(), items.len());
/// });
/// ```
pub fn toolbar_response(ui: &mut Ui, id: Id, items: &[ToolbarItem<'_>]) -> ToolbarResponse {
    if items.is_empty() {
        return ToolbarResponse::default();
    }

    let size = ControlSize::Small;
    let font = FontId::proportional(size.font_size());
    let padding = size.padding();
    let spacing = ui.spacing().item_spacing.x.max(0.0);
    let widths: Vec<f32> = items
        .iter()
        .map(|item| measure_text(ui, item.label, font.clone()) + padding.x * 2.0 + 2.0)
        .collect();
    let available = ui.available_width().max(0.0);
    let height = size.height();

    let total = widths.iter().map(|&width| f64::from(width)).sum::<f64>()
        + f64::from(spacing) * widths.len().saturating_sub(1) as f64;
    let fit = if total <= f64::from(available) {
        ToolbarFit {
            shown: widths.iter().copied().enumerate().collect(),
            ..Default::default()
        }
    } else {
        // The overflow label depends on how many items actually overflow, which in turn depends
        // on the label width. Iterate from the largest possible count; the sequence is
        // non-increasing and converges in a handful of steps for real toolbars.
        let mut count = items.len();
        let mut fitted = None;
        for _ in 0..8 {
            let width = overflow_control_width(ui, count);
            let Some(candidate) = fit_toolbar(&widths, available, spacing, width) else {
                return ToolbarResponse {
                    state: ToolbarState::NoSpace,
                    ..Default::default()
                };
            };
            let actual = candidate.overflow.len();
            if actual == count {
                fitted = Some(candidate);
                break;
            }
            count = actual.max(1);
        }
        match fitted {
            Some(fitted) => fitted,
            None => {
                // Extremely unusual font metrics: reserve for the last count and still report
                // the true contents. The label is no wider than the reserved slot.
                let width = overflow_control_width(ui, count);
                let Some(candidate) = fit_toolbar(&widths, available, spacing, width) else {
                    return ToolbarResponse {
                        state: ToolbarState::NoSpace,
                        ..Default::default()
                    };
                };
                candidate
            }
        }
    };

    let (rect, _) = ui.allocate_exact_size(vec2(available, height), Sense::hover());
    let rtl = direction::is_rtl(ui);
    let (placed, overflow) = layout_items(rect, &fit, spacing, rtl);
    let tokens = theme::active_tokens(ui.ctx());
    let mut activated = None;

    for (index, item_rect) in placed {
        let item = items[index];
        let button = ThemedButton::new(item.label)
            .variant(ButtonVariant::Ghost)
            .size(size)
            .enabled(item.enabled)
            .id(id.with(("toolbar-item", index)));
        if ui.put(item_rect, button).clicked() {
            activated = Some(index);
        }
    }

    let mut overflow_button_id = None;
    if let Some(menu_rect) = overflow {
        let count = fit.overflow.len();
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(menu_rect)
                .layout(Layout::centered_and_justified(egui::Direction::TopDown)),
            |ui| {
                let label = format!("+{count}");
                let button = egui::Button::new(
                    RichText::new(&label)
                        .color(tokens.on_primary)
                        .size(font.size),
                )
                .fill(tokens.primary)
                .stroke(Stroke::NONE)
                .min_size(menu_rect.size());
                let menu = egui::containers::menu::MenuButton::from_button(button);
                let (response, _) = menu.ui(ui, |ui| {
                    for &index in &fit.overflow {
                        let item = items[index];
                        let entry = ThemedButton::new(item.label)
                            .variant(ButtonVariant::Ghost)
                            .size(size)
                            .enabled(item.enabled)
                            .full_width()
                            .id(id.with(("overflow-item", index)));
                        if entry.show(ui).clicked() {
                            activated = Some(index);
                            ui.close();
                        }
                    }
                });
                response.widget_info(|| {
                    WidgetInfo::labeled(egui::Role::Button, true, format!("More commands: {label}"))
                });
                overflow_button_id = Some(response.id);
            },
        );
    }

    ToolbarResponse {
        state: if fit.overflow.is_empty() {
            ToolbarState::Fits
        } else {
            ToolbarState::Overflow
        },
        activated,
        shown: fit.shown.iter().map(|&(index, _)| index).collect(),
        overflow: fit.overflow,
        overflow_button_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemePreset, contrast_ratio};
    use egui::{
        Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, pos2,
        vec2,
    };

    fn setup(preset: ThemePreset, rtl: bool, _size: Vec2) -> Context {
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
        ctx
    }

    fn frame(
        ctx: &Context,
        size: Vec2,
        events: Vec<Event>,
        mut add: impl FnMut(&mut Ui),
    ) -> FullOutput {
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                focused: true,
                events,
                ..Default::default()
            },
            |ui| add(ui),
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

    fn render(preset: ThemePreset, rtl: bool, add: impl FnMut(&mut Ui)) -> FullOutput {
        let ctx = setup(preset, rtl, vec2(500.0, 300.0));
        frame(&ctx, vec2(500.0, 300.0), vec![], add)
    }

    fn toolbar_in_width(
        ctx: &Context,
        size: Vec2,
        width: f32,
        id: Id,
        items: &[ToolbarItem<'_>],
    ) -> (ToolbarResponse, FullOutput) {
        let mut result = None;
        let output = frame(ctx, size, vec![], |ui| {
            ui.allocate_ui_with_layout(
                vec2(width, 40.0),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    result = Some(toolbar_response(ui, id, items));
                },
            );
        });
        (result.expect("toolbar result"), output)
    }

    fn text_labels(output: &FullOutput) -> Vec<String> {
        output
            .shapes
            .iter()
            .filter_map(|clip| match &clip.shape {
                Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn labelled_progress_bar_stays_compact_in_a_tall_ui() {
        let ctx = setup(ThemePreset::Light, false, vec2(320.0, 400.0));
        let mut rect = Rect::NOTHING;
        let _ = frame(&ctx, vec2(320.0, 400.0), vec![], |ui| {
            rect = ProgressBar::new(0.6)
                .label("Upload")
                .show_value(true)
                .show(ui)
                .rect;
        });
        assert!(rect.height() < 80.0, "progress bar used {rect:?}");
        assert!(rect.width() <= 320.0, "progress bar used {rect:?}");
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
                        ProgressBar::new(0.42)
                            .label("Uploading")
                            .show_value(true)
                            .show(ui);
                        LoadingIndicator::new().label("Loading").show(ui);
                        StatusIndicator::new("Connected", BadgeTone::Success)
                            .detail("3 ms")
                            .show(ui);
                        Separator::horizontal().show(ui);
                        ui.horizontal(|ui| {
                            ui.label("Leading");
                            Separator::vertical().length(20.0).show(ui);
                            ui.label("Trailing");
                        });
                        let badge = ui.add(Badge::new("Hover me", BadgeTone::Primary));
                        let _ = Tooltip::new("Themed tooltip").show(badge);
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
        let ctx = setup(ThemePreset::FaseelaDark, false, vec2(500.0, 300.0));
        let (empty, _) =
            toolbar_in_width(&ctx, vec2(500.0, 300.0), 400.0, Id::unique("empty"), &[]);
        assert_eq!(empty.state, ToolbarState::Empty);
        let (narrow, _) = toolbar_in_width(
            &ctx,
            vec2(500.0, 300.0),
            5.0,
            Id::unique("too-narrow"),
            &[ToolbarItem::new("File")],
        );
        assert_eq!(narrow.state, ToolbarState::NoSpace);
        assert!(narrow.overflow_button_id.is_none());
    }

    #[test]
    fn badge_measures_and_paints_with_the_same_font() {
        let ctx = setup(ThemePreset::FaseelaDark, false, vec2(300.0, 200.0));
        let mut badge_rect = Rect::NOTHING;
        let output = frame(&ctx, vec2(300.0, 200.0), vec![], |ui| {
            badge_rect = ui.add(Badge::new("Ready", BadgeTone::Success)).rect;
        });
        let text = output
            .shapes
            .iter()
            .find_map(|clip| match &clip.shape {
                Shape::Text(text) if text.galley.text() == "Ready" => Some(text),
                _ => None,
            })
            .expect("badge text must be painted");
        assert!(
            (badge_rect.width() - (text.galley.rect.width() + BADGE_PADDING.x * 2.0)).abs() < 0.01,
            "badge width {} must match measured text width {}",
            badge_rect.width(),
            text.galley.rect.width()
        );
        assert!(
            (badge_rect.height() - (text.galley.rect.height() + BADGE_PADDING.y * 2.0)).abs()
                < 0.01
        );
    }

    #[test]
    fn every_badge_tone_meets_text_contrast_in_every_preset() {
        for preset in ThemePreset::ALL {
            let tokens = preset.tokens();
            for tone in [
                BadgeTone::Neutral,
                BadgeTone::Primary,
                BadgeTone::Success,
                BadgeTone::Warning,
                BadgeTone::Error,
            ] {
                let ratio = contrast_ratio(tone.foreground(tokens), tone.color(tokens));
                assert!(
                    ratio >= 4.5,
                    "{preset:?} {tone:?} badge contrast is {ratio:.2}:1"
                );
            }
        }
    }

    #[test]
    fn progress_bar_clamps_and_reports_layout() {
        assert_eq!(ProgressBar::new(-1.0).value, 0.0);
        assert_eq!(ProgressBar::new(2.0).value, 1.0);
        assert_eq!(ProgressBar::new(f32::NAN).value, 0.0);
        assert_eq!(ProgressBar::new(f32::INFINITY).value, 0.0);

        let output = render(ThemePreset::Light, false, |ui| {
            ProgressBar::new(0.63)
                .label("Upload")
                .show_value(true)
                .show(ui);
        });
        assert!(text_labels(&output).iter().any(|text| text.contains("63%")));
    }

    #[test]
    fn toolbar_reports_counts_and_fits_within_available_width() {
        let ctx = setup(ThemePreset::FaseelaDark, false, vec2(600.0, 300.0));
        let labels = [
            "New", "Open", "Save", "Run", "Build", "Test", "Debug", "Deploy", "Inspect", "Profile",
            "Archive", "Export", "Import", "Share", "Publish",
        ];
        let items: Vec<_> = labels.iter().map(|label| ToolbarItem::new(label)).collect();
        let id = Id::unique("widgets-overflow-toolbar");
        let (result, _) = toolbar_in_width(&ctx, vec2(600.0, 300.0), 240.0, id, &items);
        assert_eq!(result.state, ToolbarState::Overflow);
        assert!(!result.shown.is_empty());
        assert!(!result.overflow.is_empty());
        assert_eq!(result.shown.len() + result.overflow.len(), items.len());
        for index in 0..items.len() {
            assert!(result.contains(index));
        }

        let mut all_indices: Vec<_> = result.shown.clone();
        all_indices.extend(result.overflow.iter().copied());
        all_indices.sort_unstable();
        assert_eq!(all_indices, (0..items.len()).collect::<Vec<_>>());

        let overflow_id = result.overflow_button_id.expect("overflow trigger");
        let overflow_rect = ctx
            .read_response(overflow_id)
            .expect("overflow response")
            .rect;
        assert!(overflow_rect.left() >= -0.5);
        assert!(overflow_rect.right() <= 240.5);
        let mut last_right = f32::NEG_INFINITY;
        for &index in &result.shown {
            let rect = ctx
                .read_response(id.with(("toolbar-item", index)))
                .expect("visible toolbar item")
                .rect;
            assert!(rect.left() >= last_right - 0.5, "toolbar items overlap");
            last_right = rect.right();
            assert!(rect.right() <= overflow_rect.left() + 0.5);
        }

        let (fits, _) = toolbar_in_width(
            &ctx,
            vec2(600.0, 300.0),
            500.0,
            Id::unique("widgets-fitting-toolbar"),
            &[ToolbarItem::new("Open"), ToolbarItem::new("Save")],
        );
        assert_eq!(fits.state, ToolbarState::Fits);
        assert!(fits.overflow.is_empty());
    }

    #[test]
    fn overflow_label_uses_the_actual_count_including_three_digits() {
        let ctx = setup(ThemePreset::Light, false, vec2(400.0, 300.0));
        let items: Vec<_> = (0..130).map(|_| ToolbarItem::new("I")).collect();
        let (result, output) = toolbar_in_width(
            &ctx,
            vec2(400.0, 300.0),
            180.0,
            Id::unique("widgets-three-digit-toolbar"),
            &items,
        );
        assert_eq!(result.state, ToolbarState::Overflow);
        assert!(
            result.overflow.len() > 99,
            "test needs a three-digit overflow count, got {}",
            result.overflow.len()
        );
        let expected = format!("+{}", result.overflow.len());
        let labels = text_labels(&output);
        assert!(
            labels.iter().any(|label| label == &expected),
            "expected {expected} among painted labels {labels:?}"
        );
        assert!(!labels.iter().any(|label| label == "+99"));
    }

    #[test]
    fn disabled_toolbar_items_are_inert_in_the_bar_and_overflow_menu() {
        let ctx = setup(ThemePreset::FaseelaDark, false, vec2(500.0, 300.0));
        let labels = [
            "New", "Open", "Save", "Run", "Build", "Test", "Debug", "Deploy", "Inspect", "Profile",
            "Archive", "Export",
        ];
        let items: Vec<_> = labels
            .iter()
            .map(|label| ToolbarItem::new(label).disabled())
            .collect();
        let id = Id::unique("widgets-disabled-toolbar");
        let size = vec2(500.0, 300.0);
        let run = |events: Vec<Event>| {
            let mut result = None;
            frame(&ctx, size, events, |ui| {
                ui.allocate_ui_with_layout(
                    vec2(150.0, 40.0),
                    Layout::top_down(egui::Align::Min),
                    |ui| {
                        result = Some(toolbar_response(ui, id, &items));
                    },
                );
            });
            result.expect("toolbar result")
        };

        let first = run(vec![]);
        let visible_index = first.shown[0];
        let visible_id = id.with(("toolbar-item", visible_index));
        let visible = ctx.read_response(visible_id).expect("visible item");
        assert!(!visible.enabled());

        let clicked_visible = run(click_at(visible.rect.center()));
        assert_eq!(clicked_visible.activated, None);

        let trigger = first.overflow_button_id.expect("overflow trigger");
        let trigger_rect = ctx.read_response(trigger).expect("trigger response").rect;
        let opened = run(click_at(trigger_rect.center()));
        assert_eq!(opened.state, ToolbarState::Overflow);
        // One extra pass paints the popup contents so their responses can be inspected.
        let (_, _) = toolbar_in_width(&ctx, size, 150.0, id, &items);
        let overflow_index = first.overflow[0];
        let item_id = id.with(("overflow-item", overflow_index));
        let item = ctx.read_response(item_id).expect("overflow item response");
        assert!(!item.enabled());
        let clicked_overflow = run(click_at(item.rect.center()));
        assert_ne!(clicked_overflow.activated, Some(overflow_index));
    }

    #[test]
    fn toolbar_item_ids_are_stable_in_ltr_and_rtl() {
        let items = [
            ToolbarItem::new("Open"),
            ToolbarItem::new("Save"),
            ToolbarItem::new("Run"),
        ];
        let id = Id::unique("widgets-stable-toolbar");
        for rtl in [false, true] {
            let ctx = setup(ThemePreset::FaseelaDark, rtl, vec2(500.0, 300.0));
            let (result, _) = toolbar_in_width(&ctx, vec2(500.0, 300.0), 420.0, id, &items);
            assert_eq!(result.state, ToolbarState::Fits);
            for index in 0..items.len() {
                let response = ctx
                    .read_response(id.with(("toolbar-item", index)))
                    .expect("stable toolbar item id");
                assert!(response.rect.width() > 0.0);
            }
        }
    }
}
