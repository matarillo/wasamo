# M3-Phase 7 — implementation handoff

Forward-carry material for the next phase's pre-doc framing, prepared for
the Phase 7 phase-end retrospective (retrospectives.md item 15 / §6.3).
The next planned implementation phase is **M3-Phase 8**. A few entries also
target **M4 / M5** because Phase 7 intentionally stopped at positional,
single-widget iteration over scalar collection items.

`doc-folded` dispositions are not transcribed as requirements here. The
next phase should read the synced specs directly. This file records the
confirmed carry-forward constraints, out-of-phase residuals, and
next-phase-relevant learnings.

## Phase 8 handoff targets

- **Phase 7 shipped positional, un-keyed iteration only.** Generated
  children are freshened from collection values, prefix entities are retained
  across tail append / tail drop, and same-length reset updates live item
  reads without changing cardinality. This is now folded into
  `docs/dsl_spec.md` and `docs/architecture.md`. Phase 8 can rely on that
  baseline for gallery assembly, but must not imply keyed retention, reorder,
  or per-item state preservation. Re-trigger: any `key:` syntax, reorder,
  focus / input retained-state case, or state preservation across removal.

- **Per-item interaction remains deferred.** Phase 7 rejects handlers inside
  `for` bodies and rejects loop-local binder reads in handler position. Phase
  8's selected-state work may create the first concrete pressure for
  "select/delete this item" behavior. Re-trigger: M4 input work, selected
  thumbnails, delete buttons, or any handler needing the current item/index.

- **Per-item conditional presence is still outside the shipped scope.** Phase
  7 rejects binder reads in `if` conditions and does not allow an item-local
  boolean to shape the generated body. Re-trigger: the first gallery or
  selected-state case that needs per-item branching, or the next structural
  control-flow extension.

- **Nested `for`, nested template scope, and shadowing remain unset.** Phase
  7 deliberately keeps loop-local scope flat and admits one structural
  iteration layer. Re-trigger: nested structural control flow, `else` /
  `switch`, nested `for`, or any design that introduces item-template scope
  composition.

- **Member-range bodies remain deferred.** A `for` body is one widget child,
  not a range of sibling members. Phase 8 should keep using wrapper
  containers when one item needs multiple visual elements. Re-trigger: the
  first UI needing multiple sibling members per collection item without an
  explicit wrapper.

- **Loop-external collection reads are deferred.** Phase 7 provides
  expression-position item and index reads inside the loop body, but not
  `length`, empty checks, arbitrary indexed reads, or cross-collection
  composition such as `colors[index]`. Re-trigger: `length`, empty-state UI,
  element index reads outside the loop body, or cross-collection composition.

- **The gallery surfaced a per-item richness cluster, with a Phase 7b owner
  reservation.** T8 showed the first concrete app pressure for richer item
  data: structured item fields / `TypedValue`, loop-external indexed reads,
  and bindable `Box.fill` / dynamic styling. The accepted Phase 7 reduction
  used a single scalar collection and constant styling, and the owner
  reserved the option to open Phase 7b; do not write M4+ routing as settled.
  Re-trigger: a gallery thumbnail needing per-item `{ label, color }`,
  dynamic `fill`, structured item fields, or the owner opening Phase 7b.

- **Grid structural mutation is still gated by placement migration.** ZStack
  placement is child-carried after Phase 7, but Grid `cell_placements` remain
  static-only. Before admitting structural mutation under Grid (`for` of
  `Cell`s, conditional Cells, or equivalent), migrate Grid placement to the
  child-carried storage model or reopen the placement storage decision.

- **The host-state boundary remains future-compatible, not implemented.**
  Phase 7's whole-value runtime-owned collection state does not expose host
  initial collections, host replace, or write-back APIs. Re-trigger:
  host-supplied initial collections, host replacement of a collection, or an
  ABI-facing write-back surface.

## Reactive-engine residuals

- **DD-M3-P7-007 residual 1: cycle detection policy remains deferred.**
  Phase 7 preserves synchronous drain behavior for collection mutation, but
  does not define a new cycle policy. Re-trigger: any surface that lets a
  generated subtree's effect write state.

- **DD-M3-P7-007 residual 2: ordering ties remain deferred.** Phase 7's
  quiescent-order invariant is drain-order-independent, so it does not settle
  an observable inter-effect ordering contract. Re-trigger: an observable
  contract requiring inter-effect order.

- **DD-M3-P7-007 residual 3: fan-out x `MUTATION_CAP` remains deferred.**
  Phase 7 fixes structural mutation cap accounting to depth rather than
  breadth and proves convergence for many generated children, but it does not
  settle future scheduler / performance policy. Re-trigger: drain-loop
  charging changes, effect-to-signal writes, or acceptance demanding large-N
  performance semantics such as M5+ LazyList work.

- **The synchronous non-batched drain contract is closed, not carried.** T7
  preserved the same-return drain contract with handler-return assertions.

## M4 / later handoff targets

- **Dynamic styling and richer values remain out of Phase 7.** The current
  item surface is scalar (`i32[]`, `string[]`, `bool[]`) with constant widget
  styling where styling is not bindable. `TypedValue`, `f64[]`, structured
  records, expression-level conversions, and dynamic `Box.fill` remain later
  design work unless Phase 7b is explicitly opened.

- **DPI remains an M4 runtime-quality axis.** Phase 7 evidence again ran on a
  high-DPI machine where the known blur can be observed. It is not a Phase 7
  failure and does not affect iteration semantics.

- **The process learning on GUI / assertion self-falsification should be
  codified.** Phase 7 repeated the lesson that GUI evidence must distinguish
  the intended behavior from a look-alike, and that remediation assertions
  should be proven capable of failing for the bug they claim to cover.
  Re-trigger: any future GUI-render evidence, screenshot-based close
  artifact, or defensive assertion remediation.

## Closed items — do not carry as open residuals

- **Partial insert rollback proof is closed.** T9 added the direct
  production-like fault-seam test and fixed the cleanup path. The remaining
  future constraint is the general invariant below, not an unproven Phase 7
  branch.

- **Structural mutation cleanup invariant is doc-folded.** Any built child
  that is not retained by the final tree must be disposed through
  `widget_destroy`, not by an unannotated drop. This keeps staging and commit
  failure cleanup symmetric.

- **The direct `Clear` runtime path is closed.** T8/T9 added the gallery
  `Clear` action and T7 covers zero-child mutation behavior; Phase 7 did not
  leave "only add/remove" as the implemented surface.

- **ABI no-touch is closed.** Collection state stayed runtime-owned and no
  host-facing C ABI surface was added.

## Pointers (doc-folded — not transcribed)

- **Iteration grammar and runtime semantics** are folded into
  `docs/dsl_spec.md` §4.15 / §8 and `docs/architecture.md` §6.7.10 / §9:
  `for <binder> in ...`, optional index binders, scalar collection types,
  append / drop-last / clear / reset assignment forms, positional un-keyed
  identity, stage-then-commit insertion, tail-first disposal, same-return
  drain, and validation rules.
- **Child-carried placement for ZStack** is folded into
  `docs/architecture.md` §6.8.5. Grid remains static-only until the trigger
  above fires.
- **Gallery evidence** lives in the task retrospectives and
  `implementation/evidence/`: T8 assistant screenshots and T9 owner-visible
  smoke cover `Add` / `Remove` / `Clear` / `Reset` positive controls.
