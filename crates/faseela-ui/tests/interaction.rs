//! Public-API regressions for widget identity, state, and emitted accessibility metadata.
use faseela_ui::{
    buttons::{Button, ButtonVariant, ControlSize, IconButton, IconButtonStyle},
    direction::{self, Direction},
    egui::{
        self, Context, Event, Id, Key, Modifiers, Pos2, RawInput, Rect, Ui, Vec2, accesskit, vec2,
    },
    forms::{Checkbox, RadioButton, RadioGroup, SearchField, Select, Switch, TextField},
    navigation::{ButtonGroup, SegmentedControl},
    theme::{self, ThemePreset},
};

fn frame(ctx: &Context, events: Vec<Event>, add: impl FnMut(&mut Ui)) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 600.0))),
            focused: true,
            events,
            ..Default::default()
        },
        add,
    );
    output.textures_delta.clear();
    output
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

fn node(output: &egui::FullOutput, id: Id) -> &accesskit::Node {
    &output
        .platform_output
        .accesskit_update
        .as_ref()
        .expect("AccessKit enabled")
        .nodes
        .iter()
        .find(|(node_id, _)| *node_id == id.accesskit_id())
        .expect("widget node")
        .1
}

#[test]
fn repeated_options_have_distinct_ids_and_translation_preserves_them() {
    for rtl in [false, true] {
        let ctx = Context::default();
        direction::apply(&ctx, if rtl { Direction::Rtl } else { Direction::Ltr });
        let mut previous = Vec::new();
        for label in ["Repeated", "متكرر"] {
            let mut ids = Vec::new();
            frame(&ctx, vec![], |ui| {
                let mut value = false;
                ids.push(Checkbox::new(label, &mut value).show(ui).id);
                ids.push(Checkbox::new(label, &mut value).show(ui).id);
                ids.push(Switch::new(label, &mut value).show(ui).id);
                ids.push(Switch::new(label, &mut value).show(ui).id);
                ids.push(RadioButton::new(label, false).show(ui).id);
                ids.push(RadioButton::new(label, false).show(ui).id);
            });
            for (i, id) in ids.iter().enumerate() {
                assert!(!ids[..i].contains(id));
            }
            if !previous.is_empty() {
                assert_eq!(ids, previous);
            }
            previous = ids;
        }
    }
}

#[test]
fn toggles_publish_the_new_state_on_the_activation_frame() {
    for switch in [false, true] {
        let ctx = Context::default();
        ctx.enable_accesskit();
        let id = Id::unique("toggle");
        let mut value = false;
        frame(&ctx, vec![], |ui| {
            if switch {
                Switch::new("Live", &mut value).id(id).show(ui);
            } else {
                Checkbox::new("Live", &mut value).id(id).show(ui);
            }
            ui.memory_mut(|memory| memory.request_focus(id));
        });
        let output = frame(&ctx, vec![key(Key::Space)], |ui| {
            let response = if switch {
                Switch::new("Live", &mut value).id(id).show(ui)
            } else {
                Checkbox::new("Live", &mut value).id(id).show(ui)
            };
            assert!(response.changed());
        });
        assert!(value);
        let metadata = node(&output, id);
        assert_eq!(
            metadata.role(),
            if switch {
                accesskit::Role::Switch
            } else {
                accesskit::Role::CheckBox
            }
        );
        assert_eq!(metadata.toggled(), Some(accesskit::Toggled::True));
    }
}

#[test]
fn helper_and_error_relationships_are_attached_to_controls() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut checked = false;
    let mut switched = false;
    let mut selected = 0;
    let mut value = String::new();
    let mut ids = Vec::new();
    let output = frame(&ctx, vec![], |ui| {
        ids.push(
            Checkbox::new("Terms", &mut checked)
                .error("Accept terms")
                .show(ui)
                .id,
        );
        ids.push(
            Switch::new("Sync", &mut switched)
                .error("Unavailable")
                .show(ui)
                .id,
        );
        ids.push(
            RadioGroup::new(&["A", "B"], &mut selected)
                .error("Choose a plan")
                .show(ui)
                .id,
        );
        ids.push(
            TextField::new(&mut value)
                .label("Name")
                .error("Required")
                .show(ui)
                .id,
        );
        ids.push(
            Select::new("Plan", &["A", "B"], &mut selected)
                .error("Required")
                .show(ui)
                .id,
        );
    });
    for id in ids {
        let metadata = node(&output, id);
        assert_eq!(metadata.invalid(), Some(accesskit::Invalid::True));
        assert_eq!(metadata.described_by().len(), 1);
    }
}

