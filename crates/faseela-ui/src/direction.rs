//! Interface direction helpers for left-to-right and right-to-left applications.
//!
//! egui does not impose a global reading direction. Apply a direction to the context, then use
//! these helpers for layouts and navigation that should follow the interface's reading order.
//! Text and data visualization remain owned by the application.

use egui::{Align, Context, Id, Key, Layout, Rect, Ui, Vec2};

fn direction_id() -> Id {
    Id::new("faseela_ui::direction")
}

/// Reading direction of the application chrome and directional controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

/// User preference for choosing the interface direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Setting {
    /// Follow the script used by the UI language.
    #[default]
    Language,
    Ltr,
    Rtl,
}

impl Setting {
    /// Parse `language`, `ltr`, or `rtl`, ignoring ASCII case and surrounding whitespace.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "language" => Some(Self::Language),
            "ltr" => Some(Self::Ltr),
            "rtl" => Some(Self::Rtl),
            _ => None,
        }
    }

    /// Resolve the preference against a BCP-47 language tag (for example `ar-EG` or `en-US`).
    pub fn resolve(self, language_tag: &str) -> Direction {
        match self {
            Self::Ltr => Direction::Ltr,
            Self::Rtl => Direction::Rtl,
            Self::Language => language_direction(language_tag),
        }
    }
}

/// Infer direction from the primary language subtag.
///
/// This deliberately uses a small explicit RTL language list rather than guessing from the
/// machine locale. Applications with custom scripts or user overrides can select `Setting::Rtl`.
pub fn language_direction(language_tag: &str) -> Direction {
    let language = language_tag
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    match language.as_str() {
        "ar" | "arc" | "azb" | "bal" | "bqi" | "ckb" | "dv" | "fa" | "glk" | "he" | "khw"
        | "ks" | "ku" | "mzn" | "ps" | "sd" | "ug" | "ur" | "yi" => Direction::Rtl,
        _ => Direction::Ltr,
    }
}

/// Store the interface direction on the context. Call after replacing the application's egui
/// style as the direction is independent from theme selection.
pub fn apply(ctx: &Context, direction: Direction) {
    ctx.data_mut(|data| data.insert_temp(direction_id(), direction));
}

/// Anything that can provide the current interface direction.
pub trait HasDirection {
    fn direction(&self) -> Direction;
}

impl HasDirection for Direction {
    fn direction(&self) -> Direction {
        *self
    }
}

impl HasDirection for Context {
    fn direction(&self) -> Direction {
        self.data(|data| data.get_temp(direction_id()))
            .unwrap_or_default()
    }
}

impl HasDirection for Ui {
    fn direction(&self) -> Direction {
        self.ctx().direction()
    }
}

/// Whether the interface is right-to-left.
pub fn is_rtl(source: &impl HasDirection) -> bool {
    source.direction() == Direction::Rtl
}

/// A vertically centred row that follows the reading order.
pub fn row(source: &impl HasDirection) -> Layout {
    match source.direction() {
        Direction::Ltr => Layout::left_to_right(Align::Center),
        Direction::Rtl => Layout::right_to_left(Align::Center),
    }
}

/// A vertically centred row whose items are pinned to the reading end.
pub fn trailing(source: &impl HasDirection) -> Layout {
    match source.direction() {
        Direction::Ltr => Layout::right_to_left(Align::Center),
        Direction::Rtl => Layout::left_to_right(Align::Center),
    }
}

/// Alignment at the reading start of a row.
pub fn start_align(source: &impl HasDirection) -> Align {
    match source.direction() {
        Direction::Ltr => Align::Min,
        Direction::Rtl => Align::Max,
    }
}

/// X coordinate of the reading-start edge of a rectangle.
pub fn start_x(source: &impl HasDirection, rect: Rect) -> f32 {
    match source.direction() {
        Direction::Ltr => rect.left(),
        Direction::Rtl => rect.right(),
    }
}

/// X coordinate of the reading-end edge of a rectangle.
pub fn end_x(source: &impl HasDirection, rect: Rect) -> f32 {
    match source.direction() {
        Direction::Ltr => rect.right(),
        Direction::Rtl => rect.left(),
    }
}

/// Signed horizontal offset toward the reading end.
pub fn forward(source: &impl HasDirection) -> f32 {
    match source.direction() {
        Direction::Ltr => 1.0,
        Direction::Rtl => -1.0,
    }
}

/// Logical previous/next movement for a row of controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Previous,
    Next,
}

/// Interpret horizontal arrow keys in reading order. Returns `None` for other keys.
pub fn step(source: &impl HasDirection, key: Key) -> Option<Step> {
    let (previous, next) = match source.direction() {
        Direction::Ltr => (Key::ArrowLeft, Key::ArrowRight),
        Direction::Rtl => (Key::ArrowRight, Key::ArrowLeft),
    };
    if key == previous {
        Some(Step::Previous)
    } else if key == next {
        Some(Step::Next)
    } else {
        None
    }
}

/// Allocate a full-width row using the current reading direction.
pub fn horizontal<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
    let width = ui.available_size_before_wrap().x.max(0.0);
    let height = ui.spacing().interact_size.y.max(0.0);
    let size = Vec2::new(width, height);
    ui.allocate_ui_with_layout(size, row(ui), add)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_common_rtl_language_tags() {
        for tag in ["ar", "ar-EG", "fa_IR", "he", "ur-PK", "ps-AF"] {
            assert_eq!(language_direction(tag), Direction::Rtl, "{tag}");
        }
        for tag in ["en", "en-GB", "fr", "de", "tr"] {
            assert_eq!(language_direction(tag), Direction::Ltr, "{tag}");
        }
    }

    #[test]
    fn explicit_setting_overrides_language() {
        assert_eq!(Setting::Ltr.resolve("ar-EG"), Direction::Ltr);
        assert_eq!(Setting::Rtl.resolve("en-US"), Direction::Rtl);
        assert_eq!(Setting::Language.resolve("ar-EG"), Direction::Rtl);
        assert_eq!(Setting::parse(" RTL "), Some(Setting::Rtl));
        assert_eq!(Setting::parse("unknown"), None);
    }

    #[test]
    fn context_direction_and_geometry_follow_reading_order() {
        let ctx = Context::default();
        let rect = Rect::from_min_size(egui::Pos2::new(10.0, 20.0), Vec2::new(100.0, 40.0));
        assert_eq!(ctx.direction(), Direction::Ltr);
        assert_eq!(start_x(&ctx, rect), 10.0);
        assert_eq!(end_x(&ctx, rect), 110.0);
        assert_eq!(forward(&ctx), 1.0);

        apply(&ctx, Direction::Rtl);
        assert!(is_rtl(&ctx));
        assert_eq!(start_x(&ctx, rect), 110.0);
        assert_eq!(end_x(&ctx, rect), 10.0);
        assert_eq!(forward(&ctx), -1.0);
        assert_eq!(step(&ctx, Key::ArrowRight), Some(Step::Previous));
        assert_eq!(step(&ctx, Key::ArrowLeft), Some(Step::Next));
        assert_eq!(step(&ctx, Key::Enter), None);
    }
}
