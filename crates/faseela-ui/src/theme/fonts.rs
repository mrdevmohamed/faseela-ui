//! Embedded Unicode fallback installation, independent of color theme.
use egui::{Context, FontData, FontFamily, FontInsert, FontPriority, Id, InsertFontFamily};

/// Install bundled Arabic and broad-Unicode fallback fonts for UI text.
///
/// Preserves application-installed fonts and is safe to call repeatedly. Existing fonts take
/// precedence, followed by Noto Sans Arabic and then DejaVu Sans in both built-in families.
pub fn install_arabic_font(ctx: &Context) {
    // egui deduplicates installed fonts by name, but not insertions queued before the next pass.
    let pass = ctx.cumulative_pass_nr();
    let already_queued = ctx.data_mut(|data| {
        let id = Id::unique("faseela_ui::theme::font-install-pass");
        let previous = data.get_temp::<u64>(id);
        data.insert_temp(id, pass);
        previous == Some(pass)
    });
    if already_queued {
        return;
    }
    for (name, bytes) in [
        ("faseela-noto-sans-arabic", uzor_fonts::NOTO_SANS_ARABIC),
        ("faseela-arabic-fallback", dejavu::sans::regular()),
    ] {
        ctx.add_font(FontInsert::new(
            name,
            FontData::from_static(bytes),
            [FontFamily::Proportional, FontFamily::Monospace]
                .map(|family| InsertFontFamily {
                    family,
                    priority: FontPriority::Lowest,
                })
                .to_vec(),
        ));
    }
}
