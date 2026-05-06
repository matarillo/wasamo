# M2-Phase 6 pre-doc framing

**Status:** framing aligned with owner (2026-05-06); input artefact for ADR drafting
**Date:** 2026-05-06
**Targets phase:** M2-Phase 6 (`.ui → runtime` lowering)

Per the project's doc-driven workflow, individual DDs are not
negotiated one-by-one in chat — instead, framing is aligned first,
then the full ADR is drafted in one pass as `Status: Proposed`,
reviewed, and flipped to `Status: Accepted`. This note records the
framing agreement reached with the owner before ADR drafting begins;
it remains as an input artefact and is not promoted into the ADR.

---

## Phase 6 acceptance criteria (restated)

- **A1.** `examples/counter/counter.ui` drives the running Hello
  Counter in C, Rust, and Zig — replacing the M1 host-imperative tree
  construction in `examples/counter-{c,rust,zig}/`.
- **A2 fully discharged.** Reactive propagation verified end-to-end
  through the `.ui` path (Phase 5 closed A2 *partially* via the spike
  harness; Phase 6 closes it permanently).
- **Side obligation.** DD-M2-P3-002 closing instruction:
  `architecture.md` §6 (or its M2-revised section) must document the
  signal-dispatch ordering runtime contract in this phase.

---

## Agreed DD slate (9 entries)

The Phase 6 ADR (`docs/decisions/m2-phase-6-ui-lowering.md`, working
title) will carry the following nine DDs.

### DD-M2-P6-001 — Drain transaction semantics

The draft DD in
[dd-m2-p6-drain-transaction.md](./dd-m2-p6-drain-transaction.md) is
already mature: 4 design axes, 6 options (A–F), drafter's
recommendation Option D (with F as the standard extension path in
M3). This DD is folded into the Phase 6 ADR as its opening entry.

Adopting Option D requires a **mandatory** supplement to VISION §4
Principle 2 (the supplement is *not* optional under the current
draft). The note `dd-m2-p6-drain-transaction.md` is removed or
archived once the ADR carries its content.

### DD-M2-P6-002 — Normative grammar of the textual IR

DD-M2-P2-002 already chose Option B (textual). Phase 6 commits to a
concrete surface form (promote the Phase 2 spike's s-expression form,
design a new one, or pick a third) and a home for the spec (extend
`docs/dsl_spec.md` with an IR chapter, or create a separate
`docs/ir_spec.md`).

**Sub-issue:** header / version contract. Whether the normative
grammar mandates a magic + version line (e.g. `;wasamo-ir v0`) at the
top of every IR file, to enable fail-fast on stale-`wasamoc` /
new-runtime mismatches in post-M2 scenarios. M2 does not require
versioning for correctness (single-workspace co-build), but writing
the contract now is cheap.

### DD-M2-P6-003 — IR representation of `HandlerExpr` and binding expressions

How the in-memory enum from Phase 3 (`HandlerExpr`) and the
binding-evaluator input from Phase 5 serialize into the textual IR.
Either fully promote the tagged-value flavour from the Phase 2 spike,
or replace it with a different scheme.

### DD-M2-P6-004 — M2 scope of `wasamoc` activities

Of the 1–7 activities enumerated in DD-M2-P2-003 (parse → check →
type inference → property-binding lowering → handler-body lowering →
IR emit → file write-out), how much is required to drive the counter
scenario? For example, type inference may be restricted to fixed
`i32` / string only.

**Coupled consequence — must be made explicit in the option
comparison:** whether `.ui` carries `state` declarations (Signal
ownership inside `.ui`) or leaves Signal ownership on the host side
directly determines whether the host needs an element-identity API
(see DD-M2-P6-005). The DD-004 option write-up therefore enumerates
"host-visible identity API requirement" as a consequence of each
option, not just the wasamoc-internal feature cut.

### DD-M2-P6-005 — `wasamo_load_ui` C ABI shape

Single function returning the root, or split loader / instantiate?
Resource resolution and identification of the resulting root when
multiple `.ui` are loaded.

**Sub-issues:**

- **Element-identity API (conditional on DD-004).** If DD-004
  resolves toward "Signal ownership stays host-side", an
  `wasamo_find_element_by_id` style API (or auto-binding scheme) is
  required. If `.ui` owns state, identity may be unnecessary in M2.
- **Error reporting.** Whether `wasamo_load_ui` adopts a last-error
  string API (`wasamo_last_error_message`), continues the
  DD-M2-P3-003 stderr convention only, or registers a logger callback.
  Error path for `WASAMO_ERR_OBSERVER_MUTATION` (DD-001) is
  consolidated here.

