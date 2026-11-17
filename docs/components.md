# Components

All components read from the active design tokens (see [Themes](theme.md)) and borrow
application-owned values. They return responses or proposals; they do not keep a second application
model. The showcase in `examples/showcase` uses these same public APIs.

Module paths are importable directly, for example
`use faseela_ui::buttons::{Button, ButtonVariant, ControlSize};`.

## Buttons

### `Button`

A themed text button with five variants, three sizes, a leading icon, selected state, and disabled
state.

```rust
use faseela_ui::buttons::{Button, ButtonVariant, ControlSize};

let response = Button::new("Publish")
    .variant(ButtonVariant::Primary)
    .size(ControlSize::Medium)
    .icon("↑")
    .selected(false)
    .id(egui::Id::unique("publish"))
    .show(ui);

if response.clicked() {
    // Application command.
}
```

Builder methods:

| Method | Meaning |
| --- | --- |
| `variant(ButtonVariant)` | `Primary`, `Secondary`, `Outline`, `Ghost`, or `Destructive`. |
| `size(ControlSize)` | `Small` (26 pt high), `Medium` (32 pt), or `Large` (42 pt). |
| `icon(impl Into<Cow<str>>)` | Leading glyph or short icon text; placed on the reading-start side. |
| `selected(bool)` | Paints the selected treatment and reports selected state. Selection is caller-owned. |
| `enabled(bool)` | Disabled controls remain visible and named but cannot activate. |
| `id(Id)` | Stable widget ID. Omit to use egui's normal layout identity. |
| `min_width(f32)` | Minimum logical width. |
| `full_width()` | Fill the available width. |
| `.show(ui)` / `ui.add(button)` | Both `Widget` and the explicit form are supported. |

Space and Enter activate a focused button through egui's click interaction. The focus ring is painted
inside the button bounds and uses `ThemeTokens::focus`.

### `IconButton`

An icon-only button with an independent accessible label. The caller paints the icon and owns
selection.

```rust
use faseela_ui::buttons::{IconButton, IconButtonStyle};

let style = IconButtonStyle::themed(
    faseela_ui::theme::active_tokens(ui.ctx()),
    faseela_ui::buttons::ButtonVariant::Outline,
    faseela_ui::buttons::ControlSize::Medium,
);

let response = IconButton::new("Toggle sidebar", egui::vec2(36.0, 36.0), &style)
    .selected(sidebar_open)
    .id(egui::Id::unique("toggle-sidebar"))
    .show(ui, |ui, rect, tint| {
        ui.painter().circle_filled(rect.center(), 6.0, tint);
    });

if response.clicked() {
    sidebar_open = !sidebar_open;
}
```

`IconButtonStyle` has `normal`, `hovered`, `pressed`, `selected`, `disabled`, `corner_radius`, and
`focus_stroke` fields. `ButtonVisuals` has `fill`, `stroke`, and `icon`. Build a complete style with
`IconButtonStyle::themed` or supply explicit state colors.

## Form controls

All form controls update application-owned values in place and return a `Response`. Use the
application's value after the call for validation, persistence, or command dispatch.

### `TextField`

```rust
use faseela_ui::forms::TextField;

TextField::new(&mut app.name)
    .label("Full name")
    .hint("e.g. Faseela Rahman")
    .helper("Shown in your workspace")
    .width(320.0)
    .id(egui::Id::unique("name"))
    .show(ui);
```

- `label`, `hint`, `helper`, `error`, `enabled`, `id`, and `width` are supported.
- `error` paints an error border, displays the message, and sets the AccessKit invalid state.
- The requested width is clamped to the available width. The outline is painted inside the measured
  rectangle, so the control does not expand its parent by the stroke width.
- The visible label is exposed as the accessible name.

### `SearchField`

A `TextField` with a leading search glyph and `Role::SearchInput`:

```rust
use faseela_ui::forms::SearchField;

SearchField::new(&mut app.query)
    .label("Filter")
    .hint("Try “forms”")
    .show(ui);
```

`label`, `hint`, `enabled`, `id`, and `width` are available; the value is application-owned.

### `Checkbox` and `Switch`

```rust
use faseela_ui::forms::{Checkbox, Switch};

Checkbox::new("Product notifications", &mut app.notifications)
    .helper("Also updates the status panel")
    .id(egui::Id::unique("notifications"))
    .show(ui);

Switch::new("Compact spacing", &mut app.compact)
    .error("Not available in this mode")
    .show(ui);
```

Both support `enabled`, `id`, `helper`, and `error`. The whole labeled row is clickable. A focused
checkbox or switch responds to Space through egui's click handling. `Switch` is visually distinct
and reports a role of `Switch`.

### `RadioButton` and `RadioGroup`

