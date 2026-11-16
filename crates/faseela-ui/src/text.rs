//! Display helpers for bidirectional text in egui.
//!
//! Keep strings in logical Unicode order. This helper reorders word-boundary units only;
//! egui's HarfRust shaper must receive Arabic letters inside each word in logical order.

use unicode_bidi::{BidiClass, BidiInfo, Level, bidi_class};
use unicode_segmentation::UnicodeSegmentation;

/// Reorder bidirectional words for egui while preserving the letters inside each word.
///
/// egui shapes Arabic words itself, but does not yet reorder words in a bidirectional
/// paragraph. Reversing individual Arabic characters or converting them to presentation
/// forms here would double-process shaping and can make letters appear reversed or disconnected.
/// This helper is a display-only workaround for short labels; do not use it for editable text.
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
    let paragraph_level = base_level.unwrap_or_else(Level::ltr);
    let bidi = BidiInfo::new(logical_line, Some(paragraph_level));

    // egui shapes each Arabic word right-to-left, but lays words out in logical order.
    // Reorder word-boundary units while leaving every word's characters untouched.
    let words: Vec<(usize, &str)> = logical_line.split_word_bound_indices().collect();
    let mut word_levels: Vec<Level> = words
        .iter()
        .map(|(byte_index, word)| {
            let level_index = byte_index
                + word
                    .char_indices()
                    .find(|(_, character)| !character.is_whitespace())
                    .map_or(0, |(index, _)| index);
            bidi.levels[level_index]
        })
        .collect();

    // Keep integer-only tokens at the paragraph boundary in RTL paragraphs. Decimal
    // numbers remain attached to their preceding Latin run (for example, "UI 1.0").
    // egui receives the words in visual order but still shapes each word itself.
    for (index, (_, word)) in words.iter().enumerate() {
        if word.chars().all(char::is_whitespace) {
            let next_is_integer = words.get(index + 1).is_some_and(|(_, next)| {
                !next.is_empty() && next.chars().all(|character| character.is_ascii_digit())
            });

            if next_is_integer {
                word_levels[index] = paragraph_level;
            }
        }
    }

    let visual_order = BidiInfo::reorder_visual(&word_levels);

    let mut output = String::with_capacity(logical_line.len());
    for visual_index in visual_order {
        let word = words[visual_index].1;
        if word_levels[visual_index].is_rtl() {
            // egui draws every glyph left-to-right, so paired punctuation inside an RTL
            // run must be swapped here (Unicode Bidi_Mirrored), e.g. "(" becomes ")".
            output.extend(word.chars().map(|character| {
                unicode_bidi_mirroring::get_mirrored(character).unwrap_or(character)
            }));
        } else {
            output.push_str(word);
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
    fn arabic_words_reorder_but_letters_stay_logical_for_egui_shaping() {
        assert_eq!(display_text("السلام عليكم"), "عليكم السلام");
    }

    #[test]
    fn mixed_arabic_and_latin_reorders_words_without_reversing_letters() {
        let visual = display_text("مرحبًا بكم في Faseela UI 1.0");
        assert!(visual.starts_with("Faseela"), "{visual:?}");
        assert!(visual.contains("في بكم مرحبًا"), "{visual:?}");
        assert!(visual.contains("UI 1.0"), "{visual:?}");
    }

    #[test]
    fn paragraph_base_direction_comes_from_first_strong_character() {
        let arabic_first = display_text("العربية Faseela 123");
        let latin_first = display_text("Faseela العربية 123");
        assert_eq!(arabic_first, "123 Faseela العربية");
        assert_eq!(latin_first, "Faseela العربية 123");
    }

    #[test]
    fn combining_marks_stay_with_their_arabic_graphemes() {
        let original = "مُحَمَّد";
        assert_eq!(display_text(original), original);
    }

    #[test]
    fn brackets_in_rtl_runs_are_mirrored_for_left_to_right_drawing() {
        assert_eq!(display_text("الإصدار (2.0) متاح"), "متاح (2.0) الإصدار");
        assert_eq!(display_text("(مرحبا) world"), "world (مرحبا)");
    }

    #[test]
    fn brackets_in_ltr_runs_are_not_mirrored() {
        assert_eq!(display_text("Faseela (UI) 1.0"), "Faseela (UI) 1.0");
        assert_eq!(display_text("Faseela (UI) العربية"), "Faseela (UI) العربية");
    }

    #[test]
    fn numbers_and_currency_words_follow_rtl_reading_order() {
        assert_eq!(display_text("سعر المنتج 250 جنيه"), "جنيه 250 المنتج سعر");
        assert_eq!(display_text("العدد ١٢٣ في القائمة"), "القائمة في ١٢٣ العدد");
    }

    #[test]
    fn punctuation_stays_attached_to_its_arabic_word() {
        assert_eq!(
            display_text("مرحبا، Faseela! كيف الحال؟"),
            "؟الحال كيف !Faseela ،مرحبا"
        );
    }

    #[test]
    fn latin_file_names_and_paths_keep_their_internal_order() {
        assert_eq!(
            display_text("افتح الملف report.pdf الآن"),
            "الآن report.pdf الملف افتح"
        );
        assert_eq!(display_text("ملف: C:\\docs\\a.txt"), "C:\\docs\\a.txt :ملف");
    }

    #[test]
    fn hebrew_is_reordered_like_arabic() {
        assert_eq!(display_text("שלום עולם Faseela"), "Faseela עולם שלום");
    }

    #[test]
    fn empty_and_whitespace_only_input_is_preserved() {
        assert_eq!(display_text(""), "");
        assert_eq!(display_text("   "), "   ");
        assert_eq!(display_text("\n"), "\n");
    }

    #[test]
    fn line_breaks_are_preserved_and_processed_independently() {
        let visual = display_text("السلام\nFaseela");
        assert_eq!(visual.lines().count(), 2);
        assert!(visual.ends_with("Faseela"));
    }
}
