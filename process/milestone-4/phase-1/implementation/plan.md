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
**T5's row was extended at T5**, for the same reason: "the call-site audit
table" is true and incomplete. T5 also had a *discriminating* observation
available to it, because this plan had already predicted the number the
task would move — 9 tiles per row to 7 — and a throwaway declaration makes
that number readable at 125% without waiting for T9. An audit table plus a
predicted measurement is a stronger gate than an audit table, and the
measurement cost one build.

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

**Propagate corrections by proposition, not by string** (T4 delta review
finding 2, after the same failure three times). When a task falsifies a
claim, the reflex is to search this plan and the preambles for the
*phrasing* of the claim — and the documents that carry the same claim in
other words are then missed every time, however many passes are run. The
sites T4 kept missing say "does not re-decide layout", "layout results
are scale-invariant", "the DIP results are unchanged" and "invariance is
the evidence", and no string search over any of those finds the others.
So: **write the falsified proposition as one sentence first, then
enumerate the documents that assert it** — which in this phase always
includes the [ADR-set preamble](../decisions/preamble.md)'s Decisions
table, its cross-DD couplings and its verification list, this plan's task
bullets, and [preamble.md](./preamble.md) — and only then search. The
falsifiable test T5 inherits: **valid if the propagation pass names the
proposition and enumerates the asserting documents before searching;
falsified if a reviewer again finds an asserting site the pass never
visited.**

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
      **Corrected at T6 independent review:** that single primitive and
      no-new-state claim do not survive log-and-survive failure. Geometry scale
      and the DPI of the brush actually installed can diverge, so the landed
      shape is an explicit authoritative-target geometry entry plus
      `refresh_text_surfaces_recursive`, with a separate per-node
      `raster_scale` marker. T7 composes them around the nested `WM_SIZE`.
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
      **Reversed at T4** (independent review finding R-1, recorded here
      because this is a checked item describing what landed and it now
      says the opposite of the source): the type retains the **DPI** and
      derives the factor. "Every consumer wants the factor rather than
      the DPI" was the stated premise, and `window_size_to_physical`
      falsified it — an `f32` factor cannot express `dpi / 96` exactly
      unless 3 divides the DPI, so a rounding rule computed from it is
      not the rule the type documents. `Default`, `IDENTITY` and the
      zero-DPI fallback are unchanged in behaviour.
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

**Closed 2026-07-29.** Landed as **three code commits**: the seams
(`67435cd`); separately, the pre-existing `emit::flush_layout` defect
(`7b23854`), which is a behaviour change and must not ride inside a commit
whose whole claim is that nothing observable moves; and `a4939dc`, which
**replaces the readback's divisor** with the traversal root's scale
(finding R-2). Artifacts in
[log.md](./log.md) §T5: the pre-registered coordinate-carrying API
enumeration, the 13-row call-site audit built against it, a 14-row
side-effect enumeration, the trap-#3 mutator table, the three named
decisions with their rejected candidates, and two GUI artifacts of
different kinds. The task produced findings F-32 and F-33 and revisions to
§Task list, §T6, §T7, §T8, §T10, §T12, the preamble and the handoff.

**The identity claim is measured, not argued.** The six T3 evidence frames
re-captured on the T5 tree are **byte-identical to the committed T3 set**
over the client interior — across two days and two builds.

**And for the first time in the phase, a seam was observed working.**
[plan.md](./plan.md) §T10 predicted that T4's aware-plus-correction state
would go from **9 tiles per row to 7** once the inbound seam landed. It
reads 7; removing the inbound division puts it back to 9; and with the
node cache throwaway-seeded the tree fills the client instead of 1/1.25 of
it. That last state is what makes the outbound half visible too, and it is
also the reminder that **T5's shipped state renders at 80% of a 125%
client** — correct, because T6 owns the walk that writes the cache.

- [x] **Inbound, client extent** (audit rows 1–2): `wnd_proc`'s `WM_SIZE`
      client extent and `set_root`'s `GetClientRect` divided by `s`
      before reaching `run_layout_as_window_root`. **Row 2 covers two
      sites** (T1 finding F-1): `set_root` and
      [`emit::flush_layout`](../../../../wasamo-runtime/src/emit.rs),
      the reactive drain's Phase 2 layout pass, which performs the same
      `GetClientRect` → `run_layout` conversion after every
      size-affecting property write.
      Both sites convert through `DipScale::pair_to_dip`, not by
      hand-written division (T2 finding F-15).
- [x] **Inbound, pointer** (audit row 3): `WM_MOUSEMOVE` /
      `WM_LBUTTONDOWN` / `WM_LBUTTONUP` coordinates divided by `s` at the
      window procedure, so hit-testing and hover run in DIP —
      `pair_to_dip` again; inbound has one form because positions and
      extents alike are a componentwise division there.
