use super::*;
use crate::theme::ThemePreset;
use crate::{direction, theme};
use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect};
use egui::{Id, Ui, vec2};

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
fn navigation_rows_stay_compact_in_tall_containers_and_wrap_when_needed() {
    for rtl in [false, true] {
        let ctx = setup(ThemePreset::Light, rtl);
        let mut selected = 0;
        frame(&ctx, vec![], |ui| {
            let group = ButtonGroup::new(&["Columns", "Rows"], &mut selected).show(ui);
            assert!(
                group.response.rect.height() <= 42.0,
                "{:?}",
                group.response.rect
            );
            let breadcrumb = Breadcrumb::new(&["Home", "Library", "Current"]).show(ui);
            assert!(
                breadcrumb.response.rect.height() <= 42.0,
                "{:?}",
                breadcrumb.response.rect
            );
        });
        frame(&ctx, vec![], |ui| {
            ui.set_width(80.0);
            let group = ButtonGroup::new(&["Columns", "Rows", "Panes"], &mut selected).show(ui);
            assert!(group.response.rect.height() > 32.0, "must wrap");
            assert!(group.response.rect.width() <= 80.1);
        });
    }
}
