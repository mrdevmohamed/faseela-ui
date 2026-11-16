# Upstream attribution and extraction notes

Candidate upstream: [storytold/craft-libs](https://github.com/storytold/craft-libs), particularly `crates/ui`.

The upstream project declares dual licensing under MIT OR Apache-2.0. Its original license texts and copyright notices must be retained for any files copied or adapted from it. A modified upstream file must carry a prominent notice that it has been changed.

This repository is not a fork and does not imply endorsement by the Craft/ArtCraft contributors. The Faseela-owned code and branding are separate.

## Porting policy

1. Inspect the source file and all transitive local dependencies before copying it.
2. Prefer small reusable UI modules over importing Craft application-specific code.
3. Replace workspace-only dependencies with published crates or Faseela-owned equivalents.
4. Keep original attribution in derived files.
5. Add a compileable example and tests for every imported subsystem.


## Ported components

- `crates/faseela-ui/src/buttons.rs` adapts the icon button implementation from [storytold/craft-libs](https://github.com/storytold/craft-libs/tree/main/crates/ui/src/buttons). The upstream component is distributed under MIT OR Apache-2.0. The port retains the interaction and accessibility model while keeping application assets, styling values, selection state, and command handling owned by Faseela applications. The source file documents the upstream project; ongoing changes are maintained here.

- `crates/faseela-ui/src/icons.rs` adapts Craft's embedded SVG icon catalog. Faseela apps supply their own SVG assets and install egui's SVG loader; this crate only prepares and shares trusted embedded bytes.

- `crates/faseela-ui/src/tabs.rs` adapts Craft's pure tab-width fitting algorithms. It intentionally excludes tab painting, drag interactions, dialog behavior, and application-owned overflow menus so the layout logic remains independent.

- `crates/faseela-ui/src/direction.rs` is a Faseela-owned implementation of reading-direction primitives informed by Craft's `direction` module. It does not import `craft-i18n` or Craft's egui fork. Applications resolve a language preference and explicitly apply the resulting direction to their egui context.
- `crates/faseela-ui/src/tabs.rs` now also includes a caller-painted accessible `Tab` interaction primitive. It intentionally omits Craft's dialog tab style and application-specific docking/drag policy.


## Layout algorithms

- `crates/faseela-ui/src/layout/geometry.rs` adapts the stateless split-rectangle geometry from Craft UI's `crates/ui/src/layout/geometry.rs`. The Faseela module retains the MIT OR Apache-2.0 attribution, but leaves drag gestures, persistence, pane ownership, and direction-to-pane mapping to the application.
- `crates/faseela-ui/src/layout/toolbar.rs` adapts Craft UI's bounded toolbar fitting algorithm from `crates/ui/src/layout/toolbar.rs`. The caller owns the overflow control and must make every returned overflow index reachable; this library does not hide commands or build menus.
- The port intentionally excludes the larger Craft docking model and window policy. Each algorithm has independent boundary and sizing tests and uses only the existing `egui` dependency.


## Arabic text rendering

- `crates/faseela-ui/src/text.rs` is Faseela-owned display logic for mixed RTL/LTR visual ordering and mirroring of paired punctuation. It uses the `unicode-bidi`, `unicode-segmentation`, and `unicode-bidi-mirroring` crates. Arabic letter joining is done by egui's own shaper, not by this crate. Application strings must remain in logical Unicode order; call `text::display_text` only at the rendering boundary.
- `theme::install_arabic_font` installs Noto Sans Arabic before DejaVu Sans for proportional and monospace text. This follows Craft's Noto Sans Arabic-first approach for Arabic UI glyph coverage. Noto Sans Arabic is supplied by `uzor-fonts` (font data under SIL OFL 1.1; see that crate's `NOTICE`); DejaVu Sans is supplied by the `dejavu` crate and its redistribution notices are retained in [LICENSE-DejaVu.txt](LICENSE-DejaVu.txt).
- The Showcase installs the fallback in both native and WebAssembly startup paths and includes an Arabic sample to exercise the rendering path.
