# M3-Phase 8 — Implementation log

Append-only mixed log: Decisions log (mid-implementation judgments) +
CI / verification log (evidence pointers, run ids). Per-task
implementation-gate selections (start) and close artifacts are recorded
here per [preamble.md §Implementation gates](./preamble.md).

<!-- Entries land per task as execution proceeds (T1 onward). -->

## T1 start gate — responsibility cut and trap selection (2026-07-03)

T1 was re-read as a spike whose responsibility is source-grounded
uncertainty reduction and task-boundary repair. It must not begin the
production `ToggleButton` implementation, decide the owner-facing A1
placeholder agreement, or absorb downstream task choices silently. Its
close artifacts must separate (1) required plan revisions before work
continues, (2) downstream choices scoped enough to leave with T2-T8, and
(3) owner-facing judgments still owned by the staged checkpoints.

Selected traps:

| Trap | Applies? | Reason / close artifact |
|---|---:|---|
| #1 semantic migration | Yes | T1 intentionally pre-audits the `ToggleButton` kind/`checked` surface across compiler, IR, loader, and runtime dispatch sites. Close with the throwaway-build result and a source-grounded site list; T3/T4 own the authoritative call-site audit. |
| #2 missed side effects | Yes | Gallery/host recon can hide layout/capture/CI side effects if T5/T6 boundaries are wrong. Close with the retained/superseded capture-script list, host-port deltas, and downstream ownership. |
| #3 parallel/derived data drift | No | T1 changes no production data structure and introduces no parallel vector/map/index. If recon finds one, assign it to the implementation task that mutates it. |
| #4 untested authored branch | No | T1 writes no production diagnostic/reject/size branch. T3/T4 own firing tests for new rejects. |
| #5 carry-forward | Yes | T1 may discover invariants later tasks must preserve. Close with carry-forward entries or an explicit "none found", each with a re-trigger criterion. |
| #6 deterministic failure | Conditional | Any build/probe failure during the throwaway verification or recon gets one rerun plus a disposition; no "green on retry" without cause. |
| #7 GUI positive control | No | T1 has no GUI-render deliverable. T2 and T7 own screenshot evidence. |

Review lane: normal task-end review after the retrospective. T1 lands
documentation and throwaway-spike results only; no schema/IR/runtime
production migration, diagnostic branch, or GUI-render evidence is merged
in this task.

## T1 recon results — source touch points and downstream ownership (2026-07-03)

T1 read the landing files end-to-end and found the following implementation
shape. The important correction to the initial plan hypothesis is that
`ToggleButton` is **not** compile-error-forcing at the IR carrier: the IR
node kind is `IrNode.widget_type: String`, and `wasamoc check` currently
treats an unknown widget as a warning.

### Per-file touch points

