# Verification, audit, and maintenance notes

## Starting point

The implementation audit started at `c00a7d4066733cd0467128ae7a97c4b75f072963` on `main`. `origin/main`
was fetched before creating `feat/library-quality-completion`; it still pointed to that revision.
Merged PRs #1–#4 supplied the initial Unicode display work, layout algorithms, resize handles and
themed component set. Baseline `fmt`, all 104 library tests, all 11 showcase tests, and Clippy passed.
The original checks did not catch the compact-row geometry bug, duplicate label-derived option IDs,
stale checked state in the activation frame, omitted `changed()` responses, button selection contrast,
incorrect badge galley foreground, or the Arabic font installer replacing application fonts.

## Architecture and preserved boundaries

Private implementations now follow the public component responsibilities. The showcase has a small
platform entry point, `app` owns its state/frame orchestration, `shell.rs` owns responsive chrome,
`navigation.rs` owns section metadata, and each `app/demos/*.rs` owns one demo. Helpers and demo code
stay in the example crate. The library still has no router, command bus, persistence model, global app
state, window/docking controller or application lifecycle API. Existing `faseela_ui::buttons`,
`forms`, `navigation`, `theme`, and `widgets` public paths are kept through their module re-exports.

| Module | Responsibility |
| --- | --- |
| `buttons/{text,icon,style}.rs` | Text/icon interaction, visuals, sizing and shared variant state. |
| `forms/{text,toggles,selection,options,common}.rs` | egui input/combo adapters, caller-owned selection, toggle controls and common option paint/semantics. |
| `navigation/{group,segmented,breadcrumb}.rs` | Independent composable navigation controls; no route ownership. |
| `widgets/{surface,badge,feedback,toolbar}.rs` | Surfaces, semantic badges, progress/status feedback and responsive command presentation. |
| `theme/{presets,tokens,color,fonts}.rs` | Built-in themes, semantic API, contrast/interpolation and additive embedded fonts. |
| `layout/{geometry,resize,toolbar}.rs` | Stateless geometry/fitting and a delta-returning divider. |
| `app/demos/*.rs` | One public-API showcase section per module. |

No new runtime crate dependency was added. The Arabic editing/display boundary is deliberately not
misrepresented; see [Arabic support](arabic-support.md) for upstream feasibility and migration gates.

## Correctness audit implemented

- Default checkbox/switch/radio IDs no longer depend on a translated label. Radio groups preserve an
  out-of-range caller index when disabled or empty, and clamp it only when a nonempty enabled group
  is actually shown. Selection responses report `changed()`; radio/segment navigation consumes keys.
- `support_text` links visible helper/error nodes using AccessKit relationships. Invalid state reaches
  toggle, radio group, combo and text input controls. Checkboxes/switches publish updated values in
  their activation frame. Tabs publish `selected`, icon buttons preserve disabled selection, and the
  segmented group uses its visible accessible name.
- Select text truncates within its available width, emits changed response on selection, and shows a
  visible error border. Button, text field, search field and select allocations are bounded in tiny
  parents. Painter clipping and direction-aware row composition stay local to the components.
- Button/breadcrumb rows no longer expand to the full available parent height. Repeated item identity
  is independent of display labels, and toolbar overflow identity is scoped to its explicit ID.
- egui selected-text foreground uses `on_primary`, not the focus-ring token. Badge shaping and painting
  use the same semantic foreground. Progress grows from the RTL reading start; insets and item counts
  are bounded before geometry/allocation.
- `install_arabic_font` uses egui 0.37 `Context::add_font` at lowest priority to preserve application
  fonts, and skips duplicate insertions in the same egui pass. Bundled Noto OFL, DejaVu, upstream Craft,
  project MIT/Apache notices are included in the package/showcase output.
- The demo's size/icon/tooltip actions report application state; disabled-control and tooltip copy no
  longer overclaim focus/keyboard access. Mobile Escape closes section disclosure and returns focus.
- Browser screenshots caught and fixed the original navigation row expanding in tall showcase cards.
  All showcase sections are tested at seven target sizes across four presets and both directions.

## Evidence, limits, and manual follow-up

Automated evidence in this checkout:

- 111 library unit tests, 12 public-API interaction/accessibility/Unicode integration tests, 16
  rustdoc examples, and 14 showcase tests.
- Automated interaction events cover pointer and keyboard behavior, AccessKit node state, popup
  selection/dismissal, clipboard text and IME commit in egui's test harness. This does not assert
  native window-system focus behavior or native screen-reader announcements.
- The 560-case showcase geometry matrix checks all seven viewports × ten sections × four built-in
  themes × LTR/RTL for bounds, drawing and clipping. Headless Chrome production checks assert actual
  canvas CSS/intrinsic dimensions, page scroll width and absence of JS/WASM exceptions. A 560-image
  screenshot matrix is available for visual review; screenshots are not cross-driver pixel baselines.
- Contrast checks cover the published semantic token pairs and custom-widget paint tests. They are
  not certification of all app combinations, contrast during animation, images, or arbitrary custom
  paint. Disabled contrast and small type need visual/system settings review in consuming apps.

Not verified in this environment: physical native desktop/browser accessibility adapters, NVDA,
VoiceOver, Orca, TalkBack, touch hardware, Android/iOS, unusual IMEs, actual multi-monitor DPI,
Windows/macOS integration. eframe 0.37 web does not publish per-widget AccessKit nodes; this project
does not claim browser screen-reader parity. The public `text::display_text` remains a short-label
workaround, not paragraph BiDi, exact wrapped mixed-script layout, selection/caret, search or editing.

## Reproduce checks

From the repository root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo check -p faseela-ui-showcase --locked
cargo check -p faseela-ui-showcase --locked --target wasm32-unknown-unknown
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo package -p faseela-ui --locked
cargo +1.95.0 check --workspace --all-targets --locked
(cd examples/showcase && trunk build --release --locked)
node tools/browser-smoke.mjs
FASEELA_BROWSER_MATRIX=1 node tools/browser-smoke.mjs
```

The smoke tool needs Node.js 22+ and Chrome/Chromium (`CHROME=/path/to/chrome` overrides the binary).
Screenshots and a geometry/console `results.json` go to `target/browser-smoke` by default, which is
ignored by git. Set an output path as the first script argument to retain captures elsewhere. In CI,
the basic browser captures are uploaded as an artifact; run IDs are available from Actions after the
branch is pushed.

## Changes and compatibility

There are no public signature or module path changes in this refactor. It improves runtime behavior of
existing controls. Application callers depending on `response.changed()`, selected-state exposure in
the same frame, tiny-width clipping, input/helper description relationships or preservation of fonts
will see the intended correction. `ThemeTokens` layout and its preexisting migration note are
unchanged. Index-based IDs continue to mean position; apps needing identity across reordering provide
their own stable namespace/scope. The crate is `0.1.0`, and future breaking changes require release
notes before 1.0.

## Remaining work

1. Wait for or integrate a published egui/epaint release that includes the complete bidi layout,
   caret, selection and grapheme editing upstream efforts; then reevaluate `display_text`.
2. Run manual native/browser screen-reader and touch testing on supported platforms; record backend
   and browser versions.
3. Promote a selected subset of the visual capture matrix to reviewed per-platform golden baselines
   only after choosing stable fonts, GPU/browser and updating policy.
4. Remove pre-1.0 `ThemeTokens` struct-literal migration burden only through an explicitly versioned
   public API change.
