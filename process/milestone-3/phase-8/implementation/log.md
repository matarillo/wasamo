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
  accent normal states; if the two-frame evidence is ambiguous against the
  final effective Gallery background, T4 records an SI-1 implementation-
  checkpoint design revision trigger. Mica/backdrop is a possible
  ambiguity factor, not a separate proof target.
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

- Root `ZStack` with a real stretch `Grid` frame (`rows: 56 1* 28`,
  `columns: 1* 1*`). The header uses two side-by-side cells; the content
  and status regions use `column-span: 2` to span the full frame width.
  There is no root fill `Box`; the header/status background is the
  effective window/backdrop surface, while the thumbnail content area has
  its own darker `Box`.
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
  rows. Header/status areas are no longer covered by an explicit root fill
  Box; the current capture shows the dark effective window/backdrop
  surface behind those rows, with the thumbnail area still carried by the
  darker `#2f343b` content Box. This run is after the owner placement
  change to two star columns with `column-span: 2` on the content/status
  rows.
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

After the post-review R-2 discussion, the owner removed the explicit root
fill from `gallery.ui`; the retained `t2-owner-placement-*` frames were
regenerated from the saved script against that current source state.

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
- A stretch opaque root background was trialed to make the skeleton
  independent of compositor/backdrop transparency. After the R-2 review,
  the owner removed that root fill from `gallery.ui`. The current
  owner-placement captures therefore leave the header and status rows on
  the effective window/backdrop surface; only the thumbnail area keeps a
  dedicated darker `#2f343b` content `Box`.
- The trial top-row background (`#301010`, later `#272a2d`) is no longer a
  landed explicit fill. The durable requirement is not that exact color:
  tab/action labels must remain readable, the first and second rows must
  remain visually distinguishable, and T4/T7 must judge
  `ToggleButton.checked` against the final effective background rather
  than treating Mica itself as a separate acceptance target.
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
| Trial top-row and placeholder colors needed owner direction. | Owner first accepted `#272a2d` as a readable trial top-row fill, then removed the explicit root/top-row fill after the R-2 review. Current T2 captures use the effective window/backdrop surface behind the header/status rows. Owner-accepted placeholder colors remain `#4f6272` for thumbnails and `#5b7080` for the lightbox image placeholder. |
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
| Tabs | T2 plain-Button placeholder on the effective window/backdrop header surface; labels must remain readable and the header/content boundary must remain clear. The tab group is left-aligned in the first header cell, while scroll/lightbox actions are right-aligned in the second header cell. T5 replaces the tab group with 3 `ToggleButton`s and α live exclusion. R-2 is about `ToggleButton.checked` being visually unambiguous against the final effective background, not about proving Mica as a separate feature. | T5/T7 prove selected/exclusion and close R-2 with the checked/on-off positive control. |
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
| Root layout changed to `ZStack` + stretch two-column `Grid` frame with no transparent fixed sizer shim and no landed root fill `Box`. | Header/content/status placement is now the Gallery skeleton. Header/status rows use the effective window/backdrop surface; the thumbnail area has its own content `Box`. Problem B remains a layout-engine residual; T2 did not introduce a layout-engine change. |
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
- T4/T7 must close R-2 by proving that the final `ToggleButton.checked`
  visual is unambiguous against the final effective Gallery background.
  This is not a requirement to prove Mica as an independent feature; Mica
  is only one possible backdrop/theme factor that could make a
  background-only checked cue ambiguous.
- T7/T8 must close the stricter `aspect` positive-control question. T2
  only proves that current aspect placeholders render without aborting or
  collapsing; later evidence should either include a frame where
  `Box.aspect` visibly constrains size beyond a coincidental fixed cell, or
  cite the authoritative non-Gallery proof that already covers that
  behaviour.
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

## T3 start gate — carry-over check, responsibility cut, and trap selection (2026-07-04)

