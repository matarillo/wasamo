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
| `cargo test -p wasamo-runtime togglebutton -- --nocapture` | green: 19 focused runtime unit tests + 5 Windows runtime integration tests |
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
| Background brush colour now depends on checked state. | Priority is disabled > checked > normal Button style/state. `disabled_checked_togglebutton_shows_disabled_not_checked_color` pins disabled-over-checked brush priority, `disabled_togglebutton_suppresses_click_like_button` pins disabled click suppression, and checked hover/press colour arms are covered by pure colour-matrix tests. Existing Button tests remain green in `cargo test --workspace`. |
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
| malformed Button-family `ToggleButton` literal attrs | `validate_rejects_togglebutton_text_non_str_literal_runtime_ir`, `validate_rejects_togglebutton_style_non_ident_literal_runtime_ir`, `validate_rejects_togglebutton_enabled_non_bool_literal_runtime_ir` |
| non-bool `ToggleButton.checked` literal / binding | `validate_rejects_togglebutton_checked_non_bool_literal_runtime_ir`, `validate_rejects_togglebutton_checked_non_bool_binding_runtime_ir` |
| wrong expression tag for `ToggleButton.checked` direct IR | `validate_rejects_togglebutton_checked_wrong_read_tag_runtime_ir` |
| loop-local `ToggleButton` bindings stay valid | `validate_accepts_togglebutton_checked_loop_item_binding_runtime_ir`, `validate_accepts_togglebutton_text_loop_item_binding_runtime_ir`, `validate_accepts_togglebutton_text_loop_item_interpolation_runtime_ir` |
| loop index still cannot drive bool `checked` | `validate_rejects_togglebutton_checked_loop_index_binding_runtime_ir` |
| absent `checked` defaults to runtime `false`; literal `true` changes visual | `togglebutton_default_false_and_literal_checked_drive_distinct_visuals` |
| bool-state flip drives checked visual | `togglebutton_bool_state_flip_reaches_checked_visual` |
| checked colour priority and hover/press matrix | `togglebutton_disabled_color_wins_over_checked_and_pressed_state`, `togglebutton_checked_hover_press_color_matrix_is_pinned`, `disabled_checked_togglebutton_shows_disabled_not_checked_color` |
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

Final re-review result: no remaining findings. Confucius checked commit
`3039ee2`, including the interpolation loop-scope patch and the new positive
test.

## T4 Claude review remediation (2026-07-04)

Reviewer: Claude review packet supplied by the owner.

Findings accepted:

1. The end-gate branch/test table omitted authored literal-reject branches
   for `ToggleButton.text`, `ToggleButton.style`, and
   `ToggleButton.enabled`.
2. The structural side-effect table overstated the disabled fixture: it
   proved click suppression, not the disabled-over-checked brush priority.
3. The checked hover/press colour arms were implemented but not pinned.
4. The retrospective double-loop section mixed helper-extraction compile
   learning into a goal/premise retrospective.

Remediation:

- Added firing loader tests for non-string `text`, non-keyword `style`, and
  non-bool `enabled` literal rejects.
- Added pure colour-matrix tests for disabled-over-checked priority and the
  Default/Accent checked hover/press arms.
- Strengthened runtime visual fixtures with unchecked-same-colour and
  unrelated-state no-repaint negative controls, plus
  `disabled_checked_togglebutton_shows_disabled_not_checked_color`.
- Updated the T4 plan and end-gate tables so the recorded branch/test matrix
  matches the authored code.
- Revised the T4 retrospective double-loop section to keep borrow/type-name
  compile failures in deterministic-failure learning rather than goal/premise
  learning.

Deterministic-failure disposition:

- The first focused rerun failed because
  `validate_rejects_togglebutton_text_non_str_literal_runtime_ir` expected
  `str` while the existing diagnostic formatter says `string`. Disposition:
  test expectation defect; corrected to the existing diagnostic text and
  rerun green.

Verification:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | green |
| `git diff --check` | green |
| `cargo test -p wasamo-runtime togglebutton -- --nocapture` | green: 19 focused runtime unit tests + 5 Windows runtime integration tests |
| `cargo test --workspace` | green; existing `wasamo` linkable-target / `wasamo-sys` ordering warnings only |

## T5 start gate — carry-over check, responsibility cut, and trap selection (2026-07-05)

Carry-over checked before choosing the T5 approach:

- From T2 log / T2 retrospective: the owner-accepted G(1) / A1 table is the
  audit basis for Gallery UI work. T5 must preserve each row or record an
  explicit owner/review disposition; mutation controls remain omitted only
  with the Phase 7 collection/iteration coverage citation.
- From T1 / T2 retrospectives: Phase 8 capture coordinates are not retained
  ground truth from prior phases. T5 must derive any first-render /
  alpha-precheck coordinates from the final T5 surface, and must not reuse T2
  coordinates without re-auditing them.
- From T2 / T4 log and retrospectives: R-2 is the checked-visual ambiguity
  question against the final effective Gallery tab-band background, not an
  independent Mica proof. T5 must precheck the final tab band, while T7 owns
  the authoritative two-frame evidence package.
- From T3 / T4 retrospectives: `ToggleButton` compiler and runtime surfaces
  are available; T5 should use the accepted W1 alpha exclusion shape rather
  than inventing a new group widget, self-toggle, equality expression, or
  diagnostic-policy change.
- From T2 retrospective: the strict `Box.aspect` positive-control question
  remains T7/T8-owned. T5 keeps the 4:3 and 1:1 placeholders rendering but
  does not try to close the final aspect proof.

T5 responsibility after critical re-check: T5 owns the landed Rust-host
Gallery surface that T6 ports and T7 captures. It completes the T2 skeleton
with the real `ToggleButton` tab band, records the verification-screen sweep
and A1 table audit, keeps the agreed M3 placeholders honest, derives the T5
first-render / alpha-precheck capture coordinates, and runs the FD-8-G(2)
first-render direction check. T5 does not own C/Zig host parity,
authoritative GUI evidence, public-draft sync, owner final smoke, new layout
engine work, real images, thumbnail hit-testing, or collection-mutation
controls unless the A1 audit disproves the existing Phase 7 coverage
disposition.