#[test]
fn radio_navigation_consumes_arrows_and_reports_changes_in_both_directions() {
    for rtl in [false, true] {
        let ctx = Context::default();
        direction::apply(&ctx, if rtl { Direction::Rtl } else { Direction::Ltr });
        let id = Id::unique("radio");
        let mut selected = 0;
        frame(&ctx, vec![], |ui| {
            RadioGroup::new(&["A", "B", "C"], &mut selected)
                .id(id)
                .show(ui);
            ui.memory_mut(|memory| memory.request_focus(id.with(("radio-option", 0))));
        });
        let arrow = if rtl { Key::ArrowLeft } else { Key::ArrowRight };
        frame(&ctx, vec![key(arrow)], |ui| {
            let response = RadioGroup::new(&["A", "B", "C"], &mut selected)
                .id(id)
                .show(ui);
            assert!(response.changed());
            assert!(
                !ui.input(|input| input.key_pressed(arrow)),
                "handled key leaked to parent"
            );
        });
        assert_eq!(selected, 1);
        assert_eq!(
            ctx.memory(|memory| memory.focused()),
            Some(id.with(("radio-option", 1)))
        );
    }
}

#[test]
fn disabled_and_empty_radio_groups_do_not_modify_caller_state() {
    let ctx = Context::default();
    let mut selected = usize::MAX;
    frame(&ctx, vec![], |ui| {
        assert!(
            !RadioGroup::new(&["A", "B"], &mut selected)
                .enabled(false)
                .show(ui)
                .changed()
        );
        assert!(!RadioGroup::new(&[], &mut selected).show(ui).changed());
    });
    assert_eq!(selected, usize::MAX);
}

#[test]
fn segmented_control_reports_changes_and_uses_its_visible_accessible_name() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let id = Id::unique("view");
    let mut selected = 0;
    frame(&ctx, vec![], |ui| {
        SegmentedControl::new(&["List", "Grid"], &mut selected)
            .id(id)
            .label("Workspace view")
            .show(ui);
        ui.memory_mut(|memory| memory.request_focus(id.with(("segment", 0))));
    });
    let mut group = None;
    let output = frame(&ctx, vec![key(Key::End)], |ui| {
        let result = SegmentedControl::new(&["List", "Grid"], &mut selected)
            .id(id)
            .label("Workspace view")
            .show(ui);
        assert!(result.response.changed());
        group = Some(result.response.id);
    });
    assert_eq!(selected, 1);
    assert_eq!(
        node(&output, group.unwrap()).label(),
        Some("Workspace view")
    );
}

#[test]
fn disabled_icon_button_keeps_caller_selection_in_accessibility() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let id = Id::unique("bookmark");
    let style = IconButtonStyle::themed(
        ThemePreset::Light.tokens(),
        ButtonVariant::Outline,
        ControlSize::Medium,
    );
    let output = frame(&ctx, vec![], |ui| {
        IconButton::new("Bookmark", Vec2::splat(32.0), &style)
            .id(id)
            .selected(true)
            .enabled(false)
            .show(ui, |_, _, _| {});
    });
    let metadata = node(&output, id);
    assert!(metadata.is_disabled());
    assert_eq!(metadata.toggled(), Some(accesskit::Toggled::True));
}

#[test]
fn button_group_reports_changed_only_when_selection_changes() {
    let ctx = Context::default();
    let id = Id::unique("period");
    let mut selected = 0;
    frame(&ctx, vec![], |ui| {
        ButtonGroup::new(&["Day", "Week"], &mut selected)
            .id(id)
            .show(ui);
        ui.memory_mut(|memory| memory.request_focus(id.with(("group-item", 1))));
    });
    frame(&ctx, vec![key(Key::Enter)], |ui| {
        let result = ButtonGroup::new(&["Day", "Week"], &mut selected)
            .id(id)
            .show(ui);
        assert!(result.response.changed());
        assert_eq!(result.activated, Some(1));
    });
    assert_eq!(selected, 1);
}

#[test]
fn inputs_and_full_width_buttons_respect_tiny_budgets() {
    for width in [0.0, 1.0, 12.0, 24.0, 32.0, 48.0, 64.0, 120.0] {
        let ctx = Context::default();
        theme::apply_preset(&ctx, ThemePreset::Light);
        let mut value = String::new();
        frame(&ctx, vec![], |ui| {
            ui.set_width(width);
            let field = TextField::new(&mut value).width(200.0).show(ui);
            assert!(
                field.rect.width() <= width + 0.1,
                "field width {width}: {:?}",
                field.rect
            );
            let search = SearchField::new(&mut value).show(ui);
            assert!(
                search.rect.width() <= width + 0.1,
                "search width {width}: {:?}",
                search.rect
            );
            let button = Button::new("A long label").full_width().show(ui);
            assert!(
                button.rect.width() <= width + 0.1,
                "button width {width}: {:?}",
                button.rect
            );
        });
    }
}

