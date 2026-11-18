//! Semantic badges and reusable status tones.
use crate::theme::{self, ThemeTokens};
use egui::{Color32, CornerRadius, FontId, Response, Sense, Ui, Vec2, Widget, WidgetInfo, vec2};

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

pub(super) const BADGE_FONT_SIZE: f32 = 11.0;
pub(super) const BADGE_PADDING: Vec2 = vec2(8.0, 3.0);

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
        let foreground = self.tone.foreground(tokens);
        let font_id = FontId::proportional(BADGE_FONT_SIZE);
        let galley =
            ui.fonts_mut(|fonts| fonts.layout_no_wrap(self.text.to_owned(), font_id, foreground));
        let size = vec2(
            galley.rect.width() + BADGE_PADDING.x * 2.0,
            galley.rect.height() + BADGE_PADDING.y * 2.0,
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let fill = self.tone.color(tokens);
        ui.painter().rect_filled(rect, CornerRadius::same(6), fill);
        ui.painter()
            .galley(rect.center() - galley.rect.size() * 0.5, galley, foreground);
        response.widget_info(|| WidgetInfo::labeled(egui::Role::Label, true, self.text));
        response
    }
}
