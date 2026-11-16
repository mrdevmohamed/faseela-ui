//! Display helpers for bidirectional text in egui.
//!
//! Keep strings in logical Unicode order. This helper reorders directional runs only;
//! egui's text shaper must receive the characters inside each run in logical order.

use unicode_bidi::{BidiClass, BidiInfo, Level, bidi_class};
use unicode_segmentation::UnicodeSegmentation;

/// Reorder bidirectional runs for egui while preserving the logical order inside each run.
///
/// egui shapes Arabic runs itself. Reversing Arabic characters or converting them to
/// presentation forms here would double-process the run and can make letters appear reversed
/// or disconnected. This helper only moves complete directional runs into visual order.
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

    let base_level = logical_line
        .chars()
        .map(bidi_class)
        .find_map(|class| match class {
            BidiClass::R | BidiClass::AL => Some(Level::rtl()),
            BidiClass::L => Some(Level::ltr()),
            _ => None,
        });
    let bidi = BidiInfo::new(logical_line, base_level);
    let clusters: Vec<(usize, &str)> = logical_line.grapheme_indices(true).collect();

    // Build directional runs from grapheme clusters. Keep every run's source text intact:
    // the renderer performs Arabic shaping and glyph ordering inside RTL runs.
    let mut runs: Vec<(usize, usize, Level)> = Vec::new();
    for (byte_index, cluster) in clusters {
        let end = byte_index + cluster.len();
        let level = bidi.levels[byte_index];
        if let Some((_, run_end, run_level)) = runs.last_mut()
            && *run_level == level
        {
            *run_end = end;
        } else {
            runs.push((byte_index, end, level));
        }
    }

    let run_levels: Vec<Level> = runs.iter().map(|(_, _, level)| *level).collect();
    let visual_order = BidiInfo::reorder_visual(&run_levels);

    let mut output = String::with_capacity(logical_line.len());
    for visual_index in visual_order {
        let (start, end, _) = runs[visual_index];
        output.push_str(&logical_line[start..end]);
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
    fn arabic_run_stays_in_logical_order_for_egui_shaping() {
        let original = "السلام عليكم";
        assert_eq!(display_text(original), original);
    }

    #[test]
    fn mixed_arabic_and_latin_reorders_runs_without_reversing_arabic() {
        let original = "مرحبًا بكم في Faseela UI 1.0";
        let visual = display_text(original);
        assert!(visual.starts_with("Faseela"), "{visual:?}");
        assert!(visual.contains("مرحبًا بكم في"), "{visual:?}");
        assert!(visual.contains("UI 1.0"), "{visual:?}");
    }

    #[test]
    fn paragraph_base_direction_comes_from_first_strong_character() {
        let arabic_first = display_text("العربية Faseela 123");
        let latin_first = display_text("Faseela العربية 123");
        assert!(arabic_first.starts_with("Faseela"), "{arabic_first:?}");
        assert!(latin_first.starts_with("Faseela"), "{latin_first:?}");
        assert!(arabic_first.contains("العربية"), "{arabic_first:?}");
        assert!(latin_first.contains("123"), "{latin_first:?}");
        assert_ne!(arabic_first, latin_first);
    }

    #[test]
    fn combining_marks_stay_with_their_arabic_graphemes() {
        let original = "مُحَمَّد";
        assert_eq!(display_text(original), original);
    }

    #[test]
    fn line_breaks_are_preserved_and_processed_independently() {
        let visual = display_text("السلام\nFaseela");
        assert_eq!(visual.lines().count(), 2);
        assert!(visual.ends_with("Faseela"));
    }
}