Carry-over checked before choosing the T3 approach:

- From T1 log / T1 retrospective: widget kind is string-carried
  (`IrNode.widget_type: String`) and unknown widgets are warning-only at
  `wasamoc check`. T3 therefore must prove `ToggleButton` is admitted as a
  known widget with a no-unknown-warning positive fixture, and must not
  silently change the general unknown-widget policy to a hard error.
- From T1 log: parser / AST / lower / emit are generic for this surface;
  the expected production change is concentrated in `check.rs`, while
  lower/emit tests pin that the generic paths carry `ToggleButton`,
  `checked`, `clicked`, and alpha block-assignment handlers unchanged.
- From T2 log / T2 retrospective: G(1) / A1 table, coordinate drift, R-2
  visual ambiguity, and aspect positive-control carry-forwards are owned by
  T4/T5/T7/T8. They do not expand T3 into GUI/runtime work; T3 only supplies
  the alpha tab-band compile fixture that later Gallery work will consume.

T3 responsibility after critical re-check: T3 owns the authoring compiler
and textual IR boundary for `ToggleButton.checked`: widget admission,
typed attribute admission/rejection, lowering through the existing generic
IR carrier, and textual IR emission fixtures. T3 does not own runtime
loader validation, widget construction, checked visual composition, Gallery
integration, or the project-wide unknown-widget diagnostic policy.

Selected traps:

| Trap | Applies? | Reason / close artifact |
|---|---:|---|
| #1 semantic migration | Yes | New author-facing widget kind and `checked` attribute cross `KNOWN_WIDGET_TYPES`, `widget_prop_type`, lower/emit generic paths, and textual IR fixtures. Close with an `rg`-enumerated call-site audit table, including sites deliberately left generic / unchanged. |
| #2 missed side effects | No | T3 does not mutate runtime tree structure, Visual order, Gallery layout, capture coordinates, or runtime property storage. |
| #3 parallel/derived data drift | No | T3 introduces no parallel vector, derived index, or cache. |
| #4 untested authored branch | Yes | T3 adds checker admission/reject branches for `ToggleButton.checked`; each SI-3 reject must have a firing test. |
| #5 carry-forward | Yes | Preserve and re-record the string-carrier / warning-only known-widget invariant, and record any new compiler/IR invariant T4/T5 must preserve. |
| #6 deterministic failure | Conditional | Any repeatable build/test failure gets a rerun history and disposition before close. |
| #7 GUI positive control | No | T3 has no GUI-render deliverable; T4/T7 own runtime visual / screenshot evidence. |

Review lane: **full independent review**, with branch/test-focused review
folded in for the SI-3 reject matrix.

## T3 end gate — compiler / IR surface close artifacts (2026-07-04)

T3 implemented the compiler/textual-IR half only: `wasamoc` now knows
`ToggleButton`, admits `checked` as a bool property on that widget, rejects
`checked` elsewhere, and the generic lower/emit paths carry the widget kind,
`checked` literal/binding forms, `clicked`, and alpha block-assignment
handlers into textual IR. Runtime loader / widget construction / visual
state remain T4.

Verification commands:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | green |
| `cargo test -p wasamoc` | green: 403 unit tests + 7 roundtrip tests |
| `cargo test --workspace` | green |

**#1 call-site audit table**

`rg` queries used:

- `rg -n "KNOWN_WIDGET_TYPES|widget_prop_type|check_togglebutton_property_name|check_checked_attr_admission|checked|ToggleButton" wasamoc\src wasamoc\tests wasamo-ir\src`
- `rg -n "widget_type|KindPayload|emit_node|lower_node|Member::WidgetDecl|IrNode|bindings|handlers" wasamoc\src\lower.rs wasamoc\src\emit.rs wasamo-ir\src\lib.rs`

