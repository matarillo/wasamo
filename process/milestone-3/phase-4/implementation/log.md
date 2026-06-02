## Decisions log

- **T4 — unbounded-scroll-axis fixture disposition (2026-05-25).** ADR
  Phase 4 verification closure item 4's last sub-bullet permits
  downgrading the unbounded-scroll-axis runtime fixture to pure-logic
  coverage when no ergonomic `.ui` / IR-level fixture can synthesise
  the case. Every Phase 4 widget catalog parent (VStack / HStack /
  Box / WrapPanel / window root) passes a finite scroll-axis cell to
  its ScrollView child at arrange time — VStack / HStack arrange-time
  `h` derives from finite parent allocation (Fixed / Fill against
  finite window, Shrink against finite content); Box / WrapPanel
  arrange children with their resolved finite rect; the window root
  passes `window_h`. There is therefore no `.ui` that reaches
  `arrange_scroll_view` with `h = f32::INFINITY`. The pure-logic
  `scroll_view_unbounded_scroll_axis_parent_is_runtime_error` test in
  `wasamo-runtime::layout::tests` (T2) pins the
  `LayoutError::ScrollViewUnboundedAxis` branch; the integration-test
  bullet is intentionally left unchecked above so the disposition is
  visible at-a-glance. Revisit if a future phase introduces a parent
  layout that legitimately passes unbounded scroll-axis input
  downstream (none anticipated through Phase 8).

