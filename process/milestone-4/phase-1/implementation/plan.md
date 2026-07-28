## Task list

M4-Phase 1 ships no author-facing surface. The work is a pure-logic
conversion type (T2), a behaviour-identical refactor that moves the last
construction-time Visual writes into the sync pass (T3), per-window
scale state (T4), the conversion seams (T5), the rasterization surface
and its re-rasterization walk (T6), the `WM_DPICHANGED` handler (T7),
integration evidence (T8), the awareness declaration that turns all of
it on (T9), and the evidence and close gates (T10–T12) — preceded by a
pre-implementation spike (T1).

**The order is deliberate and is argued in
[preamble.md §The sequencing thesis](./preamble.md#the-sequencing-thesis-build-the-machinery-then-declare):
the declaration lands last.** Because the conversion machinery is
unconditional and an unaware process reports 96 DPI, T2–T8 land into a
world where every scale factor is exactly 1 and every conversion is the
identity — so every intermediate commit is correct and visually green,
including on the 125% development machine. T1 measured the premise
rather than inheriting it: an undeclared process is told 96 DPI on that
machine. T8 drives `s ≠ 1` synthetically so the ordering does not defer
all scaled-path risk to the end.

**What "green" is worth here changed at T1.** The clause above is a
statement about the *world* T2–T8 land into — every conversion really is
an identity — not about the suite's power to detect a wrong one. Those
are separate claims and the plan originally ran them together. The next
paragraph separates them.

**A green suite is not evidence of correctness in T3–T8** (owner-agreed
2026-07-28, on T1 finding F-4). Every existing layout integration test
drives `WidgetNode`s directly and never through a window, so no test
routes a coordinate through a window's scale; the whole conversion
machinery *plus* the awareness declaration was measured green at 125%,
indistinguishable from baseline. `cargo test --workspace` therefore stays
in every end gate as a **regression check** — it must not go red — but it
is not counted as evidence that a conversion is right. What counts per
task: T2 its own unit tests **shown to fire**, T3 the rendered gallery
frame, T4 the ordering probe's three measured window states, T5 the
call-site audit table, T6 the rendered output, T7 the structural
side-effect enumeration, T8 its own scale-driving assertions. **T4 was
added to this list at T4** — it was omitted at planning time on the
assumption that a task inert until T9 has nothing to show, and what it
turned out to have is the one artifact that separates a correct
create-then-correct path from three wrong ones.

**T2's entry carried a missing condition, supplied at T2 (finding
F-11).** Every other entry above is an artifact checkable against ground
truth; "its own unit tests" is a green/red claim of exactly the kind the
paragraph above disqualifies. The exception for pure logic is right, but
it holds only *once the tests are shown to fire*: T2 measured that eleven
green tests said nothing beyond "eleven tests exist and passed" until
seven deliberately wrong implementations showed which failure each one
catches. The mutation table in [log.md](./log.md) is T2's artifact; the
green suite is not.

Default to **one commit per task-list item** per
[AGENTS.md §Commit rules](../../../../AGENTS.md). The known exception
this phase:

- **T5** — **narrowed at T1.** The exception was written on the
  assumption that `run_layout_as_window_root`, `sync_visuals` and the
  hit-test entry points all change signature together. They do not:
  the carrier decision leaves the layout and sync signatures untouched,
  and only `hit_test_click` / `update_hover` change (their `i32`
  physical coordinates become `f32` DIP). The exception survives at that
  reduced scope — those two signatures and their 7 test call sites in 4
  files do not build in intermediate states, so they land in one
  buildable commit — but it no longer covers the seam work as a whole,
  and the rest of T5 may be split if it reads better that way.

If implementation reveals an item should split or reorder, revise this
list so it stays an accurate record rather than a frozen prediction —
plan changes mid-implementation are normal and expected.
**Sub-task lists below are planning-time hypotheses**, not frozen
contracts; T1 may re-cut them against the source, and any task may
revise its own sub-list as work surfaces.

Each task runs the implementation gates at **start** (record the trap
selection, the reasons for non-applicable traps, and the review lane in
[log.md](./log.md) *before* choosing an approach) and at **close** (the
auditable artifacts), per
[implementation-gates.md](../../../procedures/implementation-gates.md).

---

### T0 — Moment 1 closure + implementation docs open

Opens execution after ADR acceptance. Implementation (T1) begins only
after T0 closes.

- [x] ADR set `Status: Accepted` — preamble + DD-001 through DD-004
      flipped 2026-07-28 (commit `09ff0d4`).