| File | Source facts | Downstream owner |
|---|---|---|
| `wasamoc/src/lexer.rs` | No new token is required. `ToggleButton` and `checked` lex as ordinary identifiers; kebab-case and bool literals already exist. | T3: no lexer edit expected unless tests need fixtures only. |
| `wasamoc/src/parser.rs` | Widget declarations are generic `Ident { ... }`; property binds and `clicked => { ... }` are generic. Grid track-list routing is the only widget-name parser special case. | T3: parser admission should remain unchanged; positive parse tests are optional but not the main gate. |
| `wasamoc/src/ast.rs` | `Member::WidgetDecl { type_name: String, ... }` and `PropertyBind { name, value }` already carry the surface. | T3: no AST schema change expected. |
| `wasamoc/src/check.rs` | Known widget names live in `KNOWN_WIDGET_TYPES`. Typed widget props live in `widget_prop_type`; today `Button.text` and `Button.enabled` are typed, while `Button.style` is an identifier-valued keyword path. Unknown widget types are warnings, not errors. Unknown attributes are mostly enforced by per-widget special cases plus the typed-prop table; there is no complete generic "all unknown attrs reject" table for Button-family widgets today. | T3: add `ToggleButton` to `KNOWN_WIDGET_TYPES`; mirror Button's `text`/`enabled` typed rows; add `checked: bool`; add explicit admission/rejection tests so `checked` on Button/Text/others rejects and `ToggleButton` unknown attrs follow the intended existing path. |
| `wasamoc/src/lower.rs` | Lowering is generic over `widget_type` and props/bindings/handlers. Bool identifiers already lower to `HandlerExpr::BoolPropRead`; handler block assignment already supports the α exclusion shape. Parent-specific slot stripping is keyed only by parent kind. | T3: no new lowering schema is expected, but fixtures must pin `ToggleButton` kind, `checked` static prop/binding, inherited `clicked`, and the α block-assignment shape. |
| `wasamoc/src/emit.rs` | Emission is generic `node <widget_type>`, `prop`, `bind`, `on`. `KindPayload` emission is Grid-only. | T3: no new emit grammar expected; add emit/roundtrip fixture for `ToggleButton` with literal and binding `checked`. |
| `wasamo-ir/src/lib.rs` | `IrNode.widget_type` is `String`; the only current per-kind enum is `KindPayload::Grid`. `IrBinding.prop_name` is string and already supports bool expression values. | T3/T4: no IR schema variant needed. The trap-#1 audit must include wildcard/string dispatch sites because Rust will not enumerate misses. |
| `wasamo-runtime/src/ir_loader.rs` | `validate()` has phase-specific defense gates but no general widget-property admissibility mirror for Button-family props. `construct_widget` dispatches by `node.widget_type.as_str()`. `resolve_prop_key` maps `(widget_type, prop_name)` to a `u32` property key and `IrType`, selecting `register_bool_binding` for bools. Unknown widget reaches `IrLoadError::UnknownWidget`. | T4: add a `ToggleButton` construct arm; add `checked` loader re-reject in validation; map `ToggleButton.text/style/enabled/checked` in `resolve_prop_key`; reuse existing bool binding path for `checked`. |
| `wasamo-runtime/src/widget.rs` | `WidgetData::Button(Box<ButtonData>)` owns label/style/enabled/click visuals. Button visual colors flow through `effective_button_color` / `button_state_color`; disabled overrides style/state. Layout treats Button as a fixed-size leaf via `WidgetData::Button(_) => LayoutNode::rectangle(...)`. | T4: prefer Button-family sharing without erasing runtime kind: a `ToggleButton` variant or equivalent role-bearing node sharing `ButtonData`/helpers, plus a `checked` field and color computation that composes with `style` and `enabled`. Disabled should remain highest priority; checked should be visible in normal/enabled states. |
| `wasamo-runtime/src/layout.rs` | No Button-specific `WidgetKind`; Button is already a fixed leaf rectangle produced by `widget.rs`. | T4: no new layout primitive; if runtime introduces a distinct `WidgetData::ToggleButton`, `build_layout_tree` should route it through the same rectangle leaf as Button. |
| `examples/gallery/gallery.ui` | Current app is still a verification-surface stack: Grid footer-clip demo, static ten-photo WrapPanel, lightbox overlay, placement-demo overlay, scroll and collection mutation buttons, dynamic ScrollView/WrapPanel. | T2/T5: T2 builds the wireframe skeleton and G(1) table; T5 sweeps verification-only surfaces and completes the integrated gallery. |
| `examples/counter-c/*` | C template builds `.ui -> .uic -> *_uic.h` with counter-specific variable names and array guard (`COUNTER_UI`, `COUNTER_UIC`, `COUNTER_UIC_H`, `COUNTER_UIC`). It expects `target/release/wasamoc.exe` and `target/release/wasamo.dll.lib`. | T6: port names to gallery (`GALLERY_UI`, `GALLERY_UIC`, etc.), component artifact paths, executable name, README text, and keep the same build-order assumptions. |
| `examples/counter-zig/*` | Zig template invokes `wasamoc build`, embeds anonymous import `counter_uic`, and defaults `wasamoc` to `../../target/release/wasamoc.exe`. | T6: port option names/default `.ui` path/import/executable name to gallery; CI can mirror counter-zig and rely on the release workspace build, or pass `-Dwasamoc` explicitly if T6 wants the dependency more visible. |
| `examples/gallery-rust/*` | Rust gallery host already exists and compiles `examples/gallery/gallery.ui` in `build.rs` through the in-process `wasamoc` pipeline; no separate `wasamoc.exe` ordering edge. | T5/T6: Rust remains representative host; C/Zig hosts should match the same `.ui` path and load compiled IR through memory embedding. |
| `.github/workflows/ci.yml` | CI builds release+debug workspace and tests, then builds counter C/Rust/Zig examples and runs `wasamoc check counter.ui`. No gallery C/Zig steps yet. | T6: add per-example `gallery-c` and `gallery-zig` steps; optional explicit `wasamoc check examples/gallery/gallery.ui` is appropriate. |

