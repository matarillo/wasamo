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
- [x] Property accessor infrastructure on `WidgetNode`: per-widget
  property ID enumeration + dispatch (Button label/style, Text
  content/style at minimum); `wasamo_get_property` /
  `wasamo_set_property` wired
- [x] Signal / observer registry: token table; `(fn, user_data,
  destroy_fn)` lifecycle; automatic disconnect on widget/window destroy
  and on `wasamo_shutdown`; `wasamo_signal_connect` /
  `wasamo_signal_disconnect`, `wasamo_observe_property` /
  `wasamo_unobserve_property`
- [x] Queued emission machinery: re-entry flag at every public ABI
  entry; emission queue drained on exit; verify no callback fires
  during a `wasamo_*` call on the same thread
- [x] M1 experimental layer: all-at-once widget constructors
  for VStack / HStack / Text / Button (children passed at
  construction; post-construction updates via property R/W);
  `wasamo_button_set_clicked` direct callback; per-widget
  property-ID constants. All marked `WASAMO_EXPERIMENTAL`. See
  `docs/abi_spec.md` §5 / §5.1 for verification scope.
- [x] CI: C smoke test that **compiles and links** a TU including
  `wasamo.h` against `wasamo.lib` (MSVC + Clang)
- [x] `docs/abi_spec.md` finalised to match `wasamo.h`; status updated
  from "initial draft Agreed" to "Agreed"

### Phase 7 — Language bindings

The Phase 7 pre-doc
([docs/decisions/phase-7-language-bindings.md](./docs/decisions/phase-7-language-bindings.md),
DD-P7-001..006, Agreed 2026-04-30) revised the original task list.
Two facts shaped the revision: (1) M1's "C ABI verified in three
languages" criterion is hollow if Rust uses the rlib directly
instead of crossing the C ABI, so a `wasamo-sys` + safe wrapper
pair is required (DD-P7-001); (2) the cdylib's `wasamo` crate name
collides with what the safe wrapper wants to be called, so the
runtime crate is renamed to `wasamo-runtime` while keeping
`wasamo.dll` / `wasamo.dll.lib` filenames stable (DD-P7-002).
Scope is Hello-Counter-minimal, not full ABI coverage (DD-P7-004).
Per session-end agreement, each item below lands as a separate
commit.

- [x] `docs/decisions/phase-7-language-bindings.md` created, owner
  agreement obtained (DD-P7-001..006); ROADMAP task list revised
  alongside ADR
- [x] Workspace: rename runtime crate `wasamo` → `wasamo-runtime`;
  `[lib].name = "wasamo"` keeps `wasamo.dll` / `wasamo.dll.lib`
  filenames stable. Phase 2-5 visual-check examples move with the
  crate; rlib path documented as internal/dev-only in
  `architecture.md`
- [x] `wasamo-sys` crate at `bindings/rust-sys/`: raw `extern "C"`
  declarations matching `wasamo.h`; `build.rs` links
  `wasamo.dll.lib` via `dylib:+verbatim`; coverage scoped to
  Hello-Counter-minimum (observers and generic signal connect
  intentionally omitted per DD-P7-004). CI now sequences
  `cargo build --workspace` (debug) between release build and tests
  so the import lib exists before wasamo-sys's test link step