- **T5/T6 split for owner-manual GUI smoke (2026-05-25).** The
  original Task list bundled owner-manual GUI smoke as the last
  `[ ]` of T5 and treated phase-end mechanical close as T6. T5 was
  closed by the assistant after the `.ui` + Build + Start-Process
  bullets had landed, which left the smoke bullet with **no step
  that actually owns its execution** — the smoke could only happen
  after owner review of T5, but T5 was being merged. The original
  T6 (phase-end / Moment 2 re-sync) made the smoke verification
  implicit via [retrospectives.md item 17](../../../procedures/retrospectives.md#phase-end-固有-merge--main),
  which would have meant interleaving spec / plan marker flips with
  smoke verification in a single step. If smoke had failed mid-T6,
  the phase branch would carry half-flipped spec markers while a
  fix step is opened — an avoidable rollback surface.

  Resolution: split the original Task list into three steps. T5
  retains the assistant-automated bullets; **new T6** is a dedicated
  owner-manual GUI smoke step that absorbs any fix iterations
  inside its own branch; **T7 (= former T6)** runs the mechanical
  phase-close gates only once visible correctness is owner-accepted.
  Smoke gate is now explicit in the progress file rather than
  inherited from `retrospectives.md` item 17. ADR (`process/
  m3-phase-4-scroll-view.md`) numbering is not affected — the ADR
  refers to verification closure **evidence item 5**, which now
  maps onto T5 + T6 jointly; m3-plan.md Phase 4 row does not list
  T-numbers and needs no edit.

  Phase 2 / Phase 3 precedent was to bundle smoke into the
  phase-end step. Phase 4 deviates because the intermediate Visual
  + `InsetClip{0,0,0,0}` + `Visual.Offset = (0, -applied_y, 0)`
  shift + `sync_visuals` `parent_abs_offset` shift compose the
  largest Visual-layer change of M3, raising the prior on
  pixel-level regressions that integration tests cannot catch (e.g.
  clip sharpness, repaint ordering, peripheral wiring). The split
  is local to Phase 4; not a project-wide convention change.

- **T6 smoke failure mode A disposition (2026-05-25).** First owner-
  manual smoke pass on the T5 release artifacts (`target/release/
  gallery-rust.exe`, built 2026-05-25 19:49) reported (1) the ScrollView
  region under the Button row drew completely empty at `scroll_y = 0`,
  and (2) pressing "Scroll down (+100)" five times produced **no
  visible change** in the rendered window. Root cause: `gallery.ui`'s
  component root is a VStack with default `width: Fill, height:
  Shrink`; `layout::measure_vstack` for `Shrink` height excludes Fill
  children's desired_h (the standard Fill-collapses-under-Shrink-parent
  convention, intentionally pinned by
  `layout::tests::degenerate_fill_in_shrink_parent_clamps_to_zero`),
  so the root VStack resolved to `desired_h ≈ 312`
  (= Phase 3 WrapPanel + Button×2 + spacing/padding) regardless of
  window height. `arrange_vstack`'s `remaining` then clamped to `0`
  and the Fill ScrollView child received `child_h = 0`, producing an
  outer Visual sized `(w, 0)` whose `InsetClip{0,0,0,0}` auto-tracked
  to a zero-height clip rect → content never visible. The writer chain
  was equally inert by composition: `max_offset = max(0, content_h −
  0) = content_h` so `applied_offset_y = clamp(scroll_y, 0, content_h)`
  did update, but with viewport height 0 the resulting
  `Visual.Offset = (0, -applied_y, 0)` had nothing on-screen to move.

  T4's `scroll_view_layout_integration.rs` integration fixture
  (`FIXTURE_SRC`) roots a ScrollView directly under the `inherits
  Window` component (`width: Fill, height: Fill` from
  `WidgetNode::scroll_view`), bypassing the VStack-root path that
  production `.ui` (counter, bool-demo, gallery) all use. The T4
  Decisions log note "T4 fixture WrapPanel substitution vs ADR primary
  VStack (2026-05-25, commit `57f2366`)" already flagged the layout
  divergence between fixture parent and ADR's primary VStack-rooted
  envelope; this T6 finding is the visible-correctness consequence of
  that uncovered path.

  Disposition: in-scope T6 fix, **no normative spec change**.

  - **Implementation fix (a):** introduce a dedicated
    `WidgetNode::run_layout_as_window_root` that overrides the root
    LayoutNode's `width`/`height` to `SizeConstraint::Fill` before
    delegating to `layout::run_layout`; route `window.rs`'s `WM_SIZE`
    handler and `set_root` initial layout to call it. The plain
    `WidgetNode::run_layout` retains its current semantics (so
    existing mock-free integration tests like
    `wrap_panel_layout_integration.rs`, which drive `WidgetNode`s
    directly to exercise declared sizing constraints, continue to
    pass), and the pure-logic `layout::run_layout` —
    `degenerate_fill_in_shrink_parent_clamps_to_zero` and the broader
    Shrink/Fill convention — also stays untouched. The override
    formalises the implicit "Window client rect determines the root
    viewport" contract that Phase 2 / Phase 3 implicitly relied on
    but that no `.ui` had previously exercised with a Fill child at
    the root container.
  - **Fixture-visibility adjustment (b):** `examples/gallery/gallery.
    ui` bumps the ScrollView inner WrapPanel's `item-cross-size` from
    `64` to `128`. With viewport height ≈ `window_h − 312` and Box ×
    32, `item-cross-size: 64` produces `content_h` smaller than the
    viewport across 800×600 – 1280×900 windows (max_offset = 0,
    +100/-100 motion invisible even after fix (a)). Bumping to 128
    yields `content_h > viewport_h` across the same range so scroll
    motion is visually observable. ADR's "Box × 30–40" range and
    Phase 3 standalone WrapPanel slice both stay untouched.
  - **Pure-logic pinning test (c):** new `#[test]` in
    `wasamo-runtime/src/layout.rs::tests` re-states
    `degenerate_fill_in_shrink_parent_clamps_to_zero`'s outcome for a
    gallery-shaped VStack root (mixed Shrink-height + Fixed-height +
    Fill-height children including a ScrollView) and asserts that
    pre-setting the same root to `Fill` height before `run_layout`
    flips the Fill child's allocated height to non-zero. Captures
    both the basal trap and the override behaviour at the layer where
    `WidgetNode::run_layout`'s policy is enacted.
  - **Runtime integration test (d):** new `#[test]` in
    `wasamo-runtime/tests/scroll_view_layout_integration.rs` that
    lowers a VStack-rooted gallery-shaped `.ui` (Button +
    ScrollView) through `wasamoc` and `build_widget_tree`, then
    drives `WidgetNode::run_layout_as_window_root(200.0, 200.0)`
    (the same WinRT-bound entry point `window.rs` uses on `WM_SIZE`).
    Asserts the ScrollView outer Visual height > 0 at
    `scroll_y = 0` (the regression gate for fix (a)) and that the
    intermediate content Visual's `Y` offset is negative at
    `scroll_y = 100` (writer chain end-to-end on the production path,
    not just the ScrollView-root fixture path). Reuses the existing
    skip-guard discipline (fail on GitHub Actions, skip on
    `0x80070005` locally).

  T7 carry-over: `architecture.md §6` (general layout, not §6.5
  ScrollView) candidate addition documenting **"the window-root
  WidgetNode is sized to the client rect regardless of its declared
  width/height constraints"** as a single-sentence runtime-boundary
  invariant. The latent layout question "non-root VStack with mixed
  Shrink + Fill children" stays out of Phase 4 scope: it is the same
  collapse convention `degenerate_fill_in_shrink_parent_clamps_to_zero`
  pins, surfaces only when a future widget catalog ships a
  non-window-root container that wants Fill-child sizing, and is
  classified `carry-forward` to the next phase pre-doc input as a
  design-topic candidate rather than a Phase 4 bug.

  No screenshot automation is required for T6 close; owner-side
  visible smoke + the new integration test together discharge
  evidence item 5's visible-correctness half.