- [x] **Inbound, readback** (audit row 9): `visual_rect`'s
      `Visual.Offset` / `Visual.Size` readback divided by `s` alongside
      the pointer. Record honestly in [log.md](./log.md) that the two
      conversions **cancel today** — hit-testing sources its geometry
      from the visual tree — and that they stop cancelling the moment
      M4-Phase 2 sources geometry from layout or introduces a
      DIP-denominated hit-area rule.
      **Say *whose* scale divides it, because the comparison mixes two
      sources** (named at T4 after reading the landing site; the row says
      only "÷ s"). The pointer is divided at `wnd_proc` by
      `WindowState::scale`, while `visual_rect` is called from
      `hit_test_click_inner` / `update_hover_inner`, which stand on a node
      and would naturally divide by the **node cache**. The two agree
      once T6's walk runs and are both 1 before it — but they are not the
      same variable, and a node the walk never reaches (F-24's
      direct-hosting path) makes them disagree silently. `visual_rect` is
      a free function taking a `SpriteVisual` with no scale in hand, so
      the division happens at its two call sites either way; the decision
      is which scale those sites read, and it is recorded rather than
      fallen into.
      **Decided: the traversal root's scale — one divisor for the whole
      traversal**, landed as `WidgetNode::visual_rect_dip`. The readback is
      a *parent-relative* value that the traversal accumulates, so the
      composited absolute position is `Σ(local_dip × scale_i)` while
      per-node division produces `Σ local_dip`, which matches the pointer's
      space (`absolute_physical ÷ window_scale`) only if **every** node's
      scale is the window's. One divisor needs only the **root's** — and
      the mixture is reachable through F-32's path list, where a node
      attached to an already-attached tree keeps the constructor identity.
      The root's cache is what the traversal has, because it holds no
      window (T1's carrier decision) and the walk starts there.
      **This is a precondition on the public entry, not an invariant the
      runtime maintains.** `hit_test_click` / `update_hover` take the
      divisor from the receiver, so entering on a **subtree** uses that
      subtree's cache against a pointer divided by the window's, and
      `togglebutton_runtime_integration.rs` enters that way. `scale` is
      private, so a caller cannot supply the right divisor even knowingly.
      Every *production* caller enters on `WindowState::root_widget`.
- [x] **Introduce the node-side scale cache**, defaulted to 1 in every
      `WidgetNode` constructor — as `DipScale::default()`, which is the
      identity, rather than a hand-written literal (T2 finding F-16). T5
      is its first reader; T6's walk is its only writer, so between T5
      and T6 it is permanently 1 — the same identity world every other
      conversion lands into.
- [x] **Record the direct-hosting path as a stated limit** (T3 finding
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
- [x] **Fix `emit::flush_layout`'s layout entry, as its own commit**
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
      **Landed as `7b23854`, the second commit**, with the before/after
      pair in [evidence/t5-after/](./evidence/t5-after/) and
      [evidence/t5-f23-after/](./evidence/t5-f23-after/): the two
      post-click label-update frames differ by 30,800 of 224,224 pixels —
      the Grid-stretched Button reappearing — while `labelupdate-initial`
      (no drain yet) and all three gallery frames (root is not `Shrink`
      over a `Fill`) are identical. **The item stays reassignable**: it is
      isolated in one commit precisely so the owner can move it out
      without touching the conversion work.
- [x] **Outbound, Visual geometry** (audit rows 4–6): `sync_visuals`
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
- [x] **The offsets are not one case but two** (T3 finding F-19; this
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
      **Decided: `to_physical` per component; no already-relative pair
      operation is added.** The deciding fact is not how the call site
      reads but what a second operation would cost: the type would then
      hold two offset-converting operations, one enforcing
      convert-once-on-the-difference and one not, distinguished only by a
      name — and the wrong pick at row 4 type-checks, reads plausibly and
      is the two-rounding form. Reusing `extent_to_physical` was rejected
      separately, because its documented position-independence property
      would become a statement about a value that is not an extent. The
      temptation did not fire: `factor()` still has **exactly one**
      production call site in the workspace, T4's diagnostic string.
- [x] **Verify the unchanged rows as assertions, not omissions**: row 8
      (`SetRelativeSizeAdjustment(1, 1)` — a relation between two
      physical quantities), row 10 (`measure` returns DIP — the fact
      that carries "layout stays DIP"), row 11 (`size_sp` is DIP), row
      12 (`InsetClip` insets are all zero, and zero is scale-invariant).
      **Row 12's site list is ScrollView / Grid / ZStack**, not
      ScrollView / Grid / Box (T1 finding F-2): `WidgetNode::box_`
      installs no clip, and `WidgetNode::zstack` does. The row's
      conclusion is unaffected; the sites asserted against are not.
- [x] **Decide and record the unit of `WindowState`'s six callback
      slots** (T1 finding F-3). `resize_fn` / `mouse_move_fn` /
      `mouse_down_fn` / `mouse_up_fn` are invoked from `wnd_proc` with
      the raw message values, so this task changes their unit as a side
      effect. No ABI or Rust-native function installs them today —
      DD-004's claim is confirmed — but the unit must be stated
      deliberately (DIP, per W1), not inherited from the seam edit.
      **Decided: DIP, and the three pointer slots change from `i32` to
      `f32`.** Stating DIP while keeping `i32` would have delivered a
      truncated DIP position — physical 50 at 150% is 33.33 — the defect
      T1 rejected when it chose `f32` for the hit-test entries, arriving
      through a different door; a unit destroyed by its own type is a
      note, not a decision. Audited rather than assumed: the six slots
      have **zero installers** anywhere in the repository, and
      [architecture.md §7.5](../../../../docs/architecture.md) spells out
      a signature only for `resize_fn` and `key_down_fn`, both unchanged —
      so no spec statement is falsified and no spec edit is required.
      Recorded because "no edit needed" and "did not look" are different
      facts.
- [x] **Two things T4 left for T5 to pick up, both small and both
      auditable** (T4 findings, recorded so they are not discovered at
      the edit). `WindowState::scale` landed as `pub(crate)` — which is
      what makes `emit::flush_layout`'s row-2b division reachable without
      widening the public API, so T5 needs neither `pub` nor an
      accessor — and it carries a `#[allow(dead_code)]` forward pointer
      because T4 writes it and **T5 is its first reader**. Removing that
      attribute is part of this task; leaving it in place would silence a
      real warning for whatever comes next.
- [x] Apply the carrier / threading shape T1 decided (risk R-5): the
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
      **One qualification after the review** (finding R-2): `sync_visuals`
      reads `self.scale` as written, but the two hit-test traversals read
      it **once, at the root**, and carry it down as a private parameter —
      `self.scale` per node is the wrong divisor for an accumulated
      readback. The **public** signatures are still unchanged, which is
      what T1's decision was protecting, and the 7 test call sites are
      unaffected by the correction.

**Start gate:** traps #1 and #2. **End gate:** the **call-site audit
table** — DD-002's 13 rows, each with its classification, the source
location as landed, and the verification that closed it; the claim being
checked is "no coordinate enters or leaves outside these rows".
**Assemble the audit query independently of the diff** (T4 independent
review finding R-8). T4's query named the APIs T4 happened to use, so it
could not exclude the ones it did not — `MoveWindow`,
`AdjustWindowRect*`, `SetWindowPlacement`, `DeferWindowPos` — and the
reviewer, not the author, is who ran the widened search. A query derived
from what was written cannot falsify what was forgotten, and on T5 that
matters more than anywhere else in the phase, because completeness *is*
this task's artifact. Enumerate the coordinate-carrying API surface
first, then search for all of it.
**Done at the start gate, before a line was edited** ([log.md](./log.md)
§T5 §The coordinate-carrying API surface), so the query demonstrably could
not have been assembled from the diff. The result the diff could not have
suggested: `MoveWindow`, `AdjustWindowRect*`, `SetWindowPlacement`,
`DeferWindowPos`, `GetWindowRect`, `ClientToScreen`, `ScreenToClient`,
`MapWindowPoints`, `GetCursorPos`, `GetSystemMetrics*`,
`MonitorFromWindow`, `SetScale`, `SetTransformMatrix`, `SetCenterPoint`,
`SetAnchorPoint`, `SetRelativeOffsetAdjustment` and `SetTransform` appear
in **no** `.rs` in the repository, and the one `StartAnimation` animates
`"Color"`.
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

**Responsibility re-audit at task start (2026-07-30), corrected after
independent review.** The planning-time shape — one fallible walk that writes
each node's scale and immediately rebuilds its brush — is not safe enough to
be T6's contract. The first implementation replaced it with an all-or-none
prepare-then-commit walk, but the review demonstrated that this still used one
cache for two different facts: the scale at which geometry must be projected
and the DPI at which a text brush was last rasterized. It also let a WinRT
surface failure prevent the whole geometry pass and let production layout
entries infer their target from the root copy rather than receive the
authoritative `WindowState::scale`. T6 therefore owns the stronger boundary:

1. production window-layout callers pass the authoritative target explicitly;
   geometry projection does not infer it from any node cache;
2. the infallible geometry-scale commit and fallible text refresh are separate
   operations, so surface failure cannot prevent the layout / `sync_visuals`
   pass; and
3. text freshness is tracked against the DPI at which each text-bearing node
   was actually rasterized, independently of the geometry cache, so a partial
   refresh remains retryable after geometry has advanced.

The production layout boundary deliberately covers the direct tree-mutation
API and the IR conditional / `for` paths without adding a second
scale-propagation rule to each mutator. It projects the whole tree at the
caller's target, commits that geometry scale, then refreshes stale text from
its independent raster marker. Initial `set_root` still prepares the new tree
before detaching the old root. It does **not** make
`lib.rs::window_add_widget` supported content hosting: that entry runs no
layout, retains no widget in `root_widget`, and is already documented as a
direct-Composition path. T6 keeps that stated limit rather than giving a
renderer side effect to a path whose contract is specifically "no layout".
This revision keeps `sync_visuals` as the runtime's only Composition geometry
writer. A failed refresh leaves geometry coherent at the target and only the
failed text marker stale; the next eligible refresh retries it.

- [x] Allocate the drawing surface at **`ceil(dip × s)` pixels** on each
      axis, through T2's named rule — `DipScale::surface_pixels`, which
      returns a `(u32, u32)` pixel count. Two consequences of the landed
      signature (T2 finding F-14): `CreateDrawingSurface` takes an `f32`
      `Size`, so either cast at the call or move to
      `CreateDrawingSurface2`'s `SizeInt32` — DD-002's contract is the
      pixel count, not the API pair — and **remove `draw_text`'s existing
      `width.max(1.0)` / `height.max(1.0)`**, because the one-pixel floor
      now lives inside `surface_pixels` and leaving the old clamp would
      put the same rule in two places.
- [x] Set the D2D device context to **`96 × s` DPI** after `BeginDraw`,
      so `create_text_layout`'s `max_w` / `max_h` stay DIP and
      `size_sp` stays a DIP font size while rasterization and hinting
      happen at device resolution. This is the phase's **only** legitimate
      use of `DipScale::factor()` in place of a named operation (T2
      finding F-15): T2 deliberately did not wrap it, because it carries
      no rounding contract and wrapping it would put a DirectWrite
      concern inside a type whose value is having no rendering
      dependency.
      **T4's carrier reversal makes this exact and removes the `factor()`
      use entirely — decide whether to take it.** `DipScale` now retains
      the DPI, and `96 × (dpi / 96)` *is* `dpi`: the value D2D wants is
      the OS-reported DPI itself, with no multiplication and no `f32`
      round trip. Exposing it (a `d2d_dpi()` or `dpi()` accessor) would
      close the one hole F-15's carry-forward has to leave open, and T2's
      stated reason for not wrapping `96 × s` — that it carries no
      rounding contract — argues *for* this rather than against it, since
      there is now no arithmetic left to wrap. T6 decides and records
      which, because the alternative is defensible too: an accessor named
      after DirectWrite's need is the rendering dependency T2 kept out.
- [x] **Convert the atlas origin** (risk R-3): `BeginDraw`'s offset is in
      pixels and must be divided by `s` before use as the D2D drawing
      origin — `to_dip`, one component each. Write it deliberately — the
      offset is frequently `(0, 0)`, so omitting it works most of the
      time and displaces text within its own surface intermittently.
      **"Frequently `(0, 0)`" is generous, measured** (T5 finding F-33).
      T5 instrumented `draw_text` to print the offset for every call and
      ran the gallery: the offsets are `(1,2)`, `(19,2)`, `(68,2)`,
      `(125,2)`, `(199,2)`, `(255,2)`, `(345,2)`, `(348,2)` … — they march
      across the atlas and **essentially none is `(0, 0)`**. On a UI with
      more than a couple of text nodes, omitting the division is wrong
      almost everywhere rather than intermittently, so the failure this
      bullet describes as "works most of the time" would in fact be
      obvious — which is good news for catching it and no reason to write
      it any less deliberately. The same run also measured that atlas
      packing is **deterministic across launches**, which is what
      disqualified it as the explanation for F-33's frame drift.
- [x] **Implement DD-M4-P1-006's accepted brush mapping.** The Visual's
      size is the exact `f32` physical
      `dip × s` and the surface is `ceil(dip × s)` pixels, so the default
      `Uniform` / `0.5` mapping resamples the larger surface and may
      offset it. DD-M4-P1-006 requires
      `CompositionStretch::None` with alignment ratios `0.0`, which keeps
      unit scale and aligns the surface origin relative to the Visual;
      storage outside the exact Visual extent is clipped on the right and
      bottom, not shown as padding. Set that mapping at every
      `CreateSurfaceBrushWithSurface` site.
      **Measure the accepted mapping with controls.** Use a
      non-proportional surface/Visual pair and compare it with the default
      so resampling and centring displacement are observable; exercise
      both an integer and a fractional device-space Visual origin so unit
      scale is not mistaken for screen-pixel alignment. `ceil` represents
      a fractional requested extent with whole texels; it does not reserve
      visible glyph overhang outside `DWRITE_TEXT_METRICS`. A visible
      overhang regression may be recorded here, but changing the Visual's
      bounds requires an accepted revision to DD-M4-P1-002 rather than an
      ad-hoc T6 fix.
- [x] The **re-rasterization primitive**: surfaces are built at scale 1
      during construction (before the tree is attached to a window) and
      brought to the window's scale by a recursive refresh. The first
      implementation treated the T5 node cache as both geometry scale and
      raster freshness; independent review falsified that shape. The corrected
      primitive tracks the last successful text-raster DPI separately, updates
      that marker only after the replacement brush is installed, and leaves a
      failed node stale without holding geometry back. The geometry cache is
      committed by its own infallible recursive operation. This is the minimum
      retained state needed to distinguish two facts that may diverge under the
      accepted log-and-survive failure policy. `WidgetData::Text { content,
      style }` uses the node's existing
      fixed DIP extent; `ButtonData` uses `label_text` / `label_style` /
      `label_size`.
      **"After the first layout" is wrong, and it is wrong in a way T5
      photographed** (T5 finding F-34). `sync_visuals` multiplies by
      **`self.scale`, the node cache**, and `run_layout_as_window_root` —
      `set_root`'s only layout pass — calls it. So a walk that runs
      afterwards updates the cache *after* the only pass that reads it,
      rebuilds the surfaces correctly, writes no geometry (rightly), and
      leaves the Visual tree at the identity projection: **a correct DIP
      layout drawn at 1/s in the corner of the client area.** That is
      exactly T5's P1 capture, which T5's own record calls "correct for T5
      alone because T6 owns the walk" — true of T5 and not true of T6 as
      specified here. **Resolved by the responsibility re-audit above:**
      initial attachment refreshes the new tree before the first layout, while
      later production layout passes receive the window target explicitly,
      project and commit geometry at that target, and refresh stale text
      independently. The refresh depends on no layout result, since it rebuilds
      from retained state and `measure` is scale-invariant (row 10). What it
      must **not** do
      is write geometry — that breaks T3's one-pass invariant and with it the
      completeness of the T5 audit.
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
      **And the primitive has the same content boundary as `sync_visuals`,
      not a wider one** (T3 finding F-24): it starts from a layout root.
      A subtree attached through `lib.rs::window_add_widget` is never walked
      and keeps text rasterized at scale 1. Same stated limit as T5's, and
      R-1's crispness claim is bounded by it: it holds for widgets the window
      owns as content.
      **The cost of a missed walk is now the rendering half only** (T5
      independent review finding R-2). Before T5's correction an
      unwalked node was rasterized at the identity **and** hit-tested at
      coordinates it is not drawn at, because the readback was divided by
      each node's own cache; the traversal now divides by one scale, so a
      mis-scaled node is at least hit-testable where it actually is.
      Recorded because it *narrows* what T6 must guarantee — the
      remaining consequence is blurred, wrongly-sized rendering, not a
      silent input mismatch.
      **The primitive's reach includes the shipped incremental paths,
      without making each one a second writer** (T5 finding F-32). T5 ran
      trap #3 as an enumeration of every
      mutator of `WindowState::scale` and every path that attaches a node,
      and it surfaced two classes the original bullet did not name — both
      shipped, neither future: **`WidgetNode::append_child` /
      `insert_child` / `replace_child` on an already-attached tree**, and
      **the IR loader's conditional and `for` mutation sites**. Each puts
      a freshly constructed node — holding `DipScale::default()` and a
      scale-1 text surface — under a window whose scale is not 1. The IR
      conditional / `for` sites call `mark_layout_dirty_for`; the direct
      Rust and ABI mutation APIs do **not** schedule or drain layout and
      retain T3's stated limit that they wait for a later `WM_SIZE` or
      size-affecting property write. The production layout callers therefore
      invoke the geometry and refresh operations as one ordered boundary; the
      geometry operation itself takes the authoritative target rather than
      recovering it from the root.
      An IR tab switch / list append is normalized in its scheduled pass;
      a direct mutation is normalized in the first later pass. The newly
      attached subtree receives coherent target-scale geometry even if its
      text refresh fails, and its stale raster marker preserves the retry. No
      mutation primitive writes a cache by itself.
- [x] Preserve public `TextRenderer::draw_text` as the 96-DPI convenience
      entry and add a crate-private device-DPI path for the runtime's seven
      scaled call expressions (the existing five plus the walk's Text and
      Button-family arms). Note the
      borrow order T1 hit: `update_button_label` must read the node's
      scale **before** `self.button_data_mut()`, which borrows all of
      `self`; `update_text_content` / `update_text_style` destructure
      `self.data` directly and need no such care.
      **The signature being threaded into is public, and the type is
      not** (T5 finding F-35). `TextRenderer::draw_text` is `pub` on a
      type [`lib.rs`](../../../../wasamo-runtime/src/lib.rs)
      `pub use`-exports beside a public `get_text_renderer()`, so this is
      a Rust-native public change in the same class as T5's callback
      slots — and `DipScale` is crate-private, so it cannot appear there
      without making the type public. Audited at task start: `draw_text` had
      **no production caller outside `widget.rs`**, so the change was free;
      the 26 `get_text_renderer()` test sites all handed the
      renderer to a constructor rather than drawing with it. **Resolved at
      task start:** nothing new crosses the public boundary. The crate-private
      path takes the `u32` DPI D2D wants; `DipScale` stays private, the factor
      accessor is not used, and existing Rust-native callers of the public
      method keep its 96-DPI semantics. T6's mock-free integration control
      now calls the public wrapper directly to pin that retained identity path;
      it is a test caller, not a new production consumer.
- [x] Confirm re-rasterization does **not** change any node's
      `SizeConstraint::Fixed(w, h)` — `measure` is DIP and unaffected by
      scale — so it cannot invalidate layout. This is the property T7
      depends on.
- [x] If the atlas-origin conversion proves fragile in practice, the
      permitted alternative is expressing the surface's resolution as a
      context transform instead of a context DPI; the contract is
      `ceil(dip × s)` pixels and device-resolution glyphs, not the API
      pair. Record the choice and the reason in [log.md](./log.md).

**Start gate:** re-run all seven decisions rather than inheriting T1's dated
selection. The responsibility re-audit makes #1 (row 7 and the seven scaled
callers), #2 (brush/cache/tree effects), #3 (the per-node cache), #4 (the
fixed-extent size branch), #5 (single-writer and scale-independent measure),
#6 (fallible surface preparation), and #7 (GUI evidence) all applicable.
**End gate:** row 7 closed in the audit table; the independent geometry/raster
markers, authoritative production target, and every incremental attach path
enumerated; the fixed-extent branch fired by a test; the layout-invalidation
non-effect verified; and
the 100% change bounded to the text surfaces that `ceil` allocation plus
DD-M4-P1-006 intentionally changes — **captured after `cargo build --release
--workspace`, never after a host-package build** (T3 finding F-21: a
host build relinks `wasamo.dll` from a **stale uplifted rlib**, so the
DLL carries a fresh timestamp and old object code, and the frame
silently shows the previous runtime).
**"Unchanged" needs a baseline, and one capture is not one** (T5 finding
F-33). Measured on an unmodified tree: three captures inside one process
are bit-identical and two captures from different launches in the same
session are bit-identical, but **the first launch of a session was an
outlier** by up to 149 of 827,904 pixels, and a settled capture differed
from the committed set of the previous day by 25. So **one capture is not
a baseline and a committed frame set is not one either**; re-capture, and
agree multiple captures on each side.
**What the residual is, measured.** T5 classified the differing pixels: in
both same-code pairs **every one was a text pixel and none flipped between
background and covered**, so the coverage mask was identical and only the
intensity of already-covered pixels moved, bounded at **13 per channel**.

**The max per-channel delta is asymmetric evidence and does not classify.**
A **large** delta proves only that the difference is **outside the drift
bound this phase measured** — not what moved, since an intensity-only
defect can exceed the bound and the drift's own mechanism is unidentified.
A **small** delta proves nothing either, for three reasons that matter
here: a rasterization defect changes intensity **without** moving
geometry, and **a wrong D2D context DPI is precisely that — this task's
defining failure would read as drift**; a sub-pixel positional error need
not flip any pixel between covered and uncovered; and contrast belongs to
the edge rather than to the change, so a geometry move between two near
colours gives a small delta.
[evidence/compare-frames.ps1](./evidence/compare-frames.ps1) therefore
**exits non-zero on any difference by default** and reports the delta as
information; `-AllowDrift` opts into treating a small-delta difference as
a pass, which is a judgement to record rather than a default to inherit.
The threshold is a measurement on this machine, not a constant. **The
mechanism behind the drift is still unidentified** — atlas packing was
instrumented and ruled out — so a difference this gate meets is a thing to
explain, not a thing to clear.

**This gate reaches one third of what T6 does** (finding F-36). At
`s = 1` the D2D context DPI becomes `96 × 1` and the atlas origin division
becomes `÷ 1`, so **both changes that buy crispness are no-ops**. The
`ceil` allocation is not: the gallery's measured surfaces are
`15.81 × 18.62`, `46.57 × 18.62`, `72.03 × 18.62` …, every one
non-integer, so the surface changes size at 100% while the Visual keeps
the exact `f32` extent — **the two stop being the same size and the
brush's mapping between them starts to matter.**
That mapping is what
[DD-M4-P1-006](../decisions/dd-m4-p1-006-surface-brush-mapping-is-set-not-inherited.md)
fixes (`Status: Accepted`): `CompositionStretch::None` with alignment
ratios `0.0`, because the default is `Uniform` with `0.5` and would scale
the larger surface down and centre it. **T6 sets those values and confirms
the accepted mapping by measurement.** The values come from Microsoft's
documentation,
not from a measurement in this repository, and this phase's rule is that
a mechanism written into an accepted record is measured.
Two related non-signals, so the gate is not over-read: **removing
`draw_text`'s `width.max(1.0)`** produces no independent visible
difference, because `surface_pixels` already applies the one-pixel floor;
and **omitting the walk entirely** renders the same as the constructor's
scale-1 surfaces at 100%. So the gate reaches the allocation and its
brush, and certifies nothing else. That is F-31's shape a second time: a
gate
that passes while the deliverable is absent, because the deliverable is
unreachable at the scale the gate runs at.
**T5 demonstrated the technique that closes it, at a cost of one line.** A
throwaway `SetProcessDpiAwarenessContext(PMv2)` in `runtime::init()`,
reverted before close, makes the scaled path observable without waiting
for T9; T5 used it for the 9 → 7 tile control, and its **P2** capture in
[evidence/t5-probe/](./evidence/t5-probe/) is already T6's before-picture
— correct geometry at 125% with visibly soft glyphs, which is R-1's
premise rendered rather than argued. Not mandated. But "the frame at 100%
is unchanged" is not evidence that text is crisp, and this is the task
where that distinction is the whole point. Full independent review before
merge.

**Review-remediation state (2026-07-30):** R1 landed as `fad59e2` with an
explicit authoritative-target geometry entry and an independent
last-rasterized-DPI marker; the mock-free stale-raster control is 3/3 green and
the two post-remediation six-frame sets are identical to the accepted success
path. The supplied review's new minors are fixed or dispositioned in
[log.md](./log.md). The full review at `eb3021e` returned zero-major / 3 minor;
the correction-only delta through `e3e878d` then received a zero-major /
zero-minor narrow verification. The owner then ran the new integration binary
in a real Compositor-unavailable Windows session: all three named tests entered
the `runtime compositor unavailable` skip path and the binary reported 3
passed. T6's implementation, evidence, retrospective, and review gates are
therefore complete. Merge remains a separate owner-approval gate.

---

### T7 — `WM_DPICHANGED` propagation

**Responsibility re-audit at task start (2026-07-30).** The list below survives
the audit against the landed T6 code — `WindowState::scale` is the
authoritative target, `run_layout_as_window_root_at_scale` is the fallible
geometry entry that commits the node cache only after a complete
`sync_visuals`, and `refresh_text_surfaces_recursive` is the independent
fallible raster pass keyed on `raster_scale`. Five things the list did not name
are added, because each is a decision T7 makes rather than one it inherits, and
four of them are reachable only by reading the arm the handler sits beside.

1. **This handler is the first `wnd_proc` re-entrancy with a live
   `GWLP_USERDATA`, so *where the arm sits* is a soundness decision.** Every
   existing arm runs inside `if !state_ptr.is_null() { let state = &mut
   *state_ptr; … }`, a `&mut WindowState` borrow that spans the whole arm set.
   T4's correction dispatches nested messages before that pointer is installed,
   so no arm has ever re-entered with one live. An arm placed inside the block
   would hold that borrow across `SetWindowPos`, and the nested frame would
   create a second `&mut` to the same object — aliasing UB, sound only by the
   accident that nothing miscompiles today. The handler therefore sits **above**
   the block and reaches `WindowState` through short-lived accesses either side
   of `SetWindowPos`, which is the same by-construction argument T4 made for
   the creation-time correction, in the one place the pointer is live.
2. **Suppressing the nested refresh is a correctness property, not fidelity to
   the ADR's step numbering.** The `WM_SIZE` arm as landed discards the geometry
   `Result` and calls `refresh_text_surfaces_recursive` unconditionally. On an
   ordinary resize the target does not move, but that does **not** make the call
   a convergence proof: it may retry an earlier rasterization failure or prepare
   a newly attached node, and it remains unconditional even if that resize's
   geometry pass fails. That is pre-existing ordinary-resize behaviour, not a
   property T7 relies on or strengthens. (`emit::flush_layout` has the same
   pre-existing shape. The standalone `run_layout_as_window_root` differs — it
   `?`s on geometry before refreshing.) Under a
   scale change the target *has* moved, so an unconditional nested refresh
   advances raster markers to the new DPI whether or not the geometry pass that
   was supposed to accompany them succeeded — exactly the convergence claim the
   T6 independent review rejected, arriving through the message loop instead of
   through a shared cache. The suppression is what makes step 4's permission
   conditional; it is not a reordering of the accepted steps.
3. **`lParam` is a raw `RECT*` taken from a message parameter.** `wnd_proc` is
   reachable by `SendMessageW` from any process, and T8 will synthesise this
   message deliberately, so a null pointer is a reachable input rather than a
   hypothetical one. Dereferencing it is the one failure in this handler that
   does not survive. Guarded, and the guard is an authored branch with its own
   direct test (trap #4), not a defensive line without one.
4. **The handler synthesises no host callback.** `resize_fn` is invoked by the
   `WM_SIZE` arm and therefore fires on the nested path and not on the fallback
   — even though the *DIP* client extent changes on both, because the physical
   extent divided by a new scale is a new DIP extent. Row 13's reasoning covers
   this: the handler synthesises no pointer message, and a synthesised resize
   notification is the same class of invention. DD-003 does not ask for one and
   the slot has no installers. Recorded because the decision has a second
   consequence: `resize_fn` becomes the only *public* observation of whether a
   nested `WM_SIZE` ran, which is what lets the fallback test assert that it
   did not.
5. **The step-3 verdict is pure logic, and two of its three states cannot be
   produced from the OS on demand.** "A whole-tree projection has succeeded" is
   the predicate that gates step 4, and it is asked twice — once to decide
   whether the fallback is required, once to decide whether text may refresh.
   It is extracted and unit-tested over its whole input space, so the arm the
   integration tests cannot reach is still fired somewhere. The mock-free
   fallback test remains the trap-#4 artifact; the unit test pins the rule.

**Closed 2026-07-30.** Landed as **one code commit** (`e63586e`) — the handler
introduces three authored failure branches, and a commit carrying them without
the tests that fire them is the state trap #4 exists to refuse. Artifacts in
[log.md](./log.md) §T7: the 13-row structural side-effect enumeration closed
against the source, the enumeration of every `WindowState::scale` access, four
mock-free integration tests plus six pure-logic tests named per branch, and a
six-mutation table. The task produced findings F-41 and F-42.

**One claim in the re-audit above came out weaker than written, and the
measurement is what weakened it.** Re-audit point 1 and this task's second
bullet both wanted the step 1 / step 2 order held by structure. What the landed
structure gives is that **no path installs the nested-pass marker against a
stale scale** — which is real but narrower. Mutation M1 inverted the two calls
and **all four integration tests stayed green**, because an unrecognised nested
pass is an unreported one, so the fallback re-projects and the final state
converges. The stale-factor projection DD-003 warns about still happens; it is
now transient rather than persistent. Propagated by proposition rather than by
string, per §Task list: the sentence falsified is *"the step 1 / step 2 order can
be made structurally impossible to get wrong"*, and the documents asserting it
were this plan's §T7, `WindowState::pending_scale_change`'s doc comment, and
`handle_dpi_changed`'s doc comment — all three corrected. **DD-M4-P1-003 needed
no change**: its "visibly wrong for one frame at best" wording remains the
accepted design warning, but T7 established only the stale intermediate
projection. Whether that projection is presented as a frame is T11 evidence.

**And one hazard the handoff predicted would stay green does not.** M5 inherited
`SWP_NOMOVE` from `realize_dip_window_size`; the nested-path test's suggested
rectangle **moves as well as resizes**, so it fails. The handoff row for T4
records that inheriting the flag "pins the window on every monitor crossing while
every test stays green" — true of a rectangle that only changes the size, and no
longer true here.

- [x] Handle `WM_DPICHANGED` in `wnd_proc` in the **fixed order**:
      (1) update `WindowState`'s cached scale from `HIWORD(wParam)`;
      (2) apply the OS-suggested rectangle from `lParam` via
      `SetWindowPos(..., SWP_NOZORDER | SWP_NOACTIVATE)`;
      (3) the nested synchronous `WM_SIZE` re-runs layout through T5's
      inbound seam; (4) re-rasterize text surfaces through T6's walk;
      (5) return `LRESULT(0)`.
      **T6's first combined primitive could not implement this order** (T5
      finding F-34, T6 independent-review R1). Step 3's nested `WM_SIZE`
      must project geometry at the new scale before step 4 refreshes text,
      but the first implementation both inferred the target from the root
      cache and made that cache the text-freshness marker. T6 corrects the
      primitive boundary before T7: the nested layout receives
      `WindowState::scale` explicitly and projects the whole tree at that
      target; geometry-cache commit is infallible and independent; step 4
      refreshes text against a separate last-rasterized-DPI marker. T7 must
      suppress the ordinary post-layout refresh during the re-entrant
      `WM_SIZE`, record completion only after that message's geometry pass
      succeeds, then invoke the refresh after `SetWindowPos` returns. If
      `SetWindowPos` fails **or no successful nested geometry pass was
      observed**, step 3 is not skipped: read the current **physical** client
      extent, convert it through `state.scale.pair_to_dip(...)`, and run the
      geometry-only entry explicitly at the same new `WindowState::scale`
      before step 4. This fallback is required even when `SetWindowPos`
      succeeds without changing the size and therefore emits no `WM_SIZE`.
      After a successful nested or fallback geometry pass, surface failure
      leaves geometry and hit testing at the new scale while the failed text
      marker remains stale and retryable.
      **No DD-M4-P1-003 successor is needed for this shape.** Its five steps
      remain in the written order and `WindowState` remains the sole
      authoritative scale; the later node cache is only a derived geometry
      copy committed from that value. Moving the cache write into step 1 or
      moving the whole refresh before nested layout would change the accepted
      mechanism, but both alternatives are rejected here because an explicit
      target lets the original decision yield the shipped behaviour without
      either change.
      **Steps 3 and 4 are now assertions rather than descriptions** (T5).
      Step 3's inbound seam exists and divides by `WindowState::scale`, so
      "the nested `WM_SIZE` re-runs layout in DIP" is a statement about
      landed code. And step 4 is not merely fourth in a list: T5's trap-#3
      enumeration makes **refreshing the node caches the discipline that
      keeps a derived copy correct**, so a handler that updates
      `WindowState::scale` without running the walk leaves every node
      converting by the previous factor — a parallel-data defect, not a
      missed optional step (finding F-32).
- [x] Encode the reason for step 1 preceding step 2 structurally, not as
      a comment: `SetWindowPos` dispatches `WM_SIZE` **before it
      returns**, so a scale updated afterwards would leave that pass
      laying out and projecting with the stale factor. This is the
      phase's single most likely ordering defect and is invisible at
      100%. **T4's throwaway probe is the technique that fits this, and
      T7 has the problem in a sharper form** (finding F-31): T7's close
      artifact is an enumeration, which is a description, and the task
      lands before T8 drives `s ≠ 1` — so nothing in T7's own gate can
      distinguish the right order from the wrong one. T4 answered the
      same shape by instrumenting, building, running, printing the event
      order and reverting; the mechanics and the capture script are in
      [log.md](./log.md) §T4 and [evidence/](./evidence/). Not mandated —
      T7 may find a structural argument that makes the order impossible
      to invert, which is better — but "the enumeration says the order is
      right" is the outcome this bullet exists to refuse. **The premise is now measured rather than inherited** (T4):
      at 125% a size-changing `SetWindowPos` dispatches
      `WM_WINDOWPOSCHANGING`, `WM_GETMINMAXINFO`, `WM_NCCALCSIZE`,
      `WM_WINDOWPOSCHANGED`, **`WM_SIZE`**, then `WM_GETICON`, all before
      it returns — and at 100% it dispatches **no `WM_SIZE` at all**,
      because the size does not change. The second half is the sharper
      fact: this ordering defect cannot be produced, let alone observed,
      before T9.
- [x] **Do not inherit T4's flags, and do not reuse its helper**
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
- [x] Apply the suggested rectangle (do not ignore it): it preserves the
      window's logical size across the change, which is what the DIP
      contract means.
- [x] **Failure handling:** log and survive. Track successful nested geometry,
      not merely entry into `WM_SIZE`. If `SetWindowPos` fails or returns
      without such a pass, obtain the unchanged/current physical client
      extent, convert it to DIP with the new `state.scale`, and retry step 3
      through the geometry-only entry with that same explicit target. Only a
      successful whole-tree geometry pass permits step 4 to advance text
      markers. `sync_visuals` writes Visuals incrementally and commits node
      caches only after the full traversal succeeds: if both the nested
      attempt and fallback geometry fail, log the error, do not refresh text,
      and retain the honest possible state of partially updated Visuals plus
      stale geometry caches / raster markers rather than claiming convergence.
      A failed re-rasterization after
      successful geometry leaves a surface at the old resolution — visibly
      blurry and retryable; a failed `SetWindowPos` leaves only the outer
      rectangle unchanged. None of these failures tears down the window;
      `wnd_proc` returns `LRESULT(0)` regardless. The runtime is **not** put
      into `Diverged`, which is for reactive-engine divergence. Add a direct
      test that fires the no-nested-geometry fallback; a success-path handler
      test does not cover this authored branch.
- [x] `WM_GETDPISCALEDSIZE` is **not** handled this phase — recorded as
      forward exposure, not an omission.
- [x] **Place the arm above the `&mut *state_ptr` block** and reach state
      through short-lived accesses either side of `SetWindowPos`, per re-audit
      point 1. The nested frame needs its own `&mut` to lay out; what must not
      exist is an outer one alive at the same time.
- [x] **Guard the null suggested rectangle**, per re-audit point 3: log, skip
      step 2, and let step 3's fallback carry the change. The scale is already
      committed at that point, so a malformed message still converges rather
      than leaving the authoritative value ahead of every projection.
- [x] **Extract the step-3 verdict as pure logic** and unit-test its three
      states, per re-audit point 5.
- [x] **The test set, and what each member discriminates.** A mock-free
      Windows integration binary driving real `WM_DPICHANGED` messages through
      `SendMessageW` at a real window. The four cases are not four samples of
      one path:
      1. *changed rectangle* — the nested path, and the **control that makes
         case 2's negative assertion mean something**: it establishes that
         `resize_fn` fires at all.
      2. *unchanged rectangle* — `SetWindowPos` succeeds, the size does not
         change, **no `WM_SIZE` is dispatched** (T4 measured exactly this), so
         `resize_fn` does not fire and only the fallback can have produced
         target-scale geometry. This is the trap-#4 artifact.
      3. *null rectangle* — the guard from the bullet above, firing the
         fallback through a second entry condition.
      4. *both projections fail* — `VStack { Text, HStack { Box } }` reaches
         `LayoutError::BoxNoExtent` deterministically, because `measure_vstack`
         passes an infinite child height and `measure_hstack` an infinite child
         width, so the childless `Box` is measured against both. The nested pass
         and the fallback then both fail and step 4 must be **denied**: the
         Text node's surface stays at its old size and
         `reactive::runtime_health()` stays `Healthy`. This one is
         discriminating against the *pre-existing* shape rather than against a
         hypothetical one — today's arm would refresh the text regardless.
      `WindowState::scale` itself is asserted only indirectly, through the
      geometry the projection produced: the `#[doc(hidden)] pub` accessor F-29
      names belongs to T8, and adding it here would widen the surface for an
      assertion T8 needs and T7 does not.
      **A new test binary re-opens the negative-guard obligation.** The shared
      `run_on_owning_runtime_thread_or_skip` helper is unchanged and already
      verified, but T6's review classified per-binary observation as the
      requirement (round 1 R3), so the skip path of *this* binary must be seen
      firing on a Compositor-unavailable environment before T7 lands. Owner
      action, same as T6's.
- [x] **Row 10's site list is ScrollView / Grid / ZStack**, not
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
- [x] **Row 7 is now literally true and should be asserted, not
      inherited.** DD-003 row 7 says the Button label Visual's offset and
      size are covered "because DD-002 moved that write into the sync
      pass". T3 performed that move, so the assertion T7 makes is that
      the label follows a scale change through `sync_visuals` with no
      handler-specific code — the row's own stated reason for not being
      the phase's silent bug.

**Start gate:** trap #2 (the phase's primary side-effect surface), trap #4
(the no-nested-geometry fallback is an authored failure/size branch), and
trap #5 — **re-run at task start rather than inherited**, and the re-audit adds
trap #3 (the node geometry cache and the raster markers are both derived from
the value step 1 writes, and the handler is the first thing that moves it) and
trap #6 (the message loop and a live `GWLP_USERDATA` pointer are where a
symptom is most tempting to take at face value). The selection and the reasons
for the two traps still judged non-applicable are recorded in
[log.md](./log.md) §T7. **End gate:** the **structural side-effect
enumeration** —
DD-003's 13 rows, each stated as updated or verified-unchanged. Rows
9–13 (`SetRelativeSizeAdjustment`, clip insets, signal registry /
effect graph / binding state / widget pointers, `MUTATION_CAP` and drain
accounting, hover and press state) must be verified as unchanged, not
assumed: a scale change must not enter the reactive drain at all. Full
independent review before merge. The trap-#4 artifact names and fires the
fallback test directly.

