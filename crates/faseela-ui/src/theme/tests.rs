use super::*;
use egui::Color32;

#[test]
fn applies_faseela_palette() {
    let ctx = Context::default();
    apply_dark(&ctx);
    let visuals = ctx.style_of(Theme::Dark).visuals.clone();
    assert_eq!(visuals.panel_fill, palette::BACKGROUND);
    assert_eq!(visuals.selection.bg_fill, palette::PRIMARY);
    assert_eq!(
        visuals.widgets.active.fg_stroke.color,
        ThemePreset::FaseelaDark.tokens().on_primary
    );
    assert_eq!(visuals.hyperlink_color, palette::PRIMARY_LIGHT);
}

#[test]
fn light_preset_uses_light_visuals_and_background() {
    let ctx = Context::default();
    apply_preset(&ctx, ThemePreset::Light);
    let visuals = ctx.style_of(Theme::Light).visuals.clone();
    assert_eq!(visuals.panel_fill, Color32::from_rgb(0xF4, 0xF7, 0xF5));
    assert_eq!(visuals.widgets.noninteractive.bg_fill, Color32::WHITE);
    assert_eq!(
        visuals.widgets.inactive.fg_stroke.color,
        Color32::from_rgb(0x1D, 0x2B, 0x23)
    );
    assert_eq!(
        visuals.widgets.hovered.bg_fill,
        ThemePreset::Light.tokens().hover
    );
}

#[test]
fn every_preset_sets_widget_foregrounds_and_borders() {
    let ctx = Context::default();

    for preset in ThemePreset::ALL {
        apply_preset(&ctx, preset);
        let egui_theme = if preset == ThemePreset::Light {
            Theme::Light
        } else {
            Theme::Dark
        };
        let visuals = ctx.style_of(egui_theme).visuals.clone();
        let tokens = preset.tokens();

        assert_eq!(visuals.widgets.inactive.fg_stroke.color, tokens.text);
        assert_eq!(
            visuals.widgets.noninteractive.fg_stroke.color,
            tokens.muted_text
        );
        assert_eq!(visuals.widgets.active.fg_stroke.color, tokens.on_primary);
        assert_eq!(visuals.widgets.hovered.bg_fill, tokens.hover);
        assert_eq!(visuals.selection.bg_fill, tokens.selection);
        assert_eq!(visuals.error_fg_color, tokens.error);
        assert_eq!(visuals.warn_fg_color, tokens.warning);
        assert_eq!(
            visuals.widgets.inactive.bg_stroke.color,
            tokens.control_border
        );
        assert!(visuals.window_stroke.width > 0.0);
    }
}

#[test]
fn all_tokens_are_opaque() {
    for preset in ThemePreset::ALL {
        let tokens = preset.tokens();
        for color in [
            tokens.background,
            tokens.surface,
            tokens.elevated_surface,
            tokens.primary,
            tokens.secondary,
            tokens.accent,
            tokens.text,
            tokens.muted_text,
            tokens.border,
            tokens.control_border,
            tokens.selection,
            tokens.hover,
            tokens.active,
            tokens.disabled,
            tokens.disabled_text,
            tokens.focus,
            tokens.error,
            tokens.warning,
            tokens.success,
            tokens.on_primary,
            tokens.on_secondary,
            tokens.on_accent,
            tokens.on_success,
            tokens.on_warning,
            tokens.on_error,
        ] {
            assert_eq!(color.a(), 255, "{preset:?} has a translucent token");
        }
    }
}