- **T7 Moment 2 dispositions for follow-up bullets (2026-05-25).**
  Three owner judgments were requested at T7 open and resolved before
  any spec or carry-forward editing:

  - **Q1 — dsl_spec.md §4.9 Box example notation.** The
    `;`-separated single-line `Box { aspect: <r>; fill: <c>; ... }`
    examples (lines 501 / 507 / 508 / 752) are switched to the
    parser-accepted multi-line form, but the revision history wording
    is **"parser-accepted examples; semicolon member separator left
    as post-Phase-4 open question"** and a short adjacent note
    explicitly preserves the post-Phase-4 grammar option of
    admitting `;` as an optional member separator. The owner-confirmed
    framing rejects a strict-A (purely "newline confirmed as the
    member separator") reading because it would foreclose a future
    grammar extension the owner wants to keep open. Document version
    bumps to `1.2`.
  - **Q2 — Window title wiring disposition.** The
    `MainWindowTitle = "Wasamo"` vs `.ui` `title: "Gallery"`
    divergence is registered under §Out-of-phase residuals as **R1**
    with a fixed resolution condition: **"the runtime/ABI host path
    applies the component-level `title` to the native window"**, not
    "title attribute declared unsupported". The owner-confirmed
    framing classifies this as an **M3 residual, not an M4
    theming/chrome handoff**. Gate structure: M3-Phase 5 pre-doc
    input distillation assigns the owning M3 phase; implementation
    must land **no later than M3-Phase 8 Gallery E2E close**; Phase
    6 (ZStack + conditional rendering) is a natural candidate for
    absorbing the small host/window-metadata wiring task because
    the lightbox UX naturally exercises window-level metadata. The
    pre-doc carry-forward bullet records (d) this M3-Phase 5 pre-doc
    obligation.
  - **Q3 — T7 retrospective scope.** T7's original last bullet
    "Phase-end retrospective recorded under `docs/notes/m3-phase-4/`"
    conflated two distinct retrospectives in
    [retrospectives.md](../../../procedures/retrospectives.md) §進行手順:
    step-end retro (items 1-11; step → phase merge gate) and
    phase-end retro (items 12-18; phase → main merge gate). The
    bullet is split into two: T7 owns the **step-end** retrospective
    only (`t7-step-end-retrospective.md`); the phase-end
    retrospective (`phase-end-retrospective.md`) is owned by a
    separate commit on `feat/m3-phase-4` after T7 merges in and is
    the precondition for the phase → main merge gate. This avoids
    the reviewer hazard "T7 close has an unflipped phase-end
    retrospective bullet" — the two bullets now name distinct
    retrospectives at distinct gates, so the `[ ]` on the phase-end
    line is the correct state at T7 step-close.

---

## CI / verification log

- **T6 close (2026-05-25).** Local clean rebuild proxy for the T6 fix bundle (commit `ed78d6c`) on the
  `feat/m3-phase-4-t6` branch: `cargo fmt --all -- --check` zero
  exit; `cargo build --release --workspace` green; `cargo build
  --workspace` green; `cargo test --workspace` green — `wasamo-runtime`
  lib **258 passed** (T5 baseline 257 + the new pure-logic pinning
  test `shrink_vstack_root_with_fill_scroll_view_child_collapses`);
  `scroll_view_layout_integration` **3 passed** (T5 baseline 2 + the
  new mock-free runtime integration test
  `scroll_path_vstack_root_fixture_pins_window_root_fill_override`
  driving `run_layout_as_window_root`); `wrap_panel_layout_integration`
  back to **2 passed** after the `run_layout` /
  `run_layout_as_window_root` split insulated it from the window-root
  override. Assistant-side `Start-Process` on rebuilt
  `target/release/gallery-rust.exe` produced PID 3916, MainWindowTitle
  `"Wasamo"`, HasExited = `$false` after 3 s, `Stop-Process -Force`
  clean exit. Owner-manual GUI smoke (2026-05-25, re-run on the post-
  `ed78d6c` binary) returned green on all four observation points
  (viewport clip sharp, +100/-100 Buttons move content by ±100 px,
  clipped regions stay within the ScrollView outer rect, off-viewport
  thumbnails enter as `scroll_y` progresses) plus the +1 reference
  observation (scrollbar non-display, expected under Phase 4 scope);
  window close (Alt+F4 / ×) crash-free; smoke evidence committed at
  `docs/references/m3-phase-4/t6-gallery-smoke-scroll-y-{0,100,800,
  back-to-0}.png`. GitHub Actions CI green confirmation is **T7**
  phase-end gate (`workflow_dispatch`); local clean rebuild is the
  T6-step-level proxy.

- **T7 close (2026-05-25).** Local clean rebuild at commit `42ed208`
  on `feat/m3-phase-4-t7`, after the Moment 2 spec sync, m3-plan row
  flip, and M3-Phase 5 pre-doc carry-forward had landed:
  `cargo fmt --all -- --check`
  zero exit; `cargo clean` removed 3650 files / 1.0 GiB; `cargo build
  --release --workspace` 43.19 s green; `cargo build --workspace`
  (debug) 35.56 s green; `cargo test --workspace` failure 0 —
  `wasamo-runtime` lib **258 passed** (T6 baseline で不変),
  `scroll_view_layout_integration` **3 passed** (T6 baseline で不変),
  `wrap_panel_layout_integration` **2 passed** (T6 baseline で不変),
  `wasamoc` lib **227 passed**, 他 crate 全 green. GitHub Actions
  `workflow_dispatch` run `26404665377` on `feat/m3-phase-4-t7`
  completed `success` (2026-05-25 14:08 → 14:10 UTC,
  <https://github.com/matarillo/wasamo/actions/runs/26404665377>),
  discharging the **and on CI** half of T7's `cargo build --release
  --workspace` / `cargo test --workspace` / Windows-only integration
  evidence bullets. T7's mechanical close commits are: `4b41a4c`
  (T7 Task list revise + Q1/Q2/Q3 dispositions); `59729b1`
  (Moment 2 spec sync — architecture.md + dsl_spec.md + m3-plan.md);
  `42ed208` (M3-Phase 5 pre-doc carry-forward inputs); plus the
  progress-file finalize commit (this commit) and the T7 step-end
  retrospective commit. The separate phase-end retrospective (covering
  retrospectives.md items 12-18, owned by the phase → main merge gate)
  is **not** part of T7's commit set; it will land as a separate commit
  on `feat/m3-phase-4` after T7 merges in.

- **Phase-end close (2026-05-25).** Local clean rebuild on
  `feat/m3-phase-4` after the T7 merge and phase-end doc updates:
  `cargo fmt --all -- --check` green; `cargo clean` removed 2547
  files / 915.8 MiB; `cargo build --release --workspace` 44.17 s
  green; `cargo build --workspace` 36.21 s green; `cargo test
  --workspace` failure 0 — `wasamo-runtime` lib **258 passed**,
  `scroll_view_layout_integration` **3 passed**,
  `wrap_panel_layout_integration` **2 passed**, `wasamoc` lib
  **227 passed**, all other workspace tests green. GitHub Actions
  `workflow_dispatch` evidence on `feat/m3-phase-4`:
  run `26406065405` completed success
  (<https://github.com/matarillo/wasamo/actions/runs/26406065405>);
  `cargo build` job 2m2s, all build / test / smoke steps green.
  Annotations only: Node.js 20 deprecation for `mlugg/setup-zig@v2`
  and the `windows-latest` redirect notice.