Selected traps:

| Trap | Applies? | Reason / required T5 close artifact |
|---|---:|---|
| #1 semantic migration | No | T5 consumes the already-landed `ToggleButton` compiler/runtime surface and does not change enum / IR / schema types or widget catalogs. |
| #2 missed side effects | Yes | Replacing the tab band and finalizing the Gallery surface can alter layout, lightbox layering, scroll controls, retained evidence scripts, and T6/T7 assumptions. Close with a structural side-effect enumeration plus the A1 row-by-row audit and retired-script disposition. |
| #3 parallel/derived data drift | No | T5 introduces no parallel vector/map/index or cache. The three tab booleans are source state, not a derived mirror; alpha handlers update all three explicitly. |
| #4 untested authored branch | No | T5 adds no diagnostic / reject / size branch. Existing compiler/runtime ToggleButton tests cover the authored branches from T3/T4. |
| #5 carry-forward | Yes | Any final Gallery constraint later tasks must preserve (T6 port assumptions, T7 evidence coordinates, R-2 precheck outcome, omitted mutation-control citation) must be recorded with evidence and a re-trigger. |
| #6 deterministic failure | Conditional | Any recurring build/test/capture failure gets rerun history and disposition; no "green on retry" without cause. |
| #7 GUI positive control | Yes | T5 includes a Rust-host first-render / alpha-exclusion precheck for G(2). Close with launch + screenshot + analysis and a two-frame alpha precheck. T7 remains the authoritative GUI evidence owner. |

Review lane: normal task-end review as planned for T5, with the trap-#2
coordinate / structural-side-effect check and the T5-scoped #7 precheck
artifact. T7 remains the full independent review lane for authoritative
GUI-render evidence; any T5 change that expands into runtime structure,
schema/IR migration, or authoritative GUI evidence would require
reclassification before merge.

## T5 end gate — Gallery integration close artifacts (2026-07-05)

T5 completed the Rust-host Gallery surface only. The T2 tab placeholder is
now the real `ToggleButton` alpha exclusion band; the rest of the T2
owner-accepted Gallery skeleton was preserved; no C/Zig host, spec draft,
or authoritative T7 evidence work was absorbed.

Verification commands:

| Command | Result |
|---|---|
| `cargo run -p wasamoc -- check examples\gallery\gallery.ui` | green |
| `cargo build -p gallery-rust --release` | green; existing `wasamo` linkable-target warning only |
| `git diff --check` | green; existing working-copy LF→CRLF warnings only |
| `cargo fmt --all -- --check` | green |
| `cargo test --workspace` | green; existing `wasamo` linkable-target / `wasamo-sys` ordering warnings only |
| `process\milestone-3\phase-8\implementation\evidence\capture-t5-gallery.ps1` inside the sandbox | failed reproducibly with off-screen window rect `(2330,1169)` and `CopyFromScreen` invalid handle |
| Same capture script outside the sandbox (`require_escalated`) | green; captured `t5-gallery-default-all.png`, `t5-gallery-selected-albums.png`, `t5-gallery-selected-favorites.png`, `t5-gallery-lightbox.png`, `t5-gallery-closed-after-lightbox.png`, `t5-gallery-scroll-before.png`, and `t5-gallery-scrolled.png` |

FD-8-G(2): owner first-render check passed on 2026-07-05 ("G(2) OK").

**A1 / G(1) table audit**

| Gallery surface | T5 disposition |
|---|---|
| Overall frame | Preserved the T2 `ZStack` + stretch `Grid` frame (`rows: 56 / 1* / 28`, `columns: 1* / 1*`) with content/status `column-span: 2`. No transparent fixed sizer shim or layout-engine change was introduced; T5 capture uses explicit window sizing as T2 did. |
| Tabs | Replaced the three plain Buttons with three `ToggleButton`s bound to `tab_all_selected`, `tab_albums_selected`, and `tab_favorites_selected`. Each handler writes all three bool states, preserving alpha live exclusion. `default-all`, `selected-albums`, and `selected-favorites` frames show exactly one selected background at a time. |
| Thumbnail area | Preserved the darker `#2f343b` content Box, `ScrollView`, padded `VStack`, `WrapPanel`, `for label, index in labels`, `#4f6272` 1:1 Box placeholders, and Text labels. The narrow scroll pair proves the retained scroll offset surface still moves the thumbnail list. |
| Thumbnail highlight | Still omitted per TH-a / DD-001. No static selected-thumbnail highlight was reintroduced. |
| Real images / hit-testing | Still Box + Text placeholders; `Open lightbox` remains the M3 hit-testing substitute. No real image or thumbnail-click surface was added. |
| Lightbox | Preserved conditional `ZStack`, semi-transparent `#101820cc` scrim, 4:3 image placeholder, caption, close Button, and inert prev/next Buttons. The close action is live; `t5-gallery-lightbox.png` and `t5-gallery-closed-after-lightbox.png` form the T5 precheck pair. |
| Scrollbar / wheel / drag | Kept the Scroll down / Scroll up Buttons as the minimal M3 offset-control UI. `t5-gallery-scroll-before.png` and `t5-gallery-scrolled.png` prove non-zero offset after the derived click coordinate. |
| Collection mutation Buttons | Remain omitted. T5 cites the T2 owner disposition and existing Phase 7 coverage for `.append`, `.drop-last`, empty-list assignment, static-list reassignment, and dynamic `for` cardinality; T8 still owns the no-silently-deferred-surface audit citation. |
| Status | Preserved the static status text. No collection-length read was added. |
| Verification-only screens | No placement-demo state/button/overlay, footer-clip demo, standalone static ten-photo strip, mutation-control dashboard, or verification menu reappears in `examples/gallery/gallery.ui`. |

**Verification-surface sweep / retired evidence**

`rg` query used:

