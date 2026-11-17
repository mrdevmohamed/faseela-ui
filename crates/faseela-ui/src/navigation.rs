//! Navigation components with caller-owned selection and application-owned actions.
//!
//! [`ButtonGroup`] and [`SegmentedControl`] present an exclusive choice among labeled options.
//! [`Breadcrumb`] presents a navigation path whose earlier entries remain interactive. None of
//! these widgets change application routes or commands: they update the values passed in and
//! return the activated index as a proposal.

use egui::{
    CornerRadius, FontId, Id, Response, RichText, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo,
    pos2, vec2,
};

use crate::{
    buttons::{Button, ButtonVariant, ControlSize},
    direction, theme,
};

const LABEL_FONT_SIZE: f32 = 13.0;
const SEGMENT_FONT_SIZE: f32 = 12.5;
const SEGMENT_HEIGHT: f32 = 32.0;
const SEGMENT_PADDING: f32 = 10.0;

/// The result of an exclusive navigation control.
pub struct SelectionResponse {
    /// Container response, including focus and enabled state.
    pub response: Response,
    /// The index activated by click or keyboard this frame.
    pub activated: Option<usize>,
}

/// A horizontal group of related action buttons with caller-owned selection.
///
/// Clicking a button updates `selected` and returns its index in
/// [`SelectionResponse::activated`]. When the interface is right-to-left the group is laid out
/// from right to left and wraps from the right edge. An out-of-range selection paints no button
/// as selected and is not silently changed; clicking selects a valid index again.
///
/// ```
/// use faseela_ui::navigation::ButtonGroup;
///
/// egui::__run_test_ui(|ui| {
///     let mut selected = 0;
///     let result = ButtonGroup::new(&["Day", "Week", "Month"], &mut selected).show(ui);
///     let _ = result.activated;
/// });
/// ```
pub struct ButtonGroup<'a> {
    labels: &'a [&'a str],
    selected: &'a mut usize,
    enabled: bool,
    id: Option<Id>,
    size: ControlSize,
    variant: ButtonVariant,
    accessible_label: &'a str,
}

impl<'a> ButtonGroup<'a> {
    /// Create a group bound to an application-owned selected index.
    pub fn new(labels: &'a [&'a str], selected: &'a mut usize) -> Self {
        Self {
            labels,
            selected,
            enabled: true,
            id: None,
            size: ControlSize::Medium,
            variant: ButtonVariant::Outline,
            accessible_label: "Button group",
        }
    }

    /// Enable or disable the whole group.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assign a stable ID for the group and its buttons.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Control size. Defaults to [`ControlSize::Medium`].
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Button variant. Defaults to [`ButtonVariant::Outline`].
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Accessible name for the group container.
    pub fn accessible_label(mut self, label: &'a str) -> Self {
        self.accessible_label = label;
        self
    }

    /// Show the group.
    pub fn show(self, ui: &mut Ui) -> SelectionResponse {
        if !self.enabled {
            return ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner;
        }
        self.show_in(ui)
    }

    fn show_in(self, ui: &mut Ui) -> SelectionResponse {
        let enabled = ui.is_enabled();
        let id = match self.id {
            Some(id) => id,
            None => {
                let id = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);
                id
            }
        };
        let labels = self.labels;
        let size = self.size;
        let variant = self.variant;
        let mut selected = *self.selected;
        let mut activated = None;

        let group = ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
            ui.with_layout(direction::row(ui), |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (index, label) in labels.iter().enumerate() {
                        let is_selected = selected == index;
                        let response = Button::new(*label)
                            .variant(variant)
                            .size(size)
                            .selected(is_selected)
                            .id(id.with(("group-item", index)))
                            .show(ui);
                        if response.clicked() {
                            selected = index;
                            activated = Some(index);
                        }
                    }
                })
            })
        });
        let response = group.inner.inner.response;
        response.widget_info(|| WidgetInfo::labeled(Role::Toolbar, enabled, self.accessible_label));
        *self.selected = selected;
        SelectionResponse {
            response,
            activated,
        }
    }
}

