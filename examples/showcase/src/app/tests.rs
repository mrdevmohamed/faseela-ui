use super::helpers::*;
use super::*;
use eframe::egui::{Color32, Id, Rect, Vec2, vec2};
use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput};

const VIEWPORTS: [(f32, f32); 7] = [
    (320.0, 568.0),
    (360.0, 640.0),
    (390.0, 844.0),
    (680.0, 800.0),
    (768.0, 1024.0),
    (1280.0, 800.0),
    (1920.0, 1080.0),
];

fn frame(
    ctx: &egui::Context,
    size: Vec2,
    app: &mut Showcase,
    events: Vec<Event>,
) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            focused: true,
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}

fn setup(preset: ThemePreset, rtl: bool) -> egui::Context {
    let ctx = egui::Context::default();
    theme::apply_preset(&ctx, preset);
    direction::apply(&ctx, if rtl { Direction::Rtl } else { Direction::Ltr });
    ctx
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

fn response_rect(ctx: &egui::Context, id: Id) -> Rect {
    ctx.read_response(id)
        .unwrap_or_else(|| panic!("missing response for {id:?}"))
        .rect
}

#[test]
fn every_section_renders_without_horizontal_overflow_at_target_viewports() {
    for (width, height) in VIEWPORTS {
        for preset in ThemePreset::ALL {
            for rtl in [false, true] {
                let ctx = setup(preset, rtl);
                for section in 0..SECTIONS.len() {
                    let mut app = Showcase {
                        selected_section: section,
                        selected_theme: preset,
                        rtl,
                        ..Default::default()
                    };
                    let size = vec2(width, height);
                    let output = frame(&ctx, size, &mut app, vec![]);
                    assert!(
                        !output.shapes.is_empty(),
                        "{width}x{height} {preset:?} rtl={rtl} section {section} painted nothing"
                    );
                    let used = ctx.globally_used_rect();
                    assert!(
                        used.width() <= width + 1.0,
                        "{width}x{height} {preset:?} rtl={rtl} section {section} used width {}",
                        used.width()
                    );
                    assert!(
                        used.right() <= width + 1.0,
                        "{width}x{height} {preset:?} rtl={rtl} section {section} right edge {}",
                        used.right()
                    );
                    assert!(
                        used.left() >= -1.0,
                        "{width}x{height} {preset:?} rtl={rtl} section {section} left edge {}",
                        used.left()
                    );
                }
            }
        }
    }
}

#[test]
fn every_section_renders_with_bundled_fonts_in_all_themes_and_directions() {
    for preset in ThemePreset::ALL {
        for rtl in [false, true] {
            for section in 0..SECTIONS.len() {
                let ctx = setup(preset, rtl);
                theme::install_arabic_font(&ctx);
                let mut app = Showcase {
                    rtl,
                    selected_theme: preset,
                    selected_section: section,
                    ..Default::default()
                };
                let output = frame(&ctx, vec2(390.0, 844.0), &mut app, vec![]);
                assert!(
                    !output.shapes.is_empty(),
                    "{preset:?} rtl={rtl} section {section} painted nothing"
                );
                assert!(
                    ctx.globally_used_rect().width() <= 391.0,
                    "{preset:?} rtl={rtl} section {section} overflowed"
                );
            }
        }
    }
}

#[test]
fn every_enabled_size_and_icon_demo_has_an_action() {
    let ctx = setup(ThemePreset::Light, false);
    let mut app = Showcase {
        selected_section: 2,
        selected_theme: ThemePreset::Light,
        ..Default::default()
    };
    let size = vec2(1280.0, 800.0);
    for (id, expected) in [
        (Id::unique("showcase-size").with(0), "Small"),
        (Id::unique("showcase-size").with(1), "Medium"),
        (Id::unique("showcase-size").with(2), "Large"),
        (Id::unique("showcase-with-icon"), "Leading-icon"),
    ] {
        frame(&ctx, size, &mut app, vec![]);
        let rect = response_rect(&ctx, id);
        frame(&ctx, size, &mut app, click_at(rect.center()));
        assert!(app.last_action.contains(expected), "{}", app.last_action);
    }
}

#[test]
fn mobile_escape_closes_navigation_and_restores_trigger_focus() {
    let ctx = setup(ThemePreset::Light, false);
    let mut app = Showcase {
        mobile_nav_open: true,
        ..Default::default()
    };
    let size = vec2(320.0, 568.0);
    frame(&ctx, size, &mut app, vec![]);
    frame(
        &ctx,
        size,
        &mut app,
        vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert!(!app.mobile_nav_open);
    assert_eq!(
        ctx.memory(|memory| memory.focused()),
        Some(Id::unique("showcase-mobile-menu"))
    );
}

#[test]
fn desktop_sidebar_and_tab_order_follow_reading_direction() {
    for rtl in [false, true] {
        let ctx = setup(ThemePreset::Light, rtl);
        let mut app = Showcase {
            rtl,
            selected_section: 4,
            ..Default::default()
        };
        frame(&ctx, vec2(1280.0, 800.0), &mut app, vec![]);
        let sidebar = response_rect(&ctx, Id::unique("showcase-nav").with(4));
        let first = response_rect(&ctx, Id::unique("showcase-tabs").with(0));
        let second = response_rect(&ctx, Id::unique("showcase-tabs").with(1));
        if rtl {
            assert!(sidebar.center().x > first.center().x);
            assert!(first.center().x > second.center().x);
        } else {
            assert!(sidebar.center().x < first.center().x);
            assert!(first.center().x < second.center().x);
        }
    }
}

#[test]
fn desktop_navigation_clicks_change_the_section() {
    let ctx = setup(ThemePreset::FaseelaDark, false);
    let mut app = Showcase::default();
    let size = vec2(1280.0, 800.0);
    let _ = frame(&ctx, size, &mut app, vec![]);
    let rect = response_rect(&ctx, Id::unique("showcase-nav").with(4));
    let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
    assert_eq!(app.selected_section, 4);
}

#[test]
fn mobile_menu_opens_and_navigates_to_a_section() {
    let ctx = setup(ThemePreset::Light, false);
    let mut app = Showcase::default();
    let size = vec2(360.0, 640.0);
    let _ = frame(&ctx, size, &mut app, vec![]);
    assert!(!app.mobile_nav_open);
    let menu_rect = response_rect(&ctx, Id::unique("showcase-mobile-menu"));
    let _ = frame(&ctx, size, &mut app, click_at(menu_rect.center()));
    assert!(app.mobile_nav_open);

    let _ = frame(&ctx, size, &mut app, vec![]);
    let item_rect = response_rect(&ctx, Id::unique("showcase-mobile-nav").with(6));
    let _ = frame(&ctx, size, &mut app, click_at(item_rect.center()));
    assert_eq!(app.selected_section, 6);
    assert!(!app.mobile_nav_open);
}

#[test]
fn theme_buttons_apply_the_preset_on_the_following_frame() {
    let ctx = setup(ThemePreset::FaseelaDark, false);
    let mut app = Showcase {
        selected_section: 7,
        ..Default::default()
    };
    let size = vec2(1280.0, 800.0);
    let _ = frame(&ctx, size, &mut app, vec![]);
    let rect = response_rect(&ctx, Id::unique("showcase-theme").with(2));
    let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
    assert_eq!(app.selected_theme, ThemePreset::Forest);
    let _ = frame(&ctx, size, &mut app, vec![]);
    assert_eq!(theme::active_preset(&ctx), ThemePreset::Forest);
}

#[test]
fn rtl_control_switches_direction_and_persists() {
    let ctx = setup(ThemePreset::FaseelaDark, false);
    let mut app = Showcase {
        selected_section: 9,
        ..Default::default()
    };
    let size = vec2(390.0, 844.0);
    let _ = frame(&ctx, size, &mut app, vec![]);
    let rect = response_rect(&ctx, Id::unique("showcase-rtl").with(("group-item", 1)));
    let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
    assert!(app.rtl);
    let _ = frame(&ctx, size, &mut app, vec![]);
    assert!(direction::is_rtl(&ctx));
}

#[test]
fn button_and_form_interactions_update_application_state() {
    let ctx = setup(ThemePreset::FaseelaDark, false);
    let mut app = Showcase {
        selected_section: 2,
        ..Default::default()
    };
    let size = vec2(1280.0, 800.0);
    let _ = frame(&ctx, size, &mut app, vec![]);
    let rect = response_rect(&ctx, Id::unique("showcase-action-primary"));
    let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
    assert!(app.last_action.contains("Primary"));

    app.selected_section = 3;
    let _ = frame(&ctx, size, &mut app, vec![]);
    let check_id = Id::unique("showcase-notify");
    let check_rect = response_rect(&ctx, check_id);
    let _ = frame(&ctx, size, &mut app, click_at(check_rect.center()));
    assert!(app.notifications);

    let _ = frame(&ctx, size, &mut app, vec![]);
    let field_id = Id::unique("showcase-name");
    let _ = frame(&ctx, size, &mut app, vec![]);
    let _ = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            focused: true,
            ..Default::default()
        },
        |ui| {
            app.show(ui);
            ui.memory_mut(|memory| memory.request_focus(field_id));
        },
    );
    let _ = frame(
        &ctx,
        size,
        &mut app,
        vec![Event::Text("Faseela".to_owned())],
    );
    assert_eq!(app.name, "Faseela");
}

