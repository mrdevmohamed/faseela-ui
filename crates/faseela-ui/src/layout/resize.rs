//! A focusable divider that proposes bounded resize deltas.
//!
//! The application owns the split tree and applies returned deltas through SplitLayout::resized.

use std::ops::RangeInclusive;

use egui::{
    Color32, EventFilter, Id, Key, Modifiers, Rect, Response, Sense, Ui, Vec2, accesskit,
};

use super::SplitAxis;

const MAX_HANDLE_EXTENT: f32 = 1_000_000.0;
const MAX_HIT_MARGIN: f32 = 4096.0;
const MAX_STEP: f32 = 4096.0;

/// Colors used while a resize handle is hovered, dragged, or keyboard-focused.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResizeHandleStyle {
    /// Pointer-hover fill.
    pub hovered: Color32,
    /// Active pointer-drag fill.
    pub dragged: Color32,
    /// Keyboard-focus fill.
    pub focused: Color32,
}

/// A resize interaction proposal. The application applies the delta to its own split state.
pub struct ResizeResponse {
    /// The egui response for focus, hover, drag lifecycle, and interaction state.
    pub response: Response,
    /// Requested movement this frame in logical points along the split axis.
    ///
    /// This is a per-frame delta, not the total movement since the drag started. Apply it once
    /// through SplitLayout::resized; do not accumulate it a second time.
    pub delta: f32,
}

/// A focusable divider with pointer, arrow-key, and AccessKit resizing.
///
/// This widget never stores or changes a pane size. IDs and accessible labels are separate, so
/// translated labels do not change identity. Pointer drags retain egui's immediate drag behavior
/// and hold focus only for the gesture unless the handle was already keyboard-focused.
///
/// Only arrows along the split axis are consumed while this handle has focus. Tab remains
/// available for ordinary focus traversal. The accessible role is Splitter, oriented
/// perpendicular to the movement axis.
///
/// For horizontal splits the handle is vertical and moves horizontally. For vertical splits it
/// is horizontal and moves vertically.
pub struct ResizeHandle<'a> {
    id: Id,
    axis: SplitAxis,
    rect: Rect,
    label: &'a str,
    style: ResizeHandleStyle,
    hit_margin: f32,
    step: f32,
    value: Option<(f32, RangeInclusive<f32>)>,
    reversed: bool,
}

impl<'a> ResizeHandle<'a> {
    /// Create a divider with a stable ID, split axis, paint rectangle, and accessible label.
    pub fn new(
        id: Id,
        axis: SplitAxis,
        rect: Rect,
        label: &'a str,
        style: ResizeHandleStyle,
    ) -> Self {
        Self {
            id,
            axis,
            rect,
            label,
            style,
            hit_margin: 3.0,
            step: 8.0,
            value: None,
            reversed: false,
        }
    }

    /// Expand the hit rectangle along the movement axis, in logical points. Default: 3.
    ///
    /// Values outside 0..=4096 or non-finite values make the handle inert.
    pub fn hit_margin(mut self, margin: f32) -> Self {
        self.hit_margin = margin;
        self
    }

    /// Keyboard and accessibility increment/decrement in logical points. Default: 8.
    ///
    /// Values outside 0..=4096 or non-finite values make the handle inert.
    pub fn keyboard_step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Expose the current first-pane size and its valid range to accessibility.
    ///
    /// The values are logical points. The application remains responsible for updating the
    /// displayed value after accepting a delta. The current value must lie inside the finite,
    /// ordered range; otherwise the handle is inert.
    pub fn value(mut self, value: f32, range: RangeInclusive<f32>) -> Self {
        self.value = Some((value, range));
        self
    }

