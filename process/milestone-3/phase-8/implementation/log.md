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
| `Open lightbox` button + `if is_lightbox_open { ZStack ... }` | Retain and reframe as the target lightbox subtree; close remains Button-driven in M3, while prev/next remain inert placeholders unless a later accepted surface adds current-index element access. |
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

## T2 start gate — carry-over check, responsibility cut, and trap selection (2026-07-03)

Carry-over checked before choosing the T2 approach:

- From T1 log / T1 retrospective: Phase 8 capture coordinates are not
  retained ground truth from prior phases; T2/T5/T7 must derive evidence
  from the integrated Phase 8 surface. T2 therefore creates new smoke
  evidence rather than porting a prior capture coordinate.
- From T1 log / T1 retrospective: A1 placeholder mapping and owner-facing
  UI judgments were explicitly not decided by T1. T2 owns the G(1)
  agreement basis and must keep unresolved judgments visible for owner
  confirmation rather than silently folding them into the `.ui`.
- From T1 log: unknown-widget warning-only policy is a T3 carry-forward,
  not a T2 concern.

T2 responsibility after critical re-check: T2 owns the first
non-throwaway Photo Gallery wireframe skeleton plus the layout-skeleton
technical smoke and G(1) agreement table. T2 does **not** own the final
gallery assembly, verification-screen sweep, `ToggleButton` tab band,
three-host parity, or authoritative GUI evidence package. Any layout
breakage found here is either fixed using the already-shipped surface or
triaged to the owner placeholder table / Problem B; T2 must not introduce
a layout-engine change.

Selected traps:

| Trap | Applies? | Reason / close artifact |
|---|---:|---|
| #1 semantic migration | No | T2 changes no compiler/IR/schema/widget-kind surface; `ToggleButton` migration is T3/T4. |
| #2 missed side effects | Yes | Restructuring `gallery.ui` can alter layout, lightbox layering, ScrollView viewport, and later capture assumptions. Close with a structural side-effect enumeration and T5/T7 ownership notes. |
| #3 parallel/derived data drift | No | T2 introduces no parallel storage, cache, or derived index. |
| #4 untested authored branch | No | T2 adds no diagnostic/reject/size branch. |
| #5 carry-forward | Yes | The G(1) agreement table may define placeholders and constraints later tasks must preserve. Close with each carry-forward or an explicit "none". |
| #6 deterministic failure | Conditional | Any repeatable build/smoke failure gets a rerun history and disposition. |
| #7 GUI positive control | Yes | T2's smoke evidence is GUI rendering. Although not the authoritative T7 evidence, it still needs launch + DPI-aware screenshot + assistant analysis, with the positive-control discipline applied to what T2 can prove. |

Review lane: normal task-end review plus the explicit #7 smoke artifact.
T2's screenshot is an internal de-risk gate recorded here, not the
phase-authoritative GUI-render evidence package; T7 remains the full
independent-review GUI evidence task per the preamble.

## T2 implementation + technical smoke results (2026-07-03)

Implemented the first non-throwaway Photo Gallery skeleton in
`examples/gallery/gallery.ui`:

- Root `ZStack` with an opaque background `Box` and a real stretch `Grid`
  frame (`rows: 56 1* 28`, `columns: 1* 1*`). The header uses two
  side-by-side cells; the content and status regions use `column-span: 2`
  to span the full frame width.
- The left header cell holds the tab placeholder `HStack` (`All`,
  `Albums`, `Favorites`), with `All` styled `accent` only as the T2
  placeholder; T5 swaps this to `ToggleButton` / `checked`.
- The right header cell holds the operation `HStack` (`Scroll down`,
  `Scroll up`, `Open lightbox`). Header cells use direct `Cell`
  `h-align` / `v-align` placement (`start` for tabs, `end` for actions).
- Thumbnail area is a spanned content cell with a darker `Box` background
  containing `ScrollView { VStack { padding: 12px; WrapPanel { for label,
  index in labels { Box { aspect: 1:1; Text { ... } } } } } }`, using 18
  placeholder labels.
- Status strip is a spanned static-text row; no collection-length read
  exists in M3.
