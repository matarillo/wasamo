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

### T2 start gate — IR + textual-IR migration

Carry-over check from prior tasks:

- T1 left the following T2-owned carry-over items: the `IrChildSlot` /
  `IrSlotData` schema migration; lower / emit / loader migration of every
  `IrMember::Widget` call-site; canonical IR-B textual record parsing and
  emitting; loader stale-form rejects for old `Cell` placement IR and old
  bare ZStack placement props; and the trap-#1 call-site audit table at
  close.
- T1 also left Seam A as an explicit T2 -> T3 carry-forward: T2 may feed
  the legacy runtime storage (`WidgetData::Grid.cell_placements` and
  `WidgetNode.zstack_placement`) from parsed `slot_data`, but T3 owns
  removing the adapter and deleting the parallel storage / layout mirror.
- No owner-unknown item in T1 blocks T2 start. Items owned by T4/T5/T7
  stay outside T2 unless implementation discovers a new dependency.

Critical T2 responsibility re-cut:

- T2 is the IR and textual-IR boundary task, not the runtime structural
  migration. It must make `IrMember::Widget(IrChildSlot)` the only IR
  widget-child shape, make `child { placement <kind> { ... } node ... }`
  the only emitted textual-IR child shape, parse that shape, and reject
  stale old-form textual IR with named loader diagnostics.
- T2 keeps the existing authored surfaces working only as lowering input:
  Grid `Cell` and ZStack bare `h-align` / `v-align` lower into
  `slot_data`. It does not add the new `slot.*` author surface, checker
  matrix, or `.ui` migration; those remain T4-owned.
- T2's Seam A adapter must be obvious and removable: loader/building may
  derive legacy Grid `cell_placements` and ZStack insertion placement
  from `IrSlotData`, but no T2 code may claim the parallel-data drift
  class is closed. T3 owns that structural close.
- T2 should revise plan.md only where it sharpens this boundary. It must
  not reopen DD-fixed outcomes: `slot.` author surface, PM-2, VS-1a
  `SlotData`/`IrSlotData`, IR-B canonical textual skeleton, stale-form
  reject + regenerate, and constant-per-instance placement.

Selected traps and non-applicable reasons:

| Trap | Classification | Reason |
|---|---|---|
| #1 semantic migration | Applies | T2 changes the IR widget-child schema from `IrMember::Widget(IrNode)` to `IrMember::Widget(IrChildSlot)` and adds `IrSlotData`; every traversal, constructor, emitter, parser, validator, and helper such as `widget_children()` must be classified. |
| #2 structural side effects | Not applicable to T2 landing | T2 intentionally does not change runtime child storage or mutation primitives; it bridges `slot_data` back to the legacy runtime paths. T3 owns insert/remove/replace side-effect enumeration. |
| #3 parallel data drift | Not applicable to T2 landing | T2 keeps `WidgetData::Grid.cell_placements` and the layout mirror alive behind Seam A. The remaining drift is a named T3 carry-forward, not a T2 close. |
| #4 untested authored branch | Applies narrowly | T2 adds loader textual-IR parse/reject branches: canonical `child` records, invalid placement metadata, invalid placement kind for parent, and stale old-form placement IR. Direct firing tests are required. |
| #5 carry-forward | Applies | Seam A and any call-site deliberately classified as placement-insensitive must be recorded with downstream owner and close condition. |
| #6 deterministic failure disposition | Conditional | Applies only if an unexpected recurring failure appears during migration or tests; record rerun history and root-cause/disposition before close. |
| #7 GUI positive control | Not applicable | T2 has no GUI-render deliverable; GUI evidence and positive controls are T5/T6. |

Review lane:

- **Full independent review** because T2 is an IR / schema migration.
  The full review must include the trap-#4 branch/test-focused check for
  the new loader reject / diagnostic branches.

Planned proof obligations before implementation:

| Branch / behavior / invariant hypothesis | Category | T2 proof obligation |
|---|---|---|
| `IrMember::Widget` carries an `IrChildSlot` wrapper and no constructor / matcher remains on the tuple-node shape. | Semantic migration | `rg` / `git diff` call-site audit over `IrMember::Widget`, `IrChildSlot`, `IrSlotData`, and `widget_children()`; workspace build green. |
| Existing Grid `Cell` authoring lowers to a Grid `IrSlotData` record and no `Cell` node is emitted in canonical textual IR. | Semantic / IR-B emit | Lower / emit tests that fire the Grid path; output contains `child {` + `placement grid` and excludes `node Cell`. |
| Existing ZStack bare placement authoring lowers to a ZStack `IrSlotData` record and placement props no longer remain on the child node in canonical textual IR. | Semantic / IR-B emit | Lower / emit tests that fire the ZStack path; output contains `placement zstack` and excludes `prop h-align` / `prop v-align` for placement. |
| Runtime textual parser accepts the canonical child-slot record for Grid and ZStack. | Parser / semantic branch | Direct parser tests for `placement grid` and `placement zstack`, including defaulted placement when `placement` is omitted. |
| Loader rejects stale old-form placement IR instead of silently slot-ising it. | Reject / diagnostic | Direct parser/validate tests for `node Cell { prop row ... }` under Grid and child `prop h-align` / `prop v-align` under ZStack; diagnostic names `legacy-placement-ir-form`. |
| Loader rejects malformed or parent-incompatible placement metadata. | Reject / diagnostic | Direct tests for bad placement kind/key/value or placement kind under a non-admitting parent; each branch is named in the close map. |
| Seam A preserves runtime behaviour while leaving drift closure to T3. | Carry-forward / observable invariant | Build-path code derives legacy Grid `cell_placements` and ZStack insertion placement from `IrSlotData`; close artifact names T3 owner for adapter removal and parallel-vector deletion. |

Known carry-forward candidates at T2 start:

| Candidate | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| Seam A adapter removal | T3 | Loader/build path still converts IR `slot_data` into legacy runtime placement storage; leaving it unassigned would make Phase 7b look closed while parallel drift remains. | T3 removes adapter and feeds runtime child-slot records directly. |
| Runtime/layout child-slot records and parallel-vector deletion | T3 | `WidgetData::Grid.cell_placements`, `LayoutNode.cell_placements`, `WidgetNode.zstack_placement`, layout arrange reads, and mutation side effects remain structurally old. | T3 trap-#1/#2/#3 artifacts and green integration/regression tests. |
| New `slot.*` author surface and `.ui` migration | T4 | T2 continues to accept old author input as lowering input; the public surface is not yet flipped. | T4 parser/check/lower matrix, `.ui` sweep, and branch/test-focused review. |
| `docs/architecture.md` §6.7.9 landed IR spelling sync | T7 | T2 may land `IrChildSlot`, diverging from design-draft examples that used a struct variant sketch. | T7 Moment 2 docs sync or explicit disposition if T2 lands a different spelling. |

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

### T2 verification

| Command / evidence | Result | Notes |
|---|---|---|
| `cargo fmt --all` | Green | Formatting completed after the final test additions. |
| `cargo test -p wasamo-runtime --lib` | Green | 414 passed, 0 failed. This directly includes the new loader parser / validation branch tests. |
| `cargo test --workspace` | Green | Workspace tests passed, including `wasamo-ir` 24 tests, `wasamo-runtime` 414 tests, `wasamoc` 356 tests, examples, integration tests, and doctests. Cargo still emits the pre-existing warning that package `wasamo` provides no linkable target. |
| `git diff --check` | Green | No whitespace errors; output contains only Git's CRLF working-copy warnings. |

T2 review follow-up verification:

| Command / evidence | Result | Notes |
|---|---|---|
| `cargo test -p wasamo-runtime --lib` after adding the Grid slot payload roundtrip | Failed once, deterministically | `grid_slot_emit_then_parse_preserves_payload_values` failed because the test-only renderer did not emit Grid `tracks` lines, so validation rejected the rendered Grid as trackless. Disposition: added `render_tracks()` to the test renderer. |
| `cargo test -p wasamo-runtime --lib` after the renderer fix | Green | 418 passed, 0 failed. Includes `grid_slot_negative_row_rejected_at_parse`, `grid_slot_emit_then_parse_preserves_payload_values`, `child_slot_unexpected_token_rejected_at_parse`, and `grid_slot_non_keyword_alignment_rejected_at_parse`. |
| `cargo test --workspace` after the review follow-up | Green | Workspace tests passed, including `wasamo-ir` 24 tests, `wasamo-runtime` 418 tests, and `wasamoc` 356 tests. Cargo still emits the pre-existing `wasamo` linkable-target warning. |
| `cargo fmt --all -- --check` | Green | Post-follow-up formatting check. |
| `git diff --check` | Green | No whitespace errors; output contains only Git's CRLF working-copy warnings. |

T2 trap-#1 call-site audit:

| Site family | Source query / diff cue | T2 disposition |
|---|---|---|
| IR carrier and helpers | `rg -n "pub enum IrSlotData|pub struct IrChildSlot|Widget\\(IrChildSlot\\)|widget_child_slots|widget_children" wasamo-ir\src\lib.rs` -> `IrSlotData` at line 164, `IrChildSlot` at 180, `Widget(IrChildSlot)` at 202, `widget_children()` at 244, `widget_child_slots()` at 251. | Closed in T2. `widget_children()` remains placement-insensitive by returning `slot.node`; `widget_child_slots()` is the placement-sensitive helper. |
| IR constructors / matchers | `rg -n "IrMember::Widget\\(" wasamo-ir\src wasamoc\src wasamo-runtime\src wasamo-runtime\tests` shows every production matcher/constructor uses a slot binding, `child_slot(...)`, `parse_child_slot()`, or an explicit `IrChildSlot`. | Closed in T2 except runtime storage, which is explicitly Seam A / T3. |
| wasamoc lower | `rg -n "fn (lower_grid_cell_slot|lower_zstack_child_slot)|IrSlotData::(Grid|ZStack)|grid_cell_lowers_to_child_slot_with_grid_slot_data|zstack_lowers_child_placement_to_slot_data" wasamoc\src\lower.rs` -> Grid slot lower at 305/340; ZStack slot lower at 351/374; direct tests at 1252 and 1307. | Closed in T2 for old author surfaces lowering to slot data. T4 owns the new `slot.*` author surface. |
| wasamoc emit | `rg -n "fn emit_child_slot|fn emit_slot_data|placement grid|placement zstack|grid_cell_emitted_as_child_slot_with_grid_placement|zstack_emitted_as_node_with_direct_children_in_order" wasamoc\src\emit.rs` -> emitter helpers at 128/138; canonical placement strings at 148/158; direct tests at 706 and 731. | Closed in T2. The emitter canonicalizes widget children as `child { ... node ... }`. |
| runtime parser / validator branches | `rg -n "fn parse_child_slot|fn parse_slot_data|fn parse_grid_slot_data|fn parse_zstack_slot_data|legacy-placement-ir-form|invalid-placement-ir|malformed-placement-ir|child_slot_|grid_slot_|zstack_slot_|grid_legacy_cell|zstack_legacy_bare|grid_rejects_zstack|zstack_rejects_grid|placement_prop_outside" wasamo-runtime\src\ir_loader.rs` -> parser helpers at 2188, 2232, 2245, 2292; named diagnostics at 1025/1029/1032/1157/2203-2352; tests at 6306-6552, 6707-6742, and 7109. | Closed in T2 for canonical parser, stale-form rejects, and parent-compatible placement validation. |
| runtime Seam A adapter | `rg -n "grid_placement_from_slot|zstack_placement_from_slot|zstack_placement_for_parent|widget_child_slots\\(\\)|slot\\.node|cell_placements" wasamo-runtime\src\ir_loader.rs` -> build derives `cell_placements` from child slots at 3582-3586; ZStack placement adapters at 3449 and 3609; Grid adapter at 3622. | Intentionally open as Seam A. T3 owns removing these adapters and deleting runtime/layout parallel placement storage. |

T2 close gate — implemented-branch test map:

| Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
|---|---|---|---|
| `IrMember::Widget` now carries `IrChildSlot`, and `IrSlotData` is the IR carrier for parent-interpreted placement. | Semantic migration | `rg -n "pub enum IrSlotData|pub struct IrChildSlot|Widget\\(IrChildSlot\\)" wasamo-ir\src\lib.rs` | `wasamo_ir::tests::child_slot_carries_optional_slot_data` |
| Placement-insensitive IR traversal still sees widget nodes through `widget_children()`, while placement-sensitive traversal uses `widget_child_slots()`. | Semantic migration | `rg -n "widget_children|widget_child_slots" wasamo-ir\src\lib.rs` | `wasamo_ir::tests::widget_children_excludes_for_body_widgets` plus compile-forced call-site migration. |
| Existing Grid `Cell` authoring lowers to `IrSlotData::Grid` on the child slot. | Semantic branch | `rg -n "lower_grid_cell_slot|IrSlotData::Grid|grid_cell_lowers_to_child_slot_with_grid_slot_data" wasamoc\src\lower.rs` | `wasamoc::lower::tests::grid_cell_lowers_to_child_slot_with_grid_slot_data` |
| Existing ZStack bare `h-align` / `v-align` authoring lowers to `IrSlotData::ZStack` and is stripped from child props. | Semantic branch | `rg -n "lower_zstack_child_slot|IrSlotData::ZStack|zstack_lowers_child_placement_to_slot_data" wasamoc\src\lower.rs` | `wasamoc::lower::tests::zstack_lowers_child_placement_to_slot_data` |
| Grid textual IR emission canonicalizes child placement as `child { placement grid ... node ... }` and no longer emits `node Cell`. | Semantic / textual IR | `rg -n "emit_child_slot|emit_slot_data|placement grid|grid_cell_emitted_as_child_slot_with_grid_placement" wasamoc\src\emit.rs` | `wasamoc::emit::tests::grid_cell_emitted_as_child_slot_with_grid_placement` |
| ZStack textual IR emission canonicalizes child placement as `placement zstack` and no longer emits bare placement props on the child node. | Semantic / textual IR | `rg -n "placement zstack|zstack_emitted_as_node_with_direct_children_in_order" wasamoc\src\emit.rs` | `wasamoc::emit::tests::zstack_emitted_as_node_with_direct_children_in_order` |
| Parser accepts canonical child slots and rejects malformed child-slot shape: missing node, duplicate node, duplicate placement, unknown placement kind, and unexpected token. | Reject / diagnostic | `rg -n "fn parse_child_slot|fn parse_slot_data|malformed-placement-ir|child_slot_" wasamo-runtime\src\ir_loader.rs` | `child_slot_missing_node_rejected_at_parse`; `child_slot_duplicate_node_rejected_at_parse`; `child_slot_duplicate_placement_rejected_at_parse`; `child_slot_unknown_placement_kind_rejected_at_parse`; `child_slot_unexpected_token_rejected_at_parse` |
| Parser rejects malformed Grid placement payload: unknown key, duplicate key, non-positive span, negative row/column, unknown alignment, and non-keyword alignment. | Reject / diagnostic | `rg -n "fn parse_grid_slot_data|expect_nonnegative_u32|expect_positive_u32|expect_alignment|grid_slot_|grid_cell_zero_span|grid_cell_unknown_alignment" wasamo-runtime\src\ir_loader.rs` | `grid_slot_unknown_key_rejected_at_parse`; `grid_slot_duplicate_key_rejected_at_parse`; `grid_cell_zero_span_rejected`; `grid_slot_negative_row_rejected_at_parse`; `grid_cell_unknown_alignment_rejected`; `grid_slot_non_keyword_alignment_rejected_at_parse` |
| Parser rejects malformed ZStack placement payload: unknown key, duplicate key, unknown alignment. | Reject / diagnostic | `rg -n "fn parse_zstack_slot_data|zstack_slot_|zstack_child_unknown_alignment" wasamo-runtime\src\ir_loader.rs` | `zstack_slot_unknown_key_rejected_at_parse`; `zstack_slot_duplicate_key_rejected_at_parse`; `zstack_child_unknown_alignment_rejected_at_validate` |
| Runtime parser preserves non-default Grid slot payload values across emit-style render -> parse. | Semantic positive control | `rg -n "grid_slot_emit_then_parse_preserves_payload_values|render_slot_data|IrSlotData::Grid" wasamo-runtime\src\ir_loader.rs` | `grid_slot_emit_then_parse_preserves_payload_values` |
| Runtime validation reads Grid placement from `IrSlotData::Grid` and preserves range/span/overlap/default semantics. | Semantic / invariant | `rg -n "validate_grid_child_slot|grid_positive_control|grid_cell_.*rejected|grid_same_cell|grid_overlapping|grid_multi_cell|grid_direct_child_without_placement" wasamo-runtime\src\ir_loader.rs` | `grid_positive_control_validates`; `grid_cell_column_out_of_range_rejected`; `grid_cell_row_out_of_range_rejected`; `grid_cell_span_exceeds_grid_rejected`; `grid_same_cell_conflict_rejected`; `grid_overlapping_span_conflict_rejected`; `grid_multi_cell_omitted_placement_collides_at_origin`; `grid_direct_child_without_placement_defaults_to_origin` |
| Runtime validation rejects stale old-form Grid placement IR (`node Cell ...`) with the named stale-form diagnostic. | Reject / diagnostic | `rg -n "legacy-placement-ir-form|grid_legacy_cell|validate_rejects_cell_with_kind_payload" wasamo-runtime\src\ir_loader.rs` | `grid_legacy_cell_zero_content_children_rejected_as_stale_ir`; `grid_legacy_cell_two_content_children_rejected_as_stale_ir`; `validate_rejects_cell_with_kind_payload` |
| Runtime validation reads ZStack placement from `IrSlotData::ZStack` and rejects stale old-form bare placement props on a child node. | Semantic / reject | `rg -n "zstack_positive_control|zstack_legacy_bare|zstack_rejects_grid|zstack_child_zstack_accepts_placement_props" wasamo-runtime\src\ir_loader.rs` | `zstack_positive_control_validates_direct_children`; `zstack_child_zstack_accepts_placement_props`; `zstack_rejects_grid_slot_data`; `zstack_legacy_bare_child_placement_rejected_as_stale_ir` |
| Runtime validation rejects placement metadata under a non-admitting parent or with the wrong placement kind for Grid/ZStack. | Reject / diagnostic | `rg -n "invalid-placement-ir|grid_rejects_zstack|zstack_rejects_grid|placement_prop_outside" wasamo-runtime\src\ir_loader.rs` | `grid_rejects_zstack_slot_data`; `zstack_rejects_grid_slot_data`; `placement_prop_outside_zstack_child_or_grid_cell_rejected_at_validate` |
| Seam A derives legacy runtime Grid / ZStack placement storage from `IrSlotData` while preserving existing runtime behavior. | Transitional semantic bridge | `rg -n "grid_placement_from_slot|zstack_placement_from_slot|zstack_placement_for_parent|cell_placements" wasamo-runtime\src\ir_loader.rs`; `rg -n "#\\[test\\]|fn .*grid|zstack|iteration|slot|placement" wasamo-runtime\tests\grid_layout_integration.rs wasamo-runtime\tests\zstack_layout_integration.rs wasamo-runtime\tests\iteration_mutation_integration.rs` | Existing integration tests in `grid_layout_integration.rs`, `zstack_layout_integration.rs`, and `iteration_mutation_integration.rs`; T3 owns deleting the adapter. |
| New `slot.*` author surface is not implemented in T2. | Author surface | `rg -n "parse_property_bind|PropertyBind|h-align|v-align|Cell" wasamoc\src` remains the old author-surface seam. | Owner task = T4; scope = parser/check/lower/fixtures; impact = DD author surface not yet exposed; close condition = T4 branch/test matrix and `.ui` sweep. |
| Runtime/layout child-slot records and parallel placement vector deletion are not implemented in T2. | Structural / parallel-data | `rg -n "cell_placements|zstack_placement|LayoutNode::grid|arrange_grid|build_layout_tree" wasamo-runtime\src` remains the T3 hotspot set from T1. | Owner task = T3; scope = runtime `ChildSlot`, layout child-slot records, mutation helpers; impact = drift class remains behind Seam A; close condition = T3 trap-#1/#2/#3 close artifacts and tests. |
| GUI visual proof for the author-facing placement change is not implemented in T2. | GUI evidence | T2 changes IR/textual IR only and does not launch GUI hosts. | Owner task = T5; scope = human-visible behavior after T4/T5; impact = no screenshot evidence yet; close condition = launch + screenshot + positive-control analysis. |
| Normative docs sync is not implemented in T2. | Docs | `git diff --name-only` lists no files under `docs/`. | Owner task = T7; scope = `docs/architecture.md` §6.7.9 and related landed spelling; impact = design draft may lag landed `IrChildSlot`; close condition = T7 docs sync or explicit disposition. |

T2 close gate — behavior / invariant carry scan:

| Behavior / invariant | Closed in T2? | Owner / scope / impact / close condition |
|---|---|---|
| Canonical emitted textual IR uses `child { placement <kind> ... node ... }` for widget children with optional parent-interpreted slot data. | Closed | Closed by wasamoc emit tests and runtime parser tests listed above. |
| Placement-free legacy direct `node` children still parse as `IrChildSlot { slot_data: None }`; under Grid, omission defaults to origin / span 1 / stretch. | Closed in T2 as a transitional compatibility behavior | Scope = parser lenience for placement-free children only; stale placement forms still reject. Direct tests: `grid_direct_child_without_placement_defaults_to_origin`, `grid_multi_cell_omitted_placement_collides_at_origin`, and existing ZStack direct-child positive control. |
| Stale old-form placement textual IR is rejected with `legacy-placement-ir-form` rather than normalized. | Closed | Closed by stale Grid `Cell` and stale ZStack bare-child placement tests. |
| Runtime storage still has parallel placement data behind Seam A. | Not closed | Owner task = T3; scope = `WidgetData::Grid.cell_placements`, `WidgetNode.zstack_placement`, layout mirror, insertion / removal / replacement side effects; impact = parallel-data drift risk remains; close condition = T3 removes adapters and records structural side-effect / drift audit. |
| New `slot.*` author syntax and checker diagnostics are not exposed. | Not closed | Owner task = T4; scope = parser/check/lower/fixtures; impact = users still author old `Cell` / bare ZStack placement until T4; close condition = T4 direct branch tests and fixture migration. |
| Docs may still describe the pre-T2 sketch rather than the landed `IrChildSlot` textual / memory shape. | Not closed | Owner task = T7; scope = architecture / DSL / ABI sync if required; impact = reference docs lag implementation; close condition = T7 docs sync or explicit no-change record. |
| GUI-visible placement behavior was not re-proven by screenshot in T2. | Not closed | Owner task = T5; scope = launch + screenshot + positive control after author surface migration; impact = T2 has only logic/integration evidence; close condition = T5 GUI evidence artifact. |

T2 carry-forward ownership:

| Carry-forward | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| Remove Seam A and make runtime/layout child slots the placement carrier. | T3 | Runtime and layout still mirror placement into legacy storage; drift remains possible if later mutation paths are missed. | T3 deletes legacy vectors / fields or proves replacement, updates mutation side-effect enumeration, and passes integration tests. |
| Expose the new direct `slot.*` author surface and reject old author placement forms where required by DD. | T4 | T2 only migrates IR/textual IR and keeps old `.ui` inputs lowering for compatibility. | T4 parser/check/lower matrix and fixture sweep are complete. |
| Produce GUI positive-control evidence for the author-facing behavior. | T5 | T2 has no GUI deliverable. | T5 launch + screenshot + assistant analysis shows the intended screen and a positive control. |
| Sync normative/reference docs with the landed IR/textual shape. | T7 | T2 changed implementation shape; docs were intentionally not edited. | T7 updates or explicitly disposes architecture / DSL references. |

