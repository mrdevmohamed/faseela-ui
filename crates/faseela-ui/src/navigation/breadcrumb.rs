//! Wrapping navigation links; applications own routing.
use crate::{direction, theme};
use egui::{
    CornerRadius, FontId, Id, Response, RichText, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo,
    pos2, vec2,
};

fn breadcrumb_link(
    ui: &mut Ui,
    id: Id,
    label: &str,
    tokens: crate::theme::ThemeTokens,
) -> Response {
    let enabled = ui.is_enabled();
    let font = FontId::proportional(12.5);
    let galley =
        ui.fonts_mut(|fonts| fonts.layout_no_wrap(label.to_owned(), font, tokens.secondary));
    let available = ui.available_width().max(1.0);
    let width = galley.rect.width().min(available);
    let size = vec2(width.max(1.0), galley.rect.height() + 2.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let response = ui.interact(
        rect,
        id,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| WidgetInfo::labeled(Role::Link, enabled, label));
    if ui.is_rect_visible(rect) {
        let color = if !enabled {
            tokens.disabled_text
        } else if response.hovered() {
            tokens.text
        } else {
            tokens.secondary
        };
        let text_pos = pos2(rect.left(), rect.center().y - galley.rect.height() * 0.5);
        let painter = ui.painter().with_clip_rect(rect);
        painter.galley_with_override_text_color(text_pos, galley.clone(), color);
        if enabled && (response.hovered() || response.has_focus()) {
            let underline_y = (text_pos.y + galley.rect.height() - 1.0).min(rect.bottom());
            painter.line_segment(
                [
                    pos2(text_pos.x, underline_y),
                    pos2(text_pos.x + galley.rect.width(), underline_y),
                ],
                Stroke::new(1.0, color),
            );
        }
        if enabled && response.has_focus() {
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(3),
                Stroke::new(2.0, tokens.focus),
                StrokeKind::Inside,
            );
        }
        if enabled && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
    response
}

/// The result of showing a [`Breadcrumb`].
pub struct BreadcrumbResponse {
    /// Container response.
    pub response: Response,
    /// Clicked or keyboard-activated path segment, excluding the current item.
    pub activated: Option<usize>,
}

/// A navigation path whose earlier entries are links and whose last entry is the current page.
///
/// The breadcrumb wraps within the available width instead of overflowing. Link entries are
/// keyboard focusable and activate with Space or Enter through egui's link widget. The last item
/// is a plain label and cannot be activated.
///
/// ```
/// use faseela_ui::navigation::Breadcrumb;
///
/// egui::__run_test_ui(|ui| {
///     let result = Breadcrumb::new(&["Projects", "Faseela", "Settings"]).show(ui);
///     let _ = result.activated;
/// });
/// ```
pub struct Breadcrumb<'a> {
    items: &'a [&'a str],
    enabled: bool,
    id: Option<Id>,
    accessible_label: &'a str,
}

impl<'a> Breadcrumb<'a> {
    /// Create a breadcrumb for `items`. The last item is treated as the current page.
    pub fn new(items: &'a [&'a str]) -> Self {
        Self {
            items,
            enabled: true,
            id: None,
            accessible_label: "Breadcrumb",
        }
    }

    /// Enable or disable navigation links.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable ID for the breadcrumb and its links.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Accessible name for the navigation container.
    pub fn accessible_label(mut self, label: &'a str) -> Self {
        self.accessible_label = label;
        self
    }

    /// Show the breadcrumb.
    pub fn show(self, ui: &mut Ui) -> BreadcrumbResponse {
        if !self.enabled {
            return ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner;
        }
        self.show_in(ui)
    }

    fn show_in(self, ui: &mut Ui) -> BreadcrumbResponse {
        let tokens = theme::active_tokens(ui.ctx());
        let enabled = ui.is_enabled();
        let rtl = direction::is_rtl(ui);
        let id = match self.id {
            Some(id) => id,
            None => {
                let id = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);
                id
            }
        };
        let items = self.items;
        let mut activated = None;

        let group = ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
            ui.with_layout(direction::row(ui), |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (index, item) in items.iter().enumerate() {
                        if index > 0 {
                            ui.label(
                                RichText::new(if rtl { "‹" } else { "›" })
                                    .size(12.0)
                                    .color(tokens.muted_text),
                            );
                        }
                        let last = index + 1 == items.len();
                        if last {
                            ui.label(RichText::new(*item).size(12.5).strong().color(tokens.text));
                        } else if enabled {
                            let link = breadcrumb_link(ui, id.with(("crumb", index)), item, tokens);
                            if link.clicked() {
                                activated = Some(index);
                            }
                        } else {
                            ui.label(RichText::new(*item).size(12.5).color(tokens.disabled_text));
                        }
                    }
                })
            })
        });
        let response = group.inner.inner.response;
        response
            .widget_info(|| WidgetInfo::labeled(Role::Navigation, enabled, self.accessible_label));
        BreadcrumbResponse {
            response,
            activated,
        }
    }
}
