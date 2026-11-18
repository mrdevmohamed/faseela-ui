use super::*;
use super::{badge::BADGE_PADDING, toolbar::layout_items};
use crate::theme::{ThemePreset, contrast_ratio};
use crate::{layout::ToolbarFit, theme};
use egui::{Color32, Id, Layout, Ui, Vec2};
use egui::{
    Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, pos2, vec2,
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
    let (empty, _) = toolbar_in_width(&ctx, vec2(500.0, 300.0), 400.0, Id::unique("empty"), &[]);
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
        (badge_rect.height() - (text.galley.rect.height() + BADGE_PADDING.y * 2.0)).abs() < 0.01
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