| Site | Classification | T3 disposition |
|---|---|---|
| `wasamoc/src/check.rs` `KNOWN_WIDGET_TYPES` | must-dispatch | Added `ToggleButton`; positive test `togglebutton_known_widget_and_attrs_accepted_without_warning` proves no unknown-widget warning. Preserved the general `unknown_widget_type_is_warning_not_error` policy. |
| `wasamoc/src/check.rs` `widget_prop_type` | must-dispatch | Added typed rows for `ToggleButton.text`, `ToggleButton.enabled`, and `ToggleButton.checked`; `style` remains keyword-valued / untyped like Button's existing path. |
| `wasamoc/src/check.rs` property-bind dispatch | must-dispatch | Added `check_checked_attr_admission` for `checked` outside `ToggleButton`; added `check_togglebutton_property_name` so unknown `ToggleButton` attributes reject while admitted attrs flow through existing expr/type checks. Review remediation: `ScrollView` / `ZStack` container-specific unknown-attribute gates intentionally run before the generic `checked` admission gate, so their `checked` rejects use the container-specific diagnostics; component-level `checked` routes through the host-attribute reject, not this helper. |
| `wasamoc/src/lower.rs` `lower_node*` / `Member::WidgetDecl` | generic / unchanged | No schema branch needed: `IrNode.widget_type` remains `String`, props/bindings/handlers are already generic. Added tests pinning `ToggleButton` kind, literal `checked`, bool-state `checked` binding, and alpha handler lowering. |
| `wasamoc/src/emit.rs` `emit_node` / prop / bind / handler emission | generic / unchanged | No emit branch needed: node kind, props, bindings, and handlers emit generically. Added tests for literal, binding, and alpha block textual IR. |
| `wasamo-ir/src/lib.rs` `IrNode` / `IrBinding` / `HandlerExpr` | generic / unchanged | No IR schema change: `widget_type` and `prop_name` are strings, bool binding uses existing `HandlerExpr::BoolPropRead`. Existing IR tests continue green. |
| `wasamoc/tests/roundtrip.rs` public pipeline fixture | must-prove | Added `togglebutton_surface_emits_literal_and_binding_forms`, covering lex -> parse -> check -> lower -> emit for literal and binding `checked` forms plus carried Button attrs. |

**#4 branch tests**

| Branch / diagnostic | Firing test |
|---|---|
| `ToggleButton` known-widget admission and carried attrs | `togglebutton_known_widget_and_attrs_accepted_without_warning` |
| `checked` omitted (default remains runtime-owned, absent IR prop/binding) | `togglebutton_checked_absent_accepted`, `togglebutton_absent_checked_lowers_no_ir_prop_or_binding`, `togglebutton_absent_checked_emits_no_checked_prop_or_binding` |
| component-level `checked` routes to the host-attribute reject | `component_level_checked_routes_to_host_attr_reject` |
| `checked` on `Button` | `checked_on_button_rejected` |
| `checked` on `Text` | `checked_on_text_rejected` |
| `checked` on another non-supporting widget | `checked_on_other_widget_rejected` |
| `checked` on container widgets whose attr gates precede generic admission | `checked_on_scrollview_rejected_by_container_attr_gate`, `checked_on_zstack_rejected_by_container_attr_gate` |
| non-bool literal RHS for `ToggleButton.checked` | `togglebutton_checked_non_bool_rhs_rejected` |
| non-bool state RHS for `ToggleButton.checked` | `togglebutton_checked_i32_state_rejected` |
| unknown `ToggleButton` attribute | `togglebutton_unknown_attr_rejected` |
| alpha tab-band compile shape | `togglebutton_alpha_tab_band_shape_accepted` |
| lower / emit / public pipeline carry-through | `togglebutton_literal_checked_lowers_to_ir_prop`, `togglebutton_checked_binding_lowers_to_bool_prop_read`, `togglebutton_alpha_tab_band_lowers_block_assignment_handlers`, `togglebutton_checked_literal_and_button_attrs_emitted`, `togglebutton_checked_binding_emitted`, `togglebutton_alpha_tab_band_emits_block_handlers`, `togglebutton_surface_emits_literal_and_binding_forms` |