- [x] Moment 1 spec sync landed: [architecture.md §12](../../../../docs/architecture.md#coordinate-spaces)
      normative coordinate-space section + §7.3 ramp unit + open-question
      resolution (`f15eef0`); [dsl_spec.md](../../../../docs/dsl_spec.md)
      §1 units definition and the dimension-site replacements, v1.16
      (`7beac4e`); [abi_spec.md](../../../../docs/abi_spec.md) §4.1 / §4.2
      (`1769200`); [layout-engine.md §3.1](../../../../docs/notes/layout-engine.md)
      answered + [M4 plan](../../plan.md) Phase 1 row (`80c3fa4`).
- [x] `verification-environments.md` Observation 4 confirmed **held for
      Moment 2**, with the reason recorded (DD-004 §Note updates): the
      phase falsifies its premise, and the corrected capture coordinates
      can only be derived against the running surface.
- [x] This `preamble.md` + `plan.md` + skeleton [log.md](./log.md) /
      [handoff.md](./handoff.md) owner-reviewed and landed (`dedd327`,
      merged to the phase branch at `80d79c4`); front-matter `status`
      flipped `draft` → `active` 2026-07-28 on owner authorisation to
      open T1.

**Start gate:** none (doc-only). **End gate:** the implementation docs
are on the branch and the Moment 1 commit set is complete; T1 may open.

---

### T1 — Pre-implementation spike: carrier shape, signature ripple, sequencing

**No production code lands.** The compiler-verification edits are
throwaway and are reverted before T1 closes; T1's landing artifacts are
recorded decisions in [log.md](./log.md) plus any revision of this plan.
This is a risk-mitigation spike for R-2 / R-5 and a confirmation of the
sequencing thesis — not the first slice of the work.

**Closed 2026-07-28.** Every item below is discharged and its artifact is
in [log.md](./log.md) §T1. The spike produced five findings (F-1 … F-5),
two decisions (the carrier shape, the walk shape), two confirmations
(the sequencing thesis, the declaration site), and revisions to T4, T5,
T6, T9 and T12 below. All throwaway edits were reverted; no production
code lands on the T1 commit.

- [x] **Read every landing file end-to-end** (not grep-sample), per the
      [spike discipline](../../../procedures/implementation-gates.md):
      [`wasamo-runtime/src/window.rs`](../../../../wasamo-runtime/src/window.rs)
      (`create`, `create_hwnd`, `set_root`, `wnd_proc`'s six message
      arms, `WindowState`'s fields),
      [`text.rs`](../../../../wasamo-runtime/src/text.rs) (`draw_text`,
      `create_text_layout`, `measure`, `TypographyStyle::size_sp`),
      [`widget.rs`](../../../../wasamo-runtime/src/widget.rs)
      (`sync_visuals`, `run_layout_as_window_root`, `visual_rect`,
      `hit_test_click_inner`, `update_hover_inner`, every `draw_text`
      call site, the Button / ToggleButton label construction and
      label-update paths, the `InsetClip` installs),
      [`runtime.rs`](../../../../wasamo-runtime/src/runtime.rs) (`init`
      and its one-shot guard),
      [`abi.rs`](../../../../wasamo-runtime/src/abi.rs)
      (`wasamo_window_create`, `set_last_error`), and
      `wasamo-runtime/Cargo.toml`. Record the per-file touch-points.
      **Two files were added to the list by the read**: `emit.rs`
      (a second production `GetClientRect` → layout path) and `lib.rs`
      (the Rust-native `window_create` / `window_set_root`).
- [x] **Verify DD-002's 13-row audit table against the source** and
      record any row whose file / function has moved since ADR drafting,
      plus any coordinate-carrying path the table does not name. The
      table is the contract; a discrepancy is a finding to record, not a
      silent correction. **Three findings: F-1** (row 2 covers a second
      site, `emit::flush_layout`), **F-2** (row 12 names Box, which
      installs no clip; the third clip site is ZStack), **F-3** (the six
      `WindowState` callback slots carry coordinates with no stated
      unit).
- [x] **Decide and record the `DipScale` carrier and threading shape**
      (risk R-5). Both candidate shapes were built, compiled and
      reverted; the breakage sets are the compiler's, not an estimate.
      **Decided: authoritative on `WindowState`, cached on each
      `WidgetNode`, written by one walk** — 7 test call sites in 4 files,
      against 28 in 12 files for parameter threading, and the only shape
      with an answer on the `set_property` re-rasterization path.
      The pointer's DIP type is `f32`. Existing tests keep their layout
      signatures; the 7 broken sites are the `hit_test_click` literals.
- [x] **Decide and record where the re-rasterization walk lives** and
      what it re-creates (`WidgetData::Text { content, style }`,
      `ButtonData` / `ToggleButton` label state), confirming DD-002's
      claim that no new retained state is required. **Decided:**
      `WidgetNode::apply_scale_recursive`, called from `set_root` and
      from the `WM_DPICHANGED` handler. DD-002's no-new-state claim holds
      for the re-rasterization itself.
- [x] **Confirm or revise the sequencing thesis.** Check that T2 → T8
      each leave the workspace buildable, the test suite green, and the
      rendered output unchanged at the development machine's 125%. If
      any intermediate state cannot hold that, revise this task list
      before T2 opens. **Confirmed**, premise measured (an undeclared
      process is told 96 DPI on the 125% machine). No task split
      revised. **F-4 qualifies the comfort**: the suite stays green with
      the machinery *and* the declaration in place, because no existing
      test routes a coordinate through a window's scale.
- [x] **Confirm the awareness-declaration site** — that `runtime::init()`
      can declare before `CreateDispatcherQueueController`, and that the
      existing `RUNTIME.get().is_some()` early return does not cause a
      second `wasamo_init` to re-declare. **Confirmed both**, with the
      wording sharpened to "the first **OS-touching** act, below the
      one-shot guard".
- [x] **Sharpen [preamble.md §Technical risks](./preamble.md#technical-risks-planning-time-recon-t1-sharpens)**
      against the source (pin file / line hotspots), and record the
      **T5 and T6 gate selections** with reasons for non-applicable
      traps before T5 opens.

**Start gate:** read this plan, the ADR set, and the spike-discipline
gate; record T1's own gate selection, review lane, and planned proof
obligations in [log.md](./log.md) before the throwaway edits.
**End gate (spike-specific):** every open point is **assigned to a
downstream task and its scope is seen** — not "no surprises expected";
the carrier shape, the signature-breakage list, the audit-table
verification, and the sequencing confirmation are recorded in
[log.md](./log.md); all throwaway edits are reverted (no production code
on the T1 commit).

---

### T2 — `DipScale` conversion type + pure-logic unit tests

The phase's only pure-logic surface, and the one place the rounding
contract lives. No Win32 or WinRT dependency, so it is unit-testable
under [AGENTS.md §Testing rules](../../../../AGENTS.md) with no mocking
question. Lands with **no call sites** — nothing consumes it until T4.

**Closed 2026-07-28.** Landed as
[`dip_scale.rs`](../../../../wasamo-runtime/src/dip_scale.rs) with 11
unit tests and one call site — `mod dip_scale;` — which is the
declaration without which the module would not compile. The four items
below landed as **one commit**, not four: items 1 and 3 each introduce an
authored branch whose test lives in item 4, so a per-item split would
have landed two untested branches in intermediate commits. Artifacts in
[log.md](./log.md) §T2.

- [x] `DipScale` value type carrying `s`, constructed from a DPI value.
      Retains only the factor, not the originating DPI. `Default` is
      hand-written (a derived one would produce a zero factor), and a
      zero DPI falls back to the identity rather than dividing by zero.
- [x] `to_physical(dip) -> f32`, `to_dip(px) -> f32`, and the rectangle
      form (position and extent converted separately). The outbound
      position form is `relative_offset_to_physical(abs, parent_abs)`,
      whose signature is what makes convert-once-on-the-difference the
      natural call; inbound is a single `pair_to_dip`, because positions
      and extents alike are one componentwise division there.
- [x] The `ceil` surface-allocation rule as a named operation, so T6
      calls it rather than re-deriving it. Returns a `(u32, u32)` pixel
      count rather than a length, so a later cast cannot truncate it
      back, and each axis is floored at one pixel (preserving
      `draw_text`'s existing `max(1.0)`).
- [x] Unit tests, discharging **verification item 1**: conversion at
      125% / 150% / 200%; position-and-extent consistency; round-trip
      error and rounding *direction*; the `ceil` allocation contract;
      the convert-once-on-the-difference rule (that subtracting in DIP
      then multiplying differs from multiplying then subtracting, and
      that the type's API makes the former the natural call). The
      witnesses for the two `f32` claims were found by brute-force
      search, and **each test was shown to fail against a deliberately
      wrong implementation** — seven mutations, tabulated in
      [log.md](./log.md). A green pure-logic test is only informative
      once it is known to fire.

**Start gate:** trap #4 applies (new arithmetic branches ship with tests
that fire them); #5 was added at the start gate, because the rounding
contract is enforced by the API shape and a later task can defeat it by
hand-rolling the arithmetic. **End gate:** tests named per contract;
`cargo test` green; no production call site introduced. **All met.**

---

### T3 — Button / ToggleButton label Visual writes move into the sync pass

**Behaviour-identical refactor at scale 1**, landing ahead of the scale
work so a regression in shipped rendering code is bisectable
independently of the DPI change (DD-002 risk note; preamble obligation
2). The Button label's `SetOffset(PAD_H, PAD_V)` and `SetSize(lw, lh)`
are written at construction today, where no scale exists; after this
task every Composition geometry write in the runtime happens in exactly
one pass — which is what makes T5's audit *complete* rather than
approximately complete. **T1 verified the premise**: the production
Composition geometry writes are exactly the two construction-time label
writes (`widget.rs:813` / `818`), the label-update pair (`1035` /
`1040`), the `sync_visuals` node pair (`1749` / `1754`), the ScrollView
intermediate pair (`1776` / `1781`), the root's
`SetRelativeSizeAdjustment` (`window.rs:64`) and the three zero-inset
clips. After this task the first four collapse into one pass and no
other write exists to be missed.

**Closed 2026-07-28.** The code change landed as **one commit** — the
bisectability requirement (preamble obligation 2) overrides the
one-commit-per-item default, and the write sites and their receiving arm
do not render correctly in intermediate states. Artifacts in
[log.md](./log.md) §T3: the call-site audit, the 13-row side-effect
enumeration, the measurement-source decision, a three-mutation table
showing the rendered frame fires, and six pixel-identical before/after
frame pairs in [evidence/](./evidence/). The task produced findings
F-17 … F-22 and revisions to T5, T6, T7, T9, T10, T12 and the preamble.

- [x] Move the label offset / size writes out of Button construction and
      the label-update path into `sync_visuals`.
- [x] **Decide where the sync pass gets the label's measured size.**
      Named here rather than discovered at the write (found while
      preparing T3's handoff, after the T2 merge): the two relocated
      writes use `(lw, lh)` from `TextRenderer::measure`, which is in
      scope at construction and at the label update but **not** in
      `sync_visuals` — the node retains `label_text` / `label_style` but
      not the measured extent, and `sync_visuals` takes no renderer.
      **Decided: `ButtonData` retains the measured extent as
      `label_size`.** Re-measuring in the sync pass would put a fallible
      DirectWrite call in a pass that makes none, and would make the pass
      a second producer of a number the node already commits to through
      `SizeConstraint::Fixed`. Deriving it from `computed.size` minus the
      padding is not behaviour-identical — measured, as mutation N3: a
      Grid-stretched button's label smears across the whole cell. The new
      field is the reason **trap #3 stops being phase-wide
      non-applicable** (F-22).
- [x] Note that the label Visual is **not** a child `WidgetNode` — it
      lives in `ButtonData.label_visual` — so the sync pass reaches it
      through a `WidgetData::Button(btn) | ToggleButton(btn)` arm, in the
      same shape as the existing ScrollView intermediate arm, not through
      the `children` / `computed.children` zip.
- [x] `PAD_H` / `PAD_V` are declared **twice** today (in `button_family`
      and again inside `update_button_label`). The sync pass would be a
      third site; hoist them to one constant instead. Same
      rule-in-two-places class as T2 finding F-14. **Landed as
      module-level `BUTTON_PAD_H` / `BUTTON_PAD_V`.**
- [x] Cover `ToggleButton`'s label path in the same move (it reuses
      Button's leaf measure / arrange and carries the same label).
      **Mutation N2 is the evidence**: dropping `ToggleButton` from the
      arm removes exactly the three gallery tab labels.
- [x] Confirm the node's sizing still derives from the same measurement
      — the move is a write-site relocation, not a sizing change.
      `SizeConstraint::Fixed` is per axis, so this is two constraints,
      not one: `button_family` sets `Fixed(lw + PAD_H * 2.0)` /
      `Fixed(lh + PAD_V * 2.0)` and `update_button_label` re-derives the
      same pair.
- [x] Regression gate: existing Button / ToggleButton integration
      fixtures and the gallery render unchanged. Per the note above the
      **rendered frame is the gate**; the fixtures are a regression
      check, and T1 measured that they do not react to a geometry-write
      relocation the way the frame does. **T3's N1 mutation confirms it
      from the other side**: the suite stays green with every button
      label invisible.

**Start gate:** trap #2 (the write moves between passes — enumerate what
depended on it landing at construction time); #1, #3, #4, #5, #6 and #7
were added at the start gate. **End gate:** the side-effect enumeration;
fixtures green; a rendered gallery frame matching the pre-change frame.
**All met.** Review lane raised to full independent review (F-17).

---

### T4 — Per-window scale on `WindowState` + initial acquisition + DIP window sizing

Additive per-window state. Inert until T9 — with the process still
unaware, `GetDpiForWindow` returns 96, the scale is 1, and the
`SetWindowPos` correction is a no-op.

**Closed 2026-07-28.** Landed as **two code commits**: the rounding rule
in [`dip_scale.rs`](../../../../wasamo-runtime/src/dip_scale.rs) with its
tests and no caller, then the `Win32_UI_HiDpi` feature, the
`WindowState` field, its seeding and the correction together — a commit
adding the field without its consumer emits a never-read-field warning,
and the feature is the prerequisite for the `GetDpiForWindow` call in the
same commit. A third, small commit folded the **measured** nested-message
set into the placement comment after the probe ran. Artifacts in
[log.md](./log.md) §T4: the two decisions with their rejected candidates,
a 10-row call-site audit that closes DD-002 row 13, a 13-row side-effect
enumeration whose message set is measured rather than quoted, a
four-mutation table for the rounding rule, and a throwaway probe that
prints the creation ordering and measures **three** window states. The
task produced findings F-25 … F-31 and revisions to §T5, §T7, §T8, §T9,
§T10, §T12 and the preamble.

**Inertness holds, and the probe measured its shape more precisely than
the sentence above.** At `s = 1` the correction is the exact identity —
window `800 × 600` / client `784 × 561` before and after — and
`SetWindowPos` dispatches **no `WM_SIZE` at all**, because the size does
not change. So T4's placement decision is not merely identity-*valued*
before T9; its failure mode is **unreachable**. That is F-4's lesson
applied to an ordering rather than to an arithmetic, and it is why the
placement below is argued structurally (see F-31).

- [x] `DipScale` field on `WindowState`, seeded from `GetDpiForWindow`
      immediately after `CreateWindowExW` returns and **before any
      layout runs**, so `set_root`'s first pass already uses the real
      scale. Construct through `DipScale::from_dpi`, and **add no
      zero-DPI guard here** (T2 finding F-16): `from_dpi` already floors
      a zero to the identity, and a second guard would put the same rule
      in two places. **Landed as `pub(crate) scale: DipScale`** — read
      from `emit.rs` at T5 but never by a host, which is DD-004's "no
      host needs the scale factor" expressed as visibility. It carries a
      `#[allow(dead_code)]` forward pointer, in `dip_scale`'s shape,
      because T4 writes it and T5 is its first reader.
- [x] Realise the DIP `width` / `height`: create at the requested
      numbers, then apply `size × s` via `SetWindowPos` before the window
      is shown. **The correction belongs inside `window::create`, not in
      `wasamo_window_create`** (T1 finding): `window::create` has three
      callers — the ABI entry point, `wasamo_load_ui` (which creates its
      own 800 × 600 window and never goes through
      `wasamo_window_create`), and the Rust-native
      `lib.rs::window_create`. A correction placed at the ABI function
      would leave every `.ui`-loaded window — i.e. all three example
      hosts — at the wrong physical size. **Landed as
      `window::realize_dip_window_size`, called from `create`**, and the
      call-site audit confirms all three callers are covered.
- [x] **Decide the rounding of the DIP → physical window size.** Named
      here rather than met mid-edit (found while checking T4's landing
      site at T3 close). `SetWindowPos` takes `i32`, the requested size
      is `i32` DIP, and `size × s` is an `f32`: 800 DIP at 125% is
      exactly 1000, but 801 DIP at 150% is 1201.5 and something must
      decide. **T2 deliberately shipped no integer conversion except
      `surface_pixels`**, whose contract is *surface allocation* — `ceil`
      plus a one-pixel floor, chosen because a truncated surface clips
      the last column of glyph coverage. Reaching for it here would
      borrow a rule written for a different purpose, which is exactly the
      F-14 / F-15 class. The candidates are `round` (nearest physical
      size, off by at most half a pixel in either direction), `ceil`
      (never smaller than requested, consistent with `surface_pixels` but
      for no stated reason), and `trunc` (rejected on the same grounds
      `surface_pixels` rejects it). **T4 decides, records the reason, and
      decides separately whether the rule belongs inside `DipScale`** —
      if it does, it is a second rounding contract in the type and needs
      its own test; if it does not, the arithmetic lives at the call site
      and the type's single-rounding-contract story stays intact. Note
      that T10's window-measurement check (800 × 600 → 1000 × 750 at
      125%) is **exact and therefore cannot discriminate any of the
      three** — the same shape as F-13.
      **Decided: `round`, inside `DipScale` as
      `window_size_to_physical((i32, i32)) -> (i32, i32)`.** The window
      rectangle carries a logical-size *fidelity* contract rather than an
      allocation contract, so the failure is two-sided and nearest is the
      integer that minimises it; and nearest is what `MulDiv(v, dpi, 96)`
      — the OS's own rule for the `WM_DPICHANGED` suggested rectangle T7
      applies verbatim — produces, so creation and the OS agree instead of
      drifting. It lives in the type because the call-site alternative
      reaches for `factor()`, which F-15's carry-forward names as its
      re-trigger criterion. Integer in and out, `f64` internally, so 100%
      is the exact identity for every `i32`. The discriminating test was
      constructed deliberately (801 and 803 DIP at 125%); **`ceil` passes
      every exact product, T10's check included** — measured, as mutation
      W2.
- [x] **Decide where in `window::create` the correction runs, and with
      which flags.** Also named here rather than met mid-edit.
      `SetWindowPos` dispatches `WM_SIZE` **synchronously, before it
      returns** — the property DD-003 makes load-bearing for
      `WM_DPICHANGED` — and `create`'s body has a seam that decides what
      that nested message finds: `GWLP_USERDATA` is installed at
      [`window.rs:83`](../../../../wasamo-runtime/src/window.rs), after
      the `WindowState` is boxed. A correction placed **before** that
      line dispatches into a `wnd_proc` that cannot reach any state;
      placed **after**, it dispatches into the live `WM_SIZE` arm with
      `root_widget` still `None`. Both are no-ops **today**, which is
      exactly why the choice must be recorded rather than fallen into:
      T5 makes that arm divide by the window's scale and T7 makes the
      ordering a correctness constraint. **Flags are part of the
      decision**: `CW_USEDEFAULT` placement means the correction must not
      move the window, so it needs `SWP_NOMOVE` — DD-003's
      `SWP_NOZORDER | SWP_NOACTIVATE` pair is for the `WM_DPICHANGED`
      path, where the OS-suggested rectangle *is* applied, and copying it
      verbatim here would move the window to whatever `x` / `y` are
      passed.
      **Decided: before the `GWLP_USERDATA` install**, with
      `SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE`. The nested dispatch
      then cannot reach runtime state **by construction** rather than by
      the accident that the two arms it would otherwise enter are
      currently `if let Some(_)` over `None`; measured, `state_ptr` was
      null for every one of the nine messages the correction dispatches
      at 125%. The symmetry-with-T7 alternative was rejected as a *false*
      symmetry: there the window is fully built and the nested `WM_SIZE`
      is required to re-lay out. The failure result is discarded, per
      DD-003's log-and-survive and the file's existing `let _ =`
      convention, with the consequence stated rather than hidden.
- [x] **The flash-free confirmation has a sharper answer than the plan
      assumed.** Creation and `wasamo_window_show` are separate ABI
      calls, but an in-between path *does* query geometry:
      `window::set_root` calls `GetClientRect` for its first layout, and
      `wasamo_load_ui` calls it between create and show. The property
      therefore holds only because the correction runs inside
      `window::create` before it returns — which is the bullet above,
      restated as the reason it is not optional. Confirm by ordering,
      and record that `set_root`'s first layout is the consumer that
      would have seen the uncorrected rectangle.
      **Confirmed by measurement**: the probe prints the correction, then
      the `GWLP_USERDATA` install, then `create`'s return, then
      `set_root`'s `GetClientRect` reading the corrected `982 × 703`.
- [x] Enable the `Win32_UI_HiDpi` feature in
      `wasamo-runtime/Cargo.toml` (prerequisite for `GetDpiForWindow`;
      the awareness API itself is T9) and re-sync
      [architecture.md §4.5](../../../../docs/architecture.md) at T12.
      **Landed**, and measured as sufficient for T9's declaration as
      well: the throwaway probe compiled `SetProcessDpiAwarenessContext`
      and `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` against this
      feature list with no further `Cargo.toml` edit. T9's two *query*
      symbols were not exercised, so that half is not claimed.

**Start gate:** trap #5 (the per-window shape is what M4-Phase 8 will
consume; record the invariant); traps #1, #2, #4, #6 and #7 were added
at the start gate. **End gate:** scale seeded before first layout,
verified by ordering rather than by comment; workspace green as a
regression check only (see the note above). **All met** — the ordering
discharged twice, by construction (the scale is a struct field, so no
window exists without one and there is no statement order to invert) and
by a printed probe trace. Review lane raised to full independent review
(F-25).

---

### T5 — The conversion seams

**The full-review-lane structural task.** Converts at the boundary and
nowhere else. Every conversion is the identity at `s = 1`, so the
observable behaviour is unchanged until T9 — which is what makes this
landable as one reviewed commit rather than a visible regression.

- [ ] **Inbound, client extent** (audit rows 1–2): `wnd_proc`'s `WM_SIZE`
      client extent and `set_root`'s `GetClientRect` divided by `s`
      before reaching `run_layout_as_window_root`. **Row 2 covers two
      sites** (T1 finding F-1): `set_root` and
      [`emit::flush_layout`](../../../../wasamo-runtime/src/emit.rs),
      the reactive drain's Phase 2 layout pass, which performs the same
      `GetClientRect` → `run_layout` conversion after every
      size-affecting property write.
      Both sites convert through `DipScale::pair_to_dip`, not by
      hand-written division (T2 finding F-15).
- [ ] **Inbound, pointer** (audit row 3): `WM_MOUSEMOVE` /
      `WM_LBUTTONDOWN` / `WM_LBUTTONUP` coordinates divided by `s` at the
      window procedure, so hit-testing and hover run in DIP —
      `pair_to_dip` again; inbound has one form because positions and
      extents alike are a componentwise division there.
- [ ] **Inbound, readback** (audit row 9): `visual_rect`'s
      `Visual.Offset` / `Visual.Size` readback divided by `s` alongside
      the pointer. Record honestly in [log.md](./log.md) that the two
      conversions **cancel today** — hit-testing sources its geometry
      from the visual tree — and that they stop cancelling the moment
      M4-Phase 2 sources geometry from layout or introduces a
      DIP-denominated hit-area rule.
- [ ] **Introduce the node-side scale cache**, defaulted to 1 in every
      `WidgetNode` constructor — as `DipScale::default()`, which is the
      identity, rather than a hand-written literal (T2 finding F-16). T5
      is its first reader; T6's walk is its only writer, so between T5
      and T6 it is permanently 1 — the same identity world every other
      conversion lands into.
- [ ] **Record the direct-hosting path as a stated limit** (T3 finding
      F-24). T1's carry-forward wrote the cache's re-trigger as "any
      *future* path that attaches a subtree without running the walk",
      listing M4-Phase 2 and M4-Phase 8. **One such path already ships**:
      `lib.rs::window_add_widget` attaches a widget's Visual to
      `WindowState::root` without putting it in `root_widget`, so the
      subtree is outside **the whole of this phase's machinery** — no
      layout, no `sync_visuals`, no scale cache write, and (T6) no
      re-rasterization. Its scale cache stays `DipScale::default()`
      forever, which is correct-looking at 100% and wrong at any other
      scale. Nothing to fix here — the conversions are unconditional and
      an unreached node is simply unconverted — but the limit is stated
      rather than discovered at M4-Phase 8, when a tree really can move
      between differently-scaled windows.
- [ ] **Fix `emit::flush_layout`'s layout entry, as its own commit**
      (T3 finding F-23; **owner may reassign this item — it is a
      pre-existing defect, not a T5 deliverable**). `window::set_root`
      and the `WM_SIZE` arm call `run_layout_as_window_root`, which
      forces the root `LayoutNode` to `Fill` / `Fill`; the reactive
      drain's layout phase calls the plain `run_layout`, which does not.
      A root container that is `Shrink` with a `Fill` descendant
      therefore lays out correctly on resize and **collapses that
      descendant on any property write** — the M3-Phase 4 T6 failure
      that `run_layout_as_window_root`'s own doc comment describes, still
      live on the drain path. It lands here because T5 already edits that
      exact call site for the inbound conversion (row 2b), so fixing it
      elsewhere would mean touching the line twice. **Separate commit
      with its own before/after frames** — it is a behaviour change and
      must not ride inside a conversion commit.
- [ ] **Outbound, Visual geometry** (audit rows 4–6): `sync_visuals`
      node writes, the ScrollView intermediate Visual, and the Button /
      ToggleButton label writes relocated by T3 — all multiplied by `s`.
      **Through the named operations, not by hand** (T2 finding F-15):
      `extent_to_physical` for every `SetSize`. Writing
      `dip * scale.factor()` satisfies a prose reading, defeats the
      enforcement the type exists to provide, and is wrong only at
      non-dyadic scales — where, per F-13, only two of the phase's three
      test factors would notice. The ScrollView recursion stays entirely
      in DIP (`child_parent_abs` is `(offset.0, offset.1 - applied_y)` in
      DIP); only the Composition writes multiply.
- [ ] **The offsets are not one case but two** (T3 finding F-19; this
      bullet previously said `relative_offset_to_physical(abs,
      parent_abs)` for *every* `SetOffset`). **Only row 4 takes a
      difference** — the node's own write, `computed.offset −
      parent_abs_offset` — and that is the one that converts once on the
      difference: subtract in DIP, multiply the result, one rounding
      instead of two. **Rows 5 and 6 are already parent-relative** as
      landed at T3: the ScrollView intermediate's offset is
      `(0, −applied_y)` and the label's is
      `(BUTTON_PAD_H, BUTTON_PAD_V)`. There is no absolute pair to
      subtract there, and forcing the named operation would mean
      inventing one. T2's landed API has a scalar `to_physical` and an
      extent form but no already-relative *pair* form, so **T5 decides
      explicitly** between calling `to_physical` per component and adding
      a named already-relative operation, and records which and why in
      [log.md](./log.md). Either satisfies the rounding rule — a single
      multiplication of an already-computed relative quantity is exactly
      one rounding — but the choice must be made rather than fallen into,
      because this is precisely where F-15's "reach for `factor()`"
      temptation is strongest.
- [ ] **Verify the unchanged rows as assertions, not omissions**: row 8
      (`SetRelativeSizeAdjustment(1, 1)` — a relation between two
      physical quantities), row 10 (`measure` returns DIP — the fact
      that carries "layout stays DIP"), row 11 (`size_sp` is DIP), row
      12 (`InsetClip` insets are all zero, and zero is scale-invariant).
      **Row 12's site list is ScrollView / Grid / ZStack**, not
      ScrollView / Grid / Box (T1 finding F-2): `WidgetNode::box_`
      installs no clip, and `WidgetNode::zstack` does. The row's
      conclusion is unaffected; the sites asserted against are not.
- [ ] **Decide and record the unit of `WindowState`'s six callback
      slots** (T1 finding F-3). `resize_fn` / `mouse_move_fn` /
      `mouse_down_fn` / `mouse_up_fn` are invoked from `wnd_proc` with
      the raw message values, so this task changes their unit as a side
      effect. No ABI or Rust-native function installs them today —
      DD-004's claim is confirmed — but the unit must be stated
      deliberately (DIP, per W1), not inherited from the seam edit.
- [ ] **Two things T4 left for T5 to pick up, both small and both
      auditable** (T4 findings, recorded so they are not discovered at
      the edit). `WindowState::scale` landed as `pub(crate)` — which is
      what makes `emit::flush_layout`'s row-2b division reachable without
      widening the public API, so T5 needs neither `pub` nor an
      accessor — and it carries a `#[allow(dead_code)]` forward pointer
      because T4 writes it and **T5 is its first reader**. Removing that
      attribute is part of this task; leaving it in place would silence a
      real warning for whatever comes next.
- [ ] Apply the carrier / threading shape T1 decided (risk R-5): the
      scale is authoritative on `WindowState` and cached on each
      `WidgetNode`, written only by T6's walk; `sync_visuals`,
      `hit_test_click_inner` and `update_hover_inner` read `self.scale`
      and keep their signatures. The layout entry points keep theirs
      too, and their `f32` arguments become DIP. The only test edits are
      the **7 `hit_test_click` call sites in 4 files**
      (`button_enabled.rs` ×3, `togglebutton_runtime_integration.rs` ×2,
      `bool_binding_live_propagation.rs` ×1,
      `iteration_mutation_integration.rs` ×1), which change from `i32`
      to `f32` because the pointer's DIP type is `f32`.

**Start gate:** traps #1 and #2. **End gate:** the **call-site audit
table** — DD-002's 13 rows, each with its classification, the source
location as landed, and the verification that closed it; the claim being
checked is "no coordinate enters or leaves outside these rows".
**Row 13 is closed at T4, not here** (`create_hwnd`'s `CreateWindowExW`
width / height): T4's audit records the landed site and its three
callers, so T5's table cites that rather than re-deriving it. Recorded
because until T4 **no task in this plan claimed row 13** — the bullets
above cover rows 1–6 and 8–12 and §T6 covers row 7, so a T5 that closed
"every row it was given" would still have left one open (F-26).
Full independent review before merge.

---

### T6 — Text-surface resolution + the re-rasterization walk

**The phase's hard part** (preamble obligation 3, risk R-1). Coordinates
being right does not make text crisp; an implementation that stops at T5
produces exactly the blur the phase set out to remove and passes every
test.

- [ ] Allocate the drawing surface at **`ceil(dip × s)` pixels** on each
      axis, through T2's named rule — `DipScale::surface_pixels`, which
      returns a `(u32, u32)` pixel count. Two consequences of the landed
      signature (T2 finding F-14): `CreateDrawingSurface` takes an `f32`
      `Size`, so either cast at the call or move to
      `CreateDrawingSurface2`'s `SizeInt32` — DD-002's contract is the
      pixel count, not the API pair — and **remove `draw_text`'s existing
      `width.max(1.0)` / `height.max(1.0)`**, because the one-pixel floor
      now lives inside `surface_pixels` and leaving the old clamp would
      put the same rule in two places.
- [ ] Set the D2D device context to **`96 × s` DPI** after `BeginDraw`,
      so `create_text_layout`'s `max_w` / `max_h` stay DIP and
      `size_sp` stays a DIP font size while rasterization and hinting
      happen at device resolution. This is the phase's **only** legitimate
      use of `DipScale::factor()` in place of a named operation (T2
      finding F-15): T2 deliberately did not wrap it, because it carries
      no rounding contract and wrapping it would put a DirectWrite
      concern inside a type whose value is having no rendering
      dependency.
- [ ] **Convert the atlas origin** (risk R-3): `BeginDraw`'s offset is in
      pixels and must be divided by `s` before use as the D2D drawing
      origin — `to_dip`, one component each. Write it deliberately — the
      offset is frequently `(0, 0)`, so omitting it works most of the
      time and displaces text within its own surface intermittently.
- [ ] Keep the brush mapping one-to-one: the Visual's size is the exact
      `f32` physical `dip × s`, the surface is `ceil(dip × s)` pixels,
      and the at-most-one-pixel excess is transparent padding.
- [ ] The **re-rasterization walk**: surfaces are built at scale 1 during
      construction (before the tree is attached to a window) and brought
      to the window's scale by a walk run at attach. Re-creates each
      text-bearing node's surface and brush from state the node already
      holds; adds no retained state. Shape decided at T1:
      `WidgetNode::apply_scale_recursive(&mut self, compositor,
      renderer, scale)`, called from `window::set_root` after the first
      layout and from T7's handler; it writes the node-side scale cache
      T5 introduced, then rebuilds `WidgetData::Text { content, style }`
      and `ButtonData`'s `label_text` / `label_style` surfaces.
      **The walk reads `ButtonData.label_size` rather than re-measuring**
      (T3 finding F-20): T3 retained the measured extent, and `measure`
      is DIP and scale-invariant (row 10), so a re-measure inside the
      walk can only return the same pair — which would make the walk a
      second producer of a fact the node now stores, the drift F-14 and
      F-16 exist to prevent, on the phase's highest-consequence path.
      **The walk also writes no Composition geometry.** After T3 every
      `SetOffset` / `SetSize` in the runtime is inside `sync_visuals`,
      and that property is what makes the T5 audit complete; a walk that
      rewrites a Visual's size while it is there breaks it silently.
      **And the walk has the same reach as `sync_visuals`, not a wider
      one** (T3 finding F-24): both callers — `window::set_root` and T7's
      handler — traverse `state.root_widget`, so a subtree attached
      through `lib.rs::window_add_widget` is never walked and keeps text
      rasterized at scale 1. Same stated limit as T5's, and R-1's
      crispness claim is bounded by it: it holds for widgets the window
      owns as content.
- [ ] Thread the scale into `draw_text`'s five call sites. Note the
      borrow order T1 hit: `update_button_label` must read the node's
      scale **before** `self.button_data_mut()`, which borrows all of
      `self`; `update_text_content` / `update_text_style` destructure
      `self.data` directly and need no such care.
- [ ] Confirm re-rasterization does **not** change any node's
      `SizeConstraint::Fixed(w, h)` — `measure` is DIP and unaffected by
      scale — so it cannot invalidate layout. This is the property T7
      depends on.
- [ ] If the atlas-origin conversion proves fragile in practice, the
      permitted alternative is expressing the surface's resolution as a
      context transform instead of a context DPI; the contract is
      `ceil(dip × s)` pixels and device-resolution glyphs, not the API
      pair. Record the choice and the reason in [log.md](./log.md).

**Start gate:** traps #1 (audit row 7) and #6 (Composition surface
recreation is WinRT-fallible). **End gate:** row 7 closed in the audit
table; the layout-invalidation non-effect verified; local rendering
unchanged at 100% — **captured after `cargo build --release
--workspace`, never after a host-package build** (T3 finding F-21: a
host build relinks `wasamo.dll` from a **stale uplifted rlib**, so the
DLL carries a fresh timestamp and old object code, and the frame
silently shows the previous runtime). Full independent review before
merge.

---

### T7 — `WM_DPICHANGED` propagation

- [ ] Handle `WM_DPICHANGED` in `wnd_proc` in the **fixed order**:
      (1) update `WindowState`'s cached scale from `HIWORD(wParam)`;
      (2) apply the OS-suggested rectangle from `lParam` via
      `SetWindowPos(..., SWP_NOZORDER | SWP_NOACTIVATE)`;
      (3) the nested synchronous `WM_SIZE` re-runs layout through T5's
      inbound seam; (4) re-rasterize text surfaces through T6's walk;
      (5) return `LRESULT(0)`.
- [ ] Encode the reason for step 1 preceding step 2 structurally, not as
      a comment: `SetWindowPos` dispatches `WM_SIZE` **before it
      returns**, so a scale updated afterwards would leave that pass
      laying out and projecting with the stale factor. This is the
      phase's single most likely ordering defect and is invisible at
      100%. **The premise is now measured rather than inherited** (T4):
      at 125% a size-changing `SetWindowPos` dispatches
      `WM_WINDOWPOSCHANGING`, `WM_GETMINMAXINFO`, `WM_NCCALCSIZE`,
      `WM_WINDOWPOSCHANGED`, **`WM_SIZE`**, then `WM_GETICON`, all before
      it returns — and at 100% it dispatches **no `WM_SIZE` at all**,
      because the size does not change. The second half is the sharper
      fact: this ordering defect cannot be produced, let alone observed,
      before T9.
- [ ] **Do not inherit T4's flags, and do not reuse its helper**
      (T4 finding F-30). `window::realize_dip_window_size` converts a
      **DIP size** and must not move the window, so it passes
      `SWP_NOMOVE`; this step applies an **OS-supplied physical
      rectangle** whose whole content is a new position *and* size, so
      `SWP_NOMOVE` would pin the window and defeat the suggested
      rectangle on every monitor crossing. The two sites sit either side
      of the same mistake, and T4's own bullet warns against copying
      DD-003's flags **into** `create`; this is the warning in the other
      direction, which is where a reader who has just read `create` is
      standing.
- [ ] Apply the suggested rectangle (do not ignore it): it preserves the
      window's logical size across the change, which is what the DIP
      contract means.
- [ ] **Failure handling:** log and survive. A failed re-rasterization
      leaves a surface at the old resolution — visibly blurry and honest
      about it; a failed `SetWindowPos` leaves the rectangle unchanged.
      Neither tears down the window; `wnd_proc` returns `LRESULT(0)`
      regardless. The runtime is **not** put into `Diverged`, which is
      for reactive-engine divergence.
- [ ] `WM_GETDPISCALEDSIZE` is **not** handled this phase — recorded as
      forward exposure, not an omission.
- [ ] **Row 10's site list is ScrollView / Grid / ZStack**, not
      ScrollView / Grid / Box (T3 finding F-18). T1's F-2 established
      this against the source — `WidgetNode::box_` installs no clip and
      `WidgetNode::zstack` does — but dispositioned the correction only
      to T5, because the row it was reading was DD-002's row 12. The same
      wrong widget set appears independently in
      [DD-003 §Structural side-effect enumeration](../decisions/dd-m4-p1-003-dpi-change-propagation.md)
      row 10, which is *this* task's close artifact, so a T7 that builds
      its enumeration from the ADR wording would assert a site that does
      not exist while never looking at the one that does. Re-verified at
      T3: `CreateInsetClip` appears in `scroll_view`, `grid` and
      `zstack`, and nowhere else. The row's conclusion (all insets are
      zero, zero is scale-invariant) is unaffected.