- Lightbox `ZStack` + `if is_lightbox_open` is retained and reframed as a
  Box + Text placeholder with close / prev / next Buttons. Close is live;
  prev/next are inert M3 placeholders because changing the displayed item
  by index needs element access such as `labels[current]` and generalized
  interpolation, which `gallery-expression-use-cases.md` classifies as
  M-expr2a rather than current M3.

Verification commands:

| Command / evidence | Result |
|---|---|
| `cargo run -p wasamoc -- check examples\gallery\gallery.ui` | green |
| `cargo build -p gallery-rust --release` | green |
| `process\milestone-3\phase-8\implementation\evidence\capture-t2-skeleton.ps1` inside the sandbox | failed reproducibly with off-screen window rect / `CopyFromScreen` invalid handle |
| Same capture script outside the sandbox (`require_escalated`) | green; latest owner-placement run captured `t2-owner-placement-wide.png`, `t2-owner-placement-narrow.png`, `t2-owner-placement-scroll-before.png`, `t2-owner-placement-scrolled.png`, `t2-owner-placement-lightbox.png` |

Assistant image analysis:

- `t2-owner-placement-wide.png`: non-blank Gallery window; left tab group,
  right operation group, padded thumbnail grid, and status strip are
  visible. The wide frame shows 18 thumbnails arranged as 9 columns x 2
  rows. The accepted top-row and thumbnail placeholder colors keep labels
  readable. This run is after the owner placement change to two star
  columns with `column-span: 2` on the content/status rows.
- `t2-owner-placement-narrow.png`: same surface at 760px width; thumbnails
  reflow to 5 columns (with a final partial row). This is the T2
  positive control for the real stretch Grid + WrapPanel path,
  distinguishing it from a static look-alike.
- `t2-owner-placement-scroll-before.png` / `t2-owner-placement-scrolled.png`:
  same 760x420 viewport before and after clicking `Scroll down`; the
  thumbnail content is visibly offset after the click. This closes the T2
  scroll-breakage smoke for the current ScrollView `offset-y` path.
- `t2-owner-placement-lightbox.png`: clicking the right-header
  `Open lightbox` Button opens the conditional `ZStack` subtree; the
  semi-transparent scrim, centered 4:3 Box
  placeholder in the accepted color, caption, and nav/close Buttons are
  visible. This checks the T2 lightbox skeleton and aspect path.

Owner G(1) feedback folded before T2 close: the first capture used a
light tab-band chrome (`Box fill: #ececec`) that was not required by the
wireframe skeleton proof and made default Button labels (`Albums`,
`Favorites`, scroll Buttons) nearly unreadable because the runtime Button
text is light. T2 revised the skeleton to remove the light tab-band
background and to use darker neutral placeholder fills where white text is
the only available text rendering. Current evidence points at the latest
owner-placement run; earlier `t2-skeleton-*`, `t2-row56-*`,
`t2-row64-*`, `t2-accepted-colors-*`, and `t2-span-frame-*` captures were
superseded during iteration and deleted before commit.

Additional owner G(1) feedback before T2 close:

- The top tab/action Button labels looked vertically biased because the
  tab row was too short for the Button natural height plus the containing
  `HStack` padding, so the Button was clipped. T2 fixed the skeleton by
  increasing the top Grid row from `40` to `56`, then rechecked row-height
  candidates by screenshot. Current `.ui` `Grid` tracks do not support
  `auto`; `auto` is treated as future/reserved and rejected by the checker.
- The thumbnail area lacked top/left inset. T2 fixed this with existing
  DSL surface by wrapping the thumbnail `WrapPanel` in a
  `ScrollView`-content `VStack { padding: 12px }`.
- A stretch opaque root background was added so the skeleton does not
  depend on compositor/backdrop transparency or show unrelated desktop
  pixels. The latest owner placement uses that background as the header
  and status band, with a darker content `Box` behind the thumbnail area.
- The top row background was owner-reviewed away from the trial
  `#301010` color and accepted as `#272a2d`: a neutral dark band that
  separates row 1 from row 2 while preserving Button/Text contrast.
- The top row content is split into two Grid cells: the tab `HStack` is
  left-aligned in the first star column, while the scroll/lightbox
  operation `HStack` is right-aligned in the second star column.
