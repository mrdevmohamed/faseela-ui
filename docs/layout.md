# Layout primitives

The `faseela_ui::layout` module provides small, stateless layout algorithms. It does not own a dock tree, pane state, command model, menu, or pointer gesture.

## Two-pane geometry

`split_rect` divides an egui rectangle into a first pane, divider, and second pane:

```rust
use faseela_ui::layout::{split_rect, SplitAxis, SplitSize};

let split = split_rect(
    allocated_rect,
    SplitAxis::Horizontal,
    SplitSize::Ratio(0.62),
    8.0,  // divider width in logical points
    120.0 // minimum width for each pane when space permits
);

if let Some(split) = split {
    // Paint or lay out content in split.first, split.divider, and split.second.
}
```

Sizing modes are:

- `Ratio(f32)`: fraction of the space after removing the divider, assigned to the first pane.
- `FixedFirst(f32)`: logical-point width or height of the first pane.
- `FixedSecond(f32)`: logical-point width or height of the second pane.

Finite values outside the available range are clamped. When both pane minimums cannot fit, the minimum is relaxed evenly so neither pane is inverted. A zero-sized rectangle is valid. Invalid geometry, negative/non-finite gaps or minimums, and non-finite size requests return `None`.

Horizontal geometry places the first rectangle on the left and the second on the right. In an RTL interface, map your logical leading/trailing pane contents to the opposite rectangles; geometry itself does not make assumptions about application semantics.

### Resizing

The helper returns a proposal rather than changing application state:

```rust
if let Some(next_size) = split.resized(current_size, drag_delta, 160.0) {
    // Persist next_size in your application model.
}
```

The original sizing mode is retained. For example, a ratio remains a ratio and a fixed-second request remains a fixed-second request. The drag minimum can be stricter than the minimum used for the current layout. If there is no resizable space or inputs are invalid, the result is `None`.

The interactive ResizeHandle widget handles pointer dragging, axis-appropriate arrow keys, focus lifecycle, and AccessKit splitter actions. It still does not mutate application state: apply the returned delta through the split layout and persist the resulting SplitSize.

```rust
use egui::Id;
use faseela_ui::layout::{ResizeHandle, ResizeHandleStyle, SplitAxis};

let resize = ResizeHandle::new(
    Id::unique("workspace-split"),
    SplitAxis::Horizontal,
    split.divider,
    "Resize workspace panes",
    ResizeHandleStyle {
        hovered: ui.visuals().widgets.hovered.bg_fill,
        dragged: ui.visuals().widgets.active.bg_fill,
        focused: ui.visuals().selection.bg_fill,
    },
)
.value(split.first_size(), 120.0..=(split.available() - 120.0))
.show(ui);

if let Some(next_size) = split.resized(current_size, resize.delta, 120.0) {
    // Store next_size in the application's split model.
}
```

Use a stable ID that is not derived from a translated label. When using an RTL layout, set the accessible value to the logical leading-pane size and use ResizeHandle::reversed(true) when that value grows opposite to physical divider movement.

## Toolbar overflow fitting

`fit_toolbar` keeps toolbar groups at their natural widths and returns a leading sequence that fits, together with the remaining indices for an application-owned overflow menu:

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

The algorithm never compresses or skips a group to reveal later groups. It reserves the overflow control and the spacing before it. The caller must expose every returned overflow index through a reachable menu; this library does not hide commands or create that menu.

For a non-empty input, dimensions must be finite, each group width and overflow-control width must be positive, and no dimension may exceed one million logical points. Inputs with more than 4096 groups are rejected. If overflow is required but the control itself cannot fit, the function returns `None` so the application can choose an explicit fallback. An empty input returns an empty fit.

## Scope

These helpers intentionally stop before rendering a full docking workspace. Application code remains responsible for:

- persistent split trees and fixed-versus-relative policy;
- applying returned resize deltas to application state and persisting them;
- pane contents and command dispatch;
- for the pure fitting algorithms, overflow-menu rendering and ensuring all commands remain reachable (the `widgets::toolbar` component provides a ready-made reachable menu);
- mirroring logical pane placement for RTL interfaces.

The algorithms are adapted from [Craft UI](https://github.com/storytold/craft-libs/tree/main/crates/ui/src/layout) under its MIT OR Apache-2.0 license. See [UPSTREAM.md](../UPSTREAM.md).