```
rg -n "placement|footer|Photo [0-9]|Add|Remove|Clear|Reset|is_placement|ToggleButton|tab_.*selected|checked:|Open lightbox|Scroll down|Scroll up|aspect|slot\.row-span|column-span" examples\gallery\gallery.ui
```

Result: only the expected T5 surface remains: three `ToggleButton`s and
their bool states / `checked` bindings, scroll and lightbox operation
Buttons, `column-span`, `aspect`, and `slot.row-span`. The retired prior
capture scripts remain as historical evidence under their owning phase
directories:

- `process\milestone-3\phase-5\implementation\evidence\capture-smoke.ps1`
- `process\milestone-3\phase-5\implementation\evidence\resize-test.ps1`
- `process\milestone-3\phase-6\implementation\evidence\capture-lightbox.ps1`
- `process\milestone-3\phase-7\evidence\capture-iteration.ps1`
- `process\milestone-3\phase-7b\implementation\evidence\capture-placement-demo.ps1`

T5 does not delete or rewrite those historical scripts; it records that the
Phase 8 Gallery no longer exposes their old verification-only sub-screens,
and the new Phase 8 capture coordinates start from
`capture-t5-gallery.ps1`.

**#2 structural side-effect enumeration**

| Structure/state changed | Derived effect / disposition |
|---|---|
| Header tab group changed from Button placeholders to `ToggleButton`s. | Added three bool states as the source-of-truth tab state. Each click handler writes all three states, so no derived selected index or parallel cache is introduced. Capture frames prove the selected background moves and previous selection clears. |
| `All` no longer uses `style: accent`; selected state supplies the visual cue. | This keeps the V-a background-only checked cue as the tab selection signal. The T5 precheck shows the blue selected background is readable against the final effective header background. T7 still owns the authoritative selected/exclusion evidence. |
| Capture coordinates changed from T2. | New script derives T5 coordinates: Albums `(112,72)`, Favorites `(220,72)`, Open lightbox `(1100,72)`, close `(885,168)`, and Scroll down `(397,72)` after resizing to `760x420`. The first lightbox-close attempt used the wrong y coordinate; the script now captures `closed-after-lightbox` before the scroll pair to prove the close succeeded. |
| Lightbox state can obscure scroll evidence if not closed. | Closed the lightbox before resizing for scroll frames; `scroll-before` / `scrolled` now show the Gallery list rather than the lightbox overlay. |
| Verification-only surfaces remain absent. | The Gallery is now the single Rust-host verification surface for A1; T6 ports this `.ui`, and T7 captures authoritative GUI evidence from it. |

**#5 carry-forward**

- **Constraint:** T6 must port the final T5 `examples/gallery/gallery.ui`
  exactly enough that the same alpha tab band, scroll controls, and lightbox
  placeholders appear in C/Zig hosts. **Evidence:** T5 A1 table audit and
  `t5-gallery-default-all.png`. **Re-trigger:** any T6 host-port divergence
  or `.ui` edit after this commit. **Placement:** carry-forward for T6/T7.
- **Constraint:** T7 must treat T5 frames as a precheck, not the
  authoritative GUI package. It must re-capture the selected/exclusion,
  lightbox open/closed, wrap/overflow, and aspect evidence after T6 merges.
  **Evidence:** T5 review lane and `capture-t5-gallery.ps1`. **Re-trigger:**
  T7 evidence start or any final Gallery surface change. **Placement:**
  carry-forward for T7.
- **Constraint:** T7's authoritative capture must be planned as a
  visible-desktop / outside-sandbox run, and the capture harness itself is a
  known fragile dependency rather than a neutral detail: T2 and T5 both
  reproduced sandboxed off-screen `CopyFromScreen` failure before succeeding
  outside the sandbox. **Evidence:** T2 and T5 deterministic-failure
  dispositions. **Re-trigger:** T7 GUI-evidence start or any future task that
  treats `CopyFromScreen` screenshot automation as merge-gate evidence.
  **Placement:** carry-forward for T7 and phase-end item 15 consideration.
- **Constraint:** If future work moves header controls or changes window
  sizing, coordinate-based capture scripts must be re-derived and include a
  state-confirming frame after modal/lightbox close actions. **Evidence:**
  the first T5 close coordinate missed, causing the scroll pair to be
  captured under the lightbox until the script added
  `closed-after-lightbox`. **Re-trigger:** any layout-affecting Gallery UI
  change or capture-script reuse. **Placement:** carry-forward candidate for
  phase-end item 15.
- **Constraint:** Collection mutation controls remain omitted by relying on
  Phase 7 evidence, not by silently dropping the surface. **Evidence:** T2
  G(1) owner acceptance and T5 A1 audit. **Re-trigger:** T8
  no-silently-deferred-surface audit. **Placement:** carry-forward for T8.

**#6 deterministic-failure disposition**

- The first sandboxed capture failed with the same off-screen
  `CopyFromScreen` invalid-handle symptom recorded in T2. Disposition:
  sandbox/window-positioning interference, not an app regression; the same
  script succeeded outside the sandbox with the window at `(0,0)`.
- The initial T5 capture script did not stop on the `CopyFromScreen`
  exception. Disposition: script robustness defect; added
  `$ErrorActionPreference = "Stop"`.
- The first visible-desktop T5 script captured the scroll pair while the
  lightbox was still open because the close click y coordinate missed the
  close Button. Disposition: coordinate derivation defect; corrected close
  coordinate to `(885,168)` and added a `closed-after-lightbox` frame before
  the scroll pair. Rerun green.

**#7 GUI positive-control precheck**

T5 GUI evidence is a precheck for G(2), not the authoritative T7 package.
The frames show:

- `t5-gallery-default-all.png`: Rust host renders the integrated Gallery;
  All is selected, thumbnail WrapPanel renders, operation Buttons and status
  are visible.
- `t5-gallery-selected-albums.png` and
  `t5-gallery-selected-favorites.png`: clicking tabs moves the selected
  background and clears the previous tab, proving live alpha exclusion rather
  than a static selected look-alike.