**#5 carry-forward**

- **Constraint:** while widget kind remains string-carried and unknown
  widgets remain warning-only at `wasamoc check`, new widget-kind tasks must
  prove known-widget admission with a no-unknown-warning positive fixture.
  **Evidence:** preserved `unknown_widget_type_is_warning_not_error`; T3
  added `togglebutton_known_widget_and_attrs_accepted_without_warning`.
  **Re-trigger:** any future new widget kind before the diagnostic policy is
  intentionally revised. **Placement:** carry-forward candidate for
  phase-end item 15.
- **Constraint:** `ToggleButton.checked` is now compiler-admitted in textual
  IR, but runtime loader validation / property-key resolution / widget
  construction are still absent by design. **Evidence:** T3 plan/log split
  and T3 tests stop at wasamoc emit. **Re-trigger:** T4 start gate.
  **Placement:** direct T4 start-gate carry-over, not milestone handoff.
- **Constraint:** absent `ToggleButton.checked` is intentionally absent from
  textual IR (`props` / `bindings`) and T4 must materialize the runtime
  default `false`. **Evidence:** T3 review remediation tests
  `togglebutton_absent_checked_lowers_no_ir_prop_or_binding` and
  `togglebutton_absent_checked_emits_no_checked_prop_or_binding`.
  **Re-trigger:** T4 loader/widget defaulting. **Placement:** direct T4
  start-gate carry-over, not milestone handoff.

**#6 deterministic-failure disposition**

No deterministic or recurring failure occurred during T3. The only initial
issue was `cargo fmt --all -- --check` reporting formatting diffs, which was
resolved by running `cargo fmt --all`; the subsequent fmt check was green.

**#7 GUI evidence**

Not applicable. T3 has no GUI-render deliverable; T4/T7 own visual and
screenshot evidence.

## T3 independent review (2026-07-04)

Reviewer: Galileo subagent (`019f2cff-3718-79c3-a25b-53d9830219c1`).

Result: **no findings**. The reviewer checked commits `a79d15e` and
`763b011` in code-review stance and found no blocking implementation bug,
behavioral regression, SI-3 branch/test gap, or start/end-gate recording
gap.

Reviewer spot checks:

- `ToggleButton` admission and `checked` type catalog in
  `wasamoc/src/check.rs`.
- `checked` unsupported-widget rejects, `ToggleButton` unknown-attribute
  reject, and corresponding firing tests.
- lower / emit / public pipeline carry-through tests.
- T3 start/end gate records and T3/T4 responsibility split.

Reviewer-run verification:

- `git diff --check 7c27074..HEAD`
- `cargo fmt --all -- --check`
- `cargo test -p wasamoc togglebutton -- --nocapture`
- `cargo test -p wasamoc checked -- --nocapture`
- `cargo test --workspace`

All reviewer-run checks were green.

## T3 review remediation (2026-07-04)

Claude Code review after the Galileo review raised three low/minor gaps.
T3 addressed them before merge:

- F1: Added lower/emit tests proving absent `checked` produces no IR prop or
  binding, and recorded the T4 carry-forward that runtime must supply
  default `false`.
- F2: Added `ScrollView` / `ZStack` reject tests and recorded that those
  container-specific unknown-attribute gates intentionally precede the
  generic `checked` admission diagnostic.
- F3: Removed the component fallback from `check_checked_attr_admission`;
  component-level `checked` is now pinned as a host-attribute reject.

## T4 start gate — carry-over check, responsibility cut, and trap selection (2026-07-04)

Carry-over checked before choosing the T4 approach:

- From T3 log / T3 retrospective: T4 must mirror the authoring catalog at
  the runtime loader boundary: `ToggleButton` admits `text` / `style` /
  `enabled` / `checked`; `checked` is valid only on `ToggleButton`; unknown
  `ToggleButton` props/bindings must not become a direct textual-IR hole.
- From T3 review remediation: absent `checked` is intentionally absent from
  textual IR, so T4 must materialize the runtime default `false` and pin it
  with a fixture.
- From T1/T3: widget kind is string-carried, so T4 cannot rely on compiler
  exhaustiveness for runtime dispatch. It needs an `rg`-enumerated audit over
  runtime kind/property dispatch sites and direct tests for the new runtime
  branches.
- From T2 log / retrospective: R-2 closes against the final effective Gallery
  background, not by separately proving Mica. T4 can choose and test an
  unambiguous runtime checked colour, but final Gallery-background evidence
  remains T5/T7-owned if later UI work changes the surface.
- From T2/T3 retrospectives: coordinate derivation, A1/G(1) table adherence,
  C/Zig host parity, and authoritative screenshot evidence are later tasks;
  T4 should not absorb them.

T4 responsibility after critical re-check: T4 owns the runtime
defensive-reader and widget-node boundary for `ToggleButton.checked`:
loader validation, property-key resolution, widget construction, defaulting,
Button-family visual/state sharing, bool-binding propagation, alpha
exclusion runtime behaviour, and Button regression fixtures. T4 does not own
Gallery integration, final tab-band assembly, cross-host parity, or the
authoritative GUI evidence package.

Selected traps:

| Trap | Applies? | Reason / close artifact |
|---|---:|---|
| #1 semantic migration | Yes | Runtime gains a `ToggleButton` widget kind and a `checked` property path across `validate()`, `construct_widget`, `resolve_prop_key`, `WidgetData`, property setters/getters, hit testing, hover, and layout leaf dispatch. Close with an `rg`-enumerated runtime call-site audit table. |
| #2 missed side effects | Yes | `checked`, `enabled`, `style`, hover/press, click dispatch, layout sizing, and Visual brush state interact on the shared Button-family visual. Close with a structural side-effect enumeration covering colour priority, layout dirtiness, event dispatch, and Button regression. |
| #3 parallel/derived data drift | No | T4 should not introduce a separate index/cache or parallel child/property table; if a shared helper is introduced, it remains the single Button-family state object. |
| #4 untested authored branch | Yes | T4 adds loader validation/property-key/widget setter branches for `ToggleButton.checked` and malformed runtime IR. Each reject/default/propagation branch needs a direct firing test. |
| #5 carry-forward | Yes | R-2 final-background evidence and any runtime catalog invariant that T5/T7 must preserve need evidence + re-trigger. Expected carry-forward: final Gallery tab-band must re-capture checked/unchecked frames after T5 restyling. |
| #6 deterministic failure | Conditional | Any repeatable build/test/runtime failure gets a rerun history and disposition before close. |
| #7 GUI positive control | No | T4 has live Windows-runtime fixtures and visual-brush assertions, but not a GUI screenshot deliverable. Authoritative launch + screenshot + positive-control evidence is T7; T4 records only the runtime visual cue and carries R-2 to T5/T7 if needed. |

Review lane: **full independent review** because T4 is a runtime structural
change (new widget node / runtime property path) and also adds
diagnostic/reject branches; the review must include the trap-#4
branch/test-focused check.

## T4 end gate — runtime node / visual close artifacts (2026-07-04)

T4 implemented the runtime half only: the loader now accepts and constructs
`ToggleButton`, runtime validation mirrors the T3 authoring catalog for the
new kind, `checked` defaults to `false` when absent from textual IR, the
existing bool-binding writer drives the new checked property, and the
Button-family visual / hit-test / hover / layout leaf paths include the
new runtime kind. Gallery tab-band integration, cross-host parity, and
authoritative screenshot evidence remain T5-T7.