- Thumbnail placeholders were accepted as `#4f6272`, and the lightbox
  image placeholder as `#5b7080`, giving the image stand-ins more
  saturation than the earlier gray without competing with the controls.
- The owner chose to address the A2/A13 span-coverage risk in the
  `gallery.ui` skeleton rather than only by table carry-forward. The
  current owner placement uses two star columns and spans the content and
  status rows across both columns with `column-span: 2`. It also exercises
  direct `Cell` placement through the header cells' `h-align` /
  `v-align`.
- The lightbox layout was then adjusted closer to `gallery-wireframe.html`:
  the photo and caption remain centered, while the `<` / `>` nav controls
  move to the photo sides and `x` moves to the upper-right of the lightbox
  grid. Those controls are direct Grid children using `slot.row` /
  `slot.column` / `slot.h-align` / `slot.v-align`; the side controls also
  use `slot.row-span: 2`. This exercises the A13 direct `slot.*` form and
  row-span in the Gallery surface.

T2 technical findings / triage:

| Finding | Disposition |
|---|---|
| A bare root star `Grid` did not create an intrinsic initial window size in the earlier smoke; the host window collapsed to the platform minimum before the capture script explicitly resized it. | The latest owner placement no longer carries the transparent fixed sizer shim. T2 evidence relies on explicit capture-window sizing; the initial-window sizing gap remains a Problem B / explicit-window-sizing symptom, not a layout-engine change. T5 must keep, replace, or deliberately omit any sizing shim with an explicit recorded disposition. |
| Sandbox window positioning produced off-screen rects such as `(2330,1169)` and `CopyFromScreen` failed with "invalid handle"; outside-sandbox execution of the same script moved the window to `(0,0)` and captured successfully. | Classified as sandbox/window-positioning interference, not an app regression. T2 evidence uses the outside-sandbox run, and the failure is closed under trap #6 with rerun/disposition. |
| Light tab-band chrome made default Button labels unreadable. | Removed the light tab background and updated placeholder fills to maintain contrast with the current white Text/Button rendering. This is an owner-feedback correction in the G(1) checkpoint, not a new design surface. |
| Top-row Button labels looked lower than centered. | Fixed in T2 by increasing the tab row height to `56`; the cause was row clipping, not a runtime Button visual issue. `auto` row tracks are unavailable in the current checker/runtime surface. |
| Thumbnail area lacked top/left padding. | Fixed in T2 with `VStack { padding: 12px }` inside the ScrollView content. |
| Trial top-row and placeholder colors needed owner direction. | Owner accepted `#272a2d` for the top row, `#4f6272` for thumbnail placeholders, and `#5b7080` for the lightbox image placeholder. |
| The first T2 skeleton had no `column-span` after removing the old verification Grid. | Fixed in the current owner placement by using a two-star-column frame and applying `column-span: 2` to the content and status rows. This avoids silent deferral of the Grid column-span surface. |
| The first T2 owner-placement lightbox still did not exercise row-span or Grid direct-child `slot.*`. | Fixed in T2 by moving lightbox nav/close Buttons to direct Grid children with `slot.*` placement; the side nav Buttons use `slot.row-span: 2`. |
| Status text remains intentionally static and short. | Accepted M3 placeholder; dynamic collection length remains out of scope. |
| Scroll Buttons remain simple operation controls. | Kept as the current minimal operation UI for M3 offset control; `t2-owner-placement-scrolled.png` verifies a non-zero scroll offset. T5 may restyle or remove them only with an A1-table disposition. |

### T2 G(1) owner-accepted packet / A1 table

This table is the FD-8-G(1) agreement basis accepted by the owner on
2026-07-04 ("OK accept all"). Later tasks must either preserve this basis
or record an explicit deviation with owner/review disposition.

