# Faseela UI

Reusable Rust UI foundations for desktop applications in the Faseela ecosystem.

The project is based on **egui** and is designed to keep application logic separate from reusable presentation components.

## Current status

This repository is the clean Faseela-owned foundation. The Craft UI extraction is being evaluated and ported in stages; upstream code is not represented as already imported. The first milestone establishes the workspace, palette integration, and a runnable showcase. Selected reusable controls, SVG icon handling, tab sizing and interaction primitives, and explicit LTR/RTL direction helpers are now implemented. Larger docking/layout systems and font infrastructure remain under evaluation.

## Workspace

- `crates/faseela-ui`: shared widgets and design foundations, including buttons, icons, tabs, RTL helpers, and theme tokens.
- `examples/showcase`: executable preview app.

## Live showcase

The interactive Rust/egui showcase is compiled to WebAssembly and published with GitHub Pages:

- https://mrdevmohamed.github.io/faseela-ui/
- Source entry point: `examples/showcase/src/main.rs`
- Browser page: `examples/showcase/index.html`
- Build locally: install the `wasm32-unknown-unknown` target and Trunk, then run `cd examples/showcase && trunk serve`.

The showcase exercises the shared theme, icon button, tabs, selection control, and LTR/RTL direction switch. It uses the same Rust widgets as the native showcase.

## Quick start

```toml
[dependencies]
faseela-ui = { git = "https://github.com/mrdevmohamed/faseela-ui" }
```

```rust
faseela_ui::theme::apply_dark(&ctx);\nfaseela_ui::direction::apply(&ctx, faseela_ui::direction::Direction::Rtl);
```

## Design principles

- Applications own their domain logic, state, and document models.
- Shared UI components expose small composable APIs.
- Faseela palette is a default theme, not a hard-coded restriction.
- Arabic/RTL, keyboard navigation, accessibility, HiDPI, X11, and Wayland are explicit compatibility goals.
- Avoid pulling in unrelated Craft application engines or format libraries.

## Upstream inspiration

The project evaluates reusable UI work from [storytold/craft-libs](https://github.com/storytold/craft-libs), especially its `crates/ui` crate. Any source-derived files will retain their original copyright notices and attribution, and modified files will be marked as modified. See [UPSTREAM.md](UPSTREAM.md).

## License

MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