- [ ] **Row 7 is now literally true and should be asserted, not
      inherited.** DD-003 row 7 says the Button label Visual's offset and
      size are covered "because DD-002 moved that write into the sync
      pass". T3 performed that move, so the assertion T7 makes is that
      the label follows a scale change through `sync_visuals` with no
      handler-specific code — the row's own stated reason for not being
      the phase's silent bug.

**Start gate:** trap #2 (the phase's primary side-effect surface) and
trap #5. **End gate:** the **structural side-effect enumeration** —
DD-003's 13 rows, each stated as updated or verified-unchanged. Rows
9–13 (`SetRelativeSizeAdjustment`, clip insets, signal registry /
effect graph / binding state / widget pointers, `MUTATION_CAP` and drain
accounting, hover and press state) must be verified as unchanged, not
assumed: a scale change must not enter the reactive drain at all. Full
independent review before merge.

---

### T8 — Windows integration evidence (mock-free, CI-gated, fail-not-skip)

Placed **before** T9 on purpose: it drives `s ≠ 1` synthetically, so the
sequencing thesis does not defer all scaled-path risk to the end
(risk R-4).

- [ ] A created window's cached scale equals `GetDpiForWindow`.
      **This needs a test seam, which does not exist yet** (T4 finding
      F-29). The field landed as `pub(crate) scale` on `WindowState`, and
      the phase's Windows integration tests live in
      `wasamo-runtime/tests/`, i.e. in a separate crate that can reach
      only `pub` items. The established shape is a `#[doc(hidden)] pub`
      accessor in [`lib.rs`](../../../../wasamo-runtime/src/lib.rs)'s
      `ffi` module, alongside `__install_owning_thread_for_test` and its
      siblings. Widening the field to `pub` is the wrong fix: it would
      put the scale factor on a `pub use`-exported type and ship the
      host-visible surface DD-004 declines.
