# Faseela UI

Reusable Rust UI foundations for desktop applications in the Faseela ecosystem.

The project is based on **egui** and is designed to keep application logic separate from reusable presentation components.

## Current status

This repository is the clean Faseela-owned foundation. The Craft UI extraction is being evaluated and ported in stages; upstream code is not represented as already imported. The first milestone establishes the workspace, palette integration, and a runnable showcase. Subsequent milestones will bring across selected reusable controls, docking/layout pieces, icon handling, font support, and input-direction support after their dependencies and licenses are verified.

## Workspace

- `crates/faseela-ui`: shared widgets and design foundations.
- `examples/showcase`: executable preview app.

## Quick start

```toml
[dependencies]
faseela-ui = { git = "https://github.com/mrdevmohamed/faseela-ui" }
```

```rust
faseela_ui::theme::apply_dark(&ctx);
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
