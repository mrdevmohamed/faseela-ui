# Themes and design tokens

The theme engine has one source of truth:
[`faseela_ui::theme::ThemeTokens`](https://docs.rs/faseela-ui/latest/faseela_ui/theme/struct.ThemeTokens.html).
Every preset maps to a complete token set. egui's built-in widgets and Faseela's custom widgets read
from the same values, so a preset change does not leave half the interface on the old palette.

## Presets

| Preset | Character |
| --- | --- |
| `ThemePreset::FaseelaDark` | Official identity: `#0A1D1A` background, `#2FA36B` primary, `#9BEBB8` secondary, `#F2C14E` accent. |
| `ThemePreset::Midnight` | Cool indigo palette with lavender highlights. |
| `ThemePreset::Forest` | Deep green palette with warm gold accents; primary is `#46996C` so indicators remain visible on elevated surfaces. |
| `ThemePreset::Light` | Light surfaces (`#FFFFFF`/`#F4F7F5`) with dark green text and actions. Secondary is `#2B6B49`, accent is `#8F5C00`, and muted text is `#55665D` to keep contrast above 4.5:1. |

`ThemePreset::ALL` lists all presets in display order. `label`, `from_index`, and `tokens` are stable
and suitable for selection controls. Unknown indices fall back to `FaseelaDark`.

## Applying a theme

```rust
use faseela_ui::theme::{self, ThemePreset};

theme::apply_preset(&ctx, ThemePreset::Light);
```

Call `apply_preset` whenever the selection changes; it is cheap and idempotent. `apply_dark(&ctx)`
remains as a compatibility shortcut for the default preset. The function:

1. builds a `Visuals` set from the token values (text, muted text, borders, hover, active, disabled,
   selection, focus, error, warning);
2. installs it for the matching egui `Theme` (dark or light) and selects that theme;
3. sets spacing and corner defaults used by shared widgets;
4. records the preset on the egui context so `active_tokens` and `active_preset` return the same
   values.

If you switch presets from inside an `App::update` implementation, the current `Ui` may have cached
the previous style because it was created before the switch. Reset it after applying the preset:

```rust
theme::apply_preset(ui.ctx(), selected);
ui.reset_style();
ui.ctx().request_repaint();
```

Light mode is a real light palette: it changes surfaces, text, borders, hover, and selection together.
It is not implemented by tinting the dark background.

## Reading tokens in custom widgets

Widgets that paint with `ui.painter()` should not use fixed `palette` constants for state-dependent
colors. Read the active tokens instead:

```rust
use faseela_ui::theme;

let tokens = theme::active_tokens(ui.ctx());
ui.painter().rect_filled(rect, 8.0.into(), tokens.surface);
ui.painter().rect_filled(accent_rect, 8.0.into(), tokens.primary);
```

`active_tokens` returns the default preset's tokens until a preset is applied. It is backed by egui's
context data, so there is no mutable global state.

## Token reference

| Token | Role |
| --- | --- |
| `background` | Application background behind panels and windows. |
| `surface` | Ordinary surface: panels and cards. |
| `elevated_surface` | Raised surface: hovered cards, menus, trailing panes. |
| `primary` | Main action and emphasis color. |
| `secondary` | Links, captions, and highlights. |
| `accent` | Decorative accent and warning-family emphasis. |
| `text` | Primary body text. |
| `muted_text` | De-emphasized text and captions. |
| `border` | Subtle separators and decorative card outlines. |
| `control_border` | Boundary for checkboxes, switches, inputs, and other interactive controls; intended to meet the WCAG 3:1 non-text requirement. |
| `selection` | Selected rows, tabs, and text selection. |
| `hover` | Pointer-hover fill. |
| `active` | Pressed/active fill. |
| `disabled` | Disabled control fill. |
| `disabled_text` | Disabled text and icon color. |
| `focus` | Keyboard focus indicator. |
| `error` / `warning` / `success` | Semantic status colors. |
| `on_primary` | Text/icon foreground on `primary` fills and selected states. |
| `on_secondary` | Foreground on `secondary` fills. |
| `on_accent` | Foreground on `accent` fills. |
| `on_success` | Foreground on `success` fills. |
| `on_warning` | Foreground on `warning` fills. |
| `on_error` | Foreground on `error` fills. |

The `on_*` tokens are explicit per preset and tested. They are not derived from a luminance threshold
at paint time. This matters because a fill can be dark enough to need white text in one preset and
light enough to need dark text in another.

`ThemeTokens::on_color(color)` chooses the higher-contrast of white and Faseela's dark ink
(`#1D2B23`). It is a best-effort helper for arbitrary application colors: a mid-tone fill can still
be below 4.5:1 with both candidates. Use the explicit `on_*` tokens for built-in fills, and test any
custom color pair with `contrast_ratio`.

## WCAG contrast helpers

The crate implements the WCAG 2.x formulas over sRGB channels:

```rust
use faseela_ui::theme;

let luminance = theme::relative_luminance(tokens.primary);
let ratio = theme::contrast_ratio(tokens.on_primary, tokens.primary);
let blended = theme::blend(tokens.surface, tokens.primary, 0.25);
```

`relative_luminance` linearizes each channel with the WCAG piecewise sRGB transform.
`contrast_ratio` returns a value in `1.0..=21.0`; alpha is ignored for contrast decisions.
`blend` is a straight sRGB-style interpolation used for hover/pressed compositions, not an
alpha-compositing model for arbitrary stacking.

The built-in preset tests assert:

- normal text pairs (`text`, `muted_text`, `secondary`) at **4.5:1** or better on their intended
  backgrounds;
- every semantic `on_*` foreground on its fill at **4.5:1** or better;
- focus indicators and `control_border` at **3:1** or better.

These tests cover the documented token pairs. They do not prove that every arbitrary composition an
application paints meets WCAG; custom widgets must test their own color combinations and sizes.

## Custom presets

`ThemePreset` is a closed enum. Applications that need a different palette can build a
`ThemeTokens` value, install their own egui `Visuals`, and paint custom components from that token
set. The shared widgets read tokens through `active_tokens`, which is tied to `ThemePreset`; a custom
palette therefore needs its own installation path or a small local wrapper. Do not construct a
partial `ThemeTokens` literal: include every token, including `control_border`, `disabled_text`, and
the `on_*` fields.

## Fonts

`install_arabic_font(&ctx)` installs Noto Sans Arabic before DejaVu Sans for proportional and
monospace families. Call it once at startup, independently of the theme; see
[Arabic support](arabic-support.md). The font installation replaces egui's font definitions, so call
it after any other application font setup or merge that setup into the same definitions.
