# Accessibility

Accessibility is part of the widget contract, not an optional layer. Interactive widgets register a
role, label, enabled state, and selected state where applicable. Focus indicators are painted from
the theme's `focus` token, and keyboard behavior is implemented in the component or delegated to
egui's click interaction.

## What the components provide

| Component | Role / semantics | Keyboard and focus |
| --- | --- | --- |
| `buttons::Button` | `Role::Button`, selected state, visible label | Tab traversal, Space/Enter activation, inset focus ring, disabled state |
| `buttons::IconButton` | `Role::Button`, selected state, independent accessible label | Tab traversal, Space/Enter, inset focus stroke, disabled state |
| `forms::TextField` | Text input role; visible label as accessible name; error sets AccessKit invalid state | Text editing, selection, focus ring |
| `forms::SearchField` | `Role::SearchInput` with label and search glyph | Text editing, selection, focus ring |
| `forms::Checkbox` | `Role::CheckBox`, checked state, full-row click target | Space toggles when focused, focus ring |
| `forms::Switch` | `Role::Switch`, checked state | Space toggles when focused, focus ring |
| `forms::RadioButton` | `Role::RadioButton`, selected state | Space/Enter activation, focus ring |
| `forms::RadioGroup` | `Role::RadioGroup` with group label; options are `Role::RadioButton` | Up/Down and direction-aware Left/Right, focus moves with selection |
| `forms::Select` | Combo-box semantics from egui; label as accessible name; error sets invalid state | Space/Enter opens, Escape closes, arrow keys navigate the popup |
| `navigation::ButtonGroup` | `Role::Toolbar` container; items are buttons with selected state | Tab traversal, Space/Enter activation, wraps at narrow widths |
| `navigation::SegmentedControl` | `Role::RadioGroup` container; segments are `Role::RadioButton` | Up/Down, Home/End, direction-aware Left/Right, Space/Enter |
| `navigation::Breadcrumb` | `Role::Navigation` container; earlier items are `Role::Link` | Tab traversal and Space/Enter on links; current item is not interactive |
| `tabs::Tab` | `Role::Tab`, selected state, caller-painted focus outline | Caller-provided sense plus egui focus handling |
| `layout::ResizeHandle` | `Role::Splitter`, orientation, numeric value/range, Increment/Decrement/SetValue actions | Arrows along the split axis, Tab traversal, temporary focus for pointer drags |
| `widgets::ProgressBar` | `Role::ProgressIndicator`, accessible label, numeric 0–100 value | Static |
| `widgets::LoadingIndicator` | `Role::ProgressIndicator`, label or `"Loading"` | Static |
| `widgets::StatusIndicator` | Label plus `Role::Status` on the returned response | Static |
| `widgets::Badge` | `Role::Label` with its text | Static, not focusable |
| `widgets::toolbar` / `toolbar_response` | Buttons plus a menu-based overflow trigger; all commands remain reachable | Tab traversal, Space/Enter, menu navigation |
| `widgets::EmptyState` action | Ordinary button | Tab traversal, Space/Enter |
| `widgets::Tooltip` | Supporting hover text, not an accessible name | Pointer hover; see limits below |

## Direction-aware navigation

`direction::step` maps horizontal arrow keys to logical previous/next in LTR and RTL.
`direction::row`, `trailing`, `start_align`, `start_x`, `end_x`, and `forward` keep layout and
navigation consistent with the reading direction. `RadioGroup` and `SegmentedControl` use these rules
for Left/Right; Up/Down always mean previous/next. The resize handle and toolbar use the direction
stored on the egui context.

## Focus and disabled state

- Focus indicators use `ThemeTokens::focus` and are inset so they remain visible inside bounds.
- On a selected/filled control, the inset focus ring switches to the corresponding contrasting
  foreground token when that is more legible than the ordinary focus token.
- A focused button, checkbox, switch, or clickable row responds to Space; Enter also activates
  clickable buttons through egui's standard handling.
- Disabled components keep their accessible role and visible label but report `enabled: false`. They
  cannot be clicked or activated. Disabled toolbar commands behave the same in the bar and in the
  overflow menu.
- Invalid geometry makes a tab or resize handle inert rather than panicking or creating a focus trap.
- Widget IDs are stable and never derived from translated labels. For repeated items, the supplied
  stable ID is combined with the item index, for example
  `id.with(("toolbar-item", index))` or `id.with(("radio-option", index))`.

## Contrast evidence

The theme tests compute WCAG 2.x relative luminance and contrast ratios rather than applying a
luminance threshold to choose text colors. For every built-in preset they assert:

- **Text, 4.5:1 minimum:** `text`, `muted_text`, and `secondary` on their intended backgrounds.
- **Foreground pairs, 4.5:1 minimum:** `on_primary`/`primary`, `on_secondary`/`secondary`,
  `on_accent`/`accent`, `on_success`/`success`, `on_warning`/`warning`, and `on_error`/`error`.
