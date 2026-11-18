use super::style::*;
use super::*;
use crate::theme;
use crate::theme::{ThemePreset, contrast_ratio};
use egui::{Color32, CornerRadius, Id, Stroke, Ui};
use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2, vec2};

fn visuals(fill: Color32) -> ButtonVisuals {
    ButtonVisuals {
        fill,
        stroke: Stroke::NONE,
        icon: fill,
    }
}

fn style() -> IconButtonStyle {
    IconButtonStyle {
        normal: visuals(Color32::BLACK),
        hovered: visuals(Color32::GREEN),
        pressed: Some(visuals(Color32::BLUE)),
        selected: visuals(Color32::YELLOW),
        disabled: Some(visuals(Color32::GRAY)),
        corner_radius: CornerRadius::same(4),
        focus_stroke: Stroke::NONE,
    }
}

fn frame(ctx: &Context, events: Vec<Event>, add: impl FnMut(&mut Ui)) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 200.0))),
            events,
            ..Default::default()
        },
        add,
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
fn selected_visuals_take_precedence_over_hover_and_press() {
    assert_eq!(style().visuals(true, true, true), visuals(Color32::YELLOW));
}

#[test]
fn pressed_and_hovered_visuals_are_resolved() {
    let style = style();
    assert_eq!(style.visuals(false, true, true), visuals(Color32::BLUE));
    assert_eq!(style.visuals(false, true, false), visuals(Color32::GREEN));
    assert_eq!(style.visuals(false, false, false), visuals(Color32::BLACK));
}

#[test]
fn invalid_strokes_are_ignored_and_large_strokes_are_bounded() {
    let rect = Rect::from_min_size(egui::Pos2::ZERO, Vec2::splat(20.0));
    assert_eq!(
        bounded_stroke(Stroke::new(f32::NAN, Color32::WHITE), rect),
        Stroke::NONE
    );
    assert_eq!(
        bounded_stroke(Stroke::new(f32::MAX, Color32::WHITE), rect).width,
        10.0
    );
}

#[test]
fn control_sizes_have_predictable_order() {
    assert!(ControlSize::Small.height() < ControlSize::Medium.height());
    assert!(ControlSize::Medium.height() < ControlSize::Large.height());
    assert!(ControlSize::Small.font_size() < ControlSize::Large.font_size());
}

#[test]
fn all_variants_and_states_keep_foreground_contrast() {
    for preset in ThemePreset::ALL {
        let tokens = preset.tokens();
        for variant in [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Outline,
            ButtonVariant::Ghost,
            ButtonVariant::Destructive,
        ] {
            for state in [
                InteractionState::normal(ControlSize::Medium),
                InteractionState::hovered(ControlSize::Medium),
                InteractionState::pressed(ControlSize::Medium),
                InteractionState::selected(ControlSize::Medium),
            ] {
                let visuals = variant_visuals(tokens, variant, state);
                if visuals.fill != Color32::TRANSPARENT {
                    let ratio = contrast_ratio(visuals.icon, visuals.fill);
                    assert!(
                        ratio >= 4.5,
                        "{preset:?} {variant:?} {state:?} has {ratio:.2}:1 text contrast"
                    );
                }
            }
            let disabled = variant_visuals(
                tokens,
                variant,
                InteractionState::disabled(ControlSize::Medium),
            );
            assert!(
                contrast_ratio(disabled.icon, disabled.fill) >= 3.0,
                "{preset:?} {variant:?} disabled text must remain legible"
            );
        }
    }
}

#[test]
fn disabled_text_button_cannot_be_clicked() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
    let mut rect = Rect::NOTHING;
    let output = frame(&ctx, vec![], |ui| {
        let response = Button::new("Disabled").enabled(false).show(ui);
        rect = response.rect;
        assert!(!response.enabled());
    });
    assert!(!output.shapes.is_empty());
    let _ = frame(&ctx, click_at(rect.center()), |ui| {
        let response = Button::new("Disabled").enabled(false).show(ui);
        assert!(!response.clicked());
        assert!(!response.enabled());
    });
}

#[test]
fn pointer_click_activates_enabled_button_once() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let mut rect = Rect::NOTHING;
    let _ = frame(&ctx, vec![], |ui| {
        rect = Button::new("Save").show(ui).rect;
    });
    let mut clicks = 0;
    let _ = frame(&ctx, click_at(rect.center()), |ui| {
        if Button::new("Save").show(ui).clicked() {
            clicks += 1;
        }
    });
    assert_eq!(clicks, 1);
}

#[test]
fn keyboard_space_and_enter_activate_a_focused_button() {
    for key in [egui::Key::Space, egui::Key::Enter] {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
        let id = Id::unique("keyboard-button");
        let mut rect = Rect::NOTHING;
        let _ = frame(&ctx, vec![], |ui| {
            rect = Button::new("Activate").id(id).show(ui).rect;
            ui.memory_mut(|memory| memory.request_focus(id));
        });
        let mut clicks = 0;
        let _ = frame(
            &ctx,
            vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            |ui| {
                if Button::new("Activate").id(id).show(ui).clicked() {
                    clicks += 1;
                }
            },
        );
        assert_eq!(clicks, 1, "{key:?} should activate the focused button");
        let _ = rect;
    }
}

#[test]
fn icon_button_reports_selected_state_and_disabled_state() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let style = IconButtonStyle::themed(
        ThemePreset::Light.tokens(),
        ButtonVariant::Outline,
        ControlSize::Medium,
    );
    let mut output = frame(&ctx, vec![], |ui| {
        let response = IconButton::new("Toggle panel", vec2(32.0, 32.0), &style)
            .selected(true)
            .show(ui, |_, _, _| {});
        assert!(response.enabled());
        let disabled = IconButton::new("Unavailable", vec2(32.0, 32.0), &style)
            .enabled(false)
            .show(ui, |_, _, _| {});
        assert!(!disabled.enabled());
        assert!(!disabled.clicked());
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn themed_icon_style_matches_text_button_variant() {
    for preset in ThemePreset::ALL {
        let tokens = preset.tokens();
        let style = IconButtonStyle::themed(tokens, ButtonVariant::Destructive, ControlSize::Large);
        assert_eq!(style.normal.fill, tokens.error);
        assert_eq!(style.normal.icon, tokens.on_error);
        assert_eq!(
            style.disabled.expect("disabled style").fill,
            tokens.disabled
        );
        assert_eq!(style.corner_radius, ControlSize::Large.corner_radius());
    }
}