/// A connected single-choice control for compact view or mode switches.
///
/// The control shrinks and clips segment labels rather than overflowing its available width.
/// Arrow keys follow the interface reading direction, Home selects the first segment, and End
/// selects the last. Space and Enter activate the focused segment through egui's standard
/// keyboard handling.
///
/// ```
/// use faseela_ui::navigation::SegmentedControl;
///
/// egui::__run_test_ui(|ui| {
///     let mut selected = 1;
///     let result = SegmentedControl::new(&["List", "Grid"], &mut selected)
///         .label("Layout")
///         .show(ui);
///     let _ = result.activated;
/// });
/// ```
pub struct SegmentedControl<'a> {
    labels: &'a [&'a str],
    selected: &'a mut usize,
    label: Option<&'a str>,
    enabled: bool,
    full_width: bool,
    id: Option<Id>,
}

impl<'a> SegmentedControl<'a> {
    /// Create a control bound to an application-owned selected index.
    pub fn new(labels: &'a [&'a str], selected: &'a mut usize) -> Self {
        Self {
            labels,
            selected,
            label: None,
            enabled: true,
            full_width: false,
            id: None,
        }
    }

    /// Visible label above the control.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Enable or disable every segment.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Expand to the available width with equal segment widths.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    /// Assign a stable ID for the control and its segments.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Show the control.
    pub fn show(self, ui: &mut Ui) -> SelectionResponse {
        if !self.enabled {
            return ui.add_enabled_ui(false, |ui| self.show_in(ui)).inner;
        }
        self.show_in(ui)
    }

    fn show_in(self, ui: &mut Ui) -> SelectionResponse {
        let tokens = theme::active_tokens(ui.ctx());
        let enabled = ui.is_enabled();
        let id = match self.id {
            Some(id) => id,
            None => {
                let id = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);
                id
            }
        };
        let rtl = direction::is_rtl(ui);
        let labels = self.labels;
        let mut selected = *self.selected;
        let mut activated = None;

        if labels.is_empty() {
            let (rect, response) = ui.allocate_exact_size(
                vec2(ui.available_width().max(0.0), SEGMENT_HEIGHT),
                Sense::hover(),
            );
            let _ = rect;
            return SelectionResponse {
                response,
                activated: None,
            };
        }

        let response = ui
            .vertical(|ui| {
                if let Some(label) = self.label {
                    ui.label(
                        RichText::new(label)
                            .size(LABEL_FONT_SIZE)
                            .color(if enabled {
                                tokens.text
                            } else {
                                tokens.disabled_text
                            }),
                    );
                }

                let available = ui.available_width().max(0.0);
                let natural: Vec<f32> = labels
                    .iter()
                    .map(|label| {
                        let width = ui
                            .fonts_mut(|fonts| {
                                fonts.layout_no_wrap(
                                    (*label).to_owned(),
                                    FontId::proportional(SEGMENT_FONT_SIZE),
                                    tokens.text,
                                )
                            })
                            .rect
                            .width();
                        width + SEGMENT_PADDING * 2.0
                    })
                    .collect();
                let total: f32 = natural.iter().sum();
                let width = if self.full_width {
                    available
                } else {
                    total.min(available)
                };
                let widths: Vec<f32> = if self.full_width {
                    vec![width / labels.len() as f32; labels.len()]
                } else if total <= available {
                    natural
                } else {
                    natural
                        .iter()
                        .map(|natural| natural * width / total.max(1.0))
                        .collect()
                };

                let (rect, _) = ui.allocate_exact_size(vec2(width, SEGMENT_HEIGHT), Sense::hover());
                let group_response = ui.interact(rect, id.with("segment-group"), Sense::hover());
                let sense = if enabled {
                    Sense::click()
                } else {
                    Sense::hover()
                };

                let mut cursor = if rtl { rect.right() } else { rect.left() };
                let mut segments = Vec::with_capacity(labels.len());
                for (index, label) in labels.iter().enumerate() {
                    let segment_width = widths.get(index).copied().unwrap_or(0.0).min(width);
                    let segment_rect = if rtl {
                        cursor -= segment_width;
                        egui::Rect::from_min_size(
                            pos2(cursor, rect.top()),
                            vec2(segment_width, rect.height()),
                        )
                    } else {
                        let segment_rect = egui::Rect::from_min_size(
                            pos2(cursor, rect.top()),
                            vec2(segment_width, rect.height()),
                        );
                        cursor += segment_width;
                        segment_rect
                    };
                    let response = ui.interact(segment_rect, id.with(("segment", index)), sense);
                    segments.push((index, *label, segment_rect, response));
                }

                if enabled {
                    if let Some((index, _, _, _)) = segments
                        .iter()
                        .find(|(_, _, _, response)| response.clicked())
                    {
                        selected = *index;
                        activated = Some(*index);
                    }

                    let focused = segments
                        .iter()
                        .find(|(_, _, _, response)| response.has_focus())
                        .map(|(index, _, _, _)| *index);
                    if let Some(current) = focused {
                        let mut target = None;
                        ui.input(|input| {
                            if input.key_pressed(egui::Key::Home) {
                                target = Some(0);
                            } else if input.key_pressed(egui::Key::End) {
                                target = Some(labels.len() - 1);
                            } else if input.key_pressed(egui::Key::ArrowUp) {
                                target = Some(previous(current, labels.len()));
                            } else if input.key_pressed(egui::Key::ArrowDown) {
                                target = Some(next(current, labels.len()));
                            } else if input.key_pressed(if rtl {
                                egui::Key::ArrowRight
                            } else {
                                egui::Key::ArrowLeft
                            }) {
                                target = Some(previous(current, labels.len()));
                            } else if input.key_pressed(if rtl {
                                egui::Key::ArrowLeft
                            } else {
                                egui::Key::ArrowRight
                            }) {
                                target = Some(next(current, labels.len()));
                            }
                        });
                        if let Some(target) = target {
                            selected = target;
                            activated = Some(target);
                            if let Some((_, _, _, response)) = segments.get(target) {
                                response.request_focus();
                            }
                        }
                    }
                }

                // Paint after selection and focus are final for this frame.
                ui.painter().rect(
                    rect,
                    CornerRadius::same(9),
                    if enabled {
                        tokens.surface
                    } else {
                        tokens.disabled
                    },
                    Stroke::new(1.0, tokens.control_border),
                    StrokeKind::Inside,
                );
                let inner_rect = rect.shrink(2.0);
                for (index, label, segment_rect, response) in &segments {
                    let is_selected = selected == *index;
                    let segment_rect = segment_rect.intersect(inner_rect);
                    if is_selected {
                        ui.painter().rect_filled(
                            segment_rect,
                            CornerRadius::same(7),
                            if enabled {
                                tokens.selection
                            } else {
                                tokens.disabled
                            },
                        );
                    }
                    let foreground = if !enabled {
                        tokens.disabled_text
                    } else if is_selected {
                        tokens.on_primary
                    } else {
                        tokens.text
                    };
                    let galley = ui.fonts_mut(|fonts| {
                        fonts.layout_no_wrap(
                            (*label).to_owned(),
                            FontId::proportional(SEGMENT_FONT_SIZE),
                            foreground,
                        )
                    });
                    ui.painter().with_clip_rect(segment_rect).galley(
                        segment_rect.center() - galley.rect.size() * 0.5,
                        galley,
                        foreground,
                    );
                    if *index + 1 < segments.len() && !is_selected && !rtl {
                        let x = segment_rect.right();
                        ui.painter().line_segment(
                            [pos2(x, inner_rect.top()), pos2(x, inner_rect.bottom())],
                            Stroke::new(1.0, tokens.border),
                        );
                    }
                    if enabled && response.has_focus() {
                        ui.painter().rect_stroke(
                            segment_rect.shrink(1.0),
                            CornerRadius::same(6),
                            Stroke::new(2.0, tokens.focus),
                            StrokeKind::Inside,
                        );
                    }
                    response.widget_info(|| {
                        WidgetInfo::selected(Role::RadioButton, enabled, is_selected, *label)
                    });
                }

                group_response.widget_info(|| {
                    WidgetInfo::labeled(Role::RadioGroup, enabled, "Segmented control")
                });
                group_response
            })
            .inner;

        *self.selected = selected;
        SelectionResponse {
            response,
            activated,
        }
    }
}

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