---

### T8 — Windows integration evidence (mock-free, CI-gated, fail-not-skip)

Placed **before** T9 on purpose: it drives `s ≠ 1` synthetically, so the
sequencing thesis does not defer all scaled-path risk to the end
(risk R-4).

**Responsibility re-audit at task start (2026-07-31).** The list below
survives the audit against the landed T5 / T6 / T7 code, and four things it
did not name are added — three of them reachable only by working out what
the assertions would actually read.

1. **"T8 chooses the rectangle, so it can assert equality rather than a
   tolerance" is not true of the window T8 is handed** (finding F-44). The
   claim is right in principle and unreachable in practice from the created
   window: `wasamo_load_ui` asks for 800 × 600 DIP and the client extent that
   produces is 784 × 561 physical at 96 DPI (T4, measured). Preserving the
   **DIP** client extent means multiplying the physical client by `dpi / 96`,
   and 561 × 1.25 is 701.25 — not an integer, so no synthesised rectangle
   holds the DIP extent exactly, and the same is true at 144, 192 and 100 DPI.
   The fix is one step and it makes every factor exact at once: **normalise
   the physical client to a multiple of 24 before the change**, because
   `96 = 2^5 × 3` and the four DPIs under test contribute denominators 4, 2, 1
   and 24. T8 therefore sets the window rectangle once at 96 DPI, asserts the
   realised client, and only then synthesises the message. The alternative —
   an approximate invariance claim with a stated tolerance — is what T11
   already carries for the OS-shaped rectangle, and having both halves
   approximate would leave the phase with no exact statement of the property
   at all.