    /// Reverse AccessKit Increment/Decrement/SetValue semantics without reversing physical input.
    ///
    /// This is useful when the exposed value grows opposite to the physical movement direction.
    /// Pointer dragging and arrow keys always report physical movement along the split axis.
    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    /// Show the handle and return a resize proposal for the caller to apply.
    ///
    /// Invalid geometry, hit margins, keyboard steps, or configured accessible values make the
    /// handle inert. Delta is finite and nonzero only when an interaction requests a change.
    pub fn show(self, ui: &mut Ui) -> ResizeResponse {
        let hit = self.rect.expand2(match self.axis {
            SplitAxis::Horizontal => Vec2::new(self.hit_margin, 0.0),
            SplitAxis::Vertical => Vec2::new(0.0, self.hit_margin),
        });

        let valid = valid_rect(self.rect)
            && self.rect.width() > 0.0
            && self.rect.height() > 0.0
            && self.rect.width() <= MAX_HANDLE_EXTENT
            && self.rect.height() <= MAX_HANDLE_EXTENT
            && valid_rect(hit)
            && self.hit_margin.is_finite()
            && (0.0..=MAX_HIT_MARGIN).contains(&self.hit_margin)
            && self.step.is_finite()
            && self.step > 0.0
            && self.step <= MAX_STEP
            && self.value.as_ref().is_none_or(|(value, range)| {
                value.is_finite()
                    && range.start().is_finite()
                    && range.end().is_finite()
                    && range.start() <= range.end()
                    && value >= range.start()
                    && value <= range.end()
            });

        let inert = Rect::from_min_size(ui.min_rect().min, Vec2::ZERO);
        let mut response = ui.interact(
            if valid { hit } else { inert },
            self.id,
            if valid { Sense::drag() } else { Sense::hover() },
        );
        let enabled = valid && response.enabled();
        let mut delta = 0.0;

        if enabled {
            drag_focus(ui, &response);

            if response.dragged() {
                let drag = response.drag_delta();
                delta = match self.axis {
                    SplitAxis::Horizontal => drag.x,
                    SplitAxis::Vertical => drag.y,
                };
            }

            if response.has_focus() {
                ui.memory_mut(|memory| {
                    memory.set_focus_lock_filter(
                        response.id,
                        EventFilter {
                            horizontal_arrows: self.axis == SplitAxis::Horizontal,
                            vertical_arrows: self.axis == SplitAxis::Vertical,
                            ..Default::default()
                        },
                    );
                });

                let (back, forward) = match self.axis {
                    SplitAxis::Horizontal => (Key::ArrowLeft, Key::ArrowRight),
                    SplitAxis::Vertical => (Key::ArrowUp, Key::ArrowDown),
                };

                ui.input_mut(|input| {
                    if input.consume_key(Modifiers::NONE, back) {
                        delta -= self.step;
                    }
                    if input.consume_key(Modifiers::NONE, forward) {
                        delta += self.step;
                    }
                });
            }

            let mut accessibility_delta = 0.0;
            let mut set_value_delta = None;

            ui.input_mut(|input| {
                input.consume_accesskit_action_requests(response.id, |request| {
                    match request.action {
                        accesskit::Action::Increment => {
                            accessibility_delta += self.step;
                            true
                        }
                        accesskit::Action::Decrement => {
                            accessibility_delta -= self.step;
                            true
                        }
                        accesskit::Action::SetValue => {
                            if let (
                                Some((value, range)),
                                Some(accesskit::ActionData::NumericValue(new_value)),
                            ) = (&self.value, &request.data)
                            {
                                if new_value.is_finite() {
                                    let requested = new_value.clamp(
                                        f64::from(*range.start()),
                                        f64::from(*range.end()),
                                    );
                                    set_value_delta =
                                        Some((requested - f64::from(*value)) as f32);
                                }
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                });
            });

            if let Some(change) = set_value_delta {
                delta = if self.reversed { -change } else { change };
            } else {
                delta += if self.reversed {
                    -accessibility_delta
                } else {
                    accessibility_delta
                };
            }
        }

        if !delta.is_finite() {
            delta = 0.0;
        }
        if delta != 0.0 {
            response.mark_changed();
        }

        response.widget_info(|| {
            egui::WidgetInfo::labeled(accesskit::Role::Splitter, enabled, self.label)
        });

        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(accesskit::Role::Splitter);
            // A horizontal split has a vertical divider, and vice versa.
            node.set_orientation(match self.axis {
                SplitAxis::Horizontal => accesskit::Orientation::Vertical,
                SplitAxis::Vertical => accesskit::Orientation::Horizontal,
            });

            if enabled {
                node.add_action(accesskit::Action::Increment);
                node.add_action(accesskit::Action::Decrement);

                if let Some((value, range)) = &self.value {
                    node.set_numeric_value(f64::from(*value));
                    node.set_min_numeric_value(f64::from(*range.start()));
                    node.set_max_numeric_value(f64::from(*range.end()));
                    node.set_numeric_value_step(f64::from(self.step));
                    node.add_action(accesskit::Action::SetValue);
                }
            }
        });

        if enabled {
            let fill = if response.dragged() {
                Some(self.style.dragged)
            } else if response.has_focus() {
                Some(self.style.focused)
            } else if response.hovered() {
                Some(self.style.hovered)
            } else {
                None
            };

            if let Some(fill) = fill {
                ui.painter().rect_filled(self.rect, 0.0, fill);
            }

            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(match self.axis {
                    SplitAxis::Horizontal => egui::CursorIcon::ResizeHorizontal,
                    SplitAxis::Vertical => egui::CursorIcon::ResizeVertical,
                });
            }
        }

        ResizeResponse { response, delta }
    }
}