- [ ] **The integration-side positive control**: drive a scale change
      through the handler and assert that **the layout's DIP results are
      unchanged** while Visual offsets and sizes have moved by the scale
      ratio. The first half is what distinguishes a correct
      implementation from one treating physical pixels as logical —
      which would change the DIP results and, visibly, the WrapPanel
      line count.
- [ ] Exercise at 125% / 150% / 200% — but **not as three equal probes**
      (T2 finding F-13). At a power-of-two factor the multiplication is
      exact, so convert-once and convert-twice agree everywhere and a
      DIP round trip is exactly the identity; a brute-force search found
      no disagreeing pair at 200% at all, against a witness one ulp apart
      at 150%. 200% is therefore a magnitude check, and **the rule
      verification is carried by 125% and 150%**. Adding more round
      factors would not help; adding an awkward one would.
- [ ] **Record the stated limit with the test** (preamble obligation 5):
      a synthesised `WM_DPICHANGED` proves the handling path; it does
      **not** prove that crossing a real monitor boundary delivers the
      same message with a usable suggested rectangle. That half is
      T11's.
- [ ] Follow the established `0x80070005` guard pattern — **fail, not
      skip**, on a runner without Compositor capability. Any new guard
      must be shown to fire on an environment that actually lacks the
      capability before the test lands; a guard verified only on the
      happy path is not verified.