- **Non-text, 3:1 minimum:** `focus` against the backgrounds where focus rings appear, and
  `control_border` against control surfaces.

The tests exercise all four presets, not only the default. The showcase's Appearance and
Accessibility sections display the measured ratios from the same `contrast_ratio` function.

This is evidence for the documented token pairs. It is not a complete audit of every composition an
application can paint, and it does not cover text rendered at unusual sizes, custom colors, images,
or application-provided content. Test custom color pairs with `theme::contrast_ratio` and do not rely
on `ThemeTokens::on_color` for mid-tone fills: it chooses the better of white and dark ink, but both
may still fail 4.5:1.

## Tests

Accessibility behavior is covered by unit and integration tests in the workspace:

- `theme::tests::every_preset_meets_wcag_aa_for_body_text_and_foregrounds` checks text, semantic
  foregrounds, focus, and control borders across all presets.
- `theme::tests::wcag_relative_luminance_matches_reference_values` checks the WCAG formula against
  known reference values.
- `buttons::tests::keyboard_space_and_enter_activate_a_focused_button` covers keyboard activation.
- `buttons::tests::disabled_text_button_cannot_be_clicked` covers disabled behavior.
- `forms::tests::switch_toggles_by_keyboard_space_when_focused` covers switch keyboard behavior.
- `forms::tests::radio_group_changes_selection_by_click_and_arrow_key` covers radio-group keyboard
  navigation.
- `forms::tests::select_opens_with_keyboard_from_focus_and_closes_with_escape` covers select keyboard
  operation.
- `navigation::tests::segmented_control_click_and_arrow_keys_update_selection` covers segmented
  keyboard navigation.
- `navigation::tests::breadcrumb_activates_links_but_not_the_current_page` covers link/current-page
  semantics.
- `widgets::tests::toolbar_reports_counts_and_fits_within_available_width`,
  `widgets::tests::overflow_label_uses_the_actual_count_including_three_digits`, and
  `widgets::tests::disabled_toolbar_items_are_inert_in_the_bar_and_overflow_menu` cover overflow,
  large counts, disabled commands, and reachability.
- `widgets::tests::toolbar_item_ids_are_stable_in_ltr_and_rtl` covers stable IDs in both directions.
- `layout::resize::tests::accesskit_increment_decrement_and_set_value_produce_resize_deltas` covers
  AccessKit splitter actions.
- Showcase tests render all ten sections at seven target viewports, in all themes and both
  directions, and exercise navigation, forms, progress, theme switching, and RTL state.

## Platform support in egui 0.37

- **Native:** the showcase enables eframe's `accesskit` feature for native builds. Roles, labels,
  selected states, numeric progress values, and splitter actions are delivered to the platform
  AccessKit adapter.
- **Web/WebAssembly:** eframe 0.37 does not expose per-widget AccessKit nodes on the web
  (`accesskit_update` is not implemented in its web backend). The optional `web_screen_reader`
  feature speaks egui widget-event descriptions through the Web Speech API and is experimental; the
  showcase does not enable it. Browser screen-reader verification therefore requires manual testing
  and is not claimed here.
- **Platform matrix:** actual screen-reader behavior (NVDA, VoiceOver, Orca, TalkBack) depends on the
  platform and eframe backend. It has not been verified on every target.

## Known limitations

- Helper and error nodes are linked to their controls using AccessKit `described_by`; errors also
  set invalid state on checkbox, switch, radio group, select, and text input. Backend announcement
  of description relationships is not guaranteed.
- `Tooltip` uses pointer hover. It does not replace a visible label, and keyboard-only or touch users
  may never see it. Do not put essential information only in a tooltip.
- Focus order follows widget layout. Use stable IDs and explicit labels; do not encode meaning only
  in color.
- Full Arabic text editing, bidi caret movement, selection, line breaking, IME, and locale formatting
  are out of scope. See [Arabic support](arabic-support.md) for the exact list.
- The contrast tests cover token pairs, not arbitrary custom painting.

## Interaction verification

- `Checkbox` and `Switch` publish the new toggled state on the same activation frame. Their default
  IDs use egui layout identity rather than the label; translated duplicate labels remain distinct.
- `RadioGroup` and `SegmentedControl` use selected radio semantics, not checkbox toggled state. The
  segmented control's visible label is its group name. It and the radio group consume handled arrow
  keys and return `changed()` when the selected index changes.
- `ButtonGroup::response.changed()` indicates a different selected index; `activated` also reports
  activation of an already-selected item. Select returns changed response when the caller's index
  changes through the popup.
- `Tab` publishes AccessKit `selected`. `IconButton` preserves the caller's selected state while
  disabled; all disabled controls remain inert.
- Integration tests drive egui `RawInput` for pointer, Space/Enter, arrow, Home/End, Escape,
  clipboard and IME events, then assert caller state, responses and AccessKit nodes. They do not
  emulate native focus managers or a real OS/web screen-reader pipeline.
