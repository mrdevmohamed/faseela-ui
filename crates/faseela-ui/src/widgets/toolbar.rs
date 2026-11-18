//! Responsive command presentation; applications own commands and fallback policy.
use crate::{
    buttons::{Button as ThemedButton, ButtonVariant, ControlSize},
    direction,
    layout::{ToolbarFit, fit_toolbar},
    theme,
};
use egui::{Color32, FontId, Id, Layout, RichText, Sense, Stroke, Ui, Vec2, WidgetInfo, vec2};

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
pub(super) fn layout_items(
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