**Start gate:** trap #4 (each assertion fires directly, not
incidentally). **End gate:** tests green locally and in CI; the stated
limit recorded in the test and in [log.md](./log.md).

---

### T9 — Declare Per-Monitor-Aware V2 + three-host rebuild

**The commit that turns the phase on.** Everything behind it is already
under test; this flips the process posture so the OS starts reporting
real per-monitor DPI and the identity conversions become live ones.

- [ ] `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)`
      as the **first OS-touching act** of `runtime::init()` — before
      `CreateDispatcherQueueController`, before `Compositor::new`, before
      `TextRenderer::new`, and **below** the existing
      `RUNTIME.get().is_some()` early return. T1 verified both halves:
      the declaration takes effect from that position, and the existing
      one-shot already prevents a second `wasamo_init` from
      re-declaring, so **no new guard is added**. Placing it *above* the
      early return is the defect to avoid — it would re-declare and take
      `ERROR_ACCESS_DENIED` on a process that had already declared
      correctly. `capture_owning_thread()` necessarily precedes it and
      is not OS work that can lock the awareness.
- [ ] **Tolerate failure.** `ERROR_ACCESS_DENIED` means the process's
      awareness was already set — typically by a legitimate host that
      declared its own. `wasamo_init` still returns `WASAMO_OK`; the
      outcome is recorded through the existing thread-local last-error
      mechanism as a diagnostic string, not a returned status. Do **not**
      add a branch that assumes scale 1 on failure — that is the one
      option that can be wrong, and it is invisible at 100%.