2. **The outer rectangle is derived from a measured frame, not from the
   scale.** T8 must supply an *outer* rectangle while what it controls is the
   *client* extent, and the two differ by the non-client frame, which scales
   by its own DPI-indexed metrics rather than by `s` (T4 finding F-28). Below
   T9 the process is unaware, so the frame is the 96-DPI one and does not move
   when a synthesised message claims a new DPI — but that is a prediction, so
   the frame is measured as `GetWindowRect − GetClientRect` and the realised
   client is asserted afterwards rather than assumed.
3. **The two halves of the ADR's evidence item (2) are the same sentence
   until an independent witness is added** (finding F-45). "The DIP layout
   results are unchanged" and "the Visual offsets and sizes moved by the
   ratio" are one claim, not two, whenever the before-state is `s = 1`: the
   only reading of a DIP layout result the runtime offers is a Visual read
   back and divided, so `after = before × ratio` and `after ÷ ratio = before`
   are the same equation. What separates them is a **discrete** consequence of
   layout — the WrapPanel row assignment DD-002 and §T10 already name as the
   9-tiles-vs-7-tiles signature. An implementation that treats physical pixels
   as logical changes the row count; one that scales correctly cannot. T8's
   invariance witness is therefore the row structure, and the ratio assertion
   rides beside it as a second fact of a different shape (T7's F-42
   carry-forward).
