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
including on the 125% development machine. T8 drives `s ≠ 1`
synthetically so the ordering does not defer all scaled-path risk to the
end.

Default to **one commit per task-list item** per
[AGENTS.md §Commit rules](../../../../AGENTS.md). The known exception
this phase:

- **T5** — the seam conversions change `run_layout_as_window_root` /
  `sync_visuals` / hit-test signatures together (risk R-5); intermediate
  states do not build, so the site changes and their call-site updates
  land in one buildable commit.

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

- [ ] **Read every landing file end-to-end** (not grep-sample), per the
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
- [ ] **Verify DD-002's 13-row audit table against the source** and
      record any row whose file / function has moved since ADR drafting,
      plus any coordinate-carrying path the table does not name. The
      table is the contract; a discrepancy is a finding to record, not a
      silent correction.
- [ ] **Decide and record the `DipScale` carrier and threading shape**
      (risk R-5). `run_layout_as_window_root` is called from two
      production sites and at least four integration tests with literal
      DIP extents; `hit_test_click` / `update_hover` take `i32` physical
      today. Decide once: a scale-defaulting parameter, a separate
      window-scale entry point, or scale held on the node — and whether
      existing tests keep their current signatures. **Compiler-verify**
      by making the change, building the workspace to enumerate every
      breaking call site by compiler error, recording the list, then
      reverting.
- [ ] **Decide and record where the re-rasterization walk lives** and
      what it re-creates (`WidgetData::Text { content, style }`,
      `ButtonData` / `ToggleButton` label state), confirming DD-002's
      claim that no new retained state is required.
- [ ] **Confirm or revise the sequencing thesis.** Check that T2 → T8
      each leave the workspace buildable, the test suite green, and the
      rendered output unchanged at the development machine's 125%. If
      any intermediate state cannot hold that, revise this task list
      before T2 opens.
- [ ] **Confirm the awareness-declaration site** — that `runtime::init()`
      can declare before `CreateDispatcherQueueController`, and that the
      existing `RUNTIME.get().is_some()` early return does not cause a
      second `wasamo_init` to re-declare.
- [ ] **Sharpen [preamble.md §Technical risks](./preamble.md#technical-risks-planning-time-recon-t1-sharpens)**
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

- [ ] `DipScale` value type carrying `s`, constructed from a DPI value.
- [ ] `to_physical(dip) -> f32`, `to_dip(px) -> f32`, and the rectangle
      form (position and extent converted separately).
- [ ] The `ceil` surface-allocation rule as a named operation, so T6
      calls it rather than re-deriving it.
- [ ] Unit tests, discharging **verification item 1**: conversion at
      125% / 150% / 200%; position-and-extent consistency; round-trip
      error and rounding *direction*; the `ceil` allocation contract;
      the convert-once-on-the-difference rule (that subtracting in DIP
      then multiplying differs from multiplying then subtracting, and
      that the type's API makes the former the natural call).

**Start gate:** trap #4 applies (new arithmetic branches ship with tests
that fire them). **End gate:** tests named per contract; `cargo test`
green; no production call site introduced.

---

### T3 — Button / ToggleButton label Visual writes move into the sync pass

**Behaviour-identical refactor at scale 1**, landing ahead of the scale
work so a regression in shipped rendering code is bisectable
independently of the DPI change (DD-002 risk note; preamble obligation
2). The Button label's `SetOffset(PAD_H, PAD_V)` and `SetSize(lw, lh)`
are written at construction today, where no scale exists; after this
task every Composition geometry write in the runtime happens in exactly
one pass — which is what makes T5's audit *complete* rather than
approximately complete.

- [ ] Move the label offset / size writes out of Button construction and
      the label-update path into `sync_visuals`.
- [ ] Cover `ToggleButton`'s label path in the same move (it reuses
      Button's leaf measure / arrange and carries the same label).
- [ ] Confirm `SizeConstraint::Fixed(lw + PAD_H * 2.0, lh + PAD_V * 2.0)`
      still derives from the same measurement — the move is a write-site
      relocation, not a sizing change.
- [ ] Regression gate: existing Button / ToggleButton integration
      fixtures and the gallery render unchanged.

**Start gate:** trap #2 (the write moves between passes — enumerate what
depended on it landing at construction time). **End gate:** the
side-effect enumeration; fixtures green; a rendered gallery frame
matching the pre-change frame.

