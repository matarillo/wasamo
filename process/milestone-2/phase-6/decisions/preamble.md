# M2-Phase 6 — `.ui` → runtime lowering: Architecture Decisions

**Phase:** M2-Phase 6 (`.ui` → runtime lowering)
**Date:** 2026-05-07
**Status:** Accepted

## Context

M2 acceptance criteria **A1** and **A2** ([m2-plan.md](../plans/m2-plan.md#acceptance-criteria),
mirrored from [ROADMAP.md M2](../../ROADMAP.md#m2-foundation)):

> **A1.** `examples/counter/counter.ui` drives the running Hello
> Counter in C, Rust, and Zig — the M1 host-imperative trees in
> `examples/counter-{c,rust,zig}/` are replaced by hosts that load
> the DSL through the agreed wasamoc pipeline.
>
> **A2.** Reactive state propagation works without host-side
> property-set plumbing: `count++` in the host updates the visible
> label through the M2 reactive path, not through a manual
> `wasamo_set_property` call written by the application.

Phase 5 closed A2 *partially*: the reactive engine is verified through
a runtime-internal spike harness that wires a `Signal<i32>` to a `Text`
widget by hand
([wasamoc/src/main.rs `dump-ir`](../../wasamoc/src/main.rs),
[wasamo-runtime/src/experimental_ir_loader.rs](../../wasamo-runtime/src/experimental_ir_loader.rs)).
Phase 6 closes both A1 and A2 permanently by routing reactive
propagation through the `.ui` source path end-to-end. Every other M2
phase contributed structure (Phase 1 cdylib-shim split), the textual
IR shape (Phase 2), the handler-execution model (Phase 3), the
tree-mutation ABI (Phase 4), or the reactive engine (Phase 5);
Phase 6 is where these meet at the host surface.

### Side obligation carried in

DD-M2-P3-002's closing instruction requires `architecture.md` §6
(or its M2-revised equivalent) to document the **signal-dispatch
ordering runtime contract** during this phase. The drain transaction
DD below (DD-M2-P6-001) supplies the substantive content; the
documentation update lands in the same commit that flips this ADR
to `Status: Accepted`.

### Constraints carried in from prior decisions

- **DD-M2-P2-001 = Option B** (textual IR + runtime interpreter). M2
  output format is textual; this ADR commits to a normative grammar
  (DD-M2-P6-002) and the in-IR shape of expressions (DD-M2-P6-003).
- **DD-M2-P2-002 = Option B** (shipping `wasamoc` for M2). Phase 6
  promotes `wasamoc` from the Phase 2 spike-only state to a tool
  whose output drives the running counter (DD-M2-P6-004).
- **DD-M2-P2-003** enumerates 1–7 candidate `wasamoc` activities
  (parse → check → type inference → property-binding lowering →
  handler-body lowering → IR emit → file write-out). DD-M2-P6-004
  resolves which subset is required for A1.
- **DD-M2-P3-001 = Option A** (runtime-side handler interpreter).
  `HandlerExpr` is the in-runtime AST. DD-M2-P6-003 commits to the
  IR-side serialization of that AST.
- **DD-M2-P3-003** (error-reporting via stderr). DD-M2-P6-005
  decides whether `wasamo_load_ui` extends, replaces, or wraps that
  channel.
- **DD-M2-P4-001..004** (tree-mutation ABI). DD-M2-P6-005's loader
  uses these primitives; no new tree-mutation ABI is introduced
  here.
- **DD-M2-P5-001..006** (reactive engine). DD-M2-P5-005's
  `register_binding(target, HandlerExpr)` is marked provisional
  ("revisited at Phase 6 IR-loader implementation time") for the
  `properties` shape; DD-M2-P6-007 settles it. DD-M2-P5-004's
  three-stage drain framing is partially superseded by
  DD-M2-P6-001.
- **DD-P6-003 = Option A** (queued emission). The "no callback fires
  while the host is inside a `wasamo_*` call" rule is unchanged;
  DD-M2-P6-001 alters drain *contents* and *phase ordering*, not
  the firing-timing contract.
- **VISION §4 Principle 2.** Adoption of DD-M2-P6-001 = Option D
  carries a mandatory supplement to this principle (text in
  §11.1 of this ADR), recorded as a structural constraint rather
  than a convention.

### Pre-doc framing input

The owner-aligned framing of this ADR's slate, scope, and
upstream-document update bundling is recorded in
[docs/notes/m2-phase-6/m2-phase-6-pre-doc-framing.md](../notes/m2-phase-6/m2-phase-6-pre-doc-framing.md).
The drain DD's mature draft analysis is folded into DD-M2-P6-001
below, replacing
[docs/notes/m2-phase-6/dd-m2-p6-drain-transaction.md](../notes/m2-phase-6/dd-m2-p6-drain-transaction.md);
that note is archived together with the ADR's `Accepted` flip.

---

## Summary of proposed decisions

| ID | Topic | Recommendation | Impl risk | Forward-compat exposure |
|---|---|---|---|---|
| DD-M2-P6-001 | Drain transaction semantics | **Option D** — declarative transaction + post-commit pure observer; Phase 1 ordering rules (FIFO + topological + last-wins); MUTATION_CAP exhaustion = terminal error state; Phase 2 strictly read-only; Phase 1 re-entrancy permits state mutation, forbids structure change; VISION §4 P2 supplement mandatory; F as planned M3 extension | Low | Low |
| DD-M2-P6-002 | Normative grammar of textual IR | **Option B** — new normative grammar in `docs/dsl_spec.md`'s IR chapter; mandatory header line | Low–medium | Low |
| DD-M2-P6-003 | IR representation of `HandlerExpr` and bindings | **Option A** — promote tagged-value form; share between bindings and handlers | Low | Low |
| DD-M2-P6-004 | M2 scope of `wasamoc` activities | **Option B** — restricted: parse, check, lower bindings + handlers, emit, write; type inference limited to `i32` + string; `state` declarations in `.ui`; compile-time name resolution (flat namespace, no shadowing) | Low–medium | Low |
| DD-M2-P6-005 | `wasamo_load_ui` C ABI shape | **α + (A)/(C) + (i)** — single function, path or embedded blob, last-error string API; runtime-owned handles for runtime lifetime; UI-thread-confined; `WASAMO_ERR_WRONG_THREAD` on cross-thread call | Low | Low |
| DD-M2-P6-006 | Productionised placement of IR loader | **Option A** — inside `wasamo-runtime`; remove `experimental_ir_loader` flag | Low | Low |
| DD-M2-P6-007 | Final signature of `register_binding` | **Option B** — `SignalRegistry` per-type struct keyed by `wasamoc`-resolved names; supersedes DD-M2-P5-005 provisional `properties` shape only | Low | Low |
| DD-M2-P6-008 | Counter examples migration shape | **α + (X)** — direct ABI calls; shared `examples/counter/counter.ui`; embedded for C/Zig, path for Rust | Low | Low |
| DD-M2-P6-009 | IR loader malformed-input validation policy | **Option C** — defense-in-depth: header/version + reference resolution + top-level structure; trust emitter invariants including type integrity | Low | Low |
| DD-M2-P6-010 | `dirty_effects` topological sort fidelity | Housing migrated to [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-010--dirty_effects-topological-sort-fidelity) | — | — |
| DD-M2-P6-011 | String-typed property binding | Housing migrated to [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-011--string-typed-property-binding) | — | — |
| DD-M2-P6-012 | Re-entrancy and safety-guard placement principle | Housing migrated to [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-012--re-entrancy-and-safety-guard-placement-principle) | — | — |

**Aggregate impl-risk picture.** DD-M2-P6-001 and DD-M2-P6-005
introduce the new ABI-surface error codes M2 ships
(`WASAMO_ERR_OBSERVER_MUTATION`, `WASAMO_ERR_REACTIVE_DIVERGED`,
`WASAMO_ERR_REENTRANT_LOAD`, `WASAMO_ERR_WRONG_THREAD`,
plus DD-M2-P6-009's `WASAMO_ERR_IR_MALFORMED`). All five share
the `wasamo_last_error_message` channel and the TLS infrastructure
that DD-P6-003's queued-emission machinery and DD-P6-001's
observer-callback flag already require; the marginal
implementation cost per code is small. DD-M2-P6-002's grammar
rewrite is the largest *code-volume* change, but it is a
structured rewrite of an existing parser/emitter pair with the
round-trip test from the spike as a regression baseline. Every
other DD recommends an additive or scope-restricting choice;
the M2 delta is concentrated in the drain transaction (with
its operational sub-rules — ordering, divergence semantics,
re-entrancy boundary) and the loader production-ising.

**Aggregate forward-compat exposure.** All nine DDs recommend
the M3-additive option. The named successor work for M3 is:

- DD-M2-P6-001's Option F (post-event API design with concrete
  use cases).
- DD-M2-P6-002's grammar extensions for new binding shapes.
- DD-M2-P6-003's `HandlerExpr` variant additions.
- DD-M2-P6-004's general type inference rule set (paired with
  the M3 DSL spec).
- DD-M2-P6-005's logger callback (iii) and possibly
  resolution-mode (B).
- DD-M2-P6-006's potential split into `wasamo-loader` (paired
  with M5 diagnostic tooling).
- DD-M2-P6-007's `SignalRegistry` field expansion.
- DD-M2-P6-008's idiomatic per-language helpers (paired with
  M3 wrapper-crate API design).
