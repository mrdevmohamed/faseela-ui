//! Display helpers for scripts that egui does not shape or reorder automatically.
//!
//! Keep application strings in logical Unicode order. Use display_text only at the
//! presentation boundary; never store, search, edit, or copy its visual-order result.

use arabic::{Direction as ArabicDirection, reshape};
use unicode_bidi::{BidiClass, BidiInfo, Level, bidi_class};
use unicode_segmentation::UnicodeSegmentation;

/// Shape Arabic joining forms and reorder mixed RTL/LTR text for egui's left-to-right renderer.
///
/// This is intended for short UI labels, headings, and button text. Each newline-delimited
/// paragraph is processed independently. Grapheme clusters are reordered as units so Arabic
/// combining marks remain attached to their base letters, and mirrored punctuation is mapped
/// when it appears in an RTL run.
pub fn display_text(logical_text: &str) -> String {
    let mut visual_text = String::with_capacity(logical_text.len());

    for (line_index, line) in logical_text.split('\n').enumerate() {
        if line_index > 0 {
            visual_text.push('\n');
        }

        let (line, had_carriage_return) = match line.strip_suffix('\r') {
            Some(line) => (line, true),
            None => (line, false),
        };

        visual_text.push_str(&display_line(line));
        if had_carriage_return {
            visual_text.push('\r');
        }
    }

    visual_text
}

fn display_line(logical_line: &str) -> String {
    if logical_line.is_empty() {
        return String::new();
    }

    // Shape first while characters are still in logical reading order.
    let shaped = reshape(logical_line, ArabicDirection::Logical);
    // egui lays glyphs out left-to-right. Resolve the paragraph base direction from
    // the first strong character instead of always relying on the default LTR base.
    let base_level = logical_line
        .chars()
        .map(bidi_class)
        .find_map(|class| match class {
            BidiClass::R | BidiClass::AL => Some(Level::rtl()),
            BidiClass::L => Some(Level::ltr()),
            _ => None,
        });
    let bidi = BidiInfo::new(&shaped, base_level);
    let clusters: Vec<(usize, &str)> = shaped.grapheme_indices(true).collect();

    // egui consumes text left-to-right. Reorder complete grapheme clusters rather than UTF-8
    // bytes or individual code points, keeping marks and join-control sequences intact.
    let cluster_levels: Vec<_> = clusters
        .iter()
        .map(|(byte_index, _)| bidi.levels[*byte_index])
        .collect();
    let visual_order = BidiInfo::reorder_visual(&cluster_levels);

    let mut output = String::with_capacity(shaped.len());
    for visual_index in visual_order {
        let (byte_index, cluster) = clusters[visual_index];
        if bidi.levels[byte_index].is_rtl() {
            for character in cluster.chars() {
                output.push(unicode_bidi_mirroring::get_mirrored(character).unwrap_or(character));
            }
        } else {
            output.push_str(cluster);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_text_is_unchanged() {
        let original = "Faseela UI 1.0";
        assert_eq!(display_text(original), original);
    }

    #[test]
    fn arabic_text_is_shaped_and_reordered() {
        let original = "السلام عليكم";
        let visual = display_text(original);
        assert_ne!(visual, original);
        assert!(visual.chars().any(|character| {
            matches!(character, '\u{FB50}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}')
        }));
    }

    #[test]
    fn mixed_arabic_and_latin_keep_latin_runs_readable() {
        let visual = display_text("مرحبًا بكم في Faseela UI 1.0");
        assert!(visual.contains("Faseela"));
        assert!(visual.contains("UI"));
        assert!(visual.contains("1.0"));
        assert!(
            visual.starts_with("Faseela"),
            "LTR run should remain readable in visual order: {visual:?}"
        );
    }

    #[test]
    fn paragraph_base_direction_comes_from_first_strong_character() {
        let arabic_first = display_text("العربية Faseela 123");
        let latin_first = display_text("Faseela العربية 123");
        // In visual left-to-right order, the LTR run is placed before the
        // Arabic run in an RTL paragraph. A paragraph beginning with Latin
        // keeps its trailing numeric run at the visual end.
        assert!(arabic_first.starts_with("Faseela"), "{arabic_first:?}");
        assert!(latin_first.starts_with("Faseela"), "{latin_first:?}");
        assert!(arabic_first.ends_with("ﺔﻴﺑﺮﻌﻟﺍ"), "{arabic_first:?}");
        assert!(latin_first.ends_with("123"), "{latin_first:?}");
        assert_ne!(arabic_first, latin_first);
    }

    #[test]
    fn combining_marks_stay_with_their_arabic_graphemes() {
        let visual = display_text("مُحَمَّد");
        assert!(visual.contains('\u{064F}'));
        assert!(visual.contains('\u{064E}'));
        assert!(visual.contains('\u{0651}'));
    }

    #[test]
    fn line_breaks_are_preserved_and_processed_independently() {
        let visual = display_text("السلام\nFaseela");
        assert_eq!(visual.lines().count(), 2);
        assert!(visual.ends_with("Faseela"));
    }
}
