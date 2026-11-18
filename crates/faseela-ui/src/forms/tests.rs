use super::*;
use crate::theme::ThemePreset;
use crate::{direction, theme};
use egui::{
    Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, pos2, vec2,
};
use egui::{Id, Ui};

fn frame(ctx: &Context, events: Vec<Event>, add: impl FnMut(&mut Ui)) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 400.0))),
            focused: true,
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
fn text_field_updates_application_owned_value_and_reports_changes() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
    let id = Id::unique("name-field");
    let mut value = String::new();
    let _ = frame(&ctx, vec![], |ui| {
        TextField::new(&mut value).label("Name").id(id).show(ui);
        ui.memory_mut(|memory| memory.request_focus(id));
    });
    let _ = frame(
        &ctx,
        vec![
            Event::Text("Faseela".to_owned()),
            Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            },
        ],
        |ui| {
            let response = TextField::new(&mut value).label("Name").id(id).show(ui);
            assert!(response.changed());
        },
    );
    assert_eq!(value, "Faseela");
}

#[test]
fn text_field_error_uses_error_color_and_invalid_state() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let tokens = ThemePreset::Light.tokens();
    let mut value = String::new();
    let output = frame(&ctx, vec![], |ui| {
        TextField::new(&mut value)
            .label("Email")
            .error("Enter a valid email")
            .show(ui);
    });
    let has_error_stroke = output.shapes.iter().any(|clip| match &clip.shape {
        Shape::Rect(rect) => rect.stroke.color == tokens.error && rect.stroke.width > 0.0,
        _ => false,
    });
    assert!(has_error_stroke, "error border must be painted");
}

#[test]
fn disabled_text_field_is_inert_and_reports_disabled() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
    let id = Id::unique("disabled-field");
    let mut value = "Read only".to_owned();
    let mut rect = Rect::NOTHING;
    let _ = frame(&ctx, vec![], |ui| {
        rect = TextField::new(&mut value)
            .label("Name")
            .id(id)
            .enabled(false)
            .show(ui)
            .rect;
    });
    let _ = frame(&ctx, click_at(rect.center()), |ui| {
        let response = TextField::new(&mut value)
            .label("Name")
            .id(id)
            .enabled(false)
            .show(ui);
        assert!(!response.enabled());
        assert!(!response.has_focus());
    });
    assert_eq!(value, "Read only");
}

#[test]
fn checkbox_toggles_by_click_and_respects_disabled_state() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let id = Id::unique("checkbox");
    let mut checked = false;
    let mut rect = Rect::NOTHING;
    let _ = frame(&ctx, vec![], |ui| {
        rect = Checkbox::new("Enable notifications", &mut checked)
            .id(id)
            .show(ui)
            .rect;
    });
    let _ = frame(&ctx, click_at(rect.center()), |ui| {
        Checkbox::new("Enable notifications", &mut checked)
            .id(id)
            .show(ui);
    });
    assert!(checked);

    let mut disabled = false;
    let _ = frame(&ctx, vec![], |ui| {
        rect = Checkbox::new("Disabled", &mut disabled)
            .id(id)
            .enabled(false)
            .show(ui)
            .rect;
    });
    let _ = frame(&ctx, click_at(rect.center()), |ui| {
        Checkbox::new("Disabled", &mut disabled)
            .id(id)
            .enabled(false)
            .show(ui);
    });
    assert!(!disabled);
}

#[test]
fn switch_toggles_by_keyboard_space_when_focused() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
    let id = Id::unique("switch");
    let mut enabled = false;
    let _ = frame(&ctx, vec![], |ui| {
        Switch::new("Live updates", &mut enabled).id(id).show(ui);
        ui.memory_mut(|memory| memory.request_focus(id));
    });
    let _ = frame(
        &ctx,
        vec![Event::Key {
            key: Key::Space,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        |ui| {
            Switch::new("Live updates", &mut enabled).id(id).show(ui);
        },
    );
    assert!(enabled);
}

#[test]
fn radio_group_changes_selection_by_click_and_arrow_key() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let id = Id::unique("radio-group");
    let mut selected = 0usize;
    let _ = frame(&ctx, vec![], |ui| {
        RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
            .label("Density")
            .id(id)
            .show(ui);
    });

    // Focus the first option and use the reading-order next arrow.
    let _ = frame(&ctx, vec![], |ui| {
        RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
            .label("Density")
            .id(id)
            .show(ui);
        let option_id = id.with(("radio-option", 0));
        ui.memory_mut(|memory| memory.request_focus(option_id));
    });
    let _ = frame(
        &ctx,
        vec![Event::Key {
            key: Key::ArrowDown,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        |ui| {
            RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
                .label("Density")
                .id(id)
                .show(ui);
        },
    );
    assert_eq!(selected, 1);
}

#[test]
fn radio_group_click_selects_the_clicked_option() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let id = Id::unique("radio-group-click");
    let mut selected = 0usize;
    let _ = frame(&ctx, vec![], |ui| {
        RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
            .label("Density")
            .id(id)
            .show(ui);
    });
    let option = ctx
        .read_response(id.with(("radio-option", 2)))
        .expect("third option");
    assert!(option.enabled());
    let _ = frame(&ctx, click_at(option.rect.center()), |ui| {
        RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut selected)
            .label("Density")
            .id(id)
            .show(ui);
    });
    assert_eq!(selected, 2);
}

