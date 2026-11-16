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