Verification commands:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | green |
| `cargo test -p wasamo-runtime togglebutton -- --nocapture` | green: 14 focused runtime unit tests + 4 Windows runtime integration tests |
| `cargo test -p wasamo-runtime validate_rejects_checked -- --nocapture` | green: 4 non-supporting-kind loader rejects |
| `cargo test --workspace` | green; existing `wasamo` linkable-target / `wasamo-sys` ordering warnings only |

**#1 call-site audit table**

`rg` query used:

```
rg -n "ToggleButton|PROP_TOGGLEBUTTON_CHECKED|validate_phase8_togglebutton|resolve_prop_key|construct_widget|WidgetData::Button|WidgetData::ToggleButton|button_data_mut|effective_button_color|hit_test_click|update_hover|clear_hover|build_layout_tree|__togglebutton_checked" wasamo-runtime\src wasamo-runtime\tests
```

| Site | Classification | T4 disposition |
|---|---|---|
| `ir_loader::validate()` phase gates | must-dispatch | Added `validate_phase8_togglebutton_node_invariants` after the Phase 7 gates. It recurses through widget / `if` / `for` members and rejects `checked` outside `ToggleButton`, unknown `ToggleButton` attrs/bindings, `style` binding, and non-bool `checked` literal/binding forms. |
| `ir_loader::construct_widget` | must-dispatch | Added the `"ToggleButton"` arm. It extracts Button-family `text` / `style` / `enabled`, extracts `checked`, supplies runtime default `false` when absent, and defers initial bound values to the existing binding initial run. |
| `ir_loader::resolve_prop_key` + binding registration loop | must-dispatch | Added `ToggleButton.text`, `ToggleButton.enabled`, and `ToggleButton.checked`; `checked` returns `IrType::Bool` and selects `register_bool_binding` + `widget_write_property_bool`. `ToggleButton.style` remains a static prop only; validation rejects style binding before it can reach dispatch. |
| `WidgetData` / constructors | must-dispatch | Added distinct `WidgetData::ToggleButton(Box<ButtonData>)` and `WidgetNode::toggle_button`, while sharing the Button-family construction helper and `ButtonData`. Runtime kind remains visible for the `checked` support boundary. |
| `WidgetNode` property get/set | must-dispatch | Button-family `text` / `style` / `enabled` dispatch accepts both `Button` and `ToggleButton`; `PROP_TOGGLEBUTTON_CHECKED` is accepted only by `ToggleButton` and writes through `update_toggle_button_checked`. |
| Visual colour helpers | must-dispatch | `ButtonData` carries `checked`; `effective_button_color` applies disabled first, checked second, then normal Button state. `toggle_checked_color` gives the V-a background-only cue used by the runtime fixtures. |
| Hit test / hover / clear-hover | must-dispatch | Replaced Button-only matches with `button_data_mut`, so `ToggleButton` inherits click dispatch, inline handler execution, host signal enqueue, enabled suppression, hover/press state, and hover clearing. |
| Layout leaf dispatch | must-dispatch | `build_layout_tree` routes `WidgetData::ToggleButton` through the same rectangle leaf as Button; no new layout primitive or layout algorithm change. |
| Test-only accessors | must-prove | `__togglebutton_checked_for_test` and the existing enabled accessor let mock-free integration tests assert live runtime state without mocking Win32/WinRT. |
| `window.rs` event forwarding | unchanged / generic | Existing root calls to `hit_test_click`, `update_hover`, and `clear_hover` need no edit because the per-node Button-family dispatch now includes `ToggleButton`. |

**#2 structural side-effect enumeration**