### Throwaway verification / wrong-kind probe

Because the IR kind carrier is string-based, T1 did not introduce a
throwaway enum variant. Instead it ran a deliberate wrong-kind probe by
temporarily adding `implementation/t1-wrong-kind-probe.ui` with a
`ThrowawayToggleButton` node and running:

```
cargo run -p wasamoc -- check process\milestone-3\phase-8\implementation\t1-wrong-kind-probe.ui
```

Result: `wasamoc check` printed
`warning: unknown widget type 'ThrowawayToggleButton'; known types: ...`
and exited 0. The probe file was deleted before T1 close. No production
source edit remains.

Implication: T3 cannot rely on build failure to expose missed widget-kind
admission. T3 positive fixtures must prove `ToggleButton` is admitted as a
known widget (no unknown-widget warning), and T3/T4 trap-#1 audit must be
`rg`-based over string dispatch tables plus tests.

### Internal shape recommendations

- **Button-family sharing:** implement `ToggleButton` as a distinct runtime
  node/kind while sharing Button construction/update helpers. A shared
  `ButtonData` helper path is acceptable; erasing the runtime kind entirely
  into `WidgetData::Button` is discouraged because T4 needs a clear
  `checked`-support boundary and loader re-reject evidence.
- **`checked` binding path:** reuse the existing single-bool binding path:
  `resolve_prop_key("ToggleButton", "checked") -> IrType::Bool` and
  `register_bool_binding(..., widget_write_property_bool)`. No new binding
  target class is needed.
- **Visual composition:** let `enabled: false` keep the existing disabled
  override. For enabled `ToggleButton`s, `checked` should alter the
  background color in a way that remains visibly distinct from default and
  accent normal states; if the two-frame evidence is ambiguous against Mica,
  T4 records an SI-1 implementation-checkpoint design revision trigger.
- **Author-facing boundary:** do not add `checkable`, generic `Toggle`,
  self-toggle, two-way binding, group widgets, or `==` grammar in T3/T4.

### Gallery / host recon

Current `gallery.ui` sections map to the wireframe target as follows:

| Current section | T2/T5 disposition |
|---|---|
| Top `Grid` with three columns and footer-overflow clip proof | T5 sweeps as a Phase 5 verification surface unless T2 reuses only the overall Grid-frame idea. |
| Static `WrapPanel` with `Photo 1`-`Photo 10` | T5 folds into the final thumbnail area or replaces with the `for`-generated thumbnail surface; no standalone static verification strip remains. |
| `Open lightbox` button + `if is_lightbox_open { ZStack ... }` | Retain and reframe as the target lightbox subtree; prev/next/close remain Button-driven M3 placeholders. |
| `Open placement demo` button + `if is_placement_demo_open { ... }` | T5 removes; the Phase 7b evidence remains in the prior phase, with a retirement note. |
| Scroll up/down buttons + `ScrollView { WrapPanel { for label, index in labels ... } }` | Fold into the agreed operation UI after G(1); keep enough to prove ScrollView + iteration + wrap/overflow. |
| Add/Remove/Clear/Reset buttons | G(1)/T5 decide which survive as minimal A1 operation UI; verification-only buttons are swept. |

Capture-script recon: Phase 8 has no capture script yet. Prior scripts are
`phase-5/.../capture-smoke.ps1`, `phase-5/.../resize-test.ps1`,
`phase-6/.../capture-lightbox.ps1`, `phase-7/evidence/capture-iteration.ps1`,
and `phase-7b/.../capture-placement-demo.ps1`. T5/T7 should create or
derive new Phase 8 capture coordinates from the integrated surface rather
than treating any prior coordinate as retained ground truth.

### Bisectable sequencing

Default sequencing remains valid:

1. T2 may proceed before T3/T4 because it uses Button placeholders and owns
   the wireframe skeleton / G(1) agreement.