- [ ] No legacy-OS fallback: both `SetProcessDpiAwarenessContext` (1703+)
      and `GetDpiForWindow` (1607+) predate the stated Windows 10 1809
      floor.
- [ ] **No `Cargo.toml` edit is needed** — T4's `Win32_UI_HiDpi` covers
      this task's declaration symbols. Measured, not inferred: T4's
      throwaway probe compiled `SetProcessDpiAwarenessContext` and
      `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` against the landed
      feature list with no further change. The two symbols the
      effective-level assertion below needs —
      `GetWindowDpiAwarenessContext` and `AreDpiAwarenessContextsEqual` —
      were **not** exercised, so if either is missing that is a T9 edit
      and T12's §4.5 re-sync must pick it up.
- [ ] Integration test asserting the **effective** level —
      `GetWindowDpiAwarenessContext(hwnd)` compared against
      `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` with
      `AreDpiAwarenessContextsEqual`. Assert the level in force, not that
      a particular function was called.
- [ ] **Rebuild and run all three hosts** — C, Rust, Zig — with no
      manifest asset and no build-system edit (preamble obligation 6,
      risk R-8). This is the auditable artifact for the
      declarative-host boundary claim; it must be run, not inferred from
      "we did not edit them". **Build the workspace, not just the hosts**
      (T3 finding F-21): a host-package build relinks `wasamo.dll` around
      the **stale uplifted** `<profile>/libwasamo_runtime.rlib`, which
      cargo refreshes only on a primary-package build — so the hosts
      would run against **pre-T9 object code** carrying a fresh DLL
      timestamp, and the artifact would report the wrong awareness level
      while every build step looked green.