| Structure/state changed | Derived effect / disposition |
|---|---|
| Runtime widget tree gained `WidgetData::ToggleButton`. | Kept as a distinct runtime node so loader validation/property dispatch can distinguish `checked` support; layout treats it as the existing Button leaf. |
| `ButtonData` gained `checked`. | Stored in the shared Button-family state object so style, enabled, hover, and checked colour priority are computed from one source of truth. |
| Background brush colour now depends on checked state. | Priority is disabled > checked > normal Button style/state. Existing disabled contract is preserved by `disabled_togglebutton_suppresses_click_like_button`; existing Button tests remain green in `cargo test --workspace`. |
| Button-family click / hover paths now include ToggleButton. | `button_data_mut` centralizes the shared branch. Alpha fixture proves `clicked` handler block assignment updates state and bindings; disabled fixture proves click suppression still wins. |
| Binding target catalog gained `PROP_TOGGLEBUTTON_CHECKED = 7`. | It is a runtime property key for the existing bool-binding writer path; no new `PropertyValue`, reactive writer class, or ABI header constant was introduced. |
| Runtime validation now closes the direct textual-IR hole for the new kind. | T4 intentionally does not reform the older Button-wide loose catalog; the new closed `ToggleButton` catalog mirrors T3 and is pinned with reject tests. |

**#4 branch tests**

| Branch / diagnostic / behaviour | Firing test |
|---|---|
| `ToggleButton.checked` property-key is bool | `resolve_prop_key_togglebutton_checked_is_bool` |
| Button-family `ToggleButton` attrs resolve through the loader catalog | `resolve_prop_key_togglebutton_button_family_attrs` |
| `ToggleButton` checked literal validates | `togglebutton_checked_literal_validates` |
| `ToggleButton` checked bool binding validates | `togglebutton_checked_binding_validates` |
| `checked` prop on non-supporting kinds | `validate_rejects_checked_on_button_runtime_ir`, `validate_rejects_checked_on_text_runtime_ir` |
| `checked` binding on non-supporting kinds | `validate_rejects_checked_binding_on_button_runtime_ir`, `validate_rejects_checked_binding_on_text_runtime_ir` |
| unknown `ToggleButton` attr / binding | `validate_rejects_togglebutton_unknown_attr_runtime_ir`, `validate_rejects_togglebutton_unknown_binding_runtime_ir` |
| non-bindable `ToggleButton.style` | `validate_rejects_togglebutton_style_binding_runtime_ir` |
| non-bool `ToggleButton.checked` literal / binding | `validate_rejects_togglebutton_checked_non_bool_literal_runtime_ir`, `validate_rejects_togglebutton_checked_non_bool_binding_runtime_ir` |
| wrong expression tag for `ToggleButton.checked` direct IR | `validate_rejects_togglebutton_checked_wrong_read_tag_runtime_ir` |
| loop-local `ToggleButton` bindings stay valid | `validate_accepts_togglebutton_checked_loop_item_binding_runtime_ir`, `validate_accepts_togglebutton_text_loop_item_binding_runtime_ir`, `validate_accepts_togglebutton_text_loop_item_interpolation_runtime_ir` |
| loop index still cannot drive bool `checked` | `validate_rejects_togglebutton_checked_loop_index_binding_runtime_ir` |
| absent `checked` defaults to runtime `false`; literal `true` changes visual | `togglebutton_default_false_and_literal_checked_drive_distinct_visuals` |
| bool-state flip drives checked visual | `togglebutton_bool_state_flip_reaches_checked_visual` |
| alpha exclusion drains to exactly one checked | `togglebutton_alpha_exclusion_click_leaves_exactly_one_checked` |
| disabled ToggleButton suppresses click | `disabled_togglebutton_suppresses_click_like_button` |

**#5 carry-forward**

- **Constraint:** T5/T7 must re-check R-2 against the final effective Gallery
  tab-band background after any Gallery restyling, because T4 proved the
  runtime checked cue by live brush state but did not capture the final
  Gallery surface. **Evidence:** `togglebutton_default_false_and_literal_checked_drive_distinct_visuals`
  and `togglebutton_bool_state_flip_reaches_checked_visual`. **Re-trigger:**
  T5 swaps the tab band to `ToggleButton` or changes the surrounding
  background. **Placement:** carry-forward for T5/T7.