### DD-M2-P6-006 — Productionised placement of the IR loader

When the current `experimental_ir_loader` (feature-gated) graduates
to `wasamo-runtime/src/ir_loader.rs`: keep the loader inside the
runtime crate, or split it into a separate `wasamo-loader` crate?
Disposition of the experimental feature flag (delete vs retain).

**Sub-issue: malformed-IR validation policy.** How defensively the
loader treats input. Options under examination:

- (a) **Strict** — every node validated for structure / type /
  reference resolution; any irregularity fails the load.
- (b) **Lenient** — build the tree from whatever parses; warn on
  unknown tags but keep going.
- (c) **Defense-in-depth** — `wasamoc` output is trusted; the loader
  performs lightweight checks but verifies magic / version
  (DD-002 header) and reference resolution strictly.

M2 co-builds `wasamoc` and `wasamo-runtime` in a single workspace,
making (c) the practical shortest path; the choice still needs
recording because it has direct bearing on post-M2 hot-reload
defensiveness. Cross-references DD-005 error-reporting on how
detected errors surface to the host.

### DD-M2-P6-007 — Final signature of `register_binding`

The Phase 5 ADR explicitly marks
`properties: Rc<HashMap<String, Signal<i32>>>` as provisional and
"to be revisited at Phase 6 IR-loader implementation time". Settle
the degree of type erasure and the ownership model on the loader side.

### DD-M2-P6-008 — Migration shape for `examples/counter-{c,rust,zig}`

Per-language wrapper API shape (call `wasamo_load_ui` directly vs
language-specific helper) and `.ui` file location (single shared
`examples/counter/counter.ui` or per-language copies).

**Reframed core option set — resource resolution:**

- (A) **Absolute path only** — host computes the absolute path and
  passes it in.
- (B) **Path relative to the host executable** — runtime resolves
  using executable directory.
- (C) **Compile-time embedded string** — `.ui` content is embedded
  at build time and `wasamo_load_ui` accepts a memory blob, not a
  path.

post-M2 search-path / resource-bundle extensions go to Out of scope.

### Out of scope (to be carried in the ADR's Out-of-scope section)

- Hot reload of the IR (post-1.0).
- Binary IR format (M2 = textual only).
- LSP / diagnostics integration (M5).
- Resource search paths and bundle systems beyond the (A)/(B)/(C)
  selected in DD-008.

---

## Owner-agreed framing decisions (2026-05-06)

- **A. DD slate completeness.** Eight original DDs accepted as the
  cut, plus the malformed-IR validation concern folded into DD-006
  (with a cross-reference from DD-005). Final slate: 9 DDs.

- **B. Drain DD integration.** The existing
  [dd-m2-p6-drain-transaction.md](./dd-m2-p6-drain-transaction.md)
  is folded into the Phase 6 ADR as DD-M2-P6-001 essentially as-is
  (6-option comparison + Option D recommendation + §11 supplements).
  The VISION §4 P2 supplement is treated as inseparable from the ADR
  (Wasamo-identity articulation, not optional polish).

- **C. Pre-doc-discipline check.** The provisional task list
  ("restricted-scope `wasamoc` + full-feature IR loader") is the
  shortest sound path to A1/A2. Cutting the loader to Counter-only
  hardcoded behaviour would explode tech debt at M3; restricting
  `wasamoc` features (e.g. type inference) while keeping the IR
  *normative* (DD-002) is the agreed balance.

- **D. Upstream-document revision timing.** The VISION §4 P2
  supplement, the DD-M2-P5-004 supersede update, the
  `architecture.md` §6.8 drain-ordering revision, and the archival
  of `dd-m2-p6-drain-transaction.md` are bundled into the **same
  commit that flips the ADR to `Accepted`**. Implementation begins
  only after that commit lands, so structural constraints (Option D)
  are review-ready as code-review rulers.

---

## Next session — handoff

Inputs are complete. The next session begins ADR drafting:

1. Create `docs/decisions/m2-phase-6-ui-lowering.md` (working title)
   as `Status: Proposed`, carrying the 9 DDs above with full Option
   tables, Recommendation prose, and the two-axis risk/exposure
   evaluation per DD (per
   [docs/decisions/README.md](../decisions/README.md#risk-evaluation)).
2. Owner review pass.
3. On `Status: Accepted` flip, the upstream document edits enumerated
   under decision D above are bundled into the same commit.