#[test]
fn text_input_paste_copy_cut_and_ime_keep_logical_unicode() {
    for rtl in [false, true] {
        let ctx = Context::default();
        theme::install_arabic_font(&ctx);
        direction::apply(&ctx, if rtl { Direction::Rtl } else { Direction::Ltr });
        let id = Id::unique("logical-text");
        let logical = "مُرَحَّبًا Faseela ١٢٣ ۴۵۶ שלום C:\\docs\\a.rs 👨‍👩‍👧";
        let mut value = String::new();
        frame(&ctx, vec![], |ui| {
            TextField::new(&mut value)
                .label("Logical text")
                .id(id)
                .show(ui);
            ui.memory_mut(|memory| memory.request_focus(id));
        });
        frame(&ctx, vec![Event::Paste(logical.to_owned())], |ui| {
            assert!(TextField::new(&mut value).id(id).show(ui).changed());
        });
        assert_eq!(value, logical);
        let select_all = Event::Key {
            key: Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                ctrl: true,
                command: true,
                ..Modifiers::NONE
            },
        };
        frame(&ctx, vec![select_all], |ui| {
            TextField::new(&mut value).id(id).show(ui);
        });
        let output = frame(&ctx, vec![Event::Copy], |ui| {
            TextField::new(&mut value).id(id).show(ui);
        });
        assert!(
            output
                .platform_output
                .commands
                .iter()
                .any(|command| matches!(command,
            egui::OutputCommand::CopyText(text) if text == logical))
        );
        frame(&ctx, vec![Event::Cut], |ui| {
            TextField::new(&mut value).id(id).show(ui);
        });
        assert!(value.is_empty());
        frame(
            &ctx,
            vec![Event::Ime(egui::ImeEvent::Commit("فارسی ۱۲۳".to_owned()))],
            |ui| {
                TextField::new(&mut value).id(id).show(ui);
            },
        );
        assert_eq!(value, "فارسی ۱۲۳");
    }
}

#[test]
fn tab_nodes_expose_selected_rather_than_toggled_state() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let id = Id::unique("tab");
    let output = frame(&ctx, vec![], |ui| {
        faseela_ui::tabs::Tab::new(id, "Overview", true).show_at(
            ui,
            Rect::from_min_size(Pos2::ZERO, vec2(100.0, 32.0)),
            |_, _| {},
        );
    });
    let metadata = node(&output, id);
    assert_eq!(metadata.role(), accesskit::Role::Tab);
    assert_eq!(metadata.is_selected(), Some(true));
    assert_eq!(metadata.toggled(), None);
}

#[test]
fn selecting_a_popup_option_reports_change_on_the_select_response() {
    let ctx = Context::default();
    let id = Id::unique("plan");
    let mut selected = 0;
    let mut trigger = None;
    frame(&ctx, vec![], |ui| {
        let response = Select::new("Plan", &["Starter", "Team"], &mut selected)
            .id(id)
            .show(ui);
        trigger = Some(response.id);
        ui.memory_mut(|memory| memory.request_focus(response.id));
    });
    frame(&ctx, vec![key(Key::Space)], |ui| {
        Select::new("Plan", &["Starter", "Team"], &mut selected)
            .id(id)
            .show(ui);
    });
    // egui exposes native selectable options through the AccessKit tree.
    ctx.enable_accesskit();
    let output = frame(&ctx, vec![], |ui| {
        Select::new("Plan", &["Starter", "Team"], &mut selected)
            .id(id)
            .show(ui);
    });
    let update = output.platform_output.accesskit_update.as_ref().unwrap();
    let (_, option) = update
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Team"))
        .expect("Team option");
    let bounds = option.bounds().expect("option bounds");
    let pos = egui::pos2(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    let events = vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ];
    let mut changed = false;
    frame(&ctx, events, |ui| {
        changed = Select::new("Plan", &["Starter", "Team"], &mut selected)
            .id(id)
            .show(ui)
            .changed();
    });
    assert_eq!(selected, 1);
    assert!(changed);
    assert!(!egui::Popup::is_id_open(
        &ctx,
        trigger.unwrap().with("popup")
    ));
}
