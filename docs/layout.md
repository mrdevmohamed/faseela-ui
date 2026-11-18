# Layout primitives

The `faseela_ui::layout` module provides small, stateless layout algorithms. It does not own a dock
tree, pane state, command model, menu, or pointer gesture. The `tabs` module provides a related pure
width-fitting algorithm, and `widgets::toolbar_response` is the ready-made responsive command bar.

## Two-pane geometry

`split_rect` divides an egui rectangle into a first pane, divider, and second pane:

```rust
use faseela_ui::layout::{split_rect, SplitAxis, SplitSize};

let split = split_rect(
    allocated_rect,
    SplitAxis::Horizontal,
    SplitSize::Ratio(0.62),
    8.0,   // divider width in logical points
    120.0, // minimum width for each pane when space permits
);

if let Some(split) = split {
    // Lay out or paint content in split.first, split.divider, and split.second.
}
```

Sizing modes are:

- `Ratio(f32)`: fraction of the space after removing the divider, assigned to the first pane.
- `FixedFirst(f32)`: logical-point width or height of the first pane.
- `FixedSecond(f32)`: logical-point width or height of the second pane.

Finite values outside the available range are clamped. The divider is capped to the available span.
When both pane minimums cannot fit, the minimum is relaxed so each pane receives half the remaining
space and neither rectangle is inverted. A zero-sized rectangle is valid. Invalid geometry,
non-finite values, and negative/non-finite gaps or minimums return `None`.

Horizontal geometry always places the first rectangle on the left and the second on the right.
Vertical geometry places the first above the second. In an RTL interface, keep the physical geometry
and map your logical leading/trailing pane contents to the opposite rectangles, as the showcase does.

### Resizing

The layout helper returns a proposal rather than changing application state:

```rust
if let Some(next_size) = split.resized(current_size, drag_delta, 160.0) {
    // Persist next_size in your application model.
}
```

The original sizing mode is retained: a ratio remains a ratio, `FixedFirst` remains fixed-first, and
`FixedSecond` remains fixed-second. The drag minimum may be stricter than the minimum used for the
current layout. Invalid inputs or zero available space return `None`, so the caller can leave the
persisted value unchanged.

`ResizeHandle` handles pointer dragging, axis-appropriate arrow keys, focus lifecycle, and AccessKit
splitter actions. It still does not mutate application state: apply the returned delta through the
split layout and persist the resulting `SplitSize`.

```rust
use egui::Id;
use faseela_ui::layout::{ResizeHandle, ResizeHandleStyle, SplitAxis};
use faseela_ui::theme;

let tokens = theme::active_tokens(ui.ctx());
let resize = ResizeHandle::new(
    Id::unique("workspace-split"),
    SplitAxis::Horizontal,
    split.divider,
    "Resize workspace panes",
    ResizeHandleStyle {
        hovered: tokens.hover,
        dragged: tokens.active,
        focused: tokens.focus,
    },
)
.value(split.first_size(), 120.0..=(split.available() - 120.0))
.hit_margin(8.0)
.keyboard_step(8.0)
.show(ui);

if let Some(next_size) = split.resized(current_size, resize.delta, 120.0) {
    // Store next_size in the application's split model.
}
```

Use a stable ID that is not derived from a translated label. When using an RTL layout, set the
accessible value to the logical leading-pane size and use `ResizeHandle::reversed(true)` when that
value grows opposite to physical divider movement. Invalid geometry or configured values make the
handle inert and report `enabled: false`.

## Toolbar overflow fitting

`fit_toolbar` keeps toolbar groups at their natural widths and returns a leading sequence that fits,
together with the remaining indices for an application-owned overflow menu:

```rust
use faseela_ui::layout::fit_toolbar;

let widths = [56.0, 72.0, 64.0, 48.0];
if let Some(fit) = fit_toolbar(&widths, available_width, 6.0, 40.0) {
    for (index, width) in fit.shown {
        // Render group `index` at its natural `width`.
    }

    if !fit.overflow.is_empty() {
        // Render an enabled, keyboard-reachable overflow menu for every index.
    }
} else {
    // Use an explicit fallback such as a scrollable or wrapping toolbar.
}
```

The algorithm never compresses or skips a group to reveal later groups. It reserves the overflow
control and the spacing before it. The caller must expose every returned overflow index through a
reachable menu; the pure algorithm does not hide commands or create that menu.

For a non-empty input, dimensions must be finite, each group width and overflow-control width must be
positive, and no dimension may exceed one million logical points. Inputs with more than 4096 groups
are rejected. If overflow is required but the control itself cannot fit, the function returns `None`.
An empty input returns an empty fit.

