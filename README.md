# Faseela UI

Reusable Rust UI foundations for desktop applications in the Faseela ecosystem.

The project is based on **egui** and keeps application state and domain logic separate from reusable presentation components.

## Current status

Faseela UI provides a reusable foundation for application chrome: buttons, SVG icon preparation, tabs, themes, explicit LTR/RTL interface helpers, short-label BiDi display support, bounded split geometry, and responsive toolbar fitting. The interactive native/WebAssembly showcase exercises the shared components.

Craft UI functionality is being evaluated and ported in small, independently testable stages. Application-specific docking trees, commands, document models, and persistence remain outside this crate. The current Arabic display helper is transitional and must not be used for editable text.

## Workspace

- `crates/faseela-ui`: shared widgets and design foundations, including buttons, icons, tabs, layout algorithms, RTL helpers, text display helpers, and theme tokens.
- `examples/showcase`: executable native and WebAssembly preview app.

## Live showcase

The interactive Rust/egui showcase is compiled to WebAssembly and published with GitHub Pages:

- https://mrdevmohamed.github.io/faseela-ui/
- Source entry point: `examples/showcase/src/main.rs`
- Browser page: `examples/showcase/index.html`
- Build locally: install the `wasm32-unknown-unknown` target and Trunk, then run `cd examples/showcase && trunk serve`.

The showcase exercises the shared theme, icon button, tabs, selection control, LTR/RTL direction switch, and responsive split layout. It uses the same Rust widgets as the native showcase.

### Arabic and mixed-direction text

Faseela includes a display helper that reorders mixed RTL/LTR labels word by word and mirrors paired punctuation (such as brackets) inside RTL runs, plus a bundled Noto Sans Arabic fallback (with DejaVu Sans as the broad final fallback). Letter joining is left to egui's own shaper, so strings are never converted to presentation forms. Keep source strings in their original logical Unicode order, then shape them only when rendering:

```rust
faseela_ui::theme::install_arabic_font(&ctx);
let label = faseela_ui::text::display_text("مرحبًا بك في Faseela UI");
ui.label(label);
```

The helper is intended for short UI strings, not as a replacement for a full text-layout engine in editors or long-form documents. Do not pass its output to `TextEdit`, store it, search it, or copy it. See [UPSTREAM.md](UPSTREAM.md) for implementation and font attribution notes, and [docs/arabic-support.md](docs/arabic-support.md) for limitations and roadmap.

### Layout primitives

The `layout` module provides bounded two-pane geometry, an interactive splitter with pointer, keyboard, and AccessKit support, and toolbar overflow fitting. Applications still own pane state, commands, and overflow-menu contents. See [docs/layout.md](docs/layout.md) for examples and edge-case behavior.

## Quick start

```toml
[dependencies]
faseela-ui = { git = "https://github.com/mrdevmohamed/faseela-ui" }
```

```rust
faseela_ui::theme::apply_dark(&ctx);
faseela_ui::direction::apply(&ctx, faseela_ui::direction::Direction::Rtl);
```

## Design principles

- Applications own their domain logic, state, commands, and document models.
- Shared UI components expose small composable APIs and return proposals rather than mutating app state.
- Faseela palette is a default theme, not a hard-coded restriction.
- Arabic/RTL, keyboard navigation, accessibility, HiDPI, X11, and Wayland are explicit compatibility goals.
- Avoid pulling in unrelated Craft application engines or format libraries.

## Upstream inspiration

The project evaluates reusable UI work from [storytold/craft-libs](https://github.com/storytold/craft-libs), especially its `crates/ui` crate. Source-derived algorithms retain attribution and are maintained independently in this repository. See [UPSTREAM.md](UPSTREAM.md).

## License

MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