- [x] `wasamo` (safe wrapper) crate at `bindings/rust/`:
  stable-core surface at crate root (`Runtime`, `Window`, `Widget`,
  `Value`, `OwnedValue`, `Connection`, `Error`); `wasamo::experimental`
  submodule for widget constructors and `on_clicked`. `!Send` handles,
  closure-safe callbacks via trampoline + `destroy_fn` drop hook.
  Known: rlib name collision warning with wasamo-runtime (cargo#6313);
  deferred to post-M1 cleanup
- [x] `bindings/zig/wasamo.zig`: hand-written extern block + Zig-idiomatic
  wrappers (slices, error sets, tagged unions); same module split
  as Rust (`wasamo.experimental`)
- [x] `bindings/c/CMakeLists.txt` template; CI extended to build
  the existing smoke TU through CMake (MSVC generator; Release config)
- [x] `CONTRIBUTING.md` documents how to add a binding (sys/safe
  pair pattern; experimental module convention; expected coverage
  level per phase)
- [x] `docs/architecture.md` bindings section updated: crate layout,
  rlib path documented as internal/dev-only, experimental module
  convention recorded (v0.13)
- [x] CI: CMake build step + Zig install + Zig smoke step; both link
  against `wasamo.dll.lib`

### Phase 8 — Hello Counter sample × 3 languages

The Phase 8 pre-doc
([docs/decisions/phase-8-hello-counter.md](./docs/decisions/phase-8-hello-counter.md),
DD-P8-001..002, Agreed 2026-05-01) reviewed the original task list.
Two decisions shaped the revision: (1) `examples/counter/counter.ui`
exists from Phase 1 and is a reference for the future M2 lowering;
Phase 8 host programs construct the same widget tree imperatively
through the experimental C ABI, which is precisely what M1 exists to
validate (DD-P8-001). (2) `wasamo_set_property` on size-affecting
properties must trigger a re-layout pass — currently only `WM_SIZE`
does; fixed by auto-invalidate inside `set_property`, draining via the
existing queued-emission machinery (DD-P8-002).

- [x] `docs/decisions/phase-8-hello-counter.md` created, owner
  agreement obtained (DD-P8-001..002); ROADMAP task list revised
  alongside ADR
- [x] Runtime: `wasamo_set_property` triggers re-layout for
  size-affecting properties (`TEXT_CONTENT`, `TEXT_STYLE`,
  `BUTTON_LABEL`); layout drain added to queued-emission machinery;
  `architecture.md` §6 updated
- [x] `examples/counter-c/` (`CMakeLists.txt`, `main.c`, `README.md`)
- [x] `examples/counter-rust/` (`Cargo.toml`, `src/main.rs`, `README.md`)
- [x] `examples/counter-zig/` (`build.zig`, `main.zig`, `README.md`)
- [x] Each example README explains: this is the M1 host-imperative
  shape (the C ABI experimental layer); `counter.ui` shows the future
  M2 form; `wasamoc check examples/counter/counter.ui` passes
- [x] CI: build all three counter examples to release-build success
  (run is GUI-only; CI verifies link, not execution)
- [x] `README.md` Quick Start section written (C; links to Rust and Zig examples)
- [x] `abi_spec.md` §2.3 / §4.3: host→runtime string lifetime clarified
- [x] All M1 checklist items above marked complete
- [x] M1 tag `v0.1.0` released, GitHub Releases notes created

---

## M2: Foundation

**Goal:** close the loop on the DSL side — make `.ui` files actually drive the runtime, with reactive state propagation, so Hello Counter in each language is written against the DSL rather than reproducing it by hand through the experimental C ABI.

For full thesis, acceptance criteria, phase breakdown, and risks, see [docs/plans/m2-plan.md](./docs/plans/m2-plan.md).

The Alpha-style feature work originally listed under M2 — Grid layout, the DSL spec public draft, input handling, IME, AccessKit, VS Code extension — has been redistributed across M3–M6 (below). Phase numbering is local to M2 (M2-Phase 1, 2, …); ADR scope is `M<N>-P<n>` from M2 onward (see [docs/decisions/README.md](./docs/decisions/README.md#file-naming)).

## M3: DSL surface

**Thesis:** the DSL is expressive enough to write real layouts, and is published as a stable public draft.

**Acceptance criteria**

- Grid layout primitive
- ScrollView primitive
- List primitive
- DSL specification first public draft (covers M2 + M3 surface; reserves syntax for material — see M4 — without committing to its rendering semantics)

## M4: Interaction stack

**Thesis:** input, multi-window, text input, and accessibility share a focus model; they ship together so the focus model is settled once. Wasamo's identity feature (Mica/Acrylic) becomes demonstrable from this milestone, and the first contributor-facing showcase ships here.

**Acceptance criteria**

- Input handling: keyboard, mouse, touch; focus model and event routing
- Multi-window support (per-window state, cross-window focus). Included pre-1.0 because its ABI implications are cross-cutting and an append-only post-freeze surface cannot accommodate them
- TextField widget (minimum editable text widget; required by IME verification)
- IME via TSF (Japanese / CJK input)
- AccessKit / UIA integration
- Mica / Acrylic root-window backdrop; system accent color follow-through (initial — full theming surface is M5)
- First showcase application — sufficient to demonstrate Wasamo identity for contributor outreach, even if rough around polish-level details

## M5: Identity & tooling

**Thesis:** Wasamo looks like Wasamo by default, and authoring `.ui` is a first-class editor experience.

**Acceptance criteria**

- Full theming surface (light / dark, accent propagation through widgets, type ramp coverage)
- Official widget set (CheckBox, ComboBox, Menu, and the rest beyond TextField)
- VS Code extension (LSP, syntax highlighting, diagnostics). The VS Code work may begin in parallel any time after M3's DSL spec public draft is agreed; M5 is its acceptance gate, not its earliest start

## M6: 1.0 — C ABI stabilization

**Thesis:** the ABI is settled, performance targets are met, a polished showcase ships, and SemVer applies.

**Acceptance criteria**

- C ABI freeze; SemVer applies from this point
- Public backward-compatibility commitment
- Performance targets: <100 ms cold start, <30 MB memory, single-digit-MB binaries
- Polished showcase application (production-grade, distinct from M4's contributor-outreach showcase)
- C / Rust / Zig bindings mature. Swift and Go bindings are out of scope for 1.0; they are welcomed as community-prototyped bindings post-1.0 (see [VISION §11](./VISION.md#11-how-to-contribute))

## Post-1.0

- Hot reload (interpreter mode during development) — feasibility depends on the wasamoc output format chosen in M2-Phase 2
- Higher-level animation DSL (the public property-change animation API deferred from Phase 5; see [DD-V-001](./docs/decisions/vision-m1-acceptance-criteria.md))
- Advanced layout (LazyList, CollectionView)
- System tray and notification integration
- MSIX packaging integration
- Swift / Go bindings (community-maintained)