2. T3 then adds the compiler/IR surface and fixtures.
3. T4 adds runtime construction, validation, visual state, and Windows
   fixtures.
4. T5 depends on T2+T4 for the final gallery tab band.
5. T6 depends on T5's final `gallery.ui` for C/Zig host ports and CI steps.
6. T7 depends on T5+T6 for authoritative cross-host GUI evidence.

No task insertion or reorder is required. The only plan revision required
before continuing was the T1 responsibility cut recorded in `plan.md`.

### T3 start-gate selection to carry forward

T3 review lane: **full independent review**, with branch/test-focused
attention folded in for SI-3 reject tests.

| Trap | Applies? | Reason / required T3 close artifact |
|---|---:|---|
| #1 semantic migration | Yes | New author-facing widget kind and `checked` attribute cross check/lower/emit/IR fixtures. Close with an `rg`-enumerated call-site audit over `KNOWN_WIDGET_TYPES`, `widget_prop_type`, `lower_node*`, emit node/prop/bind paths, tests, and any silent wildcard/string dispatch. |
| #2 missed side effects | No | T3 should not mutate runtime tree structure, Visual order, gallery layout, or capture coordinates. |
| #3 parallel/derived data drift | No | T3 should introduce no parallel storage or cache. |
| #4 untested authored branch | Yes | SI-3 rejects (`checked` on Button/Text/other non-supporting widgets, non-bool RHS, unknown attrs as applicable) must each have firing tests. |
| #5 carry-forward | Yes | If T3 discovers compiler/IR invariants that T4/T5 must preserve, record them in log.md with a re-trigger; expected candidate is the string-carrier/no-warning-hole invariant. |
| #6 deterministic failure | Conditional | Any repeatable build/test failure gets a rerun history and disposition. |
| #7 GUI positive control | No | T3 has no GUI-render deliverable. |

### T1 responsibility buckets

Required plan revisions before work continues:

- Done in this task: `plan.md` now states T1's critical responsibility cut.
- Done after review: `plan.md` now resolves the T3/T4 bundle exception;
  T1 falsified the compile-error-forcing premise, so the default
  compiler/IR-then-runtime split holds.
- Done in this task: `preamble.md` R-1/R-6 now reflect the string-carrier
  wrong-kind probe and host-template porting facts.

Downstream choices scoped but left to owning tasks:

- T3 owns compiler admission/reject tests, no-unknown-warning positive
  fixture, and emit/roundtrip fixtures.
- T4 owns runtime representation details, `checked` visual composition, and
  loader re-reject tests.
- T2/T5 own A1 placeholder mapping, gallery sweep details, and whether each
  mutation button survives the final operation UI.
- T6 owns exact C/Zig port names and CI command spelling.
- T7 owns GUI screenshot evidence and positive-control analysis.

Owner-facing judgments explicitly not decided by T1:

- G(1) wireframe-fidelity / placeholder agreement.
- G(2) first-render UI direction check.
- SI-1 ambiguity trigger if V-a background-only is not visible enough in the
  actual captured frames.
- Any β fallback substitution; T1 found no source reason to pre-trigger it.
- Whether unknown widget kinds should remain `wasamoc check` warnings with
  exit 0 or become hard diagnostics. T1 records the hole and routes
  `ToggleButton` admission around it with positive fixtures; changing the
  diagnostic policy is a separate owner/decision question and T3 must not
  silently change it while adding `ToggleButton`.

Carry-forward found by T1:

- **Constraint:** Because the widget kind is string-carried and unknown
  widgets are warning-only at `wasamoc check`, positive fixtures for new
  widget kinds must assert the new kind is known rather than merely that
  check exits 0. **Evidence:** T1 wrong-kind probe. **Re-trigger:** any
  future task adding a widget kind while `KNOWN_WIDGET_TYPES` remains a
  warning-only catalog. **Placement:** carry-forward to T3 start gate and
  T3 close audit.

Deterministic failure disposition:

- No deterministic failure occurred. The wrong-kind probe produced a warning
  with exit 0; this is recorded as source behavior, not retried as a flake.

T1 end-gate status: every open point is assigned and scoped; the temporary
probe file was removed; no production source edit remains.
