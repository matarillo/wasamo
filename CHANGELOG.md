# Changelog

All notable shipped milestones for Wasamo. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) at
milestone granularity (see
[DD-V-013](./docs/decisions/vision-doc-system.md#dd-v-013--changelog-granularity-and-length-control)).
Per-phase decisions live in
[docs/decisions/](./docs/decisions/); per-release notes live in
[GitHub Releases](https://github.com/matarillo/wasamo/releases).

This file records what has shipped. For what is planned, see
[ROADMAP.md](./ROADMAP.md). For the current state of work, see
the **Status** section of [README.md](./README.md).

## [Unreleased] — M3: DSL surface (in progress)

### M3-Phase 1 — `bool` scalar binding (2026-05-19)

Adds `bool` as the third scalar binding type alongside `i32` and
`String`, discharging M3 acceptance **A9**. The reactive path stays
type-agnostic — `bool` threads through the same `wasamo-ir` ↔
`wasamoc` ↔ `wasamo-runtime` pipeline that `i32` and `String`
already travel, with `Button.enabled` as the live `WidgetNode`
attribute that proves end-to-end propagation. The `TypedValue`
generic value union remains deferred (F5).

`wasamo-ir` gains `IrType::Bool`, `IrLiteral::Bool`, and
`HandlerExpr::{BoolLit, BoolPropRead}` variants. `wasamoc` reserves
`true` / `false` as keywords, type-checks `state` defaults and
property-bind RHS against a soft widget-property catalog
(`Text.text` / `Button.text` / `Button.enabled`), and lowers
state-name idents to typed `*PropRead` variants based on the
declared state type. `wasamo-runtime` adds `PropertyValue::Bool`,
widens `resolve_prop_key` to return `(PropertyKey, IrType)`,
introduces the per-type binding writer seam
(`evaluate_bool_binding` + `widget_write_property_bool` +
`register_bool_binding`), and extends `EvalContext` with
`get_bool` / `read_bool_tracked` / `set_bool`. The C ABI gains
`PropertyValue::Bool` ↔ `WasamoValue::v_bool` conversion arms
across `read_property_value` / `write_property_value` /
`property_value_to_owned` / `owned_to_value` (no new public ABI
functions; the existing `WASAMO_VALUE_BOOL = 3` tag was M2-reserved).

The Phase 1 `Button.enabled` runtime contract is narrow: when
`false`, `hit_test_click_inner` suppresses host callback / inline
`clicked` handler / `enqueue_signal("clicked", …)` while preserving
child hit-test traversal; `update_hover_inner` freezes hover/press
transitions; `effective_button_color` paints a flat grey
`(A=0x40, R=G=B=0x80)` directly (no `ColorKeyFrameAnimation`); the
layout slot is preserved. Focus / a11y / keyboard activation are
deferred to M4–M5.

Visible proof: a new `examples/bool-demo/bool-demo.ui` fixture
(`state ready: bool = true`, `Button.enabled: ready`,
`clicked => { root.ready = false; }`) drives a new
`examples/bool-demo-rust/` host through the build-time `wasamoc`
pipeline (same shape as `counter-rust`), so the M2 Hello Counter
reference stays unmodified. CI coverage: pure-logic unit tests in
`wasamo-ir` / `wasamoc` / `wasamo-runtime`, an IR round-trip
integration test (`wasamoc::emit` → `wasamo-runtime::parse_ir`),
and a Windows-only mock-free integration test
(`button_enabled_property_flips_visual_and_suppresses_click`)
that drives `wasamo_set_property(PROP_BUTTON_ENABLED, …)` against a
live `WidgetNode` and asserts both the `CompositionColorBrush`
colour flip and click-callback suppression.

Per-phase spec sync ([A11](./ROADMAP.md#m3-dsl-surface)):
`docs/dsl_spec.md` 0.4 → 0.5 (§§2.1 / 2.2 / 3 / 4.3 / 4.6 / 4.7 /
4.8 / 5 / 8.2 / 8.4 / 8.6 / 8.9 / 8.12; also folds in a minimal
retroactive `state` surface entry for the M2-Phase 6 documentation
gap, owner-agreed during T10); `docs/architecture.md` §6.8.7
documents the bool path through `register_bool_binding`,
`SignalRegistry::bools`, `widget_write_property_bool`, and the
DD-M3-P1-007 per-type seam.

Decisions: [DD-M3-P1-001..010](./docs/decisions/m3-phase-1-bool-scalar.md).

## [v0.2.0] — 2026-05-11 — M2: Foundation

M2 closes the loop on the DSL side: `.ui` files now drive the runtime
through the agreed `wasamoc` -> IR -> `wasamo_load_ui` pipeline, with
reactive state propagation rather than host-side property-set plumbing.
The milestone discharges acceptance criteria A1-A6: C/Rust/Zig counter
hosts load `counter.ui`, tree-mutation ABI primitives and the cdylib shim
cleanup are in place, and the reactive foundation now has release-mode
ordering, guard-placement, and non-`i32` binding evidence.

### M2-Phase 7 — Reactive Foundation Hardening & Contract Finalization (2026-05-11)

Completes the three deferred Phase 6 DDs that distinguish "the DSL
pipeline runs" from "the DSL pipeline is a Foundation other layers can
rely on." M2 acceptance **A5** and **A6** are discharged.

`dirty_effects` now drains through a dependency-graph topological walk
instead of `EffectId` numeric ordering. The implementation adds explicit
write-edge tracking to `ReactiveGraph`, removes the `sort_unstable()`
production path, and covers chain, diamond, fan-out, and out-of-ID-order
dependency shapes with pure-logic tests. M3 residuals for cycle policy,
ordering ties, and `MUTATION_CAP` interaction are recorded in
`docs/notes/m2-to-m3-handover.md`.

Runtime guard placement is now an accepted architectural invariant:
the ABI boundary owns caller-facing diagnostics, while internal runtime
boundaries protect invariants for entry paths that do not cross the ABI.
`wasamo_run` and `wasamo_quit` now record divergence diagnostics and
return as no-ops after divergence, and focused tests cover
`drain_if_outermost` suppression, re-entrant drain behavior, and cleanup
exceptions.

String-typed property binding is implemented with
`HandlerExpr::StrPropRead`, `EvalContext` String reads, wasamoc lowering
based on declared state type, runtime IR parsing, tracked
`Signal<String>` reads, and regression coverage for the existing integer
binding path. A Windows-only headless integration test proves `.ui`
String binding reaches live `WidgetNode` property state; GitHub Actions
green is evidence that the live path ran rather than skipped.

Decisions: [DD-M2-P6-010..012](./docs/decisions/m2-phase-7-reactive-foundation.md).

### M2-Phase 6 — `.ui` → runtime lowering (2026-05-08)

Closes the loop between the DSL and the runtime: `examples/counter/counter.ui`
now drives the running Hello Counter in C, Rust, and Zig through the agreed
`wasamoc` → IR → `wasamo_load_ui` pipeline, with reactive state propagation
landing through the M2 reactive path (no host-side `wasamo_set_property`).
M2 acceptance **A1** and **A2** discharged.

`wasamoc` extends from M1's parse+check shape with `state` declarations,
restricted type inference (`i32` + string), property-binding lowering,
handler-body lowering, compile-time name resolution, and IR file emission
following the new `;wasamo-ir v0` normative grammar (`docs/dsl_spec.md` §8).
A new `wasamo-ir` crate hosts the shared `IrComponent` / `HandlerExpr`
types so the compiler emits and the runtime consumes the same in-memory
shape. `wasamo-runtime` gains `ir_loader.rs` (parse + defense-in-depth
validation per DD-M2-P6-009 — header/version, reference resolution,
top-level structure; trusts emitter type integrity), the
`wasamo_load_ui(WasamoLoadType, const void*, size_t, WasamoWindow**)`
C ABI entry point with `WASAMO_LOAD_PATH` / `WASAMO_LOAD_MEMORY` modes,
and `wasamo_last_error_message` as the thread-local last-error channel.
Five new error codes ship: `WASAMO_ERR_OBSERVER_MUTATION`,
`WASAMO_ERR_REACTIVE_DIVERGED`, `WASAMO_ERR_REENTRANT_LOAD`,
`WASAMO_ERR_WRONG_THREAD`, `WASAMO_ERR_IR_MALFORMED`.

The reactive drain transaction is rewritten to the three-phase + terminal
form (DD-M2-P6-001 = D, supersedes DD-M2-P5-004's three-stage framing):
Phase 1 mutation convergence loop (FIFO `signal_queue` + topological
`dirty_effects` + last-wins; structure-changing ABI returns
`WASAMO_ERR_REENTRANT_LOAD`), Phase 2 terminal layout pass, Phase 3
post-commit observer drain (state-mutating ABI returns
`WASAMO_ERR_OBSERVER_MUTATION`). `MUTATION_CAP` exhaustion drives an
irreversible `Healthy → Diverged` transition; subsequent ABI calls
return `WASAMO_ERR_REACTIVE_DIVERGED` except `wasamo_runtime_destroy`.
`SignalRegistry { i32s, strings }` (DD-M2-P6-007) replaces Phase 5's
provisional single-type `properties` map; the registration API
(`register_binding(target, expr)`) is preserved.

Documentation: `VISION.md §4 Principle 2` supplement (observer = post-commit
pure effect; mutation = events-up + bindings-down); `docs/architecture.md`
§6.8.3 documents the three-phase + terminal drain; `docs/abi_spec.md`
§3.1/§5.2/§6 cover the new status codes, `wasamo_load_ui`, and thread
affinity rules; `CLAUDE.md` records the build-ordering requirement
(`cargo build -p wasamoc` precedes C/Zig host builds).

The three deferred DDs — DD-M2-P6-010 (dirty_effects topo sort fidelity),
DD-M2-P6-011 (String-typed property binding), DD-M2-P6-012 (re-entrancy /
safety-guard placement principle) — remain `Proposed` and are scoped to
M2-Phase 7 alongside acceptance criteria A5 / A6 (added by the
2026-05-08 plan revision).

Decisions: [DD-M2-P6-001..009](./docs/decisions/m2-phase-6-ui-lowering.md);
DD-M2-P6-010..012 deferred to M2-Phase 7.

### M2-Phase 5 — Reactive engine (2026-05-06)

Implements the M2 thesis-validation surface for acceptance A2:
host `count.set(...)` updates a bound `Text` label without any
host-side `wasamo_set_property` call. Pure-internal Rust; no C ABI
symbol added (per DD-M2-P4-004 = A). `wasamo-runtime/src/reactive.rs`
gains `Signal<T>` + `EffectHandle` + thread-local effect stack +
forward/back dependency edges + dirty-set drain + iteration-cap
divergence trap; `with_batched_writes` body fills the Phase 4
skeleton. The handler evaluator (`HandlerExpr` + `EvalContext`)
is reused via a read-only `BindingEvalContext` that records
Signal reads as dependency edges. `WidgetNode` gains a
`bindings: Vec<EffectHandle>` field disposed at the head of the
Phase 4 `widget_destroy` sweep. `register_binding(target, expr,
write_fn, properties)` is the Phase 6-facing internal API.
`drain_if_outermost` now runs observer drain → reactive drain →
layout drain in one outermost-frame cycle, composing with
DD-P6-003 queued emission and DD-P8-002 layout invalidation. GUI
checkpoint (`exp/m2-p5-reactive-checkpoint`, commit `fdc1545`)
confirms click → label update through the reactive path on real
hardware. A2 fully discharged at Phase 6 close (counter.ui-driven).

Decisions: [DD-M2-P5-001..006](./docs/decisions/m2-phase-5-reactive-engine.md).

### M2-Phase 4 — Tree-mutation ABI primitives (2026-05-05)

Grows the stable C ABI with a sixth area (DD-P6-001 defined the
initial five): index-based widget-tree mutation. New stable-core
symbols: `wasamo_widget_append_child` (promoted from internal),
`wasamo_widget_insert_child`, `wasamo_widget_remove_child`,
`wasamo_widget_replace_child`, `wasamo_widget_child_count`,
`wasamo_widget_destroy`. `WidgetNode` gains an `attached: bool`
invariant maintained by all mutators; `wasamo_widget_destroy` rejects
attached widgets. No host-visible batching API added (DD-M2-P4-004 =
Option A; existing queue-and-drain is the M2 batching contract, now
documented in `abi_spec.md §6`). `reactive.rs` skeleton provides
`with_batched_writes` (internal-only; Phase 5 fills the body).
Acceptance criterion A4 of M2 discharged.

Decisions: [DD-M2-P4-001..004](./docs/decisions/m2-phase-4-tree-mutation-abi.md).

### M2-Phase 1 — cdylib-shim cleanup (2026-05-03)

Resolved the rlib filename collision (cargo#6313) that was worked
around in M1 by dropping `wasamo-runtime`'s rlib. `wasamo-runtime`
is now rlib-only (`[lib].name = "wasamo_runtime"`); a new
`wasamo-dll` cdylib shim depends on it and re-exports all C ABI
symbols via MSVC `/WHOLEARCHIVE`. `wasamo.dll` filename and all 20
`wasamo_*` ABI symbols are preserved. Acceptance criterion A3 of M2
discharged.

Decisions: [DD-M2-P1-001..006](./docs/decisions/m2-phase-1-cdylib-shim.md).

Release: [v0.2.0](https://github.com/matarillo/wasamo/releases/tag/v0.2.0).

---

## [v0.1.0] — 2026-05-01 — M1: Proof of Concept

Validated the core hypothesis: external DSL × C ABI × Visual
Layer. VStack / HStack / Text / Button / Rectangle render through
the Visual Layer with DWM compositor independence verified, the
minimal C ABI (`wasamo.h`) is shaped as a stable core plus an M1
experimental layer, and Hello Counter runs end-to-end in C, Rust,
and Zig (host-imperative; the `.ui → runtime` lowering is M2).

Decisions: Phase 0–8 ADRs in
[docs/decisions/](./docs/decisions/) (`DD-P2-*` … `DD-P8-*`,
`DD-V-001` … `DD-V-004`).
Release: [v0.1.0](https://github.com/matarillo/wasamo/releases/tag/v0.1.0).

## Document system

This project's document conventions changed on 2026-05-02 alongside
M1 shipping. Acceptance criteria live in
[ROADMAP.md](./ROADMAP.md), thesis-level framing in
[VISION.md §7](./VISION.md#7-roadmap), shipped milestones here, and
in-flight work in the active plan under
[docs/plans/](./docs/plans/). Rationale:
[DD-V-010..016](./docs/decisions/vision-doc-system.md).