`RadioButton` is a single caller-controlled option. `RadioGroup` owns the exclusive choice through a
borrowed index and adds arrow-key navigation.

```rust
use faseela_ui::forms::{RadioButton, RadioGroup};

RadioButton::new("Custom", app.mode == Mode::Custom)
    .id(egui::Id::unique("mode-custom"))
    .show(ui);

RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut app.density)
    .label("Density")
    .helper("Arrow keys follow the reading direction")
    .id(egui::Id::unique("density"))
    .show(ui);
```

`RadioGroup` supports `label`, `enabled`, `id`, `helper`, and `error`. Clicking an option selects it
immediately. Up/Down move backward/forward; Left/Right follow the interface reading direction.
Option IDs are `group_id.with(("radio-option", index))`, so they remain stable across layout changes.
An out-of-range selected index paints no option as selected and is clamped when the group is shown;
clicking selects a valid option again.

### `Select`

A themed combo box over a slice of labels and a borrowed selected index:

```rust
use faseela_ui::forms::Select;

Select::new("Plan", &["Starter", "Team", "Enterprise"], &mut app.plan)
    .helper("The selected index stays in application state")
    .width(260.0)
    .id(egui::Id::unique("plan"))
    .show(ui);
```

`Select` supports `enabled`, `id`, `width`, `helper`, and `error`. Click or Space/Enter opens the
popup; Escape closes it. Choosing an option updates the index and closes the popup. The visible label
is also the accessible name, and an error sets the invalid state.

## Navigation

### `ButtonGroup`

A horizontal group of related buttons with one selected index. It wraps at narrow widths and mirrors
its order in RTL.

```rust
use faseela_ui::navigation::ButtonGroup;

let result = ButtonGroup::new(&["Day", "Week", "Month"], &mut app.period)
    .accessible_label("Reporting period")
    .variant(faseela_ui::buttons::ButtonVariant::Outline)
    .size(faseela_ui::buttons::ControlSize::Medium)
    .id(egui::Id::unique("period"))
    .show(ui);

if let Some(index) = result.activated {
    // The borrowed index was already updated.
}
```

`SelectionResponse` contains the container `response` and the `activated` index. An out-of-range
starting selection paints no button as selected; clicking selects a valid index.

### `SegmentedControl`

An exclusive control with a connected segmented appearance:

```rust
use faseela_ui::navigation::SegmentedControl;

let result = SegmentedControl::new(&["Design", "Develop", "Ship"], &mut app.view)
    .label("Workspace view")
    .full_width()
    .id(egui::Id::unique("view"))
    .show(ui);
```

`label`, `enabled`, `full_width`, and `id` are supported. The control clamps its segments to the
available width and clips labels rather than overflowing its parent. Arrow keys move the selection:
Up/Down backward/forward, Left/Right in reading order; Home/End jump to the first/last segment.

### `Breadcrumb`

A navigation path whose earlier entries are links and whose current entry is not interactive.

```rust
use faseela_ui::navigation::Breadcrumb;

let result = Breadcrumb::new(&["Workspace", "Projects", "Faseela UI", "Settings"])
    .accessible_label("Breadcrumb")
    .id(egui::Id::unique("breadcrumb"))
    .show(ui);

if let Some(index) = result.activated {
    // Navigate to the application route for the activated entry.
}
```

Items wrap at narrow widths. The separator arrow mirrors in RTL. Item IDs are derived from the
breadcrumb ID and the item index.

### `Tab`

An accessible tab interaction primitive. It does not allocate space; the caller lays out the strip
and paints the tab.

```rust
use faseela_ui::tabs::Tab;

let (rect, _) = ui.allocate_exact_size(egui::vec2(100.0, 32.0), egui::Sense::hover());
let response = Tab::new(egui::Id::unique("overview-tab"), "Overview", selected)
    .focus_stroke(egui::Stroke::new(2.0, tokens.focus))
    .show_at(ui, rect, |ui, response| {
        // Paint the tab using tokens and response state.
    });
```

`Role::Tab`, selected state, label, and enabled state are registered. Invalid or inverted geometry
makes the tab inert instead of panicking.

Tab-width fitting (`tabs::fit` and `tabs::fit_whole`) is a separate pure algorithm. It preserves the
selected tab, shrinks the widest tabs first, and reports overflow indices for an application-owned
overflow menu. See [Layout](layout.md).

## Feedback and status

### `Badge`

A compact, non-focusable status label:

```rust
use faseela_ui::widgets::{Badge, BadgeTone};

ui.add(Badge::new("Build passed", BadgeTone::Success));
```

`BadgeTone::{Neutral, Primary, Success, Warning, Error}` maps to the matching token and explicit
`on_*` foreground. Measurement and painting use the same font (`FontId::proportional(11.0)`), so the
badge does not clip its own text.

