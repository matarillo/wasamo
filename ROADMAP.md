# Wasamo Roadmap

Milestones are defined by acceptance criteria, not dates.
Phase task lists are working hypotheses for satisfying those
criteria; they may be revised during a phase's pre-implementation
ADR review (see [docs/decisions/README.md](./docs/decisions/README.md#pre-doc-discipline)).
For the full vision and rationale see [VISION.md](./VISION.md).

---

## M1: Proof of Concept

**Goal:** validate the core hypothesis — external DSL × C ABI × Visual Layer.

**Acceptance criteria**

- VStack / HStack / Text / Button / Rectangle work
- Rendering through the Visual Layer (DWM compositing engaged, visual tree responsive on the compositor thread)
- Minimal C ABI header (`wasamo.h`)
- "Hello Counter" example runs in three languages: C, Rust, and Zig

### Phase 0 — Project skeleton

- [x] `docs/architecture.md` initial draft, owner agreement obtained
- [x] Cargo workspace initialized (`wasamo/`, `wasamoc/`, `bindings/rust/`, `examples/`)
- [x] `wasamo` crate configured as `cdylib` (`wasamo.dll`)
- [x] `windows` crate dependency added with required features
- [x] `.gitignore`, `rust-toolchain.toml` in place
- [x] GitHub Actions CI (`cargo build --release -p wasamo` on Windows runner)
- [x] `docs/architecture.md` updated to match actual workspace
- [x] `ROADMAP.md` created (this file)

### Phase 1 — DSL parser (wasamoc)

- [x] `docs/dsl_spec.md` M1 scope draft, owner agreement obtained
- [x] `docs/architecture.md` wasamoc section added, owner agreement obtained
- [x] Lexer for `.ui` files
- [x] Parser for `.ui` files
- [x] AST type definitions (Rust enum/struct)
- [x] `wasamoc check` CLI command
- [x] `docs/dsl_spec.md` updated to match implementation
- [x] `docs/architecture.md` wasamoc section updated
- [x] CI updated (workspace build + `cargo test --workspace`)
- [x] Unit tests added to `wasamoc` (lexer and parser)

### Phase 2 — Runtime foundation

- [x] `docs/architecture.md` Layer Diagram updated: split App Code into `.ui` DSL layer and host-code layer, clarify each layer's responsibilities
- [x] Visual Layer integration strategy agreed (`docs/architecture.md` updated)
- [x] Win32 window creation and message loop
- [x] `DispatcherQueueController` initialization
- [x] `Compositor` creation
- [x] `DesktopWindowTarget` attaches Visual Layer to HWND
- [x] Root `ContainerVisual` in place
- [x] DLL entry point and basic global state management
- [x] `docs/architecture.md` runtime section updated

### Phase 3 — Layout engine

- [x] `docs/decisions/phase-3-layout-engine.md` created, owner agreement obtained (layout algorithm, VStack/HStack semantics, fill/shrink model, LayoutNode ownership, error handling strategy)
- [x] `LayoutNode` type definition
- [x] `Rectangle` layout
- [x] `VStack` layout (spacing, padding)
- [x] `HStack` layout (spacing, padding)
- [x] Two-pass layout (measure → arrange)
- [x] Layout results applied to `SpriteVisual` offset and size
- [x] Unit tests for layout calculations (measure/arrange, VStack, HStack)
- [x] `docs/decisions/phase-3-layout-engine.md` updated to match implementation
- [x] `docs/architecture.md` layout section updated

### Phase 4 — Widget implementation

- [x] `docs/decisions/phase-4-widget-implementation.md` created, owner agreement obtained
- [x] Text: `IDWriteTextLayout` + `ICompositionDrawingSurface` rendering
- [x] Text: `font` property mapped to Windows type ramp constants
- [x] Button: hit testing (WM_LBUTTONDOWN / WM_LBUTTONUP)
- [x] Button: hover / press visual feedback
- [x] Button: `clicked` callback via C ABI function pointer
- [x] Button: `style: accent` with system accent color
- [x] `docs/decisions/phase-4-widget-implementation.md` updated

### Phase 5 — Compositor independence check

Phase 5 verifies that the Visual Layer is correctly engaged on the DWM
compositor thread. Wasamo's default property-change behavior remains
**instant** (consistent with SwiftUI / Compose / Flutter / CSS); the
public opt-in animation API is deferred to M5. This phase delivers
two things: (1) Button's internal hover/press transition animation
as a permanent product behavior aligned with industry convention
(widgets animating their own state transitions), and (2) a
verification example exhibiting a continuous synthetic visual that
demonstrates compositor-thread independence under app-thread blocking.
See
[docs/decisions/vision-m1-acceptance-criteria.md](./docs/decisions/vision-m1-acceptance-criteria.md)
(DD-V-001) and
[docs/decisions/phase-5-compositor-independence-check.md](./docs/decisions/phase-5-compositor-independence-check.md)
(DD-P5-004..006).

- [x] `docs/decisions/vision-m1-acceptance-criteria.md` created, owner agreement obtained (DD-V-001)
- [x] `docs/decisions/phase-5-implicit-animations-dev-api.md` (DD-P5-001..003) — agreed but **superseded**; pre-doc review found the premise contradicted DD-V-001 (see ADR notes)
- [x] `docs/decisions/phase-5-compositor-independence-check.md` created, owner agreement obtained (DD-P5-004..006)
- [x] Button hover/press brush transition animated with `ColorKeyFrameAnimation` (83 ms hover-in/press-down; 167 ms hover-out/press-up; linear easing; internal Button implementation, no public API). Concrete values recorded in DD-P5-005 post-implementation update.
- [x] `examples/phase5_visual_check.rs`:
  - Existing Button group + a corner `SpriteVisual` with a continuous looping `Vector3KeyFrameAnimation` (~2 s period)
  - 'B' blocks the app thread for ~2 s; the synthetic visual must continue animating during the block
- [x] Minimum runtime hook for the verification example to attach a Visual to the root container (`WindowState::root` public field — no new API surface needed)
- [x] `docs/architecture.md` animation section added (distinguishes widget-internal state-transition animation from the deferred public property-change API; latter belongs to M5)

### Phase 6 — C ABI header

`abi_spec.md` is structured in **two layers**: a **stable core** intended as a
candidate for the M4 ABI freeze, and an **M1 experimental** layer that exists
because M1 `wasamoc` is parser-only and host code must construct the widget
tree imperatively. The split is to avoid letting M1 stopgap shapes leak into
long-term ABI commitments.

The Phase 6 pre-doc explicitly **defers** two questions: (a) where DSL inline
handler bodies (`clicked => { … }`) will execute — host-side vs runtime-side;
(b) wasamoc's M2 output format — host-language codegen vs IR + runtime
interpretation. The stable core is sized so it survives either resolution.

The pre-doc agreement (ADR + abi_spec) surfaced implementation work the
original task list had hidden inside the single line "wasamo.h
implemented" — property R/W dispatch on widgets, a token-based
signal/observer registry, queued emission for re-entrancy, thread-local
last-error storage, and DLL build configuration are all distinct work
items. The checklist below reflects that decomposition.

- [x] `docs/decisions/phase-6-c-abi.md` created, owner agreement obtained
  (DD-P6-001..007: stable-core scope; signal model; callback contract
  incl. destroy_fn / lifetime; threading and re-entrancy; error
  convention; header generation method; DLL boundary contract — export
  macro / calling convention / memory ownership)
- [x] `docs/abi_spec.md` initial draft, owner agreement obtained
  (two-layer: **stable core** + **M1 experimental**, each marked clearly)
- [x] `wasamo` crate `Cargo.toml`: `crate-type = ["cdylib", "rlib"]`;
  `wasamo.dll` + `wasamo.dll.lib` (import library) emit on build
- [x] `wasamo.h` placed at `bindings/c/wasamo.h` with header preamble:
  `WASAMO_EXPORT` / `WASAMO_API` (`__cdecl`) / `WASAMO_EXPERIMENTAL`
  macros, opaque handle typedefs (`WasamoWindow`, `WasamoWidget`)
- [x] Rust-side `#[repr(C)]` types: `WasamoStatus` constants,
  `WasamoValue` tagged union, callback fn-ptr typedefs
  (`WasamoSignalHandlerFn`, `WasamoPropertyObserverFn`,
  `WasamoDestroyFn`)
- [x] Thread-local last-error storage + `wasamo_last_error_message`
- [x] Existing `wasamo_*` (init / window_create / show / destroy / run)
  migrated to `WasamoStatus` + out-param shape; `wasamo_shutdown` and
  `wasamo_quit` added
- [ ] Property accessor infrastructure on `WidgetNode`: per-widget
  property ID enumeration + dispatch (Button label/style, Text
  content/style at minimum); `wasamo_get_property` /
  `wasamo_set_property` wired
- [ ] Signal / observer registry: token table; `(fn, user_data,
  destroy_fn)` lifecycle; automatic disconnect on widget/window destroy
  and on `wasamo_shutdown`; `wasamo_signal_connect` /
  `wasamo_signal_disconnect`, `wasamo_observe_property` /
  `wasamo_unobserve_property`
- [ ] Queued emission machinery: re-entry flag at every public ABI
  entry; emission queue drained on exit; verify no callback fires
  during a `wasamo_*` call on the same thread
- [ ] M1 experimental layer: imperative widget builder for
  VStack/HStack/Text/Button; `wasamo_button_set_clicked` direct
  callback; per-widget property-ID constants. All marked
  `WASAMO_EXPERIMENTAL`.
- [x] CI: C smoke test that **compiles and links** a TU including
  `wasamo.h` against `wasamo.lib` (MSVC + Clang)
- [ ] `docs/abi_spec.md` finalised to match `wasamo.h`; status updated
  from "initial draft Agreed" to "Agreed"

### Phase 7 — Language bindings

- [ ] C: `wasamo.h` + `wasamo.lib` placed in `bindings/c/`, CMake sample
- [ ] Rust: `wasamo-sys` crate with `build.rs` linkage
- [ ] Rust: safe wrapper crate (`wasamo`)
- [ ] Zig: `bindings/zig/wasamo.zig` with `@cImport`
- [ ] `CONTRIBUTING.md` documents how to add a binding
- [ ] `docs/architecture.md` bindings section updated
- [ ] CI: Zig and CMake/C build steps added

### Phase 8 — Hello Counter sample × 3 languages

- [ ] `examples/counter/counter.ui`
- [ ] `examples/counter-c/main.c`
- [ ] `examples/counter-rust/src/main.rs`
- [ ] `examples/counter-zig/main.zig`
- [ ] Each example has a README with build instructions
- [ ] `README.md` Quick Start section written
- [ ] All M1 checklist items above marked complete
- [ ] M1 tag released, GitHub Releases notes created

---

## M2: Alpha

- Major layout primitives (Grid, ScrollView, List)
- Basic input handling (keyboard, mouse, touch)
- IME support via TSF (Japanese input)
- Initial accessibility (AccessKit integration)
- VS Code extension (LSP, syntax highlighting, diagnostics)
- First public draft of the DSL specification

## M3: Beta

- Hot reload (interpreter mode during development)
- Official widget set (TextField, CheckBox, ComboBox, Menu, …)
- Full Mica / Acrylic, theming, and accent color support
- Performance targets met (<100 ms startup, <30 MB memory)
- Comprehensive documentation (tutorials, API reference, samples)

## M4: 1.0 — C ABI stabilization

- C ABI freeze; SemVer applies from this point
- Public backward-compatibility commitment
- Production-grade quality
- Rust / Swift / Zig / Go bindings mature

## M5 and beyond

- Advanced layout (LazyList, CollectionView)
- Higher-level animation DSL
- Multi-window management
- System tray and notification integration
- MSIX packaging integration