fn valid_rect(rect: Rect) -> bool {
    rect.min.is_finite()
        && rect.max.is_finite()
        && rect.width().is_finite()
        && rect.height().is_finite()
        && rect.min.x <= rect.max.x
        && rect.min.y <= rect.max.y
}

/// A pointer drag focuses its handle for the gesture only and gives focus up on release, so the
/// app's shortcuts and canvas arrow keys work again. A handle focused from the keyboard before
/// the drag keeps its focus.
fn drag_focus(ui: &Ui, response: &Response) {
    let key = response.id.with("pointer-focus");

    if response.drag_started() {
        if !response.has_focus() {
            ui.ctx().data_mut(|data| data.insert_temp(key, ()));
        }
        response.request_focus();
    }

    if response.drag_stopped()
        && ui
            .ctx()
            .data_mut(|data| data.remove_temp::<()>(key))
            .is_some()
    {
        response.surrender_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{
        CentralPanel, Context, Event, FullOutput, Id, PointerButton, Pos2, RawInput, pos2, vec2,
    };

    struct HandleHarness {
        ctx: Context,
        axis: SplitAxis,
        rect: Rect,
        enabled: bool,
        value: f32,
        reversed: bool,
    }

    impl HandleHarness {
        fn new() -> Self {
            Self {
                ctx: Context::default(),
                axis: SplitAxis::Horizontal,
                rect: Rect::from_min_size(pos2(100.0, 20.0), vec2(4.0, 140.0)),
                enabled: true,
                value: 100.0,
                reversed: false,
            }
        }

        fn frame(&mut self, events: Vec<Event>) -> (ResizeResponse, FullOutput) {
            let ctx = self.ctx.clone();
            let axis = self.axis;
            let rect = self.rect;
            let enabled = self.enabled;
            let value = self.value;
            let reversed = self.reversed;
            let mut result = None;

            let output = ctx.run(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 200.0))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    CentralPanel::default().show(ctx, |ui| {
                        result = Some(
                            ui.add_enabled_ui(enabled, |ui| {
                                ResizeHandle::new(
                                    Id::new("test-divider"),
                                    axis,
                                    rect,
                                    "Resize panels",
                                    style(),
                                )
                                .value(value, 0.0..=300.0)
                                .reversed(reversed)
                                .show(ui)
                            })
                            .inner,
                        );
                    });
                },
            );

            (result.expect("the central panel must render the handle"), output)
        }

        fn settle(&mut self) -> ResizeResponse {
            let _ = self.frame(vec![]);
            self.frame(vec![]).0
        }
    }

    fn style() -> ResizeHandleStyle {
        ResizeHandleStyle {
            hovered: Color32::GREEN,
            dragged: Color32::RED,
            focused: Color32::BLUE,
        }
    }

    fn key(key: Key) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    fn pointer(pos: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }

    fn accesskit_event(
        id: Id,
        action: accesskit::Action,
        data: Option<accesskit::ActionData>,
    ) -> Event {
        Event::AccessKitActionRequest(accesskit::ActionRequest {
            action,
            target_tree: accesskit::TreeId::ROOT,
            target_node: id.accesskit_id(),
            data,
        })
    }

    #[test]
    fn pointer_drag_reports_only_the_delta_since_the_previous_frame() {
        let mut harness = HandleHarness::new();
        let center = harness.rect.center();
        let _ = harness.settle();

        let _ = harness.frame(vec![Event::PointerMoved(center)]);
        let _ = harness.frame(vec![pointer(center, true)]);

        let (response, _) =
            harness.frame(vec![Event::PointerMoved(center + vec2(12.0, 0.0))]);
        assert!(response.response.dragged());
        assert_eq!(response.delta, 12.0);

        let (response, _) =
            harness.frame(vec![Event::PointerMoved(center + vec2(16.0, 0.0))]);
        assert_eq!(response.delta, 4.0);

        let (response, _) = harness.frame(vec![pointer(center + vec2(16.0, 0.0), false)]);
        assert!(response.response.drag_stopped());
        assert_eq!(response.delta, 0.0);
        assert!(!response.response.has_focus());
    }

    #[test]
    fn focused_handle_uses_only_the_arrows_along_its_axis() {
        for (axis, forward, backward) in [
            (SplitAxis::Horizontal, Key::ArrowRight, Key::ArrowLeft),
            (SplitAxis::Vertical, Key::ArrowDown, Key::ArrowUp),
        ] {
            let mut harness = HandleHarness::new();
            harness.axis = axis;

            let response = harness.settle();
            harness
                .ctx
                .memory_mut(|memory| memory.request_focus(response.response.id));
            let _ = harness.frame(vec![]);

            let (response, _) = harness.frame(vec![key(forward)]);
            assert!(response.response.has_focus());
            assert_eq!(response.delta, 8.0);

            let (response, _) = harness.frame(vec![key(backward)]);
            assert!(response.response.has_focus());
            assert_eq!(response.delta, -8.0);

            let perpendicular = match axis {
                SplitAxis::Horizontal => Key::ArrowUp,
                SplitAxis::Vertical => Key::ArrowRight,
            };
            let (response, _) = harness.frame(vec![key(perpendicular)]);
            assert_eq!(response.delta, 0.0);
        }
    }

    #[test]
    fn pointer_drag_releases_temporary_focus_but_preserves_existing_keyboard_focus() {
        let mut harness = HandleHarness::new();
        let center = harness.rect.center();
        let _ = harness.settle();

        let _ = harness.frame(vec![Event::PointerMoved(center)]);
        let _ = harness.frame(vec![pointer(center, true)]);
        let _ = harness.frame(vec![Event::PointerMoved(center + vec2(10.0, 0.0))]);
        let (response, _) = harness.frame(vec![pointer(center + vec2(10.0, 0.0), false)]);
        assert!(!response.response.has_focus());
        assert!(harness.ctx.memory(|memory| memory.focused()).is_none());

        let mut harness = HandleHarness::new();
        let center = harness.rect.center();
        let response = harness.settle();
        harness
            .ctx
            .memory_mut(|memory| memory.request_focus(response.response.id));
        let _ = harness.frame(vec![]);

        let _ = harness.frame(vec![Event::PointerMoved(center)]);
        let _ = harness.frame(vec![pointer(center, true)]);
        let _ = harness.frame(vec![Event::PointerMoved(center + vec2(10.0, 0.0))]);
        let (response, _) = harness.frame(vec![pointer(center + vec2(10.0, 0.0), false)]);
        assert!(response.response.has_focus());
    }

    #[test]
    fn accesskit_increment_decrement_and_set_value_produce_resize_deltas() {
        let mut harness = HandleHarness::new();
        harness.ctx.enable_accesskit();
        let response = harness.settle();
        let id = response.response.id;

        let (response, _) = harness.frame(vec![accesskit_event(
            id,
            accesskit::Action::Increment,
            None,
        )]);
        assert_eq!(response.delta, 8.0);

        let (response, _) = harness.frame(vec![accesskit_event(
            id,
            accesskit::Action::Decrement,
            None,
        )]);
        assert_eq!(response.delta, -8.0);

        let (response, _) = harness.frame(vec![accesskit_event(
            id,
            accesskit::Action::SetValue,
            Some(accesskit::ActionData::NumericValue(250.0)),
        )]);
        assert_eq!(response.delta, 150.0);

        harness.reversed = true;
        let (response, _) = harness.frame(vec![accesskit_event(
            id,
            accesskit::Action::SetValue,
            Some(accesskit::ActionData::NumericValue(250.0)),
        )]);
        assert_eq!(response.delta, -150.0);
    }

    #[test]
    fn disabled_and_invalid_handles_never_resize() {
        let mut harness = HandleHarness::new();
        let response = harness.settle();
        harness
            .ctx
            .memory_mut(|memory| memory.request_focus(response.response.id));
        let _ = harness.frame(vec![]);
        harness.enabled = false;

        let (response, _) = harness.frame(vec![key(Key::ArrowRight)]);
        assert_eq!(response.delta, 0.0);
        assert!(!response.response.enabled());

        harness.enabled = true;
        harness.rect.min.x = f32::NAN;
        let (response, _) = harness.frame(vec![key(Key::ArrowRight)]);
        assert_eq!(response.delta, 0.0);
        assert!(!response.response.enabled());
    }

    #[test]
    fn invalid_accessibility_ranges_make_the_handle_inert() {
        let ctx = Context::default();
        let id = Id::new("invalid-range");
        let rect = Rect::from_min_size(pos2(100.0, 20.0), vec2(4.0, 140.0));
        let mut delta = None;

        let _ = ctx.run(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 200.0))),
                ..Default::default()
            },
            |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    let response = ResizeHandle::new(
                        id,
                        SplitAxis::Horizontal,
                        rect,
                        "Invalid divider",
                        style(),
                    )
                    .value(400.0, 0.0..=300.0)
                    .show(ui);
                    delta = Some(response.delta);
                });
            },
        );

        assert_eq!(delta, Some(0.0));
    }
}
