//! Containers, headings, separators, and supporting tooltips.
use crate::theme::{self, ThemeTokens};
use egui::{
    Color32, CornerRadius, InnerResponse, Margin, Response, RichText, Sense, Stroke, Ui, vec2,
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
                let available = ui.available_width().max(0.0);
                let width = self.length.unwrap_or(available).min(available);
                vec2(width.max(0.0), 1.0)
            }
            SeparatorOrientation::Vertical => vec2(1.0, self.length.unwrap_or(20.0)),
        };
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let inset = self.inset.min(
            match self.orientation {
                SeparatorOrientation::Horizontal => rect.width(),
                SeparatorOrientation::Vertical => rect.height(),
            } * 0.5,
        );
        let line_rect = match self.orientation {
            SeparatorOrientation::Horizontal => egui::Rect::from_min_max(
                egui::pos2(rect.left() + inset, rect.center().y),
                egui::pos2(rect.right() - inset, rect.center().y),
            ),
            SeparatorOrientation::Vertical => egui::Rect::from_min_max(
                egui::pos2(rect.center().x, rect.top() + inset),
                egui::pos2(rect.center().x, rect.bottom() - inset),
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
