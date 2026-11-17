# Themes and design tokens

The theme engine has one source of truth: [`faseela_ui::theme::ThemeTokens`](https://docs.rs/faseela-ui/latest/faseela_ui/theme/struct.ThemeTokens.html). Every preset maps to a complete token set, and both egui's built-in widgets and Faseela's custom widgets read from that set.

## Presets

| Preset | Character |
| --- | --- |
| `ThemePreset::FaseelaDark` | Official identity: `#0A1D1A` background, `#2FA36B` primary, `#9BEBB8` secondary, `#F2C14E` accent. |
| `ThemePreset::Midnight` | Cool indigo palette with lavender highlights. |
| `ThemePreset::Forest` | Deep green palette with warm gold accents. |
| `ThemePreset::Light` | Light surfaces (`#FFFFFF`/`#F4F7F5`) with dark green text and actions. |

`ThemePreset::ALL` lists all presets in display order; `from_index` and `label` are stable and suitable for selection controls.

## Applying a theme

```rust
use faseela_ui::theme::{self, ThemePreset};

theme::apply_preset(&ctx, ThemePreset::Light);
```

Call `apply_preset` whenever the selection changes; it is cheap and idempotent. `apply_dark(&ctx)` remains as a compatibility shortcut for the default preset. The function:

1. builds a `Visuals` set from the token values (text, muted text, borders, hover, active, disabled, selection, focus, error, warning);
2. installs it for the matching egui `Theme` (dark or light) and sets that theme as active;
3. records the preset on the context so `active_tokens` returns the same tokens.

Light mode is a real light palette: it changes surfaces, text, borders, hover and selection colors together. It is not implemented by tinting the dark background.

## Reading tokens in custom widgets

Widgets that paint with `ui.painter()` should never use fixed `palette` constants for state-dependent colors. Read the active tokens instead:

```rust
use faseela_ui::theme;

let tokens = theme::active_tokens(ui.ctx());
ui.painter().rect_filled(rect, 8.0.into(), tokens.surface);
ui.painter().rect_filled(accent_rect, 8.0.into(), tokens.primary);
```

`active_tokens` returns the default preset's tokens until a preset is applied. It is backed by egui's context data (no global mutable state).

## Token reference

| Token | Role |
| --- | --- |
| `background` | Application background behind panels. |
| `surface` | Ordinary surface: panels and cards. |
| `elevated_surface` | Raised surface: hovered cards, menus, trailing panes. |
| `primary` | Main action/emphasis color. |
| `secondary` | Links, captions and highlights. |
| `accent` | Decorative accent. |
| `text` | Primary body text. |
| `muted_text` | De-emphasized text. |
| `border` | Separators and control outlines. |
| `selection` | Selected rows, tabs, and text selection. |
| `hover` | Pointer-hover fill. |
| `active` | Pressed/active fill. |
| `disabled` | Disabled control fill. |
| `focus` | Keyboard focus indicator. |
| `error` / `warning` / `success` | Semantic status colors. |

`ThemeTokens::on_color(color)` picks readable foreground text (white or dark) from the luminance of a fill, which keeps primary-colored buttons legible in both light and dark presets.

## Fonts

`install_arabic_font(&ctx)` installs Noto Sans Arabic before DejaVu Sans for proportional and monospace families. Call it once at startup, independently of the theme; see [Arabic support](arabic-support.md).
