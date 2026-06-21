# M3-Phase 7b — implementation log

This is the in-flight record for M3-Phase 7b. It is the mutable sibling
of [plan.md](./plan.md) / [preamble.md](./preamble.md): the **Decisions
log** (additional decisions that surface during implementation,
including each task's implementation-gate **start-gate** trap selection)
and the **CI / verification log** (build / test / integration evidence,
the trap **close-gate** artifacts, and CI run ids). See
[workflow.md §5.3](../../../procedures/workflow.md) and
[implementation-gates.md](../../../procedures/implementation-gates.md).

Evidence files (screenshots, capture scripts, CI logs) live under
[evidence/](./evidence/), named `tN-<purpose>.<ext>`.

## Decisions log

_(append as decisions surface — T1 records the carrier spelling,
bisectable sequencing, seams, and the T2 impl-gates selection here;
each subsequent task records its start-gate trap selection before
choosing an approach.)_

### T1 start gate — pre-implementation spike

Carry-over check:

- `log.md` had no prior task entries and no recorded carry-over from T0
  or earlier Phase 7b tasks. T0's closed state remains represented in
  [plan.md](./plan.md); no unresolved `log.md` item blocks T1.

Critical T1 responsibility re-cut:

- T1 is a recon / sequencing task, not a production migration slice.
  Production Rust source edits made for compiler verification are
  throwaway and must be reverted before close.
- T1 may revise the mutable plan if recon shows the default T2 → T3 →
  T4 sequencing is not buildable, but it must not reopen DD-fixed
  outcomes: `slot.` author surface, PM-2 Grid accept-set, VS-1a
  `SlotData`, `IrSlotData`, IR-B textual skeleton, stale-form reject +
  regenerate, and constant-per-instance placement.
- T1's durable outputs are the carrier spelling, bisectable sequencing
  / seams, source call-site map, T2 start-gate selection, and downstream
  owner assignment for every open point.

Selected traps and non-applicable reasons:

| Trap | Classification | Reason |
|---|---|---|
| #1 semantic migration | Applies | T1 intentionally performs a throwaway IR/runtime carrier shape change to let Rust enumerate migration call-sites. The close artifact is a compiler-error-derived pre-audit plus an `rg` call-site map; T2/T3 own the authoritative close audits. |
| #2 structural side effects | Not applicable to T1 landing | No tree/state mutation production code lands in T1. T1 still records T3's side-effect audit scope (`insert` / `remove` / `replace`, layout invalidation, Visual order, registry, effect ownership). |
| #3 parallel data drift | Not applicable to T1 landing | T1 does not migrate or update parallel data. It records the known drift sites for T3: `WidgetData::Grid.cell_placements`, `LayoutNode.cell_placements`, `LayoutNode::grid`, `arrange_grid`, and `build_layout_tree`. |
| #4 untested authored branch | Not applicable | T1 adds no reject / diagnostic / size / semantic branch. T2 owns loader stale-form rejects; T4 owns author-surface rejects. |
| #5 carry-forward | Applies | T1 must assign each discovered open point to a downstream task with scope / impact / close condition before T2 opens. |
| #6 deterministic failure disposition | Conditional | The planned throwaway `cargo build` failure is evidence, not a flake. If any unexpected recurring failure appears during recon, T1 must record rerun history and disposition before close. |
| #7 GUI positive control | Not applicable | T1 has no GUI-render deliverable; T5 owns assistant screenshot evidence and positive controls. |

Review lane:

- **No special review** for T1 itself: no production code, schema
  migration, runtime structural change, GUI-render evidence, or new
  reject / diagnostic branch lands. The ordinary task review still checks
  that the T1 recon artifacts are complete. T2 and T3 remain full-review
  lanes; T4 remains branch/test-focused; T5 remains full-review.

Planned proof obligations before implementation:

| Branch / behavior / invariant hypothesis | Category | T1 proof obligation |
|---|---|---|
| `IrMember::Widget(IrNode)` can move to an explicit `IrChildSlot` wrapper carrying `slot_data` only by touching every traversal / construction / emit / loader site. | Semantic migration | Throwaway compile-error enumeration plus `rg` map over `IrMember::Widget`, `IrMember`, and `widget_children()`. |
| `SlotData` can replace ZStack child-slot placement and later Grid parallel placement without reopening DD-fixed naming. | Semantic / invariant | Record concrete runtime child-slot record spelling and the sites T3 must migrate. |
| T2 can be buildable by parsing / emitting IR-B while adapting loader output to legacy runtime storage (Seam A). | Observable buildability seam | Confirm against source; if not viable, revise plan before migration code lands. |
| T3 can remove the legacy adapter and make loader feed runtime `SlotData` directly (Seam B). | Structural invariant | Record runtime / layout sites and T3 close audit scope. |
| T4 can add dotted `slot.*` author keys without changing AST variant shape. | Semantic branch hypothesis | Decide AST storage seam (`PropertyBind.name = "slot.h-align"` vs new variant) from parser / AST source and assign branch tests to T4. |
| No production code remains from the spike. | Observable invariant | `git diff` after revert must contain only T1 process artifacts. |

Known carry-forward candidates at T1 start:

| Candidate | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| T2 schema migration call-sites | T2 | `wasamo-ir`, `wasamoc` lower/emit/check, runtime loader parser/validator/traversal. Misses can silently drop slot metadata. | T2 trap-#1 call-site audit table and build/tests green. |
| Runtime `SlotData` side-effect set | T3 | `WidgetNode` child slot, Grid storage, layout mirror, insert/remove/replace, Visual order, layout dirty, registry, effect ownership. | T3 trap-#1/#2/#3 artifacts; no surviving mutated-path parallel placement vector. |
| `slot.*` parser/checker/lower matrix | T4 | Dotted key parsing, PM-2 mixing vs non-admitting-parent split, constant RHS, value namespace, ZStack bare reject, `.ui` migration. | T4 forcing-table direct tests and branch/test-focused review. |
| T4 reject sub-row visibility | T4 | Unknown `slot.*` key, malformed dotted key, stray placement on non-admitting parent, Cell direct-vs-wrapper mixing, placement keyword vs state-name namespace, constant-only RHS, ZStack bare-placement reject, per-container default preservation. | T4 start gate links or expands the forcing table and close gate names each direct firing test. |
| GUI same-position proof baseline | T5 | Old ZStack bare syntax cannot be regenerated after T4; baseline must be chosen before capture. | T5 start gate names baseline source and close evidence includes positive controls. |
| `docs/architecture.md` §6.7.9 IR spelling sync | T7 | Two member-level structural IR code/prose examples currently show the design-draft `Widget { node, slot_data }` spelling; the landed `IrChildSlot` wrapper changes that illustrative Rust spelling without changing the normative child-slot model. | T7 Moment 2 docs sync updates §6.7.9 to the landed shape or records why T2 chose a different final spelling. |
| Phase-end residual ledger | T7 / phase-end | Pre-1.0 wrapper rule, VS-2/VS-3 triggers, Grid mutation trigger, bindable placement trigger, default alignment, key/value spelling. | T7 candidate ledger plus phase-end handoff finalization. |

### T1 decisions — carrier spelling, sequencing, and seams

Source files read:

- `wasamo-ir/src/lib.rs`
- `wasamo-runtime/src/widget.rs`
- `wasamo-runtime/src/layout.rs`
- `wasamo-runtime/src/ir_loader.rs`
- `wasamoc/src/lexer.rs`
- `wasamoc/src/parser.rs`
- `wasamoc/src/ast.rs`
- `wasamoc/src/check.rs`
- `wasamoc/src/lower.rs`
- `wasamoc/src/emit.rs`

Machine queries used for recon:

- `rg -n "IrMember::Widget|IrMember|widget_children\(|cell_placements|zstack_placement|LayoutNode::grid|arrange_grid|build_layout_tree|parse_property_bind|PropertyBind|CHILD_PLACEMENT_ATTRS|CELL_ATTRS|h-align|v-align" wasamo-ir\src wasamoc\src wasamo-runtime\src`
- `rg -n "fn build_layout_tree|fn insert_child_inner|fn remove_child|fn replace_child|pub fn grid|fn arrange_grid|cell_placements|zstack_placement|fn validate_grid|fn validate_phase5|fn validate_phase6|fn parse_node|fn parse_member|fn construct_widget|fn build_node|Cell" wasamo-runtime\src\widget.rs wasamo-runtime\src\layout.rs wasamo-runtime\src\ir_loader.rs wasamoc\src\check.rs wasamoc\src\parser.rs wasamoc\src\lower.rs wasamoc\src\emit.rs`
- Throwaway compiler probe: `cargo build --workspace` after changing
  `IrMember::Widget(IrNode)` to a struct variant and temporarily adding a
  runtime `SlotData` skeleton. This probe enumerated the compile surface;
  the T2 target spelling is the `IrChildSlot` wrapper recorded below.

Carrier spelling:

- Naming note: `ChildSlot` is **not** ADR-fixed. The DD-fixed shape is
  the child-slot record carrying a node plus `SlotData`. T1 recommends
  `ChildSlot` for the runtime and layout module-local record names
  because it matches the ADR vocabulary; an implementation task may choose
  an equivalent local name only with an explicit reason in its start /
  close artifact.

- IR carrier: add `IrChildSlot { node: IrNode, slot_data:
  Option<IrSlotData> }` and change the widget member spelling to
  `IrMember::Widget(IrChildSlot)` in `wasamo-ir/src/lib.rs`.
- IR Rust-spelling tradeoff:
  - Pros for `IrChildSlot`: it makes the IR child-slot record first-class
    like the runtime / layout `ChildSlot` records; keeps future slot-local
    fields such as `key` or lifecycle metadata off the enum variant; and
    gives helpers a single slot object when placement is relevant.
  - Cons for `IrChildSlot`: it adds one named wrapper type and more
    pattern-match churn than `IrMember::Widget { node, slot_data }`;
    placement-insensitive callers must deliberately unwrap `slot.node`.
  - Disposition: prefer `IrChildSlot` because Phase 7b's durable model is
    child-slot-carried across IR / runtime / layout, and diff size is not
    the primary optimization criterion for T2/T3.
- IR payload: use a closed broad carrier, with per-container struct-style
  variants:
  - `IrSlotData::Grid { row: u32, column: u32, row_span: u32,
    column_span: u32, h_align: IrAlignment, v_align: IrAlignment }`
  - `IrSlotData::ZStack { h_align: IrAlignment, v_align: IrAlignment }`
  - add `IrAlignment::{ Start, Center, End, Stretch }` rather than
    carrying placement values as raw strings.
- Runtime carrier: replace bare child-node storage with an explicit
  child-slot record, recommended spelling:
  `struct ChildSlot { node: Box<WidgetNode>, slot_data:
  Option<SlotData> }`, and store `WidgetNode.children: Vec<ChildSlot>`.
  `SlotData` is spelled as `SlotData::{ Grid(CellPlacement),
  ZStack(ZStackPlacement) }`, reusing the existing layout-engine
  placement payload types for the runtime / layout boundary this phase.
  This replaces `WidgetNode.zstack_placement` without moving placement
  onto the child widget itself.
- Runtime storage tradeoff:
  - Lower-cost alternative: `WidgetNode.zstack_placement` could be
    renamed / widened to `WidgetNode.slot_data: Option<SlotData>` and
    still satisfy the Phase 7b one-carrier / no-parallel-vector
    requirement once Grid's parallel vectors are removed.
  - Pros for explicit `ChildSlot`: it makes the durable child-slot model
    true in the runtime type shape, keeps future slot-local fields off
    the widget node, and aligns runtime / layout with the `IrChildSlot`
    IR spelling.
  - Cons for explicit `ChildSlot`: it is a wider structural migration
    than a field rename; every child-list traversal / splice path must be
    re-audited; and T7 must re-sync `docs/architecture.md` §6.7.9 from
    the design-draft `Widget { node, slot_data }` spelling to the landed
    wrapper shape.
  - Disposition: choose explicit child-slot records because the phase
    target is the durable child-slot-carried model across IR / runtime /
    layout, and minimizing the T3 diff is a lower-priority axis than
    avoiding another representational split.
- Layout carrier: likewise make the layout tree's children explicit
  child-slot records (recommended module-local name: `ChildSlot`) so
  layout placement is read from the slot record, not from
  `LayoutNode.cell_placements` or a placement field on the child node.
- `None` means "no explicit slot data carried on this child slot"; when
  the immediate parent admits placement, the parent path applies the
  per-container default for omitted placement. Placement-free parents
  normalize child `slot_data` to `None`.

Textual IR mapping:

- `IrSlotData::Grid { ... }` maps to:
  `child { placement grid { row: ..., column: ..., row-span: ...,
  column-span: ..., h-align: ..., v-align: ... } node <Widget> { ... } }`
- `IrSlotData::ZStack { ... }` maps to:
  `child { placement zstack { h-align: ..., v-align: ... } node <Widget>
  { ... } }`
- T2's emitter should choose stable key order for roundtrip tests:
  Grid `row`, `column`, `row-span`, `column-span`, `h-align`, `v-align`;
  ZStack `h-align`, `v-align`.

Author-surface AST seam:

- T4 stores dotted `slot.*` keys in the existing
  `Member::PropertyBind { name, value, span }` shape, with `name`
  canonicalized to e.g. `slot.h-align`. No new AST member variant is
  needed.
- Parser work in T4: fold `Ident("slot") Dot Ident(<key>) Colon` into a
  property bind; reject malformed `slot:` / `slot..h-align` / `slot.` at
  parser stage. The lexer already emits `Dot`, so no lexer token is
  required unless T4 chooses to improve diagnostics locally.

Bisectable sequencing:

1. **T2 — IR + textual IR + loader parser, with Seam A.** Migrate
   `IrMember::Widget` to `IrMember::Widget(IrChildSlot)`, add
   `IrSlotData`, lower the existing Grid `Cell` and ZStack bare-placement
   author surfaces into `slot_data`, emit the IR-B
   `child { placement ... node ... }` skeleton, parse it in the runtime
   loader, reject stale old-form IR, then adapt parsed `slot_data` back
   into the legacy runtime storage: `WidgetData::Grid.cell_placements`
   and `WidgetNode.zstack_placement`. Workspace remains buildable before
   T3.
2. **T3 — runtime/layout child-slot record migration, Seam A removed.**
   Replace bare `WidgetNode.children: Vec<Box<WidgetNode>>` storage with
   child-slot records carrying `SlotData`, remove
   `WidgetNode.zstack_placement`, remove `WidgetData::Grid.cell_placements`,
   remove the layout mirror `LayoutNode.cell_placements`, and make the
   layout tree carry child-slot records as well. Loader / build-layout /
   arrange read placement from the slot record directly. This is the
   structural full-review task.
3. **T4 — author surface.** Add the `slot.*` parser/check/lower surface
   and migrate in-repo `.ui` files. Lower all accepted author forms into
   the T2/T3 slot record. This remains after T3 so author-surface proof
   runs against the final runtime storage, not the Seam A adapter.

Seam notes:

- Seam A is viable because T2 can parse / lower / emit `slot_data` while
  `build_node` still has access to the parent context and can populate
  the legacy Grid vector or ZStack insertion placement.
- Seam B is T3's removal of that adapter: after T3, loader materializes
  runtime child-slot records directly, layout carries child-slot records,
  and no Grid placement vector remains.
- Seam C is T4's AST/check/lower expansion: the new direct author form
  reuses `PropertyBind.name = "slot.*"` and lowers into the existing
  `IrSlotData`.

T2 implementation-gate selection before T2 opens:

| Trap | Classification | Reason |
|---|---|---|
| #1 semantic migration | Applies | T2 changes the IR schema from tuple `IrMember::Widget(IrNode)` to `IrMember::Widget(IrChildSlot)` and must classify every traversal, construction, emit, and loader site. |
| #2 structural side effects | Not applicable to T2 landing | T2 intentionally keeps runtime storage behavior behind Seam A; tree mutation and runtime side-effect enumeration are T3. |
| #3 parallel data drift | Not applicable to T2 landing | T2 keeps legacy runtime parallel storage temporarily and must name it as Seam A, not close the drift class. T3 owns deletion / structural close. |
| #4 untested authored branch | Applies narrowly | T2 adds loader textual-IR stale-form / malformed-placement rejects and must ship direct firing tests for those loader branches. Author `.ui` rejects are T4. |
| #5 carry-forward | Applies | T2 must carry the Seam A adapter and its removal owner (T3) explicitly; any IR call-site deliberately left legacy-shaped must name its owner. |
| #6 deterministic failure disposition | Conditional | Applies only if unexpected recurring failures appear while making the schema migration buildable. |
| #7 GUI positive control | Not applicable | T2 has no GUI-render deliverable. |

T2 review lane:

- **Full independent review** because T2 is an IR / schema migration.
  The review must also include the trap-#4 branch/test check for T2's
  loader reject branches.

T1 compiler-verification results:

| Probe step | Machine output cue | Disposition |
|---|---|---|
| First throwaway build after `IrMember::Widget { node, slot_data }` | `wasamo-ir/src/lib.rs:225` failed in `IrNode::widget_children()` with `expected tuple struct or tuple variant, found struct variant` | Confirms the helper silently filters widget children today and must be classified in T2's trap-#1 audit. |
| Second throwaway build after temporarily updating `widget_children()` | `wasamoc/src/emit.rs:89`, `wasamoc/src/lower.rs:181`, `wasamoc/src/lower.rs:275`; runtime loader sites beginning at `wasamo-runtime/src/ir_loader.rs:364`; runtime storage sites around `wasamo-runtime/src/widget.rs:235`, `:1383` | Confirms the migration is compile-error-forcing across emitter, lowering, parser/loader traversal, validator, structural builder, and runtime storage. The runtime helper type mismatch was throwaway-edit noise; the meaningful T3 cue is the `zstack_placement` field/helper family. |
| Revert check | `git status --short` after `git restore --source=HEAD -- wasamo-ir/src/lib.rs wasamo-runtime/src/widget.rs` showed only `implementation/log.md` and `implementation/plan.md` modified | No production source remains from the spike. |
| Green check after revert | `cargo build --workspace` finished successfully | The workspace returned to buildable state after the throwaway migration was reverted. |

Recon call-site map:

| Site family | Source query / diff cue | Owner | Classification |
|---|---|---|---|
| IR carrier definition and helper | `wasamo-ir/src/lib.rs:171`, `:214`, `:216` from the `rg` query | T2 | Must add `IrChildSlot` and migrate to `IrMember::Widget(IrChildSlot)`; `widget_children()` must return `slot.node` and deliberately ignore `slot_data` only where placement-insensitive traversal is correct. |
| IR tests / direct construction | `wasamo-ir/src/lib.rs:417`, `:488`, `:546`, `:554` | T2 | Must update constructors to build `IrChildSlot`; tests should include at least one direct `IrSlotData` roundtrip. |
| wasamoc lower construction | `wasamoc/src/lower.rs:181`, `:275` from compiler errors and `rg` | T2 | Must construct `IrMember::Widget(IrChildSlot { node, slot_data })`; existing Grid `Cell` and ZStack bare placement lower into `slot_data` in T2. T4 adds direct `slot.*` author lowering later. |
| wasamoc lower traversal tests | `wasamoc/src/lower.rs:619`, `:1225`, `:1292`, `:1347`, `:1399` | T2 | Must update test pattern matches and classify placement-insensitive traversal. |
| wasamoc emit traversal | `wasamoc/src/emit.rs:87`, `:89`, `:488`, `:818`, `:825` | T2 | Must emit `child { placement ... node ... }` for widget members; control-flow body emission must preserve slot data for generated child records. |
| runtime loader annotation / validation traversals | `wasamo-runtime/src/ir_loader.rs:360`, `:364`, `:528`, `:544`, `:560`, `:627`, `:683`, `:804`, `:850`, `:910`, `:1012`, `:1124`, `:1347` | T2 | Must classify each traversal as placement-sensitive, placement-insensitive, or stale-form reject. Wildcard/filter helpers are the main trap-#1 risk. |
| runtime loader textual parser constructors | `wasamo-runtime/src/ir_loader.rs:2151`, `:2238`, `:2279` | T2 | Must parse `child` records into `IrMember::Widget(IrChildSlot { node, slot_data })`; stale `node Cell { prop row ... }` and bare ZStack placement props become named rejects. |
| runtime loader builder / control-flow staging | `wasamo-runtime/src/ir_loader.rs:2766`, `:2804`, `:2814`, `:2837`, `:2885`, `:2914`, `:3175`, `:3317`, `:3450`, `:3498`, `:3507` | T2/T3 | T2 uses Seam A to adapt `slot_data` to legacy runtime storage; T3 removes adapter and carries `SlotData` through runtime child-slot records, including generated-child staging / commit. |
| runtime ZStack child slot storage | `wasamo-runtime/src/widget.rs:226`, `:233`, `:240`, `:351`, `:1378`, `:1428`, `:1443`, `:1447`, `:1473`, `:1677` | T3 | Replace child-node field storage with `ChildSlot { node, slot_data }`; preserve insert/remove/replace semantics and no-placement normalization. |
| runtime Grid parallel vector | `wasamo-runtime/src/widget.rs:176`, `:651`, `:662`, `:1652`, `:1655`; `wasamo-runtime/src/ir_loader.rs:3450`, `:3451` | T3 | Delete vector after Seam A; placement rides runtime child-slot records. |
| layout Grid mirror vector | `wasamo-runtime/src/layout.rs:250`, `:445`, `:448`, `:469`, `:1291`, `:1327` | T3 | Delete layout mirror and make `LayoutNode.children` carry child-slot records; `arrange_grid` must not `zip(children, cell_placements)` after T3. |
| layout ZStack placement read | `wasamo-runtime/src/layout.rs:254`, `:478`, `:500`, `:1408`, `:2940` | T3 | Move/widen to layout child-slot records and preserve default-center behavior. |
| AST dotted-key seam | `wasamoc/src/ast.rs:227`; `wasamoc/src/parser.rs:228`, `:379`; `wasamoc/src/lexer.rs:771`; `wasamoc/src/check.rs:42`, `:1643`, `:1654` | T4 | Keep `PropertyBind`; parser folds dotted key; checker distinguishes `slot.*` placement from ordinary props and from old bare ZStack placement. |

Known risk table sharpening:

| Risk | T1 sharpened hotspot |
|---|---|
| R-A IR carrier migration | `IrMember::Widget` appears in IR helpers, wasamoc lower/emit tests, and many runtime loader validation / parse / build traversals. `widget_children()` is the silent-filter helper to classify explicitly. |
| R-B runtime storage migration | There are two Grid vectors: `WidgetData::Grid.cell_placements` and `LayoutNode.cell_placements`; `build_layout_tree` copies one into the other; `arrange_grid` consumes the layout mirror via `zip`. |
| R-D PM-2 diagnostic split | Existing checker treats `CHILD_PLACEMENT_ATTRS = ["h-align", "v-align"]` as bare placement and special-cases `Cell`; T4 must split `slot.*` on `Cell` from `slot.*` inside `Cell`. |
| R-E dotted-key seam | Lexer already emits `Dot`; parser currently accepts property binds only as `Ident Colon`. T4 can fold `Ident("slot") Dot Ident(key) Colon` into the existing AST without introducing expression-member access. |

T1 close gate — implemented-branch test map:

| Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
|---|---|---|---|
| T1 no-production-code spike boundary made explicit in the task plan. | Process / invariant | `git diff -- process\milestone-3\phase-7b\implementation\plan.md` shows T1 changed to "No production code lands" and throwaway revert wording. | `cargo build --workspace` green after revert; ordinary T1 review checks plan wording. |
| T1 start-gate artifact recorded before production migration work. | Process / gate artifact | `git diff -- process\milestone-3\phase-7b\implementation\log.md` shows `### T1 start gate`. | Ordinary T1 review. |
| Compiler-error-forcing pre-audit performed and reverted. | Semantic migration recon | Throwaway `cargo build --workspace` errors named above; `git status --short` after restore shows only plan/log modified. | `cargo build --workspace` green after revert. |
| T2 IR carrier migration branches are not implemented in T1. | Semantic migration | `rg -n "IrMember::Widget|IrMember|widget_children\(" wasamo-ir\src wasamoc\src wasamo-runtime\src` | Owner task = T2; scope = IR schema / textual IR / loader; impact = missed slot metadata or stale IR acceptance; close condition = T2 trap-#1 table + tests. |
| T2 loader stale-form reject branches are not implemented in T1. | Reject / diagnostic | `rg -n "node Cell|prop h-align|prop v-align|parse_node|validate_phase6" wasamo-runtime\src\ir_loader.rs` | Owner task = T2; scope = textual IR loader; impact = stale old-form IR silently accepted; close condition = direct firing loader tests. |
| T3 runtime/layout child-slot record migration and parallel-vector deletion is not implemented in T1. | Structural / parallel-data | `rg -n "cell_placements|zstack_placement|LayoutNode::grid|arrange_grid|build_layout_tree" wasamo-runtime\src` | Owner task = T3; scope = runtime `ChildSlot`, layout child-slot records, and vector deletion; impact = drift class remains; close condition = T3 #1/#2/#3 artifacts and tests. |
| T4 `slot.*` author surface and reject matrix are not implemented in T1. | Author surface / diagnostics | `rg -n "parse_property_bind|PropertyBind|CHILD_PLACEMENT_ATTRS|CELL_ATTRS|h-align|v-align" wasamoc\src` | Owner task = T4; scope = parser/check/lower/fixtures; impact = A13 not reachable; close condition = forcing-table direct tests. |

T1 close gate — behavior / invariant carry scan:

| Behavior / invariant | Closed in T1? | Owner / scope / impact / close condition |
|---|---|---|
| No source migration code may survive T1. | Closed | Verified by source restore and `cargo build --workspace` green; only plan/log remain modified. |
| IR child widgets carry optional slot data at the IR boundary. | Not closed | Owner task = T2; scope = IR type, emit, loader parser; impact = no canonical IR-B record; close = T2 build/tests + trap-#1/#4 artifacts. |
| Runtime and layout child-slot records carry `SlotData` and no Grid placement vector survives mutated paths. | Not closed | Owner task = T3; scope = `WidgetNode.children: Vec<ChildSlot>`, layout child-slot records, `WidgetData::Grid`, loader builder, mutation helpers; impact = parallel drift class remains; close = T3 structural audit and integration fixtures. |
| Dotted `slot.*` is parsed as an attribute key, not expression access. | Not closed | Owner task = T4; scope = parser/check/lower; impact = author surface ambiguous or rejected incorrectly; close = parser/check tests including malformed key cases and value namespace. |
| T4 reject matrix must stay sub-row auditable. | Not closed | Owner task = T4; scope = unknown `slot.*` key, malformed dotted key, stray placement, Cell mixing, placement keyword namespace, constant RHS, ZStack bare reject, defaults; impact = one reject branch can hide behind a broad matrix row; close = T4 forcing table names each subcase and direct test. |
| GUI placement evidence must use a baseline that survives the old-syntax removal. | Not closed | Owner task = T5; scope = assistant screenshot evidence; impact = same-position proof could become unverifiable after T4; close = T5 start gate names baseline and close evidence includes contrast frames. |
| `docs/architecture.md` §6.7.9 illustrative IR spelling must match the landed wrapper shape. | Not closed | Owner task = T7; scope = member-level structural IR code/prose examples; impact = spec-visible design draft remains stale after `IrChildSlot`; close = T7 Moment 2 docs sync updates §6.7.9 or records a different T2 landed shape. |
| Deferred wrapper-rule / VS-2 / VS-3 / bindable-placement triggers remain phase-close residuals. | Not closed | Owner task = T7 / phase-end; scope = candidate ledger and handoff; impact = pre-1.0 residuals lost; close = T7 ledger plus phase-end handoff. |

Carry-forward ownership:

- No owner-unknown unresolved point remains from T1. All open items are
  assigned above to T2, T3, T4, T5, T7, or phase-end.

## CI / verification log

_(append build / test / integration / CI-run evidence and the per-task
close-gate auditable artifacts — trap-#1 call-site audit tables,
trap-#2 side-effect enumerations, trap-#3 parallel-data greps, trap-#4
firing-test names, trap-#7 GUI evidence pointers.)_

### T1 verification

| Command / evidence | Result | Notes |
|---|---|---|
| Throwaway `cargo build --workspace` after `IrMember::Widget { node, slot_data }` | Failed as intended | First stopped at `wasamo-ir/src/lib.rs:225` (`widget_children()`); after temporarily updating that helper, downstream errors named wasamoc emit/lower, runtime loader traversal/parser/build sites, and runtime `zstack_placement` storage sites. Recorded above under "T1 compiler-verification results." |
| `git restore --source=HEAD -- wasamo-ir/src/lib.rs wasamo-runtime/src/widget.rs` | Reverted throwaway source edits | Required escalation because sandbox blocked `.git/index.lock`; target was limited to the two spike-touched source files. |
| `cargo build --workspace` after revert | Green | Workspace returned to buildable state; only `process/milestone-3/phase-7b/implementation/plan.md` and `log.md` remain modified. |