- `t5-gallery-lightbox.png` and
  `t5-gallery-closed-after-lightbox.png`: the conditional lightbox subtree is
  present then absent after the close Button.
- `t5-gallery-scroll-before.png` and `t5-gallery-scrolled.png`: at the narrow
  viewport, clicking Scroll down moves the thumbnail content from
  `IMG 001`-`IMG 010` to `IMG 006`-`IMG 015`, proving the retained
  `ScrollView.offset-y` operation.

T7 remains responsible for the final assistant GUI evidence with the full
positive-control set, including strict aspect closure or the T8 citation.

## T5 independent review (2026-07-05)

Reviewer: Peirce subagent (`019f2f55-8a72-7bf1-9456-9f882646503c`).

Result: **no findings**.

Review scope: commits `8f906f9` and `96e1a68`; T5 Gallery `.ui`
integration, start/end gate artifacts, T5 evidence script and committed PNG
frames, and `retrospectives/t5.md`.

Reviewer confirmation:

- `ToggleButton` tab integration is confined to the Gallery surface and does
  not touch compiler/runtime/layout code.
- T5 responsibility split is clear: C/Zig hosts remain T6, authoritative GUI
  evidence remains T7, and public-draft/audit work remains T8 onward.
- Start/end gates include the A1/G(1) audit, trap #2 structural side-effect
  enumeration, trap #5 carry-forward entries, and the T5-scoped #7 precheck.
- Committed PNG evidence shows exclusive selected-tab movement, lightbox
  open/close, and scroll offset movement.
- T5 retrospective records the task checklist and T6/T7/T8 ownership.

Reviewer-noted test gaps are intentional downstream ownership: T7 must
re-derive absolute-coordinate capture frames for final evidence; strict
`Box.aspect` positive-control closure remains T7/T8; mutation-control
omission remains T8's no-silently-deferred-surface audit item.

## T5 Claude review remediation (2026-07-05)

Reviewer: Claude review packet supplied by the owner.

Findings accepted:

1. The T5 retrospective double-loop section treated the missed close-click
   coordinate mostly as an execution-method lesson. It needed to ask the
   deeper premise question: whether coordinate-based screenshot capture is an
   adequate evidence form for the T7 authoritative GUI package after T2 and
   T5 both exposed the same class of harness fragility.
2. The T7 carry-forward under-surfaced the known capture-harness dependency:
   authoritative evidence should be planned as visible-desktop /
   outside-sandbox work, not as a neutral rerun of the same coordinate script
   style.

Remediation:

- Strengthened the T5 retrospective double-loop section so the planning
  premise under review is the evidence style itself, while the close-click
  miss remains in the deterministic-failure / single-loop record.
- Added an explicit T7 carry-forward that names the sandboxed
  `CopyFromScreen` / coordinate harness as a twice-observed fragile
  dependency and requires T7 to plan authoritative capture on the
  visible-desktop path.

No code, GUI surface, evidence frames, or verification results changed.

## T6 start gate — carry-over check, responsibility cut, and trap selection (2026-07-05)

Carry-over checked before choosing the T6 approach:

- From T1 log: `counter-c` and `counter-zig` are templates, not generic
  Gallery hosts. T6 must port counter-specific artifact names
  (`COUNTER_UI`, `COUNTER_UIC`, `COUNTER_UIC_H`, `COUNTER_UIC`,
  `counter_uic`, executable names, `.ui` paths, README text) to Gallery
  names and keep the build-order assumption that `target/release/wasamoc.exe`
  and `target/release/wasamo.dll.lib` already exist.
- From T5 log / T5 retrospective: `examples/gallery/gallery.ui` is the final
  Rust-host Gallery surface for T6. T6 must not change tab-band, scroll, or
  lightbox semantics unless a host-port issue forces a recorded disposition.
- From T5 log / T5 retrospective: authoritative GUI evidence remains T7. T6
  may capture a default-render cross-host parity precheck, but it must not
  treat T5 coordinates or any T6 coordinates as T7 ground truth.
- From T2/T5 deterministic-failure records: any screenshot-based T6 parity
  run is planned as visible-desktop / outside-sandbox evidence; sandboxed
  `CopyFromScreen` off-screen failure is a known harness risk, not a neutral
  detail.
- From T5 carry-forward to T8: collection mutation controls remain omitted by
  owner disposition and Phase 7 evidence citation. T6 should not reintroduce
  mutation UI while porting hosts.

T6 responsibility after critical re-check: T6 owns the C and Zig Gallery host
ports, local clean-order build rehearsal, CI per-example steps, and a
default-view cross-host parity precheck. It does not own the authoritative
two-frame selected/exclusion, lightbox, wrap/overflow, or aspect evidence
package; T7 re-captures that final state set after T6 merges. T6 must keep
the new hosts declarative (`WASAMO_LOAD_MEMORY` / embedded `.uic`, no
host-side widget mutation) and must not change compiler/runtime/layout or the
Gallery `.ui` surface unless the plan is revised first.

Selected traps:

| Trap | Applies? | Reason / required T6 close artifact |
|---|---:|---|
| #1 semantic migration | No | T6 adds example hosts and CI steps only; no enum, IR, schema, widget catalog, or runtime property surface changes. |
| #2 missed side effects | Yes | Adding C/Zig hosts changes example inventory, generated artifact names, build ordering assumptions, runtime DLL requirements, README instructions, CI coverage, and downstream T7 parity assumptions. Close with a structural side-effect enumeration and host-port delta table. |
| #3 parallel/derived data drift | No | T6 introduces no parallel source/runtime data structure. Generated `.uic` / embedded headers are build artifacts derived by the host build systems, not committed source mirrors. |
| #4 untested authored branch | No | T6 adds no diagnostic / reject / size branch. Build-script failure messages are ported from the existing templates; direct firing tests are not appropriate unless T6 authors a new conditional branch beyond template adaptation. |
| #5 carry-forward | Yes | T7 needs final host paths, parity-frame assumptions, and any host-port divergence; T8/phase-end may need CI/build-order learning. Close with carry-forward entries and re-trigger criteria or explicit none. |
| #6 deterministic failure | Conditional | Any recurring CMake/Zig/build/capture failure gets a rerun history and disposition; no "green on retry" without cause. |
| #7 GUI positive control | Yes, scoped | T6's parity precheck is GUI-host rendering. Close with launch + default-view screenshots + analysis that distinguishes "host loaded the integrated Gallery" from merely staying alive. T7 remains the authoritative GUI positive-control owner. |