4. **Two `#[doc(hidden)] pub` seams, not one.** F-29 named the scale
   accessor. The mixed-scale hit-test bullet needs a second one — a way to
   set a single node's cached geometry scale stale *after* geometry exists —
   and the bullet already says so; it is listed here as a deliverable because
   it is a public-surface addition and therefore a decision rather than an
   implementation detail.

**The binary is new rather than an extension of T7's** — decided at the
re-audit. `dpi_change_propagation_integration.rs` fires T7's authored
branches and says in its own header what it is and is not; a scale matrix and
a hit-test property landing inside it would blur both, and the ADR's evidence
item (2) is easier to cite as a named artifact. The cost is real and is
accepted: a new test binary re-opens the per-binary
Compositor-unavailable observation (T6 round-1 R3), which is an owner run and
a landing blocker, exactly as it was at T6 and T7.

- [ ] **The two test seams**, in [`lib.rs`](../../../../wasamo-runtime/src/lib.rs)'s
      `ffi` module and on `WidgetNode`, in the established
      `__install_owning_thread_for_test` shape. Both take or return a `u32`
      DPI rather than a `DipScale`, so the carrier stays crate-private —
      the same resolution T6 reached for
      `__run_layout_as_window_root_at_dpi_for_test`. Widening
      `WindowState::scale` to `pub` is the wrong fix: it would put the scale
      factor on a `pub use`-exported type and ship the host-visible surface
      DD-004 declines.
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
      **Hold the *client* extent constant across the change, and say so**
      (T4 finding F-28, and the T4 independent review's finding R-2 that
      the correction had not reached this bullet). "Unchanged DIP layout
      results" is exactly true only while the DIP extent handed to layout
      is exactly preserved, and the **outer** rectangle preserving its
      logical size does not preserve the client one: measured at T4, the
      non-client frame scales by its own DPI-indexed metrics, so an
      800 × 600 DIP outer request yields 784 × 561 DIP of client at 96
      DPI and 785.6 × 562.4 DIP at 120 DPI. T8 synthesises the message
      and therefore **chooses the rectangle**, so it can preserve the
      client extent and assert equality rather than a tolerance — which
      is the stronger test and the one that isolates the handler. What it
      must not do is assert exact equality against an
      OS-shaped rectangle and call that the same claim.
      **"Client extent" now has two readings and they are opposite** (T5).
      Before the inbound seam landed, the client extent layout received
      *was* the physical one; since T5 it is `physical ÷ s`. What T8 must
      hold constant is the **DIP** client extent, which means the
      synthesised rectangle's *physical* client must move **by the scale
      ratio**. A test that holds the physical client constant asserts the
      opposite of the intended claim: the DIP extent would change by the
      ratio and the layout results with it.
      **And the invariance half needs a witness the ratio half cannot
      produce** (re-audit point 3, finding F-45): with the before-state at
      `s = 1` the two halves are algebraically one claim, so the DIP-side
      assertion is the **WrapPanel row assignment** — which tile sits on
      which line — and not a number derived from the Visual geometry the
      ratio assertion already reads. The fixture is sized so the row break
      is the 9-vs-7 signature §T10 records: 12 tiles of `item-cross-size: 88`
      at `item-spacing: 12` in a 720 DIP client give 7 per row, and a client
      that grew by the ratio because the inbound seam was missed gives 9.