- **Constraint:** `ToggleButton` runtime validation is intentionally stricter
  than the older Button direct-IR catalog; future new widget kinds should
  mirror their compiler catalog at the runtime defensive-reader boundary
  rather than inheriting Button's older loose path. **Evidence:** T4
  `validate_phase8_togglebutton_node_invariants` and reject matrix.
  **Re-trigger:** any future new widget kind or direct textual-IR catalog
  change. **Placement:** phase-end item 15 candidate; may become local-only
  if a broader runtime catalog policy lands before phase close.

**#6 deterministic-failure disposition**

- First focused build failed because `button_data_mut()` hid the disjoint
  `self.data` / `self.visual` field borrow that the old Button-only code
  relied on. Disposition: code defect introduced by the helper extraction;
  fixed by cloning the `SpriteVisual` handle before borrowing Button-family
  state, then rerun green.
- A second focused build failed because the new integration test named the
  loader return type `BuiltComponent`; the actual type is `BuiltUi`.
  Disposition: test type-name error; fixed and rerun green.

**#7 GUI evidence**

Not applicable for T4. The task has mock-free Windows runtime fixtures that
read live `CompositionColorBrush` values and hit-test live `WidgetNode`s, but
no launch + screenshot deliverable. T7 remains the authoritative GUI evidence
owner.

Review lane remains **full independent review** before merge; this close adds
runtime structural change and diagnostic/reject branches.

## T4 independent review and remediation (2026-07-04)

Reviewer: Confucius subagent (`019f2d45-5b56-7463-b0bb-6fa5a4ec6e14`).

Initial result: two findings.

1. The T4 validator accepted malformed direct IR where a `checked` binding
   used the wrong expression tag but referenced a bool state (for example
   `str-prop-read selected` with `selected: bool`). The bool binding
   evaluator would then reject at runtime instead of loader validation
   re-rejecting the malformed IR.
2. The T4 validator recursed into `for` bodies without preserving loop scope,
   so valid loop-local `ToggleButton` bindings such as `checked: flag` for a
   bool collection item or `text: label` for a string collection item were
   rejected at runtime load.

Remediation:

- `validate_phase8_togglebutton_node_invariants` now carries
  `LoopReadScope` through `if` / `for` recursion, matching the existing
  Phase 7 reference validator's loop-local type rules.
- `validate_scalar_binding_expr_type` now checks expression kind and target
  type together: bool targets require bool literals / `bool-prop-read` /
  valid bool loop items, string targets require string forms, and wrong read
  tags are rejected before binding registration.
- Added firing tests for the wrong-tag same-state-type reject, valid
  loop-local `checked` and `text` bindings, and invalid loop-index-to-bool
  `checked`.

Re-review result: one remaining finding.

1. The remediation carried loop scope for direct `ToggleButton.text` loop-item
   bindings, but the string interpolation branch still called
   `validate_expr_references` without the active `LoopReadScope`; valid
   runtime IR such as `bind text = (interp "Tab " ((item-read label)))`
   inside `for label in labels` was still rejected.

Second remediation:

- `validate_scalar_binding_expr_type` now passes `loop_scope` into the
  `Interpolation` reference validator, matching the Phase 7 validator path.
- Added
  `validate_accepts_togglebutton_text_loop_item_interpolation_runtime_ir` as
  a firing positive-control test for loop-local text interpolation.

Final remediation verification:

| Command | Result |
|---|---|
| `cargo fmt --all` | green |
| `cargo fmt --all -- --check` | green |
| `git diff --check` | green |
| `cargo test -p wasamo-runtime togglebutton -- --nocapture` | green: 14 focused runtime unit tests + 4 Windows runtime integration tests |
| `cargo test --workspace` | green; existing `wasamo` linkable-target / `wasamo-sys` ordering warnings only |
