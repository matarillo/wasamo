---
title: M4-Phase 1 handoff
status: skeleton
source-phase: M4-Phase 1
---

# M4-Phase 1 — Handoff

> **Status: skeleton.** Finalized at phase close, distilled from the
> T12 carry-forward ledger in [log.md](./log.md), per
> [workflow.md](../../../procedures/workflow.md) and
> [retrospectives.md](../../../procedures/retrospectives.md). The
> sections below are the shape the close will fill, not claims.

## Main learnings

<!-- Filled at phase close. -->

## Carry-forward to later phases

Known at planning time from the ADR's forward-compat exposure; each is
re-confirmed or revised at phase close, and each carries a re-trigger
criterion rather than a date.

| Item | Lands at | Re-trigger criterion |
|---|---|---|
| Layout-derived hit rectangles — stop reading geometry back off the Visual; cache each node's arranged DIP rectangle during layout | M4-Phase 2 | The event-routing model needing layout-derived hit rectangles, or a DIP-denominated minimum hit target. Both expected in that phase. The T5 pointer / readback conversions cancel today precisely because hit-testing sources geometry from the visual tree; they stop cancelling here |
| Host-visible scale or work-area query | M4-Phase 7 ABI wave, or M4-Phase 8 with `WindowConfig` | A host must express or receive a length not expressible in DIP — a device-pixel budget, a monitor work area, a screen coordinate. Not retrofitted into this phase, and not pre-built on a prediction |
| Per-window differing scale factors | M4-Phase 8 | Additive by construction — the scale is already per `WindowState`. Confirm no shared state crept in |
| Client-size window semantics | M4-Phase 8 / AC11 | Arrives as a new named attribute, never as a reinterpretation of `width` / `height` |
| Screen-coordinate mapping (IME caret and composition rectangles; top-layer placement) | M4-Phase 5 / 6, M4-Phase 9 | Lands as `visual absolute physical → ClientToScreen` with **no** scale multiplication, because the visual tree is in the space the Win32 call expects. The concrete downstream payoff of keeping the visual tree physical |
| Resolution-dependent image asset selection | M4-Phase 4 | The second rasterized asset kind arrives on the same surface-resolution contract |
| Integer pixel snapping | Deferred | Would extend the rounding contract inside `DipScale` rather than change the space definition |
| Text rendering-quality tuning (rendering mode, gamma, explicit hinting) | M5 theming wave | This phase's obligation ends at "drawn at the correct resolution" |
| Custom title bar / client-area frame extension | M5 theming wave | Would make V2's automatic non-client scaling scale a surface Wasamo also paints; the full reliance must then be re-decided |
| Non-zero clip insets | Whenever introduced | Clip insets are exempt from conversion only because they are all zero. A non-zero inset puts that audit row back into the converted set |
| A scale-dependent `measure` (explicit hinting, snapped metrics) | M5 text-quality wave | Would turn T7's re-rasterize-after-re-layout ordering from a free choice into a correctness constraint; the reason is recorded so it can be re-derived rather than rediscovered |
| Phantom-typed length newtypes (`Dip<T>` / `Px<T>`) | Available, not scheduled | Adopted only if a unit-mixing defect actually recurs — not on a prediction that it might |
| `WM_GETDPISCALEDSIZE` | Available, not scheduled | A phase wanting to propose its own post-change window size (author-specified sizing, AC11 / M4-Phase 8) |
| Synthesised pointer update after a scale change | M4-Phase 2 event model | If hover correctness across a resize turns out to matter |
| **The per-node scale cache has exactly one writer** — the attach / scale-change walk; `WindowState` holds the authoritative value (T1 decision, [log.md](./log.md) §T1) | In force from T5 onward | Any path that attaches, re-parents, or re-materialises a subtree **without** running the walk leaves stale scales behind: staged iteration subtrees, M4-Phase 2 event-model tree edits, M4-Phase 8 moving a tree between windows. Such a path must call the walk |
| **`cargo test --workspace` needs `cargo build -p wasamo-runtime` first from a cold target directory** — `wasamo-dll/build.rs` whole-archives the *uplifted* `<profile>/libwasamo_runtime.rlib`, which cargo produces only once `wasamo-runtime` has been built as a primary package (T1 finding F-5; pre-existing, not introduced by this phase) | T12's clean-rebuild gate; [AGENTS.md §Build ordering](../../../../AGENTS.md) | Any clean rebuild, any CI cache miss, or any toolchain update that invalidates the uplifted rlib. Cold-directory failure is `LNK1356`; a stale uplifted rlib fails later as `LNK2019` on `core` / `std` symbols. `cargo check` never links and stays green through both |
| **The same whole-archive path also fails *quietly*: a host-package build relinks `wasamo.dll` around the stale uplifted rlib** — cargo recompiles `wasamo-runtime` as a dependency and relinks the DLL, but refreshes `<profile>/libwasamo_runtime.rlib` only on a primary-package build, so the fresh DLL carries the previous runtime (T3 finding F-21, the row above's root cause with the opposite symptom; pre-existing, not introduced by this phase) | Every GUI evidence gate — T6, T9, T10 — and [AGENTS.md §Build ordering](../../../../AGENTS.md) at T12 | Any capture, smoke, or host run intended as evidence for a runtime change. Fails **silently and green, with a fresh DLL timestamp**, so a freshness check does not detect it. Precede every capture with `cargo build --release --workspace` |
| **`emit::flush_layout` uses the wrong layout entry** — `window::set_root` and the `WM_SIZE` arm call `run_layout_as_window_root` (root forced to `Fill` / `Fill`); the reactive drain's layout phase calls the plain `run_layout`, so a root `VStack` holding a `Fill` child lays out correctly on resize and **collapses that child on any property write**. The M3-Phase 4 T6 failure, still live on the drain path. Found incidentally by T3's evidence UI; pre-existing, not introduced by this phase | Recommended: **T5**, which already edits that call site for audit row 2b — as its own commit with its own before/after frames | Any re-layout triggered by a property write rather than a resize, on a tree whose root container is `Shrink` with a `Fill` descendant |
| **Button-family widgets render no label without a layout pass**, and `lib.rs::window_add_widget` deliberately runs none — so that path shows a Button's background and not its label (T3, Codex review finding R-1) | Documented at T3 as a stated limit; **`window_add_widget` itself is a cleanup candidate** — a caller-less public entry left behind when `window_set_root` superseded it at `163067a` | Any new attach path that puts a widget on screen **as content** without a layout pass. The label geometry now lives only in `sync_visuals`, so "attached" and "laid out" stopped being separable for Button-family widgets |
| **Every Composition geometry write in the runtime happens in exactly one pass** (`sync_visuals`) — the property that makes DD-002's conversion-site audit complete rather than approximately complete (T3) | In force from T3 onward | Any task adding a `SetOffset` / `SetSize` / `SetScale` outside `sync_visuals` — a constructor, a property setter, or T6's re-rasterization walk — breaks it silently and reintroduces exactly the class DD-002 §Row 6 detail closed |
| **`ButtonData.label_size` has exactly two writers**, both of which also write `label_text` and the node's `SizeConstraint::Fixed` pair (T3) | In force from T3 onward | A third path that changes a Button-family label — a typed property writer, an iteration-materialised label rebind, M4-Phase 2's event model — must write all three, or the label renders at the previous text's extent |

## Residuals

<!-- Filled at phase close: anything left undone, with its reason. -->

## Verification residue

<!-- Filled at phase close: what the phase's evidence does and does not
     establish, including the synthesised-message limit and the
     trap-#4 disposition. -->