- [ ] Exercise at 125% / 150% / 200% — but **not as three equal probes**
      (T2 finding F-13). At a power-of-two factor the multiplication is
      exact, so convert-once and convert-twice agree everywhere and a
      DIP round trip is exactly the identity; a brute-force search found
      no disagreeing pair at 200% at all, against a witness one ulp apart
      at 150%. 200% is therefore a magnitude check, and **the rule
      verification is carried by 125% and 150%**. Adding more round
      factors would not help; adding an awkward one would.
- [ ] **Drive at least one DPI that is not a standard scaling** (T4
      independent review finding R-1, generalised here because T8 is where
      it bites next). All three factors above come from DPIs that are
      multiples of 24, so `dpi / 96` is exactly representable in `f32` for
      every one of them — which is precisely the property that hid a
      real arithmetic defect through eleven green T4 tests until a
      reviewer asked what the *documented rule* said. T8 synthesises the
      message and therefore chooses `HIWORD(wParam)` freely, so this costs
      one more case: **100 DPI** is the measured witness (804 DIP is
      exactly 837.5 physical there, the tie the `f32` route resolved the
      wrong way). The general lesson to carry, not just the value: **a
      test suite that only uses the inputs the product is expected to see
      cannot find a rule that is wrong outside them.**
- [ ] **Record the stated limit with the test** (preamble obligation 5):
      a synthesised `WM_DPICHANGED` proves the handling path; it does
      **not** prove that crossing a real monitor boundary delivers the
      same message with a usable suggested rectangle. That half is
      T11's. **A second limit belongs beside it**, from the bullet above:
      the exact-invariance assertion holds because T8 preserves the
      client extent, and the OS's suggested rectangle preserves the outer
      one instead — so on the real path the DIP layout input moves by a
      DIP or two and invariance is approximate. T11 is where that shows,
      and it must not read as a failure.