Review lane: normal task-end review with explicit checks for trap #2
host/build/CI side effects and the T6-scoped GUI parity artifact. If T6
expands into runtime structure, compiler/IR migration, diagnostic branches,
or the authoritative T7 GUI evidence package, the review lane must be
reclassified before merge.

## T6 end gate — Gallery C/Zig hosts + CI step close artifacts (2026-07-05)

T6 completed the missing C and Zig Gallery hosts and added CI build coverage
for both. The T5 `examples/gallery/gallery.ui` surface was not changed; the
new hosts load the same compiled `.uic` through memory embedding and perform
no host-side widget mutation. T7 remains the owner of the authoritative
positive-control GUI evidence package.

**Host-port delta table**

| Template surface | Gallery port |
|---|---|
| `examples/counter-c/CMakeLists.txt` | New `examples/gallery-c/CMakeLists.txt` points at `examples/gallery/gallery.ui`, emits `gallery.uic` / `gallery_uic.h`, uses `GALLERY_UIC`, builds `gallery-c.exe`, and keeps the release `wasamoc.exe` / `wasamo.dll.lib` ordering checks. |
| `examples/counter-c/embed_uic.cmake` | New Gallery copy defaults to `GALLERY_UIC` and `WASAMO_GALLERY_UIC_H`, with optional `ARRAY_NAME` / `HEADER_GUARD` inputs from CMake. |
| `examples/counter-c/main.c` | New `gallery-c/main.c` includes `gallery_uic.h` and calls `wasamo_load_ui(WASAMO_LOAD_MEMORY, GALLERY_UIC, GALLERY_UIC_LEN, ...)`; no `wasamo_set_property` calls. |
| `examples/counter-zig/build.zig` | New `examples/gallery-zig/build.zig` accepts `-Dgallery-ui`, compiles `gallery.ui` to `gallery.uic`, exposes anonymous import `gallery_uic`, and builds `gallery-zig.exe`. |
| `examples/counter-zig/main.zig` | New `gallery-zig/main.zig` embeds `gallery_uic` and calls `wasamo_load_ui(WASAMO_LOAD_MEMORY, ...)`; no host-side mutation. |
| `.github/workflows/ci.yml` | Added `gallery-c (CMake, Release)`, `gallery-zig (Zig, ReleaseSafe)`, and `wasamoc check gallery.ui` steps mirroring the counter ordering after the release workspace build. |

Verification commands:

| Command | Result |
|---|---|
| `cargo build --release --workspace` | green; existing `wasamo` linkable-target warning only |
| `cargo run --release -p wasamoc -- check examples\gallery\gallery.ui` | green |
| `cmake -S examples/gallery-c -B build/gallery-c` via the Visual Studio CMake path | green; plain `cmake` was not on this shell's `PATH` |
| `cmake --build build/gallery-c --config Release` via the Visual Studio CMake path | green; produced `build/gallery-c/Release/gallery-c.exe` |
| `zig build -p . ...` inside the sandbox | failed with `AccessDenied` reading Zig std/compiler cache |
| same `zig build -p . ...` outside the sandbox | green |
| `zig build -p ../../build/gallery-zig ...` outside the sandbox | green; produced `build/gallery-zig/bin/gallery-zig.exe` |
| `cargo fmt --all -- --check` | green |
| `zig fmt --check examples\gallery-zig\build.zig examples\gallery-zig\main.zig` outside the sandbox | green |
| `git diff --check` | green; existing working-copy LF->CRLF warnings only |
| `cargo build --workspace` | green |
| `cargo test --workspace` | green; existing `wasamo` linkable-target / `wasamo-sys` ordering warnings only |
| `capture-t6-parity.ps1` outside the sandbox | green; captured `t6-parity-rust.png`, `t6-parity-c.png`, and `t6-parity-zig.png` |

Remote GitHub Actions was not run for T6 before this task-end record because
`feat/m3-phase-8-t6` has no visible remote tracking branch in this workspace
and push is a separate owner gate. The new CI commands were locally rehearsed
in the same build order; a workflow run id remains phase-branch / phase-end
owned if the owner wants remote CI before merge.

**#2 structural side-effect enumeration**

| Structure / state changed | Derived effect / disposition |
|---|---|
| Example inventory now has `examples/gallery-c/` and `examples/gallery-zig/`. | README / build scripts identify them as Gallery hosts, not counter variants. Generated Zig outputs from local verification were removed from the source tree; committed files are source + evidence only. |
| C host build now shells out to `wasamoc build examples/gallery/gallery.ui`. | The CMake script keeps the same release build ordering guard as `counter-c`; missing `wasamoc.exe`, `wasamo.dll.lib`, or `gallery.ui` fails at configure/build time with explicit messages. |
| C embedded artifact names changed. | `gallery.uic`, `gallery_uic.h`, `GALLERY_UIC`, and `GALLERY_UIC_LEN` are used consistently across CMake, generated header, and `main.c`; no `COUNTER_*` source names remain in `gallery-c`. |
| Zig host build now shells out to `wasamoc build examples/gallery/gallery.ui`. | `build.zig` keeps the release defaults for `wasamoc.exe`, `wasamo.dll.lib`, and `bindings/zig/wasamo.zig`; local verification also built to `build/gallery-zig` to avoid leaving source-tree artifacts. |
| Zig embedded artifact name changed. | `gallery_uic` is used consistently as the anonymous import and `@embedFile` key; no `counter_uic` source names remain in `gallery-zig`. |
| CI per-example coverage changed. | Added Gallery C/Zig and Gallery `wasamoc check` steps after the release workspace build, matching the counter ordering and adding no new language/build system. |
| T6 GUI parity evidence added. | The capture script uses host executable paths for Rust/C/Zig and adds `target/release` to `PATH` so C/Zig can locate `wasamo.dll`; screenshots are default-view parity prechecks only and are not T7 positive-control ground truth. |