| Gallery surface | M3 implementation / placeholder agreement candidate | Later owner |
|---|---|---|
| Overall frame | Real stretch Grid frame (`rows: 56 / 1* / 28`, `columns: 1* / 1*`) with content/status spanning both columns via `column-span: 2`; no transparent fixed sizer shim in the current `.ui`. | T5 keeps/revises with a recorded Problem B disposition; no layout-engine change in Phase 8. |
| Tabs | T2 plain-Button placeholder on the accepted `#272a2d` header band; labels must remain readable. The tab group is left-aligned in the first header cell, while scroll/lightbox actions are right-aligned in the second header cell. T5 replaces the tab group with 3 `ToggleButton`s and α live exclusion. | T5/T7 prove selected/exclusion. |
| Thumbnail area | Spanned content cell with darker `#2f343b` background; `ScrollView` + padded content `VStack` + `WrapPanel` + `for` over placeholder labels; `#4f6272` Box + Text stands in for images. | T5 final A1 integration; T7 wrap/overflow evidence. |
| Thumbnail highlight | Omitted per TH-a / DD-001; no static selected-thumbnail highlight in M3. | None unless owner revises DD scope. |
| Real images / hit-testing | Box + Text placeholders; `Open lightbox` Button is the M3 hit-testing substitute. | M4. |
| Lightbox | Conditional `ZStack` subtree with semi-transparent scrim (`#101820cc`), accepted `#5b7080` 4:3 Box placeholder, caption, and Button-based controls. The controls are now closer to `gallery-wireframe.html`: `<` / `>` sit at the photo sides using direct Grid-child `slot.*` placement with `slot.row-span: 2`, and `x` sits near the upper-right. The initial `spec.md` pre-doc put scrim opacity styling out of scope to avoid adding alpha styling controls (theming / named palettes / dynamic alpha); Phase 6 FD-G later confirmed that a literal `#RRGGBBAA` scrim is still in scope because it uses the already-admitted `Box.fill` color literal and adds no new styling surface. T2 therefore keeps the semi-transparent scrim as the A4/ZStack transparency exercise, not as a new styling feature. Close is live in M3. Prev/next are intentionally inert placeholders: making them update the displayed label/photo would require current-index element access such as `labels[current]` plus generalized interpolation, classified by `gallery-expression-use-cases.md` as M-expr2a and outside current M3. | T5 finalizes wording/placement; T7 captures open/closed positive control, not prev/next navigation. If T5 chooses the strict opaque-scrim fallback instead, it must record the A4 coverage disposition. |
| Scrollbar / wheel / drag | Programmatic `offset-y` only; scroll Buttons are the candidate minimal operation UI. | T5 decides whether the Buttons survive final A1 UI. |
| Collection mutation Buttons | Not present in T2 skeleton. Owner judgment: omit from the final Gallery UI because `.append`, `.drop-last`, empty-list assignment, static-list reassignment, and dynamic `for` cardinality were already verified by the Phase 7 collection/iteration work rather than needing to remain as end-user Gallery controls. This is a coverage decision, not merely a visual-minimalism choice. | T5/T8 must cite the existing Phase 7 coverage during A1/no-silent-deferral audits; reintroduce a minimal operation UI only if that audit requires Gallery-local exercise. |
| Status | Static text; no collection-length read in M3. | T5 wording only. |
| Verification-only screens | Not carried into the T2 skeleton. T5 still owns the formal sweep / retirement notes for prior phase evidence surfaces and scripts. | T5. |

### T2 end gate - close artifacts (2026-07-04)