---

### T4 — Per-window scale on `WindowState` + initial acquisition + DIP window sizing

Additive per-window state. Inert until T9 — with the process still
unaware, `GetDpiForWindow` returns 96, the scale is 1, and the
`SetWindowPos` correction is a no-op.

- [ ] `DipScale` field on `WindowState`, seeded from `GetDpiForWindow`
      immediately after `CreateWindowExW` returns and **before any
      layout runs**, so `set_root`'s first pass already uses the real
      scale.
- [ ] Realise `wasamo_window_create`'s DIP `width` / `height`: create at
      the requested numbers, then apply `size × s` via `SetWindowPos`
      before the window is shown. Confirm the flash-free property holds
      structurally — that creation and `wasamo_window_show` are separate
      ABI calls and no in-between path queries geometry.
- [ ] Enable the `Win32_UI_HiDpi` feature in
      `wasamo-runtime/Cargo.toml` (prerequisite for `GetDpiForWindow`;
      the awareness API itself is T9) and re-sync
      [architecture.md §4.5](../../../../docs/architecture.md) at T12.

**Start gate:** trap #5 (the per-window shape is what M4-Phase 8 will
consume; record the invariant). **End gate:** scale seeded before first
layout, verified by ordering rather than by comment; workspace green.

---

### T5 — The conversion seams

**The full-review-lane structural task.** Converts at the boundary and
nowhere else. Every conversion is the identity at `s = 1`, so the
observable behaviour is unchanged until T9 — which is what makes this
landable as one reviewed commit rather than a visible regression.

- [ ] **Inbound, client extent** (audit rows 1–2): `wnd_proc`'s `WM_SIZE`
      client extent and `set_root`'s `GetClientRect` divided by `s`
      before reaching `run_layout_as_window_root`.
- [ ] **Inbound, pointer** (audit row 3): `WM_MOUSEMOVE` /
      `WM_LBUTTONDOWN` / `WM_LBUTTONUP` coordinates divided by `s` at the
      window procedure, so hit-testing and hover run in DIP.
- [ ] **Inbound, readback** (audit row 9): `visual_rect`'s
      `Visual.Offset` / `Visual.Size` readback divided by `s` alongside
      the pointer. Record honestly in [log.md](./log.md) that the two
      conversions **cancel today** — hit-testing sources its geometry
      from the visual tree — and that they stop cancelling the moment
      M4-Phase 2 sources geometry from layout or introduces a
      DIP-denominated hit-area rule.
- [ ] **Outbound, Visual geometry** (audit rows 4–6): `sync_visuals`
      node writes, the ScrollView intermediate Visual, and the Button /
      ToggleButton label writes relocated by T3 — all multiplied by `s`,
      **converting once on the difference**: subtract in DIP, multiply
      the result. The ScrollView recursion stays entirely in DIP
      (`child_parent_abs` is `(offset.0, offset.1 - applied_y)` in DIP);
      only the two Composition writes multiply.
- [ ] **Verify the unchanged rows as assertions, not omissions**: row 8
      (`SetRelativeSizeAdjustment(1, 1)` — a relation between two
      physical quantities), row 10 (`measure` returns DIP — the fact
      that carries "layout stays DIP"), row 11 (`size_sp` is DIP), row
      12 (`InsetClip` insets are all zero, and zero is scale-invariant).
- [ ] Apply the carrier / threading shape T1 decided (risk R-5); update
      the affected integration-test call sites without pushing scale
      awareness into tests that have no business with it.

**Start gate:** traps #1 and #2. **End gate:** the **call-site audit
table** — DD-002's 13 rows, each with its classification, the source
location as landed, and the verification that closed it; the claim being
checked is "no coordinate enters or leaves outside these rows".
Full independent review before merge.

---