**#5 carry-forward**

- **Constraint:** T7 must re-capture the authoritative selected/exclusion,
  lightbox, wrap/overflow, and aspect/citation evidence after the T6 host
  additions; T6 parity frames are default-view no-regression evidence only.
  **Evidence:** `capture-t6-parity.ps1` and `t6-parity-*.png` show all hosts
  load the same initial Gallery, but no T7 state transitions are exercised.
  **Re-trigger:** T7 evidence start or any post-T6 Gallery UI / host-path
  change. **Placement:** carry-forward for T7.
- **Constraint:** Future C/Zig Gallery host changes must preserve the
  declarative memory-load boundary unless a separate task/decision changes
  host responsibilities. **Evidence:** both new hosts embed `.uic` and call
  `wasamo_load_ui(WASAMO_LOAD_MEMORY, ...)` with no host-side widget mutation.
  **Re-trigger:** any future edit to `examples/gallery-c/main.c`,
  `examples/gallery-zig/main.zig`, or their build scripts. **Placement:**
  carry-forward candidate for phase-end item 15.
- **Constraint:** CI/local build instructions for C/Zig Gallery hosts depend
  on a prior release build of `wasamoc.exe` and `wasamo.dll.lib`, matching the
  existing counter examples. **Evidence:** CMake configure checks and Zig
  defaults both point at `target/release`; local rehearsal built release
  workspace first. **Re-trigger:** any CI step reorder, host build-script
  path change, or future attempt to run these examples without the release
  workspace build. **Placement:** carry-forward candidate for phase-end
  item 15.

**#6 deterministic-failure disposition**

- Plain `cmake` failed because CMake was not on this PowerShell PATH.
  Disposition: environment PATH issue, not a Gallery host defect. The Visual
  Studio-installed CMake executable at
  `C:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe`
  configured and built the host green.
- The first sandboxed Zig build failed with `AccessDenied` while reading Zig
  std/compiler-cache files under the WinGet Zig installation. Disposition:
  sandbox access limitation, not a Gallery host defect. The same command
  succeeded outside the sandbox, and a second outside-sandbox build placed
  outputs under `build/gallery-zig`.

**#7 GUI parity precheck**

`capture-t6-parity.ps1` ran through the approved visible-desktop path and
captured:

- `t6-parity-rust.png`
- `t6-parity-c.png`
- `t6-parity-zig.png`

Assistant analysis: all three frames show the same integrated Gallery
default view at 1200x760 / 96 DPI: window title "Gallery"; All selected in
the `ToggleButton` tab band; Albums and Favorites unselected; Scroll down,
Scroll up, and Open lightbox controls in the header; 18 thumbnail
placeholders labelled `IMG 001 #0` through `IMG 018 #17`; and the status
strip text `18 placeholders - Image and hit-testing are M4`. This proves
each host loaded and rendered the same Gallery surface rather than merely
starting a process. T7 remains responsible for the selected/exclusion,
lightbox, wrap/overflow, and aspect positive controls.

## T6 independent review (2026-07-05)

Reviewer: Cicero subagent (`019f2fda-1fc6-7ea3-b095-a1f41f0b4601`).

Result: **no findings**.

Review scope: commits `f3ccaef` and `74a38b6`; new Gallery C/Zig hosts, CI
steps, T6 plan/log/evidence, committed parity PNGs, and `retrospectives/t6.md`.

Reviewer confirmation:

- C/Zig host names and paths are consistently ported to Gallery.
- Both new hosts preserve the embedded-IR `WASAMO_LOAD_MEMORY` boundary and
  do not perform host-side widget mutation.
- CI Gallery steps follow the release workspace build and match the AGENTS.md
  `wasamoc` build-ordering requirement.
- No tracked generated artifacts (`.zig-cache`, `zig-out`, generated `.uic`,
  or generated headers) leaked into the commit set.
- The committed T6 parity PNGs show the same initial Gallery view across
  Rust, C, and Zig.

Reviewer-noted residuals are intentional downstream ownership: remote GitHub
Actions run id remains push / phase-end owned, and the selected/exclusion,
lightbox, wrap/overflow, and aspect positive controls remain T7-owned.

## T6 Claude review disposition (2026-07-05)

Reviewer: Claude review packet supplied by the owner.

Findings disposition:

1. **CI YAML parse-level verification not run before merge** (low).
   Accepted by the owner on 2026-07-05. The T6 end-gate relaxation is
   intentional: push is a separate gate, and phase-end already owns the
   push + GitHub Actions CI confirmation procedure. T6 therefore closes with
   local rehearsal of the CI commands and records that remote workflow parse /
   run evidence remains phase-end owned rather than task-close owned.
2. **Retrospective double-loop section lightly mixes execution mechanics**
   (low). Recorded as learning for the next start gate: keep premise /
   planning-hypothesis learning separate from deterministic-failure and
   environment mechanics.
3. **`embed_uic.cmake` added a `HEADER_GUARD` parameter while described as a
   pure port** (trivial). Recorded as harmless; behavior is correct and the
   host boundary remains the counter-template memory-load pattern.

No code or evidence changes were required.

## T7 start gate — carry-over check, responsibility cut, and trap selection (2026-07-05)

Carry-over checked before choosing the T7 approach:

- From T1/T2/T5 log and retrospectives: Phase 8 GUI evidence coordinates are
  not retained ground truth. T7 must re-derive the capture coordinates from
  the final post-T6 Gallery surface and must not reuse T2/T5 coordinates as a
  contract.
