# Reusable widgets

All widgets in this crate read from the active design tokens (see [Themes](theme.md)) and return proposals rather than mutating application state. The showcase in `examples/showcase` uses these same public widgets.

## `Card`

A themed surface for grouping content. `Card::new()` uses `surface`; `elevated(true)` uses `elevated_surface`. Both use `border` for the outline. Padding, outer margin, fill, stroke and corner radius can be overridden.

```rust
use faseela_ui::widgets::Card;

Card::new().show(ui, |ui| {
    ui.heading("Details");
    ui.label("Grouped content inherits the active theme.");
});
```

## `SectionHeader`

A title with an optional muted caption, used to label a section without repeating style code.

```rust
use faseela_ui::widgets::SectionHeader;

SectionHeader::new("Appearance").caption("Choose a visual preset").show(ui);
```

## `Badge`

A compact, accessible status label. `BadgeTone::{Neutral, Primary, Success, Warning, Error}` maps to the corresponding token and uses `ThemeTokens::on_color` for readable text.

```rust
use faseela_ui::widgets::{Badge, BadgeTone};

ui.add(Badge::new("Build passed", BadgeTone::Success));
```

## `EmptyState`

A centered message for an empty list, with an optional action button. The action click is returned as a proposal.

```rust
use faseela_ui::widgets::EmptyState;

let response = EmptyState::new("No notifications")
    .description("Enable notifications to see updates here.")
    .action("Enable notifications")
    .show(ui);
if response.inner.action_clicked {
    // Application enables notifications and re-renders the populated state.
}
```

`action_enabled(false)` keeps the action visible and accessible but non-activatable; the returned `action_enabled` mirrors that state.

## `toolbar`

A responsive command bar built on `layout::fit_toolbar`. Items keep natural widths; trailing commands move into a keyboard-reachable overflow menu. Direction follows the context's reading direction, and every command remains reachable.

```rust
use egui::Id;
use faseela_ui::widgets::{ToolbarItem, toolbar};

let items = [
    ToolbarItem::new("New"),
    ToolbarItem::new("Open"),
    ToolbarItem::new("Save").disabled(),
];
if let Some(index) = toolbar(ui, Id::unique("main-toolbar"), &items) {
    // Dispatch items[index]'s command from the application.
}
```

`None` means nothing was clicked; when the width cannot fit even the overflow control nothing is rendered, so the application must provide an explicit fallback (a wrapping row or its own menu).

## `IconButton`

An accessible icon-only button from `buttons`, with caller-supplied icon painting and caller-owned selection. Style states (`normal`, `hovered`, `pressed`, `selected`, `focus_stroke`) are plain structs; build them from tokens via `theme::active_tokens(ui.ctx())`.

```rust
use egui::Sense;
use faseela_ui::buttons::{IconButton, IconButtonStyle};

// `style` built from the active tokens; see examples/showcase/src/main.rs.
let response = IconButton::new("Toggle sidebar", egui::vec2(32.0, 32.0), &style)
    .selected(sidebar_open)
    .show(ui, |ui, rect, tint| {
        ui.painter().circle_filled(rect.center(), 6.0, tint);
    });
if response.clicked() {
    sidebar_open = !sidebar_open;
}
```

## `Tab`

An accessible tab interaction primitive from `tabs`. It does not allocate space; pass the rectangle from the tab-strip layout and a painter. Its accessible role is `Tab`, with a stable label and selected state. Use `Ui::push_id` or a derived `Id` for identity that does not depend on translated labels.

```rust
use faseela_ui::tabs::Tab;

let (rect, _) = ui.allocate_exact_size(egui::vec2(100.0, 32.0), egui::Sense::hover());
let response = Tab::new(id, "Overview", selected).show_at(ui, rect, |ui, response| {
    // Paint using the active tokens and response state.
});
```

Tab-width fitting (`tabs::fit` / `tabs::fit_whole`) remains a separate pure algorithm; see [Layout](layout.md).

## Accessibility notes

See [Accessibility](accessibility.md) for roles, keyboard behavior, focus handling, and tests.