- DD-M2-P6-009's validation-path reuse for hot reload.
- DD-M2-P6-010, 011, 012 — successor work completed in M2-Phase 7; see [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md) for the accepted decisions and per-DD forward-compat treatment.

**Pre-doc validation spike.** Not required for this ADR. The
Phase 2 spike already round-trips the IR through
`experimental_ir_loader`; the Phase 5 reactive engine is
verified pure-logic-side; the C/Rust/Zig binding-author audience
exercise is the GUI checkpoint that closes A1/A2 at Phase 6
implementation completion. The drain transaction's structural
correctness is established by analysis; its operational
behaviour on the counter shape is identical to the Phase 5
single-pass shape (no observer in the M2 acceptance set), so
the regression risk against existing exercise is zero.

## Out of scope

- **Hot reload of the IR.** Post-1.0. DD-M2-P6-002's header,
  DD-M2-P6-005's split-on-demand allowance, DD-M2-P6-006's
  loader colocation, and DD-M2-P6-009's defense-in-depth
  validation collectively keep the M2 architecture amenable;
  no M2 work item enables it. **Assumed model: tear-down +
  full rebuild.** When designed post-1.0, the existing window
  tree, dependency graph, signal_queue, and observer queue are
  atomically destroyed; a new IR is loaded and instantiated.
  State preservation across reload (incremental hot reload) is
  a separate post-1.0 question and is *not* implied by the M2
  choices that keep hot reload amenable. This assumption is
  recorded so the M2 choices (header line, loader placement,
  validation policy, single-thread affinity) can be validated
  against a concrete future shape; the actual hot-reload DD
  will revisit the model and may refine it.