- From T5 log / retrospective: T5 frames are precheck evidence only. T7 must
  re-capture selected/exclusion, lightbox, wrap/overflow, and aspect evidence
  on the final surface, and include state-confirming frames after state
  changes such as lightbox close.
- From T5 retrospective: authoritative GUI capture must be planned as a
  visible-desktop / outside-sandbox activity because the coordinate +
  `CopyFromScreen` harness is a twice-observed fragile dependency, not a
  neutral implementation detail.
- From T6 log / retrospective: T6 parity frames prove only default-view
  no-regression across Rust/C/Zig. They do not exercise selected/exclusion,
  lightbox, wrap/overflow, or aspect positive controls.
- From T2/T4/T5 carry-forward: R-2 remains open until the final Gallery
  tab-band screenshots prove the `ToggleButton.checked` background is
  visually unambiguous on the effective Gallery background.
- From T2 carry-forward: the strict `Box.aspect` positive-control question
  remains open for T7/T8; T7 should close it with frames that distinguish a
  live aspect constraint from a no-op look-alike, or record the T8 audit
  citation path explicitly.

T7 responsibility after critical re-check: T7 owns the authoritative
assistant-visible evidence package for the final post-T6 Gallery surface and
the FD-8-G(3) owner confirmation over the captured selected/exclusion and
lightbox positive controls. It does not own new Gallery semantics, host-port
changes, the T8 no-silently-deferred-surface audit, or T10 human-visible
smoke. The capture script may add robustness around window positioning and
state-confirming frames, but any UI or runtime behaviour change would require
a plan revision before implementation.

Selected traps:

| Trap | Applies? | Reason / required T7 close artifact |
|---|---:|---|
| #1 semantic migration | No | T7 changes no compiler/IR/schema/widget-kind/runtime property surface. |
| #2 missed side effects | Yes | T7 adds/updates evidence files and a capture harness whose coordinates, window sizes, foreground/topmost handling, PATH, and state transitions affect the evidence. Close with a structural side-effect enumeration and evidence inventory. |
| #3 parallel/derived data drift | No | T7 introduces no parallel runtime/source data structure. Evidence PNGs are generated artifacts with the capture script and README as their durable provenance, not a source-of-truth mirror. |
| #4 untested authored branch | No | T7 adds no diagnostic / reject / size branch. Capture-script guard failures are operational checks, not product branches requiring unit tests. |
| #5 carry-forward | Yes | Any evidence residual (especially aspect citation vs direct proof, owner G(3) status, and capture-harness constraints) must be recorded with evidence and a re-trigger criterion. |
| #6 deterministic failure | Conditional | Any recurring build/capture failure gets rerun history and disposition; no accepting "green on retry" without explaining the harness/environment cause. |
| #7 GUI positive control | Yes | T7's deliverable is GUI-render evidence. Close with launch + DPI-aware screenshots + assistant analysis + positive controls for selected/exclusion, lightbox, wrap/overflow, and aspect/citation. |

Review lane: **full independent review** because T7 is the authoritative
GUI-render evidence package. The review must check the captured frames,
analysis, positive-control reasoning, and G(3) owner-confirmation record.

## T7 end gate — assistant GUI evidence package close artifacts (2026-07-05)

T7 captured the authoritative assistant-visible GUI evidence package for the
final post-T6 Gallery surface on the Rust host. The T5/T6 frames remain
prechecks only; all T7 positive-control frames were re-captured after the
C/Zig host additions.

Coordinate provenance: T6 did not change `examples/gallery/gallery.ui`, so
the T5 header/control coordinates remained valid. T7 confirmed that invariant
against the post-T6 surface and reused those coordinates as re-validated
capture inputs rather than treating the T5 script as evidence or contract.

Capture command:

```
powershell -ExecutionPolicy Bypass -File process\milestone-3\phase-8\implementation\evidence\capture-t7-gallery.ps1
```

The command ran on a visible Windows desktop outside the filesystem sandbox.
All frames reported DPI 96 and physical window rect `(0,0)`:

| Frame | Evidence role |
|---|---|
| `evidence/t7-gallery-default-all.png` | Default view; All selected; 1200x760. |
| `evidence/t7-gallery-selected-albums.png` | Selected/exclusion transition 1; Albums selected and All cleared. |
| `evidence/t7-gallery-selected-favorites.png` | Selected/exclusion transition 2; Favorites selected and Albums cleared. |
| `evidence/t7-gallery-lightbox-open.png` | Conditional lightbox subtree present. |
| `evidence/t7-gallery-lightbox-closed.png` | Conditional lightbox subtree absent after close; state-confirming frame before narrow/scroll capture. |
| `evidence/t7-gallery-narrow-before-scroll.png` | Narrow-width reflow frame before scroll; 760x420. |
| `evidence/t7-gallery-narrow-after-scroll.png` | Narrow-width scroll-offset frame after Scroll down. |

Detailed assistant analysis is recorded in `evidence/README.md`.

**FD-8-G(3) owner confirmation**

Owner confirmed G(3) OK on 2026-07-05 after being given the selected/exclusion
and lightbox positive-control review instructions. The confirmed frame groups:

- selected/exclusion:
  `t7-gallery-default-all.png` →
  `t7-gallery-selected-albums.png` →
  `t7-gallery-selected-favorites.png`
- lightbox:
  `t7-gallery-lightbox-open.png` →
  `t7-gallery-lightbox-closed.png`

**#2 structural side-effect enumeration**

| Structure / state changed | Derived effect / disposition |
|---|---|
| Added `capture-t7-gallery.ps1`. | The script depends on visible-desktop `CopyFromScreen`, foreground/topmost positioning, and absolute coordinates re-validated against the final post-T6 Rust host. Because T6 did not change `gallery.ui`, the T5 coordinates were confirmed still valid and reused. It records state-confirming frames after selected-tab clicks and lightbox close. |
| Added seven T7 PNG frames. | They are generated evidence artifacts tied to the script and README analysis; they do not replace the `.ui` source or T5/T6 precheck frames. |
| Added `evidence/README.md`. | The README names T7 as authoritative assistant evidence, classifies T2/T5/T6 frames as prechecks, and records positive-control analysis plus known M4 residuals. |
| Updated `plan.md` T7 checkboxes. | T7 evidence, positive-control verification, and G(3) owner confirmation are closed; T8/T10 ownership remains unchanged. |
| Recorded T7 start/end gates in this log. | Review lane is full independent review; phase-end / T8 carry-forward remains separate. |