### Ready-made toolbar

`widgets::toolbar_response` builds on `fit_toolbar` and supplies the reachable overflow menu, stable
IDs, direction-aware layout, and state reporting:

```rust
use faseela_ui::widgets::{ToolbarItem, ToolbarState, toolbar_response};

let items = [
    ToolbarItem::new("New"),
    ToolbarItem::new("Open"),
    ToolbarItem::new("Save").disabled(),
];
let result = toolbar_response(ui, egui::Id::unique("main-toolbar"), &items);

match result.state {
    ToolbarState::Empty => {}
    ToolbarState::Fits => {}
    ToolbarState::Overflow => {
        // result.overflow lists the commands in the menu.
    }
    ToolbarState::NoSpace => {
        // Nothing was painted; use your own compact fallback.
    }
}
```

The overflow label uses the actual number of overflowed items (`+3`, `+100`, and beyond) and the
reserved width is measured from that label. Disabled items stay visible, named, and non-activatable in
both the bar and the menu. IDs are stable: visible item `i` is `id.with(("toolbar-item", i))`, menu
item `i` is `id.with(("overflow-item", i))`, and `overflow_button_id` identifies the trigger.

`toolbar(ui, id, items)` remains a compatibility wrapper that returns only the activated index.

## Tab-width fitting

`tabs::fit` shrinks the widest tabs before sending any to overflow. It retains the selected tab and
reports visible `(index, width)` pairs plus overflow indices. `tabs::fit_whole` keeps whole tabs at
natural widths, sends later tabs to overflow, and clips an oversized selected tab to the remaining
budget. Neither algorithm paints tabs or builds a menu.

```rust
use faseela_ui::tabs;

let fit = tabs::fit(&[110.0, 90.0, 80.0], selected, available, 40.0, 48.0);
for (index, width) in fit.shown {
    // Allocate `width` for tab `index`.
}
if !fit.overflow.is_empty() {
    // The application renders the overflow menu.
}
```

## Responsive showcase behavior

The showcase is the integration test for these primitives:

- At **680 logical points and wider**, it uses a sidebar shell. Below 680, it uses a compact header
  with a "Sections" disclosure menu.
- Content uses wrapping rows for buttons, badges, swatches, and metrics.
- The segmented control clamps to the available width and clips labels instead of expanding its
  parent.
- Split preview panes keep physical geometry stable and swap logical pane order in RTL.
- Button groups and breadcrumbs occupy a compact height in tall parents and wrap in narrow parents.
- The toolbar's overflow menu remains reachable at every tested width.
- Progress fills originate at the reading-start edge in RTL; split rectangles remain physical.

Automated showcase tests render all ten sections at **320×568, 360×640, 390×844, 680×800, 768×1024,
1280×800, and 1920×1080**, with all four themes and both directions. They assert that the used drawing
width does not exceed the viewport, and they exercise desktop navigation, the mobile menu, theme
switching, RTL persistence, buttons, forms, and progress controls. These tests are evidence against
horizontal overflow in the showcase, not a universal guarantee for arbitrary application content.

`node tools/browser-smoke.mjs` additionally runs the production WebAssembly in headless Chrome. It
captures a basic representative section set plus all seven CSS viewport sizes, checks the canvas
dimensions against the viewport and detects JS/WASM exceptions. `FASEELA_BROWSER_MATRIX=1` captures
all ten sections × four themes × two directions × seven viewport sizes (560 full-matrix images).
Captures are intended for visual review, not pixel-perfect cross-GPU comparison. Actual native
platform screenshots, Android/iOS, touch devices, and browser screen-reader trees need separate
manual device verification.

## Scope

These helpers intentionally stop before rendering a full docking workspace. Application code remains
responsible for:

- persistent split trees and fixed-versus-relative policy;
- applying returned resize deltas to application state and persisting them;
- pane contents and command dispatch;
- for the pure fitting algorithms, overflow-menu rendering and ensuring all commands remain reachable
  (the `widgets::toolbar` component provides a ready-made reachable menu);
- mirroring logical pane placement for RTL interfaces;
- choosing a fallback when `ToolbarState::NoSpace` occurs.

The geometry, resize, toolbar, and tab algorithms are adapted from
[Craft UI](https://github.com/storytold/craft-libs/tree/main/crates/ui/src/layout) under its MIT OR
Apache-2.0 license. See [UPSTREAM.md](../UPSTREAM.md).