Selected traps from the T2 start gate were #2, #5, #6, and #7. The
non-applicable traps from the start gate remain unchanged: no IR/schema
migration (#1), no parallel derived data (#3), and no new diagnostic /
reject / size branch (#4).

**#2 structural side-effect enumeration**

| Structure/state changed | Derived effect / disposition |
|---|---|
| Placement-demo state and overlay (`is_placement_demo_open`) removed from the Gallery surface. | The T2 Gallery now carries only the Photo Gallery skeleton. Phase 7b placement capture surfaces are no longer reachable from `examples/gallery/gallery.ui`; T5 owns the formal retirement/sweep wording for prior verification-only scripts. |
| Footer-overflow clip demo removed from the Gallery surface. | The final app no longer exposes the old top Grid/footer clipping proof. The Grid concept is reused only as the Gallery frame. |
| Static ten-photo thumbnail proof replaced by an 18-label `for`/`WrapPanel` thumbnail surface. | A1 thumbnail wrapping remains exercised, with enough cardinality to show wrap and scroll; old static `Photo 1`-style evidence frames were deleted as obsolete. |
| Add/Remove/Clear/Reset mutation controls removed. | Owner accepted omission from the final Gallery UI because Phase 7 already verified `.append`, `.drop-last`, empty-list assignment, static-list reassignment, and dynamic `for` cardinality. T5/T8 must cite that coverage if the controls remain omitted. |
| Root layout changed to `ZStack` + stretch two-column `Grid` frame with no transparent fixed sizer shim. | Header/content/status placement is now the Gallery skeleton. Problem B remains a layout-engine residual; T2 did not introduce a layout-engine change. |
| Header split into left tab group and right operation group. | Plain Buttons remain T2 placeholders; T5 swaps tabs to `ToggleButton`. The row height is recorded as the T2 value (`56`) while the durable constraint is "no clipping / readable labels." |
| Content/status rows span both Grid columns. | `column-span` is exercised in the Gallery surface instead of silently deferred. |
| Lightbox controls moved to direct Grid children with `slot.*` placement; side controls use `slot.row-span: 2`. | A13 direct `slot.*` and row-span are now exercised by the Gallery surface. Prev/next remain inert placeholders; close remains live. |
| Lightbox scrim retained as `#101820cc`. | The scrim is intentionally semi-transparent as an A4/ZStack transparency exercise using literal `Box.fill`; it is not a new alpha styling API. |
| Capture script coordinates re-derived after owner layout changes. | Current retained script clicks `Scroll down` at `(397,72)` in the narrow/small frame and `Open lightbox` at `(1100,72)` in the wide frame. All obsolete T2 capture generations were deleted. |

**#5 carry-forward**

- T5 must preserve the owner-accepted G(1) / A1 table above or record a
  deviation with owner/review disposition.
- If T5 changes the semi-transparent scrim to the strict opaque fallback,
  it must record the A4/ZStack transparency coverage disposition.
- If T5 removes or restyles the scroll operation controls, it must record
  how M3 offset control remains exercised or why it is no longer needed
  in the Gallery surface.
- If mutation controls remain omitted, T5/T8 must cite the existing Phase
  7 collection/iteration coverage rather than treating the omission as a
  purely visual cleanup.
- T7 authoritative GUI evidence must re-derive coordinates after final
  ToggleButton/restyling work; T2 coordinates are not a reusable contract.
- Problem B remains outside T2: if later work needs a sizing shim or a
  layout-engine correction, it must be owned explicitly by the later task.

**#6 deterministic-failure rerun / disposition**

The recurring `CopyFromScreen` / invalid-handle failures were reproduced
inside the sandboxed path and then cleared by running the GUI capture
through the approved visible-desktop `Start-Process` path. The successful
outside-sandbox run captured all retained frames from the saved script.
Disposition: sandbox/window-positioning interference, not an application
regression. No "green on retry" claim is used as evidence.

**#7 GUI evidence**

Pre-close verification:

- `cargo run -p wasamoc -- check examples\gallery\gallery.ui` -> green.
- `cargo build -p gallery-rust --release` -> green.
- `process\milestone-3\phase-8\implementation\evidence\capture-t2-skeleton.ps1`
  -> green when run through the approved visible-desktop path.

Retained evidence frames:

- `t2-owner-placement-wide.png` and `t2-owner-placement-narrow.png` are
  the resize positive control: the thumbnail wrap and header action
  placement change with viewport width while the app remains readable.
- `t2-owner-placement-scroll-before.png` and
  `t2-owner-placement-scrolled.png` are the scroll positive control: the
  visible thumbnail range changes after clicking `Scroll down`, proving a
  non-zero `scroll_y` path rather than a static look-alike.
- `t2-owner-placement-lightbox.png` is the conditional-overlay positive
  control: the accepted lightbox surface is visibly open, with the
  semi-transparent scrim, centered 4:3 placeholder, side controls using
  direct Grid `slot.*`, row-span side controls, caption, and close button.

Review lane: T2 involved GUI-render evidence, so the close is subject to
independent review before merge. Claude review comments were addressed
before this T2 commit; the commit carries the review trailer.
