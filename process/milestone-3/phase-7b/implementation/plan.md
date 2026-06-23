## Task list

Phase 7b is a corrective phase: it ships **one placement surface**
(`slot.*`) and **one internal model** (child-slot `SlotData`) across
every side (A11), with no new layout primitive. The work is an IR
carrier migration (T2), a runtime storage migration (T3), an
author-surface flip (T4), GUI evidence (T5–T6), and the close gates +
Moment 2 (T7) — preceded by a pre-implementation spike (T1) that
compiler-verifies the migration and fixes the bisectable sequencing.
The final-task ownership split
([preamble.md §Step-end / phase-end retrospective split](./preamble.md#step-end--phase-end-retrospective-split-final-task-ownership))
is represented in T7 from the start.

Default to **one commit per task-list item** per
[AGENTS.md §Commit rules](../../../../AGENTS.md#commit-rules). The known
exceptions this phase are:

- **T2** — the IR carrier change breaks `wasamo-ir`, `wasamoc`
  (lower / emit / check), and the runtime loader simultaneously
  (preamble risk R-A), so it bundles into one buildable commit.
- **T4** — the ZStack bare-alignment reject breaks every in-repo `.ui`
  using it (preamble risk R-C), so the author-surface flip and the
  in-repo `.ui` migration land together to keep the build green.

If implementation reveals an item should split or reorder, revise this
list so it stays an accurate record rather than a frozen prediction
(plan changes mid-implementation are normal). **Sub-task lists below are
planning-time hypotheses**, not frozen contracts — T1 may re-cut them
against the source, and any task may revise its own sub-list as work
surfaces.

Each task runs the implementation gates at **start** (record the trap
selection + review lane in [log.md](./log.md) before choosing an
approach) and **close** (the auditable artifacts), per
[implementation-gates.md](../../../procedures/implementation-gates.md).

---

### T0 — Moment 1 closure + implementation docs open

Opens execution after ADR acceptance. Moment 1 (the design-spec draft)
is already largely landed; T0 confirms the commit set is complete and
lands the implementation docs. Implementation (T1) begins only after T0
closes.

- [x] ADR set `Status: Accepted` (both DDs flipped 2026-06-21 after the
      PM-2 integration review).
- [x] `docs/dsl_spec.md` §4.16 placement chapter + §8.5 / §8.11
      supporting additions (v1.10); `docs/architecture.md` §6.8.6
      `SlotData` storage + §6.8.4 Grid (SM-B) — Moment 1 normative spec
      body landed (commit `4e5312c`).
- [x] `process/_roadmap.md` A13 + `process/milestone-3/plan.md` Phase 7b
      row + Revision log (branch (a)) landed.
- [x] `process/milestone-3/phase-7b/implementation/preamble.md` and this
      `plan.md` opened with `status: active`; skeleton `log.md` /
      `handoff.md` opened (owner review passed, Codex 2-pass folds;
      commit `cdc735c`, merged onto `feat/m3-phase-7b` at `5c4c312`).
- [x] `docs/notes/architectural-family.md` FD-7b-C confirm-within-family
      (1) entry landed revise-in-place at **Moment 1** (commit
      `0864c4a`); no Moment 2 carry needed.

**Start gate:** none (doc-only). **End gate (closed):** the
implementation docs are on the branch and the Moment 1 commit set is
complete; T1 may open.

---

### T1 — Pre-implementation spike: carrier shape, sequencing, call-site recon

Discharges ADR obligations 1 and 2
([preamble.md §Obligations](./preamble.md#obligations-carried-from-the-adr-represented-in-this-plan-from-the-start)).
**No production code lands**; the compiler-verification edit is
throwaway and must be reverted before T1 closes. T1's landing artifacts
are recorded decisions in [log.md](./log.md) plus any revision of this
plan. This is a risk-mitigation spike for R-A / R-B / R-D / R-E (the
migration's compile surface and the parser's dotted-key seam), not the
first slice of the migration itself.

T1 owns only the implementation-recon boundary: the carrier spelling the
DDs left as an implementer recommendation, the source call-site map, the
bisectable sequencing / seams, and the downstream owner assignment for
each open point. If recon shows that the default T2 → T3 → T4 sequence
cannot keep the workspace buildable, T1 revises the task split before
any migration code lands.

- [x] **Read every landing file end-to-end** (not grep-sample), per the
      [spike discipline](../../../procedures/implementation-gates.md):
      `wasamo-ir/src/lib.rs` (`IrMember` / `IrNode` / `KindPayload`),
      `wasamo-runtime/src/widget.rs` (`zstack_placement`, `WidgetData::Grid`,
      `insert_child_inner` / `remove_child` / `replace_child`, `build_layout_tree`),
      `wasamo-runtime/src/layout.rs` (`ZStackPlacement` / `CellPlacement` /
      `LayoutNode::grid` / `arrange_grid`), `wasamo-runtime/src/ir_loader.rs`
      (Grid `Cell` `IrProp` extraction, ZStack placement read),
      `wasamoc/src/{lexer,parser,ast,check,lower,emit}.rs` (current
      `Cell` and bare `h-align` / `v-align` handling). **Include
      `ast.rs`** — a child attribute is today an AST
      `Member::PropertyBind { name, value, span }`
      ([ast.rs:227](../../../../wasamoc/src/ast.rs#L227)), so **how the
      `slot.` dotted key is stored on the AST is a T1 decision** (the
      canonical key `slot.h-align` inside `PropertyBind.name`, vs a new
      placement-bind variant); record it in [log.md](./log.md) as it
      drives the parser / check / lower shape. Record the per-file
      placement touch-points.
- [x] **Compiler-verify the carrier migration** — introduce a throwaway
      `IrMember::Widget { node, slot_data }` (struct variant) +
      `IrSlotData` and a runtime `SlotData` skeleton, `cargo build` the
      workspace to **enumerate every breaking call-site by compiler
      error**, record the site list, then **revert** (no production code
      from the spike). This was a compile-surface probe only; the T2
      target spelling is the `IrChildSlot` wrapper below. This is the
      trap-#1 pre-audit; the authoritative audit table is T2/T3's close
      artifact.
- [x] **Fix and record the carrier in-memory / IR spelling** the DDs
      left as an implementer recommendation (DD-002 SI-1 / SI-3): the
      Rust `IrChildSlot` wrapper + `IrSlotData` shape, the runtime
      `SlotData::{ Grid(..), ZStack(..) }` shape, and the IR-B textual
      skeleton field mapping (`placement <kind> { … }`). The
      **DD-fixed** parts (`SlotData` broad name; VS-1a closed enum;
      reject + regenerate; the normative IR-B keywords / nesting) are not
      reopened.
- [x] **Fix and record the bisectable sequencing** (T2 → T3 → T4) and
      the inter-task seams: **Seam A** — T2's loader converts IR
      `slot_data` into the *legacy* runtime storage (`zstack_placement`
      + `cell_placements`) so the workspace builds before T3; **Seam B** —
      T3 removes the adapter and the loader feeds `SlotData` directly;
      **Seam C** — T4's `slot.` surface lowers into the T2 IR carrier and
      flips the in-repo `.ui` files in the same commit. Revise this plan
      if the default order changes (e.g. if Seam A's adapter is not
      cleanly separable and T2/T3 must bundle).
- [x] **Sharpen the preamble §Technical risks table** against the
      current source (pin file/line hotspots for R-A / R-B / R-E);
      record the **T2 impl-gates selection** (review lane + applicable
      traps with reasons for non-applicable ones) before T2 opens.

**Start gate:** read this plan + the ADR set + the spike-discipline
gate; check [log.md](./log.md) for prior carry-over; then record T1's
own implementation-gate selection, review lane, planned proof
obligations, and known carry-forward candidates in [log.md](./log.md)
before the throwaway carrier edit. **End gate (spike-specific):** every
open point is **assigned to a downstream task and its scope is seen** —
not "no surprises expected"; the carrier spelling, sequencing, seams,
and call-site list are recorded in [log.md](./log.md); the throwaway
migration is reverted (no production code on the T1 commit).

---

### T2 — IR + textual-IR migration (IR-2 / IR-B)

The **schema / IR-migration full-review-lane** task (gates trap #1 +
trap #4 for the loader rejects; risk R-A / R-F). Lands the IR carrier +
emit + loader parse + stale-form reject as one buildable commit. At T2
close the loader still feeds the **legacy** runtime storage via the Seam
A adapter (T3 removes it) — so T2 is internally bisectable and the
runtime behaviour is unchanged. T2's responsibility boundary is the
IR/textual-IR contract plus the temporary adapter: it normalises the
existing authored Grid `Cell` and ZStack bare-placement surfaces into
`IrChildSlot.slot_data`, emits only the canonical `child { placement ...
node ... }` textual IR, parses that canonical form, and rejects stale
old-form textual IR. It does **not** delete `WidgetData::Grid.cell_placements`,
`WidgetNode.zstack_placement`, or any layout mirror; those drift-closing
structural changes are T3.

- [x] `wasamo-ir`: `IrMember::Widget(IrNode)` →
      `IrMember::Widget(IrChildSlot)`, with
      `IrChildSlot { node: IrNode, slot_data: Option<IrSlotData> }`.
      Add the `IrSlotData` carrier (closed Grid / ZStack payload per
      VS-1a, broad name mirroring runtime `SlotData`). `KindPayload::Grid`
      (track lists) stays; only per-child placement moves to
      `slot_data`.
- [x] IR Rust spelling tradeoff recorded for review:
      `IrMember::Widget(IrChildSlot)` is preferred over
      `IrMember::Widget { node, slot_data }`.
      Pros: it makes the IR child-slot record first-class like the
      runtime/layout `ChildSlot` records; keeps slot-local future fields
      (`key`, lifecycle metadata) off the enum variant; and gives helper
      APIs a single slot object to pass when placement is relevant.
      Cons: it adds one named wrapper type and slightly more pattern-match
      churn than a struct variant; placement-insensitive callers must
      consciously unwrap `slot.node`.
- [x] Migrate every construction / match site so the workspace builds:
      `wasamoc` lower routes the **existing** authored placement (Grid
      `Cell` extraction, ZStack bare `h-align` / `v-align`) into
      `slot_data` (the new `slot.*` author surface is T4 — T2 keeps the
      existing surface working, only re-routing where it *lowers to*);
      `wasamoc` emit serialises the IR-B `child { placement <kind> { … }
      node … }` skeleton; the runtime loader parses IR-B and (Seam A)
      converts `slot_data` back into `zstack_placement` /
      `cell_placements` for the unchanged runtime.
- [x] Runtime validation stays intentionally transitional: Grid is
      validated against canonical child-slot records by reading
      `IrSlotData::Grid`, then the Seam A builder derives the legacy
      `cell_placements` vector from the same slots; ZStack placement is
      likewise read from `IrSlotData::ZStack` and bridged to the existing
      `insert_child_with_zstack_placement` path. Any child slot whose
      placement kind is invalid for its immediate parent is rejected by
      the loader. The absence of parallel-vector deletion is a named T3
      carry-forward, not a T2 close condition.
- [x] Loader **reject + regenerate**: stale old-form placement IR (a
      `Cell` node with placement `IrProp`s, or bare ZStack placement
      props on a child) is a **named loader diagnostic**
      (`legacy-placement-ir-form`-style), not silently slot-ised. Direct
      firing test (trap #4).
- [x] IR-type unit tests cover the `slot_data` encoding and the IR-B
      roundtrip at the **IR level** — construct `IrSlotData` directly and
      assert emit → load preserves it for the Grid and ZStack payloads;
      placement defaults preserved. (Tests that the **authored** direct
      `slot.*` form lowers to this record are **T4's** — that surface does
      not exist until T4, so it is not auditable as a T2 close-gate item.
      T2's roundtrip is exercised via the existing Grid `Cell` / ZStack
      lowering and direct IR construction.)
- [x] **Close artifact (trap #1):** the `rg`-enumerated call-site audit
      table over `IrMember` / `IrNode` placement extraction / emit /
      loader, each site classified (extended / correctly unaffected /
      deliberately rejects), `IrNode::widget_children()` and every
      widget-only filter explicitly classified.

**Sub-task hypothesis:** (a) `IrChildSlot` + `IrSlotData`; (b) lower
re-route + emit IR-B; (c) loader parse + Seam A adapter; (d) stale-form
reject + roundtrip tests. T1 may merge/split these.

**Start gate:** T1 recon + T2 trap selection recorded. **End gate:**
workspace builds + all tests green; trap-#1 audit table + trap-#4 reject
test recorded; **full independent review** before merge.

---

### T3 — Runtime `SlotData` storage migration (IM-4 phase-wide / SM-B)

The **runtime-structural full-review-lane** task (gates traps #1 / #2 /
#3; risk R-B / R-G). Converges both containers onto the child-slot
`SlotData` model and removes the last parallel placement vector. T3 is
not a parser / author-surface task: T2 already made the textual IR carry
`IrChildSlot.slot_data`; T3 removes the Seam A runtime adapter and makes
runtime + layout child lists themselves carry the placement record.

- [x] Introduce an explicit runtime child-slot record (recommended local
      type name: `ChildSlot`, not ADR-fixed) so `WidgetNode.children`
      stores `{ node: Box<WidgetNode>, slot_data: Option<SlotData> }`
      records instead of bare child nodes. `SlotData` remains the VS-1a
      closed enum `SlotData::{ Grid(..), ZStack(..) }`; this replaces
      `WidgetNode.zstack_placement` without making placement an intrinsic
      child-widget field.
- [x] **SM-B: migrate Grid** — remove `WidgetData::Grid.cell_placements`
      (the parallel vector); Grid per-child placement rides
      `SlotData::Grid` on the runtime child-slot record. The loader
      converts T2's already-canonical `IrSlotData::Grid` into runtime
      `SlotData::Grid` at child insertion time (Seam B — the T2 adapter is
      removed; loader materializes runtime child slots directly).
- [x] Migrate the layout read-path — **there are two parallel vectors,
      not one.** Besides `WidgetData::Grid.cell_placements`, the layout
      mirror has its own `LayoutNode.cell_placements`
      ([layout.rs:250](../../../../wasamo-runtime/src/layout.rs#L250))
      populated by `build_layout_tree` and consumed by `arrange_grid`'s
      `children.zip(cell_placements)`
      ([layout.rs:1327](../../../../wasamo-runtime/src/layout.rs#L1327)).
      Both must move onto an explicit layout child-slot record
      (`LayoutNode.children` stores child slots, not bare children), or
      the `WidgetData` / loader vector is deleted while the layout mirror
      keeps the drift class. `build_layout_tree` / `arrange_grid` / the
      ZStack arrange path read placement from the child slot.
- [x] Re-enumerate the splice / insert / remove / replace side-effect
      bundle for the migrated path (trap #2 close artifact): child list
      splice (placement riding the slot), Visual sibling order, layout
      invalidation, widget-pointer registry, effect ownership — restated
      from DD-M3-P7-006 for the Grid path. `insert_child_inner` /
      `remove_child` / `replace_child` mutate child-slot records:
      insertion computes slot data from the parent context, removal drops
      the detached slot metadata while returning a bare widget subtree,
      replacement preserves / recomputes the slot metadata on the slot
      while replacing the node, and placement-free parents carry `None`.
- [x] **Close artifact (trap #3):** no parallel placement vector remains
      on any mutated path — the audit table has an **independent row per
      site**: `WidgetData::Grid.cell_placements`,
      `LayoutNode.cell_placements`, `LayoutNode::grid` (constructor
      signature), `arrange_grid` (the `zip`), and `build_layout_tree`
      (the copy) — each shown migrated to the child slot or deleted
      (greppable: no `cell_placements` / `zstack_placements` survives on
      a mutated path); no-placement containers carry `None`.
- [x] Windows-runtime integration fixtures (CI-gated, fail-not-skip):
      ZStack and Grid layout read placement from child-slot storage;
      structural insert / remove **and `replace_child`** under ZStack
      preserve child order, Visual sibling order, placement, and
      invalidation. `replace_child` is a mutated path that carries slot
      placement onto the replacement
      ([widget.rs:1432](../../../../wasamo-runtime/src/widget.rs#L1432))
      and is ABI-exposed
      ([abi.rs:491](../../../../wasamo-runtime/src/abi.rs#L491)), so the
      side-effect bundle lists `insert / remove / replace` — **fire
      replace directly** or record in [log.md](./log.md) why the pure
      mirror test + the trap-#2 close artifact suffice without a separate
      integration run. Also: destroy / detach leaks no placement
      metadata; `if` / `for`-generated ZStack children carry
      `SlotData::ZStack` through staging → commit (CF-1 / R-G; storage
      only — **no Grid mutation path** is built, DD-002 §Out of scope).
- [x] Regression gate: Phase 5 Grid fixtures (track sizing, spanning,
      membership / conflict, arrange overflow) + Phase 6/7 ZStack
      fixtures run unchanged.

**Sub-task hypothesis:** (a) runtime `ChildSlot` record + ZStack
migration; (b) Grid `cell_placements` removal + loader Seam B; (c)
layout child-slot read-path; (d) splice side-effect re-enumeration; (e)
integration + regression fixtures. T1 may re-cut.

**Start gate:** T2 merged; T3 trap selection recorded. **End gate:**
workspace + integration + regression green; trap #1/#2/#3 artifacts
recorded; **full independent review** before merge.

---

### T4 — `wasamoc` `slot.` author surface (A13) + PM-2 matrix + `.ui` migration

The author-reachable grammar + diagnostics (gates trap #4; risk
R-C / R-D / R-E). Discharges ADR evidence item (1) (compile-time
positive + the full reject matrix). **Branch/test-focused review** for
the reject branches. T4 owns the author surface only; runtime storage is
T3-owned and the textual-IR carrier is T2-owned (T4 lowers into it).
T4 does **not** own GUI-visible proof, normative/reference docs sync, or
future Grid structural-mutation policy; those remain T5 / T7 carry-forward
items. T4 may update this task list as the reject matrix is enumerated,
but it must not weaken the PM-2 accept-set or the ZStack bare-placement
reject fixed by the DDs.

- [x] Lexer / parser: the `slot.` dotted property-key lexeme on a child
      (`slot.row` / `slot.column` / `slot.row-span` / `slot.column-span`
      / `slot.h-align` / `slot.v-align`) read into the existing
      `Member::PropertyBind { name = "slot.<key>", ... }` AST shape as a
      **dotted property key, not an expression member-access** (R-E). No
      new AST member variant or runtime node type is introduced for the
      `slot.*` path. Malformed key shapes (`slot:` / `slot..h-align` /
      `slot.`) are **parser-stage** rejects.
- [x] `wasamoc check` — the full DD-001 §Spec-impact forcing table
      (each row a named diagnostic with a firing test, trap #4):
      - **Per-key admission:** Grid admits the 6 keys (inside `Cell`
        **and** as direct `slot.*`); ZStack admits `slot.h-align` /
        `slot.v-align`; rejected everywhere else.
      - **PM-2 strict mixing reject:** `slot.*` among a `Cell` node's own
        attrs → **mixing** reject (distinct diagnostic).
      - **Non-admitting-parent reject:** `slot.*` on a widget *inside* a
        `Cell` → **non-admitting parent** reject (a *different*
        diagnostic from mixing — R-D).
      - **Stray placement** under a non-admitting parent (e.g. VStack);
        **unknown slot key** (`slot.foo`); **constant-RHS** reject
        (binding / state expr RHS → placement is constant per instance);
        the **value-namespace rule** (`end` is the placement keyword even
        when a state named `end` exists — R-E); ZStack **bare**
        `h-align` / `v-align` → named reject (no long-lived alias, R-C);
        placement-vs-unknown-widget-prop split in both directions.
      - Defaults preserved per container (Grid `stretch`, ZStack
        `center`). A Grid child with direct `slot.*` and no `row` /
        `column` follows the child-slot default path established by
        T2/T3; the retained `Cell` grouped form keeps its existing
        single-Cell / multi-Cell placement rules.
- [x] Lower all three authored forms — Grid `Cell`, Grid direct
      `slot.*`, ZStack `slot.*` — into the **one** T2 IR slot record
      (model-level unification); CF-1 (placement on the `for` / `if`
      body's root child) inherits the static-child surface, no new
      syntax.
- [x] **`.ui` migration (bundled — R-C):** flip every in-repo `.ui`
      (`examples/gallery/`, `examples/**`, `wasamoc` / runtime test
      fixtures) off bare ZStack `h-align` / `v-align` onto
      `slot.h-align` / `slot.v-align`; Grid `Cell` stays (retained
      grouped form). Greppable sweep; the build (which compiles
      `gallery.ui`) stays green. Grid examples default to `Cell`
      (provisional convention); show direct `slot.*` where it
      illustrates the unified surface.
- [x] Tests: positive controls (`slot.*` under both containers compiles
      and lowers; `Cell` and Grid direct `slot.*` both lower to the same
      record; a state named `end` / `append` still parses) + one reject
      test per matrix row; emit-roundtrip shape tests (with T2).

**Sub-task hypothesis:** (a) parser dotted-key storage in existing
`PropertyBind`; (b) check admission matrix + the two Cell-related
rejects; (c) lower Grid direct `slot.*`, retained Grid `Cell`, ZStack
`slot.*`, and CF-1 body-root placement into `IrSlotData`; (d) `.ui`
migration sweep. Splittable per the matrix.

**Start gate:** **T3 merged** (the runtime feeds `SlotData` directly —
the Seam A legacy adapter is gone, so what T4 lowers is exercised
against the *final* storage, not the T2 transitional path) **and** T2
merged (IR carrier exists to lower into); T4 trap selection recorded. If
T1 revises the default order so T4 lands before T3, the T4 start-gate
note in [log.md](./log.md) must state explicitly **what the author
surface is being proven against** (legacy adapter vs final `SlotData`)
so a reviewer can see the proof is not on a transitional path. **End
gate:** workspace + tests green; every forcing-table row has a firing
test; **branch/test-focused review** of the reject branches before
merge.

---

### T5 — GUI evidence (assistant build / launch / screenshot / analysis)

Discharges the assistant-automated portion of ADR evidence item (4)
(gates trap #7; GUI-render high-risk class → **full independent
review**). The owner-visible portion is T6's. Assistant evidence is
**launch + DPI-aware screenshot capture + assistant analysis**;
`Start-Process` survival is a supporting signal only.

**T5 re-cut against the actual gallery (recorded in [log.md](./log.md)).**
The planning-time hypothesis assumed the shipped gallery already exercises
the placement positive controls. A pre-implementation probe (environment
capability check + capture of the current `examples/gallery/gallery.ui`,
frames under [evidence/](./evidence/) `t5-probe-*.png`) disproved that:

- The gallery's ZStack children use **only `slot.h-align: stretch`** —
  there is **no `end` alignment and no alignment contrast** anywhere in
  the shipped surface, so the ZStack positive control cannot be read off
  the current gallery.
- The main-screen Grid does **not appear** in an 800×600 capture; the
  only Grid placement surface is the **lightbox** (centred 400px column +
  per-row cells), which must be opened to be observed.
- The proven `capture-lightbox.ps1` **click coordinates are stale** for
  the current layout (the probe's "open" frame equals the "closed" frame
  — the lightbox did not open), so the script cannot be reused verbatim.

Therefore T5 owns **building a deliberate placement positive-control
surface**, not relying on the gallery's incidental layout. Per the owner
decision (2026-06-23, recorded in [log.md](./log.md)), the surface is a
**toggled placement-demo sub-screen added to `gallery.ui`** (option A):
T5 and T6 share one host, and it follows the established pattern of the
gallery accumulating per-phase verification surfaces (Footer clip = P5,
lightbox = P6/7, reactive list = P7) that the **Phase 8 close will sweep
together**. The added surface carries an explicit Phase-8-removal marker
and is recorded as a Phase-8 cleanup carry-forward (T7 / phase-end
ledger).

- [x] Add a **placement-demo sub-screen** to `examples/gallery/gallery.ui`
      (a `state is_placement_demo_open` + "Open placement demo" button +
      `if`-overlay; self-contained, marked for Phase 8 removal) that makes
      the positive controls **visible and contrastive**:
      - **ZStack:** three overlay children at `slot.h-align: start` /
        omitted → default centre / `end` land at **three different
        horizontal positions** — a single static frame a wrong
        implementation could not equally produce.
      - **Grid:** cells at distinct row/column/span **and** an explicit
        `h-align: center` cell contrasted against a **stretch-default**
        cell (the per-container default).
      - **Deviations (recorded in [log.md](./log.md)):** the overlay
        content is wrapped in a stretched `VStack` because a Grid cannot
        be a placed ZStack child (`check_grid` rejects `slot.*`; a
        ZStack-centred Grid measures 0×0 — pinned by
        `zstack_grid_child_slot_alignment_rejected`, carry-forward to T7);
        `aspect` Boxes are avoided in demo cells (they abort the intrinsic
        measure). The surface defaults to `false` and is opened by a click
        in the capture; synthetic input drives the Composition app's
        buttons on a real / elevated desktop (a sandboxed session drops the
        input — environment note, not an app limitation).
- [x] Build and run `examples/gallery-rust/`. **Capture mechanics —
      reuse the proven Phase 6/7 pattern**
      ([capture-lightbox.ps1](../../phase-6/implementation/evidence/capture-lightbox.ps1)):
      per-monitor-DPI-aware, enumerate the top-level HWND by title,
      `CopyFromScreen` over `GetWindowRect` (not `PrintWindow`, which
      reads back blank under DirectComposition). The navigation
      coordinates were re-derived; the re-tuned capture driver
      ([capture-placement-demo.ps1](./evidence/capture-placement-demo.ps1))
      lands under [evidence/](./evidence/).
- [x] Record assistant evidence as labelled frames under
      [evidence/](./evidence/) (`t5-home.png`, `t5-placement-demo.png`,
      `t5-lightbox-slot-current.png`) with the analysis in
      [evidence/README.md](./evidence/README.md) noting what each frame
      proves and the positive control it carries. DPI blur, if any, is the
      known M4 residual, not a Phase 7b failure.
- [x] **Same-position proof (closed in T5).** Capture the current-branch
      `slot.*` lightbox at the Phase 6/7 baseline size and compare to the
      pre-migration lightbox evidence.

**Baseline for the same-position proof.** Because T4 rejects the old
ZStack bare syntax, the bare-surface baseline **cannot be regenerated on
this branch**. The correct baseline is the **T4-pre commit** (`3134287`,
the last commit on the bare surface, with the *same* lightbox content) —
**not** the Phase 6/7 evidence, which is an older gallery version whose
unrelated `.ui` evolution would confound the comparison (a ~20px offset).
T5 built `gallery-rust` at `3134287` in a throwaway worktree and captured
`evidence/t5-lightbox-bare-baseline.png`, then captured the current
`slot.*` lightbox `evidence/t5-lightbox-slot-current.png` at the same
800×600. The two are **pixel-position identical** (photo/caption bbox
`x=150..648 y=60..544` in both), proving the `slot.*` migration is
position-preserving (assistant portion closed in T5; owner-visible
confirmation is T6's). The **contrast** half (varied ZStack alignment →
varied position; omitted Grid placement → default) is the placement-demo
sub-screen's `t5-placement-demo.png`.

**Start gate:** **T3 merged** (placement renders from final `SlotData`
storage) **and** T4 merged (`.ui` on the new surface); the baseline
source named; T5 trap selection recorded. **End gate:** screenshots +
analysis + positive controls under `evidence/`; the placement-demo
surface recorded as a Phase-8 removal carry-forward; **full independent
review**.

---

### T6 — Owner-manual GUI smoke

Discharges the owner-visible smoke for ADR evidence item (4); a separate
gate from T5's assistant baseline. The assistant prepares the runnable
host + the detailed owner observation script; the smoke itself is
owner-performed and cannot be discharged by the assistant baseline.

- [ ] Owner runs the gallery / lightbox and observes, with the
      **positive control = placement varied** (an explicit alignment
      lands where expected, a contrasting one lands elsewhere, omitted
      placement falls to the default — a single static frame a hardcoded
      tree could equally produce is not evidence): ZStack `slot.*`
      placement and Grid cell placement render at the same positions as
      the old surface; WrapPanel reflow / ScrollView behaviour stay
      correct around placed children.
- [ ] Owner explicitly accepts the smoke result, or records a fail
      observation; fixes land additively on the T6 branch and the
      checklist re-runs to green before T6 closes.
- [ ] T6 step-end retrospective recorded at
      `process/milestone-3/phase-7b/retrospectives/t6.md`.

**Start gate:** T5 merged; runnable host + observation script prepared.
**End gate:** owner acceptance (or fail → fix → re-run) recorded.

---

### T7 — Step-end local gates + Moment 2 re-sync + A12 closure

Discharges the T7-owned portion of the phase-end criteria, the
[ADR Moment 2 commit set](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2),
and the **A12 spec-closure gate**. The step-end retrospective is **owned
by T7**; the phase-end retrospective, CI run id, handoff finalization,
and the implementation-preamble status flip are **NOT owned by T7** (see
the [preamble split](./preamble.md#step-end--phase-end-retrospective-split-final-task-ownership)).
Before closing, cross-check this T0-frozen task list against mid-phase
owner decisions and revise where they diverge.

Critical responsibility split (rechecked at T7 start): T7 is a
**document-sync and step-close task**, not the phase-close task. It owns
local verification, the Moment 2 implementation sync, the M3 plan row,
and a carry-forward **candidate ledger** in `log.md`. The phase-end
retro owns final `implementation/handoff.md`, the GitHub Actions CI
run-id, the phase-end retrospective, and `implementation/preamble.md`
status flip.

- [ ] T7 start gate recorded in [log.md](./log.md): carry-over from
      `log.md` and every Phase 7b task retrospective checked; relevant
      implementation-gates selected; T7 / phase-end ownership split made
      auditable before the docs are edited.
- [ ] `cargo fmt --all -- --check` green locally.
- [ ] Local clean rebuild green locally (`cargo clean` →
      `cargo build --release --workspace` → `cargo build --workspace` →
      `cargo test --workspace`). CI green is phase-end-owned.
- [ ] `docs/dsl_spec.md` §4.16 / §8.5 / §8.11 marker flips to
      `M3-Phase 7b closed; implementation-synced`; document Status
      header updated; divergence corrections folded (the design-draft
      token / skeleton spellings pinned to the landed shapes).
- [ ] **A12 spec-closure gate:** the placement chapter at the
      external-reader bar — the `slot.*` surface, the PM-2 two-form Grid
      accept-set, the invalid examples matching the shipped diagnostics,
      the constant-per-instance rule, the per-container defaults.
- [ ] `docs/architecture.md` §6.8.6 (`SlotData` storage) + §6.8.4 (Grid
      SM-B) re-synced to the landed shape; the splice side-effect
      re-enumeration confirmed against the code; the future-API
      non-committal constraint (no generic child property setter) present
      in prose.
- [ ] `docs/architecture.md` §6.7.9 member-level structural IR
      re-synced to the landed Rust spelling: both code blocks / prose
      examples that currently show the design-draft
      `Widget { node, slot_data }` spelling are updated to the landed
      `IrChildSlot` wrapper if T2 lands that shape.
- [ ] `docs/notes/architectural-family.md` FD-7b-C confirm-within-family
      (1) entry landed (revise-in-place) if not already at Moment 1.
- [ ] `docs/abi_spec.md` re-confirmed untouched; any forced ABI surface
      escalates with owner confirmation.
- [ ] `process/milestone-3/plan.md` Phase 7b row Status flips to
      `implementation complete; phase-end pending`.
- [ ] ADR set touched **only** if a retrospectives.md §phase-sync
      ADR-touch case applies; otherwise it stays at its Moment 1
      Accepted state.
- [ ] [log.md](./log.md) records the phase-close evidence pointers and
      implementation summary distilled from T1–T6.
- [ ] [log.md](./log.md) records the phase-end handoff **candidate
      ledger** distilled from T1–T6 and T7: the **pre-1.0 wrapper-rule
      decision** (PM-2 → PM-1 / PM-3) with its re-triggers; the
      **VS-2 / VS-3** carrier triggers; the **Grid structural-mutation**
      trigger (DD-M3-P7-006 recursive — migrate before any Grid mutation
      path); the **bindable-placement** trigger (joint `BindingTarget` +
      child-slot effect lifecycle); the default-alignment-unification and
      key/value-spelling deferrals.
- [ ] Carry-forward inputs to the Phase 8 pre-doc recorded under
      [handoff.md](./handoff.md). Completed by the phase-end handoff, not
      by T7.
- [ ] Front-matter `status` on [preamble.md](./preamble.md) flips
      `active` → `closing` at the **phase-end batch commit**, not at T7
      step-close. Completed by the phase-end documentation batch.
- [ ] **T7 step-end retrospective recorded** at
      `process/milestone-3/phase-7b/retrospectives/t7.md` (items 1–11;
      **owned by T7**).
- [ ] **Phase-end retrospective recorded** at
      `process/milestone-3/phase-7b/retrospectives/phase-end.md` (items
      12–18; **NOT owned by T7**; separate retro work on the phase branch
      after T7 merged in). No open `phase-sync` items survive past phase
      close. CI run id is recorded by the phase-end log / retro update
      after the `workflow_dispatch` gate goes green.

**Start gate:** T6 merged; T7 start-gate recorded. **End gate (T7
step-close):** local gates green, Moment 2 synced, candidate ledger
recorded, T7 retro done — `status` stays `active` (phase-end owns the
flip).