#[test]
fn every_preset_meets_wcag_aa_for_body_text_and_foregrounds() {
    for preset in ThemePreset::ALL {
        let tokens = preset.tokens();
        for (name, base) in [
            ("background", tokens.background),
            ("surface", tokens.surface),
            ("elevated surface", tokens.elevated_surface),
        ] {
            for (role, color) in [
                ("text", tokens.text),
                ("muted text", tokens.muted_text),
                ("secondary text", tokens.secondary),
            ] {
                let ratio = contrast_ratio(color, base);
                assert!(
                    ratio >= 4.5,
                    "{preset:?}: {role} on {name} is {ratio:.2}:1, expected at least 4.5:1"
                );
            }

            for (role, color) in [
                ("control border", tokens.control_border),
                ("focus indicator", tokens.focus),
                ("primary indicator", tokens.primary),
                ("selection", tokens.selection),
            ] {
                let ratio = contrast_ratio(color, base);
                assert!(
                    ratio >= 3.0,
                    "{preset:?}: {role} on {name} is {ratio:.2}:1, expected at least 3:1"
                );
            }
        }

        for (role, foreground, background) in [
            ("primary", tokens.on_primary, tokens.primary),
            ("secondary", tokens.on_secondary, tokens.secondary),
            ("accent", tokens.on_accent, tokens.accent),
            ("success", tokens.on_success, tokens.success),
            ("warning", tokens.on_warning, tokens.warning),
            ("error", tokens.on_error, tokens.error),
        ] {
            let ratio = contrast_ratio(foreground, background);
            assert!(
                ratio >= 4.5,
                "{preset:?}: {role} foreground is {ratio:.2}:1, expected at least 4.5:1"
            );
        }

        let disabled = contrast_ratio(tokens.disabled_text, tokens.disabled);
        assert!(
            disabled >= 3.0,
            "{preset:?}: disabled text is {disabled:.2}:1, expected at least 3:1"
        );
    }
}

#[test]
fn semantic_status_colors_are_readable_on_surfaces() {
    for preset in ThemePreset::ALL {
        let tokens = preset.tokens();
        for base in [tokens.background, tokens.surface] {
            for (name, color) in [
                ("error", tokens.error),
                ("warning", tokens.warning),
                ("success", tokens.success),
            ] {
                let ratio = contrast_ratio(color, base);
                assert!(
                    ratio >= 4.5,
                    "{preset:?}: {name} text is {ratio:.2}:1 on its surface"
                );
            }
        }
    }
}

#[test]
fn light_preset_is_a_light_palette_not_a_dark_background() {
    let light = ThemePreset::Light.tokens();
    let dark = ThemePreset::FaseelaDark.tokens();
    assert!(relative_luminance(light.background) > relative_luminance(light.text));
    assert!(relative_luminance(light.surface) > relative_luminance(light.text));
    assert!(relative_luminance(dark.background) < relative_luminance(dark.text));
    assert_ne!(light.primary, dark.primary);
}

#[test]
fn active_tokens_follow_the_last_applied_preset() {
    let ctx = Context::default();
    assert_eq!(active_preset(&ctx), ThemePreset::FaseelaDark);
    assert_eq!(active_tokens(&ctx), ThemePreset::FaseelaDark.tokens());

    for preset in ThemePreset::ALL {
        apply_preset(&ctx, preset);
        assert_eq!(active_preset(&ctx), preset);
        assert_eq!(active_tokens(&ctx), preset.tokens());
    }
}

#[test]
fn switching_theme_updates_the_current_ui_style() {
    let ctx = Context::default();
    let mut dark_fill = None;
    let mut output = ctx.run_ui(Default::default(), |ui| {
        apply_preset(ui.ctx(), ThemePreset::FaseelaDark);
        ui.reset_style();
        dark_fill = Some(ui.visuals().panel_fill);

        apply_preset(ui.ctx(), ThemePreset::Light);
        // `Ui` caches its style at creation; `reset_style` makes the switch immediate.
        assert_eq!(ui.visuals().panel_fill, dark_fill.unwrap());
        ui.reset_style();
        assert_eq!(
            ui.visuals().panel_fill,
            ThemePreset::Light.tokens().background
        );
        assert_ne!(ui.visuals().panel_fill, dark_fill.unwrap());
    });
    output.textures_delta.clear();
}