- [ ] **Assert that a mixed-scale tree hit-tests correctly from the window
      root** (finding F-37). T5's traversal divides every `visual_rect`
      readback by the **traversal root's** scale, so a tree containing a
      descendant whose cached scale is *not* the window's still resolves
      to the rectangle the widget is actually composited at. That is the
      property one divisor gives and per-node division did not, and it
      needs a scale change driven through the handler, which is why it
      lands here.
      **Not** the stale-subtree *receiver* case: entering on a subtree
      whose cache is not the window's is a documented misuse, and pinning
      it with a test would fix a stated limit as a regression contract.
      That limit lives on `hit_test_click`'s doc comment and in
      [handoff.md](./handoff.md), with no test.
      **Constructibility is resolved by T6.** Its layout-entry primitive
      normalizes every incremental attach path F-32 enumerated before
      `sync_visuals`. A direct mutation can hold a stale, still-unlaid-out
      descendant while it waits for the API's pre-existing later-layout
      trigger, but no legitimate path leaves one stale once the mutation
      becomes geometrically observable. T8 therefore needs a
      `#[doc(hidden)] pub` seam to set a stale cache after geometry exists —
      the `lib.rs::ffi` shape F-29 already names for the scale accessor. The
      seam exists only to test the one-divisor traversal property; it is not
      a production attach path.
- [ ] **Normalise the physical client to a multiple of 24 before the
      change**, per re-audit point 1, and assert the realised client rather
      than assuming the non-client frame stayed put (point 2). The
      normalisation is an ordinary `SetWindowPos` at 96 DPI, so it drives the
      ordinary `WM_SIZE` path and needs nothing new in the runtime.
- [ ] Follow the established `0x80070005` guard pattern — **fail, not
      skip**, on a runner without Compositor capability. Any new guard
      must be shown to fire on an environment that actually lacks the
      capability before the test lands; a guard verified only on the
      happy path is not verified. **This is a new binary, so the
      observation is owed again** (T6 round-1 R3) and is a landing blocker
      closed by an owner run, not by this task.
- [ ] **Show each assertion go red against a deliberately wrong
      implementation.** T8 is where the phase's "green proves nothing"
      thesis is finally testable against real scaled behaviour, so the
      close artifact is a mutation table, not a passing run: at minimum the
      inbound client-extent seam removed (the row count must move), the
      per-node divisor restored in `visual_rect_dip` (the mixed-scale hit
      test must fail), and the outbound multiplication dropped. A mutation
      that leaves every test green is the finding, not the failure.