No owner-unknown unresolved point remains from T2. Deterministic-failure
trap #6 did not trigger: after final test additions, `cargo test -p
wasamo-runtime --lib` and `cargo test --workspace` both completed green
without rerun-only failures.

### T3 start gate — runtime `SlotData` storage migration

Carry-over check:

- T2 left the Seam A adapter as explicit T3-owned carry-over:
  `wasamo-runtime/src/ir_loader.rs` still converts `IrSlotData` into
  legacy runtime placement storage via `grid_placement_from_slot`,
  `zstack_placement_from_slot`, `zstack_placement_for_parent`, and
  `insert_child_with_zstack_placement`.
- Runtime storage remains split: `WidgetNode.children` stores bare
  children, `WidgetNode.zstack_placement` stores a ZStack parent-owned
  child-slot fact on the child node, and `WidgetData::Grid.cell_placements`
  stores Grid placement in a parallel vector.
- Layout storage remains split: `LayoutNode.children` stores bare
  children, `LayoutNode.zstack_placement` stores ZStack child placement on
  the child node, and `LayoutNode.cell_placements` is a second Grid
  parallel vector consumed by `arrange_grid`.
- T2 also carried forward T4/T5/T7 items (`slot.*` author surface, GUI
  evidence, docs sync), but none blocks T3 as long as T3 does not expose a
  new author syntax or edit normative docs.

Critical T3 responsibility re-cut:

- T3 is a runtime/layout structural migration, not an IR parser or
  author-surface task. T2 already made textual IR canonical; T3 consumes
  the existing `IrChildSlot.slot_data` and removes the runtime Seam A
  adapter.
- T3's central responsibility is to make the runtime and layout child
  lists carry an explicit child-slot record with `SlotData`, so placement
  moves with the child through insert / remove / replace and through
  layout-tree construction.
- T3 must delete both Grid parallel vectors (`WidgetData::Grid.cell_placements`
  and `LayoutNode.cell_placements`) and the old ZStack child field
  (`WidgetNode.zstack_placement` / `LayoutNode.zstack_placement`). A rename
  that leaves placement on the child widget/node rather than the child slot
  would not close IM-4.
- T3 may add Windows-runtime integration coverage for the new storage
  paths, but pure unit tests remain appropriate only for extracted or
  mirror logic that has no Win32/WinRT dependency.

Selected traps and non-applicable reasons:

| Trap | Classification | Reason |
|---|---|---|
| #1 semantic migration | Applies | T3 changes the runtime/layout carrier shape for child placement: `WidgetNode.children` and `LayoutNode.children` become child-slot records; every traversal, constructor, layout read, sync, dispose, mutation, and loader insertion site must be classified. |
| #2 structural side effects | Applies | T3 changes tree mutation primitives. Insert / remove / replace must preserve Visual sibling order, layout invalidation callers, registry/effect disposal ownership, attached flags, and placement metadata riding on the slot. |
| #3 parallel data drift | Applies | T3 deletes the drift class by removing `WidgetData::Grid.cell_placements` and `LayoutNode.cell_placements`; the close artifact must independently audit each hotspot and prove no mutated path still carries placement in a parallel vector. |
| #4 untested authored branch | Not applicable | T3 does not add parser/checker authored-surface branches or new named diagnostics; T4 owns `slot.*` accept/reject branches. If T3 introduces a new runtime reject branch unexpectedly, the close gate must add a direct firing test and upgrade this classification. |
| #5 carry-forward | Applies | T3 must carry forward any runtime/layout invariant that T4/T5/T7 could trip, especially the absence of Grid structural mutation paths and the requirement that new author forms lower into runtime `SlotData` rather than legacy adapters. |
| #6 deterministic failure disposition | Conditional | Applies only if recurring or retry-sensitive failures appear while running runtime integration / workspace tests. Any such failure must get rerun history and disposition before close. |
| #7 GUI positive control | Not applicable | T3 has no GUI-render evidence deliverable. T5 owns launch + screenshot + positive-control analysis after T4 exposes the author surface. |

Review lane:

- **Full independent review** because T3 is a runtime structural change and
  the close artifacts include trap #1/#2/#3 structural migration proof.

Planned proof obligations before implementation:

| Branch / behavior / invariant hypothesis | Category | T3 proof obligation |
|---|---|---|
| Runtime child storage is `ChildSlot { node, slot_data }`, not a child widget field. | Semantic migration / invariant | `rg` call-site table over `WidgetNode.children`, `zstack_placement`, `SlotData`, insert/remove/replace, dispose, sync, and layout construction; direct tests or integration evidence for mutation paths. |
| Grid placement has no runtime parallel vector. | Parallel-data / invariant | `rg` / diff cue proving `WidgetData::Grid.cell_placements` is deleted and Grid insertion converts each `IrChildSlot` to runtime `SlotData::Grid` on the child slot. |
| Layout placement has no mirror parallel vector. | Parallel-data / invariant | `rg` / diff cue proving `LayoutNode.cell_placements`, `LayoutNode::grid(..., cell_placements)`, and `arrange_grid`'s `children.zip(cell_placements)` are migrated to layout child slots. |
| ZStack placement no longer lives on child nodes. | Semantic migration / invariant | `rg` / diff cue proving `WidgetNode.zstack_placement` and `LayoutNode.zstack_placement` are deleted; arrange reads `SlotData::ZStack` from child slots and defaults omitted placement to center. |
| Insert / remove / replace keep placement tied to the slot while preserving existing side effects. | Structural side effects | Side-effect enumeration at close plus direct integration or justified pure mirror coverage for insert, remove, and replace; removed/detached returned subtrees carry no slot metadata. |
| Conditional / for-generated ZStack children carry `SlotData` through staging -> commit. | Semantic / structural invariant | Existing or new integration tests that mutate generated children under ZStack and observe placement/order/invalidation; no Grid mutation path is introduced. |
| Placement-free parents normalize child slots to `None`. | Semantic invariant | Direct test or audited call site showing generic insertion under non-placement parents does not retain stale slot data. |

Known carry-forward candidates at T3 start:

| Candidate | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| New `slot.*` author surface and in-repo `.ui` migration | T4 | T3 consumes IR slot data only; users still author old ZStack bare placement / Grid `Cell` until T4. | T4 parser/check/lower matrix, fixture sweep, and branch/test-focused review. |
| GUI positive-control evidence | T5 | T3 may prove runtime/layout storage by tests but does not prove visible author-facing placement after `slot.*`. | T5 launch + screenshot + assistant analysis with positive controls. |
| Normative/reference docs sync for landed runtime/layout shape | T7 | T3 may change the precise implementation shape that `docs/architecture.md` must describe at Moment 2. | T7 docs sync or explicit disposition. |
| Grid structural mutation paths remain out of scope | Future phase / T7 ledger | T3 migrates storage but does not add direct `for` / `if` of Grid cells; future Grid mutation work must build on child-slot storage and re-run side-effect gates. | T7 candidate ledger / phase-end handoff records the trigger and close condition. |

### T3 verification

| Command / evidence | Result | Notes |
|---|---|---|
| `cargo test -p wasamo-runtime --lib` after the first migration pass | Failed, compile-forced | The errors named the remaining unmigrated layout test constructors / pushes and the old widget pure-mirror helper imports. Disposition: introduced `ChildSlots` test-compatible wrapper, updated pure mirror tests, and converted Grid/ZStack layout tests to slot-carried placement helpers. |
| `cargo test -p wasamo-runtime --lib` after fixing compile-forced sites | Green | 417 passed, 0 failed. |
| `cargo test -p wasamo-runtime --test zstack_layout_integration` after adding the replace-child integration | Green | 3 passed, including `zstack_replace_child_preserves_child_slot_placement`. |
| `cargo test -p wasamo-runtime` | Green | Runtime unit + Windows integration tests passed. Includes Grid, ZStack, conditional, and iteration mutation integration coverage. |
| `cargo fmt --all -- --check` | Green | Formatting check passed. |
| `cargo test --workspace` | Green | Workspace tests passed, including `wasamo-ir` 24 tests, `wasamo-runtime` 417 lib tests plus integrations, `wasamoc` 356 tests, examples, integration tests, and doctests. Cargo still emits the pre-existing warning that package `wasamo` provides no linkable target. |
| `git diff --check` | Green | No whitespace errors; output contains only Git's CRLF working-copy warnings. |
| `rg -n "cell_placements\|zstack_placement\|insert_child_with_zstack_placement\|zstack_placement_for_parent" wasamo-runtime\src` | No matches | Confirms the old runtime/layout storage names and old ZStack insertion adapter no longer survive in source. |

T3 trap-#1 call-site audit:

| Site family | Source query / diff cue | T3 disposition |
|---|---|---|
| Runtime child storage carrier | `rg -n "pub struct ChildSlot|pub children: Vec<ChildSlot>|slot_data: Option<SlotData>|impl (Deref|AsRef)<WidgetNode> for ChildSlot" wasamo-runtime\src\widget.rs` | Closed in T3. Runtime child slots now carry `{ node, slot_data }`; deref / `AsRef` preserve existing child-node traversal ergonomics without moving placement onto the child widget. |
| Runtime insert/remove/replace mutation primitives | `rg -n "insert_child_with_slot_data|fn insert_child_inner|fn remove_child|fn replace_child|ChildSlot::new|replacement_slot_data|into_node" wasamo-runtime\src\widget.rs` | Closed in T3. Insert stores slot data, remove drops slot metadata by returning the bare node, and replace preserves the existing slot data while replacing the node. |
| Runtime layout-tree construction | `rg -n "fn build_layout_child_slots|LayoutChildSlot::new\\(slot.build_layout_tree\\(\\), slot.slot_data\\)|WidgetData::Grid \\{ columns, rows \\}|LayoutNode::grid\\(" wasamo-runtime\src\widget.rs` | Closed in T3. Runtime child slots are copied into layout child slots; Grid no longer copies a parent vector. |
| Loader child materialisation | `rg -n "slot_data_for_parent|insert_child_with_slot_data|grid_placement_from_slot|zstack_payload_from_ir_slot|WidgetNode::grid\\(compositor, columns, rows\\)" wasamo-runtime\src\ir_loader.rs` | Closed in T3. T2's Seam A adapter is removed; loader converts `IrSlotData` into runtime `SlotData` at insertion time for static, conditional, and `for`-generated children. |
| Loader Grid build-path unification | `git diff eb924b8^..eb924b8 -- wasamo-runtime\src\ir_loader.rs`; `rg -n "validate_grid_invariants|IrMember::ControlFlow|append_static_member|slot_data_for_parent_kind" wasamo-runtime\src\ir_loader.rs` | Closed in T3 + review follow-up. The old Grid-only static child construction path was unified with the generic child append path; safety for Grid control-flow remains at the validator boundary (`validate_grid_invariants` rejects direct Grid `if` / `for` before build). |
| Layout child storage carrier | `rg -n "pub enum SlotData|pub struct LayoutChildSlot|pub struct ChildSlots|pub children: ChildSlots|push_slot" wasamo-runtime\src\layout.rs` | Closed in T3. Layout children are explicit child slots; test-only bare pushes normalize placement to `None`. |
| Layout Grid read path | `rg -n "fn arrange_grid|Some\\(SlotData::Grid|CellPlacement::default_grid|pub fn grid\\(columns: Vec<TrackSize>, rows: Vec<TrackSize>\\)" wasamo-runtime\src\layout.rs` | Closed in T3. Grid arrange reads placement from each child slot and defaults omitted placement to origin/span-1/stretch. |
| Layout ZStack read path | `rg -n "fn arrange_zstack|Some\\(SlotData::ZStack|ZStackPlacement::centered|with_zplace" wasamo-runtime\src\layout.rs` | Closed in T3. ZStack arrange reads placement from each child slot and defaults omitted placement to center/center. |

T3 trap-#2 structural side-effect enumeration:

| Mutated path / side effect | T3 disposition | Direct evidence |
|---|---|---|
| Child list splice | `WidgetNode.children` stores `ChildSlot`, so slot metadata is inserted / removed / replaced atomically with the child entry. | `widget::tests::insert_stores_zstack_slot_data_on_the_slot`; `widget::tests::insert_stores_grid_slot_data_on_the_slot`; `widget::tests::remove_returns_detached_subtree_without_slot_metadata`; `widget::tests::replace_preserves_existing_slot_data_on_new_child`. |
| Visual sibling order | Existing `InsertAtTop` / `InsertBelow` / remove / top insert behavior is unchanged; slot wrapping does not change the Visual used for ordering (`ChildSlot` derefs to `WidgetNode`). | `conditional_toggle_preserves_declared_visual_order_and_disposes_registry`; `reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity`; `zstack_replace_child_preserves_child_slot_placement` rechecks live Visual order after replacement. |
| Layout invalidation | Existing structural mutation callers still mark layout dirty after conditional / for mutations; public insert/remove/replace keep their previous mutation API shape. | `conditional_toggle_drains_fresh_subtree_effects_before_return`; `reactive_for_zstack_tail_append_uses_child_carried_placement`; `staged_for_insert_commit_failure_rolls_back_partial_inserts`. |
| Widget-pointer registry / effect ownership | Removal and rollback still return/destroy the bare subtree; child-slot metadata is dropped before `widget_destroy`, while binding disposal / registry severing remains subtree-owned. | `destroy_child_binding_also_stopped`; `conditional_toggle_preserves_declared_visual_order_and_disposes_registry`; `staged_for_insert_build_failure_leaves_tree_unchanged`; `staged_for_insert_commit_failure_rolls_back_partial_inserts`. |
| Placement ownership | Placement is parent/slot-owned, not child-widget-owned. Removed returned subtrees carry no slot metadata; replacements inherit the slot metadata of the replaced position. | `remove_returns_detached_subtree_without_slot_metadata`; `replace_preserves_existing_slot_data_on_new_child`; `zstack_replace_child_preserves_child_slot_placement`. |
| Grid build-path side effect | Grid child construction now uses the generic child append path instead of a Grid-only static loop. | `validate_rejects_direct_conditional_grid_member`; `validate_rejects_direct_conditional_cell_member`; `for_member_rejects_direct_disallowed_containers`; `grid_rooted_fixture_lays_out_cells_through_visual_tree`. |

T3 trap-#3 parallel-data drift audit:

| Parallel-data hotspot | Source query / diff cue | T3 disposition |
|---|---|---|
| `WidgetData::Grid.cell_placements` | `rg -n "cell_placements" wasamo-runtime\src` -> no matches | Deleted. Grid variant now stores only `columns` / `rows`; child placement rides runtime `ChildSlot.slot_data`. |
| `LayoutNode.cell_placements` | `rg -n "cell_placements" wasamo-runtime\src` -> no matches | Deleted. Layout Grid placement rides `LayoutChildSlot.slot_data`. |
| `LayoutNode::grid` constructor signature | `rg -n "pub fn grid\\(columns: Vec<TrackSize>, rows: Vec<TrackSize>\\)" wasamo-runtime\src\layout.rs` | Migrated. No placement vector argument remains. |
| `arrange_grid` zip over parallel vector | `rg -n "fn arrange_grid|zip\\(|Some\\(SlotData::Grid|CellPlacement::default_grid" wasamo-runtime\src\layout.rs` | Migrated. `arrange_grid` iterates child slots and reads `SlotData::Grid` or the Grid default. |
| `build_layout_tree` copy into layout mirror | `rg -n "fn build_layout_child_slots|LayoutChildSlot::new\\(slot.build_layout_tree\\(\\), slot.slot_data\\)" wasamo-runtime\src\widget.rs` | Migrated. Runtime slot data is copied directly into layout slot data. |
| Old ZStack child field / insertion adapter | `rg -n "zstack_placement|insert_child_with_zstack_placement|zstack_placement_for_parent" wasamo-runtime\src` -> no matches | Deleted. ZStack placement is `SlotData::ZStack` on the child slot. |

T3 close gate — implemented-branch test map:

| Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
|---|---|---|---|
| Runtime children are explicit `ChildSlot` records carrying `Option<SlotData>`. | Semantic migration | `rg -n "pub struct ChildSlot|pub children: Vec<ChildSlot>|slot_data: Option<SlotData>" wasamo-runtime\src\widget.rs` | `widget::tests::insert_stores_zstack_slot_data_on_the_slot`; `widget::tests::insert_stores_grid_slot_data_on_the_slot` |
| Widget-level placement-free insertion stores `None` when the caller supplies no slot data. | Semantic invariant | `rg -n "insert_child\\(|insert_child_with_slot_data|ChildSlot::new\\(child, slot_data\\)" wasamo-runtime\src\widget.rs` | `widget::tests::non_placement_parent_insert_normalizes_slot_data_to_none` |
| Loader parent-kind slot mapping returns ZStack, Grid, or `None` for non-placement parents and applies omitted-payload defaults. | Semantic invariant | `rg -n "fn slot_data_for_parent_kind|slot_data_for_parent_kind\\(" wasamo-runtime\src\ir_loader.rs` | `ir_loader::tests::slot_data_for_parent_kind_maps_zstack_slot_payload`; `ir_loader::tests::slot_data_for_parent_kind_maps_grid_slot_payload`; `ir_loader::tests::slot_data_for_parent_kind_normalizes_non_placement_parent_to_none`; `ir_loader::tests::slot_data_for_parent_kind_defaults_missing_zstack_payload_to_center`; `ir_loader::tests::slot_data_for_parent_kind_defaults_missing_grid_payload_to_origin_stretch` |
| Runtime remove returns a bare detached subtree and drops slot metadata. | Structural side effect | `rg -n "fn remove_child|remove\\(index\\)\\.into_node\\(\\)|removed.attached = false" wasamo-runtime\src\widget.rs` | `widget::tests::remove_returns_detached_subtree_without_slot_metadata`; existing `remove_returns_detached` |
| Runtime replace preserves the slot metadata while swapping the child node. | Structural side effect | `rg -n "fn replace_child|replacement_slot_data|ChildSlot::new\\(new_child, replacement_slot_data\\)" wasamo-runtime\src\widget.rs` | `widget::tests::replace_preserves_existing_slot_data_on_new_child`; `zstack_replace_child_preserves_child_slot_placement` |
| Loader static children insert runtime `SlotData` instead of calling a ZStack-only adapter. | Semantic migration | `rg -n "append_static_member|insert_child_with_slot_data|slot_data_for_parent\\(parent, child\\)" wasamo-runtime\src\ir_loader.rs` | Covered by `grid_rooted_fixture_lays_out_cells_through_visual_tree`; `zstack_rooted_fixture_preserves_live_visual_order_and_clip` |
| Loader conditional children carry slot data through remove/reinsert. | Structural / semantic | `rg -n "mutate_conditional_subtree|slot_data_for_parent\\(parent, body\\)|remove_structural_child" wasamo-runtime\src\ir_loader.rs` | `conditional_zstack_reinsert_uses_declared_placement_metadata`; `conditional_toggle_preserves_declared_visual_order_and_disposes_registry` |
| Loader `for`-generated ZStack children carry slot data through staging -> commit and rollback. | Structural / semantic | `rg -n "mutate_for_loop_subtree|let slot_data = slot_data_for_parent|insert_structural_child\\(parent, insert_index, child, slot_data\\)" wasamo-runtime\src\ir_loader.rs` | `reactive_for_zstack_tail_append_uses_child_carried_placement`; `static_for_under_zstack_preserves_child_carried_placement`; `staged_for_insert_commit_failure_rolls_back_partial_inserts` |
| Grid runtime constructor no longer accepts a placement vector. | Parallel-data | `rg -n "WidgetNode::grid\\(compositor, columns, rows\\)|pub\\(crate\\) fn grid\\(" wasamo-runtime\src\ir_loader.rs wasamo-runtime\src\widget.rs` | `grid_rooted_fixture_lays_out_cells_through_visual_tree`; `grid_vstack_root_fixture_pins_production_root_shape` |
| Layout children are explicit `LayoutChildSlot` records carrying `Option<SlotData>`. | Semantic migration | `rg -n "pub struct LayoutChildSlot|pub struct ChildSlots|pub children: ChildSlots" wasamo-runtime\src\layout.rs` | `layout::tests::grid_arrange_fixed_cell_rectangles`; `layout::tests::zstack_arrange_alignment_overrides` |
| Layout Grid arrange reads `SlotData::Grid` from each child slot and applies defaults for omitted placement. | Semantic / size behavior | `rg -n "fn arrange_grid|Some\\(SlotData::Grid|CellPlacement::default_grid|grid_arrange_bare_child_slot_uses_default_placement" wasamo-runtime\src\layout.rs` | `layout::tests::grid_arrange_alignment_within_cell`; `layout::tests::grid_arrange_preserves_document_order`; `layout::tests::grid_arrange_bare_child_slot_uses_default_placement`; runtime loader tests `grid_direct_child_without_placement_defaults_to_origin`, `grid_multi_cell_omitted_placement_collides_at_origin` |
| Layout ZStack arrange reads `SlotData::ZStack` from each child slot and applies center defaults for omitted placement. | Semantic / size behavior | `rg -n "fn arrange_zstack|Some\\(SlotData::ZStack|ZStackPlacement::centered" wasamo-runtime\src\layout.rs` | `layout::tests::zstack_arrange_alignment_overrides`; `layout::tests::zstack_defaults_to_fill_fill_and_centers_children` |
| Old parallel placement storage and old ZStack child field are absent from runtime/layout source. | Parallel-data invariant | `rg -n "cell_placements\|zstack_placement\|insert_child_with_zstack_placement\|zstack_placement_for_parent" wasamo-runtime\src` -> no matches | Owner closed in T3; grep is the direct forcing artifact. |
| New `slot.*` author surface is not implemented in T3. | Author surface | `rg -n "parse_property_bind|PropertyBind|slot\\.|h-align|v-align|Cell" wasamoc\src` remains the T4 hotspot set. | Owner task = T4; scope = parser/check/lower/fixtures; impact = DD author surface not yet exposed; close condition = T4 branch/test matrix and `.ui` sweep. |
| GUI positive-control evidence is not implemented in T3. | GUI evidence | T3 changes runtime/layout storage and does not launch GUI hosts for screenshot evidence. | Owner task = T5; scope = launch + screenshot + positive controls after T4; impact = no assistant visual proof yet; close condition = T5 evidence files + analysis. |
| Normative docs sync is not implemented in T3. | Docs | `git diff --name-only` lists no files under `docs/`. | Owner task = T7; scope = `docs/architecture.md` storage / IR spelling sync; impact = reference docs lag implementation; close condition = T7 Moment 2 docs sync or explicit disposition. |

T3 close gate — behavior / invariant carry scan:

| Behavior / invariant | Closed in T3? | Owner / scope / impact / close condition |
|---|---|---|
| Runtime and layout child-slot records are now the only placement carriers for Grid/ZStack runtime layout. | Closed | Closed by source shape (`ChildSlot`, `LayoutChildSlot`, `SlotData`) and old-storage grep with no matches. |
| Non-placement parents normalize child slot data to `None` at the loader mapping boundary. | Closed | Closed by `slot_data_for_parent_kind_normalizes_non_placement_parent_to_none`; the earlier mirror-only row was corrected in the review follow-up. |
| Grid omitted placement still defaults to origin / span 1 / stretch. | Closed | Closed by `CellPlacement::default_grid`, `slot_data_for_parent_kind_defaults_missing_grid_payload_to_origin_stretch`, `arrange_grid`, and direct loader/layout tests listed above. |
| ZStack omitted placement still defaults to center / center. | Closed | Closed by `ZStackPlacement::centered`, `slot_data_for_parent_kind_defaults_missing_zstack_payload_to_center`, `arrange_zstack`, and direct layout/runtime integration tests listed above. |
| Remove/destroy leaks no slot metadata. | Closed | Runtime `remove_child` returns `ChildSlot::into_node()`, dropping slot data; covered by pure mirror and integration destruction tests. |
| Replacement preserves the parent slot metadata for the new child. | Closed | Covered by pure mirror and `zstack_replace_child_preserves_child_slot_placement`. |
| Grid structural mutation paths remain out of scope even though storage has migrated. | Not closed in T3, intentionally deferred | Owner task = T7 / phase-end handoff; scope = future direct `for` / `if` of Grid cells; impact = future mutation work must re-run trap #2/#3 on Grid; close condition = T7 candidate ledger records the trigger and future owner. |
| New direct `slot.*` author syntax and ZStack bare-placement reject are not exposed yet. | Not closed | Owner task = T4; scope = parser/check/lower/fixtures; impact = users still author old ZStack bare placement until T4; close condition = T4 direct branch tests and fixture migration. |
| GUI-visible same-position proof has not run after storage migration. | Not closed | Owner task = T5; scope = assistant screenshot + positive controls after T4; impact = storage has test evidence but no visual proof; close condition = T5 evidence artifacts. |
| Docs may still describe design-draft storage spelling rather than landed runtime/layout child-slot wrappers. | Not closed | Owner task = T7; scope = architecture / DSL implementation sync; impact = reference docs lag implementation; close condition = T7 docs sync or explicit no-change record. |

T3 carry-forward ownership:

| Carry-forward | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| Expose the new direct `slot.*` author surface and reject old author placement forms where required by DD. | T4 | Runtime/layout storage is now final, but author syntax is still the old surface. | T4 parser/check/lower matrix and fixture sweep are complete. |
| Produce GUI positive-control evidence against final storage and author surface. | T5 | T3 has runtime/integration evidence only. | T5 launch + screenshot + assistant analysis shows ZStack and Grid positive controls. |
| Sync normative/reference docs with landed runtime/layout child-slot shape. | T7 | T3 changed implementation shape; docs were intentionally not edited. | T7 updates or explicitly disposes architecture / DSL references. |
| Record the future Grid structural-mutation trigger after storage migration. | T7 / phase-end | Storage migration is complete, but future Grid mutation paths still need a re-trigger rule. | T7 candidate ledger / phase-end handoff records scope, impact, and close condition. |

No owner-unknown unresolved point remains from T3. Deterministic-failure
trap #6 did not trigger: the initial compile failure was the expected
semantic-migration enumeration, not a flaky runtime/test failure; all
post-fix reruns listed above completed green.

### T4 start gate — author surface

Carry-over check:

- T3 carry-forward assigns the new direct `slot.*` author surface and
  ZStack bare-placement reject to T4. This includes parser / check /
  lower / fixture sweep, with impact that users are still on the old
  ZStack bare placement surface until T4 closes.
- T3 carry-forward for GUI positive-control evidence remains T5-owned;
  T4 must migrate examples / fixtures but does not prove visible render.
- T3 carry-forward for normative/reference docs sync and future Grid
  structural-mutation trigger remains T7 / phase-end-owned; T4 must not
  silently edit those docs as part of the author-surface commit unless a
  new discovered divergence forces a plan revision.

Critical T4 responsibility re-cut:

- T4 is the author-surface flip over the already-final T2/T3 IR/runtime
  storage, not another carrier migration. Its durable output is a single
  accepted author surface (`slot.*` for direct parent-owned placement),
  direct branch tests for the PM-2 matrix, lowering into `IrSlotData`,
  and an in-repo `.ui` migration away from ZStack bare placement.
- The parser stores dotted placement keys in the existing AST shape:
  `Member::PropertyBind { name: "slot.<key>", value, span }`. No new AST
  member variant is planned; malformed dotted shapes are parser-stage
  rejects.
- Grid retains `Cell` as grouped sugar. Direct `slot.*` on a Grid child
  is added as an equal accepted form; mixing `slot.*` inside a `Cell`
  remains a distinct reject from `slot.*` on a widget nested inside a
  `Cell`.
- T4 does not own assistant GUI evidence, owner GUI smoke, or Moment 2
  docs sync. Those remain T5 / T6 / T7 carry-forward items.

Selected traps and non-applicable reasons:

| Trap | Classification | Reason |
|---|---|---|
| #1 semantic migration | Not applicable | T4 does not change an enum / schema carrier shape. It extends parser/check/lower behavior over the existing `PropertyBind` and `IrChildSlot` / `IrSlotData` carriers. |
| #2 structural side effects | Not applicable | No runtime tree mutation, layout invalidation, Visual order, registry, or effect ownership path changes are planned. |
| #3 parallel data drift | Not applicable | T3 already deleted parallel placement storage. T4 lowers author syntax into the unified child-slot data and introduces no parallel vector / cache. |
| #4 untested authored branch | Applies | T4 adds parser rejects, checker diagnostics, accept branches, and lowering branches for `slot.*`, strict mixing, stale ZStack bare placement, constant-only RHS, value namespace, and placement-vs-unknown-property splits; every branch needs a direct firing test. |
| #5 carry-forward | Applies | T4 leaves GUI proof, docs sync, and future Grid mutation policy to later tasks; any branch not implemented in T4 must have owner / scope / impact / close condition. |
| #6 deterministic failure disposition | Conditional | Applies only if a recurring build / test / runtime failure appears during T4. A deterministic failure cannot be re-rolled to green without disposition. |
| #7 GUI positive control | Not applicable | T4's deliverable is compiler/parser/check/lower behavior and fixture migration, not GUI-render evidence. T5 owns launch + screenshot + positive controls. |

Review lane:

- **Branch-test-focused review** because T4 adds diagnostic / reject /
  accept / lowering branches but does not perform schema migration,
  runtime structural change, or GUI-render evidence. The review must
  check the implemented-branch test map row by row.

Planned proof obligations before implementation:

| Branch / behavior / invariant hypothesis | Category | T4 proof obligation |
|---|---|---|
| `slot.<key>:` parses as a property-bind key and is not expression member access. | Parser / semantic | Parser tests for canonical `slot.h-align`; parser rejects for `slot:`, `slot..h-align`, and `slot.`. |
| Grid direct `slot.row` / `slot.column` / `slot.row-span` / `slot.column-span` / `slot.h-align` / `slot.v-align` are accepted on direct child widgets. | Semantic / accept | Checker positive and lowering tests showing direct Grid child `IrSlotData::Grid`. |
| Grid `Cell` remains accepted grouped sugar and lowers to the same `IrSlotData::Grid` shape. | Semantic invariant | Existing Cell tests plus a direct equivalence test between `Cell` and direct `slot.*`. |
| ZStack direct children accept only `slot.h-align` / `slot.v-align`; defaults remain center/center when one axis is omitted. | Semantic / accept | Checker and lowering tests for direct ZStack `slot.*` and default preservation. |
| `slot.*` inside `Cell`'s own attrs is a strict PM-2 mixing reject. | Diagnostic / reject | Direct checker test whose message names mixing, distinct from non-admitting-parent. |
| `slot.*` on a widget nested inside `Cell` is a non-admitting-parent reject, not the mixing diagnostic. | Diagnostic / reject | Direct checker test whose message names non-admitting parent / inside `Cell` content. |
| `slot.*` under non-admitting parents and at component / host level is rejected distinctly from unknown widget properties where practical. | Diagnostic / reject | Direct checker tests for VStack and component-level placement. |
| Unknown slot keys are rejected by the slot-key path, while unknown widget props still use ordinary unknown-property diagnostics. | Diagnostic / reject | Direct tests for `slot.foo` and `foo:` on a widget. |
| Slot placement RHS is constant per instance and uses the placement keyword namespace; state-backed `end` must not override keyword `end`. | Diagnostic / semantic | Direct tests for state-backed placement RHS reject and state named `end` still allowing `slot.h-align: end`. |
| ZStack bare `h-align` / `v-align` on direct children is rejected; no long-lived alias remains. | Diagnostic / reject | Direct checker test and `.ui` sweep with no surviving bare ZStack placement. |
| CF-1 body-root placement inherits the static child surface. | Semantic | Lowering test for `if` or `for` body root under ZStack / Grid carrying slot data. |
| In-repo `.ui` files no longer use ZStack bare placement and still compile through the workspace build. | Observable behavior | Greppable sweep plus `cargo test --workspace` / example build evidence. |

Known carry-forward candidates at T4 start:

| Candidate | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| Assistant GUI positive-control proof | T5 | T4 changes author syntax and examples but does not produce screenshot evidence; visible placement could still regress despite compiler tests. | T5 evidence files + analysis show ZStack and Grid positive controls. |
| Owner manual GUI smoke | T6 | Human-visible validation remains separate from assistant screenshots. | Owner accepts or records fail/fix/re-run. |
| Moment 2 docs sync for landed `slot.*` surface and child-slot implementation | T7 | T4 implementation may diverge in small spelling/details from Moment 1 prose. | T7 updates `docs/dsl_spec.md` / `docs/architecture.md` or records explicit disposition. |
| Future Grid structural-mutation trigger | T7 / phase-end | T4 admits direct Grid child placement but does not add direct Grid `if` / `for` mutation paths. | T7 candidate ledger / phase-end handoff records trigger, scope, impact, and close condition. |

### T4 close gate — author surface

T4 verification:

| Command / evidence | Result | Notes |
|---|---|---|
| `cargo test -p wasamoc` | Green | 374 lib tests + roundtrip tests passed after parser / check / lower / emit updates. |
| `cargo test --workspace` initial T4 run | Failed deterministically | `conditional_zstack_reinsert_uses_declared_placement_metadata` failed because a runtime integration authored-source fixture still used ZStack bare `h-align` / `v-align`. Disposition: migrated runtime authored-source fixtures to `slot.*`; this was not a flaky failure. |
| `cargo test -p wasamo-runtime --test conditional_toggle_integration` | Green | Direct rerun of the previously failing deterministic case passed after fixture migration. |
| `cargo test --workspace` final T4 run | Green | Workspace tests passed, including `wasamoc`, runtime unit tests, Windows integration tests, examples, and doctests. Cargo still emits the pre-existing warning that package `wasamo` provides no linkable target. |
| `rg -n "slot\\.h-align\|slot\\.v-align\|h-align:\|v-align:" examples wasamo-runtime\tests wasamoc\src -g "*.ui" -g "*.rs"` | Reviewed | Remaining bare `h-align` / `v-align` hits are retained Grid `Cell` tests / fixtures, textual-IR emitter strings, and explicit bare-reject tests. ZStack authored fixtures and examples use `slot.*`. |
| T4 branch-test-focused review | Insufficient; follow-up applied | Review found missing direct firing tests for Grid direct slot value-validation sub-branches and ZStack value namespace. Disposition: added the tests listed in the branch map below and split the malformed parser assertions by subcase. |
| `cargo test -p wasamoc --lib` after review follow-up | Green | 382 passed, 0 failed. |
| `cargo test --workspace` after review follow-up | Green | Workspace tests passed after the added direct branch tests and retrospective/log updates. Cargo still emits the pre-existing warning that package `wasamo` provides no linkable target. |

T4 close gate — implemented-branch test map:

| Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
|---|---|---|---|
| `slot.<key>:` parses into existing `Member::PropertyBind { name: "slot.<key>", ... }`. | Parser / semantic | `rg -n "parse_slot_property_bind\|slot_dotted_property_bind_canonicalizes_name" wasamoc\src\parser.rs` | `parser::tests::slot_dotted_property_bind_canonicalizes_name` |
| Malformed `slot:` / `slot..h-align` / `slot.` are parser-stage rejects. | Parser reject | `rg -n "malformed_slot_property_keys_rejected_at_parse\|malformed slot property key" wasamoc\src\parser.rs` | `parser::tests::malformed_slot_property_keys_rejected_at_parse` |
| Grid direct child `slot.row` / `slot.column` / `slot.h-align` is accepted. | Semantic / accept | `rg -n "check_grid_direct_child_slot\|grid_direct_slot_child_accepted" wasamoc\src\check.rs` | `check::tests::grid_direct_slot_child_accepted` |
| Grid direct child with no `slot.*` uses child-slot default placement instead of the old non-Cell reject. | Semantic / default | `rg -n "GridPlacementChild::Direct\|grid_direct_child_without_slot_uses_default_placement" wasamoc\src\check.rs` | `check::tests::grid_direct_child_without_slot_uses_default_placement`; loader default remains covered by `ir_loader::tests::grid_direct_child_without_placement_defaults_to_origin` |
| Grid direct slot and retained `Cell` grouped form can coexist and both participate in overlap checks. | Semantic / invariant | `rg -n "GridPlacementChild\|check_cell_overlaps\|grid_cell_and_direct_slot_forms_can_coexist\|grid_direct_slot_overlaps_cell_rejected" wasamoc\src\check.rs` | `check::tests::grid_cell_and_direct_slot_forms_can_coexist`; `check::tests::grid_direct_slot_overlaps_cell_rejected` |
| Grid direct `slot.*` lowers to the same `IrSlotData::Grid` as `Cell`; slot props are stripped from the child node. | Semantic lowering | `rg -n "lower_grid_direct_child_slot\|grid_direct_slot_lowers_to_same_grid_slot_data_as_cell" wasamoc\src\lower.rs` | `lower::tests::grid_direct_slot_lowers_to_same_grid_slot_data_as_cell` |
| Unknown Grid slot key rejects through the slot-key diagnostic path. | Diagnostic reject | `rg -n "unknown `.*slot key\|grid_direct_slot_unknown_key_rejected" wasamoc\src\check.rs` | `check::tests::grid_direct_slot_unknown_key_rejected` |
| Grid direct slot placement RHS is constant per instance for integer placement. | Diagnostic reject | `rg -n "grid_direct_slot_constant_rhs_rejected\|must be a non-negative integer literal" wasamoc\src\check.rs` | `check::tests::grid_direct_slot_constant_rhs_rejected` |
| Grid direct slot row / column out-of-range checks fire independently. | Diagnostic reject | `rg -n "grid_direct_slot_(row_out_of_range|column_out_of_range)_rejected\|placement exceeds the grid" wasamoc\src\check.rs` | `check::tests::grid_direct_slot_row_out_of_range_rejected`; `check::tests::grid_direct_slot_column_out_of_range_rejected` |
| Grid direct slot negative index literal rejects through the direct-slot index branch. | Diagnostic reject | `rg -n "grid_direct_slot_negative_index_rejected\|must be a non-negative integer \\(got" wasamoc\src\check.rs` | `check::tests::grid_direct_slot_negative_index_rejected` |
| Grid direct slot zero / non-literal span rejects through the direct-slot span branches. | Diagnostic reject | `rg -n "grid_direct_slot_(zero_span|non_literal_span)_rejected\|must be a positive integer" wasamoc\src\check.rs` | `check::tests::grid_direct_slot_zero_span_rejected`; `check::tests::grid_direct_slot_non_literal_span_rejected` |
| Grid direct slot bad alignment keyword and non-keyword alignment reject through separate branches. | Diagnostic reject | `rg -n "grid_direct_slot_(bad_alignment_keyword|non_keyword_alignment)_rejected\|slot.h-align" wasamoc\src\check.rs` | `check::tests::grid_direct_slot_bad_alignment_keyword_rejected`; `check::tests::grid_direct_slot_non_keyword_alignment_rejected` |
| Placement value namespace treats `end` as the alignment keyword even when a state named `end` exists. | Semantic branch | `rg -n "(grid_direct_slot|zstack_direct_slot)_value_namespace_prefers_alignment_keyword" wasamoc\src\check.rs` | `check::tests::grid_direct_slot_value_namespace_prefers_alignment_keyword`; `check::tests::zstack_direct_slot_value_namespace_prefers_alignment_keyword` |
| `slot.*` among a `Cell` node's own attrs is the strict PM-2 mixing reject. | Diagnostic reject | `rg -n "strict PM-2 mixing\|slot_property_inside_cell_attrs_is_mixing_reject" wasamoc\src\check.rs` | `check::tests::slot_property_inside_cell_attrs_is_mixing_reject` |
| `slot.*` on a widget inside `Cell` content is the non-admitting-parent reject, distinct from mixing. | Diagnostic reject | `rg -n "check_slot_property_outside_parent\|slot_property_inside_cell_content_is_non_admitting_parent_reject" wasamoc\src\check.rs` | `check::tests::slot_property_inside_cell_content_is_non_admitting_parent_reject` |
| `slot.*` under a non-admitting parent and at component level is rejected as parent-owned placement data. | Diagnostic reject | `rg -n "slot_property_under_non_admitting_parent_rejected\|slot_property_at_component_level_rejected" wasamoc\src\check.rs` | `check::tests::slot_property_under_non_admitting_parent_rejected`; `check::tests::slot_property_at_component_level_rejected` |
| ZStack direct child accepts only `slot.h-align` / `slot.v-align`. | Semantic / accept | `rg -n "ZSTACK_SLOT_KEYS\|zstack_direct_child_alignment_accepted" wasamoc\src\check.rs` | `check::tests::zstack_direct_child_alignment_accepted` |
| ZStack unknown slot key rejects through the slot-key diagnostic path. | Diagnostic reject | `rg -n "zstack_slot_unknown_key_rejected\|unknown `ZStack` slot key" wasamoc\src\check.rs` | `check::tests::zstack_slot_unknown_key_rejected` |
| ZStack slot alignment RHS is constant keyword placement data, not a state-backed binding. | Diagnostic reject | `rg -n "zstack_slot_constant_rhs_rejected\|ZStack child `slot.h-align`" wasamoc\src\check.rs` | `check::tests::zstack_slot_constant_rhs_rejected`; subcases `check::tests::zstack_child_bad_alignment_value_rejected`, `check::tests::zstack_child_non_keyword_alignment_value_rejected` |
| ZStack bare child `h-align` / `v-align` is rejected; no long-lived alias remains. | Diagnostic reject | `rg -n "ZStack child bare\|zstack_child_bare_alignment_rejected" wasamoc\src\check.rs` | `check::tests::zstack_child_bare_alignment_rejected` |
| ZStack `slot.*` lowers into `IrSlotData::ZStack`, strips `slot.*` props, and defaults omitted axis to center. | Semantic lowering | `rg -n "slot.h-align\|slot.v-align\|zstack_slot_defaults_omitted_axis_to_center" wasamoc\src\lower.rs` | `lower::tests::zstack_lowers_child_placement_to_slot_data`; `lower::tests::zstack_slot_defaults_omitted_axis_to_center` |
| CF-1 body-root placement inherits the static child surface under ZStack. | Semantic lowering | `rg -n "conditional_body_root_slot_lowers_under_zstack" wasamoc\src\lower.rs` | `lower::tests::conditional_body_root_slot_lowers_under_zstack`; runtime fixtures `conditional_zstack_reinsert_uses_declared_placement_metadata`, `static_for_under_zstack_preserves_child_carried_placement`, `reactive_for_zstack_tail_append_uses_child_carried_placement` use authored `slot.*` after the fixture migration. |
| In-repo authored `.ui` and runtime authored-source fixtures migrated from ZStack bare placement to `slot.*`; retained bare `h-align` / `v-align` are Grid `Cell`, textual-IR, or explicit reject tests. | Observable / fixture migration | `rg -n "slot\\.h-align\|slot\\.v-align\|h-align:\|v-align:" examples wasamo-runtime\tests wasamoc\src -g "*.ui" -g "*.rs"` | Owner closed in T4 by source sweep + `cargo test --workspace`. |
| GUI positive-control evidence is not implemented in T4. | GUI evidence | `git diff --name-only` lists no T5 evidence screenshots or capture scripts. | Owner task = T5; scope = launch + screenshot + positive controls; impact = no assistant visual proof yet; close condition = T5 evidence files + analysis. |
| Normative docs sync is not implemented in T4. | Docs | `git diff --name-only` lists no files under `docs/`. | Owner task = T7; scope = `docs/dsl_spec.md` / `docs/architecture.md` Moment 2 sync; impact = reference docs lag implementation; close condition = T7 sync or explicit disposition. |

T4 close gate — behavior / invariant carry scan:

| Behavior / invariant | Closed in T4? | Owner / scope / impact / close condition |
|---|---|---|
| Dotted `slot.*` is an author property key, not expression member access. | Closed | Parser canonicalizes into existing `PropertyBind.name`; parser tests listed above close this behavior. |
| Grid direct children are now admitted by the author checker and default through child-slot placement when `slot.*` is omitted. | Closed | Closed by `check_grid_direct_child_slot`, `grid_direct_child_without_slot_uses_default_placement`, and workspace tests. This is a new observable author behavior versus the old "must be wrapped in Cell" reject. |
| Retained Grid `Cell` syntax and direct Grid `slot.*` syntax lower to the same `IrSlotData::Grid` model. | Closed | Closed by `grid_direct_slot_lowers_to_same_grid_slot_data_as_cell`; retained Cell fixture hits remain intentional. |
| ZStack direct placement is only `slot.h-align` / `slot.v-align`; bare child alignment is now a named reject. | Closed | Closed by `zstack_child_bare_alignment_rejected`, fixture migration, and workspace tests. |
| Placement values are constant per instance and use the placement keyword namespace. | Closed | Closed for direct Grid and ZStack slot branches by the constant-RHS and keyword-namespace tests listed above. Bindable placement remains out of scope. |
| Grid direct `if` / `for` mutation paths are still out of scope even though static direct Grid child authoring is now accepted. | Not closed | Owner task = T7 / phase-end handoff; scope = future Grid structural mutation paths; impact = future work must re-run trap #2/#3 before admitting direct Grid control-flow; close condition = T7 candidate ledger / phase-end handoff records trigger. |
| Assistant-visible GUI proof has not run after author-surface migration. | Not closed | Owner task = T5; scope = gallery launch + screenshot + positive controls; impact = compiler/runtime tests do not prove visible placement; close condition = T5 evidence artifacts. |
| Docs still contain Moment 1 design-draft wording until Moment 2. | Not closed | Owner task = T7; scope = DSL / architecture implementation sync; impact = external-reader docs may lag the shipped `slot.*` details; close condition = T7 updates or explicit no-change record. |

T4 carry-forward ownership:

| Carry-forward | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| Produce assistant GUI positive-control evidence against the new author surface. | T5 | T4 changed examples / fixtures but did not capture rendered output. | T5 launch + screenshot + assistant analysis for ZStack and Grid positive controls. |
| Owner manual GUI smoke. | T6 | Human-visible check remains separate from automated tests. | Owner records accept or fail/fix/re-run. |
| Moment 2 docs sync for landed author surface and storage model. | T7 | No `docs/` files were edited in T4. | T7 syncs `docs/dsl_spec.md` / `docs/architecture.md` or records disposition. |
| Future Grid structural-mutation trigger. | T7 / phase-end | Direct static Grid child authoring is accepted, but direct Grid `if` / `for` remains out of scope. | T7 candidate ledger / phase-end handoff records trigger, scope, impact, and close condition. |

No owner-unknown unresolved point remains from T4. Trap #6 did trigger
for the initial deterministic workspace failure; the root cause was
stale authored ZStack bare-placement fixtures, and the disposition was
fixture migration followed by a direct rerun and full workspace green.

### T5 start gate — GUI evidence (assistant build / launch / screenshot / analysis)

Carry-over check:

- T2 / T3 / T4 all carry the same T5-owned item: produce assistant
  GUI positive-control evidence (launch + screenshot + analysis) against
  the **final** author surface (`slot.*`) and **final** storage
  (`SlotData`), for ZStack and Grid. No owner-unknown item from any prior
  task blocks T5; T6 (owner smoke) and T7 (docs sync) carry-forwards stay
  outside T5.
- T4 left the `.ui` already migrated to `slot.*`, so what T5 renders is
  the final surface, not the T2 Seam A transitional path (the T5 start
  gate condition "what the author surface is proven against" is satisfied:
  T3 merged → final `SlotData` storage).

Pre-implementation probe (recorded before choosing the approach):

- **Environment capability — confirmed.** Built `gallery-rust` release and
  ran the proven `capture-lightbox.ps1` against the current gallery.
  Result: non-blank 800×600 frames rendering the live Gallery (the
  throwaway probe frames were later consolidated into the final
  `evidence/t5-home.png`). Assistant-visible capture is dischargeable in
  this environment (visible desktop + live Compositor).
- **Surface gap — disproves the planning hypothesis.** The shipped
  gallery does **not** surface the placement positive controls:
  (a) every ZStack child uses `slot.h-align: stretch` — no `end`, no
  alignment contrast; (b) the main-screen Grid is not visible in the
  800×600 frame; (c) `capture-lightbox.ps1`'s hardcoded click coordinates
  are stale — the probe's "open" frame equals "closed" (the lightbox did
  not open). Recorded; plan.md T5 re-cut to own building a deliberate
  placement-demo surface.

Critical T5 responsibility re-cut:

- T5 is the assistant GUI-evidence task, not an author-surface or runtime
  task. It must produce screenshot + analysis + a **positive control**
  (varied alignment → varied position; omitted → default), not merely a
  launched/surviving process or a single static frame.
- Because the gallery's incidental layout does not exercise the contrast,
  T5 owns adding a **toggled placement-demo sub-screen to `gallery.ui`**
  (owner decision A, 2026-06-23): ZStack `start` / `center`-or-omitted /
  `end` overlay children at three distinct positions, and a Grid with
  distinct cell placement plus one omitted-placement child defaulting to
  `stretch`. The demo uses only the **already-accepted** T4 `slot.*` /
  `Cell` author forms — it adds no new compiler branch.
- The demo surface is throwaway verification scaffolding: it carries an
  explicit Phase-8-removal marker and is recorded as a Phase-8 cleanup
  carry-forward (T7 / phase-end ledger), consistent with the gallery's
  existing per-phase verification surfaces (P5 Footer clip, P6/7 lightbox,
  P7 reactive list).
- T5 must not reopen DD-fixed outcomes or alter T2/T3/T4 carriers; it is
  author-input + capture-tooling + analysis only.

Selected traps and non-applicable reasons:

| Trap | Classification | Reason |
|---|---|---|
| #1 semantic migration | Not applicable | T5 changes no enum / IR / schema carrier. The placement-demo `.ui` uses the shipped `slot.*` / `Cell` surface; no new variant or field. |
| #2 structural side effects | Not applicable | No runtime tree-mutation primitive changes. The demo toggle reuses the existing `if`-conditional rendering path already migrated and proven in T3. |
| #3 parallel data drift | Not applicable | T5 introduces no parallel vector / index / cache. Placement rides the unified child slot established by T3. |
| #4 untested authored branch | Not applicable (positive exercise only) | T5 adds no new reject / diagnostic / size branch to the compiler. The demo `.ui` is a **positive exercise** of T4's already-tested accept branches (`slot.h-align: end`, omitted → default); build-green is the artifact. If the demo were to require a new compiler branch, that is T4 territory and would re-classify. |
| #5 carry-forward | Applies | T5 leaves the Phase-8 removal of the demo surface, the T6 owner-smoke shared-surface dependency, and the layout-coupled capture-driver to downstream owners with re-trigger criteria. |
| #6 deterministic failure disposition | Conditional | Applies if a recurring build / launch / capture failure appears (e.g. Observation-5-class Compositor reuse, or capture flake). Any such failure gets rerun history + root-cause/disposition before close, not a re-roll to green. |
| #7 GUI positive control | Applies (central trap) | T5's deliverable is GUI-host rendering. Evidence must be launch + DPI-aware `CopyFromScreen` capture + assistant pixel analysis **with a positive control** that distinguishes the intended placement from a look-alike. Process survival is a supporting signal only. |

Review lane:

- **Full independent review** — GUI-render high-risk class. Because T5
  adds no new compiler reject branch (trap #4 non-applicable), the lanes
  do not compose; the full review covers the GUI evidence quality, the
  positive-control strength, and that the demo `.ui` compiles through the
  shipped surface.

Planned proof obligations before implementation:

| Branch / behavior / invariant hypothesis | Category | T5 proof obligation |
|---|---|---|
| The same widget authored with `slot.h-align: start` / `center`-or-omitted / `end` under ZStack lands at three distinct horizontal positions. | Observable behavior / positive control | Captured frame + assistant analysis reading three distinct x-positions; a wrong implementation collapsing to one position fails. |
| A Grid child with omitted placement falls to the per-container default (`stretch`), contrasting an explicitly placed/aligned cell. | Observable behavior / positive control | Captured frame + analysis showing the omitted child stretched vs an explicit cell at a distinct row/column/span. |
| The `slot.*` migration preserves on-screen positions (same-position re-render). | Observable invariant | Lightbox frame read against the Phase 6/7 lightbox evidence; scrim / photo at the same positions. |
| The placement-demo `.ui` compiles through the shipped T4 `slot.*` / `Cell` surface. | Build invariant / positive exercise | `cargo build --release -p gallery-rust` green (build-time check of `gallery.ui`). |
| Assistant-visible capture is non-blank and shows the intended sub-screen. | Environment / observable | Probe already confirmed non-blank; final demo frames re-confirm with the re-tuned navigation. |

Known carry-forward candidates at T5 start:

| Candidate | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| Phase-8 removal of the placement-demo surface | T7 / phase-end ledger → Phase 8 | The demo is throwaway verification scaffolding in `gallery.ui`; if not tracked it survives into the Phase 8 close cleanup. | T7 candidate ledger / phase-end handoff records it alongside the existing P5/P6/P7 verification surfaces for the Phase 8 sweep. |
| T6 owner smoke shares the placement-demo surface | T6 | T6's positive control (placement varied) reads the same sub-screen; the runnable host + observation script T5 prepares feed T6. | T6 owner runs the demo and accepts / records fail-fix-rerun. |
| Capture-driver coordinates are layout-coupled | T6 / future gallery editor | The re-tuned navigation coordinates assume the current gallery layout; a later layout change re-staleness them (as happened to the inherited script). | The re-tuned driver under `evidence/` documents its layout assumption; whoever next changes the gallery layout re-derives coordinates. |
| Moment 2 docs sync | T7 | T5 edits no `docs/`; only `gallery.ui` + evidence + log. | T7 Moment 2 sync, unrelated to T5 deliverables. |

No owner-unknown unresolved point at T5 start: every open item above is
assigned to T6, T7, or the Phase 8 close.

### T5 verification

| Command / evidence | Result | Notes |
|---|---|---|
| `cargo build --release -p gallery-rust` (placement-demo `.ui` added) | Green | Build-time `wasamoc check` of `gallery.ui` passes — the demo uses only the shipped T4 `slot.*` / `Cell` accept surface. |
| `cargo build --release --workspace` | Green | Whole workspace builds with the `gallery.ui` change; pre-existing `wasamo` no-linkable-target warning unchanged. No Rust source changed in T5. |
| Launch + DPI-aware capture (`capture-placement-demo.ps1`, non-sandbox desktop) | Non-blank render captured | `evidence/t5-home.png` (clean `false`-state home) → click → `evidence/t5-placement-demo.png` renders both positive controls; `evidence/t5-lightbox-slot-current.png` + `t5-lightbox-bare-baseline.png` are the same-position pair. |
| Assistant pixel analysis | Positive controls + same-position confirmed | ZStack: `start`/omitted-`center`/`end` at three distinct x-positions; Grid: r0c0 stretch-fill vs r0c2 centered, r1 span-3; lightbox `slot.*` vs T4-pre bare bbox identical (`x=150..648 y=60..544`) — analysed in [evidence/README.md](./evidence/README.md). |

T5 close gate — implemented-branch / behavior test map:

(T5 implements **no compiler reject / diagnostic / size / semantic
branch** — trap #4 non-applicable. The "implemented" artifacts are the
authored placement-demo surface + capture driver + evidence; each row's
proof is the rendered frame or the build, sourced from `git diff` /
captured pixels.)

| Implemented artifact / behavior | Category | Source query / diff cue | Direct proof or owner |
|---|---|---|---|
| Placement-demo surface added to `gallery.ui` (state + button + `if is_placement_demo_open` overlay). | GUI fixture | `git diff -- examples/gallery/gallery.ui` shows `is_placement_demo_open`, "Open placement demo", and the overlay `ZStack`/`VStack`. | `cargo build --release -p gallery-rust` green (build-time `wasamoc check`). |
| ZStack `slot.h-align` start / omitted→center / end render at three distinct horizontal positions. | Observable / positive control | `evidence/t5-placement-demo.png` top panel; authored at `gallery.ui` ZStack children. | Assistant pixel analysis (left/center/right boxes) — `evidence/README.md`. |
| Grid cell placement (row/column/span) + stretch-default vs centered alignment render distinctly. | Observable / positive control | `evidence/t5-placement-demo.png` lower panel; authored Grid `Cell` placement in `gallery.ui`. | Assistant pixel analysis (r0c0 stretch-fill, r0c2 centered, r1 span-3). |
| Same-position re-render: current `slot.*` lightbox lands at the pre-migration positions. | Observable / positive control | `evidence/t5-lightbox-slot-current.png` vs `evidence/t5-lightbox-bare-baseline.png` (T4-pre `3134287`, same content). | Bbox scan: photo/caption region `x=150..648 y=60..544` identical in both. |
| Re-tuned DPI-aware capture driver for the current layout. | Tooling | `git status` shows new `evidence/capture-placement-demo.ps1`. | Produced the captured frames; documents the layout assumption + non-sandbox input requirement in the script header. |
| Grid-as-placed-ZStack-child `slot.*` reject pinned by a direct test (current behavior; accept-vs-reject deferred). | Reject (pin of existing branch) | `rg -n "zstack_grid_child_slot_alignment_rejected" wasamoc\src\check.rs` | `wasamoc::check::tests::zstack_grid_child_slot_alignment_rejected` |
| No **new** compiler reject / diagnostic / size branch (the pinned Grid reject already existed in `check_grid`). | (trap #4 non-applicable) | `git diff` adds no new `check`/`lower` branch, only a test. | T4 owns the `slot.*` reject matrix; T5 adds only the review-requested pin test. |

T5 close gate — behavior / invariant carry scan:

| Behavior / invariant | Closed in T5? | Owner / scope / impact / close condition |
|---|---|---|
| `slot.h-align` placement is visibly read from the migrated model and produces alignment-keyword-driven positions (assistant baseline). | Closed (assistant portion) | Closed by `evidence/t5-placement-demo.png` + analysis. The **owner-visible** smoke remains T6. |
| Grid cell placement (row/column/span/alignment) is visibly reflected (assistant baseline). | Closed (assistant portion) | Closed by the Grid panel in the same frame + analysis. Owner-visible smoke = T6. |
| Same-position re-render proof (lightbox lands where it did pre-migration). | Closed (assistant portion) — see review response below | Closed by `evidence/t5-lightbox-slot-current.png` (current `slot.*`) vs `evidence/t5-lightbox-bare-baseline.png` (T4-pre `3134287` bare syntax, **same gallery content**): photo/caption bbox **pixel-identical** (`x=150..648 y=60..544`). The Phase 6 frame is a different gallery version and is **not** the baseline. Owner-visible confirmation remains T6. |
| Gallery placement-demo surface is throwaway and must be removed at Phase 8. | Not closed (deferred) | Owner = T7 / phase-end ledger → Phase 8; scope = `is_placement_demo_open` state + button + overlay in `gallery.ui` + `evidence/` driver; close = Phase 8 cleanup sweep removes it with the other per-phase verification surfaces. |

T5 carry-forward ownership:

| Carry-forward | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| Owner-visible GUI smoke. | T6 | The assistant baseline proves the contrast (click-opened demo) and the same-position re-render (`slot.*` vs T4-pre bare, pixel-identical); human correctness judgement remains owner-owned. | Owner runs the gallery, accepts or records fail/fix/re-run. |
| Phase-8 removal of the placement-demo surface. | T7 / phase-end → Phase 8 | `gallery.ui` demo state/button/overlay + `evidence/` capture driver are verification scaffolding. | T7 candidate ledger / phase-end handoff records it; Phase 8 sweep removes it. |
| **Finding — Grid cannot be a placed ZStack/overlay child.** `check_grid` consumes `slot.*` among a Grid's own members and rejects it ("found inside Grid"), so a Grid cannot carry `slot.h-align`/`v-align`; and a Grid centered by the ZStack default measures 0×0 (`measure_grid` reads the `Fill` width/height constraint, not the track sums) → invisible. Worked around in T5 by wrapping Grid content in a stretched `VStack`. **Current reject pinned** by `wasamoc::check::tests::zstack_grid_child_slot_alignment_rejected` (per the Codex review). | T7 / phase-end (triage) | Author-surface ↔ layout interaction gap surfaced by T5; the DD intent ("slot.* valid on a ZStack direct child") would accept it, so this is a real gap, not intended semantics. Not a Phase-7b regression. Impact: authors cannot place a Grid directly in a ZStack overlay. | T7 decides accept (needs `check_grid` + layout change) vs explicit-reject-as-spec (needs a clearer diagnostic + docs); the pin test updates with the decision. |
| **Finding — `aspect` Box inside a Grid cell aborts arrange** when the cell is measured under an unbounded intrinsic probe (`BoxAspectUnboundedBoth` during the enclosing VStack/ZStack measure), silently dropping the whole subtree. | T7 / phase-end (triage) | Pre-existing layout behavior surfaced by T5 (the documented aspect-needs-a-bounded-axis rule). Worked around by using plain fill Boxes in demo cells. | T7 records for owner triage; no Phase-7b code change. |
| **Capture-environment note (corrected, see review response)** — synthetic input (`SetCursorPos`+`mouse_event` and posted `WM_LBUTTON*`) drives the Composition app's buttons on a **real / elevated desktop** but is **dropped in a restricted (sandboxed) session**. The original close-gate claim that it "does not drive the buttons" was a sandbox artifact, not an app limitation. | (resolved in T5) | The captures are produced on a non-sandboxed desktop; `gallery.ui` defaults `is_placement_demo_open` to `false` and every frame is regenerable by clicking. M4+ interactive-GUI test automation still needs an owning-thread message-pump driver, but T5's evidence does not depend on it. |

No owner-unknown unresolved point remains from T5: every open item is
assigned to T6, T7 / phase-end, or the Phase 8 close. Trap #6 did not
trigger as a flake — the demo "invisible" outcomes were deterministic
layout aborts that were root-caused (star-track unbounded measure;
`aspect`-in-cell `BoxAspectUnboundedBoth`; Grid-as-ZStack-child 0-size)
and fixed by re-authoring, not re-rolled to green.

### T5 review response (Codex full independent review, 2026-06-23)

Codex's full independent review (GUI-render lane) judged the core
positive controls sufficient but found three gaps; all three are resolved
on this branch (follow-up commit):

1. **Same-position proof closed in T5, not punted to T6.** Re-ran the
   capture on a non-sandboxed desktop where synthetic input works and
   opened the current-branch `slot.*` lightbox. This also corrected the
   earlier "synthetic input does not drive the buttons" finding — it was a
   sandbox artifact. **(Second review round, 2026-06-23:** Codex measured
   a 20px Y-offset vs the Phase 6 frame. Root cause: the Phase 6 evidence
   is a *different gallery version*, so it conflated gallery-`.ui`
   evolution with the migration. Re-did the proof against the **correct**
   baseline — the T4-pre commit `3134287` (last bare-syntax commit, same
   gallery content), built in a throwaway worktree because the current
   `wasamoc` rejects bare syntax. `evidence/t5-lightbox-bare-baseline.png`
   vs `evidence/t5-lightbox-slot-current.png` are **pixel-position
   identical** (bbox `x=150..648 y=60..544`), proving the migration is
   position-preserving. The Phase 6 frame is no longer cited as the
   baseline.)
2. **Grid-as-placed-ZStack-child behavior pinned.** Not fixed in T5
   (accepting it needs both a `check_grid` change and a layout change —
   out of T5's GUI-evidence scope), so the **current reject is pinned by a
   direct test** `wasamoc::check::tests::zstack_grid_child_slot_alignment_rejected`,
   and the accept-vs-reject design decision stays a T7 / phase-end
   carry-forward (the test comment names the deferral). This is the
   carry-forward path Codex allowed; T5 does **not** claim the constraint
   is "closed".
3. **`t5-gallery-home-no-demo.png` ambiguity removed.** Reverted
   `is_placement_demo_open` to its committed default `false` so the home
   frame is a genuine, regenerable false-state. The old probe image is
   replaced by `evidence/t5-home.png` (clean home, shows the "Open
   placement demo" button); the demo frame is reached by one click.

Updated trap-#4 note: T5 still adds **no new** compiler reject / diagnostic
/ size branch (the Grid-as-ZStack-child reject already existed in
`check_grid`); the added test **pins existing behavior** flagged by the
review, so the branch/test-focused check inside the full review is
satisfied by `zstack_grid_child_slot_alignment_rejected`.

Verification after the follow-up:

| Command / evidence | Result | Notes |
|---|---|---|
| `cargo test -p wasamoc --lib zstack_grid_child_slot_alignment_rejected` | Green | Pins the current Grid-as-ZStack-child `slot.*` reject. |
| `cargo build --release -p gallery-rust` (demo default `false`) | Green | Build-time `wasamoc check` of `gallery.ui` passes. |
| Non-sandbox capture (`-OpenDemoAt` / `-OpenLightboxAt`) | Frames captured | `t5-home.png`, `t5-placement-demo.png`, `t5-lightbox-slot-current.png` regenerated from the committed `false` default by clicking. |

### T6 start gate — owner-manual GUI smoke

Carry-over check:

- T5 (and T2/T3/T4 before it) carry exactly one T6-owned item: the
  **owner-visible GUI smoke** for ADR evidence item (4), a separate gate
  from T5's assistant baseline. T5 closed the **assistant** portion
  (launch + screenshot + analysis + positive controls; same-position
  proof against the T4-pre `3134287` bare baseline, pixel-identical) and
  explicitly left the human-correctness judgement to T6.
- T5 carry-forward also assigns to T6 (jointly with future gallery
  editors) the **layout-coupled capture-driver coordinates** note; and to
  T7 / phase-end the **Phase-8 removal** of the placement-demo surface and
  the two **layout findings** (Grid-as-placed-ZStack-child reject;
  `aspect`-in-cell arrange abort). None of these block T6.
- Start-gate precondition satisfied: T5 is merged into `feat/m3-phase-7b`
  (`ce5cc4d`); the T6 branch `feat/m3-phase-7b-t6` is at the same commit;
  working tree clean. No owner-unknown item blocks T6.

Critical T6 responsibility re-cut (plan.md is a hypothesis):

- T6's substance is the **owner-performed** GUI smoke + explicit
  acceptance. The assistant **cannot** discharge it; the assistant's T6
  deliverable is bounded to (a) confirming the start-gate precondition,
  (b) preparing the runnable host, and (c) authoring the detailed owner
  observation script. This split was not auditable in the prior plan.md
  T6 checklist, so plan.md was revised to add a checkable **Assistant
  prep** item and keep the smoke / acceptance / retro items `[ ]` until
  the owner reports.
- The prior plan.md wording ("ZStack `slot.*` / Grid placement render at
  the **same positions as the old surface**") was corrected: the bare
  ZStack surface is rejected on this branch (T4), so **no live old-vs-new
  comparison is possible**, and the same-position invariant was already
  closed by T5 (assistant portion). The owner's positive control is
  therefore **placement varied** on the placement-demo sub-screen (varied
  alignment → varied position; omitted → per-container default), with the
  recorded T5 baseline pair as same-position corroboration.
- The **fix container** is named explicitly (`feat/m3-phase-7b-t6`) per
  [retrospectives.md §step-end item 11](../../../procedures/retrospectives.md):
  if the owner smoke fails, the visible-correctness fix lands additively
  on the T6 branch and re-enters the appropriate production review lane
  (M3-Phase 4 T6 smoke-fail → fix → re-smoke precedent).
- The assistant must not fabricate the owner result or check the
  owner-performed items; the T6 step-end retrospective is recorded **after**
  the owner smoke result is known.

Selected traps and non-applicable reasons:

| Trap | Classification | Reason |
|---|---|---|
| #1 semantic migration | Not applicable to assistant prep | T6 assistant prep changes no enum / IR / schema carrier; it adds an observation-script doc + plan/log process edits only. If a smoke-fail fix later touches a carrier, it re-enters trap #1 on the fix. |
| #2 structural side effects | Not applicable to assistant prep | No tree / state mutation production code lands in T6 assistant prep. A smoke-fail fix touching a runtime mutation path would re-classify (as M3-Phase 4 T6's fix bundle did). |
| #3 parallel data drift | Not applicable | T6 introduces no parallel vector / index / cache. Placement already rides the unified child slot (T3). |
| #4 untested authored branch | Not applicable to assistant prep | T6 assistant prep adds no compiler reject / diagnostic / size branch. A smoke-fail fix that added one would ship a direct firing test and re-classify. |
| #5 carry-forward | Applies | T6 carries the owner-acceptance gate, the inherited Phase-8 demo removal + layout findings (T7 / phase-end), the layout-coupled capture coordinates, and the conditional smoke-fail fix container. Each is recorded with owner / scope / close condition. |
| #6 deterministic failure disposition | Conditional | Applies only if the host build / launch shows a recurring failure (e.g. an Observation-5-class Compositor reuse). Any such failure gets a root cause, not a re-roll to green. None occurred: build green, launch survived. |
| #7 GUI positive control | Applies (central, owner-side) | T6's deliverable **is** the GUI smoke. The assistant prep supplies the runnable host + an observation script that names the positive control (placement varied) and per-control pass/fail; the **owner** discharges the rendered-evidence judgement. The assistant launch-survival check is a supporting "no early crash" signal only — it does **not** substitute for the owner smoke or the T5 screenshot baseline. |

Review lane:

- **No special production-code review for the assistant prep** — it lands
  process docs (observation script + plan/log) and a host build only, no
  production Rust. The external review checks the observation-script
  completeness and the responsibility-split auditability. **Conditional:**
  any owner-smoke-fail fix that lands production code re-enters its
  implementation-gates §4 lane (full independent review for runtime /
  GUI-render changes; branch/test-focused for reject-branch additions).

Planned proof obligations before completion (assistant prep hypotheses):

| Branch / behavior / invariant hypothesis | Category | T6 proof obligation |
|---|---|---|
| The host on the post-T5-merge branch builds and launches without early crash. | Observable / host-runnable | `cargo build --release -p gallery-rust` green; `Start-Process` survival (supporting signal). |
| The placement-demo sub-screen and lightbox are reachable by the documented navigation (Open placement demo / Open lightbox / Close demo). | Observable / owner-followable | Observation script names the exact buttons; T5 already captured the rendered sub-screens (`t5-placement-demo.png`, `t5-lightbox-slot-current.png`). |
| The observation script names a positive control the owner can read (varied alignment → varied position; omitted → default; stretch-vs-centered Grid contrast). | Owner-evidence quality | Script has a per-control section with explicit PASS / FAIL. |
| No live old-surface comparison is required of the owner. | Scope correctness | Plan re-cut + script "Same-position note" point to the T5 baseline pair; the same-position invariant is T5-closed. |

Known carry-forward candidates at T6 start:

| Candidate | Owner | Scope / impact | Close condition |
|---|---|---|---|
| Owner-acceptance gate | T6 (owner) | The smoke + explicit acceptance cannot be discharged by the assistant; until recorded, T6 cannot close. | Owner runs the observation script and accepts, or records fail → fix → re-run. |
| Smoke-fail fix container | T6 branch (conditional) | Any visible-correctness fix lands additively on `feat/m3-phase-7b-t6` and re-enters the appropriate review lane. | Re-smoke green + owner acceptance. |
| Phase-8 removal of placement-demo surface + capture driver | T7 / phase-end → Phase 8 | Inherited from T5; throwaway verification scaffolding in `gallery.ui` + `evidence/`. | T7 candidate ledger / phase-end handoff records it; Phase 8 sweep removes. |
| Layout findings (Grid-as-placed-ZStack-child reject; `aspect`-in-cell arrange abort) | T7 / phase-end (triage) | Inherited from T5; not Phase-7b regressions. | T7 triages accept-vs-spec-note; the pin test updates with the decision. |
| Capture-driver coordinates are layout-coupled | T6 / future gallery editor | Inherited from T5; navigation coords assume the current gallery layout. | Whoever next changes the gallery layout re-derives coordinates. |
| Moment 2 docs sync | T7 | T6 edits no `docs/`. | T7 Moment 2 sync, unrelated to T6 deliverables. |

No owner-unknown unresolved point at T6 start: every open item above is
assigned to T6 (owner), the T6 fix container, T7 / phase-end, or the
Phase 8 close.

### T6 end gate — assistant-prep portion

The owner-smoke close gate (owner acceptance) is recorded **after** the
owner runs the smoke; this is the auditable artifact for the
assistant-completable portion only.

T6 assistant-prep verification:

| Command / evidence | Result | Notes |
|---|---|---|
| `git rev-parse --short HEAD` / `feat/m3-phase-7b` | `ce5cc4d` / `ce5cc4d` | T6 branch at the T5-merge commit; start-gate precondition "T5 merged" satisfied; working tree clean before T6 edits. |
| `cargo build --release -p gallery-rust` | Green | Runnable host built on the T6 branch (`Finished release` in 3.48s). Build-time `wasamoc check` of `gallery.ui` passes. |
| `Start-Process target/release/gallery-rust.exe` survival | Alive after 3s, title `Gallery` | Supporting "no early crash" signal only — **not** GUI evidence (T5 owns screenshot evidence). |

T6 close gate — implemented-branch / behavior test map:

(T6 implements **no compiler reject / diagnostic / size / semantic
branch** — trap #4 non-applicable. The assistant-prep "implemented"
artifacts are the observation script + host-runnable confirmation + the
plan responsibility re-cut; each row's proof is the build / launch or the
`git diff`. The owner-performed evidence rows are owner=T6.)

| Implemented artifact / behavior | Category | Source query / diff cue | Direct proof or owner |
|---|---|---|---|
| Owner observation script authored (launch + per-control navigation + PASS/FAIL + same-position note). | GUI smoke prep | `git status` shows new `evidence/t6-owner-smoke-script.md`. | The file's per-control PASS/FAIL sections; reviewed for completeness in the external review. |
| Runnable host built + launches without early crash on the T6 branch. | Host-runnable | `cargo build --release -p gallery-rust` green; `Start-Process` survival. | Build + launch evidence above (supporting signal). |
| plan.md T6 re-cut: assistant-prep vs owner-smoke split made auditable; same-position wording corrected; fix container named. | Process / ownership | `git diff -- process/milestone-3/phase-7b/implementation/plan.md` shows the `[x]` Assistant-prep item, the responsibility-split paragraph, and `feat/m3-phase-7b-t6` as the fix container. | External review of the plan diff. |
| Owner-visible smoke (ZStack three-position / Grid stretch-vs-centered / lightbox / close) is **not** discharged by the assistant. | GUI positive control (owner) | `evidence/t6-owner-smoke-script.md` Observations 1–4. | Owner task = T6; close = owner runs the script and accepts (or fail → fix → re-run). |
| No **new** compiler reject / diagnostic / size branch. | (trap #4 non-applicable) | `git diff` adds no `check` / `lower` branch (no Rust source changed in T6 prep). | T4 owns the `slot.*` reject matrix; T5 owns the screenshot baseline. |

T6 close gate — behavior / invariant carry scan:

| Behavior / invariant | Closed in T6 prep? | Owner / scope / impact / close condition |
|---|---|---|
| Runnable host + detailed owner observation script are prepared for the owner smoke. | Closed (assistant prep) | Closed by the build/launch evidence + `evidence/t6-owner-smoke-script.md`. |
| Owner-visible placement smoke (positive control = placement varied) is performed and accepted. | Not closed (owner-performed) | Owner = T6; scope = gallery placement-demo + lightbox; impact = ADR evidence item (4) owner half open until accepted; close = owner accept or fail → fix (on `feat/m3-phase-7b-t6`) → re-run. |
| Phase-8 removal of the placement-demo surface + capture driver. | Not closed (inherited, deferred) | Owner = T7 / phase-end ledger → Phase 8; close = Phase 8 cleanup sweep. |
| Layout findings (Grid-as-ZStack-child reject; aspect-in-cell abort). | Not closed (inherited, deferred) | Owner = T7 / phase-end triage; close = accept-vs-spec-note decision. |

T6 carry-forward ownership:

| Carry-forward | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| Owner-visible GUI smoke + acceptance. | T6 (owner) | Human correctness judgement is owner-owned; assistant prep is complete. | Owner runs the observation script, accepts or records fail/fix/re-run. |
| Smoke-fail visible-correctness fix. | T6 branch (conditional) | Lands additively on `feat/m3-phase-7b-t6`; re-enters its review lane. | Re-smoke green + owner acceptance. |
| Phase-8 removal + layout findings + capture-coord staleness. | T7 / phase-end → Phase 8 | Inherited from T5; recorded in the candidate ledger. | T7 ledger / phase-end handoff; Phase 8 sweep. |

No owner-unknown unresolved point remains from the T6 assistant prep:
every open item is assigned to T6 (owner), the T6 fix container, or
T7 / phase-end. Deterministic-failure trap #6 did not trigger — the host
build and launch were green on the first run.

### T6 close gate — owner-smoke result (2026-06-23)

The owner ran the GUI smoke on a visible desktop with a real mouse per
[evidence/t6-owner-smoke-script.md](./evidence/t6-owner-smoke-script.md)
and **accepted all observations as pass** (no fix iteration needed).
Owner-captured frames committed under [evidence/](./evidence/):

| Frame | Observation | Owner result |
|---|---|---|
| `t6-placement-demo.png` | Obs 1 ZStack `slot.h-align` start/omitted/end → left/center/right (three distinct x); Obs 2 Grid r0c0 stretch-fill vs r0c2 centered + r1 span-3 (distinct row/column/span + alignment contrast) | pass |
| `t6-lightbox.png` | Obs 3 lightbox card centered (same-position corroboration vs `t5-lightbox-slot-current.png`); WrapPanel / ScrollView around placed children correct | pass |
| `t6-home.png` | false-state home (both buttons, no overlay) — context | pass |
| (window close) | Obs 4 Alt+F4 / × crash-free | pass |

Assistant corroboration of the owner frames (sanity check before
committing them as evidence — not a substitute for the owner judgement):
both positive-control frames are non-blank and show the expected spatial
contrasts (ZStack three-position spread; Grid stretch-vs-centered). The
fixed-track Grid not resizing with the window is **expected** (T5
deliberate authoring — star tracks / `aspect` cells abort arrange in this
nested overlay; T7 / phase-end triage carry-forward), and Obs 2's positive
control is the in-frame alignment contrast, not resize responsiveness.

This closes the **owner half** of ADR evidence item (4); the assistant
half was closed in T5. No deterministic-failure (trap #6) and no
smoke-fail fix iteration occurred. The T6 step-end retrospective is at
[../retrospectives/t6.md](../retrospectives/t6.md).

### T6b start gate — Grid-as-ZStack-child `slot.*` checker fix

T6b is a **mid-phase inserted task** (owner decision 2026-06-24): correct
the checker so a `Grid` placed as a direct `ZStack` child may carry
`slot.h-align` / `slot.v-align`, per the DD-M3-P7b-001 intent that
`slot.*` is valid on a ZStack direct child.

Carry-over check from prior tasks:

- The relevant carry-over is the T5/T6 **layout finding** (Grid-as-ZStack-
  child reject; aspect-in-cell abort), recorded with close condition
  "accept-vs-spec-note decision; owner = T7 / phase-end triage"
  (`log.md` T5 §close gate; T6 carry-forward ownership). The owner chose
  **accept-and-fix the checker half** and to insert T6b for it, leaving the
  layout (sizing) half as a recorded carry-forward.
- No owner-unknown item from T1–T6 blocks T6b. The Phase-8 demo-removal and
  candidate-ledger carry-forwards stay T7/phase-end-owned and are untouched
  by T6b.

Critical T6b responsibility re-cut (two-problem boundary):

- **Problem A (T6b owns):** `check_grid`'s member loop
  ([check.rs:1240](../../../../wasamoc/src/check.rs#L1240)) calls
  `check_slot_property_outside_parent(key, Some("Grid"), None, …)` on a
  `slot.*` PropertyBind among the Grid's **own** members, rejecting it as
  "inside `Grid`". But the same member is **already** validated correctly by
  the generic walk `check_members_inner` via the `parent_widget == Some("ZStack")`
  branch ([check.rs:1982](../../../../wasamoc/src/check.rs#L1982)). The Grid
  pass is a duplicate, wrong evaluation. Fix: `check_grid` **skips** `slot.*`
  PropertyBinds (parent-owned, validated by the parent), keeping the
  unknown-non-slot Grid-attribute reject.
- **Problem B (NOT T6b):** the layout 0×0 collapse is the Phase 5
  (`measure_grid` Fill→0) + Phase 6 (ZStack measure/anchor) contract,
  unrelated to the slot redesign (git-verified: the 7b layout commits did
  not change `measure_grid` / `axis_is_stretchy` bodies). It is the symptom
  of the long-deferred `width`/`height` author surface, recorded in
  [author-controllable-sizing notes](../../../../docs/notes/author-controllable-sizing.md);
  Vision DR scheduled for Phase 8 framing, hard backstop pre-1.0 / M6
  ABI-freeze prep.

Selected traps and non-applicable reasons:

| Trap | Classification | Reason |
|---|---|---|
| #1 semantic migration | Not applicable | No enum / IR / schema variant or field change; T6b edits checker control flow only. No traversal call-site set changes shape. |
| #2 structural side effects | Not applicable | No tree-structure / state mutation; the checker emits diagnostics, it does not mutate the widget tree or derived runtime state. |
| #3 parallel data drift | Not applicable | No parallel vector / derived index / cache is touched. |
| #4 untested authored branch | **Applies** | T6b changes a reject branch to an accept and relies on a different (parent) branch for validation; it must ship direct tests that fire the new accept, the preserved value-validation reject, and the preserved non-admitting-parent reject. |
| #5 carry-forward | **Applies** | Problem B (layout 0×0 / `width`-`height` sizing) is a recorded carry-forward with a re-trigger criterion; T6b must point at the docs/notes home and the Vision DR timing, not silently leave it. |
| #6 deterministic failure disposition | Conditional | Only if an unexpected recurring test failure appears; record rerun history + root cause before close. |
| #7 GUI positive control | Not applicable | T6b has no GUI-render deliverable. The visible Grid-in-ZStack render is the problem-B-limited case, explicitly not fixed here; the positive control is at the checker (accept / value / non-admitting) level. The T5/T6 GUI evidence stands. |

Review lane:

- **Branch/test-focused review** (not full). T6b is a diagnostic / reject /
  accept branch change in `wasamoc check` — not a schema/IR migration,
  runtime structural change, or GUI-render evidence. The review must check
  the trap-#4 branch/test map (accept + value-validation reject +
  non-admitting reject). Owner runs an external-agent review after commit.

Planned proof obligations before implementation (hypotheses):

| Branch / behavior / invariant hypothesis | Category | T6b proof obligation |
|---|---|---|
| `ZStack { Grid { slot.h-align: end … } }` is accepted (no "inside `Grid`" / parent-owned diagnostic). | Semantic (accept) | Flip `zstack_grid_child_slot_alignment_rejected` → accepted: assert the parsed component produces no placement-misplacement error. |
| The `slot.h-align` **value** on a Grid-in-ZStack is still validated by the parent ZStack. | Reject (value) | Direct test: `slot.h-align: <bogus>` on a Grid-in-ZStack rejected as a bad alignment (`check_zstack_child_align` fires). |
| `slot.*` on a Grid under a **non-admitting** parent (VStack / component level) is still rejected, exactly once. | Reject (position) + invariant | Direct test: `VStack { Grid { slot.h-align: … } }` rejected as parent-owned/"inside `Grid`"; no duplicate diagnostic. |
| `check_slot_property_outside_parent` is not left dead by the edit. | Invariant (no dead_code) | It remains called from `check_members_inner` (component-level + non-admitting child); workspace builds with no new `dead_code` warning. |
| No existing `.ui` / fixture / spec depends on the old reject. | Observable invariant | `rg`/`git grep` for other tests asserting "inside `Grid`" on a Grid-own `slot.*`; workspace tests stay green. |

Known carry-forward candidates at T6b start:

| Candidate | Owner task | Scope / impact | Close condition |
|---|---|---|---|
| Problem B — layout 0×0 / author-controllable `width`-`height` sizing | T7 ledger → Phase 8 Vision DR | A Fill-default container nested on a Shrink ancestor axis collapses; `slot.*` on a Grid-in-ZStack now compiles but only renders when the ZStack has a definite size. | docs/notes home landed; T7 candidate ledger records responsibility = Vision DR, trigger, ABI-impact-pending; Vision DR at Phase 8 framing assigns the milestone home; hard backstop pre-1.0 / M6 ABI-freeze prep. |
| Phase-8 demo-removal + candidate ledger (inherited) | T7 / phase-end → Phase 8 | Untouched by T6b; stays as T5/T6 recorded. | T7 ledger / phase-end handoff; Phase 8 sweep. |

### T6b verification

| Command / evidence | Result | Notes |
|---|---|---|
| `cargo test -p wasamoc --lib grid_child_slot` | Green | 3 passed: `zstack_grid_child_slot_alignment_accepted`, `zstack_grid_child_slot_alignment_value_still_validated`, `nonadmitting_parent_grid_child_slot_still_rejected`. |
| `cargo test --workspace` | Green | wasamoc lib 385 passed (was 382; +3 T6b tests, old `..._rejected` renamed to `..._accepted` in place), wasamo-runtime 423, wasamo-ir 24, all integration / doctests pass. No new failures. |
| `cargo fmt --all -- --check` | Green | Exit 0 on post-edit state. |
| `cargo build -p wasamoc` warning scan | Green | No new `dead_code` / unused warning; `check_slot_property_outside_parent` stays used by `check_members_inner`. |
| `git grep "inside \`Grid\`" wasamoc/src/check.rs` | Only new comments + the non-admitting test | No other test asserted a Grid-own `slot.*` reject, so nothing else broke. |

T6b close gate — implemented-branch test map (trap #4):

Enumeration source: `git diff` of `wasamoc/src/check.rs` (one production
hunk in `check_grid`, three test hunks) + `git grep`.

| Implemented branch / behavior | Category | Source query / diff cue | Direct test or owner |
|---|---|---|---|
| `check_grid` no longer consumes `slot.*` among the Grid's own members; the `slot_key(name).is_some()` arm is now a no-op (parent-owned, delegated to the generic walk). | Semantic (reject→accept) | `git diff wasamoc/src/check.rs` shows the `Member::PropertyBind` arm in `check_grid` changed from `check_slot_property_outside_parent(...)` to the skip comment; `rg -n "slot_key\\(name\\)\\.is_some\\(\\)" wasamoc/src/check.rs` | `wasamoc::check::tests::zstack_grid_child_slot_alignment_accepted` |
| Parent ZStack still validates the `slot.h-align` **value** of a Grid-in-ZStack child (not a blanket accept). | Reject (value) — positive control | `rg -n "fn zstack_grid_child_slot_alignment_value_still_validated" wasamoc/src/check.rs`; needle: error contains `ZStack child \`slot.h-align\` must be one of` (fired by `check_zstack_child_align` via the `parent_widget == ZStack` branch) | `wasamoc::check::tests::zstack_grid_child_slot_alignment_value_still_validated` |
| `slot.*` on a Grid under a non-admitting parent (VStack) is still rejected, and **exactly once** (no Grid-pass duplicate). | Reject (position) + invariant — positive control | `rg -n "fn nonadmitting_parent_grid_child_slot_still_rejected" wasamoc/src/check.rs`; needle: count of errors containing `parent-owned child placement data` && `inside \`Grid\`` equals 1 | `wasamoc::check::tests::nonadmitting_parent_grid_child_slot_still_rejected` |
| Unknown **non-slot** Grid attribute (`Grid { foo: 0 }`) is still rejected by `check_grid`. | Reject (preserved) | `git diff` shows the `else` arm (`unknown Grid attribute …`) unchanged | Pre-existing `wasamoc::check::tests` Grid-attribute coverage (unchanged; not re-authored in T6b) |