#[test]
fn progress_controls_update_the_bar_value() {
    let ctx = setup(ThemePreset::Light, false);
    let mut app = Showcase {
        selected_section: 6,
        ..Default::default()
    };
    let size = vec2(390.0, 844.0);
    let _ = frame(&ctx, size, &mut app, vec![]);
    let before = app.progress;
    let rect = response_rect(&ctx, Id::unique("showcase-progress-advance"));
    let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
    assert!(app.progress > before);
    assert!(app.progress <= 1.0);
}

#[test]
fn contrast_checks_pass_for_every_preset() {
    for preset in ThemePreset::ALL {
        let checks = contrast_checks(preset.tokens());
        for check in checks {
            assert!(
                check.passes(),
                "{preset:?} {} was {:.2}:1 below {}:1",
                check.label,
                check.ratio(),
                check.minimum
            );
        }
    }
}

#[test]
fn colors_are_reported_as_uppercase_hex() {
    assert_eq!(hex(Color32::from_rgb(0x0A, 0x1D, 0x1A)), "#0A1D1A");
}

#[test]
fn keyboard_review_checkbox_is_handled_without_errors() {
    let ctx = setup(ThemePreset::FaseelaDark, false);
    let mut app = Showcase {
        selected_section: 8,
        ..Default::default()
    };
    let size = vec2(390.0, 844.0);
    let _ = frame(&ctx, size, &mut app, vec![]);
    let key = Event::Key {
        key: Key::Tab,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    let _ = frame(&ctx, size, &mut app, vec![key]);
    assert!(ctx.globally_used_rect().width() <= 391.0);
}
