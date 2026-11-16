//! Display helpers for scripts that egui does not shape or reorder automatically.
//!
//! Keep application strings in logical Unicode order. Use display_text only at the
//! presentation boundary; never store, search, edit, or copy its visual-order result.

use arabic::{Direction as ArabicDirection, reshape};
use unicode_bidi::BidiInfo;
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
    let bidi = BidiInfo::new(&shaped, None);
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
                output.push(
                    unicode_bidi_mirroring::get_mirrored(character).unwrap_or(character),
                );
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
        let expected = reshape(original, ArabicDirection::RightToLeft);
        assert_eq!(display_text(original), expected);
        assert_ne!(display_text(original), original);
    }

    #[test]
    fn mixed_arabic_and_latin_keep_latin_runs_readable() {
        let visual = display_text("مرحبًا بكم في Faseela UI 1.0");
        assert!(visual.contains("Faseela"));
        assert!(visual.contains("UI"));
        assert!(visual.contains("1.0"));
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