T6b close gate — behavior / invariant carry scan:

| Behavior / invariant | Closed in T6b? | Owner / scope / impact / close condition |
|---|---|---|
| A `Grid` that is a direct `ZStack` child may carry `slot.h-align` / `slot.v-align` (checker accepts). | **Closed** | Verified by `zstack_grid_child_slot_alignment_accepted`; value-validation and non-admitting reject preserved. No `.ui` / spec change needed (DD already states `slot.*` valid on a ZStack direct child). |
| Layout: a Fill-default Grid nested on a Shrink ancestor axis collapses to 0×0 (the `slot.*` now compiles but may not render). | **Not closed (intentionally deferred)** | Owner = T7 ledger → Phase 8 Vision DR; scope = author-controllable `width`/`height` sizing; impact = Grid-in-ZStack renders only when the ZStack has a definite size; close = Vision DR assigns milestone home; trigger + pre-1.0 backstop recorded in [author-controllable-sizing notes](../../../../docs/notes/author-controllable-sizing.md). |
| Default-center / start / end alignment is a visual no-op on a Fill container (only content-sized children anchor). | **Not closed (documented limitation)** | Owner = same as above (it is a facet of the `width`/`height` gap); impact = authors cannot anchor a *smaller* Grid in a ZStack; close = the sizing Vision DR. Recorded in the docs/notes home, not a new T6b artifact. |

T6b carry-forward ownership:

- No owner-unknown unresolved point remains from T6b. Problem B and its
  facets are owned by **T7 candidate ledger → Phase 8 Vision DR**, with the
  docs/notes home landed and the re-trigger / hard backstop recorded.
- Deterministic-failure trap #6 did not trigger — the three new tests and
  the workspace suite passed on the first run after the edit.