- [ ] **Re-run the trap-#4 decision explicitly** for the diagnostic
      branch (see [preamble.md §Implementation gates](./preamble.md#implementation-gates)).
      If the tolerated-failure path cannot be fired by a test because
      process DPI awareness is a one-shot per process, record that as a
      **stated limit with its reason** in [log.md](./log.md) — not as an
      inherited "non-applicable".

**Start gate:** traps #1, #2, #4 (the diagnostic branch), #5. **End
gate:** effective-level assertion green; three-host rebuild recorded;
the trap-#4 disposition recorded either as a firing test or as a stated
limit. Full independent review before merge.

---

### T10 — Assistant GUI evidence (positive controls A, B, and C's path form)

Assistant-automated evidence is **launch + screenshot capture +
analysis of the captured image**. `Start-Process` survival is a
supporting "no early crash" signal only. Capture mechanics
(`CopyFromScreen`, not `PrintWindow`, which is blank under Composition)
are in
[verification-environments.md](../../../../docs/notes/verification-environments.md)
§Observation 4. This does not replace the owner smoke.

- [ ] **Positive control A — crispness, before and after.** The same
      text at the same monitor scale, captured before and after the
      change, compared at magnification. **The pair is the control; the
      "after" frame alone proves nothing.** If a pre-change frame is
      reused rather than re-captured, check the commit it was captured
      at against the current surface first.
- [ ] **Positive control B — logical layout invariance.** The same `.ui`
      at the same logical window size, captured at 100% and at 125%,
      with wrap positions and element order compared. **Invariance is
      the evidence.** Note that "the window's physical size scales with
      the scale factor" is *not* a control — DWM bitmap stretching
      satisfies it too.
      **The invariance is not bit-exact, and the control must say by how
      much** (T4 finding F-28). DD-004 defines `width` / `height` as the
      **outer** rectangle, and only that rectangle scales exactly:
      measured, an 800 × 600 DIP request is 1000 × 750 physical at 125%,
      while the *client* area goes from 784 × 561 DIP at 100% to
      982 × 703 physical = **785.6 × 562.4 DIP** at 125%, because the
      non-client frame is 8 px per side at 96 DPI and 9 px at 120 DPI and
      therefore scales by its own rounded metric rather than by `s`.
      Layout receives the client extent, so a correct implementation lays
      out into ~1.6 DIP more width at 125% and a wrap position near a
      line-break boundary may legitimately move. A control that demands
      identical wrap positions can therefore fail a correct build. State
      the tolerance, or drive both captures from a controlled **client**
      size rather than a controlled outer size.
- [ ] **Positive control C, path form.** Two frames across a display
      setting scale change on the development machine while the window
      is up, showing text still crisp and the logical layout unchanged.
- [ ] **Window measurement check** (risk R-9): a window created at
      800 × 600 DIP measures 1000 × 750 physical at 125%. Cheap,
      concrete, and the only in-phase check of DD-004's outer-window
      -rectangle claim — but **not a positive control on its own**, and
      the plan originally treated it as one. T1 measured the *unaware*
      baseline at exactly 1000 × 750 as well, because DWM stretches the
      logical 800 × 600 by the same factor. The number is therefore
      satisfied by a build that never declares awareness at all. Pair it
      with something that separates the two: the effective-context
      assertion from T9, or the crispness pair (control A) on the same
      frame. **T4 measured all three outcomes in one session and one
      build tree** — the plan previously carried two of them from T1 plus
      a stated failure direction, and one of those numbers was
      conditional in a way the restatement dropped (F-27):

      | State | window rect | client | gallery tiles/row |
      |---|---|---|---|
      | unaware (with or without the correction — it is the identity) | 1000 × 750 | 980 × 701 | 7 |
      | aware, correction absent | 800 × 600 | 782 × 553 | **7** |
      | aware, correction present, **T5's inbound seam still absent** | 1000 × 750 | 982 × 703 | 9 |

      The plan's earlier "drops from 7 tiles per row to 6" is **T1's
      number for T1's build**, which carried the *complete* conversion
      machinery — its client 782 physical became 625.6 DIP. Read as a
      property of the missing correction alone it does not reproduce:
      aware-without-correction reads 7. The correct reading is that
      **rows 1 and 3 share a rectangle and rows 1 and 2 share a tile
      count**, so no single number separates the three and only the pair
      does. Frames and numbers: [log.md](./log.md) §T4 and
      [evidence/t4-probe/](./evidence/t4-probe/).
      Note also that the third row's **9** is the pre-T5 signature; once
      the inbound seam lands the same state must read **7** again, and a
      T10 that inherits 9 as the expected number would be pinning a
      half-finished phase.
- [ ] **Re-derive the capture coordinates** for later phases against the
      new coordinate space, as the evidence artifact T12's
      `verification-environments.md` revision consumes (risk R-7).
- [ ] **Deliver the runnable set to the owner's laptop** — host
      executable + `wasamo.dll` + compiled `.uic` — so T11 is one
      observation rather than a build-and-deliver task (preamble
      obligation 7).

- [ ] **Every capture is preceded by `cargo build --release
      --workspace`** (T3 finding F-21). A host-package build *does*
      recompile the runtime and *does* relink `wasamo.dll` — but it
      whole-archives the **stale uplifted** rlib, so the DLL is fresh by
      timestamp and old by content. Measured at T3, where a mutation
      built that way produced a frame identical to the unmutated build.
      **A freshness check on the DLL does not detect this**, which is why
      the remedy is the build command and not a guard. This is the one
      failure mode that can make every control in this task pass against
      code that is not the code under test.
- [ ] **Reusable from T3**: the capture script
      [evidence/capture-t3-label-writes.ps1](./evidence/capture-t3-label-writes.ps1)
      carries the working mechanics — PMv2 capture process,
      `CopyFromScreen` over `GetWindowRect`, click points derived from a
      probe frame, and the swap-the-compiled-IR trick that runs an
      evidence `.ui` through a built host without touching the repo.

**Start gate:** trap #7. **End gate:** screenshots, the assistant's
analysis of each, and the pair-based positive controls; the re-derived
capture coordinates recorded. Full independent review before merge.

---

### T11 — Owner human-visible smoke (positive control C, literal form)

The half of AC7's third requirement that a synthesised message cannot
reach. Owner-executed on a laptop plus external display at different
scale factors.

- [ ] Drag the window between monitors at different scale factors;
      confirm the logical layout is preserved and text stays crisp
      through the crossing.
- [ ] Confirm the non-client area (caption, borders) scales with the
      window — the V2 automatic behaviour this phase relies on in full.
- [ ] Record the verdict in [log.md](./log.md). Neither this nor T8
      alone discharges AC7's third requirement; both together do.

**Start gate:** none (owner-executed). **End gate:** owner verdict
recorded; any finding triaged to a task or to
[handoff.md](./handoff.md).

---

### T12 — Step-end local gates + Moment 2 re-sync + step retro

- [ ] Clean rebuild + `cargo test --workspace` green; all three example
      hosts build in the documented order. **From a cold target
      directory the workspace test build needs
      `cargo build -p wasamo-runtime` first** (T1 finding F-5,
      pre-existing): `wasamo-dll/build.rs` whole-archives the uplifted
      `<profile>/libwasamo_runtime.rlib`, which cargo produces only once
      `wasamo-runtime` has been built as a primary package. Also correct
      [AGENTS.md §Build ordering](../../../../AGENTS.md)'s claim that
      workspace-wide builds implicitly satisfy the ordering, which holds
      for `counter-rust` but not for the cdylib from a cold directory.
      **The same correction carries T3's finding F-21**, which that
      section is silent on and which shares F-5's root cause: because the
      whole-archived rlib is the **uplifted** copy, a host-package build
      relinks `wasamo.dll` around object code that cargo did not refresh,
      and the result is a fresh DLL timestamp over a stale runtime. F-5
      is that path failing loudly when the uplifted rlib is absent; F-21
      is it succeeding quietly when it is merely old. One revision, one
      root cause, two symptoms.
- [ ] **Moment 2 doc sync — divergence correction.** Re-verify each
      Moment 1 statement against what actually landed and correct
      divergences. The statements flagged at ADR time as most at risk
      are the outer-window-rectangle claim and the font-size unit; both
      are checked against running behaviour, not assumed. **The
      outer-rectangle half already has its measurement** (T4): an
      800 × 600 DIP request produced a 1000 × 750 physical *outer*
      rectangle at 125%, while the client area landed at
      785.6 × 562.4 DIP rather than 784 × 561. So the claim is true of
      the outer rectangle and **false if a reader transfers it to the
      client area** — the wording must stay where DD-004 put it. Cite
      [log.md](./log.md) §T4 rather than re-deriving it. Flip the
      status markers in
      [architecture.md](../../../../docs/architecture.md),
      [dsl_spec.md](../../../../docs/dsl_spec.md), and
      [abi_spec.md](../../../../docs/abi_spec.md) to
      implementation-synced, and re-sync architecture §4.5 (`windows`
      crate feature list) and §5.2 (initialization sequence) to the
      landed code.
- [ ] **Revise `verification-environments.md` Observation 4** with the
      capture coordinates T10 re-derived (risk R-7). Its stated premise
      — the host is DPI-unaware, so DWM stretches logical 800×600 to
      physical 1000×750 — is falsified by this phase, and later phases
      read the note as procedure.
- [ ] Flip the [M4 plan](../../plan.md) Phase 1 row to complete.
- [ ] Carry-forward to [handoff.md](./handoff.md) with re-trigger
      criteria: layout-derived hit rectangles (M4-Phase 2); the
      host-visible scale / work-area query trigger (M4-Phase 7 / 8);
      per-window differing scale (M4-Phase 8); resolution-dependent
      image assets (M4-Phase 4); integer pixel snapping; text
      rendering-quality tuning (M5); the custom-title-bar re-examination
      of V2 non-client scaling (M5); the non-zero clip inset re-check;
      and the note that a scale-dependent `measure` would turn T7's step
      ordering into a correctness constraint.
- [ ] Step retrospective per
      [retrospectives.md](../../../procedures/retrospectives.md).

Owned by the **phase-end batch**, not by T12 — these stay `[ ]` at T12
close: the CI run id, `handoff.md` finalization, the phase
retrospective, [preamble.md](./preamble.md)'s `status` flip, and:

- [ ] **File the vision decision record for the "show it goes red"
      obligation** (owner decision on the T3 retrospective, recorded in
      [log.md](./log.md)). Scope as decided: **mandatory for pure-logic
      unit tests only** — a new rounding-rule / unit-conversion /
      boundary-condition surface ships with at least one deliberately
      wrong implementation shown to turn its tests red — and the wider
      "any green / identical / passing observation" form is **explicitly
      not** codified. It changes trap #4's close artifact in
      [implementation-gates.md](../../../procedures/implementation-gates.md)
      from "the test name per added branch" to "the test name per added
      branch, plus the wrong implementation it was shown to catch";
      trap #7's artifact is unchanged. Per
      [AGENTS.md §Process rule lifecycle](../../../../AGENTS.md) the SSOT
      edit lands in the **same commit batch** that flips the record to
      `Accepted`, which is why no task before this one touches
      `implementation-gates.md`. Evidence to cite: T2's seven-mutation
      table and T3's three-mutation frame set, the second of which is the
      argument for *not* widening the rule.

**Start gate:** trap #3's documentation analogue (do not restate spec or
handoff content in derived prose — cite the owning document) and trap
#5. **End gate:** local gates green; the Moment 2 divergence corrections
recorded per statement; carry-forward recorded with re-trigger criteria.