### `ProgressBar`

A determinate bar with an optional label and percentage:

```rust
use faseela_ui::widgets::ProgressBar;

ProgressBar::new(0.65)
    .label("Release upload")
    .show_value(true)
    .show(ui);
```

Values are clamped to `0.0..=1.0`; non-finite values become zero. `width` overrides the available
width when it is positive. The label row stays compact even inside a tall `Ui`.

### `LoadingIndicator`

```rust
use faseela_ui::widgets::LoadingIndicator;

LoadingIndicator::new()
    .label("Syncing workspace…")
    .size(16.0)
    .show(ui);
```

The spinner reports `Role::ProgressIndicator` and the supplied label (or `"Loading"`).

### `StatusIndicator`

```rust
use faseela_ui::widgets::{BadgeTone, StatusIndicator};

StatusIndicator::new("API connected", BadgeTone::Success)
    .detail("42 ms")
    .show(ui);
```

The tone controls the dot color; the label and optional detail use text tokens.

### `Separator`

```rust
use faseela_ui::widgets::Separator;

Separator::horizontal().show(ui);
Separator::vertical().length(80.0).inset(4.0).show(ui);
```

A decorative rule. It is not focusable and does not imply a semantic grouping.

### `Tooltip`

```rust
use faseela_ui::widgets::Tooltip;

let response = ui.button("Hover for details");
let response = Tooltip::new("Save the current document").show(response);
```

The tooltip is attached with egui's hover handling. It is supporting text, not a replacement for a
visible label or accessible name; keyboard-only and touch users may never see it.

### `Card` and `SectionHeader`

```rust
use faseela_ui::widgets::{Card, SectionHeader};

Card::new().elevated(true).show(ui, |ui| {
    SectionHeader::new("Appearance")
        .caption("Choose a visual preset")
        .show(ui);
});
```

`Card::new()` uses `surface`; `elevated(true)` uses `elevated_surface`. Both use `border` for the
outline. `fill`, `stroke`, `corner_radius`, `padding`, and `outer_margin` can be overridden.

### `EmptyState`

```rust
use faseela_ui::widgets::EmptyState;

let response = EmptyState::new("No notifications")
    .description("Enable notifications to see updates here.")
    .action("Enable notifications")
    .show(ui);

if response.inner.action_clicked {
    // Application enables notifications and re-renders.
}
```

`action_enabled(false)` keeps the action visible and named but non-activatable. The response also
reports `action_enabled` so the caller can mirror the state.

## Toolbar

### `toolbar_response`

The full responsive command-bar result:

```rust
use faseela_ui::widgets::{ToolbarItem, ToolbarState, toolbar_response};

let items = [
    ToolbarItem::new("New"),
    ToolbarItem::new("Open"),
    ToolbarItem::new("Save").disabled(),
];
let result = toolbar_response(ui, egui::Id::unique("main-toolbar"), &items);

if let Some(index) = result.activated {
    // Dispatch items[index]'s command.
}
```

`ToolbarResponse` fields:

| Field | Meaning |
| --- | --- |
| `state` | `Empty`, `Fits`, `Overflow`, or `NoSpace`. |
| `activated` | Index activated this frame, or `None`. |
| `shown` | Visible item indices, in input order. |
| `overflow` | Item indices listed in the overflow menu, in input order. |
| `overflow_button_id` | Stable ID of the overflow trigger when shown. |

`contains(index)` reports whether an index is reachable either visibly or through the menu. Items keep
their natural widths. The overflow label is the actual count (`+3`, and correctly measured for
`+100` and beyond) and drives the reserved width. Every item has a stable ID: visible item `i` is
`id.with(("toolbar-item", i))`, menu item `i` is `id.with(("overflow-item", i))`.

Disabled items stay visible and named in both locations but cannot activate. If even the overflow
control does not fit, `state` is `NoSpace`; nothing is painted and the caller must provide an explicit
fallback such as a wrapping row or a compact menu. Do not silently hide commands.

### `toolbar`

The compatibility wrapper returns `Option<usize>` and delegates to `toolbar_response`:

```rust
use faseela_ui::widgets::{ToolbarItem, toolbar};

if let Some(index) = toolbar(ui, egui::Id::unique("main-toolbar"), &items) {
    // Dispatch the command.
}
```

New code that needs layout state or overflow contents should use `toolbar_response`.

## Layout components

See [Layout primitives](layout.md) for `split_rect`, `ResizeHandle`, `fit_toolbar`, RTL mapping, and
sizing guarantees.

## Accessibility notes

See [Accessibility](accessibility.md) for roles, keyboard behavior, contrast requirements, and
platform limitations.
