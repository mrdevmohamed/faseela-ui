//! Connected exclusive selection with direction-aware keyboard navigation.
use super::SelectionResponse;
use crate::{direction, theme};
use egui::{
    CornerRadius, EventFilter, FontId, Id, RichText, Role, Sense, Stroke, StrokeKind, Ui,
    WidgetInfo, pos2, vec2,
};

const LABEL_FONT_SIZE: f32 = 13.0;
const SEGMENT_FONT_SIZE: f32 = 12.5;
const SEGMENT_HEIGHT: f32 = 32.0;
const SEGMENT_PADDING: f32 = 10.0;

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
                        ui.memory_mut(|memory| {
                            memory.set_focus_lock_filter(
                                segments[current].3.id,
                                EventFilter {
                                    horizontal_arrows: true,
                                    vertical_arrows: true,
                                    ..Default::default()
                                },
                            )
                        });
                        let mut target = None;
                        ui.input_mut(|input| {
                            if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
                                target = Some(0);
                            } else if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
                                target = Some(labels.len() - 1);
                            } else if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                                target = Some(previous(current, labels.len()));
                            } else if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                            {
                                target = Some(next(current, labels.len()));
                            } else if input.consume_key(
                                egui::Modifiers::NONE,
                                if rtl {
                                    egui::Key::ArrowRight
                                } else {
                                    egui::Key::ArrowLeft
                                },
                            ) {
                                target = Some(previous(current, labels.len()));
                            } else if input.consume_key(
                                egui::Modifiers::NONE,
                                if rtl {
                                    egui::Key::ArrowLeft
                                } else {
                                    egui::Key::ArrowRight
                                },
                            ) {
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
                    if !segment_rect.is_positive() {
                        continue;
                    }
                    if is_selected
                        || (enabled && (response.hovered() || response.is_pointer_button_down_on()))
                    {
                        ui.painter().rect_filled(
                            segment_rect,
                            CornerRadius::same(7),
                            if !enabled {
                                tokens.disabled
                            } else if is_selected {
                                tokens.selection
                            } else if response.is_pointer_button_down_on() {
                                tokens.active
                            } else {
                                tokens.hover
                            },
                        );
                    }
                    let foreground = if !enabled {
                        tokens.disabled_text
                    } else if is_selected || response.is_pointer_button_down_on() {
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
                    if *index + 1 < segments.len() && !is_selected {
                        let x = if rtl {
                            segment_rect.left()
                        } else {
                            segment_rect.right()
                        };
                        ui.painter().line_segment(
                            [pos2(x, inner_rect.top()), pos2(x, inner_rect.bottom())],
                            Stroke::new(1.0, tokens.border),
                        );
                    }
                    if enabled && response.has_focus() {
                        ui.painter().rect_stroke(
                            segment_rect.shrink(1.0),
                            CornerRadius::same(6),
                            Stroke::new(
                                2.0,
                                if is_selected {
                                    tokens.on_primary
                                } else {
                                    tokens.focus
                                },
                            ),
                            StrokeKind::Inside,
                        );
                    }
                    response.widget_info(|| {
                        WidgetInfo::selected(Role::RadioButton, enabled, is_selected, *label)
                    });
                }

                group_response.widget_info(|| {
                    WidgetInfo::labeled(
                        Role::RadioGroup,
                        enabled,
                        self.label.unwrap_or("Segmented control"),
                    )
                });
                group_response
            })
            .inner;

        let mut response = response;
        if *self.selected != selected {
            response.mark_changed();
        }
        *self.selected = selected;
        SelectionResponse {
            response,
            activated,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn step_helpers_wrap_both_directions() {
        assert_eq!(previous(0, 3), 2);
        assert_eq!(next(2, 3), 0);
        assert_eq!(previous(4, 0), 0);
        assert_eq!(next(4, 0), 0);
    }
}
