# Faseela UI

Reusable Rust/egui components and design foundations for Faseela applications. The crate owns
presentation and interaction primitives; applications own routes, commands, form values, selection,
and persistence.

- **Stack:** Rust 2024, MSRV 1.95, egui/eframe 0.37, native and WebAssembly.
- **Themes:** four presets backed by one semantic token set, including explicit `on_*` foreground
  tokens and measured WCAG contrast.
- **Components:** buttons, form controls, navigation, layout, feedback, and status widgets.
- **Showcase:** a real interactive application, not a gallery of static mockups, published with
  Trunk on GitHub Pages.

## Requirements

- Rust 1.95 or newer with the 2024 edition.
- egui/eframe 0.37 for the host application.
- For WebAssembly: the `wasm32-unknown-unknown` target and
  [Trunk](https://trunkrs.dev/).

## Install

The crate is part of this workspace and is not currently published on crates.io.

```toml
[dependencies]
faseela-ui = { path = "../faseela-ui/crates/faseela-ui" }
# or
faseela-ui = { git = "https://github.com/mrdevmohamed/faseela-ui" }
```

Quick start:

```rust
use faseela_ui::theme::{self, ThemePreset};
use faseela_ui::buttons::Button;

theme::apply_preset(&egui_ctx, ThemePreset::FaseelaDark);
theme::install_arabic_font(&egui_ctx);

if Button::new("Save").show(ui).clicked() {
    // Dispatch the application command.
}
```

Applications should call `apply_preset` whenever the selected preset changes. The call is cheap,
idempotent, and records the active tokens so both egui's built-in widgets and Faseela's custom
widgets use the same palette.

## Components

| Module | Public components |
| --- | --- |
| `buttons` | `Button` (primary, secondary, outline, ghost, destructive; small/medium/large; leading icon, selected, disabled, full width), `IconButton`, `ControlSize`, `IconButtonStyle` |
| `forms` | `TextField`, `SearchField`, `Checkbox`, `Switch`, `RadioButton`, `RadioGroup`, `Select` |
| `navigation` | `ButtonGroup`, `SegmentedControl`, `Breadcrumb`, `SelectionResponse`, `BreadcrumbResponse` |
| `widgets` | `Card`, `SectionHeader`, `Badge`, `ProgressBar`, `LoadingIndicator`, `StatusIndicator`, `Separator`, `Tooltip`, `EmptyState`, `toolbar`, `toolbar_response` |
| `layout` | `split_rect`, `SplitSize`, `SplitAxis`, `ResizeHandle`, `fit_toolbar` |
| `tabs` | `Tab`, `fit`, `fit_whole` width-fitting algorithms |
| `theme` | `ThemePreset`, `ThemeTokens`, WCAG helpers, `apply_preset`, `active_tokens`, `install_arabic_font`, `palette` |
| `direction` | `Direction`, `Setting`, direction-aware row/trailing/alignment/step helpers |
| `text` | `display_text` for short RTL/LTR display labels |
| `icons` | Shared embedded SVG asset preparation |

### Buttons

```rust
use faseela_ui::buttons::{Button, ButtonVariant, ControlSize};

let save = Button::new("Save")
    .variant(ButtonVariant::Primary)
    .size(ControlSize::Medium)
    .show(ui);
if save.clicked() {
    // Application action.
}
```

`Button` supports `variant`, `size`, `icon`, `selected`, `enabled`, `id`, `min_width`, `full_width`,
and both `.show(ui)` and `ui.add(button)`. A disabled button stays visible and named but does not
activate. Space and Enter activate a focused button through egui's click handling.

### Form controls

All form widgets borrow application-owned values or indices and report changes through the returned
`Response`; they do not keep a duplicate model.

```rust
use faseela_ui::forms::{Checkbox, RadioGroup, Select, TextField};

let name = TextField::new(&mut app.name)
    .label("Full name")
    .helper("Shown in the workspace")
    .show(ui);

Checkbox::new("Notifications", &mut app.notifications).show(ui);
RadioGroup::new(&["Compact", "Comfortable"], &mut app.density)
    .label("Density")
    .show(ui);
Select::new("Plan", &["Starter", "Team"], &mut app.plan).show(ui);
```

Each control accepts a stable `id`, a visible label or checkbox/switch label, optional helper or
error text, and `enabled(false)`. Text controls accept a `width` that is clamped to the available
space, so a requested width never causes horizontal overflow. `Select` opens with click or
Space/Enter and closes with Escape or selection.

### Navigation

`ButtonGroup` and `SegmentedControl` update an application-owned selected index and return the
activated index. `Breadcrumb` keeps earlier entries interactive, mirrors its order in RTL, and
returns the activated entry. Repeated item IDs are derived from the supplied stable ID and the item
index, not from translated labels.

```rust
use faseela_ui::navigation::{Breadcrumb, SegmentedControl};

let crumbs = Breadcrumb::new(&["Workspace", "Projects", "Settings"]).show(ui);
if let Some(index) = crumbs.activated {
    // Navigate to the application route at `index`.
}

let view = SegmentedControl::new(&["Design", "Develop"], &mut app.view).show(ui);
if let Some(index) = view.activated {
    // Apply the application's view mode.
}
```

### Feedback and status

`ProgressBar::new(0.0..=1.0)` has an optional label and accessible percentage. `LoadingIndicator`
wraps an egui spinner with an accessible label. `StatusIndicator` combines a semantic tone with a
label and optional detail. `Separator` supports horizontal and vertical rules. `Badge` tones use the
matching explicit foreground token.

### Toolbar

`toolbar_response` reports the full layout result; the older `toolbar` function remains as a
compatibility wrapper returning only the activated command.

```rust
use faseela_ui::widgets::{ToolbarItem, ToolbarState, toolbar_response};

let items = [ToolbarItem::new("Open"), ToolbarItem::new("Save").disabled()];
let result = toolbar_response(ui, egui::Id::unique("main-toolbar"), &items);

if let Some(index) = result.activated {
    // Dispatch the command.
}
if result.state == ToolbarState::NoSpace {
    // Provide an explicit compact or wrapping fallback.
}
```

Trailing items move into a keyboard-reachable overflow menu. The overflow label uses the actual
overflow count (`+3`, or `+100` and beyond), and the reserved width is measured for that label. A
disabled item stays visible and named in both the bar and the overflow menu.

## Themes and contrast

The four presets are `FaseelaDark` (default), `Midnight`, `Forest`, and `Light`. `ThemePreset::ALL`,
`from_index`, and `label` provide stable selection data. `ThemeTokens` contains colors for surfaces,
text, borders, interaction states, status, disabled content, and explicit foregrounds
(`on_primary`, `on_secondary`, `on_accent`, `on_success`, `on_warning`, `on_error`).

```rust
use faseela_ui::theme::{self, ThemePreset};

theme::apply_preset(&ctx, ThemePreset::Forest);
let tokens = theme::active_tokens(&ctx);

let ratio = theme::contrast_ratio(tokens.text, tokens.background);
assert!(ratio >= 4.5);
```

The crate computes WCAG 2.x relative luminance and contrast ratios from sRGB channels. Built-in
tokens are tested for:

- **4.5:1** for normal-size text, including muted text, secondary text, and every `on_*` pair;
- **3:1** for non-text boundaries and focus indicators, including `control_border` and `focus`.

`ThemeTokens::on_color` is a best-effort helper for arbitrary application colors: it chooses the
higher-contrast of white and dark ink, but a mid-tone fill can still fail with both choices. Use
the explicit `on_*` tokens for built-in semantic fills. Applications that add custom presets are
responsible for testing their own contrast.

The subtle `border` token is for decorative separators. Interactive control boundaries use
`control_border`, because WCAG requires 3:1 for non-text UI components.

## Layout and responsive behavior

`split_rect` returns first pane, divider, and second pane rectangles with `Ratio`, `FixedFirst`, or
`FixedSecond` sizing. `ResizeHandle` adds pointer dragging, arrow keys, focus, and AccessKit
splitter actions, but returns a delta; the application persists the resulting `SplitSize`.
`fit_toolbar` is the stateless fitting algorithm; `toolbar_response` is the ready-made widget.

The showcase has a desktop sidebar shell at widths of 680 logical points and above, and a mobile
header with a "Sections" disclosure below that breakpoint. The content area wraps, metric cards and
swatches use wrapping rows, and segmented controls clamp and clip rather than forcing their parent
wider. The automated showcase tests cover 320×568, 360×640, 390×844, 680×800, 768×1024, 1280×800,
and 1920×1080 in all ten sections, all four themes, and both directions.

## Accessibility

Accessible roles, labels, enabled state, selected state, and focus painting are implemented for the
interactive components. Form controls expose their label as the accessible name; an error also sets
the AccessKit invalid state. Keyboard behavior includes:

- Space/Enter activation for focused buttons and clickable controls;
- Tab traversal and visible inset focus rings;
- arrow-key navigation for radio groups and segmented controls, following the reading direction;
- Escape to close a `Select` popup;
- pointer drag and keyboard resizing for `ResizeHandle`.

Known limits:

- Native AccessKit delivery depends on the eframe platform backend. The showcase enables eframe's
  `accesskit` feature for native builds.
- eframe 0.37's web backend does not expose per-widget AccessKit nodes to browser screen readers.
  Browser screen-reader verification still requires manual testing and is not claimed here.
- Helper and error text is visible, but explicit description relationships beyond the accessible
  name and invalid state are not guaranteed across backends.
- Full Arabic text editing, bidi caret movement, line breaking, IME, and locale formatting are not
  implemented. See [docs/arabic-support.md](docs/arabic-support.md).

## Arabic and RTL

- `direction::apply` sets the interface direction; `direction::row`, `trailing`, `start_align`,
  `start_x`, `end_x`, `forward`, and `step` keep layout and keyboard navigation logical.
- `theme::install_arabic_font` installs bundled Noto Sans Arabic before DejaVu Sans.
- `text::display_text` performs limited visual reordering and paired-punctuation mirroring for short
  display-only strings. Letter joining is left to egui's shaper.

Keep source strings in logical Unicode order. Do not pass `display_text` output to `TextEdit`, store
it, search it, or use it as a general-purpose text layout engine.

## Showcase

```bash
cd examples/showcase
trunk serve
```

Native:

```bash
cargo run -p faseela-ui-showcase
```

The live showcase is published at <https://mrdevmohamed.github.io/faseela-ui/>. It contains ten
sections: Overview, Foundations, Buttons, Form Controls, Navigation, Layout, Feedback & Status,
Appearance, Accessibility, and RTL & Arabic. Every control changes real application state; there are
no placeholder buttons.

The production WebAssembly build used by CI and Pages:

```bash
cd examples/showcase
trunk build --release
```

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p faseela-ui-showcase
cargo check -p faseela-ui-showcase --target wasm32-unknown-unknown
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo package -p faseela-ui --allow-dirty
```

CI additionally checks the MSRV toolchain (1.95) and performs a release Trunk build. GitHub Pages
deployment remains in `.github/workflows/pages.yml`.

## Workspace

- `crates/faseela-ui`: the reusable library.
- `examples/showcase`: the native/WebAssembly application used for visual and interaction testing.

Application-specific docking trees, command registries, document models, routing, persistence, and
business rules remain outside this crate.

## Compatibility and versioning

The workspace is pre-1.0 (`0.1.0`). Before 1.0, minor releases may contain breaking API changes;
they will be documented in release notes. The current compatibility guarantees are:

- **MSRV:** Rust 1.95, edition 2024.
- **egui:** 0.37. Upgrading egui is a semver-breaking change for this crate.
- **Stable IDs:** widgets accept caller-supplied IDs and derive repeated-item IDs from the supplied
  ID plus an index. Applications should not derive IDs from translated labels.
- **Compatibility wrappers:** `toolbar` delegates to `toolbar_response`; `apply_dark` delegates to
  `apply_preset(ctx, ThemePreset::FaseelaDark)`; the existing `IconButton` API is preserved.
- **ThemeTokens literals:** the struct gained `control_border`, `disabled_text`, and `on_*` fields.
  Code that constructed it with a complete struct literal must add those fields (or use
  `ThemePreset::tokens()`). No built-in token values are silently replaced at paint time.
- **Applications own state:** widgets borrow application values and return proposals; there is no
  global application state, router, or command bus in this crate.

Migration from an early Faseela UI checkout:

1. Replace `ui.add(IconButton::new(...))` only if you need the newer `Button` API; `IconButton`
   remains available.
2. Replace direct `ThemeTokens { ... }` literals with `ThemePreset::tokens()` or include the new
   tokens.
3. Prefer `toolbar_response` when you need overflow state; existing `toolbar` callers continue to
   receive `Option<usize>`.
4. Call `theme::install_arabic_font` once at startup if you display Arabic labels, and keep logical
   text separate from `display_text` output.

## Upstream inspiration

Some interaction and fitting algorithms are adapted from
[storytold/craft-libs](https://github.com/storytold/craft-libs), especially `crates/ui`, under MIT OR
Apache-2.0. Faseela-owned modules and changes are documented in [UPSTREAM.md](UPSTREAM.md).

## License

MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
