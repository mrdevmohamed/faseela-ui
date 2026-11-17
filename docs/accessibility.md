# Accessibility

Accessibility is part of the widget contract, not an optional layer. Every interactive widget registers an accessible role, enabled state, and label; keyboard activation and focus indicators are provided where applicable.

## What the widgets provide

| Widget | Role / semantics | Keyboard & focus |
| --- | --- | --- |
| `buttons::IconButton` | `Role::Button`, selected state, independent accessible label | egui click/focus handling, inset focus stroke |
| `tabs::Tab` | `Role::Tab`, selected state, label independent of id | egui interaction plus inset focus stroke |
| `layout::ResizeHandle` | `Role::Splitter`, orientation, numeric value/range, Increment/Decrement/SetValue actions | arrows along the split axis, Tab traversal, temporary focus for pointer drags |
| `widgets::Badge` | `Role::Label` with its text | static, not focusable |
| `widgets::toolbar` | ordinary buttons plus a menu-based overflow control | Tab traversal, menu items are real buttons |
| `widgets::EmptyState` action | ordinary button | Tab traversal and activation |

## Direction-aware navigation

`direction::step` maps horizontal arrow keys to logical previous/next in LTR and RTL. `direction::row`, `trailing`, `start_align`, `start_x`, `end_x`, and `forward` keep layout and navigation consistent with the reading direction. The resize handle and toolbar use the direction stored on the egui context.

## Focus and disabled state

- Focus indicators are drawn from `ThemeTokens::focus` and are inset so they remain visible inside bounds.
- Disabled widgets keep their accessible role and label but report `enabled: false`; invalid geometry makes a widget inert rather than focusable.
- Widget IDs are stable and never derived from translated labels. `Id::unique` hashes a fixed salt; append an index for repeated items.

## Tests

Accessibility behavior is covered by unit tests in the crate:

- `tabs::tests::tab_reports_tab_role_label_and_selected_state` verifies the emitted `WidgetInfo` role, selected state, label, and enabled flag after a click.
- `layout::resize::tests::accesskit_increment_decrement_and_set_value_produce_resize_deltas` verifies AccessKit actions.
- `layout::resize::tests::focused_handle_uses_only_the_arrows_along_its_axis` verifies arrow-key handling and focus locking.
- `layout::resize::tests::disabled_and_invalid_handles_never_resize` verifies disabled/inert semantics.
- `widgets::tests::*` render all widgets for every preset and both directions and verify toolbar LTR/RTL placement.

Limitations: egui/AccessKit integration is exercised through egui's in-process harness; real screen-reader verification still requires manual testing on each target platform. Full Arabic text-editing accessibility is out of scope (see [Arabic support](arabic-support.md)).