**Start gate:** trap #4 (each assertion fires directly, not
incidentally); re-run the selection at task start rather than inheriting
it — the re-audit adds trap #1 (two new `#[doc(hidden)] pub` seams are
public-surface call sites), trap #2 (the normalisation drives a real
`WM_SIZE` before the change) and trap #5 (the seams and the one-divisor
property are invariants a later task can trip). **End gate:** tests green
locally and in CI; the mutation table; the stated limits recorded in the
test and in [log.md](./log.md).

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
      **That check is necessary and not sufficient** (T5 finding F-33).
      Measured at the *identical commit*: two settled captures a day apart
      differ by 25 of 827,904 pixels on the gallery frames, and a
      session's first launch differed from its own second and third by
      149, at up to 13 per channel. Every differing pixel is a text pixel and none flips between background and covered, so the coverage mask is identical and only intensity moves â but "tile-label glyph antialiasing" was wrong twice over: a **button** label is in the set, and antialiasing is where the pixels are, not an established cause. So a
      reused frame can show a regression that does not exist even when the
      commit matches. **Re-capture, and agree multiple captures on each
      side of the change.**
      **What the residual tells you, and what it does not**, measured. The
      drift is **intensity-only on already-covered text pixels, bounded at
      13 per channel**, with the coverage mask unchanged. A **large** max
      per-channel delta proves only that the difference is **outside that
      measured bound** — not what caused it, since an intensity-only
      defect can exceed it and the drift's own mechanism is unidentified.
      A **small** delta proves nothing either. **The number does not
      classify; it only says which side of a measurement you are on.**
      [evidence/compare-frames.ps1](./evidence/compare-frames.ps1) exits
      non-zero on any difference and reports the delta as information.
      **Control A must not lean on it.**
      Crispness is a **glyph-shape** judgement, made by looking at the
      magnified pair — stems, counters, fringing — and a pixel count
      cannot stand in for it in either direction: neither a large delta
      nor a small one tells you whether text got sharper.
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
      non-client frame scales by its own **DPI-indexed system metrics**
      rather than by `s` — decomposed with `GetSystemMetricsForDpi` at
      the T4 review: width is `2 × (SM_CXSIZEFRAME + SM_CXPADDEDBORDER)`
      = 16 at 96 DPI and 18 at 120 DPI, height adds `SM_CYCAPTION` for
      39 and 47. (Those are the probe machine's theme metrics; the
      invariant is that they are DPI-indexed, not the specific numbers.)
      Layout receives the client extent, so a correct implementation lays
      out into ~1.6 DIP more width at 125% and a wrap position near a
      line-break boundary may legitimately move. A control that demands
      identical wrap positions can therefore fail a correct build. State
      the tolerance, or drive both captures from a controlled **client**
      size rather than a controlled outer size.
- [ ] **Positive control C, path form.** Two frames across a display
      setting scale change on the development machine while the window
      is up, showing text still crisp and the logical layout unchanged
      — **to the same tolerance control B carries, not bit-exactly**
      (T4 delta review finding 2; this bullet read as an absolute while
      the bullet above it already conceded the drift). A display-setting
      change is the real path, so the window follows the OS-suggested
      rectangle and the client extent moves with the non-client metrics.
      Element order and wrap structure are the invariants; a single wrap
      position sitting on a boundary is not.
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
      **Measured at T5, so this stops being a prediction.** With the
      inbound seam landed and a throwaway declaration, the same state
      reads **7**; removing the division puts it back to 9. Two further
      readings T10 should expect rather than discover: at T5 the tree
      occupies **1/1.25 of the client** (785.6 × 562.4 of 982 × 703),
      because the outbound writes multiply by the *node* cache and T6's
      walk is its only writer — so a capture taken between T5 and T6 looks
      small and is correct (**and if it still looks small after T6, the
      walk ran after the layout that reads the cache** — finding F-34,
      folded into §T6 and §T7); and with the cache seeded the tree fills the
      client at 7 tiles while the glyphs stay soft, which is R-1's premise
      rendered rather than argued. Frames: [evidence/t5-probe/](./evidence/t5-probe/).
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
      **Cross-tree and mutation comparisons also isolate their artifact
      directories** (T6 finding F-40). A parent worktree and the task tree
      must not write the same cargo target, and any deliberately mutated
      build finishes with `cargo clean -p wasamo-runtime --release` plus an
      accepted-source workspace rebuild before the final capture. T6 caught
      cargo reporting the task runtime fresh after the shared target had
      just built the parent; a fresh DLL timestamp cannot distinguish this
      case either. A byte-identical restored frame distinguishes the accepted
      build from the specific render-changing mutation, not from every possible
      source tree: render-neutral mutations require the clean/rebuild record or
      another structural/source artifact.
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
      **A second correction belongs in the same pass** (T5 finding F-33):
      the note is where the frame-reuse procedure lives for later phases,
      and "check the commit the evidence was captured at" is measurably
      not sufficient — the baseline must come from two agreeing captures
      in the same session as the comparison. The same proposition is
      stated in the ADR set's verification-closure item 3 and in
      [constraints §9](../requirements/constraints.md); **whether either
      of those needs a dated annotation is an owner decision** raised in
      [log.md](./log.md) §T5, not T12's to take.
      **A third correction is T6 finding F-40:** cross-tree baselines use
      separate cargo target directories, and mutation evidence ends with a
      package clean plus accepted-source rebuild. Timestamp freshness and an
      unqualified cargo "fresh" result are not source-identity evidence when
      two source trees reused one artifact directory; neither is frame identity
      general source-identity evidence when the mutation could be render-neutral.
- [ ] **Four Moment 2 divergence items named at T5**, so they are folded
      into the pass above rather than found during it. (i)
      [architecture.md §12.4](../../../../docs/architecture.md#coordinate-spaces)
      says the atlas origin "is frequently `(0, 0)`, so omitting it" works
      most of the time; measured on the gallery, essentially no surface
      lands at `(0, 0)`. The normative statement is a general claim and
      the spec is the document Moment 2 exists to reconcile — so it is
      corrected there, not by an implementation task. (ii) The
      frame-reuse procedure above, if the owner's answer puts it in a
      spec or note rather than only in this plan. (iii) **Added after the
      T5 round-4 review** (finding 2):
      [architecture.md §12.4](../../../../docs/architecture.md#coordinate-spaces)
      says "the Visual carrying the surface brush has the exact `dip × s`
      device size, so surface texels map one-to-one onto device pixels.
      Crispness follows from those two numbers agreeing, not from a
      filtering mode" — while the same section allocates `ceil(dip × s)`,
      which is what stops them agreeing. The requirement is right and the
      explanation is not; the brush mapping must be set rather than
      inherited. **Closed by accepted design**:
      [DD-M4-P1-006](../decisions/dd-m4-p1-006-surface-brush-mapping-is-set-not-inherited.md)
      (`Accepted`) supersedes the default-mapping and transparent-padding
      mechanism sentences, and **§12.4 now states `None` / `0.0` as the
      accepted design**. T12 re-verifies the spec against the landed T6
      implementation instead of re-deriving the rejected default. (iv) **Added after the
      T5 independent review** (finding R-1):
      [architecture.md §12.3](../../../../docs/architecture.md#coordinate-spaces)
      states the inbound client-extent seam as "at window attach and on
      every window-resize message", which **omits the reactive drain's
      layout pass** — the third site, on the busiest path in the runtime,
      landed as audit row 2b. The spec understates the seam class it
      defines, and the same omission is in DD-002's row 2 (whose ADR-side
      handling is an owner decision, raised in [log.md](./log.md) §T5).
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

- [ ] **Decide whether `workflow.md`'s status vocabulary needs the
      supersede-vs-annotate distinction** (raised at T4, owner decision
      recorded in [log.md](./log.md) §T4). T4 landed the ADR set's first
      supersede —
      [DD-M4-P1-005](../decisions/dd-m4-p1-005-unconditional-size-correction.md)
      — and, on the same DD and the same day, an in-place **annotation**
      that is deliberately not one. The boundary applied was: *supersede
      when a reader implementing the original text would not obtain the
      shipped behaviour; annotate when the decision still produces it and
      only a statement around it was too strong.*
      [workflow.md](../../../procedures/workflow.md) lists `Proposed` /
      `Accepted` / `Superseded` and has **no word for the second case**,
      so the distinction currently exists only by precedent. Decide
      whether it gets a line there, belongs in ADR authoring guidance, or
      stays precedent — a process question, hence phase-end and not T12.
- [ ] **Safety net only — confirm T7 closed DD-M4-P1-003's step-ordering
      record**, and file it here only if it did not. **The primary owner
      is T7, not phase-end**: T7 chooses the ordering shape, so the record
      can be decided at its own close, and §T7 places it with the owner
      there.
      The substance, for the audit: step 3's nested `WM_SIZE` re-lays out
      through `sync_visuals`, which reads the **node** caches, while step 4
      is what writes them — so the fixed order projects with the previous
      scale (F-34). The
      *shape* is T6's and T7's — cache-write-only into step 1, or the
      whole walk — and **the record follows the shape, at T7's close**:
      a dated annotation if the decision still produces the shipped
      behaviour, a successor if the fixed order itself changes. Unlike
      the two items above it, this one is **not** phase-end's to decide;
      it is phase-end's to check.
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