fn previous(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (current + len - 1) % len
    }
}

fn next(current: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (current + 1) % len }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemePreset;
    use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect};

    fn setup(preset: ThemePreset, rtl: bool) -> Context {
        let ctx = Context::default();
        theme::apply_preset(&ctx, preset);
        direction::apply(
            &ctx,
            if rtl {
                direction::Direction::Rtl
            } else {
                direction::Direction::Ltr
            },
        );
        ctx
    }

    fn frame(ctx: &Context, events: Vec<Event>, mut add: impl FnMut(&mut Ui)) -> egui::FullOutput {
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(420.0, 320.0))),
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

    #[test]
    fn button_group_click_updates_caller_selection() {
        let ctx = setup(ThemePreset::Light, false);
        let id = Id::unique("button-group");
        let mut selected = 0usize;
        let _ = frame(&ctx, vec![], |ui| {
            ButtonGroup::new(&["Day", "Week", "Month"], &mut selected)
                .id(id)
                .show(ui);
        });
        let rect = ctx
            .read_response(id.with(("group-item", 2)))
            .expect("third group button")
            .rect;
        let _ = frame(&ctx, click_at(rect.center()), |ui| {
            let result = ButtonGroup::new(&["Day", "Week", "Month"], &mut selected)
                .id(id)
                .show(ui);
            assert_eq!(result.activated, Some(2));
        });
        assert_eq!(selected, 2);
    }

    #[test]
    fn segmented_control_click_and_arrow_keys_update_selection() {
        let ctx = setup(ThemePreset::FaseelaDark, false);
        let id = Id::unique("segmented");
        let mut selected = 0usize;
        let _ = frame(&ctx, vec![], |ui| {
            SegmentedControl::new(&["List", "Grid", "Board"], &mut selected)
                .label("Layout")
                .id(id)
                .show(ui);
        });
        let second = ctx
            .read_response(id.with(("segment", 1)))
            .expect("second segment")
            .rect;
        let _ = frame(&ctx, click_at(second.center()), |ui| {
            SegmentedControl::new(&["List", "Grid", "Board"], &mut selected)
                .label("Layout")
                .id(id)
                .show(ui);
        });
        assert_eq!(selected, 1);

        let _ = frame(&ctx, vec![], |ui| {
            SegmentedControl::new(&["List", "Grid", "Board"], &mut selected)
                .label("Layout")
                .id(id)
                .show(ui);
            ui.memory_mut(|memory| memory.request_focus(id.with(("segment", 1))));
        });
        let _ = frame(
            &ctx,
            vec![Event::Key {
                key: egui::Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            |ui| {
                SegmentedControl::new(&["List", "Grid", "Board"], &mut selected)
                    .label("Layout")
                    .id(id)
                    .show(ui);
            },
        );
        assert_eq!(selected, 2);
    }

    #[test]
    fn segmented_control_never_exceeds_available_width() {
        for rtl in [false, true] {
            let ctx = setup(ThemePreset::Midnight, rtl);
            let mut selected = 0usize;
            let id = Id::unique("segmented-narrow");
            let _ = frame(&ctx, vec![], |ui| {
                ui.set_width(120.0);
                let result = SegmentedControl::new(
                    &[
                        "A very long first option",
                        "Another long option",
                        "A third long option",
                    ],
                    &mut selected,
                )
                .id(id)
                .show(ui);
                assert!(result.response.rect.width() <= 120.5);
            });
            let mut right = None;
            for index in 0..3 {
                let rect = ctx
                    .read_response(id.with(("segment", index)))
                    .expect("segment response")
                    .rect;
                assert!(rect.width() >= 0.0);
                if index == 0 {
                    right = Some(rect.right());
                }
            }
            if rtl {
                assert!(right.expect("first segment") >= 119.0);
            }
        }
    }

    #[test]
    fn disabled_group_and_segmented_control_do_not_activate() {
        let ctx = setup(ThemePreset::Forest, false);
        let group_id = Id::unique("disabled-group");
        let segmented_id = Id::unique("disabled-segmented");
        let mut group = 1usize;
        let mut segmented = 1usize;
        let _ = frame(&ctx, vec![], |ui| {
            ButtonGroup::new(&["One", "Two"], &mut group)
                .id(group_id)
                .enabled(false)
                .show(ui);
            SegmentedControl::new(&["One", "Two"], &mut segmented)
                .id(segmented_id)
                .enabled(false)
                .show(ui);
        });
        let group_rect = ctx
            .read_response(group_id.with(("group-item", 0)))
            .expect("group item")
            .rect;
        let segment_rect = ctx
            .read_response(segmented_id.with(("segment", 0)))
            .expect("segment")
            .rect;
        let _ = frame(
            &ctx,
            {
                let mut events = click_at(group_rect.center());
                events.extend(click_at(segment_rect.center()));
                events
            },
            |ui| {
                ButtonGroup::new(&["One", "Two"], &mut group)
                    .id(group_id)
                    .enabled(false)
                    .show(ui);
                SegmentedControl::new(&["One", "Two"], &mut segmented)
                    .id(segmented_id)
                    .enabled(false)
                    .show(ui);
            },
        );
        assert_eq!(group, 1);
        assert_eq!(segmented, 1);
    }

    #[test]
    fn breadcrumb_activates_links_but_not_the_current_page() {
        let ctx = setup(ThemePreset::Light, false);
        let id = Id::unique("breadcrumb");
        let mut activated = None;
        let _ = frame(&ctx, vec![], |ui| {
            activated = Breadcrumb::new(&["Projects", "Faseela", "Settings"])
                .id(id)
                .show(ui)
                .activated;
        });
        assert_eq!(activated, None);

        let link = ctx
            .read_response(id.with(("crumb", 1)))
            .expect("second crumb");
        assert!(link.enabled());
        let result = {
            let mut activated = None;
            let _ = frame(&ctx, click_at(link.rect.center()), |ui| {
                activated = Breadcrumb::new(&["Projects", "Faseela", "Settings"])
                    .id(id)
                    .show(ui)
                    .activated;
            });
            activated
        };
        assert_eq!(result, Some(1));
    }

    #[test]
    fn every_navigation_widget_renders_in_all_themes_and_directions() {
        for preset in ThemePreset::ALL {
            for rtl in [false, true] {
                let ctx = setup(preset, rtl);
                let mut group = 1usize;
                let mut segmented = 1usize;
                let output = frame(&ctx, vec![], |ui| {
                    ButtonGroup::new(&["One", "Two", "Three"], &mut group).show(ui);
                    SegmentedControl::new(&["One", "Two", "Three"], &mut segmented)
                        .label("View")
                        .full_width()
                        .show(ui);
                    Breadcrumb::new(&["Home", "Library", "Current"]).show(ui);
                });
                assert!(!output.shapes.is_empty(), "{preset:?} rtl={rtl}");
            }
        }
    }

    #[test]
    fn step_helpers_wrap_both_directions() {
        assert_eq!(previous(0, 3), 2);
        assert_eq!(next(2, 3), 0);
        assert_eq!(previous(4, 0), 0);
        assert_eq!(next(4, 0), 0);
    }
}