### T6 — Text-surface resolution + the re-rasterization walk

**The phase's hard part** (preamble obligation 3, risk R-1). Coordinates
being right does not make text crisp; an implementation that stops at T5
produces exactly the blur the phase set out to remove and passes every
test.

- [ ] Allocate the drawing surface at **`ceil(dip × s)` pixels** on each
      axis, through T2's named rule.
- [ ] Set the D2D device context to **`96 × s` DPI** after `BeginDraw`,
      so `create_text_layout`'s `max_w` / `max_h` stay DIP and
      `size_sp` stays a DIP font size while rasterization and hinting
      happen at device resolution.
- [ ] **Convert the atlas origin** (risk R-3): `BeginDraw`'s offset is in
      pixels and must be divided by `s` before use as the D2D drawing
      origin. Write it deliberately — the offset is frequently `(0, 0)`,
      so omitting it works most of the time and displaces text within its
      own surface intermittently.
- [ ] Keep the brush mapping one-to-one: the Visual's size is the exact
      `f32` physical `dip × s`, the surface is `ceil(dip × s)` pixels,
      and the at-most-one-pixel excess is transparent padding.
- [ ] The **re-rasterization walk**: surfaces are built at scale 1 during
      construction (before the tree is attached to a window) and brought
      to the window's scale by a walk run at attach. Re-creates each
      text-bearing node's surface and brush from state the node already
      holds; adds no retained state.
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
unchanged at 100%. Full independent review before merge.

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
      100%.
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
- [ ] **The integration-side positive control**: drive a scale change
      through the handler and assert that **the layout's DIP results are
      unchanged** while Visual offsets and sizes have moved by the scale
      ratio. The first half is what distinguishes a correct
      implementation from one treating physical pixels as logical —
      which would change the DIP results and, visibly, the WrapPanel
      line count.
- [ ] Exercise at 125% / 150% / 200%.
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
      as the **first act** of `runtime::init()` — before
      `CreateDispatcherQueueController`, before `Compositor::new`, before
      `TextRenderer::new`. Guard it with the existing one-shot so a
      second `wasamo_init` does not re-declare.
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
- [ ] Integration test asserting the **effective** level —
      `GetWindowDpiAwarenessContext(hwnd)` compared against
      `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` with
      `AreDpiAwarenessContextsEqual`. Assert the level in force, not that
      a particular function was called.
- [ ] **Rebuild and run all three hosts** — C, Rust, Zig — with no
      manifest asset and no build-system edit (preamble obligation 6,
      risk R-8). This is the auditable artifact for the
      declarative-host boundary claim; it must be run, not inferred from
      "we did not edit them".
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
- [ ] **Positive control C, path form.** Two frames across a display
      setting scale change on the development machine while the window
      is up, showing text still crisp and the logical layout unchanged.
- [ ] **Window measurement check** (risk R-9): a window created at
      800 × 600 DIP measures 1000 × 750 physical at 125%. Cheap,
      concrete, and the only in-phase check of DD-004's outer-window
      -rectangle claim.
- [ ] **Re-derive the capture coordinates** for later phases against the
      new coordinate space, as the evidence artifact T12's
      `verification-environments.md` revision consumes (risk R-7).
- [ ] **Deliver the runnable set to the owner's laptop** — host
      executable + `wasamo.dll` + compiled `.uic` — so T11 is one
      observation rather than a build-and-deliver task (preamble
      obligation 7).

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
      hosts build in the documented order.
- [ ] **Moment 2 doc sync — divergence correction.** Re-verify each
      Moment 1 statement against what actually landed and correct
      divergences. The statements flagged at ADR time as most at risk
      are the outer-window-rectangle claim and the font-size unit; both
      are checked against running behaviour, not assumed. Flip the
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
retrospective, and [preamble.md](./preamble.md)'s `status` flip.

**Start gate:** trap #3's documentation analogue (do not restate spec or
handoff content in derived prose — cite the owning document) and trap
#5. **End gate:** local gates green; the Moment 2 divergence corrections
recorded per statement; carry-forward recorded with re-trigger criteria.
