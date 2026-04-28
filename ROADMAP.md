# Wasamo Roadmap

Milestones are defined by acceptance criteria, not dates.
For the full vision and rationale see [VISION.md](./VISION.md).

---

## M1: Proof of Concept

**Goal:** validate the core hypothesis — external DSL × C ABI × Visual Layer.

**Acceptance criteria**

- VStack / HStack / Text / Button / Rectangle work
- Rendering through Visual Layer with implicit animations
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

- [ ] `docs/rfcs/0001-layout-model.md` created, owner agreement obtained
- [ ] `LayoutNode` type definition
- [ ] `Rectangle` layout
- [ ] `VStack` layout (spacing, padding)
- [ ] `HStack` layout (spacing, padding)
- [ ] Two-pass layout (measure → arrange)
- [ ] Layout results applied to `SpriteVisual` offset and size
- [ ] Unit tests for layout calculations (measure/arrange, VStack, HStack)
- [ ] `docs/rfcs/0001-layout-model.md` updated to match implementation
- [ ] `docs/architecture.md` layout section updated

### Phase 4 — Widget implementation

- [ ] `docs/rfcs/0002-widget-model.md` created, owner agreement obtained
- [ ] Text: `IDWriteTextLayout` + `ICompositionDrawingSurface` rendering
- [ ] Text: `font` property mapped to Windows type ramp constants
- [ ] Button: hit testing (WM_LBUTTONDOWN / WM_LBUTTONUP)
- [ ] Button: hover / press visual feedback
- [ ] Button: `clicked` callback via C ABI function pointer
- [ ] Button: `style: accent` with system accent color
- [ ] Unit tests for hit-testing coordinate logic
- [ ] `docs/rfcs/0002-widget-model.md` updated

### Phase 5 — Implicit animations

- [ ] `ImplicitAnimationCollection` on each `SpriteVisual`
  - Offset: `Vector3KeyFrameAnimation` (150 ms, cubic-ease)
  - Size: `Vector2KeyFrameAnimation` (150 ms)
  - Opacity: `ScalarKeyFrameAnimation` (100 ms)
- [ ] Animation setup centralized in a runtime helper
- [ ] `docs/architecture.md` animation section added

### Phase 6 — C ABI header

- [ ] `docs/abi_spec.md` initial draft, owner agreement obtained
- [ ] `wasamo.h` implemented (init / window / widget / property / hierarchy APIs)
- [ ] All public functions carry `WASAMO_EXPORT`
- [ ] Opaque pointer types (`WasamoWindow*`, `WasamoWidget*`)
- [ ] `docs/abi_spec.md` finalized to match `wasamo.h`
- [ ] CI: C header compilation smoke test added (`wasamo.h` compiles with MSVC/Clang)

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