**#5 carry-forward**

- **Constraint:** Future reuse of the T7 capture harness must treat
  coordinate-based `CopyFromScreen` as a visible-desktop, final-surface
  evidence path, not a portable semantic test. **Evidence:** T2/T5 sandboxed
  capture failures and T7's successful outside-sandbox capture at `(0,0)`.
  **Re-trigger:** any future GUI evidence task that reuses or adapts
  `capture-t7-gallery.ps1`, or any Gallery layout/header coordinate change.
  **Placement:** carry-forward candidate for phase-end item 15.
- **Constraint:** T8 should treat the T7 aspect closure as Gallery-level
  visual evidence backed by already-landed Phase 2 aspect tests, not as a new
  source or spec change. **Evidence:** T7 frames show square thumbnail
  placeholders and the 4:3 lightbox placeholder; Phase 2 tests pin the exact
  aspect measure/arrange branches. **Re-trigger:** if T8's public-draft smoke
  finds `Box.aspect` unreproducible from `docs/dsl_spec.md` alone.
  **Placement:** carry-forward for T8.
- **Constraint:** T10 owner human-visible smoke remains separate from T7 and
  should use T7's frame set only as prep material. **Evidence:** AGENTS.md and
  `retrospectives.md` distinguish assistant-visible capture from owner smoke;
  T7 closed G(3), not G(5). **Re-trigger:** T10 smoke prep. **Placement:**
  carry-forward for T10.

**#6 deterministic-failure disposition**

No deterministic build or capture failure occurred during T7. The release
workspace build was already green, and the visible-desktop capture completed
on the first T7 run. The known sandboxed `CopyFromScreen` failure class from
T2/T5 was handled by planning T7 capture outside the sandbox rather than
rerolling a failed sandbox run.

**#7 GUI positive-control evidence**

- Selected/exclusion: the All → Albums → Favorites frames prove live
  controlled `ToggleButton.checked` propagation and alpha exclusion. Each
  transition shows the newly clicked tab checked and the previous tab cleared,
  closing R-2 on the final effective Gallery background.
- Lightbox: the open/closed pair proves the conditional lightbox subtree is
  present and then absent after close.
- Wrap/overflow: the 1200x760 default frame shows nine columns; the 760x420
  narrow frame reflows to five columns; after Scroll down, the visible labels
  advance from an `IMG 001`-anchored range to an `IMG 006`-anchored range.
- Aspect: the Gallery frames show the 1:1 thumbnail placeholders and 4:3
  lightbox placeholder in the final UI; the stricter "not a no-op look-alike"
  reasoning is recorded in `evidence/README.md` and backed by the Phase 2
  aspect tests (`wasamo-runtime/src/layout.rs` aspect unit tests and
  `wasamo-runtime/tests/box_layout_integration.rs`).

Known M4 residuals remain residuals, not T7 failures: real images,
thumbnail hit-testing, wheel/drag scrolling, modal focus, dynamic title/status,
and runtime DPI-awareness.

## T7 independent review (2026-07-05)

Reviewer: Jason subagent (`019f3024-a9e8-7ed2-9b75-a386f87e4372`).

Result: **no findings**.

Review scope: T7-only working tree changes — `plan.md`, `log.md`,
`evidence/README.md`, `evidence/capture-t7-gallery.ps1`, and the seven
`t7-gallery-*.png` frames.

Reviewer confirmation:

- GUI positive controls are recorded and visually match the PNGs:
  selected/exclusion, lightbox present/absent, narrow reflow/scroll, and
  aspect/citation coverage.
- Structural side effects are enumerated for the capture script, generated
  PNGs, README, plan, and log effects.
- Carry-forward is explicit for capture-harness fragility, T8 aspect/spec
  audit handling, and T10 owner-smoke separation.
- G(3) owner confirmation is recorded with the confirmed frame groups.
- T7 responsibility boundaries are clear and do not absorb T8/T10 ownership.

Reviewer-noted residual: aspect evidence is partly visual and partly by
citation to already-landed Phase 2 tests, not a fresh no-aspect comparison
screenshot. This is consistent with the T7 plan wording; T8 should still
verify that the public draft makes the aspect surface reproducible without
relying on private implementation memory.

## T7 Claude review disposition (2026-07-05)

Reviewer: Claude review packet supplied by the owner.

Findings disposition:

1. **Coordinate provenance wording overstated "re-derive".** Accepted. T7
   reused the same absolute coordinates as T5 after confirming that T6 did not
   change `gallery.ui`; the frames are correct, but the end-gate wording needed
   to say "re-validated and reused" rather than imply fresh derivation.
   Remediation: added the coordinate-provenance note above and corrected the
   #2 structural side-effect row.
2. **Aspect remains the weakest positive-control leg because it is visual +
   citation-backed, not a fresh no-aspect comparison screenshot.** Accepted as
   a plan-sanctioned residual, not a T7 blocker. Direct T7 closure would require
   an additional comparison fixture/frame pair, but that would add a special
   evidence surface beyond the final Gallery capture. T7 keeps the current
   disposition: Gallery visual evidence plus Phase 2 aspect tests, with T8
   required to verify `Box.aspect` reproducibility during the external-reader
   smoke.

Owner decision surface for Finding 2: either (A) accept the current
plan-sanctioned T8 carry-forward, or (B) reopen T7 to add a dedicated
aspect-vs-no-aspect comparison capture. Current recorded disposition follows
(A); choose (B) only if T7 must be self-contained for aspect without relying on
the already-landed Phase 2 tests and T8 public-draft smoke.