#[test]
fn wcag_relative_luminance_matches_reference_values() {
    assert!((relative_luminance(Color32::BLACK) - 0.0).abs() < 0.000_1);
    assert!((relative_luminance(Color32::WHITE) - 1.0).abs() < 0.000_1);
    // Reference value for #808080 sRGB.
    assert!((relative_luminance(Color32::from_rgb(0x80, 0x80, 0x80)) - 0.215_86).abs() < 0.001);
    assert!((contrast_ratio(Color32::WHITE, Color32::BLACK) - 21.0).abs() < 0.01);
    assert!((contrast_ratio(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 0.001);
}

#[test]
fn on_color_uses_real_contrast_not_a_weighted_threshold() {
    // The old weighted threshold produced white for this mid-tone green, at only 4.14:1.
    assert_eq!(
        ThemeTokens::on_color(Color32::from_rgb(0x23, 0x7A, 0x50)),
        Color32::WHITE
    );
    assert_eq!(
        ThemeTokens::on_color(Color32::from_rgb(0xF2, 0xC1, 0x4E)),
        Color32::from_rgb(0x1D, 0x2B, 0x23)
    );
    // Official Faseela primary is exactly the case the old heuristic got wrong: white is
    // only 3.20:1, while the dark ink reaches 5.46:1.
    let primary = palette::PRIMARY;
    let chosen = ThemeTokens::on_color(primary);
    assert_eq!(chosen, Color32::from_rgb(0x1D, 0x2B, 0x23));
    assert!(contrast_ratio(chosen, primary) > contrast_ratio(Color32::WHITE, primary));
    assert!(contrast_ratio(chosen, primary) >= 4.5);
}

#[test]
fn blending_is_bounded_and_alpha_preserving() {
    let a = Color32::from_rgb(1, 2, 3);
    let b = Color32::from_rgb(5, 6, 7);
    assert_eq!(blend(a, b, -1.0), a);
    assert_eq!(blend(a, b, 2.0), b);
    assert_eq!(blend(a, b, f32::NAN), a);
    assert_eq!(blend(a, b, 0.5), Color32::from_rgb(3, 4, 5));
}

fn missing_glyphs(font: &'static [u8], characters: impl IntoIterator<Item = char>) -> Vec<char> {
    let face = ttf_parser::Face::parse(font, 0).expect("bundled font must parse");
    characters
        .into_iter()
        .filter(|character| face.glyph_index(*character).is_none())
        .collect()
}

#[test]
fn noto_sans_arabic_covers_core_arabic_ui_characters() {
    let required = ('\u{0621}'..='\u{063A}') // hamza..ghain
        .chain('\u{0641}'..='\u{064A}') // feh..yeh
        .chain('\u{064B}'..='\u{0652}') // harakat
        .chain('\u{0660}'..='\u{0669}') // Arabic-Indic digits
        .chain(['\u{060C}', '\u{061B}', '\u{061F}']); // comma, semicolon, question mark

    assert_eq!(
        missing_glyphs(uzor_fonts::NOTO_SANS_ARABIC, required),
        Vec::<char>::new()
    );
}

#[test]
fn noto_sans_arabic_covers_persian_and_urdu_additions() {
    let required = [
        '\u{067E}', // peh
        '\u{0686}', // tcheh
        '\u{0698}', // jeh
        '\u{06A9}', // keheh
        '\u{06AF}', // gaf
        '\u{06CC}', // farsi yeh
        '\u{06F0}', '\u{06F9}', // extended Arabic-Indic digits
    ];

    assert_eq!(
        missing_glyphs(uzor_fonts::NOTO_SANS_ARABIC, required),
        Vec::<char>::new()
    );
}

#[test]
fn dejavu_fallback_covers_printable_ascii() {
    assert_eq!(
        missing_glyphs(dejavu::sans::regular(), ' '..='~'),
        Vec::<char>::new()
    );
}

#[test]
fn every_preset_has_a_stable_label_and_index() {
    assert_eq!(ThemePreset::from_index(3), ThemePreset::Light);
    assert_eq!(ThemePreset::Light.label(), "Light");
    assert_eq!(ThemePreset::ALL.len(), 4);
    for (index, preset) in ThemePreset::ALL.into_iter().enumerate() {
        assert_eq!(ThemePreset::from_index(index), preset);
    }
}