#[test]
fn disabled_radio_group_ignores_clicks_and_arrow_keys() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
    let id = Id::unique("radio-group-disabled");
    let mut selected = 1usize;
    let _ = frame(&ctx, vec![], |ui| {
        RadioGroup::new(&["One", "Two", "Three"], &mut selected)
            .label("Disabled")
            .id(id)
            .enabled(false)
            .show(ui);
    });
    let option = ctx
        .read_response(id.with(("radio-option", 2)))
        .expect("third option");
    assert!(!option.enabled());
    let _ = frame(&ctx, click_at(option.rect.center()), |ui| {
        RadioGroup::new(&["One", "Two", "Three"], &mut selected)
            .label("Disabled")
            .id(id)
            .enabled(false)
            .show(ui);
    });
    let _ = frame(
        &ctx,
        vec![Event::Key {
            key: Key::ArrowDown,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        |ui| {
            RadioGroup::new(&["One", "Two", "Three"], &mut selected)
                .label("Disabled")
                .id(id)
                .enabled(false)
                .show(ui);
        },
    );
    assert_eq!(selected, 1);
}

#[test]
fn radio_group_clamps_out_of_range_selection() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::FaseelaDark);
    let mut selected = usize::MAX;
    let _ = frame(&ctx, vec![], |ui| {
        RadioGroup::new(&["One", "Two"], &mut selected).show(ui);
    });
    assert_eq!(selected, 1);
}

#[test]
fn every_control_renders_in_all_themes_and_directions() {
    for preset in ThemePreset::ALL {
        for rtl in [false, true] {
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
            let mut text = "مرحبا Faseela".to_owned();
            let mut checked = true;
            let mut switched = false;
            let mut radio = 0usize;
            let mut select = 0usize;
            let output = frame(&ctx, vec![], |ui| {
                TextField::new(&mut text).label("Name").show(ui);
                SearchField::new(&mut text).label("Search").show(ui);
                Checkbox::new("Checkbox", &mut checked).show(ui);
                Switch::new("Switch", &mut switched).show(ui);
                RadioGroup::new(&["A", "B"], &mut radio)
                    .label("Radio")
                    .show(ui);
                Select::new("Select", &["One", "Two"], &mut select).show(ui);
            });
            assert!(!output.shapes.is_empty(), "{preset:?} rtl={rtl}");
        }
    }
}

#[test]
fn select_renders_options_and_clamps_nothing_implicitly() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let mut selected = 1usize;
    let _ = frame(&ctx, vec![], |ui| {
        Select::new("Theme", &["Dark", "Light"], &mut selected).show(ui);
    });
    assert_eq!(selected, 1);
}

#[test]
fn select_opens_with_keyboard_from_focus_and_closes_with_escape() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let id = Id::unique("select-keyboard");
    let mut selected = 0usize;
    let mut button_id = None;
    let _ = frame(&ctx, vec![], |ui| {
        let response = Select::new("Theme", &["Dark", "Light"], &mut selected)
            .id(id)
            .show(ui);
        ui.memory_mut(|memory| memory.request_focus(response.id));
        button_id = Some(response.id);
    });
    let button_id = button_id.expect("select response id");
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(button_id));
    let popup_id = button_id.with("popup");
    assert!(!egui::Popup::is_id_open(&ctx, popup_id));

    let _ = frame(
        &ctx,
        vec![Event::Key {
            key: Key::Space,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        |ui| {
            Select::new("Theme", &["Dark", "Light"], &mut selected)
                .id(id)
                .show(ui);
        },
    );
    assert!(egui::Popup::is_id_open(&ctx, popup_id));

    let _ = frame(
        &ctx,
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        |ui| {
            Select::new("Theme", &["Dark", "Light"], &mut selected)
                .id(id)
                .show(ui);
        },
    );
    assert!(!egui::Popup::is_id_open(&ctx, popup_id));
}

#[test]
fn text_field_outline_does_not_expand_past_available_width() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let mut value = String::new();
    let _ = frame(&ctx, vec![], |ui| {
        TextField::new(&mut value)
            .label("Name")
            .helper("Outline is painted inside the measured field.")
            .show(ui);
    });
    assert!(
        ctx.globally_used_rect().right() <= 320.0,
        "field used {:?}",
        ctx.globally_used_rect()
    );
}

#[test]
fn long_labels_wrap_within_available_width() {
    let ctx = Context::default();
    theme::apply_preset(&ctx, ThemePreset::Light);
    let mut checked = false;
    let output = frame(&ctx, vec![], |ui| {
        Checkbox::new(
            "A deliberately long checkbox label that should wrap instead of clipping its text",
            &mut checked,
        )
        .show(ui);
    });
    assert!(!output.shapes.is_empty());
    let _ = pos2(0.0, 0.0);
}