- **Binary IR format.** M2 = textual only (DD-M2-P2-001 = B);
  binary is post-M2.
- **LSP / diagnostics integration.** M5. DD-M2-P6-002's
  grammar-as-spec and DD-M2-P6-005's last-error mechanism are
  the surfaces an M5 LSP attaches to.
- **Resource search paths and bundle systems.** Beyond
  DD-M2-P6-005's recommended (A) and (C), and the deferred
  (B), additional resource-resolution shapes are post-M2.
- **General type inference.** DD-M2-P6-004 = B restricts to
  `i32` and string; M3 takes the general case alongside the
  DSL spec finalisation.
- **`wasamo_post_event` API (Option F).** Not adopted in M2.
  Designed in M3 against concrete observer-trigger use cases.
- **Idiomatic per-language wrapper APIs.** DD-M2-P6-008 = α
  uses direct ABI calls; M3 designs the wrapper crates.
- **Element-identity API
  (`wasamo_find_element_by_id`-style).** Made unnecessary by
  DD-M2-P6-004 = B's choice to put Signal ownership in `.ui`.
- **Logger callback registration ((iii) variant of error
  reporting).** Planned M3 path; DD-M2-P6-005 = (i) ships
  M2.
- **Dependency-cycle visualisation tooling.** DD-M2-P6-001 = D's
  divergence diagnostics emit a structured payload (offending
  Effect ID, iteration count, last-iteration dirty Signal IDs)
  through `wasamo_last_error_message`. Tooling that consumes
  this payload to render the cyclic sub-graph or to time-travel
  into the diverging frame is post-M2; the M2 contract only
  guarantees the raw material is available.

## VISION §4 Principle 2 supplement (mandatory; bundled with Accepted commit)

DD-M2-P6-001 = D's adoption requires the following text appended
to VISION §4 Principle 2 (final wording subject to the same
review pass that flips this ADR to `Accepted`):

> Property observers (host-registered watchers on property
> changes) are post-commit pure effects: they observe a fully
> converged frozen state and perform external side effects
> (logging, telemetry, I/O) without mutating runtime state.
> State mutation **into the runtime** flows exclusively
> through user events (signal handlers) and reactive bindings
> (declarative property bindings). This makes the
> unidirectional model structurally enforced at the runtime
> boundary rather than merely conventional. Host-side state
> external to the runtime may be mutated freely; the
> constraint applies to the runtime's own state — Signals,
> properties, and the dependency graph — and to the channels
> that mutate it.

The supplement is recorded here as inseparable from the DD's
acceptance: choosing D and not writing the supplement leaves
the structural enforcement undocumented at vision level.

## Revisions

- **2026-05-08.** DD-M2-P6-010, 011, 012 housing migrated to
  [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md).
  These DDs were drafted as part of this ADR's slate but never
  Accepted under it; they were carried as `Proposed` and explicitly
  deferred at Phase 6 closing per the 2026-05-08 acceptance-criteria
  revision (recorded in [m2-plan.md](../plans/m2-plan.md)'s Progress
  section, which also added A5/A6 and scoped Phase 7). The migration
  is a *housing move*, not a content rewrite: full DD bodies are
  preserved verbatim in the Phase 7 ADR, with explicit notes where
  Phase 7 pre-doc may revise the inherited recommendation under the
  A5/A6 framing. This ADR retains stub anchors at the original DD
  section locations for inbound link stability. The Phase 6 ADR
  itself remains `Accepted` for DD-M2-P6-001..009; this revisions
  entry does not modify any Accepted decision.
