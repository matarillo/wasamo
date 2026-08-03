# M4-Phase 1 — Implementation log

Append-only mixed log: decisions log (mid-implementation judgments) +
CI / verification log (evidence pointers, run ids). Per-task
implementation-gate selections (start) and close artifacts land here per
[preamble.md §Implementation gates](./preamble.md#implementation-gates).

Three entry kinds this phase carries by obligation, so they are named
here rather than discovered:

- **The T5 / T6 call-site audit table** — DD-002's 13 rows, each with
  its classification, its source location as landed, and the
  verification that closed it.
- **The T7 structural side-effect enumeration** — DD-003's 13 rows, each
  stated as updated or verified-unchanged.
- **Stated limits**, recorded with their reason rather than elided: the
  synthesised-`WM_DPICHANGED` limit (T8) and the trap-#4 disposition for
  the tolerated-declaration-failure branch (T9).

---

## T1 — Pre-implementation spike

### Start gate (recorded 2026-07-28, before any edit)

Read before selecting: [plan.md](./plan.md) §T1, the ADR set (preamble +
DD-001 … DD-004), and
[implementation-gates.md](../../../procedures/implementation-gates.md).

**Trap selection.**

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes** | T1's second obligation *is* a call-site audit: DD-002's 13-row table must be verified against the source, and any coordinate-carrying path the table does not name is a finding. T1 adds no variant itself; it establishes the baseline T5 closes against. |
| 2 | Missed side effects | no | No state or structure change lands on the T1 commit. T1 *scopes* T7's enumeration but does not perform it; performing it here would fabricate an artifact for a change that has not been written. |
| 3 | Parallel/derived data drift | **yes**, documentation analogue | T1's landing artifacts are prose in this log plus revisions to [plan.md](./plan.md), both of which sit alongside the ADR set. The failure mode is restating ADR content instead of citing it, creating a second source of truth for the audit table and the risk register. |
| 4 | Untested authored branch | no | No branch, diagnostic, or reject arm lands. T9's diagnostic branch is explicitly re-decided at T9 per [preamble.md §Implementation gates](./preamble.md#implementation-gates); T1 does not discharge it. |
| 5 | Carry-forward underweighted | **yes** | The carrier shape and the walk site are invariants every task from T2 to T8 must preserve. They are recorded here with re-trigger criteria, not left as tacit context. |
| 6 | Symptom taken at face value | **yes**, low expectation | T1 builds the workspace repeatedly with throwaway edits. A build or test failure that is *not* the expected signature breakage must be root-caused, not reverted past. |
| 7 | Weak GUI evidence | no | T1 renders nothing and launches no host. |

**Review lane.** Normal. T1 lands no production code, so none of the
high-risk classes in
[gates §4](../../../procedures/implementation-gates.md) applies to the
T1 commit itself. The decisions recorded here feed T5 / T6 / T7, which
carry the full independent review.

**Planned proof obligations** (each closed below before T1 ends):

1. Per-file touch-point record for every landing file, from an
   end-to-end read.
2. DD-002's 13 rows verified against the source, with discrepancies and
   unnamed coordinate-carrying paths recorded as findings.
3. The `DipScale` carrier and threading shape, decided once, with the
   breakage set enumerated **by the compiler** — throwaway edit, build,
   record, revert.
4. Where the re-rasterization walk lives and what it re-creates.
5. The sequencing thesis confirmed or revised, task by task from T2 to
   T8.
6. The awareness-declaration site confirmed against `runtime::init()`'s
   one-shot guard.
7. Risk sharpening against source line numbers, plus the T5 and T6 gate
   selections.

**Exit criterion** (spike-specific, per
[spike discipline](../../../procedures/implementation-gates.md) and
[plan.md](./plan.md) §T1): every open point is assigned to a downstream
task **and its scope is seen** — not "no surprises expected".

**Baseline for the revert.** Throwaway edits are made on top of
`8f9e4e3` (T0 closure). `git status` must be clean of `wasamo-*` changes
at the T1 commit.

### Landing-file touch-points (end-to-end read)

Every file below was read in full, not sampled. Line numbers are as of
`8f9e4e3`.

| File | Touch-points | Task |
|---|---|---|
| `window.rs` (339 lines) | `WindowState` fields 29–51 (no scale today; six callback slots; `tracking_mouse` / `mouse_down`); `create` 57–86; `create_hwnd` 88–119 (`CreateWindowExW` at 102, `CW_USEDEFAULT` placement); `SetRelativeSizeAdjustment` 64; `set_root` 140–174 (`GetClientRect` 160, layout 171); `wnd_proc` 233–339 — six arms: `WM_DESTROY` 243, `WM_ERASEBKGND` 249, `WM_SIZE` 256–266, `WM_KEYDOWN` 268, `WM_MOUSEMOVE` 276–297, `WM_MOUSELEAVE` 299, `WM_LBUTTONDOWN` 310, `WM_LBUTTONUP` 323. No `WM_DPICHANGED`, no `WM_NCCREATE`, no `WM_GETDPISCALEDSIZE` arm exists. | T4, T5, T7 |
| `text.rs` (194 lines) | `TypographyStyle::size_sp` 39–46 (12/14/20/28); `TextRenderer::new` 61–100; `measure` 103–108; `draw_text` 111–160 (`CreateDrawingSurface` 119, `BeginDraw` + `offset` 128–130, `DrawTextLayout(origin)` 150–155); `create_text_layout` 162–193 (`max_w` / `max_h` reach `CreateTextLayout`; `size_sp` reaches `CreateTextFormat`). `draw_text` takes no scale and never calls `SetDpi`. | T6 |
| `widget.rs` (2697 lines) | Ten constructors, each ending in the same `attached / bindings` field block; `WidgetNode::text` 453–489 and `button_family` 765–850 both `draw_text` at construction; **construction-time label writes 813–818**; label-update writes 1035–1040 (`update_button_label`); `update_text_content` 1133–1165 and `update_text_style` 1167–1202 re-rasterize on a property write; `hit_test_click(_inner)` 1216–1294; `update_hover(_inner)` 1298–1355; `clear_hover` 1358–1376; `run_layout` 1571–1575; `run_layout_as_window_root` 1606–1616; `build_layout_tree` 1626–1731; `sync_visuals` 1742–1793 (node write 1749–1757, ScrollView intermediate 1776–1784, `child_parent_abs` 1785); `visual_rect` 1886–1896; `InsetClip` installs at 602–604 (ScrollView), 654–656 (Grid), 676–678 (ZStack). | T3, T5, T6 |
| `runtime.rs` (93 lines) | `init` 39–60 — `capture_owning_thread()` 40, one-shot early return 41–43, `CreateDispatcherQueueController` 49, `Compositor::new` 50, `TextRenderer::new` 51. | T9 |
| `abi.rs` (1288 lines) | `set_last_error` 136–140 and `clear_last_error` 142–144 (thread-local `CString`); `wasamo_init` 268–280 (maps `Err` → `WASAMO_ERR_RUNTIME`, clears last-error on success); `wasamo_window_create` 307–346 (`width` / `height` pass straight through to `window::create`); `wasamo_load_ui` 1172–1240 with `DEFAULT_WINDOW_WIDTH` / `_HEIGHT` = 800 / 600 at 1119–1121. | T4, T9 |
| `Cargo.toml` | `windows` 0.58 feature list 22–48; `Win32_UI_HiDpi` absent. | T4 |
| `emit.rs` (349 lines) — **not on the plan's landing-file list; added by this spike** | `flush_layout` 127–149: a second production `GetClientRect` → layout path, reached from `drain_if_outermost` Phase 2. See finding F-1. | T5 |
| `lib.rs` (140 lines) — **added by this spike** | `window_create` 79–85 / `window_set_root` 116–121: the Rust-native API duplicates the ABI entry points, so `WasamoWindow`'s DIP contract has a second public caller. | T4 |

### Findings

**F-1 — the audit table is missing a production inbound seam.**
[`emit.rs::flush_layout`](../../../../wasamo-runtime/src/emit.rs)
calls `GetClientRect` and feeds the result to `run_layout`, exactly as
`set_root` does, on every reactive-drain Phase 2. DD-002 row 2 names
only `set_root`'s `GetClientRect`. Under C3 this path takes physical
pixels and hands them to the layout engine as DIP — the same defect row
2 exists to prevent, on a path that runs after **every size-affecting
property write**. Row 2 must be read as covering both sites.
*Disposition:* T5, as row 2b. Not a table correction made silently — the
ADR is immutable and the finding is recorded here.

**F-2 — the audit table's row 12 names the wrong widget set.** Row 12
reads "ScrollView / Grid / Box `InsetClip` insets". The source installs a
zero-inset clip in `scroll_view` (602–604), `grid` (654–656) and
**`zstack`** (676–678); `box_` (507–535) installs **no clip at all**.
The row's *conclusion* (all insets are zero, zero is scale-invariant) is
unaffected — but T5 must assert against ScrollView / Grid / ZStack, and a
T5 audit that dutifully checked "Box" would be checking a site that does
not exist while missing one that does. *Disposition:* T5.

**F-3 — six window callback slots carry coordinates and have no stated
unit.** `WindowState`'s `resize_fn` / `mouse_move_fn` / `mouse_down_fn` /
`mouse_up_fn` (and `key_down_fn` / `mouse_leave_fn`, which carry none)
are `pub` on a `pub use`-exported type and are invoked from `wnd_proc`
with the raw message values. No ABI function and no Rust-native function
installs them, which is what
[DD-004 §Does the host need the scale factor](../decisions/dd-m4-p1-004-unit-contract-and-spec-wording.md)
relies on — that claim is **confirmed**. But T5 divides those same
message values at the seam, so the slots' unit changes as a side effect
of a decision that never mentioned them. *Disposition:* T5 decides
explicitly and records that they are invoked in DIP (consistent with
W1), rather than letting the unit change fall out of the seam edit.

**F-4 — the existing suite is scale-insensitive, so R-2 has no automated
defence before T8.** Every layout integration test drives `WidgetNode`s
directly and never through a window, so no test routes a coordinate
through a window's scale. Verified: with the full conversion machinery
*and* the Per-Monitor-Aware V2 declaration in place on the 125%
development machine, all 32 test binaries passed with zero failures —
the same result as baseline. R-2 therefore cannot be caught by CI at
100% **or** by the development machine at 125%; T8's synthesised scale
change is the only automated defence, which raises its weight above
"placed before T9 for sequencing reasons".

**F-5 — `cargo test --workspace` fails from a cold target directory.**
Pre-existing, unrelated to this phase, observed twice. Root cause:
[`wasamo-dll/build.rs`](../../../../wasamo-dll/build.rs) passes
`/WHOLEARCHIVE:<profile>/libwasamo_runtime.rlib`, and cargo only uplifts
that rlib to `<profile>/` once `wasamo-runtime` has been built as a
primary package. From a cold target directory the cdylib link runs
first and fails with `LNK1356` ("library specified for whole-archive not
found"); with a *stale* uplifted rlib from a previous toolchain it fails
later and more confusingly, as `LNK2019` on `core` / `std` symbols in
the test binaries. `cargo check` never linked, so it stayed green
throughout. Remedy, verified: `cargo build -p wasamo-runtime` before
`cargo test --workspace`, after which the full suite is green.
*Disposition:* T12's clean-rebuild gate must use that ordering, and
[AGENTS.md §Build ordering](../../../../AGENTS.md)'s claim that
workspace-wide builds "implicitly satisfy the ordering" needs the
cold-directory qualification. Carried to
[handoff.md](./handoff.md); not fixed by T1.

### Decision — the `DipScale` carrier and threading shape (risk R-5)

**Decided: the scale is authoritative on `WindowState` (DD-003 S2) and
cached on each `WidgetNode`, written by a single walk at attach and on
scale change. No layout or sync signature changes.**

The choice was made against a compiler-enumerated breakage set, not an
estimate. Two shapes were built, compiled and reverted:

| Shape | Production sites touched | Test call sites broken | Files |
|---|---|---|---|
| **A — thread a `scale` parameter** through `run_layout`, `run_layout_as_window_root`, `hit_test_click`, `update_hover` | 7 (`window.rs` ×6, `emit.rs` ×1) | **28** | 12 |
| **B — cache the scale on the node** (recursive passes read `self.scale`) | 7 (same) | **7** | 4 |

Shape A's 28 sites are 21 layout entries (`run_layout_as_window_root`
×13 in 6 files, `run_layout` ×8 in 3 files) plus 7 pointer entries.
Shape B breaks none of the 21: the recursive passes already carry
`&mut self`, so they read the scale from the node they are standing on
and their signatures are unchanged. The plan's estimate — "two
production sites and at least four integration tests" — undercounts the
layout entries by a factor of three and omits `run_layout` entirely;
`plan.md` §T5 and
[preamble.md §Technical risks](./preamble.md#technical-risks-planning-time-recon-t1-sharpens)
R-5 are revised to the measured numbers.

The decisive argument is not the count, though. It is that
**`set_property` has no window in hand.** `update_text_content`,
`update_text_style` and `update_button_label` all re-rasterize, so all
three need the scale, and they are reached from
`wasamo_set_property` / the reactive writer with nothing but a
`*mut WidgetNode`. Under shape A each would have to find its owning
window by the O(windows × nodes) pointer search
`emit::mark_layout_dirty_for` performs. Under shape B each reads
`self.scale`. Shape A does not merely cost more edits; it has no clean
answer on the property-write path.

Shape B was compiled green across the whole workspace and the full suite
run: **32 test binaries, 0 failures**, with the only test edits being the
7 `hit_test_click` literals.

**Sub-decision — the pointer's DIP type is `f32`, not `i32`.**
`hit_test_click` / `update_hover` take `i32` physical coordinates today.
Under H2 the entry point is DIP; keeping `i32` would truncate (physical
50 at 150% is DIP 33.33) and would make hit-test edges depend on the
scale for no benefit. `f32` costs exactly the 7 literal edits counted
above (`hit_test_click(50, 20)` → `(50.0, 20.0)`), which is why they
appear in shape B's column at all — they are the honest cost of the unit
change, not of the carrier.

**Carry-forward (trap #5).** The node's scale is a **cache with exactly
one writer** — the attach / scale-change walk — and `WindowState` holds
the authoritative value. *Re-trigger criterion:* any future path that
attaches, re-parents, or re-materialises a subtree without running the
walk (staged iteration subtrees, M4-Phase 2 event-model tree edits,
M4-Phase 8 moving a tree between windows) leaves stale scales behind and
must call the walk. Recorded in [handoff.md](./handoff.md).

### Decision — where the re-rasterization walk lives

**Decided: `WidgetNode::apply_scale_recursive(&mut self, compositor,
renderer, scale)`, called from `window::set_root` after the first layout
and from the `WM_DPICHANGED` handler.** It writes `self.scale`, then
re-creates the surface and brush for the two text-bearing shapes and
recurses:

- `WidgetData::Text { content, style }` → `measure` + `draw_text` →
  `CreateSurfaceBrushWithSurface` → `self.visual.SetBrush`.
- `WidgetData::Button(btn) | ToggleButton(btn)` →
  `btn.label_text` / `btn.label_style` → same → `btn.label_visual.SetBrush`.

DD-002's claim that no new retained state is required **holds for the
re-rasterization itself** — both shapes are rebuilt from fields the node
already carries. The `scale` field is new retained state, but it is the
carrier decision above, not a requirement of the walk; the walk would
work without it if every reader had a window in hand, which F-3's
sibling argument shows they do not.

One borrow-checker point found by building it: `update_button_label`
reads `self.scale` **before** `self.button_data_mut()`, because that
method borrows all of `self`. `update_text_content` / `update_text_style`
destructure `self.data` directly and need no such care. Scope seen; T6.

### Confirmation — the sequencing thesis

Confirmed, and its load-bearing premise was measured rather than assumed.
A throwaway probe on the 125% development machine reported:

```
T1 PROBE: GetDpiForWindow=96 cached_scale=1 pmv2=false unaware=true
```

So an undeclared process is told 96 DPI even on a scaled monitor, every
scale factor is exactly 1, and every conversion T2–T8 introduces is the
identity. With the declaration added the same probe reported
`GetDpiForWindow=120 cached_scale=1.25 pmv2=true unaware=false`, and the
whole suite stayed green — which is F-4, and the reason the thesis is
*confirmed* but its comfort is smaller than it looks: T2–T8 are green
because nothing tests them, not only because they are identities.

No task split is revised on sequencing grounds. T2 → T8 each leave the
workspace buildable and the suite green.

### Confirmation — the awareness-declaration site

Confirmed, with one wording sharpening. `SetProcessDpiAwarenessContext`
placed in `runtime::init()` **after** the `RUNTIME.get().is_some()` early
return and **before** `CreateDispatcherQueueController` returned
`Ok(())`, and the resulting effective context was Per-Monitor-Aware V2.

The placement is not free-floating. DD-001 says "the first act of
`runtime::init()`"; the accurate statement is **the first OS-touching
act, after the one-shot guard**. `capture_owning_thread()` necessarily
runs first (it is a thread-id capture, not OS work that can lock the
awareness), and the declaration must sit *below* the early return — a
declaration above it would re-run on a second `wasamo_init` and take
`ERROR_ACCESS_DENIED` on a process that had already declared correctly.
Verified: with the declaration below the guard, a second `wasamo_init`
returned `WASAMO_OK` and did not re-declare. **The existing one-shot is
sufficient; T9 adds no new guard.** *Disposition:* T9.

### Spike evidence — the T6 approach buys crispness

Not required by T1's checklist, captured because R-1 is the phase's
defining failure and the throwaway happened to contain a complete T6.
The gallery host was launched, captured (`CopyFromScreen` over
`GetWindowRect`, per
[verification-environments.md](../../../../docs/notes/verification-environments.md)
§Observation 4) and the same 200 × 26 physical status-bar region
magnified 5× from each frame:

- **Before** (unaware, DWM-stretched): window measured 1000 × 750
  physical for an 800 × 600 request — Observation 4's stated premise,
  confirmed while it is still true. Glyph stems are 2–3 px with soft grey
  fringes on both sides; counters in `e` / `a` / `o` are filled in.
- **After** (V2 declared, `ceil(dip × s)` surface + `SetDpi(96 × s)` +
  origin ÷ s): stems are clean 1–2 px, counters open, the hyphen and `I`
  are single sharp strokes.

Two further observations, both expected and both useful:

- The window measured **800 × 600 physical**, not 1000 × 750, because the
  throwaway omits T4's `SetWindowPos` correction. That is the concrete
  demonstration that `CreateWindowExW`'s arguments become physical the
  moment the process is aware, and it confirms
  [DD-004](../decisions/dd-m4-p1-004-unit-contract-and-spec-wording.md)'s
  outer-window-rectangle claim from the direction of its failure.
- The gallery's WrapPanel laid out **6 tiles per row instead of 7**,
  correctly, because a 800 px physical client area is 640 DIP wide. This
  is positive control B's signal working — and a reminder that control B
  is only meaningful once T4 makes the two windows the same *logical*
  size.

This is spike evidence, not T10's artifact: it is a single before/after
pair on one machine, and T10 owns the controls with their own capture
discipline.

### Owner decisions on the T1 retrospective's open questions (2026-07-28)

All three carried as asked; recorded here so T5 onward does not re-open
them.

1. **The pointer's DIP type is `f32`.** The `i32` alternative breaks no
   test but truncates (physical 50 at 150% → DIP 33), and the truncation
   would make hit-test edges scale-dependent. The 7 literal edits in 4
   test files are accepted as the cost of the unit change.
2. **A green suite is downgraded to a regression check for T3–T8.** It
   stays in every end gate and must not go red, but it is not counted as
   evidence that a conversion is correct — F-4 measured it green with the
   full machinery *and* the declaration in place. Each task's real gate
   is its own artifact; the substitution table is in
   [plan.md](./plan.md) §Task list.
3. **T8 keeps its position after T7.** Bringing it forward would need a
   test seam that swaps the scale without going through the
   `WM_DPICHANGED` handler — new surface, built to compensate for an
   ordering change, on the one task whose value is that it drives the
   real handler.

### Plan-hypothesis re-audit (2026-07-28, owner-prompted)

The first pass of T1's plan revision updated only the tasks T1's
findings *pointed at* — T4, T5, T6, T9, T12 — rather than re-reading the
whole task list against what the spike had learned. A second pass at the
owner's prompting found five more places where a planning-time
hypothesis had been falsified or sharpened and was still standing.
Recorded as findings because the omission class matters more than the
five items: **a spike's output is not only "what I looked for" but "what
else in the plan is now known to be wrong."**

- **F-6 — the T5 commit-bundling exception's stated reason was
  falsified.** It bundled T5 because `run_layout_as_window_root` /
  `sync_visuals` / hit-test signatures "change together". Under the
  carrier decision the first two do not change at all. The exception
  survives at the reduced scope of the two hit-test entry points and
  their 7 call sites; the rest of T5 is free to split.
- **F-7 — the DIP window-size correction must live in
  `window::create`, not `wasamo_window_create`.** T4's wording named the
  ABI function. `window::create` has three callers, and `wasamo_load_ui`
  — the path every example host takes — is not one of them via the ABI
  entry point. A correction at the named site would leave all three
  hosts at the wrong physical size, which is precisely the R-9 defect
  T10 is supposed to catch, arriving through the door the plan left
  open. **This is the one that would have shipped broken.**
- **F-8 — the flash-free property has an in-between geometry query
  after all.** T4 asked to confirm "no in-between path queries
  geometry". `window::set_root` calls `GetClientRect` for its first
  layout, and `wasamo_load_ui` calls it between create and show. The
  property still holds, but for a different reason than the plan gave:
  the correction runs inside `window::create` before it returns. The
  confirmation is now written as a consequence of F-7 rather than as an
  independent structural fact.
- **F-9 — T10's window-measurement check is not a positive control.**
  "800 × 600 DIP measures 1000 × 750 physical at 125%" is satisfied by
  the **unaware** baseline too, because DWM stretches by the same
  factor — T1 measured exactly 1000 × 750 before any change. A build
  that never declares awareness passes it. It must be read alongside
  T9's effective-context assertion or control A. T1 also measured the
  third outcome (aware, correction absent: 800 × 600 physical, WrapPanel
  7 tiles → 6), so all three states are separable once the check is
  paired.
  **Corrected in place at T4 (finding F-27):** the "7 tiles → 6" half of
  that sentence drops a condition the spike-evidence section above
  states — T1's throwaway carried the **complete** conversion machinery,
  so its client extent was divided back into DIP. With T4's correction
  absent and *only* T4's code present, the same state measures **7**
  tiles, not 6. The three-state separation still holds; the numbers are
  in §T4.
- **F-10 — `SizeConstraint::Fixed` is per axis.** T3 wrote
  `Fixed(lw + PAD_H * 2.0, lh + PAD_V * 2.0)` as a single two-argument
  constraint; the source has two one-argument constraints, one per axis.
  Cosmetic, but a T3 that copies the plan's shape will not compile.

*Disposition:* all five folded into [plan.md](./plan.md) in the same
commit as this entry. F-6 → the commit-rules exception; F-7 / F-8 → T4;
F-9 → T10; F-10 → T3. F-7 is also the reason the T3 premise ("every
Composition geometry write happens in exactly one pass") is now stated
with T1's verified write list rather than as an assertion.

### T5 and T6 gate selections (armed by T1, before T5 opens)

Recorded here rather than at T5's own start gate because
[plan.md](./plan.md) §T1 assigns the arming to the spike — the point
being that the traps are chosen while the source is fresh, not while an
approach is being defended. T5 and T6 still re-read this at their start
and may add, never silently drop.

**T5 — the conversion seams.** Review lane: full independent review.

| # | Applies | Reason |
|---|---|---|
| 1 | **yes** | The task *is* the audit. DD-002's 13 rows plus F-1's second site for row 2, F-2's corrected row-12 site list, and F-3's callback slots. The claim under check is "no coordinate enters or leaves outside these rows"; F-1 and F-3 are proof that the ADR-time enumeration was not complete, so a T5 that audits only the 13 rows audits the wrong set. |
| 2 | **yes** | Moving the seam changes the unit seen by everything downstream of it — including the six callback slots (F-3), which no one installs today and which therefore change silently. |
| 3 | no | No parallel vector, index, or cache is added. The node-side scale cache T5 introduces is written by nobody until T6 and has one writer thereafter; it is covered as trap #5, not as parallel data. |
| 4 | no | No reject, diagnostic, or size branch. Every conversion is unconditional — which is the property the whole sequencing thesis rests on. |
| 5 | **yes** | The single-writer invariant on the node scale cache is exactly the kind of ordering rule a later task trips. Recorded with its re-trigger criterion in [handoff.md](./handoff.md). |
| 6 | no | Nothing in T5 is expected to be flaky; carry it if a failure recurs. |
| 7 | no | T5's evidence is the audit table plus a green suite. F-4 says the suite proves little here, which is an argument for weighting the audit — not for manufacturing GUI evidence T10 owns. |

**T6 — text-surface resolution and the re-rasterization walk.** Review
lane: full independent review.

| # | Applies | Reason |
|---|---|---|
| 1 | **yes** | Row 7 only. `draw_text` has five call sites — two constructors (`WidgetNode::text`, `button_family`) and three property-update paths (`update_button_label`, `update_text_content`, `update_text_style`) — and the walk adds a sixth caller. A missed call site keeps a DIP-sized surface and blurs one widget kind at scale ≠ 1, which is R-1 in miniature and passes every test. |
| 2 | **yes** | The walk mutates every text-bearing node's brush. The enumeration must state what it does **not** touch: no `SizeConstraint::Fixed` changes (because `measure` is DIP), therefore no layout invalidation — the property T7 depends on. |
| 3 | no | No parallel structure. |
| 4 | no | No authored branch. The permitted `SetTransform` alternative is a substitution, not an added arm; if it is taken, the reason is recorded per [plan.md](./plan.md) §T6. |
| 5 | **yes** | The walk is the node scale cache's only writer (see T5 #5), and the "re-rasterization cannot invalidate layout" property is a carry-forward with a stated re-trigger: a scale-dependent `measure`. |
| 6 | **yes** | `CreateDrawingSurface` / `BeginDraw` / brush creation are WinRT-fallible, and the walk runs them O(text nodes) times per attach. A recurring failure here is a root-cause obligation, not a retry. |
| 7 | **yes** | R-1 cannot be discharged by any test. T6's own end gate is "local rendering unchanged at 100%"; the crispness pair is T10's, and T1's magnified before/after is the pre-evidence that the approach is sound. |

### Close gate

- **#1 call-site audit** — DD-002's 13 rows verified against the source;
  discrepancies recorded as F-1 (missing row) and F-2 (wrong widget
  set); the coordinate-carrying paths the table does not name recorded as
  F-1 and F-3. Queries: `rg` over `*.rs` for
  `SetOffset|SetSize|SetScale|SetClip|SetRelativeSizeAdjustment|CreateDrawingSurface|CreateInsetClip|ClientToScreen|SetWindowPos|CreateWindowExW|Offset\(\)|Size\(\)`
  and for
  `run_layout_as_window_root|\.run_layout\(|hit_test_click|update_hover|visual_rect|draw_text|GetClientRect|sync_visuals`.
  Every production Composition geometry write is accounted for by a row.
- **#3 documentation analogue** — the findings above cite the ADR rows
  they contradict rather than restating the table; the table itself is
  not copied into this log, and `plan.md` continues to point at
  DD-002 as the audit artifact.
- **#5 carry-forward** — the node-scale cache invariant and its
  re-trigger criterion, plus F-5's build ordering, recorded in
  [handoff.md](./handoff.md).
- **#6 deterministic-failure disposition** — F-5. Two occurrences, root
  cause identified in `wasamo-dll/build.rs`'s whole-archive path, remedy
  verified, no re-roll.
- **Revert** — every throwaway edit reverted; `git status` clean of
  `wasamo-*` changes at this commit. The two throwaway shapes and the
  probe exist only in this record.
- **Exit criterion** — every open point below is assigned and scoped:
  F-1 / F-2 / F-3 → T5; F-4 → raises T8's weight, no new work; F-5 →
  T12 + handoff; carrier and pointer type → T5; walk shape and the
  `update_button_label` borrow order → T6; declaration placement and the
  one-shot → T9.

---

## T2 — `DipScale` conversion type + pure-logic unit tests

### Start gate (recorded 2026-07-28, before choosing the approach)

Read before selecting: [plan.md](./plan.md) §T2 and its §Task list
preamble, the ADR set (DD-002 §The carrier of the arithmetic / §The
rounding contract for surfaces / §The conversion sites rows 4–7, DD-003
§`WM_DPICHANGED`, DD-004 §The unit), the T1 entries above, and
[implementation-gates.md](../../../procedures/implementation-gates.md).

**Trap selection.** [plan.md](./plan.md) §T2 pre-names trap #4; #5 is
added here (the gate permits adding, never silently dropping).

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | no | No enum, IR, or schema type gains a variant or a field. `DipScale` is a new standalone type with no traversal over it and **no call sites** — it introduces no site into DD-002's audit table, which is T5's artifact. DD-004 already records why the schema-migration gate is non-applicable phase-wide: the unit is a semantic statement about existing literals, not a new encoding. |
| 2 | Missed side effects | no | Nothing lands in any existing state or structure: no field is added to any type, no pass is reordered, no constructor changes. The derived effects this phase does carry belong to their own tasks — the `WindowState` field to T4, the node-side cache to T5, the brush rebuild to T6. Enumerating them here would fabricate an artifact for changes not yet written. |
| 3 | Parallel/derived data drift | no | No parallel vector, map, index, or cache. The related design point is answered rather than deferred: `DipScale` retains **only** the factor and not the originating DPI, so there is no second representation of the same fact to drift. The *cached* copies of the factor (on `WindowState`, on each `WidgetNode`) are T4/T5/T6's, and T1 already armed them as trap #5 with a single-writer invariant. |
| 4 | Untested authored branch | **yes** | Two authored branches ship: `from_dpi`'s zero-DPI fallback to `IDENTITY`, and `surface_pixels`' one-pixel floor (which also absorbs non-finite and negative input through a saturating cast). Each gets a test that fires it directly, not incidentally. This is also the trap [plan.md](./plan.md) §T2 names as the start gate — read there as "new arithmetic branches ship with tests that fire them". |
| 5 | Carry-forward underweighted | **yes** | The rounding contract T2 encodes is an invariant later tasks must preserve: `ceil` for the surface and exact `f32` for the Visual size (T6), and convert-once-on-the-difference (T5). DD-002 states both; what T2 adds is that the **API shape is now the enforcement**, which is a fact a later task can defeat by hand-rolling the arithmetic instead of calling the type. Recorded with its re-trigger criterion at the close gate. |
| 6 | Symptom taken at face value | no | T2's tests are deterministic pure-`f32` arithmetic with no OS surface, no window, and no timing — there is no flake source to root-cause. The one deterministic failure in reach is **F-5**'s cold-directory link error, which already has a root cause and a verified remedy; using that remedy is not a re-roll. If a *different* failure recurs, this trap re-arms. |
| 7 | Weak GUI evidence | no | T2 renders nothing, launches no host, and lands no call site, so there is no frame to capture and nothing a screenshot could distinguish. R-1's crispness evidence is T6's rendering gate and T10's control A. |

**Review lane.** **Branch/test-focused review** ([gates §4](../../../procedures/implementation-gates.md)).
T2 is not one of the high-risk classes: no schema or IR migration, no
runtime structural change (nothing existing is touched — the module has
no callers), and no GUI-render evidence. It *does* add two authored
branches, which is precisely the class §4 assigns the narrower lane, and
"no full review" is not "no review". The full-review lanes stay where T1
armed them: T5, T6, T7, T9, T10.

**Planned proof obligations** (each closed at the close gate):

1. The named operations exist for every conversion T5 and T6 will make,
   so neither re-derives the arithmetic: length, extent, relative offset,
   inbound pair, surface pixel count.
2. Verification item 1 discharged as tests: conversion at 125 / 150 /
   200%; position-and-extent consistency; round-trip error **and**
   rounding direction; the `ceil` allocation contract; the
   convert-once-on-the-difference rule, including that the type's API
   makes the one-rounding form the natural call.
3. The two authored branches each fired by a named test (#4).
4. No production call site introduced — `cargo build` shows the module
   is reachable only from its own tests, and the forward-pointer
   `#![allow(dead_code)]` names the tasks that remove it.

**Approach note recorded before coding.** The witness values for the two
`f32` claims (round-trip inexactness with a two-sided direction, and
convert-once ≠ convert-twice) were **found by brute-force search over
`f32`, not chosen by hand**, because a hand-picked pair that happens to
agree would turn each of those tests into a tautology that passes against
a wrong implementation. The search results and the witnesses they
produced are recorded at the close gate.

### The landed surface

[`dip_scale.rs`](../../../../wasamo-runtime/src/dip_scale.rs), 155
production lines plus 11 tests, declared from
[`lib.rs`](../../../../wasamo-runtime/src/lib.rs) as `mod dip_scale;`.
The named operations, one per conversion DD-002's table needs, so that no
seam re-derives a rule:

| Operation | Audit rows served | Note |
|---|---|---|
| `from_dpi(u32)` / `IDENTITY` / `Default` | T4's seeding, T7's `HIWORD(wParam)` | Retains only the factor. `Default` is hand-written; a derived one yields a zero factor. |
| `to_physical(f32)` | scalar outbound | |
| `extent_to_physical((f32, f32))` | 4, 5, 6 (the `SetSize` half) | Position-independent by construction. |
| `relative_offset_to_physical(abs, parent_abs)` | 4, 5, 6 (the `SetOffset` half) | Converts once on the difference. |
| `to_dip(f32)` | 7's atlas origin | |
| `pair_to_dip((f32, f32))` | 1, 2 (+ F-1's 2b), 3, 9 | Positions and extents alike; inbound has no difference-taking form. |
| `surface_pixels((f32, f32)) -> (u32, u32)` | 7's allocation | The `ceil` rule; integer return so a later cast cannot truncate it back. |
| `factor()` | 7's `96 × s`, T8's ratio assertions | |

**Two deliberate exclusions, recorded rather than left as omissions.**

- **No `d2d_dpi()` helper for `96 × s`.** [plan.md](./plan.md) §T2 names
  the `ceil` rule as the operation to own, and the reason generalises:
  the rounding rule is owned here because it *has* a contract to get
  wrong (up, not toward zero). `96 × s` has no rounding and no contract —
  it is one multiplication off `factor()`, and pulling it in would put a
  DirectWrite-context concern in a type whose value is that it has no
  rendering dependency. T6 writes it at its call site.
- **No `from_factor(f32)` constructor.** Every real source of a scale is
  a DPI (`GetDpiForWindow`, `HIWORD(wParam)`), including T8's synthetic
  changes, which drive the handler rather than constructing a scale.
  A bare-factor constructor would exist only to let a caller invent one.

**Commit shape.** One commit for all four task-list items rather than the
default one-per-item ([AGENTS.md §Commit rules](../../../../AGENTS.md)).
The items do not split trap-#4-clean: items 1 and 3 each introduce an
authored branch (the zero-DPI fallback, the one-pixel floor) whose test
lives in item 4, so a per-item split would land two untested branches in
intermediate commits. [plan.md](./plan.md) §T2 is updated to record what
actually happened.

### Close gate

**#4 — branch tests, verified by mutation, not by assertion.** Each test
was shown to **fail against a deliberately wrong implementation**. A
green test proves nothing about whether it fires; these runs are the
evidence that it does. Seven throwaway mutations, each applied to a
restored-from-backup copy, run with
`cargo test -p wasamo-runtime --lib dip_scale`, then reverted (final state
verified green, and `git status` shows only the intended two files):

| Mutation | Tests that failed | |
|---|---|---|
| **M1** `ceil` → truncate | `surface_allocation_rounds_up`, `surface_allocation_is_never_zero_pixels` | the ceil contract |
| **M2** drop the `.max(1)` floor | `surface_allocation_is_never_zero_pixels` | **authored branch 2** |
| **M3** drop the zero-DPI guard | `zero_dpi_falls_back_to_identity` | **authored branch 1** |
| **M4** multiply both operands, then subtract | `converting_once_on_the_difference_differs_from_converting_twice`, `position_and_extent_convert_separately` | the convert-once rule |
| **M5** factor inverted (`96 / dpi`) | 7 of 11 | |
| **M6** `to_dip` multiplies instead of dividing | `conversion_at_125_150_200_percent`, both round-trip tests | |
| **M7** `extent_to_physical` converts inbound | `conversion_at_125_150_200_percent`, `position_and_extent_convert_separately` | |

An eighth wrong implementation was **not expressible**: deriving an
extent by converting two edges and subtracting cannot be written against
`extent_to_physical`, because the signature is handed an extent and never
a position. That is the API doing the work a test would otherwise have
to.

**Verification item 1, test by test.**

| Claim | Test |
|---|---|
| Conversion at 125 / 150 / 200% | `from_dpi_yields_the_documented_factors`, `conversion_at_125_150_200_percent` (incl. 800 × 600 DIP → 1000 × 750 physical at 125%, DD-004's window-size claim as arithmetic) |
| Position-and-extent consistency | `position_and_extent_convert_separately` |
| Round-trip error | `round_trip_error_is_bounded_by_one_ulp` |
| Rounding **direction** | `round_trip_rounds_to_nearest_in_both_directions` (two-sided) contrasted with `surface_allocation_rounds_up` (one-sided, upward) |
| The `ceil` allocation contract | `surface_allocation_rounds_up`, `surface_allocation_is_never_zero_pixels` |
| Convert-once-on-the-difference | `converting_once_on_the_difference_differs_from_converting_twice`, `the_difference_rule_is_only_observable_at_non_dyadic_scales` |
| The identity world T2–T8 land into | `identity_and_default_are_one_hundred_percent` |

**Three measured facts worth carrying, from the brute-force searches.**
All three are recorded because each is a statement about what *testing*
can and cannot catch in this phase, not just about the type.

1. **A round trip is inexact in the common case, not the exceptional
   one.** Over two million consecutive `f32` starting at 0.1:
   750,000 non-exact round trips at 125% and 500,000 at 150%, worst
   relative error 7.45 × 10⁻⁸ — consistent with the two-rounding bound of
   one `f32::EPSILON`, which is what the test asserts. The direction is
   two-sided: the ulp-neighbours of 0.1 at bit offsets +2 and +4 round
   down and up respectively, so **no site may lean on an inequality
   surviving a round trip**.
2. **At 200% the round trip is exactly the identity**, because the factor
   is a power of two. A test written only at 200% would therefore assert
   a stronger property than the type has, and pass a broken 125%.
3. **The convert-once rule is unobservable at 100% and 200%.** The search
   found **no** disagreeing pair at 200% at all, whereas at 150% the
   witness `abs = 10.1`, `parent = 5.7` gives `once = 6.600001` (the
   correctly-rounded answer, exactly) against `twice = 6.6000013` (one
   ulp away). This is a second, arithmetic-level instance of F-4's
   lesson: the scales at which the phase's rules are checkable are
   125% and 150%, and a round-number scale hides them.

**#5 — carry-forward.** The invariant: **the rounding contract is now
enforced by the API shape, and only for callers that use it.**
`extent_to_physical` cannot be given a position, `relative_offset_to_physical`
cannot be reached without both absolute DIP positions, and
`surface_pixels` returns a count rather than a length. A later task that
writes `dip * scale.factor()` by hand at a seam, or that reconstructs a
pixel count as an `f32`, defeats all three silently and only at
non-dyadic scales. *Re-trigger criteria:* (a) any new conversion site
that reaches for `factor()` instead of a named operation — legitimate
only for T6's `96 × s`; (b) an integer-pixel-snapping policy, which
DD-002 §Forward-compat 5 says extends this type's rounding contract
rather than the space definition; (c) a scale-dependent `measure`, which
is already recorded against T6/T7. Recorded here rather than in
[handoff.md](./handoff.md): the consumers are all in-phase and
[plan.md](./plan.md) §T5 / §T6 already carry the obligation. T12
re-evaluates whether (b) belongs in the phase handoff.

**End-gate items from [plan.md](./plan.md) §T2.**

- *Tests named per contract* — the mapping table above.
- *`cargo test` green* — `cargo build -p wasamo-runtime` then
  `cargo build --workspace` then `cargo test --workspace` (the F-5
  ordering, used as a matter of course rather than after a failure):
  **32 test binaries, 0 failures**, the runtime lib going 446 → 457 as
  the 11 new tests land. `cargo fmt --all -- --check` and
  `git diff --check` clean. Per the owner-agreed downgrade this is a
  **regression check** for the rest of the phase — but T2 is the stated
  exception where the task's own tests carry real information, and the
  mutation table above is why that claim is checkable rather than
  asserted.
- *No production call site introduced* — audited by
  `Select-String` for `dip_scale|DipScale` over `wasamo-runtime/src`,
  `wasamo-runtime/tests`, `wasamo-runtime/tests/common`, `wasamoc/src`,
  `wasamo-dll`, and `bindings/rust/src`, excluding the module itself.
  **One hit: `lib.rs:6: mod dip_scale;`** — the declaration without which
  the module would not compile at all. No warning is emitted for the
  unreachable surface because of the forward-pointer allow, which names
  T4 / T5 / T6 as the tasks that remove it.

### Plan-hypothesis re-audit (2026-07-28, owner-prompted)

The first pass of T2's plan revision updated **only §T2** — ticking its
items and recording the one-commit landing — without re-reading the rest
of the task list against what T2 had learned. That is the same omission
class T1 recorded as F-6 … F-10, recurring one task later and under the
same prompt. Recorded as findings again, and the recurrence is the more
important half: **the re-audit is not a spike-only obligation. Any task
that measures something can falsify a hypothesis in a task it never
touched.**

Six standing hypotheses were falsified or sharpened.

- **F-11 — the plan's own gate-substitution table handed T2 the one
  artifact an agent can fabricate.** [plan.md](./plan.md) §Task list
  reads "What counts per task: T2 its own unit tests (pure logic,
  genuinely informative), T3 the rendered gallery frame, T5 the
  call-site audit table, T6 the rendered output, T7 the structural
  side-effect enumeration, T8 its own scale-driving assertions." Every
  entry but T2's is checkable against ground truth; "its own unit tests"
  is a green/red claim, which is exactly what F-4 had just finished
  disqualifying. The exception for pure logic is right but its
  **condition was missing**: pure logic is informative *once the tests
  are shown to fire*. T2 measured the gap — eleven green tests said only
  "eleven tests exist and passed" until seven mutations showed which
  wrong implementation each one catches. *Disposition:* §Task list
  revised; the mutation table is T2's real artifact.
- **F-12 — the plan's narrowing of trap #4 was itself incomplete.**
  [preamble.md §Implementation gates](./preamble.md#implementation-gates)
  records the ADR's phase-wide "trap #4 non-applicable" and then narrows
  it with one exception: "**T9** does add a diagnostic branch." T2 added
  two authored branches (the zero-DPI fallback, the one-pixel surface
  floor), and [plan.md](./plan.md) §T2 already said so — so the preamble
  and the plan disagreed with each other from the moment they were
  written, and the landing made it concrete. The consequence was not
  cosmetic: the preamble's review-lane table assigns T2 "Normal review"
  on the strength of "pure logic", whereas
  [gates §4](../../../procedures/implementation-gates.md) assigns an
  authored-branch task the **branch/test-focused review**. T2's start
  gate classified it that way independently; the preamble is corrected to
  match rather than the other way round. *Disposition:* preamble §Implementation
  gates + review-lane table.
- **F-13 — 200% cannot discriminate the convert-once rule, so T8's three
  scale factors are not three equal probes.** Measured: at a power-of-two
  factor the multiplication is exact, so "subtract in DIP then multiply"
  and "multiply then subtract" agree **everywhere** — a brute-force
  search found no disagreeing pair at 200% at all, against a witness at
  150% one ulp apart. The round trip is likewise exactly the identity at
  200% and inexact for the majority of `f32` at 125%. T8 keeps all three
  factors, but 200% is a magnitude check; the rule verification is
  carried by 125% and 150%. This is F-4's lesson at the arithmetic level:
  the scales at which this phase's rules are observable are the awkward
  ones. *Disposition:* [plan.md](./plan.md) §T8 + preamble §The
  sequencing thesis.
- **F-14 — T6's `ceil` bullet needs the landed signature, and the
  existing `max(1.0)` becomes a second home for the floor.**
  `surface_pixels` returns `(u32, u32)`, so T6 either casts for
  `CreateDrawingSurface`'s `f32` `Size` or moves to
  `CreateDrawingSurface2`'s `SizeInt32` — both permitted by DD-002, whose
  contract is the pixel count and not the API pair. Separately, the
  one-pixel floor now lives **in the type**, so `draw_text`'s existing
  `width.max(1.0)` / `height.max(1.0)` must be removed rather than left
  in place: harmless arithmetically, but it is the rounding rule living
  in two places, which is the drift T2 exists to prevent. *Disposition:*
  [plan.md](./plan.md) §T6.
- **F-15 — "convert once on the difference" is now enforced by API shape,
  and only for callers that use the API.** [plan.md](./plan.md) §T5 states
  the rule as prose ("subtract in DIP, multiply the result"). A T5 that
  writes `dip * self.scale.factor()` satisfies the prose reading, defeats
  the enforcement, and is wrong only at non-dyadic scales — where, per
  F-13, only two of the phase's test factors would notice. The bullets are
  revised to name the operations. The single legitimate `factor()` use in
  the phase is T6's `96 × s`, which T2 deliberately did not wrap because it
  carries no rounding contract. *Disposition:* [plan.md](./plan.md) §T5,
  §T6.
- **F-16 — two downstream tasks would otherwise re-implement what the
  type already does.** T4 needs no zero-DPI guard of its own
  (`from_dpi` floors), and T5's "defaulted to 1 in every `WidgetNode`
  constructor" is `DipScale::default()` rather than a hand-written
  literal. Cosmetic individually; together they are the same
  second-home-for-a-rule failure as F-14. *Disposition:*
  [plan.md](./plan.md) §T4, §T5.

*Disposition summary:* all six folded into [plan.md](./plan.md) and
[preamble.md](./preamble.md) in the same commit as this entry. F-11 →
§Task list; F-12 → preamble §Implementation gates and the review-lane
table; F-13 → §T8 and preamble §The sequencing thesis; F-14 → §T6;
F-15 → §T5 and §T6; F-16 → §T4 and §T5.

---

## T3 — Button / ToggleButton label Visual writes move into the sync pass

### Start gate (recorded 2026-07-28, before choosing the approach)

Read before selecting: [plan.md](./plan.md) §T3 and its §Task list
preamble, [preamble.md](./preamble.md) (§Implementation gates, the
review-lane table, §Technical risks), the ADR set (DD-002 §The conversion
sites row 6 and its detail paragraph, §When surfaces learn their scale,
§Technical risk re-evaluation; DD-003 §Structural side-effect enumeration
row 7), the T1 and T2 entries above, and
[implementation-gates.md](../../../procedures/implementation-gates.md).
Source read end-to-end for the change: `widget.rs` `button_family`
(765–850), `update_button_label` (1005–1047), `set_property`'s
`size_affecting` clause (915–1003), `insert_child_inner` (1450–1491),
`run_layout` / `run_layout_as_window_root` / `sync_visuals` (1571–1793);
`window.rs::set_root` (140–174); `emit.rs` `mark_layout_dirty_for` /
`flush_layout` (100–149); `abi.rs` `wasamo_set_property` (717–749) and
`wasamo_widget_insert_child` (421–453); `ir_loader.rs`'s conditional and
`for`-range mutation sites (3388–3557).

**Trap selection.** [plan.md](./plan.md) §T3 pre-names trap #2. Four more
are added here; the gate permits adding, never silently dropping.

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes**, in its call-site-audit form | No enum gains a variant, but the task *is* a relocation of a write between call sites, and it is complete only if (a) **both** sites that write the label geometry today are relocated — construction (`widget.rs:813` / `818`) and the label-update path (`1035` / `1040`) — and (b) the receiving arm covers **both** `WidgetData` variants that carry a `ButtonData`. Dropping `ToggleButton` from the new arm is exactly trap #1's shape: a filter that silently omits one variant, invisible to every existing test because no test can read `label_visual` (it is private to `widget.rs`, verified by search). |
| 2 | Missed side effects | **yes** | The named start gate. A write moves between passes: it stops happening at construction / at the property write and starts happening on every layout pass. What depended on it landing at construction time is enumerated at the close gate. |
| 3 | Parallel/derived data drift | **conditionally yes** — decided with the trap in view, not after | Applies **iff** the measurement-source decision below retains the measured extent on `ButtonData`. That field would be a cached derivative of (`label_text`, `label_style`) sitting beside two *existing* derivatives of the same measurement — `self.width` / `self.height` as `SizeConstraint::Fixed`. If taken, it must be written atomically inside the same primitive that writes `label_text`, at **both** writers. Recorded now so the decision is made against the trap rather than justified past it. |
| 4 | Untested authored branch | **yes**, with an unusual discharge | The new `WidgetData::Button(btn) \| ToggleButton(btn)` arm in `sync_visuals` is an authored conditional. It has **no test-visible surface**: `label_visual` is private and no integration test can observe it, so there is no test to name. Per the T2 lesson (F-11), a green suite would say nothing here anyway. The branch's firing is demonstrated by the **rendered frame** instead — labels are visible only if the arm fires, and per widget kind — and that substitution is recorded rather than the trap being marked non-applicable. |
| 5 | Carry-forward underweighted | **yes** | After this task the invariant is **"every Composition geometry write in the runtime happens in exactly one pass"**, which is the property that makes T5's audit complete rather than approximately complete (DD-002 §Row 6 detail; [plan.md](./plan.md) §T3). It is an invariant a later task can trip by adding a geometry write at construction. Recorded with a re-trigger criterion at the close gate. |
| 6 | Symptom taken at face value | **yes**, low expectation | The workspace suite is run as a regression check, and [verification-environments.md](../../../../docs/notes/verification-environments.md) Observation 5 records a known, root-caused `scroll_view_layout_integration` access violation whose trigger is process-global Compositor reuse. If it appears it is dispositioned against that root cause, not re-rolled. F-5's cold-directory link failure likewise has a verified remedy that is used as a matter of course. |
| 7 | Weak GUI evidence | **yes** | T3's substantive gate is a rendered frame (T1 finding F-4: the fixtures do not react to a geometry-write relocation). Evidence is launch + `CopyFromScreen` capture + analysis of the captured image, and the **positive control is the before/after pair**, not the "after" frame alone. `Start-Process` survival is a supporting "no early crash" signal only. |

**Review lane — corrected here, and the correction is a finding.**
[preamble.md](./preamble.md)'s review-lane table assigns T3 **Normal
review**, on the stated ground that it is a behaviour-identical refactor
that "carries an explicit regression check against shipped rendering".
That reason was written when the regression check was understood to be
the existing fixtures. T1's finding F-4 and the §Task list revision that
followed it removed that basis: the fixtures **do not react** to a
geometry-write relocation, and [plan.md](./plan.md) §T3 now says in
terms that "the **rendered frame is the gate**". T3's evidence class is
therefore **GUI-render evidence**, which
[gates §4](../../../procedures/implementation-gates.md) names as a
**high-risk class requiring a full independent review** — and the task
additionally relocates a write between passes in shipped rendering code,
which reads as a runtime structural change under the same list.

This is the same documents-disagree-from-birth shape as T2's F-12, one
task later, and it is recorded as **F-17** rather than resolved by
preferring whichever row is more convenient. *Disposition:* the
review-lane row is corrected to **full independent review**, and the
consequence — that the merge gate now needs a review this assistant
cannot perform on its own work — is raised with the owner at the merge
gate rather than absorbed silently.

**Planned proof obligations** (each closed at the close gate):

1. The pre-change frame set is captured **before any code edit**, on a
   build of the unmodified tree, and its capture commit is recorded. A
   pre-change frame reconstructed afterwards is not a control.
2. The trap-#2 enumeration: every consumer that depended on the label
   geometry landing at construction time, each stated as *preserved*,
   *changed-and-why-that-is-safe*, or *pre-existing*.
3. The measurement-source decision, made inside that enumeration, with
   the rejected candidates and their rejection reasons.
4. Both write sites relocated and both `ButtonData`-carrying variants
   covered (trap #1).
5. `PAD_H` / `PAD_V` hoisted to one declaration — the sync pass would
   otherwise be their third site (same rule-in-two-places class as F-14).
6. The node's sizing still derives from the same measurement:
   `SizeConstraint::Fixed` is **per axis** (F-10), so `button_family` and
   `update_button_label` each keep their two one-argument constraints.
7. `cargo build -p wasamo-runtime` → `cargo build --workspace` →
   `cargo test --workspace` green as a **regression check only** (the
   F-5 ordering, used as a matter of course).
8. The post-change frame set captured the same way and compared against
   the pre-change set, with the assistant's analysis of what the
   comparison does and does not discriminate.
9. **The plan re-audit as an in-gate item, not an after-gate one**:
   [plan.md](./plan.md) §T4 … §T12 and [preamble.md](./preamble.md)
   re-read in order, with a task-by-task verdict table. T1 and T2 each
   lost this and were caught by the owner; T2's retrospective recorded
   the falsifiable remediation test that T3 is the first run of.

**Open decision, named by the plan and deliberately not pre-empted
here.** The two relocated writes use `(lw, lh)` from
`TextRenderer::measure`, which is in scope at construction and at the
label update but **not** in `sync_visuals`. Candidates, to be decided
inside the trap-#2 enumeration: re-measure in the sync pass; retain the
measured extent on `ButtonData`; derive it from `computed.size` minus the
padding. The decision criteria fixed **before** looking for a preferred
answer: (a) the value written must be **bit-identical** to today's at
scale 1, or the task is not a behaviour-identical refactor; (b) it must
not add a fallible OS call to a pass that currently makes none; (c) it
must not put a rule in a second place (F-14 / F-16); (d) whatever it
costs must be paid against trap #3 explicitly.

**Commit shape.** The code change is **one commit**, per DD-002's
bisectability requirement (preamble obligation 2) — a regression in
shipped rendering code must be bisectable independently of the DPI
change. That overrides the one-commit-per-task-list-item default in the
other direction from T2's: here several checklist items must land
*together*, because the write sites and their receiving arm do not build
or render correctly in intermediate states.

### The decision — where the sync pass gets the label's measured size

Taken inside the trap-#2 enumeration below, against the four criteria
fixed at the start gate. **Decided: `ButtonData` retains the measured
extent as `label_size: (f32, f32)`.**

| Candidate | Verdict |
|---|---|
| **Retain the measured extent on `ButtonData`** | **Taken.** The exact `(lw, lh)` `measure` returned is stored and written, so the value is bit-identical to today's (criterion a). It adds no OS call to the sync pass (b). It puts no rule in a second place: the padding arithmetic stays where it was and `BUTTON_PAD_H` / `_V` collapse to one declaration (c). Its cost is trap #3, paid below (d). |
| Re-measure in the sync pass | Rejected on (b) and (c). `sync_visuals` holds no `TextRenderer` and makes no fallible DirectWrite call today; supplying one means either changing its signature — which ripples up through `run_layout` and `run_layout_as_window_root` to the 21 layout call sites T1 counted in 6 files — or reaching for `crate::runtime::get()`, which couples the sync pass to the global runtime. And it makes the walk a **second independent producer** of a number the node already commits to through `SizeConstraint::Fixed`, which is trap #3 with the roles reversed. |
| Derive from `computed.size` minus the padding | Rejected on (a): **not behaviour-identical**. `computed.size` is the *arranged* size, which stops being `Fixed(lw + 2·PAD)` the moment a parent stretches the button. Measured, not argued — mutation **N3** below. |
| Derive from `self.width` / `self.height` minus the padding | Rejected on (a) and (c). It avoids new state but recovers the measurement by `f32` subtraction — `(lw + 32.0) - 32.0` is not `lw` for every `lw` — and gives `BUTTON_PAD_H` a third arithmetic role. |

### Close gate

**#1 — call-site audit.** The claim under check: *every site that wrote
the label Visual's geometry has moved, and the receiving arm covers every
`WidgetData` variant that carries a `ButtonData`.*

| Site (pre-change line) | Classification | As landed |
|---|---|---|
| `button_family` `SetOffset(PAD_H, PAD_V)` / `SetSize(lw, lh)` (`widget.rs:813` / `818`) | must-move — construction precedes attachment, so no scale can exist here | removed; the `cast()` and `InsertAtTop` parenting stay, and `label_size: (lw, lh)` is recorded in the `ButtonData` literal |
| `update_button_label` `SetOffset` / `SetSize` (`1035` / `1040`) | must-move — same write, second site | removed; `btn.label_size = (lw, lh)` written beside `btn.label_text` |
| `sync_visuals` — the receiving arm | must-cover **both** `Button` and `ToggleButton` | `if let WidgetData::Button(btn) \| WidgetData::ToggleButton(btn) = &self.data`, placed after the node's own offset/size write and before the ScrollView arm |
| `sync_visuals` node `SetOffset` / `SetSize` (audit row 4) | untouched | unchanged |
| `sync_visuals` ScrollView intermediate (row 5) | untouched | unchanged |
| `window.rs` `SetRelativeSizeAdjustment(1, 1)` (row 8) | untouched | unchanged |

Query: `SetOffset|SetSize|PAD_H|PAD_V|label_size` over
`wasamo-runtime/src`. **After the change every `SetOffset` / `SetSize` in
the runtime is inside `sync_visuals`** — three pairs: the node, the
Button-family label, the ScrollView intermediate — and the only other
Composition geometry call anywhere is `window.rs`'s
`SetRelativeSizeAdjustment`. That is the state DD-002 §Row 6 detail
describes and the precondition that makes T5's audit complete rather than
approximately complete.

`PAD_H` / `PAD_V` had two declarations (`button_family`,
`update_button_label`) and the sync pass would have been a third; they are
now one pair of module-level constants, `BUTTON_PAD_H` / `BUTTON_PAD_V`,
read by all three. Same rule-in-two-places class as F-14.

**#2 — structural side-effect enumeration.** What depended on the label
geometry landing at construction time. Rows marked *unchanged* are
assertions, not omissions.

| # | Path | Before | After |
|---|---|---|---|
| 1 | `window::set_root` — the production attach path | label already placed at construction; the first layout placed everything else | **preserved.** `set_root` runs `run_layout_as_window_root` immediately after inserting the root Visual and before the window is shown, so the label is placed on the same pass as every other Visual, before the first frame |
| 2 | `wnd_proc` `WM_SIZE` → `run_layout_as_window_root` | label untouched by resize (written once, never again) | **changed, deliberately.** The label is now rewritten on every resize with the same values. This is the property T5 needs — a resize at a new scale must re-project the label too |
| 3 | `emit::flush_layout` → `run_layout` (the drain's layout phase) | label untouched | **preserved and load-bearing**: this is the pass that now carries the relocated label-update write |
| 4 | `set_property(PROP_BUTTON_LABEL)` on an **attached** widget | brush, geometry and `SizeConstraint` all written inline | **changed.** Geometry moves to the drain's layout phase, which `wasamo_set_property` runs synchronously at the tail of the same ABI call (`drain_if_outermost`) because `size_affecting` already lists this property. No frame is presented between the brush write and the geometry write — Composition commits at the dispatcher tick, not per property set. Verified by frame: `labelupdate-clicked` and `-clicked-twice` are pixel-identical before and after |
| 5 | `set_property(PROP_BUTTON_LABEL)` on an **unattached** widget | geometry written eagerly | **changed.** `mark_layout_dirty_for` is a no-op when the widget belongs to no window, so no layout pass runs and the label Visual keeps its previous geometry until the subtree is attached — at which point `set_root`'s first pass writes it. Self-healing, and the widget is by definition not on screen in the interval |
| 6 | **The tree-mutation API** — `WidgetNode::append_child` / `insert_child` / `replace_child`, all `pub`, plus their ABI wrappers `wasamo_widget_append_child` / `_insert_child` / `_replace_child` (**re-scoped twice: the row first named only `insert_child`, then only the ABI wrappers — review findings R-2 and, on re-review, R-1's first half**) | the new button's label was placed; the button itself was not | **pre-existing class, not introduced here.** None of these marks layout dirty and none drains, so the new node's *own* Visual already had no offset or size until the next layout pass. T3 makes the label match the button instead of floating at (16, 8) over a zero-sized background. The class is closed only by the next `WM_SIZE` or size-affecting property write. **The boundary is the `WidgetNode` method, not the ABI wrapper** — `WindowState::root_widget` is `pub`, so `window.root_widget.as_mut().unwrap().append_child(button)` reaches the same code with no ABI hop |
| 6b | **Direct Composition hosting** — `lib.rs::window_add_widget`, and equivalently any caller doing `window.root.Children()?.InsertAtTop(&widget.visual.cast()?)` by hand, since `WindowState::root` and `WidgetNode::visual` are both `pub` (**review finding R-1; this row was missing entirely, and its scope was widened on re-review**) | the label carried constructor geometry `(16, 8)` / `(lw, lh)` while the background carried none, so a caller who sized the background by hand got a rendered label | **changed, and this is a real behaviour regression on a public API.** The widget never enters `root_widget`, so it is not merely unlaid-out — **no later pass exists**, and the label keeps `Size = (0, 0)` permanently. The pre-change behaviour was itself half-formed (label placed, background not), but "half-formed" and "no label" are not the same thing. Not fixed by reintroducing a construction-time write, which would destroy the one-pass invariant this task exists to create. Disposition below |
| 7 | `ir_loader` conditional / `for`-range mutations | as above | **preserved.** Both sites call `mark_layout_dirty_for` after the mutation, so the drain's layout phase places the new subtree. Exercised by the `gallery-lightbox` frame, whose three buttons are constructed *after* the tree was attached |
| 8 | Visual parenting / Z-order (`bg_container.Children().InsertAtTop`) | at construction | **unchanged** — only the two geometry writes moved |
| 9 | The node's `SizeConstraint::Fixed` pair | derived from `(lw, lh)` in both writers | **unchanged**, and still per axis (F-10): `Fixed(lw + BUTTON_PAD_H * 2.0)` and `Fixed(lh + BUTTON_PAD_V * 2.0)` in each |
| 10 | Hit-testing / hover (`visual_rect`) | reads the **node's** visual, never the label's | **unchanged** |
| 11 | `update_button_style` / `update_button_enabled` / `update_toggle_button_checked` | touch the background brush only | **unchanged** |
| 12 | `draw_text`'s surface size at construction (`lw.max(1.0)`) | — | **unchanged.** F-14 removes that clamp at T6, not here |
| 13 | Per-pass cost | — | **changed**: two extra WinRT property writes per Button-family node per layout pass. Bounded — the gallery has nine — on an event that is a resize or a property write |
| 14 | **A layout pass that fails** (**Codex review finding R-3; rows 1, 3 and 7 said "preserved" without this qualification**) | the label carried constructor geometry regardless of whether layout succeeded | **changed.** `run_layout_as_window_root` and `run_layout` propagate `layout::run_layout`'s error with `?` **before** reaching `sync_visuals`, and both callers discard the `Result` (`window::set_root` and `emit::flush_layout` each do `let _ = …`). So a subtree carrying a known layout error — a Grid-cell `Box` with `aspect` unbounded on both axes — leaves every Visual in that tree unwritten, and the label now has no constructor geometry to fall back on. The rows above are accurate **conditional on layout succeeding**, which is the normal case and was the unstated assumption |

**#3 — parallel/derived data.** `label_size` is a cached derivative of
(`label_text`, `label_style`), sitting beside two existing derivatives of
the same measurement (`self.width` / `self.height`). It is written in the
**same statement group** as `label_text` and as the `SizeConstraint`
pair, in **both** primitives that produce a measurement —
`button_family`'s `ButtonData` literal and `update_button_label` — so
there is no primitive that mutates the source without updating the cache.
`label_style` has no setter (`PROP_BUTTON_STYLE` carries `ButtonStyle`,
not typography), so it changes only at construction. The one remaining
drift risk is a future third writer of `label_text`; recorded as
carry-forward below.

**#4 — the authored branch, fired.** The new arm has no test-visible
surface: `label_visual` is private to `widget.rs` and no integration test
can observe it, so there is no test name to give. Following T2's lesson
that a green result carries no information until it is shown to fire,
the branch's firing is demonstrated by **deliberately wrong
implementations photographed against the gallery**. Each was built with
`cargo build --release --workspace`, captured, and reverted; frames in
[evidence/mutations/](./evidence/mutations/).

| Mutation | Frame | Reads |
|---|---|---|
| **N1** the arm removed entirely | `n1-arm-removed-gallery.png` | **all six** button labels vanish — the three tabs and the three toolbar buttons. The arm is what draws them now; nothing else does |
| **N2** `ToggleButton` dropped from the pattern | `n2-togglebutton-dropped-gallery.png` | exactly the **three tab labels** vanish while the three `Button` labels stay. This is trap #1's failure shape, and the frame separates it per widget kind |
| **N3** extent derived from `computed.size` minus padding | `n3-arranged-size-labelupdate.png` | only the Grid-stretched button smears — its label surface brush is stretched from ~137 px of text across the full ~528 px cell. **The gallery frames are byte-identical to baseline under N3** (0 differing pixels), which is why the evidence UI gained a stretched button; without one, no frame in the set could have distinguished the taken decision from the rejected one |

**#5 — carry-forward.** Two invariants:

1. **Every Composition geometry write in the runtime now happens in
   exactly one pass.** This is what makes T5's audit complete rather than
   approximately complete. *Re-trigger criterion:* any task that adds a
   `SetOffset` / `SetSize` / `SetScale` outside `sync_visuals` — in a
   constructor, a property setter, or T6's re-rasterization walk — breaks
   the property silently and reintroduces exactly the class DD-002 §Row 6
   detail closed. T6 is the near-term risk: its walk rebuilds brushes and
   must **not** take the opportunity to write geometry.
2. **`label_size` has exactly two writers**, both of which also write
   `label_text` and the `SizeConstraint` pair. *Re-trigger criterion:* a
   third path that changes a Button-family label — a typed property
   writer, an iteration-materialised label rebind, M4-Phase 2's event
   model — must write all three or the label renders at the previous
   text's extent.

Both are in-phase for T5/T6 and forward for later milestones; recorded in
[handoff.md](./handoff.md).

**#6 — deterministic-failure disposition.** None to disposition.
Observation 5's `scroll_view_layout_integration` access violation did not
appear in any run. The one build surprise encountered is **F-21** below;
it was root-caused rather than re-rolled, and the invalid capture it
produced was discarded and redone.

**#7 — GUI evidence.** Launch + `CopyFromScreen` over `GetWindowRect` +
analysis, per
[verification-environments.md](../../../../docs/notes/verification-environments.md)
§Observation 4. Script:
[evidence/capture-t3-label-writes.ps1](./evidence/capture-t3-label-writes.ps1).
Environment: the 125% development machine —
`GetDpiForMonitor(primary, EFFECTIVE)` = 120, while the still-unaware
gallery host is told `GetDpiForWindow` = 96, matching T1's probe. Both
sets were captured under identical conditions, so the comparison is
unaffected by either number.

Six frame pairs, each captured on a `--release --workspace` build of the
tree with and without the change, compared pixel by pixel over the client
interior (the window rect minus the caption and border, so desktop bleed
at the frame edge cannot contribute):

| Pair | Covers | Differing pixels |
|---|---|---|
| `gallery-default` | three `ToggleButton` + three `Button` labels placed by `set_root`'s first pass | **0** of 828,360 |
| `gallery-tab-albums` | the same labels after a click re-runs layout through the drain | **0** of 828,360 |
| `gallery-lightbox` | the `<` / `>` / `x` buttons of the conditional subtree — constructed *after* attach | **0** of 828,360 |
| `labelupdate-initial` | the bound-text button before any click, plus a Grid-stretched button | **0** of 224,480 |
| `labelupdate-clicked` | the relocated label-update write, once | **0** of 224,480 |
| `labelupdate-clicked-twice` | the same write a second time | **0** of 224,480 |

**Correction, and then closure (review finding R-4).** The set originally
started its counter at 0, and these three frames were described as
covering "three different label widths". They did not:
`Counted 0 / 1 / 2 times` differ in glyph but **not in measured width** —
the digits share an advance — so the Button was the same width in all
three, and an implementation writing
`btn.label_size = (btn.label_size.0, lh)` would have kept two writers,
kept the statement-group shape, and passed every frame.

**The evidence UI now starts at 9**, so the click crosses 9 → 10, the
label gains a digit, and the Button's measured width moves. Mutation
**N4** confirms the set reacts: with the stale-width write in place the
counting Button renders **completely blank** (`n4-stale-width-labelupdate.png`),
and both `labelupdate-initial` and `-clicked` go red — the initial frame
too, because the bound label is written through `update_button_label`
once at registration, when the constructed label was still empty. The
hole R-4 identified is closed rather than only recorded.

**The height component is not closed, and cannot be by this set.**
`measure` returns `DWRITE_TEXT_METRICS::height` for an unconstrained
single-line layout, so `lh` is a font-metric constant for a given
`TypographyStyle` — and `label_style` has no setter. A mutation writing
`(lw, btn.label_size.1)` therefore produces an identical frame here.
It is **not** unreachable in principle: a label whose script changes can
draw a fallback font with different line metrics, and multi-line labels
or a future style setter would make it ordinary. Recorded as a stated
gap with that re-trigger rather than closed with a mutation that cannot
be made to fire.

**What the pair does and does not discriminate**, stated rather than
implied. It **does** show that the relocated write lands, for both widget
kinds, on all three paths that reach a Button (first layout, drain
re-layout, post-attach construction), and across a label-width change on
the update path — N1 and N2 are the proof that a frame in this set goes
red when it should. It **does not** show that the *old* writes are gone:
an implementation that added the sync arm and left the construction
writes in place would produce exactly these frames. That half is closed
by the #1 audit above, which is a source claim, not a pixel claim. The
two artifacts are complementary and neither is sufficient alone.

The label-update path has **no shipped example** — no `.ui` in
`examples/` binds a Button's `text` — so the evidence UI
[evidence/t3-label-update.ui](./evidence/t3-label-update.ui) exists for
it, loaded through the gallery host by swapping the compiled IR at the
path the host was built against. It is evidence scaffolding, not a new
example, and the script restores the gallery IR in a `finally` block.

**End-gate items from [plan.md](./plan.md) §T3.**

- *Side-effect enumeration* — the 13-row table above.
- *Fixtures green* — `cargo build -p wasamo-runtime` → `cargo build
  --workspace` → `cargo test --workspace` (the F-5 ordering, used as a
  matter of course): **32 test binaries, 0 failures**, unchanged from the
  T2 baseline. Per the owner-agreed downgrade this is a **regression
  check** only; T1's F-4 measured that these fixtures do not react to a
  geometry-write relocation, and T3's own N1 mutation confirms it — the
  suite stays green with every button label invisible.
- *A rendered gallery frame matching the pre-change frame* — the six
  pairs above, all zero.
- `cargo fmt --all -- --check` and `git diff --check` clean.

### F-21 — a host-package build produces a freshly linked DLL from stale object code

Found while running the N2 mutation, and recorded because it is a
**false-negative generator for every GUI evidence gate in this phase**,
not a T3 detail.

Measured: the first N2 run — `ToggleButton` dropped from the sync arm —
was built with `cargo build --release -p gallery-rust` and produced a
gallery frame **identical to the unmutated build**, which briefly read as
"the mutation does not fire". Rebuilt with
`cargo build --release --workspace`, the same mutation removed exactly
the three tab labels.

**The mechanism first recorded here was wrong** (Codex review finding
R-5), and the correction matters because it changes what the symptom
tells you. The original claim was that the host does not depend on
`wasamo-runtime` through a cargo edge and therefore does not rebuild it.
`cargo tree -p gallery-rust` shows the edge plainly —
`gallery-rust → wasamo-sys → wasamo-dll → wasamo-runtime` — and a probe
(append a comment to `widget.rs`, build `-p gallery-rust`, compare
timestamps) shows cargo **does** recompile `wasamo-runtime` and **does**
relink `wasamo.dll`:

| Artifact | Before probe | After `-p gallery-rust` |
|---|---|---|
| `target/release/wasamo.dll` | 18:31:51 | **19:54:53** — relinked |
| `target/release/deps/libwasamo_runtime-<hash>.rlib` | — | **19:54:53** — recompiled |
| `target/release/libwasamo_runtime.rlib` (**uplifted**) | 18:31:47 | **18:31:47** — unchanged |

The real cause is
[`wasamo-dll/build.rs`](../../../../wasamo-dll/build.rs): it
`/WHOLEARCHIVE`s the **uplifted** `<profile>/libwasamo_runtime.rlib`, and
cargo refreshes that copy only when `wasamo-runtime` is built as a
**primary package**. A dependency build writes the hashed rlib in
`deps/` and leaves the uplifted one alone, so the DLL is genuinely
relinked — new timestamp, no warning — around **stale object code**.

So this is **not a mechanism distinct from F-5; it is the same root cause
with the opposite symptom.** F-5 is that whole-archive path failing loudly
(`LNK1356`) when the uplifted rlib is absent; F-21 is it succeeding
quietly when the uplifted rlib is merely **old**. The two belong in one
entry, and the "adjacent but distinct" framing originally written here is
withdrawn. A freshness check on `wasamo.dll`'s timestamp does **not**
detect F-21, which is the practical trap.

*Disposition unchanged, and it was correct even while the reasoning was
not:* every capture in this phase is preceded by
`cargo build --release --workspace`, folded into [plan.md](./plan.md) §T6,
§T9 and §T10 and into preamble R-1b; carried to
[handoff.md](./handoff.md) as F-5's second symptom rather than as a
separate item, and folded into T12's existing
[AGENTS.md §Build ordering](../../../../AGENTS.md) correction.

### Plan-hypothesis re-audit (2026-07-28, in-gate — not owner-prompted)

T1 and T2 each landed a first pass that revised only the tasks their
findings *pointed at*, and each was caught by the owner. T2's
retrospective recorded the correction as a falsifiable test: **T3 is
valid if it detects at least one item unprompted, and falsified if it
self-reports zero and the owner then finds one.** This section is that
run, and it is an item *inside* the close gate rather than after it.

[plan.md](./plan.md) §T4 … §T12 and [preamble.md](./preamble.md) (the
review-lane table, §Implementation gates, §Technical risks, §The
sequencing thesis) were re-read in order against what T3 landed and
measured. Verdicts, every entry, including the ones with nothing to
correct:

| Re-read | Verdict |
|---|---|
| §Task list preamble (gate-substitution table, commit rules) | no additional correction — T3's row already reads "the rendered gallery frame", and T3's evidence is that plus the #1 source audit; the substitution table is not weakened by adding the second artifact |
| §T3 | corrections: checklist ticked, the measurement-source decision recorded as taken, the one-commit landing recorded |
| §T4 | no additional correction. T3 touches nothing T4 depends on; F-16's `from_dpi` note and F-7's `window::create` siting stand |
| §T5 | **correction — F-19**: "`relative_offset_to_physical(abs, parent_abs)` for every `SetOffset`" is falsified for audit rows 5 and 6 |
| §T6 | **corrections — F-20** (the walk can read `label_size` and must not become a second measurer) and **F-21** (workspace build before the rendering gate) |
| §T7 | **correction — F-18**: the clip-inset row inherits the Box-vs-ZStack error that F-2 corrected only on the DD-002 side |
| §T8 | no additional correction. T3 adds no test and no assertion surface; `label_visual` stays private, so T8's Visual-ratio assertions remain node-level as written |
| §T9 | **correction — F-21**: the three-host rebuild is the artifact for DD-001's boundary claim, and a host-package build would run it against a DLL relinked around pre-T9 object code |
| §T10 | **correction — F-21**, and a confirmation: T3's capture script and its click-point derivation are reusable, and the aware/unaware `MoveWindow` / `GetWindowRect` asymmetry T3 worked through is the same thing R-7 already assigns to T10 |
| §T11 | no additional correction — owner-executed, unaffected |
| §T12 | **correction — F-21's [AGENTS.md](../../../../AGENTS.md) half**, folded into the existing F-5 correction item rather than added as a second bullet |
| preamble §The sequencing thesis | no additional correction |
| preamble §Verification closure | no additional correction |
| preamble §Obligations carried | no additional correction — obligation 2 (T3 lands in its own commit ahead of the scale work) is discharged as written |
| preamble §Implementation gates | **correction — F-22**: the phase-wide "trap #3 non-applicable" is falsified by T3's landing |
| preamble review-lane table | corrected at the start gate — **F-17** |
| preamble §Technical risks | **correction**: R-1b gains F-21 as a second instance of "the build command did less than it looked like it did" |

Six findings, five of them in tasks T3 never touched.

- **F-18 — T7's clip-inset row audits a site that does not exist and
  misses one that does.** T1's F-2 established that the three zero-inset
  `InsetClip` installs are `scroll_view`, `grid` and **`zstack`**, and
  that `box_` installs none — and dispositioned the correction to T5,
  because the row it was reading was DD-002's row 12. The **same wrong
  widget set appears independently in
  [DD-003 §Structural side-effect enumeration](../decisions/dd-m4-p1-003-dpi-change-propagation.md)
  row 10**, which is the enumeration [plan.md](./plan.md) §T7 names as its
  close artifact. A T7 that builds its enumeration from DD-003's wording
  would assert "Box" unchanged — a site with no clip — while never
  looking at ZStack. Re-verified against the source at T3:
  `CreateInsetClip` appears at `scroll_view`, `grid`, `zstack`, and
  nowhere else. The row's *conclusion* (all insets are zero, zero is
  scale-invariant) is unaffected. This is the general lesson of F-2
  landing in only one of the two places that carried the error.
  *Disposition:* [plan.md](./plan.md) §T7.
- **F-19 — "convert once on the difference" does not describe two of the
  three outbound rows.** [plan.md](./plan.md) §T5 says to use
  `relative_offset_to_physical(abs, parent_abs)` for **every** `SetOffset`.
  Only audit row 4 — the node's own write — takes a difference of two
  absolute DIP positions. Row 5's ScrollView intermediate offset is
  `(0, −applied_y)` and row 6's label offset is
  `(BUTTON_PAD_H, BUTTON_PAD_V)` **as landed at T3**: both are already
  parent-relative, with no absolute pair to subtract, so applying the
  named operation would require inventing one. T2's landed API has a
  scalar `to_physical` and an extent form but no already-relative *pair*
  form. The rule itself is not weakened — a single multiplication of an
  already-computed relative quantity is exactly the one rounding the rule
  asks for — but the bullet as written cannot be followed literally, and
  the F-15 concern it was written to prevent (reaching for `factor()` and
  hand-rolling) applies with more force where no named operation fits.
  *Disposition:* [plan.md](./plan.md) §T5 — the bullet is split per row,
  and T5 decides explicitly between calling `to_physical` per component
  and adding a named already-relative form, recording which and why.
- **F-20 — T6's walk would otherwise re-derive a number the node now
  holds.** T1 decided the walk rebuilds each Button label from
  `btn.label_text` / `btn.label_style` via `measure` + `draw_text`. After
  T3 the measured extent is retained as `btn.label_size`, and `measure`
  is DIP and scale-invariant (audit row 10), so a re-measure inside the
  walk can only return the same pair — which makes it a second producer
  of a fact the node stores, the drift F-14 and F-16 were recorded to
  prevent, on the phase's highest-consequence path. *Disposition:*
  [plan.md](./plan.md) §T6 — the walk reads `label_size` for the surface
  extent and does not re-measure; if a future change makes `measure`
  scale-dependent, that is the already-recorded re-trigger and both this
  and T7's step ordering are re-derived together.
- **F-21 — a host-package build relinks the DLL around stale object
  code.** Recorded in full above, with the mechanism corrected at the
  independent review: cargo *does* recompile the runtime and *does*
  relink `wasamo.dll`; the stale input is the **uplifted** rlib that
  `wasamo-dll/build.rs` whole-archives, which makes this F-5's root cause
  with the opposite symptom. *Disposition:* [plan.md](./plan.md) §T6,
  §T9, §T10, §T12; preamble R-1b; [handoff.md](./handoff.md).
- **F-22 — the phase-wide "trap #3 non-applicable" is falsified.**
  [preamble.md](./preamble.md) §Implementation gates records trap #3 as
  non-applicable for the phase, "no parallel vectors or derived indices
  are added; the scale is a single scalar per window". T3 adds
  `ButtonData.label_size`, a cached derivative of the node's label text
  and style that sits beside two existing derivatives of the same
  measurement. The judgment was reasonable when written — the ADR was
  reasoning about the scale — but it is now false as stated, and it is the
  **third** instance of a phase-wide non-applicability being narrowed by
  what actually landed (trap #4 at T2 via F-12, the review lane at T3 via
  F-17, trap #3 here). *Disposition:* preamble §Implementation gates
  gains the narrowing with T3 named as the site and the single-writer
  discipline as the close artifact.
- **F-17** was recorded at T3's start gate and folded there; it is listed
  in the table above so the count is honest, not carried twice.

*Disposition summary:* all folded into [plan.md](./plan.md),
[preamble.md](./preamble.md) and [handoff.md](./handoff.md) in the same
commit as this entry. F-17 → preamble review-lane table (landed at the
start gate); F-18 → §T7; F-19 → §T5; F-20 → §T6; F-21 → §T6, §T9, §T10,
§T12, preamble R-1b, handoff; F-22 → preamble §Implementation gates.

### Owner decisions on the T3 retrospective's open questions (2026-07-28)

All three answered; recorded here so no later task re-opens them.

1. **Review lane (F-17) — discharge the full independent review through
   another agent.** Option (a): Codex reviews the branch against a
   written brief, rather than the owner reading it or the assistant's own
   audit standing in as a substitute. The merge gate is **blocked until
   that review is complete and its findings are dispositioned**;
   remediation commits carry `Reviewed-by: codex <codex@openai.com>`
   alongside the Claude trailer, and the doc gates plus the retrospective's
   commit list are re-run and updated against the final branch state.
2. **The "show it goes red" discipline — codify option A only, leave the
   wider form to judgment (option C), and file the vision decision record
   at Phase 1 close.** The scope decided:
   - **Mandatory, once the record lands: pure-logic unit tests.** A new
     rounding-rule / unit-conversion / boundary-condition surface ships
     with at least one deliberately wrong implementation shown to turn
     its tests red. This is T2's mutation table promoted from "what T2
     happened to do" to an obligation, and it touches trap #4's close
     artifact — "the test name per added branch" becomes "the test name
     per added branch, plus the wrong implementation it was shown to
     catch".
   - **Not mandated: the wider form.** T3 measured that the same
     discipline works on a rendered frame and that it found a defect in
     the evidence pipeline rather than in the code (F-21), but one
     instance is not enough to put a rebuild-and-recapture cycle on every
     GUI gate. Trap #7's close artifact is **unchanged**, and T6 / T10
     inherit **no new obligation** — the technique stays recommended and
     recorded, not required.
   - **Timing.** The record is filed at **Phase 1 close**, not now.
     Under [AGENTS.md §Process rule lifecycle](../../../../AGENTS.md) a
     structural change updates its SSOT in the same commit batch that
     flips the record to `Accepted`, so
     [implementation-gates.md](../../../procedures/implementation-gates.md)
     is **deliberately left untouched by T3**. Recording the decision now
     and the edit later is the point: the scope is fixed while the
     evidence is fresh, and the SSOT never carries a rule whose record is
     still open. Owner of the filing: the **phase-end batch**, listed in
     [plan.md](./plan.md) §T12.
   - **In-phase consequence: none.** T2 was the phase's only pure-logic
     surface and already discharged the obligation ahead of it existing.
     A later task that introduces pure logic picks it up.
3. **Evidence scaffolding stays under [evidence/](./evidence/).** The
   capture swaps the compiled `.uic` at the absolute path
   `gallery-rust`'s `build.rs` baked into the executable — a gitignored
   build product under `target/` — and restores it in a `finally`. The
   residue: a hard kill between the swap and the restore leaves the
   evidence IR in place, and cargo will not regenerate it unless
   `gallery.ui` changes, so a later gallery capture would photograph the
   evidence UI and report success. **Hardened at the owner's direction**:
   the script now detects a leftover backup at start and restores before
   capturing, which makes the interrupted state self-healing on the next
   run rather than silent until someone reads a frame closely. The guard
   was verified by **reproducing the interrupted state** — evidence IR
   written over `gallery.uic`, backup file present — not only on the
   happy path: the warning fired, the gallery IR was restored, the run
   completed, and the recaptured frames are pixel-identical to the
   committed set over the client interior.

### Independent review disposition (Codex, 2026-07-28)

The full independent review the lane raise (F-17) required. Five
findings: one major, four minor. **The verdict was not "zero", and three
of the five contradict claims this log made** — one of them a mechanism
asserted from a truncated build log rather than measured. Each was
re-verified against the source before being accepted; none was taken on
the reviewer's word.

| # | Finding | Verified | Disposition |
|---|---|---|---|
| R-1 | **major** — `lib.rs::window_add_widget` runs no layout and no sync, so removing the constructor geometry leaves a Button-family label permanently at `Size = (0, 0)` on that path | **Confirmed.** The function attaches a Visual to `window.root.Children()` and returns; its own doc names "no layout, no hit-test" as the point. The enumeration missed it because the enumeration walked *layout* paths, and this is the one attach path that deliberately is not one | Enumeration row **6b** added; the misleading `button_family` comment corrected (it claimed construction happens in the IR loader, which is not the only constructor); `window_add_widget`'s doc states the consequence. **Not** fixed by reintroducing a construction-time write — that would destroy the one-pass invariant the task exists to create. **Widened on re-review** (see the second-round table below) and the API's disposition taken as T3's own |
| R-2 | **minor** — enumeration row 6 named only `wasamo_widget_insert_child`; `append_child` and `replace_child` are the same class | **Confirmed** at `abi.rs:388` and `abi.rs:491`: neither marks layout dirty, neither drains | Row 6 rewritten to name all three |
| R-3 | **minor** — the "preserved" verdicts hold only when layout succeeds; `run_layout*` propagates the error with `?` before `sync_visuals`, and both callers discard the `Result` | **Confirmed** by reading `run_layout` / `run_layout_as_window_root` and the two `let _ = …` call sites | Enumeration row **14** added, stating the condition the other rows assumed |
| R-4 | **minor** — the three label-update frames were described as "three different label widths"; the widths are identical | **Confirmed.** `Counted 0/1/2 times` differ in glyph, not in advance width. A mutation keeping the stale width would pass the whole set | The claim is corrected in place. The gap was first recorded as missing; **the second round closed its width half with mutation N4** (see S-5 below), and the height half is stated with its re-trigger |
| R-5 | **minor** — F-21's mechanism ("no cargo dependency edge") is wrong | **Confirmed, and the claim was wrong.** `cargo tree` shows the edge; a timestamp probe shows the DLL *is* relinked. The stale artifact is the **uplifted** rlib that `wasamo-dll/build.rs` whole-archives | F-21 rewritten with the measured timestamps; the "distinct from F-5" framing withdrawn — it is F-5's root cause with the opposite symptom. Downstream copies in [plan.md](./plan.md), [preamble.md](./preamble.md) and [handoff.md](./handoff.md) corrected |

**What the review confirmed independently**, and so is not re-argued
here: the `label_size` two-writer discipline holds; `label_style` has no
setter; `SetOffset` / `SetSize` really are confined to `sync_visuals`'s
three pairs, with the geometry-API sweep extended past the grep terms
used here (`SetScale`, rotation, transform matrix, relative offset,
centre/anchor, geometry clip, offset/size animations — none present; the
only `StartAnimation` targets `"Color"`); F-18, F-19 and F-20 are
correct; the six before/after pairs match to SHA-256; N1 / N2 / N3 show
what they are said to show.

**The disposition of `window_add_widget` (R-1): document and keep.**
Taken as T3's own decision. It was briefly raised as an owner question
and should not have been — none of the three owner-confirm criteria fires
(no AC or phase change, no new cross-task constraint beyond the
carry-forward already recorded, no downstream task revision), and the
question was about how to *describe* the change rather than what to
build. The grounds:

- **Nothing is blocked.** `window_add_widget` appears in no spec —
  `docs/architecture.md`, `docs/abi_spec.md` and `docs/dsl_spec.md` do
  not mention it — carries no ABI surface, and has **zero callers** in
  the workspace. `bindings/rust` depends on `wasamo-sys`, i.e. the C ABI,
  not on this crate.
- **It is already superseded.** `window_set_root` was added at `163067a`
  for exactly the case that needs layout, and the runtime difference
  between the two entries (layout pass, hit-test registration) is
  recorded as far back as
  [M2-Phase 7 framing O6](../../../milestone-2/phase-7/requirements/framing-dd-010.md),
  where it was found by GUI execution rather than source review.
- **The alternatives cost more than the problem.** Giving it a layout
  pass would invert its stated "no layout" contract and change behaviour
  on a task chartered as behaviour-identical; removing it is a public-API
  deletion and belongs to a cleanup task, not here.

Recorded as a **stated limit**, so the phase does not close claiming a
behaviour-identical refactor with no exception attached, and carried to
[handoff.md](./handoff.md) as a cleanup candidate — a caller-less public
entry left behind when `window_set_root` superseded it.

**A note on the review's own limit.** Every finding concerns a path or a
claim; none contradicts the landed arithmetic or the write relocation
itself. What an independent review of this shape still could not supply
is a mutation nobody thought of — R-4 is the case in point: the reviewer
found the missing control by reading the evidence *description*, not by
running a mutation the assistant had not designed.

### Second review round (Codex, 2026-07-28) — delta review of the disposition

Seven findings: one major, four minor, two nits. **Every one is real**,
re-verified against the source before acceptance. The first round's
lesson repeated in a sharper form: the corrections were right where they
were applied and **incomplete in where they were applied**.

| # | Finding | Verified | Disposition |
|---|---|---|---|
| S-1 | **major** — rows 6 / 6b counted the *ABI wrappers* and `window_add_widget`, missing direct Rust-native mutation and direct Composition hosting | **Confirmed.** `WindowState::root_widget`, `WindowState::root` and `WidgetNode::visual` are all `pub`, and `append_child` / `insert_child` / `replace_child` are `pub fn` on `WidgetNode` — so the ABI is not the boundary | Rows 6 and 6b re-scoped **by mechanism instead of by wrapper**: row 6 is the tree-mutation API with the ABI functions as one caller; row 6b is direct Composition hosting, of which `window_add_widget` is the convenience form. This is the fourth path class the re-framed enumeration was asked for |
| S-2 | **minor** — the `button_family` comment ("every path that puts a widget on screen as content runs a layout pass") contradicts row 6 | **Confirmed** — the comment was written before rows 6 / 6b / 14 existed | Comment rewritten to name which paths do and do not, and to name the omission (`mark_layout_dirty_for`) that reproduces the defect |
| S-3 | **minor** — F-21's old mechanism survives in the retrospective's learnings and share-forward sections and in this log's re-audit bullet | **Confirmed** at three sites | All three corrected. **This is F-18's own failure class, committed by the correction to F-18**: the fix was applied to the documents that *carried the finding* and not to the documents that *summarised* it |
| S-4 | **minor** — "three different label widths" survives in the pair-analysis paragraph, directly after the paragraph withdrawing it | **Confirmed** | Corrected |
| S-5 | **minor** — the R-4 gap statement covers width and understates the height component | **Confirmed** — `(lw, stale_height)` also passes the set | Gap restated for both components. The **width** half is now **closed** by evidence (below); the **height** half is recorded with the reason it cannot be closed by this set and its re-trigger |
| S-6 | **nit** — "not rasterized to screen" misdescribes the mechanism; the label *is* rasterized and brushed, and is not composited because the Visual is zero-sized | **Confirmed** — `draw_text` and `CreateSurfaceBrushWithSurface` both run at construction | Doc corrected. The misdescription would have sent a reader to re-create the surface, which changes nothing |
| S-7 | **nit** — the R-1 row still called the API's future an owner decision after the text below settled it | **Confirmed** | Row corrected |

**The width gap is closed, not just recorded (S-5).** The evidence `.ui`
now starts its counter at **9**, so the click crosses 9 → 10 and the
label gains a digit; mutation **N4** (`label_size = (label_size.0, lh)`)
turns the counting Button completely blank and reddens two frames. Both
sets were re-captured on their respective builds and all six pairs are
again zero.

**A pre-existing defect the new evidence exposed, recorded as F-23.**
In every frame after a click, the Grid-stretched Button **disappears** —
in the pre-change build too, so it is not T3's and the pairs stay zero.
The cause is a real inconsistency between the phase's two re-layout
paths: `window::set_root` and the `WM_SIZE` arm call
`run_layout_as_window_root`, which forces the root `LayoutNode` to
`Fill` / `Fill`, while [`emit::flush_layout`](../../../../wasamo-runtime/src/emit.rs)
— the reactive drain's layout phase — calls the plain `run_layout`,
which does not. A root `VStack` (`Shrink`) holding a `Grid`
(`Fill` / `Fill`) therefore lays out correctly on resize and **collapses
the Grid to zero height on any property write**. That is exactly the
M3-Phase 4 T6 failure `run_layout_as_window_root`'s doc comment describes,
still live on the drain path. *Not fixed here* — T3 must stay
behaviour-identical, and this is a behaviour change with its own
evidence needs. *Disposition:* **[plan.md](./plan.md) §T5** as its own
commit with its own before/after frames — T5 already edits that call site
for the inbound DIP conversion (audit row 2b), so fixing it elsewhere
would mean touching the line twice. The item is marked reassignable: it
is a pre-existing defect, not a T5 deliverable, and the owner may move it
without argument. Also in [handoff.md](./handoff.md).

### Re-audit addendum — the review rounds re-run against the task list

The in-gate re-audit (above) ran against what **T3's own work** measured.
The two review rounds then produced further facts, and re-reading the
task list against *those* is the same obligation, one level out — the
failure mode T1 and T2 recorded is "a source of new facts that is not
re-read against the plan", and a review is such a source. Two items, both
in tasks T3 does not touch.

- **F-23 — `emit::flush_layout` uses a different layout entry from every
  other path.** Recorded above; folded into [plan.md](./plan.md) §T5.
- **F-24 — the direct-hosting path is outside the whole phase's
  machinery, and T1's carry-forward was written as if no such path
  existed yet.** [handoff.md](./handoff.md) states the scale cache's
  re-trigger as "any path that attaches … *without* running the walk",
  and illustrates it with M4-Phase 2 and M4-Phase 8 — i.e. as a future
  hazard. `lib.rs::window_add_widget` is already one: a subtree attached
  through it never enters `root_widget`, so it gets **no layout, no
  `sync_visuals`, no scale-cache write and no re-rasterization walk** —
  both walk callers (`window::set_root`, T7's handler) traverse
  `root_widget`. Consequences worth stating before they are discovered:
  T5's cache stays at the identity there regardless of the window's
  scale, and **T6's crispness claim is bounded** — it holds for widgets
  the window owns as content, not for anything hosted directly. Nothing
  needs fixing (the conversions are unconditional; an unreached node is
  simply unconverted), but a limit that is stated is not a surprise at
  M4-Phase 8. *Disposition:* [plan.md](./plan.md) §T5 and §T6 as stated
  limits, and the handoff re-trigger corrected from "future" to "one
  already ships".

Verdict for every other task, re-read again against the review findings:
**T4, T7, T8, T9, T10, T11, T12 and the preamble — no further
correction.** T7's enumeration is about what a scale change drags along,
a different shape from the attach-path question; T8's assertions stay
node-level because `label_visual` is private; T9 / T10 / T12 already
carry F-21's corrected wording.

---

## T4 — Per-window scale on `WindowState` + initial acquisition + DIP window sizing

### Start gate (recorded 2026-07-28, before choosing the approach)

Read before selecting: [plan.md](./plan.md) §T4 and its §Task list
preamble, [preamble.md](./preamble.md) (§Implementation gates, the
review-lane table, §Technical risks R-9, §The sequencing thesis), the
ADR set (DD-001 §The ordering obligation and §Failure handling; DD-002
§The conversion sites rows 1, 2, 13 and §The rounding contract for
surfaces; DD-003 §Where the scale is held, §Initial scale acquisition,
§`WM_DPICHANGED`, §Structural side-effect enumeration; DD-004 §What
`width` / `height` denote and §Does the host need the scale factor), the
T1 / T2 / T3 entries above — in particular the three plan-hypothesis
re-audits and the two review-round dispositions — and
[implementation-gates.md](../../../procedures/implementation-gates.md).
Source read end-to-end for the change:
[`window.rs`](../../../../wasamo-runtime/src/window.rs) in full (339
lines), [`dip_scale.rs`](../../../../wasamo-runtime/src/dip_scale.rs) in
full, `abi.rs` `wasamo_window_create` (307–346) and `wasamo_load_ui`
(1172–1240) with its `DEFAULT_WINDOW_WIDTH` / `_HEIGHT` constants,
`lib.rs` `window_create` / `window_add_widget` / `window_set_root`,
`emit.rs` `flush_layout` (127–149), and `wasamo-runtime/Cargo.toml`.

**Trap selection.** [plan.md](./plan.md) §T4 pre-names trap #5. Four
more are added here; the gate permits adding, never silently dropping.

| # | Trap | Applies | Reason |
|---|---|---|---|
| 1 | Semantic-migration miss | **yes**, in its call-site-audit form | No enum or schema gains a variant, but T4's defining risk *is* a call-site miss and T1 already caught one: **F-7**, the correction placed at `wasamo_window_create` rather than `window::create`, which would have left every `.ui`-loaded window — all three example hosts — at the wrong physical size. The claim to check is "every path that turns a host's DIP size into an HWND passes through the corrected site, and every path that reads a window's geometry reads it after the correction". Sites to enumerate: the three callers of `window::create`, the one `CreateWindowExW`, the one `WindowState` literal, and the two `GetClientRect` consumers. |
| 2 | Missed side effects | **yes** | `SetWindowPos` dispatches window messages **synchronously, before it returns** (the property DD-003 makes load-bearing for `WM_DPICHANGED`), so this task inserts a **re-entry into `wnd_proc` in the middle of window construction**. What that nested dispatch can reach is decided by where the call sits relative to the `GWLP_USERDATA` install at `window.rs:83` — which is one of the two decisions the plan names. The enumeration must state what the nested messages find and what they touch, and it must be **measured** rather than asserted: which messages `SetWindowPos` actually dispatches at a size-preserving call is not something the ADR states. |
| 3 | Parallel/derived data drift | no, with the reason stated rather than inherited | The `WindowState` scale is a cache of `GetDpiForWindow`, but trap #3's discipline — update the parallel copy atomically inside the primitive that mutates its source — has no purchase here: the source is the OS's per-window DPI, which the runtime cannot observe mutating. It learns of a change through `WM_DPICHANGED` (T7), which is an **ordering** obligation, carried as trap #5 and enumerated as trap #2. The phase's genuine parallel copy is the **node-side** cache, which T5 introduces and T6 writes. Separately: T4 stores the scale and **not** the requested DIP size, so there is no second representation of the window's logical size to drift. That exclusion is deliberate and is checked at the close gate. Recorded explicitly rather than inherited because F-22 was this phase's third instance of a phase-wide non-applicability falsified by what landed. |
| 4 | Untested authored branch | **yes**, and the discharge is that no branch is authored | DD-003 I1 words the correction as "**if the scale is not 1**, apply `size × s`". Written that way it is an authored branch that **cannot be fired by any test until T9**, on the one path every host takes — and a second code path is precisely what DD-001 §Failure handling relies on *not* existing ("the conversion machinery is unconditional… there is no second code path to keep correct"). The approach is therefore chosen so the correction is unconditional, and the absence of the branch is the artifact. F-16 already forbids the other candidate branch (a zero-DPI guard): `from_dpi` floors it. Separately, T4 adds one new piece of **pure arithmetic with a rounding contract** (decision 1 below), which takes T2's evidence standard — a test that fires it plus a deliberately wrong implementation shown to turn that test red — voluntarily, ahead of the vision decision record that will make it mandatory. |
| 5 | Carry-forward underweighted | **yes** | The named start gate. The per-window shape is what M4-Phase 8 consumes (DD-003 S2: a second `WindowState` carries a second scale with no structural change), and T5 / T6 / T7 each depend on an invariant this task establishes. Recorded with re-trigger criteria at the close gate. |
| 6 | Symptom taken at face value | **yes**, low expectation | The build and suite are run as a regression check. F-5's cold-directory link failure has a verified remedy used as a matter of course; Observation 5's `scroll_view_layout_integration` access violation has a recorded root cause. **F-21 is the one that matters here**: the probe below launches a host, and a host-package build would run it against a DLL relinked around stale object code — silently, green, with a fresh timestamp. Every build feeding a launch is `cargo build --release --workspace`. |
| 7 | Weak GUI evidence | **yes**, for the probe rather than for the landing | Nothing T4 lands changes a rendered frame: at scale 1 the correction is the identity and the seeded scale has no reader that alters output. So T4's *deliverable* is not GUI evidence. But the artifact that discharges the end gate **is** a launched, captured frame (proof obligation 6), and the moment a screenshot is used as evidence the trap's discipline governs it: a captured frame a wrong implementation could equally produce is not evidence, so the probe must carry a positive control that separates the three states T1 measured. |

**Review lane — raised here, and the raise is a finding.**
[preamble.md](./preamble.md)'s review-lane table assigns T4 **Normal
review**, grouped with T8, on the stated ground "**Additive per-window
state** (T4), test-only (T8)". Re-checked against what T4 actually does,
that ground is **incomplete in the same way F-17 found T3's to be**: the
`DipScale` field is additive, but the task also inserts a call that
**re-enters `wnd_proc` synchronously in the middle of `window::create`**,
on the single path all three example hosts and both public window-create
entries take. [gates §4](../../../procedures/implementation-gates.md)
names *runtime structural change* as a high-risk class, and the preamble
itself justifies T7's full lane as "runtime structural change with
**re-entrancy through the message loop**" — which is the same property,
arriving four tasks earlier and at a point where the object it re-enters
is half-constructed.

Recorded as **F-25** and dispositioned to **full independent review**,
consistent with the owner's T3 decision that such a review is discharged
by Codex against a written brief and that the merge gate is blocked until
its findings are dispositioned. The consequence is raised with the owner
at the merge gate rather than absorbed silently. This is the **fourth**
phase-wide or table-level judgment narrowed by what actually landed —
trap #4 at T2 (F-12), the review lane at T3 (F-17), trap #3 at T3
(F-22), and the review lane again here.

**Planned proof obligations** (each closed at the close gate):

1. The trap-#1 call-site audit: every `window::create` caller, the one
   `CreateWindowExW`, the one `WindowState` literal, and both
   `GetClientRect` consumers, each classified and each stated as covered
   by the correction or as reading after it.
2. The trap-#2 enumeration of what the nested synchronous dispatch
   finds and touches — with the message set **measured**, not asserted
   from the ADR's wording.
3. **Decision 1 — the DIP → physical window-size rounding rule**, with
   the rejected candidates and their rejection reasons, and the separate
   sub-decision of whether the rule belongs inside `DipScale`.
4. **Decision 2 — where in `window::create` the correction runs and with
   which flags**, recorded rather than fallen into, with the `SWP_NOMOVE`
   / `SWP_NOZORDER | SWP_NOACTIVATE` contrast against DD-003's
   `WM_DPICHANGED` path stated as part of the decision.
5. **The ordering claim discharged by construction and by measurement,
   not by comment.** The end gate asks that the scale be seeded before
   the first layout "verified by ordering rather than by comment". Two
   artifacts, because the plan's own wording is a description and a
   description is what the gate exists to exclude:
   - *By construction* — the scale is a field of `WindowState`, so there
     is no window whose scale is unset and no statement order a later
     edit can invert. `set_root`, the first layout, cannot run before a
     `WindowState` exists.
   - *By measurement* — a **throwaway probe**, in T1's shape (instrument,
     build, run, capture, revert), printing the actual event order
     through `create` and the nested dispatch.
6. **The probe carries a positive control, because T4 has no other
   discriminating evidence.** At scale 1 every candidate implementation —
   including one that never applies the correction at all — produces
   identical output, which is F-4's problem in its sharpest form. The
   probe therefore also adds the T9 declaration **as throwaway** and
   measures the third of F-9's three states: T1 measured *unaware* (window
   1000 × 750 physical by DWM stretch, gallery WrapPanel 7 tiles per row)
   and *aware without the correction* (800 × 600 physical, 6 tiles). The
   state neither measured is **aware with T4's correction**, which must be
   1000 × 750 physical *and* 7 tiles — the pair being what separates it
   from both baselines, since the rectangle alone is satisfied by the
   unaware build (F-9) and the tile count alone by the unaware build too.
   The declaration is reverted with the rest of the probe; T9 remains the
   task that lands it.
7. Regression check only, per the owner-agreed downgrade:
   `cargo build -p wasamo-runtime` → `cargo build --workspace` →
   `cargo test --workspace` green (the F-5 ordering, used as a matter of
   course), plus `cargo fmt --all -- --check` and `git diff --check`.
8. **The plan re-audit as an in-gate item, not an after-gate one**:
   [plan.md](./plan.md) §T5 … §T12 and [preamble.md](./preamble.md)
   re-read in order, with a task-by-task verdict table. T3 was the first
   run that detected items unprompted; this is the second.

**Open decisions, named by the plan and deliberately not pre-empted
here.** The criteria are fixed now, before an answer is looked for; both
are recorded as taken in their own sections below.

- **Decision 1 — rounding.** `SetWindowPos` takes `i32`; 801 DIP at 150%
  is 1201.5. Candidates: `round`, `ceil`, `trunc`. Criteria: (a) it must
  be argued from *this* quantity's contract, not borrowed from
  `surface_pixels`, whose `ceil` exists because a truncated surface clips
  glyph coverage — reaching for it here is the F-14 / F-15 class; (b) it
  must state what it optimises, since T10's 800 × 600 → 1000 × 750 check
  is exact and **cannot discriminate any of the three** (the F-13 shape),
  so the test that does discriminate has to be constructed deliberately;
  (c) the sub-decision of whether the rule lives in `DipScale` turns on
  whether a call-site expression would have to reach for `factor()` —
  which F-15's carry-forward names as a re-trigger criterion, legitimate
  only for T6's `96 × s`.
- **Decision 2 — placement and flags.** The seam is the `GWLP_USERDATA`
  install at `window.rs:83`. Criteria: (a) whichever side is chosen, the
  reason must be a property of the code rather than a comment, because
  T5 makes the `WM_SIZE` arm divide by the window's scale and T7 makes
  the ordering a correctness constraint; (b) the flags are part of the
  decision, and DD-003's `SWP_NOZORDER | SWP_NOACTIVATE` **must not be
  copied verbatim** — that pair belongs to the path that *applies* the
  OS-suggested rectangle, and here `CW_USEDEFAULT` placement means the
  window must not move.

**Commit shape.** Two code commits, against the one-commit-per-item
default. The rounding rule lands alone in
[`dip_scale.rs`](../../../../wasamo-runtime/src/dip_scale.rs) with its
tests and no caller — the module already carries the forward-pointer
`allow(dead_code)`, so that commit is buildable and warning-free. The
`Win32_UI_HiDpi` feature, the `WindowState` field, its seeding and the
correction then land together: a commit that adds the field without its
consumer emits a never-read-field warning, and the feature flag is the
prerequisite for the `GetDpiForWindow` call that appears in the same
commit.

### The decision — the DIP → physical window-size rounding rule

Taken against the criteria fixed at the start gate. **Decided: round to
nearest, and the rule lives inside `DipScale` as
`window_size_to_physical((i32, i32)) -> (i32, i32)`.**

`SetWindowPos` and `CreateWindowExW` take `i32` device pixels;
`wasamo_window_create`'s `width` / `height` are `i32` DIP. This is the
phase's **second and last** place where a real number becomes an
integer, and the plan's warning was that borrowing `surface_pixels`'
`ceil` would be the F-14 / F-15 class — a rule written for one purpose
reused for another because it was to hand.

| Candidate | Verdict |
|---|---|
| **`round`** | **Taken.** The two rules differ because the two quantities do. `surface_pixels` rounds *up* because a surface is an **allocation** and a truncated one clips the final column of glyph coverage — a one-sided failure, so a one-sided rule. Nothing is clipped by a window half a pixel small: the client extent is read back through `GetClientRect` and converted, never assumed, so the failure is two-sided and its magnitude is what matters. What this quantity carries instead is a **logical-size fidelity contract** — an 800 DIP window is meant to *be* 800 DIP on every monitor — and nearest is the integer that minimises the DIP error the author observes. Second, independent ground: nearest is `MulDiv(v, dpi, 96)`, which is what the OS itself uses to compute the `WM_DPICHANGED` suggested rectangle that T7 applies **verbatim**. Choosing it makes creation and the OS's later suggestion produce the same number, instead of two sources of a window rectangle that disagree by a pixel and drift on every monitor crossing. |
| `ceil` | Rejected. Consistent with `surface_pixels` only in appearance: the reason for that rule is clipping, and there is no clipping here. It biases every window's realised logical size upward — by up to `1/s` DIP per axis, on every window, forever — in exchange for nothing stated. |
| `trunc` | Rejected on the same ground `surface_pixels` rejects it, plus the bias argument in the other direction. |

**Sub-decision — the rule lives in the type, not at the call site.**
The plan framed this as "a second rounding contract in the type, which
needs its own test" against "the type's single-rounding-contract story
stays intact". The deciding fact is what the call site would have to
write. `window::create` holds an `i32` DIP pair and a `DipScale`; without
a named operation the seam reads either
`(width as f32 * scale.factor()).round() as i32` — which reaches for
`factor()`, the exact expression **F-15's carry-forward names as its
re-trigger criterion and permits only for T6's `96 × s`** — or
`scale.to_physical(width as f32).round() as i32`, which leaves the
*integer* rule at the seam while the type owns the other integer rule.
That is F-14's two-homes shape, and it also leaves an `f32` in the
caller's hand one keystroke away from `as i32`, which truncates silently.

So the count of contracts in the type is not the thing to preserve —
the count of **places a rounding rule lives** is, and that is what the
type was introduced for. The type now owns both integer rules, each
named after what it converts, each documented against the other, and
each with tests that fire.

**One consequence of the integer signature, decided rather than
inherited.** The arithmetic widens to `f64`. At 100% the conversion is
then the **exact identity for every `i32`**, including values above
2^24 that `f32` cannot represent — so "T2 through T8 land into a world
where every conversion is the identity" is a property of the type here
rather than of the magnitudes hosts happen to pass. No `s == 1`
short-circuit is added to buy that; a branch would be the thing trap #4
is about.

*Record-keeping note.* The start gate's §Commit shape paragraph already
said this rule would land in `dip_scale.rs`, which anticipated the
sub-decision's outcome before this section argued it. Recorded rather
than quietly re-ordered: the criteria above were fixed first and the
argument is the one that was run, but the gate's "decide before
choosing" discipline was not perfectly clean on this point.

### The decision — where in `window::create` the correction runs, and with which flags

**Decided: immediately after `create_hwnd` returns — before the
`WindowState` is boxed and before `GWLP_USERDATA` is installed — with
`SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE`.**

The seam the plan named is `window.rs`'s `SetWindowLongPtrW` call. Before
it, `wnd_proc` reads a null pointer and falls through to
`DefWindowProcW`; after it, the nested messages enter the live arms with
`root_widget` still `None`.

| Candidate | Verdict |
|---|---|
| **Before the `GWLP_USERDATA` install** | **Taken.** The nested dispatch **cannot reach runtime state at all** — a property of where the call sits, not of the fact that the arms it would otherwise enter happen to be no-ops today. That distinction is the whole point: T5 puts a division by `state.scale` in the `WM_SIZE` arm and T7 makes ordering a correctness constraint, so "it is harmless because `root_widget` is `None` and `resize_fn` is `None`" is a claim about the *current* bodies of two `if let`s, which is exactly the kind of reasoning a later task invalidates without noticing. It also keeps the window's rectangle correct from the earliest possible moment — before the `DesktopWindowTarget` is attached and before the root Visual's `SetRelativeSizeAdjustment(1, 1)` starts tracking a client area — rather than resizing underneath them. |
| After the `GWLP_USERDATA` install | Rejected. Its attraction was symmetry with T7 — same shape, one mechanism, exercised on every startup. But the symmetry is **false**: at T7 the window is fully built and the nested `WM_SIZE` is *required* to run the re-layout, while at creation it must do nothing and there is no root widget for it to lay out. Making the two look alike would invite a reader to transfer T7's ordering reasoning to a site that does not have it, and would re-enter a `WindowState` that `emit::register_window` has not yet seen. |

**Flags.** `SWP_NOMOVE` is required and is the part that must **not** be
copied from DD-003. Placement is `CW_USEDEFAULT`'s choice, so `x` / `y`
have no meaningful value to pass and the window must stay where the OS
put it. DD-003's `SWP_NOZORDER | SWP_NOACTIVATE` pair belongs to the
`WM_DPICHANGED` path, which **applies an OS-suggested rectangle** and
therefore moves the window deliberately; copying that pair verbatim here
would move the window to `(0, 0)`. `SWP_NOZORDER` and `SWP_NOACTIVATE`
are kept for their own reasons — the window is not yet shown and neither
its Z-order nor the foreground focus is this function's business. No
further flag is added: `SWP_NOREDRAW` and `SWP_NOSENDCHANGING` would each
be a behaviour choice with nothing asking for it.

**Failure handling.** The result is discarded. DD-003 §Failure handling
fixes the posture — log and survive; a failed `SetWindowPos` leaves the
rectangle unchanged and nothing tears the window down — and the runtime
has no logging facility, so `let _ =` is the same shape `apply_mica` and
`show` already use in this file. Recorded rather than left implicit: the
disclosure mechanism DD-001 uses is the ABI-layer thread-local
last-error, and reaching into it from `window::create` would be a new
diagnostic surface with no test, which is the trap #4 shape. **The
consequence is stated, not hidden**: if the correction fails, the window
keeps the requested numbers as physical pixels and is visibly small at
any scale but 100%.

### Close gate

**#1 — call-site audit.** The claim under check: *every path that turns
a host's DIP window size into an HWND passes through the corrected site,
and every path that reads a window's geometry reads it after the
correction.* Queries over `wasamo-runtime/src`, `wasamo-dll`,
`bindings/rust/src`, `wasamoc/src`, `examples`:
`WindowState \{|CreateWindowExW|GetWindowRect|SetWindowPos|GetClientRect|window::create|GetDpiFor`.

**The query was too narrow to be the forcing artifact it claims to be**
(T4 independent review finding R-8): it names the APIs this task
*happens to use* and so cannot exclude the ones it does not —
`MoveWindow`, `AdjustWindowRect` / `AdjustWindowRectExForDpi`,
`SetWindowPlacement`, `DeferWindowPos`, `GetWindowPlacement`. Re-run with
those terms added over the whole repository: **no hit in any runtime
source**, so the table's conclusion is unchanged and the reviewer reached
it independently. The lesson is about the artifact rather than the
result — an audit query assembled from the diff cannot falsify itself,
and the reviewer's extension is what made "there is no second path" a
checkable statement instead of a restatement.

| Site | Classification | As landed |
|---|---|---|
| `abi.rs:335` `wasamo_window_create` → `window::create` | must be covered | covered — the correction is inside the callee, so the ABI function is untouched |
| `abi.rs:1224` `wasamo_load_ui` → `window::create(title, 800, 600)` | must be covered, **and is the one T1's F-7 said would have been missed** | covered. This is the path all three example hosts take; it never passes through `wasamo_window_create`, so a correction placed there would have mis-sized every host |
| `lib.rs:88` `window_create` (Rust-native) → `window::create` | must be covered | covered — same callee |
| `window.rs` `create_hwnd` → the single `CreateWindowExW` | the site that consumes the uncorrected numbers | unchanged. `width` / `height` still reach it as the caller's DIP integers; correcting *before* creation is impossible because the monitor, hence the DPI, is not known until the HWND exists (DD-003 I1, `CW_USEDEFAULT`) |
| `window.rs` the single `WindowState` literal | must carry the scale | carries `scale`, from the value read once after `create_hwnd` |
| `window.rs:160` `set_root`'s `GetClientRect` → first layout | must read after the correction | reads after: the correction runs inside `create`, and `set_root` is a separate call every caller makes later. **Measured** — probe step 9 |
| `emit.rs:137` `flush_layout`'s `GetClientRect` (audit row 2b, T1's F-1) | must read after the correction | reads after, necessarily: the reactive drain cannot run before the window exists. Not otherwise touched by T4 |
| `GetWindowRect` | — | none in the runtime, before or after |
| `SetWindowPos` | the new site | exactly one, in `realize_dip_window_size` |
| `GetDpiForWindow` | the new site | exactly one, in `create` |

**This closes DD-002 audit row 13** (`create_hwnd`'s `CreateWindowExW`
width / height, "DIP → physical, per DD-003"). Recorded here because
**no task in [plan.md](./plan.md) claimed that row** — §T5 covers rows
1–6 and 8–12, §T6 covers row 7, and row 13 was named by neither. See
F-26.

**#2 — structural side-effect enumeration.** What the inserted
`SetWindowPos` drags along. The message set is **measured**, not taken
from the ADR's wording: a throwaway probe (below) instrumented
`wnd_proc` behind a flag set only around the correction call.

| # | Effect | Measured / stated |
|---|---|---|
| 1 | **Nested synchronous dispatch, size unchanged (`s = 1`, the shipped state)** | `WM_WINDOWPOSCHANGING` (0x0046), `WM_GETMINMAXINFO` (0x0024). **No `WM_SIZE`.** |
| 2 | **Nested synchronous dispatch, size changed (`s = 1.25`, declaration added as throwaway)** | `WM_WINDOWPOSCHANGING`, `WM_GETMINMAXINFO`, `WM_NCCALCSIZE` (0x0083), `WM_WINDOWPOSCHANGED` (0x0047), **`WM_SIZE` (0x0005)**, then three `WM_GETICON` (0x007F). DD-003's load-bearing property — `SetWindowPos` dispatches `WM_SIZE` before it returns — is therefore **measured**, not inherited |
| 3 | **What those messages reach** | Nothing. `state_ptr` was null for **every** message in both runs, so all of them went to `DefWindowProcW`. That is the placement decision's artifact |
| 4 | `resize_fn` / `mouse_*_fn` callback slots | **unchanged** — unreachable during the nested dispatch (row 3), and no ABI or Rust-native function installs them anyway (T1 finding F-3, confirmed) |
| 5 | `root_widget` and the layout pass | **unchanged.** `root_widget` is `None` for the whole of `create` by construction — every caller calls `set_root` afterwards — so no layout runs inside the correction under either placement |
| 6 | `emit::register_window` | **unchanged, and deliberately after.** The correction precedes it, so a nested message cannot reach a window the emit registry has not yet seen |
| 7 | The root Visual's `SetRelativeSizeAdjustment(1, 1)` (audit row 8) | **unchanged**, and now also *unaffected by ordering*: the correction runs before `create_desktop_window_target`, so the target is attached to a window that is already the right size rather than resized under it |
| 8 | Window **position** | **unchanged** — `SWP_NOMOVE`. Measured: the captured windows sat at `(192,192)`, `(256,256)` and `(33,33)` across the three probe runs, i.e. wherever `CW_USEDEFAULT` put them, never `(0,0)` |
| 9 | Window **Z-order / activation** | **unchanged** — `SWP_NOZORDER \| SWP_NOACTIVATE`, and the window is not yet shown |
| 10 | The **client** extent | changes with the outer rectangle, and **not by the same factor**. Measured at 125%: outer 800 → 1000 DIP-exactly, client 784 × 561 → 982 × 703 physical, which is 785.6 × 562.4 DIP. The non-client frame scales by its own **DPI-indexed system metrics**, not by `s`. *(Mechanism corrected at the independent review, finding R-6: the original entry derived "8 px per side at 96 DPI, 9 px at 120" from the width alone, which does not account for the height at all. `GetSystemMetricsForDpi` on the probe machine gives `SM_CXSIZEFRAME` 4 / 4, `SM_CXPADDEDBORDER` 4 / 5, `SM_CYCAPTION` 23 / 29 at 96 / 120 DPI, so width is 2 × (4 + 4) = 16 → 2 × (4 + 5) = 18 and height is 2 × (4 + 4) + 23 = 39 → 2 × (4 + 5) + 29 = 47 — both matching the measured rectangles exactly. Re-measured independently here, not taken on the reviewer's word. These are this machine's theme metrics; the invariant is that they are DPI-indexed and independent of `s`.)* See F-28 — this is a real qualification on T10's control B |
| 11 | The **requested DIP size** | **not retained.** `create` keeps the scale and not the pair, so there is no second representation of the window's logical size to drift. This is the trap-#3 exclusion the start gate promised to check |
| 12 | Reactive drain / signal registry / binding state | **unchanged** — no property is written, no node is created, nothing is enqueued. Window creation does not enter the drain and this task does not change that |
| 13 | Behaviour at `s = 1` | **the exact identity, measured.** Probe steps 3 and 6, same run: window `800x600` client `784x561` before the correction and `800x600` / `784x561` after it, with the correction target printed as `(800, 600)` |

**#4 — the authored branch that was not written.** DD-003 I1 words the
correction as "**if the scale is not 1** apply `size × s`". Written that
way it is a branch reachable only once T9 lands, on the path every host
takes, and it would sit directly against DD-001 §Failure handling's
structural argument — that tolerating a failed declaration is safe
*because the conversion machinery has no second code path to keep
correct*. The correction is therefore unconditional, and the artifact for
this trap is the absence: `realize_dip_window_size` has no conditional,
`from_dpi` already floors a zero DPI so F-16's second candidate branch is
not written either, and the only new arithmetic ships with tests shown to
fire (below).

The rounding rule takes T2's evidence standard voluntarily — the owner's
decision makes it mandatory only once the vision decision record lands at
phase end, and T4 is not a pure-logic task, but the operation is pure
logic and a green test that has not been shown to fire says nothing
(F-11). Four wrong implementations, each applied to a
restored-from-backup copy, run with
`cargo test -p wasamo-runtime --lib dip_scale`, then reverted:

| Mutation | Tests that failed | |
|---|---|---|
| **W1** `round` → `trunc` | `window_size_rounds_to_nearest_in_both_directions` | the rejected candidate |
| **W2** `round` → `ceil` | `window_size_rounds_to_nearest_in_both_directions` | the other rejected candidate — and note it does **not** redden `window_size_converts_at_125_150_200_percent`, which is F-13's shape measured for this rule: every exact product, T10's 800 × 600 → 1000 × 750 included, is satisfied by all three candidates |
| **W3** `f64` → `f32` arithmetic | `window_size_is_the_exact_identity_at_one_hundred_percent` | the identity claim is about the type, not about typical magnitudes |
| **W4** the second axis reuses the first | `window_size_converts_at_125_150_200_percent`, `window_size_rounds_to_nearest_in_both_directions` | |

Final state re-run green: **14 tests** in `dip_scale`, 0 failures.

**#5 — carry-forward.** Three invariants, each with a re-trigger
criterion; recorded in [handoff.md](./handoff.md).

1. **The scale is per window and authoritative on `WindowState`**, seeded
   once from the monitor the OS chose. *Re-trigger:* any second source of
   a window's scale — a process-global cache, a monitor query at a point
   of use, a second seeding site — reintroduces exactly the drift DD-003
   rejected as option S1/S3, and M4-Phase 8 is where it would show.
2. **`realize_dip_window_size` runs before `GWLP_USERDATA` is
   installed, and its flag set is not the `WM_DPICHANGED` path's.**
   *Re-trigger:* any task that moves the correction later, that adds a
   geometry call to `create` after the pointer install, or that reuses
   this helper for the change path. The second is T7's live hazard — see
   F-30 — because the helper converts a **DIP size** while T7 applies an
   **OS-supplied physical rectangle**, and inheriting `SWP_NOMOVE` there
   would pin the window on every monitor crossing.
3. **The window's logical size is realised on the *outer* rectangle, and
   the client rectangle does not follow by the same factor.** *Re-trigger:*
   any evidence gate or future `WindowConfig` attribute that treats a
   DIP client size as derivable from the DIP outer size. Measured at
   125%: 800 DIP outer is exactly 1000 physical, while the client is
   785.6 DIP rather than 784.

**#6 — deterministic-failure disposition.** None to disposition. The
suite was green on every run; Observation 5's
`scroll_view_layout_integration` access violation did not appear. One
non-failure surprise was root-caused rather than worked around: the
first mutation loop restored `dip_scale.rs` with `Copy-Item`, which
**preserves the source file's modification time**, so cargo's
mtime-based fingerprint considered the build current and re-ran the
*mutated* test binary — reporting the last mutation's result as the
restored state's. Caught because the restored run was expected green and
was not. The loop was rewritten to restore by writing content (which
updates the mtime), and the restored state then measured green. Recorded
because it is a **false-negative generator for any mutation loop**, the
same family as F-21 and F-5: a build step that looks like it did
something and did not.

**#7 — GUI evidence, and what the probe does and does not show.** T4's
landing changes no rendered frame, so this is evidence for the *ordering
and correction claims*, not a rendering gate. Method: throwaway
instrumentation → `cargo build --release --workspace` (F-21) → launch →
`CopyFromScreen` over `GetWindowRect` from a Per-Monitor-V2 capture
process → revert. **The window is never moved or resized**, unlike T3's
script: the size it was created at *is* the measurement. Script:
[evidence/capture-t4-probe.ps1](./evidence/capture-t4-probe.ps1); frames
in [evidence/t4-probe/](./evidence/t4-probe/). Environment: the 125%
development machine.

The ordering, printed rather than described — the `s = 1.25` run, which
is the only one where the ordering can be wrong at all:

```
PROBE 0 SetProcessDpiAwarenessContext(PMv2) -> Ok(())
PROBE 1 create(width=800, height=600) [DIP]
PROBE 2 CreateWindowExW returned; GetDpiForWindow=120 scale=1.25
PROBE 3 before the correction: window=800x600 client=782x553
PROBE 4 correction target = (1000, 750)
PROBE 5   nested dispatch during SetWindowPos: msg=0x0046 ... state_ptr_null=true
PROBE 5   ... 0x0024, 0x0083, 0x0047, 0x0005 (WM_SIZE), 0x007F x3 — all state_ptr_null=true
PROBE 6 after the correction: window=1000x750 client=982x703
PROBE 7 GWLP_USERDATA installed (wnd_proc can now reach WindowState)
PROBE 8 create returns; state.scale=1.25
PROBE 9 set_root first layout: client=(982x703) physical, state.scale=1.25, => 785.6x562.4 DIP
```

**The positive control: three states, all measured in one session on one
build tree**, rather than two cited from T1 and one predicted. Each pair
of rows shares one column, so **no single number separates the three**
and only the pair does:

| State | build | window rect | client | tiles/row | frame |
|---|---|---|---|---|---|
| **A** — unaware + correction (**what T4 ships**) | as committed | **1000 × 750** | 980 × 701 | **7** | `unaware-with-correction.png` |
| **B** — aware, correction suppressed | throwaway declaration, correction call removed | 800 × 600 | 782 × 553 | **7** | `aware-without-correction.png` |
| **C** — aware + correction | throwaway declaration only | **1000 × 750** | 982 × 703 | **9** | `aware-with-correction.png` |

Read as a set: **A and C are indistinguishable by the window rectangle**
— which is F-9 measured directly rather than quoted — and **A and B are
indistinguishable by the tile count**. A build that never declares
awareness (A) and one that declares and corrects (C) both measure
1000 × 750, and only the WrapPanel separates them; a build that declares
without correcting (B) is separated by the rectangle and not by the
tiles. This is why the plan's window-measurement check is not a control
on its own, and the probe now says so from its own numbers.

**Two things the probe falsified, both recorded as findings rather than
smoothed over.**

- The control predicted at the start gate was **wrong**: aware + correction
  was expected to restore the unaware baseline's 7 tiles, and it reads 9.
  That is the *correct* answer for T4 alone — the correction fixes the
  window's outer rectangle while the **inbound** conversion that would
  divide the client extent back into DIP is T5's, so layout receives 982
  and treats it as DIP. The prediction assumed the whole machinery for a
  task that lands half of it. F-27.
- The plan's stated failure signature — "with awareness declared and T4's
  correction absent … the WrapPanel drops from 7 tiles per row to 6" —
  does **not** reproduce at T4: state B reads 7. T1's 6 is not wrong; T1's
  throwaway carried the *complete* conversion machinery, so its client
  782 physical became 625.6 DIP. The number is right for T1's build and
  the plan restates it without that condition. F-27.

**Crispness, stated as an observation and not as evidence.** State A's
glyphs are visibly softer than state C's in the captured frames — the
DWM stretch — which is consistent with T1's magnified pair and with R-1's
premise. It is not offered as evidence: the comparison here is at
capture resolution with no magnification, the two frames differ in more
than one variable, and positive control A is T10's with its own capture
discipline.

**End-gate items from [plan.md](./plan.md) §T4.**

- *Scale seeded before the first layout, verified by ordering rather than
  by comment* — discharged twice. **By construction**: the scale is a
  `WindowState` field initialised in the struct literal, so there is no
  window without one and no statement order for a later edit to invert;
  `set_root`, the first layout, cannot run before a `WindowState` exists.
  **By measurement**: probe steps 2 → 8 → 9 above, where step 9 reads the
  seeded 1.25 off the state at the first layout.
- *Workspace green as a regression check only* —
  `cargo build -p wasamo-runtime` → `cargo build --workspace` →
  `cargo test --workspace` (the F-5 ordering, used as a matter of
  course): **32 test binaries, 0 failures, 949 tests**, the runtime lib
  going 457 → 460 as the three new `dip_scale` tests land. Per the
  owner-agreed downgrade this is a regression check and nothing more —
  and T4 is a task where that is unusually visible, since **every one of
  the three probe states above is green**, including the two that are
  wrong.
- `cargo fmt --all -- --check` and `git diff --check` clean.
- *Throwaway reverted* — `git status` clean of `wasamo-*` changes; the
  instrumentation and the declaration exist only in this record. The
  declaration remains T9's to land.

### Plan-hypothesis re-audit (2026-07-28, in-gate — not owner-prompted)

T1 and T2 each revised only the tasks their findings *pointed at* and
were caught by the owner; T3 ran the re-audit as an item inside the close
gate and found six items unprompted. This is the second run of that
shape. [plan.md](./plan.md) §T5 … §T12 and [preamble.md](./preamble.md)
(the review-lane table, §Implementation gates, §Technical risks, §The
sequencing thesis, §Verification closure, §Obligations carried) were
re-read **in order** against what T4 landed and measured. Verdicts,
every entry, including the ones with nothing to correct:

| Re-read | Verdict |
|---|---|
| §Task list preamble (gate-substitution table, commit rules) | **correction**: the table names T2, T3, T5, T6, T7 and T8 and **omits T4** — reasonable when T4 was thought to have nothing to show, false once its real gate turned out to be the three-state probe. T4's row added |
| §T4 | corrections: checklist ticked, both decisions recorded as taken, the two-commit landing recorded, and the inertness claim sharpened from "the correction is a no-op" to "no `WM_SIZE` is dispatched at all, so the placement's failure mode is unreachable" |
| §T5 | **corrections**: **F-26** (audit row 13 had no owner in the plan; T4 closes it) plus the two pickups T4 leaves — the `pub(crate)` visibility that makes row 2b reachable without widening the API, and the `#[allow(dead_code)]` T5 removes as its first reader |
| §T6 | no additional correction. T4 touches nothing T6 depends on: `surface_pixels` is unchanged, the walk's callers are unchanged, and the scale reaches T6 through the node cache T5 introduces rather than through `WindowState` |
| §T7 | **correction — F-30**: T7 must not inherit `SWP_NOMOVE` nor reuse `realize_dip_window_size`. Plus a confirmation folded into the step-ordering bullet: the synchronous `WM_SIZE` premise is now **measured**, and so is its absence at `s = 1` |
| §T8 | **correction — F-29**: the cached-scale assertion has no reachable field. `pub(crate)` plus a separate test crate means T8 must add a `#[doc(hidden)] pub` seam in `lib.rs::ffi`, and must **not** widen the field |
| §T9 | **correction**: `Win32_UI_HiDpi` is measured sufficient for the declaration symbols, so T9 needs no `Cargo.toml` edit — stated with its limit, since the two query symbols the effective-level assertion uses were not exercised |
| §T10 | **corrections — F-27** (the three-state table measured in one session; the "7 → 6" signature was T1's number for a build carrying the full machinery, and does not reproduce at T4) and **F-28** (control B's invariance is not bit-exact, because the client rectangle does not scale by `s`) |
| §T11 | no additional correction — owner-executed, and nothing T4 landed changes what the owner is asked to observe |
| §T12 | **correction**: DD-004's outer-window-rectangle claim, flagged at ADR time as most at risk, now has its measurement; recorded with the trap that the same claim is **false** of the client area, so the Moment 2 wording must stay where DD-004 put it |
| preamble §The sequencing thesis | **correction — F-31**: the thesis's cost paragraph is written for arithmetic and understates the cost for ordering decisions |
| preamble §Verification closure | no additional correction. Evidence item (2)'s task mapping is unchanged; F-29 is about *how* T8 reaches the value, not about which task owns the claim |
| preamble §Obligations carried | no additional correction |
| preamble §Implementation gates | **correction**: trap #4's narrowing gains T4 as a third site — one where the judgment survives only because the approach was chosen against DD-003's own "if the scale is not 1" phrasing |
| preamble review-lane table | corrected at the start gate — **F-25** |
| preamble §Technical risks | **correction**: R-9 sharpened in both halves — the placement risk is worse than "inert" and the arithmetic risk is now measured rather than deferred to T10 |

Seven findings, six of them in tasks T4 never touched.

- **F-25 — the T4 review lane's stated ground was incomplete.** Recorded
  at the start gate and folded there. Listed here so the count is
  honest, not carried twice.
- **F-26 — DD-002 audit row 13 had no owner.** The plan assigns rows 1–6
  and 8–12 to §T5 and row 7 to §T6. **Row 13** — `create_hwnd`'s
  `CreateWindowExW` width / height, "DIP → physical, per DD-003" — is
  named by neither, and §T4's bullets describe the correction without
  ever saying it closes an audit row. The consequence is not
  hypothetical: T5's end gate is "DD-002's 13 rows, each with its
  classification", so a T5 that closed every row it had been handed would
  have produced a 12-row table and called it 13, on the one task whose
  artifact *is* completeness. Closed by T4's call-site audit above.
  *Disposition:* [plan.md](./plan.md) §T5's end gate states where row 13
  is closed and why the gap existed.
- **F-27 — the "7 tiles → 6" failure signature does not reproduce at
  T4, because it was never T4's number.** [plan.md](./plan.md) §T10 reads
  "with awareness declared and T4's correction absent, the window
  measures 800 × 600 physical and the WrapPanel drops from 7 tiles per
  row to 6". Measured at T4 with the declaration added as throwaway and
  the correction removed: **7 tiles**, not 6. T1's number is not wrong —
  T1's throwaway carried the *complete* conversion machinery, so its
  client extent of 782 physical became 625.6 DIP — but the plan restates
  it as a property of the missing correction alone, and read that way it
  is false. This is the same shape as S-3 at T3: **a correction applied
  to the document that carried the finding and not to the document that
  summarised it**, one level further along. T4 replaces the citation with
  three states measured in one session, and the more useful reading falls
  out of them: rows 1 and 3 share a window rectangle, rows 1 and 2 share
  a tile count, so **no single number separates the three**. Also
  recorded: the aware-plus-correction row reads **9** tiles only because
  T5's inbound seam is absent, and must read 7 once it lands — a T10 that
  inherited 9 would pin a half-finished phase. *Disposition:*
  [plan.md](./plan.md) §T10.
- **F-28 — positive control B's invariance is not bit-exact, and a
  control that assumes it is can fail a correct build.** DD-004 defines
  `width` / `height` as the **outer** rectangle, and that is the
  rectangle the correction scales exactly: 800 × 600 DIP → 1000 × 750
  physical at 125%. The **client** rectangle does not follow by the same
  factor. Measured: 784 × 561 at 96 DPI and 982 × 703 at 120 DPI, i.e.
  785.6 × 562.4 DIP — because the non-client frame scales by its own
  DPI-indexed system metrics rather than by `s` (decomposition in the
  side-effect enumeration row 10, corrected at the review). Layout
  receives the client extent, so a correct implementation lays out into
  about 1.6 DIP more width at 125%, and a wrap position sitting near a
  line-break boundary may legitimately move. [plan.md](./plan.md) §T10
  states control B as "wrap positions and element order compared" with
  "invariance is the evidence", which as written would redden a correct
  build. *Disposition:* §T10 gains the tolerance and the alternative
  (drive both captures from a controlled *client* size).
- **F-29 — T8's first assertion has no reachable field.** "A created
  window's cached scale equals `GetDpiForWindow`" is an integration-test
  claim, and the phase's Windows integration tests live in
  `wasamo-runtime/tests/` — a separate crate, which can reach only `pub`
  items. T4 landed `pub(crate) scale`, deliberately: DD-004 walks every
  M4 phase and concludes no host needs the scale factor, and `WindowState`
  is `pub use`-exported, so `pub` would ship exactly the surface that
  decision declines. The fix is the established `#[doc(hidden)] pub`
  seam in `lib.rs::ffi` beside `__install_owning_thread_for_test`, and
  naming it now is the point — a T8 that meets this mid-edit is one
  keystroke from widening the field instead. *Disposition:*
  [plan.md](./plan.md) §T8.
- **F-30 — T7 must not inherit T4's flags, and the plan only warns in
  the other direction.** §T4 warns against copying DD-003's
  `SWP_NOZORDER | SWP_NOACTIVATE` into `create`, because `CW_USEDEFAULT`
  placement means the correction must not move the window. The reciprocal
  hazard is now the live one: `window::realize_dip_window_size` exists,
  is the only `SetWindowPos` in the runtime, and passes `SWP_NOMOVE` —
  and T7's step 2 applies an **OS-supplied physical rectangle** whose
  entire content is a new position *and* size. Inheriting `SWP_NOMOVE`
  there, or reusing the helper (which converts a **DIP size**, a
  different input entirely), would pin the window on every monitor
  crossing while every test stayed green. A T7 author arrives at that
  step having just read `create`. *Disposition:* [plan.md](./plan.md)
  §T7, as its own bullet rather than a clause.
- **F-31 — the sequencing thesis costs more for ordering decisions than
  for arithmetic ones, and the plan's cost paragraph is written for
  arithmetic.** [preamble.md](./preamble.md) §The sequencing thesis says
  the cost of landing the declaration last is that "`s ≠ 1` is not
  exercised by the *real* OS path until T9", and F-4 sharpened that with
  "and the suite would not notice either". T4 measured a third, worse
  case. An identity *conversion* still executes: the multiplication is in
  the code path, and it becomes wrong only when the factor changes. An
  identity *resize* does not even happen — measured, a size-preserving
  `SetWindowPos` dispatches `WM_WINDOWPOSCHANGING` and
  `WM_GETMINMAXINFO` and **no `WM_SIZE` at all**. So the question T4's
  placement decision answers ("what does the nested message find?") has
  no answer to get wrong before T9: the failure mode is not invisible,
  it is unreachable. Any task placing work relative to a message dispatch
  — T4's correction, T7's step ordering — therefore cannot lean on
  "nothing went wrong in this build", and must argue structurally. F-4 is
  "a green suite proves nothing about a conversion"; this is "a green run
  proves nothing about an ordering". *Disposition:* preamble §The
  sequencing thesis, and the reasoning is already what §T4's placement
  decision was argued from.

*Disposition summary:* all folded into [plan.md](./plan.md) and
[preamble.md](./preamble.md) in the same commit as this entry. F-25 →
preamble review-lane table (landed at the start gate); F-26 → §T5;
F-27 → §T10; F-28 → §T10; F-29 → §T8; F-30 → §T7; F-31 → preamble §The
sequencing thesis. Non-numbered corrections in the same batch: §Task
list's gate-substitution table gains T4; §T4's checklist and decisions;
§T5's two pickups; §T7's measured-premise confirmation; §T9's
`Cargo.toml` finding; §T12's outer-rectangle measurement; preamble
§Implementation gates trap #4; preamble R-9.

**Applying T3's derived discipline — correct the summaries, not only the
carriers.** T3's second review round found that its own correction to
F-18 had been applied to the documents that *carried* the claim and not
to the ones that *summarised* it (S-3). Each correction above was
therefore searched for in three forms — full statement, summary, and
quotation — across [plan.md](./plan.md), [preamble.md](./preamble.md),
[handoff.md](./handoff.md), this log, and the ADR set. Results:

- **F-27's "7 tiles → 6"** has **three** sites, not one, and the search is
  what found the third. (i) [plan.md](./plan.md) §T10 — the full
  statement; corrected. (ii) **This log's own T1 §Plan-hypothesis
  re-audit, F-9's closing sentence** ("T1 also measured the third
  outcome (aware, correction absent: 800 × 600 physical, WrapPanel 7
  tiles → 6)") — the same restatement, dropping the same condition,
  *inside the document that carries the original*; corrected in place
  below, following the in-place-correction practice T3 used for F-21's
  mechanism rather than leaving a wrong sentence standing under an
  append-only header. (iii) [log.md](./log.md) §T1's spike-evidence
  section — the original measurement, which states its own reason
  ("because a 800 px physical client area is 640 DIP wide", i.e. the
  inbound division was present) and is therefore **correct as written and
  deliberately left alone**. Not present in the preamble, the handoff, or
  the ADRs. That (ii) exists and (iii) is fine is S-3's shape exactly:
  the original is self-qualifying, the summaries are not.
- **F-28's outer-vs-client** claim appears in
  [DD-004 §What `width` / `height` denote](../decisions/dd-m4-p1-004-unit-contract-and-spec-wording.md)
  and in [DD-003 §Initial scale acquisition](../decisions/dd-m4-p1-003-dpi-change-propagation.md),
  both of which say **outer** and are therefore correct and immutable.
  Two further sites, both left unedited with the reason recorded:
  - [framing.md](../requirements/framing.md) §positive control B states
    the control as "同じ論理サイズのウィンドウ … 折り返し位置・並びを比較 …
    不変であることが正しさの証拠" — the same bit-exactness assumption. It
    is a **phase requirements document, written before the ADR set**, and
    correcting an upstream agreement record from an implementation task
    is not this task's call. The operative correction lives in
    [plan.md](./plan.md) §T10, which is the document T10 executes from;
    the divergence is flagged for T12's Moment 2 pass, which exists to
    reconcile stated claims against what landed.
  - [DD-002](../decisions/dd-m4-p1-002-coordinate-space-and-conversion-boundary.md)'s
    rejection of option C1 says WrapPanel line breaks "can differ between
    scales" under layout-in-physical. That is a claim about layout
    *arithmetic* accumulating `f32` error, and it stands; F-28 is a
    different cause — the client extent handed *to* an unchanged layout —
    so the ADR's argument is unaffected and nothing is being corrected
    there.
- **F-30's flag pair** appears in §T4 (the warning in one direction,
  already there), §T7 (added), and DD-003 (immutable, and correct for its
  own path).
- **F-31** is a new claim with no prior statement to correct; it is
  added to the preamble and reflected in §T4's inertness paragraph.
- **F-26 / F-29** are gaps rather than wrong statements, so there is no
  summary to chase — but both were checked against the preamble's
  §Verification closure table, which maps evidence items to tasks and
  needs no change: F-26 is about which task closes a row, F-29 about how
  a task reaches a value.

### Independent review disposition (Codex, 2026-07-29)

The full independent review the lane raise (F-25) required. Eight
findings: three major, four minor, one nit. **Four of the eight
contradict a claim this log made**, one of them a documented arithmetic
contract the implementation did not keep. Each was re-verified against
the source or re-measured before being accepted; none was taken on the
reviewer's word, and one measurement (R-6's system metrics) was
reproduced independently rather than quoted.

| # | Finding | Verified | Disposition |
|---|---|---|---|
| R-1 | **major** — the `round` contract is not implemented for arbitrary DPI: `from_dpi` rounds the factor into `f32` first, and widening the product to `f64` cannot recover what construction discarded. Witness `dpi = 100, dip = 804`: exactly 837.5, implementation 837 | **Confirmed, and the claim was wrong.** Reproduced the witness, then swept `dpi` 1–600 × `dip` 1–4000 against exact `i128` rational rounding: **21,190 disagreements of 2.4M, all of them on inputs whose true product is an exact half, worst error one pixel — and zero at any of the ten standard Windows scalings**, whose factors are all exact in `f32`. Custom scaling reaches the rest | `DipScale` now **retains the DPI and derives the factor**; `window_size_to_physical` computes `MulDiv` in `i64`. Every `f32` conversion is bit-identical (`factor()` is the expression the field was initialised with). Two tests added — the reviewer's witness plus a property check against `i128` over eleven awkward DPIs. See the two withdrawn claims below |
| R-2 | **major** — F-28's correction reached [plan.md](./plan.md) §T10 but not §T8, not [preamble.md](./preamble.md)'s verification-closure table, and not DD-003's three statements of exact layout invariance | **Confirmed.** The three-form search recorded above chased the *tile-count* claim and the *outer-vs-client* claim, and stopped at the documents phrasing them as **evidence**; it never asked which documents assert the **property** that F-28 qualifies. DD-003 §Context ("does not re-decide a single layout number"), its enumeration row 4 and its §Verification all do | In-phase documents corrected: §T8 now holds the **client** extent constant and says why that is the stronger test, §T8's stated-limit bullet gains the real-path counterpart, and the preamble's item (2) and positive-control discipline carry the qualification. **The ADR set is left untouched** and the divergence is recorded — raised to the owner below, because whether an Accepted DD's property statement needs a successor is not this task's call |
| R-3 | **major** — the unconditional `SetWindowPos` is an observable departure from DD-003 I1's "if the scale is not 1", not an implementation detail; the retrospective's claim that the recommendation was unchanged is wrong | **Confirmed on the reading, and the retrospective sentence is withdrawn.** This log's own probe shows two messages dispatched at `s = 1` that a guarded implementation would not send, so the difference is observable in the runtime's own message stream | The retrospective is corrected. **The implementation is left unconditional pending an owner decision** — raised below with the argument on both sides, rather than settled in an implementation log as it was |
| R-4 | **minor** — `SetWindowPos`'s failure is swallowed, against DD-003's "log **and** survive"; the "no logging facility" reason is false | **Confirmed, and the reason was wrong.** Sixteen `wasamo:`-prefixed `eprintln!` diagnostics exist across `handler.rs`, `ir_loader.rs` and `reactive.rs` | A diagnostic naming the requested DIP size, the physical size that was not realised, the scale, and the surviving state |
| R-5 | **minor** — "the nested dispatch cannot reach runtime state" over-generalises: `WM_DESTROY` and `WM_ERASEBKGND` are handled above the null check and the first calls `PostQuitMessage` | **Confirmed** by reading `wnd_proc` — and the narrow claim (a half-built `WindowState` is unreachable) is what the placement actually needs | Narrowed in the source comment, [log.md](./log.md) and §T4: the two arms above the check are safe because neither is in the **measured** message set, which is a fact about the set and not about the pointer |
| R-6 | **minor** — F-28's mechanism explains the width and not the height | **Confirmed, and re-measured here rather than accepted.** `GetSystemMetricsForDpi` on the probe machine: `SM_CXSIZEFRAME` 4 / 4, `SM_CXPADDEDBORDER` 4 / 5, `SM_CYCAPTION` 23 / 29 at 96 / 120 DPI, so width is 16 → 18 and height is 2 × (4 + 4) + 23 = 39 → 2 × (4 + 5) + 29 = 47. Both match the measured rectangles exactly | Enumeration row 10, [handoff.md](./handoff.md) and §T10 restated with the decomposition, and bounded to this machine's theme metrics — the invariant being that the metrics are DPI-indexed, not the numbers |
| R-7 | **minor** — "nothing is clipped by a window half a pixel small" is too strong; the `GetClientRect` readback guarantees layout sees the realised extent, not that a fixed-size subtree fits | **Confirmed** | `window_size_to_physical`'s doc now claims only what the readback supports: the quantity carries no allocate-at-least-as-much obligation, which is the asymmetry the rejection of `ceil` rests on, and *not* that clipping is impossible |
| R-8 | **nit** — the audit query names the APIs the task uses and so cannot exclude `MoveWindow`, `AdjustWindowRect*`, `SetWindowPlacement`, `DeferWindowPos` | **Confirmed as a defect in the artifact**, not in the conclusion: re-run with those terms over every `.rs` in the repository, **zero hits** | The recorded query is widened, with the lesson stated — an audit query assembled from the diff cannot falsify itself |

**Two claims withdrawn outright, both from the rounding decision.**

1. **"Nearest is what the OS uses to compute the `WM_DPICHANGED`
   suggested rectangle."** Withdrawn. Microsoft's `WM_DPICHANGED`
   contract says only that `lParam` carries a scaled suggested rectangle;
   it does not specify `MulDiv`, and the reviewer is right that the
   `MulDiv` reference on that page is a different example. This was
   flagged as unverified in the review brief and it did not survive.
   **The decision does not fall with it**: ground (a) — a window
   rectangle carries a logical-size fidelity contract, not an allocation
   contract, so the error is two-sided and nearest minimises it — stands
   on its own, and R-7 narrows it without removing it. What is gone is
   the second, independent reason; the remaining claim is the weaker and
   true one that the implementation now computes `MulDiv`'s documented
   semantics, whatever the OS computes.
2. **"At 100% the conversion is the exact identity for every `i32`,
   because the arithmetic widens to `f64`."** True as stated, and beside
   the point: `f64` widening bought exactness only at the one factor that
   was already exact. The claim is now a consequence of integer
   arithmetic rather than the reason for it.

**The mutation set was re-run against the new implementation, and it
found a hole in the tests rather than in the code.** Six wrong
implementations: nearest → `trunc`, nearest → `ceil`, back to the `f32`
factor route, second axis reuses the first, half-away-from-zero → always
up, and **saturate → wrap**. The first five reddened a named test
immediately. The sixth **passed** — no test reached the clamp, because a
100% identity never leaves the range it starts in. `window_size_saturates_rather_than_wrapping`
was added and the mutation then fired. Worth recording plainly: the
technique's value here was not confirming the tests, it was the one case
where it refused to.

**Raised to the owner rather than settled here — and answered
2026-07-29; see §Owner decisions below.** Both are ADR-adjacent and
neither meets the bar for an implementation-log decision.

- **R-2 — does DD-003's exact-invariance property need a successor DD?**
  [DD-003 §Context](../decisions/dd-m4-p1-003-dpi-change-propagation.md)
  states that a DPI change "does not re-decide a single layout number",
  and §Verification asks the integration test to assert unchanged DIP
  results. The *mechanism* is sound — the engine never receives a scale
  — but the *consequence* holds only while the DIP extent handed to
  layout is preserved, and the OS's suggested rectangle preserves the
  **outer** rectangle, whose client area then moves by a DIP or two.
  In-phase this is closed by T8 controlling the rectangle, so nothing is
  blocked. What is open is whether an Accepted DD may carry a property
  statement now known to be inexact on the real path, with the
  correction living only in the plan.

### Owner decisions on the T5 open questions (2026-07-29)

Two of the three raised at the merge gate are answered; the third is the
review's R-1 and is still open.

1. **The F-23 layout-entry fix stays in T5**, as its own commit
   (`7b23854`). The cost the owner weighed was that T5's frame evidence
   then carries two baselines — one for "nothing changed" and one for
   "these two frames changed" — and the deciding question was whether
   that is a one-off or a standing condition. **Measured: one-off.** A
   task's tip is a single state, so T6 onward captures its own baseline
   against the current tree; the two-baseline shape exists only inside
   T5's record because T5 makes two different claims.
   **One thing did leak, and it is now closed.** T3's committed
   [evidence/after/](./evidence/after/) is stale for two frames after the
   F-23 fix — 30,800 of 224,224 pixels each — so a later task reusing it
   as "the last known-good set" would report a regression that is not
   one. That is F-33's own trap with this phase's artifact as the bait.
   The current reference set is named in
   [evidence/README.md](./evidence/README.md).
2. **The frame-reuse procedure is not annotated in the ADR set or in
   [constraints §9](../requirements/constraints.md)** — option (A). The
   operative correction stays in [plan.md](./plan.md) §T6 / §T10, and
   T12 folds it into
   [verification-environments.md](../../../../docs/notes/verification-environments.md)
   Observation 4, which is the document later phases actually read as
   capture procedure. **The owner attached a condition: only if a later
   task can act on it.** That condition was not met as written, and
   meeting it is recorded below.

### Making option (A)'s condition true — a decision rule, not a warning

> **SUPERSEDED (final review S-1). Everything below this line is the dated
> record of what T5 decided at this point and is wrong in its central
> claim.** The rule it builds — "any real change moves geometry, therefore
> a large per-channel delta classifies and a small one clears" — is
> unsound; the sharpest counterexample is T6's own defining failure, a
> wrong D2D context DPI, which changes intensity without moving geometry.
> The `-Exact` switch it describes **no longer exists**. The operative
> description is §Final review disposition below and the header of
> [evidence/compare-frames.ps1](./evidence/compare-frames.ps1).
> *(Warning hoisted to the top of the section at round 4, finding 8 — it
> previously sat two paragraphs in, after the wrong reasoning.)*

What §T6 and §T10 said was "agree multiple captures on each side, and
treat a residual difference as unresolved". That is a warning, not a
procedure: it tells a task that something is wrong and nothing about what
to do when it meets one. The owner's condition is exactly that gap.

The material to close it came from **classifying the differing pixels
instead of describing them** (the F-33 correction above): in both
same-code pairs every differing pixel is a text pixel, **not one flips
between background and covered**, and the intensity change is bounded at
**13 per channel**. The complementary fact is structural rather than
measured — **any real change moves geometry, and moving geometry swaps a
covered pixel for an uncovered one**, which is a full-contrast difference.
Measured instances: text over a gallery tile is 174 apart, and the F-23
fix produced a max delta of **221**.

So the discriminator is the **maximum per-channel delta**, and it does not
depend on the UI's palette — which matters, because a palette-dependent
rule would not survive the gallery being edited or a second example being
used.

**Superseded by the final review (S-1) — the rule below is unsound and
the script no longer implements it.** Left standing as the dated record of
what T5 decided at this point, with the correction forward in §Final
review disposition; the operative description is that section and the
header of
[evidence/compare-frames.ps1](./evidence/compare-frames.ps1). In
particular the `-Exact` switch named below **no longer exists** — the
default is now strict and `-AllowDrift` is the opt-in — and "any real
change moves geometry" is false, most sharply for T6, whose defining
failure is intensity-only.

Landed in [evidence/compare-frames.ps1](./evidence/compare-frames.ps1)
rather than in prose, so the answer is produced rather than judged:

| Case | Verdict | Exit |
|---|---|---|
| `t5-baseline` vs committed `after/` (same code, a day apart) | 25 / 6 / 25 px, **max delta 1** — within measured drift | 0, printed loudly |
| `t5-baseline-run1` vs `t5-baseline` (same code, session's first launch) | 149 / 75 / 149 px, **max delta 13** — within measured drift | 0, printed loudly |
| `t5-after` vs `t5-f23-after` (the F-23 behaviour change) | 30,800 px, **max delta 221** — **material** | 1 |
| `t5-review-after` vs `t5-f23-after` | identical | 0 |
| any of the above with `-Exact` | the allowance is refused | 1 |

All five verified by running them. Three properties are deliberate:

- **A drift-only result exits zero and says so loudly.** It is not a
  clean pass to be filed silently — the mechanism is still unidentified,
  so the task records the counts and the max delta with its evidence.
- **`-Exact`** exists for a comparison whose whole claim is byte-identity,
  and its message says "exact comparison requested" rather than claiming
  a full-contrast pixel moved. (It did claim that on the first cut, on a
  pair whose max delta was 1 — a message asserting something untrue, which
  is the class this phase keeps catching, here caught before landing.)
- **The threshold is a measurement on this machine, not a constant**, and
  the script says so: a later phase measuring a larger drift raises it
  deliberately rather than widening it to make a gate pass.

**One consequence worth stating for T10 rather than leaving to be
worried about.** Control A's subject *is* glyph rendering, and so is the
drift — but they are distinguishable by the same rule: a crispness change
alters coverage (a soft 2–3 px stem becoming a sharp 1–2 px one moves the
mask), while the drift does not touch the mask at all. **A control-A pair
showing only ≤13 intensity differences has not demonstrated crispness.**
- **R-3 — unconditional correction, or restore DD-003 I1's guard?** The
  case for unconditional: DD-001 §Failure handling's tolerance of a
  failed declaration rests on the conversion machinery having no second
  code path, and a guard is a branch no test can fire until T9, on the
  path every host takes. The case for the guard: it is the Accepted
  text, and the reviewer is right that the difference is observable —
  `WM_WINDOWPOSCHANGING` and `WM_GETMINMAXINFO` are dispatched at `s = 1`
  that otherwise would not be, and one syscall per window is spent
  achieving nothing. **Recommendation: keep it unconditional**, and if
  that stands, record it as a narrowing of DD-003 I1 with DD-001's
  structural argument as the reason — the same shape as the four
  phase-wide judgments already narrowed by what landed. The
  implementation is left unconditional while the decision is open, and
  the merge gate is the decision point.

**What the review confirmed independently**, so it is not re-argued: the
probe frames' tile counts (7 / 7 / 9); F-27's re-reading of T1's number;
that `runtime.rs` carries no residue of the throwaway declaration; the
correction's placement before `apply_mica` and the target creation; the
flag set; the review-lane raise itself; and the suite at 32 binaries and
0 failures.

**A note on the review's own limit, and on mine.** R-1 is the finding
that matters, and it was reachable only by asking what the *documented
rule* says and then testing the implementation against it rather than
against its own tests. Every test T4 wrote used a standard scaling, where
the defect is provably invisible — the same shape as F-13 and F-4, on the
one operation the task existed to get right. The review brief asked
specifically about the `MulDiv` claim and the `f64` widening, which is
some evidence that naming one's own weak claims is worth doing; it also
did not name R-1, which is the finding under them both.

### Owner decisions on the T4 review findings (2026-07-29)

Three questions, all answered; recorded here so no later task re-opens
them.

1. **The review-lane raise (F-25) is approved** after the fact. T4 was
   reviewed under the full independent lane and the merge gate stayed
   blocked until the findings were dispositioned.
2. **R-2 and R-3 are recorded as dated annotations on DD-M4-P1-003, not
   as a successor record and not as a plan-side correction alone.**
3. Same disposition for both, because they are one governance question
   with two instances.

**Why the middle option, stated because the reasoning generalises.**
The three candidates were: (A) leave the correction in
[plan.md](./plan.md) and this log; (B) annotate the ADR in place, body
unchanged, pointing at the measurement; (C) file a successor record.

- **(C) is disproportionate.** A successor record exists to re-choose an
  option. Neither finding re-chooses one: DD-003 still applies the
  OS-suggested rectangle, still fixes the same step ordering, and still
  creates-then-corrects. What is wrong is the accuracy of one property
  statement (R-2) and the conditionality of one clause inside an
  option's description (R-3). A successor with nothing to decide would
  make the record set harder to read, not easier.
- **(A) is where the phase was already heading, and it fails quietly.**
  The correction would live only in implementation documents, while the
  ADR — the decision SSOT — kept the strong claim. T12's Moment 2
  divergence pass covers `architecture.md`, `dsl_spec.md` and
  `abi_spec.md`; **it does not cover ADRs**, so nothing in the process
  would ever revisit DD-003's sentence. And DD-003 is not an ADR that
  goes unread: [plan.md](./plan.md) §T7 makes its enumeration T7's close
  artifact, so the next reader is a task that consumes exactly the rows
  in question. This is F-18's shape — a correction landing in one of the
  two documents that carry the error — pre-empted rather than repeated.
- **(B) has precedent in this repository.**
  [doc-system.md](../../../cross-milestone/decisions/doc-system.md)
  carries an in-place "**Superseded in part (2026-06-19, DD-V-026)**"
  block, and
  [governance-rfc-deferral.md](../../../cross-milestone/decisions/governance-rfc-deferral.md)
  records the same discipline in words: add a line pointing at the
  resolution, **do not rewrite the historical note**. So the mechanism
  was available rather than invented, and the immutability rule is
  honoured — the original prose stands, and the annotation is dated,
  attributed and evidence-linked.

**What landed.** Two annotations in
[dd-m4-p1-003](../decisions/dd-m4-p1-003-dpi-change-propagation.md) —
§Context "Qualified in part" and §Initial scale acquisition "Narrowed" —
plus a row in that DD's revision history and a row in the ADR-set
[preamble](../decisions/preamble.md)'s. **Every `Status:` stays
`Accepted`.** Bodies are unchanged.

**A note for the phase-end batch.** This is the set's first
post-acceptance annotation, and it establishes by use a distinction the
process documents do not currently name: *superseded* (an option is
re-chosen) versus *qualified / narrowed* (the decision stands and a
statement around it is corrected). Whether that distinction deserves a
line in [workflow.md](../../../procedures/workflow.md)'s status
vocabulary — which today lists only `Proposed` / `Accepted` /
`Superseded` — is a process question, not a phase question, and it goes
to the phase-end batch alongside the "show it goes red" vision decision
record rather than being decided here.

### Delta review disposition (Codex, 2026-07-29) — the disposition re-reviewed

Six findings: two major, four minor. **The integer rounding rule itself
drew no correctness finding** — the reviewer re-ran the 1–600 DPI ×
1–4000 DIP sweep independently and reproduced 21,190 mismatches at one
pixel, checked the negative and boundary behaviour, and confirmed the
`f32` conversions are bit-identical. Every finding is about a **claim
made around the code**, which is the same distribution as the first
round.

| # | Finding | Verified | Disposition |
|---|---|---|---|
| 1 | **major** — the I1 annotation changes what option I1 *is*, and adjudicates a DD-001 / DD-003 conflict, so "no decision, option … changes" and "nothing is superseded" are false; a successor record is required | **Confirmed, and the claim was wrong.** The condition sits inside I1's definition, so a reader implementing I1 as written does not get the shipped behaviour — which is the operative test for "decision changed". Worse, the precedent the disposition rested on undercuts it: [doc-system.md](../../../cross-milestone/decisions/doc-system.md)'s "Superseded in part" block **points at successor DD-V-026**; it does not stand in for one. The citation was to the annotation's shape while dropping the thing it exists to reference | Annotation re-headed **"Superseded in part — successor pending"**, the false framing quoted and withdrawn in place, and both revision histories corrected. **The successor is not written here**: the owner's alternative — restore the guard and accept the untested branch — is live again now that the cheap path is closed, and writing a record that may be discarded is the wrong order. Raised below |
| 2 | **major** — the exact-invariance qualification reached DD-003 but not the ADR-set preamble's own body, which asserts the unqualified property in four further places; and plan §T10's control C still demands "logical layout unchanged" absolutely, contradicting control B directly above it | **Confirmed at all five sites.** The preamble's DD-003 summary row, the cross-DD dependency chain, verification item 2 and positive control B each state the property in **different words**, and control C states it as an absolute for the one path where it is least true — a real display-setting change, where the OS-suggested rectangle moves the client extent | All five annotated or corrected in place. See the process note below: this is the third occurrence of the same class, and the reason it recurred is now diagnosable rather than merely embarrassing |
| 3 | **minor** — the `DipScale` reversal did not propagate to [plan.md](./plan.md) §T2's checked item ("retains only the factor") or to the retrospective, which counts two implementation commits and says no existing function was rewritten | **Confirmed.** §T2's item is a record of what landed and now says the opposite of the source; the retrospective's two statements describe the pre-review branch | §T2 gains the reversal with its reason; the retrospective's commit count and §5 corrected. The **T2 entry in this log is deliberately left alone** — it is a dated record of what T2 decided, and finding R-1's disposition above already corrects it forward, which is the distinction between a historical record and a standing claim |
| 4 | **minor** — the failure diagnostic asserts the requested numbers "remain as physical pixels", which is only true of an aware process; DWM stretches the rectangle for the unaware one DD-001 explicitly supports | **Confirmed.** The claim silently assumed the post-T9 world inside the one code path that exists to survive its absence | Diagnostic and doc comment now say the `CreateWindowExW` rectangle **remains uncorrected**, and the doc states both readings rather than picking the one that happens to be false today |
| 5 | **minor** — "no host can observe them" is too broad: a native host can install a `WH_CALLWNDPROC` hook on the thread and see sent messages before the window procedure | **Confirmed.** The null `GWLP_USERDATA` supports "no Wasamo runtime state and no Wasamo-exposed callback observes them" and nothing wider | Narrowed in the DD-003 annotation |
| 6 | **minor** — "an `f32` cannot hold `dpi / 96` exactly unless the DPI is a multiple of 24" is false; `96 = 2^5 × 3`, so the condition is that **3 divides the DPI**. And "both answers are exactly derivable" contradicts the paragraph saying the factor is rounded | **Confirmed, and the falsifier was in my own output.** The R-1 probe printed *333 exact factors in `dpi` 1–1000* — precisely the count of multiples of 3 — in the same run that produced the witness. Every standard Windows scaling happens to be a multiple of 24, so the wrong claim still predicted the standard set correctly and read as confirmed | Both corrected, with the self-falsification recorded rather than quietly fixed. The design argument is restated as **integer results exactly, `f32` factor deterministically** |

**Why the propagation failure recurred three times, stated as a
mechanism rather than as an apology.** T3's S-3 established "correct the
summaries, not only the carriers", and T4 ran that discipline twice —
once in the in-gate re-audit, once after the first review. Both times it
searched for the **phrasing** of the corrected claim: "7 tiles", "outer
versus client", `SWP_NOMOVE`. The sites it kept missing state the same
**proposition** in words that share no phrase with it — "does not
re-decide layout", "layout results are scale-invariant", "the DIP
results are unchanged", "invariance is the evidence". A string search
cannot find those, and running it twice cannot either.

The correction that follows: **search by proposition, not by string.**
Before folding a finding, write the proposition it falsifies as one
sentence, then ask which documents assert *that*, independently of how
they word it. For a claim about a normative property, the candidate set
is every document that summarises the decision — which in this phase is
always the ADR-set preamble, the implementation preamble, and the plan's
task bullets. This is recorded as the T4 remediation for the
re-audit discipline and is the falsifiable test the next task inherits:
**T5 is valid if its propagation pass names the proposition and
enumerates the asserting documents before searching; falsified if a
reviewer again finds an asserting site the search did not visit.**

**Raised to the owner — the R-3 question is reopened, and the option
set has changed.** Yesterday's approval of "annotate rather than
supersede" was given on my recommendation, and **that recommendation was
wrong for the I1 half**. The cheap path is closed; the honest choice is
now between two real costs.

- **Keep the correction unconditional and file a successor record**
  (DD-M4-P1-005, superseding I1's conditional clause only). Cost: one
  short ADR, owner review, and the set's first supersede. Buys: no
  branch that cannot be fired until T9 on the path every host takes.
- **Restore the guard** and let DD-003 stand exactly as written. Cost: an
  authored branch that trap #4 exists to prevent, untestable in either
  direction before T9, on `window::create`. Buys: no ADR work, and the
  record set stays untouched.

**Recommendation: the successor record.** The branch is a permanent
correctness cost paid on every window creation forever; the ADR is a
one-time documentation cost. But the case is closer than I made it
sound yesterday, and the previous recommendation's reasoning does not
survive, so it is put fresh rather than restated.

**The §Context qualification is unaffected** by any of this. It changes
no option, and option B remains the right disposition for it — the delta
review agrees.

### Owner decision on the delta review's finding 1 (2026-07-29)

**Decided: keep the correction unconditional and file a successor
record.** The alternative — restore DD-M4-P1-003 option I1's guard and
leave the ADR set untouched — was rejected on the argument recorded in
the delta disposition above: the branch is a permanent correctness cost
on the runtime's single window-creation path, testable in neither
direction before T9, while the record is a one-time documentation cost.

**Landed:**
[DD-M4-P1-005](../decisions/dd-m4-p1-005-unconditional-size-correction.md),
`Status: Proposed`. The owner has approved the **substance**; the
`Accepted` flip awaits their review of the text, per the standing
discipline that a decision record is not finalised at the moment its
direction is chosen. Consequent edits landed with it: DD-M4-P1-003's I1
annotation now reads "Superseded in part … by DD-M4-P1-005", both
revision histories carry the supersede, and the ADR-set preamble gains a
Decisions row plus a note on the *Effective awareness vs. declared
awareness* coupling — which is where the conflict actually lived and
which the table stated as a consequence rather than as a constraint on
its dependents' wording.

**The boundary this establishes, stated once so it is reusable.** Two
corrections landed on DD-M4-P1-003 on the same day and they took
different routes, which is only defensible if the test between them is
statable:

> **Supersede** when a reader implementing the original text would not
> obtain the shipped behaviour. **Annotate** when the text's decision
> still produces the shipped behaviour and what is wrong is a statement
> around it.

Option I1's conditional fails that test — implement it literally and the
correction does not run at 100%. The §Context layout-invariance
qualification passes it — implement DD-003 exactly as written and the
behaviour is correct; only the claim about what the results *are* was
too strong. The distinction is not in
[workflow.md](../../../procedures/workflow.md)'s status vocabulary,
which lists `Proposed` / `Accepted` / `Superseded` and has no word for
the second case, and that gap goes to the phase-end batch.

**What did not change.** The shipped code is untouched by this record —
`realize_dip_window_size` has had no conditional since it landed. This
is a case of the implementation being right and its justification being
filed in the wrong document, which is worth distinguishing from a record
retrofitted to match code nobody thought about: T4's start gate recorded
the no-branch decision, with DD-001's structural property as its reason,
**before an approach was chosen**. What was missing was the recognition
that a reason of that weight belongs in a decision record rather than in
a gate entry.

### Post-review plan re-audit (2026-07-29, owner-prompted)

**The in-gate re-audit ran before the reviews. After them, only
*propagation* ran — the specific findings were folded into the specific
documents that carried them, and the task list was never re-read as a
whole against what two review rounds had taught.** That is the T1/T2
shape again in a new place: the earlier failure was revising only the
tasks a finding *named*; this one is revising only the documents a
finding *touched*, and skipping the pass that asks what else is now
known to be wrong. Recorded plainly because the owner asked whether it
had been done and the honest answer was no.

This entry is that pass, run **proposition-first** per the delta
review's finding 2 rather than by searching for phrasings. Seven
propositions came out of the two review rounds; each was written as a
sentence, then the documents asserting it were enumerated, then checked.

| Proposition established by the reviews | Asserting documents checked | Verdict |
|---|---|---|
| **P1.** A rule stated against an external contract must be tested against that contract's semantics, not against the inputs the product expects | §T2 (closed), **§T8**, §T10's measurement check, preamble §Verification closure item (1) | **correction — §T8.** Its three factors are 120 / 144 / 192 DPI, every one a multiple of 24, so every one has an exactly-representable `f32` factor. That is the property that hid R-1 behind eleven green tests. T8 synthesises `HIWORD(wParam)` and can drive **100 DPI** for one more case |
| **P2.** `DipScale` retains the DPI and derives the factor | §T2 (corrected), **§T6**, §T5's named operations, DD-002 §The carrier (states no representation — unaffected) | **correction — §T6.** `96 × s` is now exactly `dpi`, so the phase's one sanctioned `factor()` use has an exact, multiplication-free alternative. T6 decides whether to take it; the option did not exist when the bullet was written |
| **P3.** The client rectangle does not scale by `s` | §T8, §T10 controls B and C, §T12, implementation preamble ×2, ADR-set preamble ×4, DD-003 ×3 | no further correction — all propagated across the two disposition commits, and the delta review's finding 2 closed the last four |
| **P4.** An ordering decision placed relative to a message dispatch is unverifiable at `s = 1` | §T4 (closed), **§T7**, preamble §The sequencing thesis (corrected) | **correction — §T7.** Its close artifact is an enumeration, i.e. a description, and it lands before T8 drives `s ≠ 1`; DD-003 calls its step ordering the phase's single most likely defect. T4's probe technique is the fitting answer and was not offered to the task that needs it most |
| **P5.** An audit query assembled from the diff cannot falsify what the diff forgot | **§T5**, §T6 (row 7 only), §T7 | **correction — §T5.** Completeness *is* T5's artifact, and T4's query named only the APIs T4 used. Enumerate the coordinate-carrying API surface first, then search |
| **P6.** Corrections must be propagated by proposition, not by string | §Task list preamble (the gate paragraph) — the only place a rule for *all* remaining tasks can live | **correction — §Task list preamble**, with the falsifiable test T5 inherits |
| **P7.** The set now has a supersede-vs-annotate boundary that the process vocabulary cannot express | §T12's phase-end-owned list | **correction — §T12 phase-end list.** It was recorded only in [log.md](./log.md) and the retrospective, i.e. nowhere the phase-end batch reads its own work items from |

**Tasks re-read with nothing to correct**, stated so the pass is
auditable rather than a list of hits: **§T9** (the `Win32_UI_HiDpi`
sufficiency note and the trap-#4 re-decision both already carry T4's
findings; DD-001's last-error diagnostic channel differs from the
`eprintln!` T4 used, and that is the ADR's explicit choice rather than
an inconsistency to fix — noted here so T9 makes it deliberately);
**§T10** (the three-state table and both control corrections landed at
the earlier passes); **§T11** (owner-executed, unaffected); **§T12**'s
existing items; the implementation preamble's §Obligations carried,
§Verification closure and review-lane table; and the ADR set, whose two
annotations and one supersede are the reviews' own disposition.

### T5's landing site, read before hand-off

The plan names T5's work but not its shape at the source, and T3 set the
precedent of naming the next task's open points before handing over
(`cc99a72`). Read end-to-end: `widget.rs` `sync_visuals` (1763–1836),
`hit_test_click` / `_inner` (1240 ff.), `update_hover` / `_inner`
(1334 ff.), `visual_rect` (1930), `run_layout` / `run_layout_as_window_root`
(1587–1640), the ten constructors' shared field block; `window.rs`
`set_root` and the four coordinate-carrying `wnd_proc` arms;
`emit.rs::flush_layout` (127–149).

Facts worth having before the edit, none of which contradict the plan:

- **The scale cache field lands in one struct and eleven construction
  sites** — the `attached: false` field block that every constructor
  ends with. There is **no `WidgetNode` struct literal outside
  `widget.rs`**: a repo-wide search for one matches only function
  signatures returning `&WidgetNode`, so T1's compiler-measured breakage
  set of 7 test call sites stands and no test constructs the type by
  literal.
- **`sync_visuals` has one recursive call and two entry points**
  (`run_layout`, `run_layout_as_window_root`), both of which call it with
  `(0.0, 0.0)` as the root's parent offset. The outbound conversions are
  therefore confined to one function, as T3 arranged.
- **`visual_rect` has exactly two call sites**, both inside the
  hit-test / hover recursion, and both immediately add the readback to an
  accumulated absolute offset. So the inbound row-9 conversion has two
  sites and no third consumer.
- **`emit::flush_layout` reads `state.hwnd` inside `if let Some(ref mut
  root) = state.root_widget`.** Reading `state.scale` in the same place
  is the same disjoint-field borrow and needs no restructuring — worth
  stating because T1's F-3 borrow note (`update_button_label` must read
  the scale before `button_data_mut()`) is about a *method* borrowing all
  of `self`, and a reader could over-apply it here.
- **The one open point the plan did not name**, now folded into §T5: the
  pointer is divided by `WindowState::scale` at `wnd_proc` while the
  `visual_rect` readback is divided inside a node — two different
  variables compared against each other in the same expression. They
  agree from T6 onward and are both 1 before it, so nothing is broken;
  what is missing is the decision being recorded rather than fallen into,
  which is exactly the shape of T4's own two open points.

---

## T5 — The conversion seams

### Start gate (recorded 2026-07-29, before any edit)

Read before selecting: [plan.md](./plan.md) §T5 and the whole of its
§Task list preamble (including the propagate-by-proposition rule T4 added
and the falsifiable test T5 inherits), [preamble.md](./preamble.md)
(§Implementation gates, the review-lane table, §Technical risks, §The
sequencing thesis, §Verification closure), the ADR set — preamble
§Decisions / §Cross-DD dependencies / §Verification closure; DD-002
§Which space layout works in, §The conversion sites with its rows 4/5/6
and row 6 details, §Which space hit-testing runs in (H2), §The carrier of
the arithmetic; DD-003 §Where the scale is held and its two 2026-07-29
annotations; DD-004 §What `width` / `height` denote and §Does the host
need the scale factor; DD-005 in full — the T1 … T4 entries above with
their four §Plan-hypothesis re-audits, T4's §Post-review plan re-audit and
its §T5's landing site section, and
[implementation-gates.md](../../../procedures/implementation-gates.md).
Source read end-to-end for the change:
[`widget.rs`](../../../../wasamo-runtime/src/widget.rs) `WidgetNode`'s
field block and all ten constructors, `hit_test_click` / `_inner`,
`update_hover` / `_inner`, `clear_hover`, `run_layout`,
`run_layout_as_window_root`, `build_layout_tree`, `sync_visuals`,
`visual_rect`; [`window.rs`](../../../../wasamo-runtime/src/window.rs) in
full; [`emit.rs`](../../../../wasamo-runtime/src/emit.rs)
`mark_layout_dirty_for` / `flush_layout`;
[`lib.rs`](../../../../wasamo-runtime/src/lib.rs) in full;
[`dip_scale.rs`](../../../../wasamo-runtime/src/dip_scale.rs) in full;
[architecture.md §7.5](../../../../docs/architecture.md) and
[§12.3](../../../../docs/architecture.md#coordinate-spaces).

**Trap selection.** [plan.md](./plan.md) §T5 names traps #1 and #2, and T1
armed the full table in §T5 and T6 gate selections above. T5 re-reads that
table at its start, per the gate: **adding is permitted, silently dropping
is not.** Three rows change, and the first of them is a correction to a
recorded judgment rather than an addition.

| # | Trap | T1 armed | T5 | Reason |
|---|---|---|---|---|
| 1 | Semantic-migration miss | **yes** | **yes** | Unchanged, and it is the task. The claim under check is "no coordinate enters or leaves outside DD-002's rows", widened by F-1's second site for row 2, F-2's corrected row-12 site list and F-3's callback slots. The audit query is enumerated **below, before any edit**, per T4 independent review finding R-8. |
| 2 | Missed side effects | **yes** | **yes** | Unchanged. Moving the seam changes the unit seen by everything downstream — including the six callback slots (F-3), which no one installs and which therefore change silently, and including the pointer's static type, which becomes `f32` at two public entry points. |
| 3 | Parallel/derived data drift | no | **yes** — corrected, see **F-32** | T1's reason reads "No parallel vector, index, or **cache** is added" and then names the cache it adds. The node-side `DipScale` is a derived copy of `WindowState::scale` whose source the runtime *does* mutate (T7's handler), and trap #3's discipline — the copy is refreshed inside the primitive that mutates the source — is the obligation this task creates for T7. Filing it only under trap #5 buys a re-trigger sentence where the trap asks for an enumeration of the source's mutators. Close artifact below. |
| 4 | Untested authored branch | no | no | Unchanged, and re-checked against what T5 actually writes: every conversion is unconditional, no reject / diagnostic / size arm is added, and the two `WidgetData` matches T5 touches (`sync_visuals`'s Button-family arm and its ScrollView arm) already exist and gain no new pattern. The property that makes the sequencing thesis work — the machinery has no second code path (DD-001 §Failure handling, DD-005) — is the same property that keeps this trap non-applicable. |
| 5 | Carry-forward underweighted | **yes** | **yes** | Unchanged. The single-writer invariant on the node cache, the "whose scale divides the readback" answer, and the callback-slot unit are each an ordering / identity rule a later task can trip. |
| 6 | Symptom taken at face value | no | **yes**, low expectation | Raised from T1's "no" for the same reason T3 and T4 carried it: this task builds, runs the suite, and **launches a host** (below), so F-5's cold-directory link failure, F-21's stale-uplifted-rlib relink, Observation 5's `scroll_view_layout_integration` access violation and T4's `Copy-Item` mtime false negative are all in reach. Each has a recorded root cause; using a known remedy is not a re-roll, and a *different* recurring failure is a root-cause obligation. |
| 7 | Weak GUI evidence | no | **yes** — see proof obligations 7 and 8 | T1's "no" was right about what it was answering ("do not manufacture GUI evidence T10 owns") and wrong as a blanket. T5 edits the single function that writes every Composition geometry value in the runtime, and T3 measured that the test suite does not react to a geometry write while a rendered frame does. Two artifacts follow, and **the trap's own discipline is what separates them**: a frame captured in the shipped (unaware) state is a **regression check only** — at `s = 1` every conversion is the identity, so a build with no inbound seam at all produces the identical frame — while a frame captured against a throwaway declaration is a **positive control**, because 125% makes the seam's presence a countable difference. |

**Review lane.** **Full independent review**, as
[preamble.md](./preamble.md)'s table has assigned since drafting —
"runtime structural change across every coordinate-carrying path". Not a
raise: T5 is the row the table got right first time, and the two raises
this phase (F-17 at T3, F-25 at T4) were both about tasks *reaching* this
class rather than about this one. Discharged per the owner's T3 decision:
Codex reviews the branch against a written brief, and **the merge gate is
blocked until the findings are dispositioned**. Remediation commits carry
`Reviewed-by: codex <codex@openai.com>`.

### The coordinate-carrying API surface (enumerated before the edit)

T4's audit query named the APIs T4 happened to use, so it could not
exclude the ones it did not, and the reviewer rather than the author ran
the widened search (independent review finding R-8; proposition P5 in the
post-review re-audit). The remedy the plan states is "enumerate the
coordinate-carrying API surface first, then search for all of it". This
section is that enumeration, **written before a single line of T5 is
edited**, so it demonstrably cannot have been assembled from the diff. It
is derived from the question *"what can carry a length, a position or an
extent into or out of this runtime?"* — answered against the `windows`
0.58 surfaces the crate can reach and the runtime's own internal seams —
not from what T5 expects to touch.

| Class | Names searched for |
|---|---|
| Composition geometry **write** | `SetOffset`, `SetSize`, `SetScale`, `SetRotationAngle`, `SetRotationAngleInDegrees`, `SetRotationAxis`, `SetOrientation`, `SetTransformMatrix`, `SetCenterPoint`, `SetAnchorPoint`, `SetRelativeOffsetAdjustment`, `SetRelativeSizeAdjustment`, `SetClip`, `SetBorderMode`, `StartAnimation` on a geometry property |
| Composition geometry **read** | `.Offset()`, `.Size()`, `.Scale()`, `.TransformMatrix()`, `.CenterPoint()`, `.AnchorPoint()`, `.RelativeOffsetAdjustment()`, `.RelativeSizeAdjustment()` |
| Composition clip / surface | `CreateInsetClip`, `CreateRectangleClip`, `CreateGeometricClip`, `CreateDrawingSurface`, `CreateDrawingSurface2`, `CreateSurfaceBrushWithSurface`, `SetStretch`, `SetHorizontalAlignmentRatio`, `SetVerticalAlignmentRatio` |
| Win32 window geometry **write** | `CreateWindowExW`, `SetWindowPos`, `MoveWindow`, `SetWindowPlacement`, `DeferWindowPos`, `BeginDeferWindowPos`, `EndDeferWindowPos`, `AdjustWindowRect`, `AdjustWindowRectEx`, `AdjustWindowRectExForDpi` |
| Win32 window geometry **read** | `GetClientRect`, `GetWindowRect`, `GetWindowPlacement`, `ClientToScreen`, `ScreenToClient`, `MapWindowPoints`, `GetCursorPos`, `SetCursorPos`, `GetSystemMetrics`, `GetSystemMetricsForDpi`, `MonitorFromWindow`, `GetMonitorInfoW` |
| Win32 message payloads carrying coordinates | `WM_SIZE`, `WM_SIZING`, `WM_MOVE`, `WM_MOVING`, `WM_WINDOWPOSCHANGING`, `WM_WINDOWPOSCHANGED`, `WM_NCCALCSIZE`, `WM_GETMINMAXINFO`, `WM_MOUSEMOVE`, `WM_LBUTTONDOWN`, `WM_LBUTTONUP`, `WM_MOUSEWHEEL`, `WM_MOUSEHWHEEL`, `WM_NCHITTEST`, `WM_DPICHANGED`, `WM_GETDPISCALEDSIZE`, and the raw extraction shape `lparam.0 & 0xFFFF` / `>> 16` |
| DPI sources | `GetDpiForWindow`, `GetDpiForSystem`, `GetDpiForMonitor`, `GetScaleFactorForMonitor`, `GetAwarenessFromDpiAwarenessContext` |
| D2D / DirectWrite | `SetDpi`, `SetTransform`, `BeginDraw`, `DrawTextLayout`, `CreateTextLayout`, `SetMaxWidth`, `SetMaxHeight`, `GetMetrics`, `DrawRectangle`, `FillRectangle` |
| Runtime-internal seams | `run_layout`, `run_layout_as_window_root`, `sync_visuals`, `visual_rect`, `hit_test_click`, `update_hover`, `clear_hover`, `measure`, `draw_text`, `applied_offset_y`, `computed.offset`, `computed.size`, `BUTTON_PAD_H`, `BUTTON_PAD_V`, `label_size`, `DipScale`, `factor()` |

Search scope: every `.rs` in the repository (`wasamo-runtime/src`,
`wasamo-runtime/tests`, `wasamo-dll`, `wasamoc/src`, `wasamo-ir`,
`bindings/rust/src`, `examples`), plus the C and Zig hosts for the
window-geometry class. Results at the close gate.

### Planned proof obligations (each closed at the close gate)

1. **The #1 call-site audit table: DD-002's 13 rows**, each with its
   classification, its source location as landed, and the verification
   that closed it — assembled against the enumeration above rather than
   against the diff. **Row 13 is cited from T4, not re-derived**
   ([plan.md](./plan.md) §T5 end gate; F-26), and row 7 is T6's.
2. **The #2 side-effect enumeration**: what changes unit as a consequence
   of moving the seam, including the two public signatures, the six
   callback slots, and the `Result`-discarding call sites.
3. **The #3 artifact**: every mutator of `WindowState::scale` and every
   path that attaches a subtree, each stated as running the walk (from
   T6) or as a stated limit. This is the enumeration trap #5's re-trigger
   sentence does not force.
4. **Decision A — the operation used for the already-parent-relative
   offsets** (audit rows 5 and 6; T3 finding F-19), with the rejected
   candidate and its reason.
5. **Decision B — whose scale divides the `visual_rect` readback** (audit
   row 9; named at T4 after reading the landing site).
6. **Decision C — the unit, and therefore the type, of the six
   `WindowState` callback slots** (T1 finding F-3).
7. **GUI evidence, in two artifacts that are not the same kind of claim.**
   - *Regression, shipped state.* T3's six before/after frame pairs
     re-captured against T5's tree and compared to the committed set.
     **This is not a positive control**: at `s = 1` every conversion is
     the identity, so an implementation that omits the inbound seam
     entirely produces byte-identical frames. What it does catch is the
     class the identity hides nothing about — a transposed axis, a wrong
     variable, a lost write — which is exactly what T3's N1 / N2 / N3
     mutations showed the frames react to and the test suite does not.
   - *Positive control, throwaway-declaration state.* The plan already
     states the number this task must move: T4 measured aware +
     correction at **9 tiles per row**, "the pre-T5 signature", and
     records that "once the inbound seam lands the same state must read
     **7** again" ([plan.md](./plan.md) §T10, F-27). So T5 has a
     *predicted, discriminating* observation available before the work
     starts, using T4's existing probe script and a throwaway declaration
     reverted before the task closes. 9 → 7 separates "inbound seam
     present" from "absent" at 125%, which is the one thing no `s = 1`
     artifact can do.
8. **The mutation, because a predicted number is only evidence once it is
   shown to move.** With the throwaway declaration in place, remove the
   inbound division and re-capture: the tile count must return to 9. That
   is T2's and T3's discipline applied to the artifact this task's
   correctness actually rests on.
9. Regression check only, per the owner-agreed downgrade:
   `cargo build -p wasamo-runtime` → `cargo build --workspace` →
   `cargo test --workspace` green (the F-5 ordering, used as a matter of
   course), plus `cargo fmt --all -- --check` and `git diff --check`.
   Every build feeding a launch is `cargo build --release --workspace`
   (F-21).
10. **The plan re-audit as an in-gate item, and again after the review.**
    [plan.md](./plan.md) §T6 … §T12 and [preamble.md](./preamble.md)
    re-read in order with a verdict table — and, per T4's own recorded
    failure, **the same pass re-run once the review findings are
    dispositioned**, because a review is a source of new facts and not
    only of corrections to apply. Both passes run **proposition-first**:
    name the falsified proposition as one sentence, enumerate the
    documents that assert it — always including the ADR-set preamble's
    Decisions table, its cross-DD couplings and its verification list,
    this plan's task bullets and the implementation preamble — and only
    then search.

### Open decisions, criteria fixed before an answer is looked for

- **Decision A — rows 5 and 6's `SetOffset`.** Criteria: (a) exactly one
  rounding per written quantity, which both candidates satisfy, so it
  cannot decide; (b) the choice must not weaken the enforcement
  `relative_offset_to_physical`'s signature exists to provide — F-15's
  carry-forward names "a new conversion site that reaches for `factor()`"
  as its re-trigger, and this is the site where that reach is most
  tempting; (c) an operation added to `DipScale` is a permanent API
  surface, so it must be argued from what it prevents rather than from
  how the call site reads.
- **Decision B — the readback's divisor.** Criteria: (a) the answer must
  be a property of what the value *is*, not of which variable happens to
  be in scope; (b) it must state what disagrees, and when, rather than
  leaning on "both are 1 today" — which is F-31's lesson in a second
  place; (c) T1's carrier decision fixes the signatures, so a candidate
  that threads a scale through the hit-test recursion re-opens a decision
  rather than making one.
- **Decision C — the callback slots.** Criteria: (a) DD-004 fixes the
  outward unit as DIP, and these are `pub` fields on a `pub use`-exported
  type, so "outward" is what they are; (b) if the unit is DIP, the type
  must carry it without the truncation T1 rejected for the hit-test
  entries (physical 50 at 150% is DIP 33.33), or the decision is nominal;
  (c) any signature change must be audited for installers across the
  workspace and against the normative specs, and recorded either way.

**Commit shape.** Against the one-commit-per-item default, and narrower
than the plan's original exception (F-6). Planned as three commits:

1. **The seams** — the node-side cache, the four inbound sites, the three
   outbound pairs, the two public signature changes and their 7 test call
   sites, in **one** commit. The signature change is the reason: the
   `i32` → `f32` pointer unit does not build in intermediate states
   across `widget.rs`, `window.rs` and four test files, which is the
   reduced scope F-6 left the exception standing at. The conversions ride
   with it because a seam converted while its counterpart is not is a
   state that builds and is *wrong*, which is worse than one that does
   not build.
2. **`emit::flush_layout`'s layout entry** (F-23) — its own commit with
   its own before/after frames, per [plan.md](./plan.md) §T5. It is a
   **behaviour change** and a pre-existing defect, not a conversion, and
   must not ride inside a commit whose whole claim is that nothing
   observable changed. **The owner may move this item to its own task**;
   it is written here as a separate commit precisely so that moving it
   costs nothing.
3. Docs (this log, the plan, the preamble, the handoff) per the
   review-concern rule.

### Decision A — the operation for the already-parent-relative offsets

Taken against the criteria fixed at the start gate. **Decided: the scalar
`to_physical`, once per component, at audit rows 5 and 6. No
already-relative pair operation is added to `DipScale`.**

Criterion (a) does not decide it: a single multiplication of an
already-computed relative quantity is exactly one rounding either way, so
both candidates keep the rule. Criterion (b) does.

| Candidate | Verdict |
|---|---|
| **`to_physical` per component** | **Taken.** It is already the named operation for "one DIP length becomes one physical length", it carries the rounding rule, and it cannot be confused with the difference-taking form because it does not take a pair at all. The cost is that the two components are written out rather than converted as a unit — visible in the diff and worth it for what the alternative costs. |
| A named already-relative pair form (`offset_to_physical(rel)`) | Rejected on (b) and (c). It would put **two offset-converting operations** in the type, one enforcing convert-once-on-the-difference and one not, distinguished only by a name — `relative_offset_to_physical` versus `offset_to_physical`. Row 4 is the site that must take the difference, and the wrong pick there is silent: `offset_to_physical(convert(abs) - convert(parent))` type-checks, reads plausibly, and is the two-rounding form F-15 exists to make unreachable. The type's value here is not that a conversion is available but that **only the right one is**, and adding a second entry point spends exactly that. |
| `extent_to_physical` reused for the offset pair | Rejected on (c). Arithmetically identical, and it would make the operation's own documented property — "the result depends only on the DIP extent, so two widgets of equal DIP size receive bit-identical physical sizes wherever they sit" — a statement about a value that is not an extent. A doc comment that is false about one of its callers is worse than a second call. |

Recorded because this is the site [plan.md](./plan.md) §T5 names as the
one where "F-15's reach for `factor()` is strongest", and the close-gate
search confirms the reach did not happen: **`factor()` has exactly one
production call site in the workspace**, T4's diagnostic string in
`realize_dip_window_size`, which is not a conversion.

### Decision B — whose scale divides the `visual_rect` readback

**Decided: the node's own cache, `self.scale`, at both call sites.**

The criteria asked for a property of the value rather than of what is in
scope, and there is one. **Row 9 exists to undo row 4.** The number
`visual_rect` reads back is the number this node's own `sync_visuals`
wrote, and `sync_visuals` multiplies by `self.scale`; so the divisor that
inverts it is `self.scale` by construction, not by two variables agreeing.
Had the two disagreed, the correct reading of the readback would still be
"divide by whatever multiplied it" — the node's cache — and the pointer
would be the value in the wrong space.

What the alternative would have been, and why it is not merely worse:
dividing by `WindowState::scale` would make the hit-test comparison
*correct as long as the two agree* and silently wrong when they do not,
which inverts where the error surfaces. The node cache makes row 9 an
identity-restoring operation whose correctness does not depend on the walk
having run at all.

**What disagrees, and when** — stated rather than left at "both are 1
today" (F-31's lesson in a second place):

- The pointer is divided at `wnd_proc` by `WindowState::scale`; the
  readback is divided on the node by `self.scale`. From T6 the walk writes
  the second from the first for every node in `state.root_widget`, so they
  agree; before T6 both are the identity.
- A node the walk never reaches keeps `DipScale::default()` while the
  window's scale is not 1. **The only such path that ships is
  `lib.rs::window_add_widget`** (T3 finding F-24) — and on that path the
  disagreement is *unreachable for row 9*, because hit-testing and hover
  traverse `state.root_widget`, which such a subtree never enters. It is
  therefore never hit-tested at all. Recorded as a limit rather than as a
  hazard, and the hazard it *is* — text rasterized at the identity — is
  T6's, already recorded.
- The residual real case is M4-Phase 8's tree moved between differently
  scaled windows, which is the walk's existing carry-forward.

### Decision C — the unit and type of the six callback slots

**Decided: DIP for every slot that carries a coordinate, and the three
that carried `i32` change to `f32`.**

**The counts, stated exactly** (T5 final review finding S-7 — the first
version of this section said "four pointer slots change from `i32`",
which conflates two different fours): there are **six** callback fields;
**four** carry coordinates — `resize_fn` plus `mouse_move_fn` /
`mouse_down_fn` / `mouse_up_fn`; **three** change type, because
`resize_fn` was already `(f32, f32)` and changes only its *unit*; and
`key_down_fn` / `mouse_leave_fn` carry no coordinate and are untouched in
both respects.

Criterion (a) settles the unit without argument: DD-M4-P1-004 fixes DIP as
the unit of every outward-facing length, and these are `pub` fields on a
`pub use`-exported type, so they are outward-facing whether or not anyone
has installed one. `resize_fn` needs no signature change — it is already
`(f32, f32)` — and it changes unit, from the raw `WM_SIZE` client extent
to the DIP extent layout receives.

Criterion (b) is what forces the type change. Keeping `i32` would have
delivered a truncated DIP position — physical 50 at 150% is 33.33 — which
is the defect T1 rejected when it chose `f32` for the hit-test entry
points, arriving through a different door. A unit declared in a comment
and destroyed by the type is not a decision, it is a note.

Criterion (c), audited rather than assumed:

- **Installers:** none. A search for `resize_fn|key_down_fn|mouse_down_fn|mouse_move_fn|mouse_leave_fn|mouse_up_fn`
  over the whole repository finds the six declarations, the six `None`
  initialisers and the six `wnd_proc` invocations, and nothing else — no
  ABI function, no Rust-native function, no example, no binding, no test.
  T1's F-3 said the same and it is re-confirmed against the source here.
  This is DD-M4-P1-004's "no host needs the scale factor" holding from a
  second direction: no host is even receiving a coordinate today.
- **Specs:** [architecture.md §7.5](../../../../docs/architecture.md)'s
  table names the slots and spells out the signature for `resize_fn` and
  `key_down_fn` only — both unchanged — so no written type is falsified.
  §12.3 already states the pointer seam normatively and is satisfied by
  this landing. **No spec edit is required, and this is recorded rather
  than passed over**, because "no edit needed" and "did not look" are
  different facts.

### Close gate

**#1 — call-site audit.** The claim under check: *no coordinate enters or
leaves the runtime outside DD-M4-P1-002's rows.* The query is the
**pre-registered enumeration above**, not a list assembled from the diff
(T4 independent review finding R-8), and the two results that matter are
the ones the diff could not have suggested:

- **Absent from every `.rs` in the repository:** `MoveWindow`,
  `AdjustWindowRect` / `AdjustWindowRectEx` / `AdjustWindowRectExForDpi`,
  `SetWindowPlacement`, `GetWindowPlacement`, `DeferWindowPos`,
  `BeginDeferWindowPos` / `EndDeferWindowPos`, `GetWindowRect`,
  `ClientToScreen`, `ScreenToClient`, `MapWindowPoints`, `GetCursorPos`,
  `SetCursorPos`, `GetSystemMetrics` / `GetSystemMetricsForDpi`,
  `MonitorFromWindow`, `GetMonitorInfoW`, `GetDpiForSystem`,
  `GetDpiForMonitor`, `GetScaleFactorForMonitor`, `SetScale`,
  `SetRotationAngle*`, `SetOrientation`, `SetTransformMatrix`,
  `SetCenterPoint`, `SetAnchorPoint`, `SetRelativeOffsetAdjustment`,
  `SetBorderMode`, `CreateRectangleClip`, `CreateGeometricClip`,
  `CreateDrawingSurface2`, `SetStretch`, `SetTransform`, `SetDpi`.
  (`GetSystemMetricsForDpi` and `SetProcessDpiAwarenessContext` appear
  only in T4's and T5's reverted throwaways and in this log.)
- **`StartAnimation` exists but carries no geometry:** its single
  production site animates `"Color"`.
- **`wnd_proc` handles eight messages** — `WM_DESTROY`, `WM_ERASEBKGND`,
  `WM_SIZE`, `WM_KEYDOWN`, `WM_MOUSEMOVE`, `WM_MOUSELEAVE`,
  `WM_LBUTTONDOWN`, `WM_LBUTTONUP` — and none of the other
  coordinate-carrying messages in the enumeration (`WM_SIZING`, `WM_MOVE`,
  `WM_MOVING`, `WM_WINDOWPOSCHANGING/ED`, `WM_NCCALCSIZE`,
  `WM_GETMINMAXINFO`, `WM_MOUSEWHEEL`, `WM_MOUSEHWHEEL`, `WM_NCHITTEST`,
  `WM_DPICHANGED`, `WM_GETDPISCALEDSIZE`) is reached at all.

| Row | Site | Direction | Classification | As landed | What closed it |
|---|---|---|---|---|---|
| 1 | `wnd_proc` `WM_SIZE` client extent | in | must convert | `window.rs` — `scale.pair_to_dip((lo, hi))`, feeding both `resize_fn` and `run_layout_as_window_root` | source + the throwaway-declaration capture: removing this division and row 2's puts the gallery back to 9 tiles per row |
| 2 | `set_root` `GetClientRect` → first layout | in | must convert | `window.rs::set_root` — `state.scale.pair_to_dip(...)` | the same capture pair; this is the row the probe's first layout actually exercises, since the probe never resizes |
| 2b | `emit::flush_layout` `GetClientRect` → drain layout | in | must convert — **the site DD-M4-P1-002 does not name** (T1 finding F-1) | `emit.rs` — `state.scale.pair_to_dip(...)`, a disjoint-field read beside the `root_widget` borrow | source; exercised by every click in the frame set |
| 3 | `wnd_proc` `WM_MOUSEMOVE` / `WM_LBUTTONDOWN` / `WM_LBUTTONUP` | in | must convert (H2) | `window.rs` — `scale.pair_to_dip(pointer_physical(lparam))` at all three arms; the signed `lParam` extraction is hoisted to one helper rather than written a third time | source; the three arms are the only `lparam` coordinate readers in the file |
| 4 | `sync_visuals` node `SetOffset` / `SetSize` | out | must convert, **and this is the one that takes a difference** | `widget.rs` — `self.scale.relative_offset_to_physical(computed.offset, parent_abs_offset)` and `extent_to_physical(computed.size)` | source + the node-cache-seeded capture: with the cache throwaway-seeded to 120 DPI the tree fills the 982 x 703 client instead of 785.6 x 562.4 of it |
| 5 | `sync_visuals` ScrollView intermediate | out | must convert, **already parent-relative** (F-19) | `widget.rs` — `to_physical(0.0)` / `to_physical(-applied)` and `extent_to_physical(computed.size)`; `child_parent_abs` stays DIP | source; decision A above |
| 6 | Button / ToggleButton label | out | must convert, **already parent-relative** (F-19) | `widget.rs` — `to_physical(BUTTON_PAD_H)` / `to_physical(BUTTON_PAD_V)` and `extent_to_physical(btn.label_size)`, inside the arm T3 relocated | source; decision A above |
| 7 | `draw_text` surface + D2D DPI + atlas origin | out | **T6's** | unchanged | not this task's; `CreateDrawingSurface` has exactly one call site and `surface_pixels` carries the `allow(dead_code)` naming T6 |
| 8 | root `SetRelativeSizeAdjustment(1, 1)` | out | **unchanged — asserted** | `window.rs:106`, untouched | verified as the only `SetRelativeSizeAdjustment` in the workspace; a ratio between two physical quantities has no scale to apply |
| 9 | `visual_rect` readback → hit-test / hover | in | must convert (H2) | **Corrected after the independent review (R-2), and again at the final review (S-3), which found this row still describing the first landing.** `widget.rs` — the conversion is in **`WidgetNode::visual_rect_dip`**, the single caller of `visual_rect`, and it divides by the **traversal root's** scale rather than by each node's. `visual_rect_dip` has two call sites, `hit_test_click_inner` and `update_hover_inner` | source: `visual_rect` now has exactly **one** caller and `visual_rect_dip` exactly two, both inside the hit-test / hover recursion. Decision B, **as rewritten below**, records the divisor and its precondition |
| 10 | `TextRenderer::measure` → layout | — | **unchanged — asserted** | untouched | `measure` returns DirectWrite metrics computed at 96 DPI, i.e. already DIP; its five call sites all feed `SizeConstraint` / `draw_text` and none is scaled here. This is the fact that carries "layout stays DIP" |
| 11 | `TypographyStyle::size_sp` | — | **unchanged — asserted** | untouched | one call site, `create_text_layout`'s `CreateTextFormat`; DD-M4-P1-004 defines it as DIP and T6 keeps it DIP by setting the context DPI instead |
| 12 | `InsetClip` insets | out | **unchanged — asserted** | untouched | `CreateInsetClip` has **three** call sites — `scroll_view`, `grid`, `zstack`, and **not `box_`**, which installs none (T1 finding F-2 corrects the ADR's site list; the conclusion is unaffected). All insets are zero and zero is scale-invariant |
| 13 | `create_hwnd` `CreateWindowExW` width / height | in | **closed at T4** | `window::realize_dip_window_size` | cited from §T4's call-site audit, not re-derived ([plan.md](./plan.md) §T5 end gate; F-26). Re-confirmed only to the extent the widened query above found no second window-sizing API |

**#2 — structural side-effect enumeration.** What moving the seam drags
along. Rows marked *unchanged* are assertions.

| # | Effect | Verdict |
|---|---|---|
| 1 | `hit_test_click` / `update_hover` **public signatures** | **changed:** `i32` physical → `f32` DIP. 7 call sites in 4 test files, exactly T1's compiler-measured set; no production caller outside `wnd_proc`. No `WidgetNode` struct literal exists outside `widget.rs`, so the new field breaks no construction site |
| 2 | The `WindowState` callback slots | **changed in unit** for the **four** that carry a coordinate (`resize_fn` plus the three pointer slots) **and in type** for the **three** that carried `i32`; `resize_fn` was already `f32` and changes unit only, and `key_down_fn` / `mouse_leave_fn` carry none. Zero installers workspace-wide. Counts corrected at the final review (S-7). Decision C |
| 3 | The layout entry points `run_layout` / `run_layout_as_window_root` | **unchanged.** Their `f32` arguments change *meaning* from physical to DIP, which is invisible to the 21 test call sites T1 counted because those drive `WidgetNode`s directly and never through a window — the same property F-4 recorded, here working in the task's favour |
| 4 | `sync_visuals`' signature and recursion shape | **unchanged.** One recursive call, two entry points, both passing `(0.0, 0.0)`. `child_parent_abs` stays DIP, so the recursion carries no physical value |
| 5 | The reactive drain | **unchanged** by the conversion commit: no property is written, nothing is enqueued, `MUTATION_CAP` and drain accounting are untouched. (The *second* commit changes which layout entry the drain calls — a behaviour change, isolated there on purpose) |
| 6 | Hover and press state | **unchanged.** `update_hover_inner` compares DIP to DIP where it compared physical to physical; the state machine, the colour targets and the animation durations are untouched |
| 7 | `clear_hover` | **unchanged** and deliberately not converted — it takes no coordinate |
| 8 | `visual_rect` itself | **unchanged.** It stays a free function returning the physical readback; the division happens at its two callers, which is what keeps it the single readback point the audit names |
| 9 | Layout results at `s = 1` | **unchanged, bit-exactly.** `to_physical` and `to_dip` at the identity are multiplication and division by exactly `1.0f32`, so every value round-trips to itself. Measured, not argued: the six T3 frames are byte-identical to the committed set |
| 10 | `WindowState::scale`'s `#[allow(dead_code)]` | **removed.** T5 is its first reader, and leaving it would silence a real warning for T6 / T7 |
| 11 | `dip_scale`'s module-level `#![allow(dead_code)]` | **removed and narrowed** to `surface_pixels` and its private helper, which T6 is the first caller of. Same reason as row 10, one scope out |
| 12 | Visual parenting, Z-order, clip installs, brush creation | **unchanged** — no constructor is touched except by the added field |
| 13 | `emit::mark_layout_dirty_for`'s O(windows x nodes) pointer search | **unchanged.** It compares pointers, not coordinates |
| 14 | A layout pass that fails | **unchanged from T3's row 14.** `run_layout*` still propagates the error with `?` before `sync_visuals`, and both callers still discard the `Result`. The conversion adds no fallible step: `pair_to_dip` cannot fail |

**#3 — parallel/derived data (the trap T1 marked non-applicable; see
F-32).** The node-side cache is a derived copy of `WindowState::scale`.
The trap asks that the copy be refreshed inside the primitive that mutates
its source, so the artifact is the enumeration of that source's mutators
and of the paths that attach a subtree:

| Site | Mutates the source? | Refreshes the copy? |
|---|---|---|
| `window::create` — the `WindowState` literal | **seeds** it, once, from `GetDpiForWindow` | not applicable: no widget tree exists yet. `root_widget` is `None` for the whole of `create` |
| T7's `WM_DPICHANGED` handler | **will mutate** it — the only mutation the runtime performs | **obligation created here:** it must run T6's walk, which is already step 4 of DD-M4-P1-003's fixed order. Recorded as carry-forward below so it is an invariant rather than a step that happens to be listed |
| `window::set_root` | no | **obligation created here:** it attaches a subtree built at the identity, so it must run T6's walk after the first layout — which is the walk's other stated caller |
| `lib.rs::window_add_widget` | no | **no, and it cannot** — the subtree never enters `root_widget`, so neither walk caller reaches it (F-24). Stated limit, unchanged from T3's; for row 9 specifically the consequence is unreachable, because hit-testing traverses `root_widget` too |
| `WidgetNode::append_child` / `insert_child` / `replace_child` and their ABI wrappers | no | **no.** A child attached to an already-attached tree keeps its constructor identity until a walk runs over it. Today no walk exists, so nothing is wrong; from T6 this is a live re-trigger and is recorded as such |
| The IR loader's conditional / `for` mutation sites | no | same as the row above — they build subtrees and call `mark_layout_dirty_for`, which schedules layout but not a scale walk |

The last two rows are the substance the trap buys over trap #5's
re-trigger sentence: **`set_root` and `WM_DPICHANGED` are not the only
paths that put a fresh node under a scaled window**, and T6 must decide
whether its walk covers the incremental ones or whether that is a stated
limit. Carried below.

**#4 — untested authored branch.** Non-applicable, and re-checked against
what landed rather than inherited: `git diff` adds no `if`, no `match`
arm, and no `?`-bearing fallible step. Every conversion is unconditional,
which is the property DD-M4-P1-001 §Failure handling and DD-M4-P1-005 both
rest on.

**#5 — carry-forward.** Three invariants, each with a re-trigger
criterion; recorded in [handoff.md](./handoff.md).

1. **Row 9 divides by the traversal root's scale, and that is correct
   only for a traversal rooted at the window's root.** *(Rewritten twice:
   the first landing divided by each node's own cache, corrected at the
   independent review as R-2; the correction's own claim of being
   "unconditional" was then corrected at the final review as S-2.)*
   *Re-trigger:* any caller that enters `hit_test_click` or `update_hover`
   on something other than the tree the window laid out — the public
   entries permit it and `togglebutton_runtime_integration.rs` does it —
   and M4-Phase 2's option H3, which deletes row 9 entirely by caching hit
   rectangles from layout.
2. **The two conversions on the hit-test path cancel today**, because
   hit-testing sources its geometry from the visual tree (DD-M4-P1-002
   §Which space hit-testing runs in, stated honestly at ADR time).
   *Re-trigger:* the moment M4-Phase 2 sources geometry from layout or
   introduces a DIP-denominated hit-area rule, they stop cancelling and
   row 9 becomes load-bearing rather than symmetric.
3. **Every path that attaches a node under an already-attached tree needs
   the walk**, not only `set_root` and the `WM_DPICHANGED` handler — see
   the #3 table. *Re-trigger:* T6 deciding the walk's reach; M4-Phase 2's
   event-model tree edits; M4-Phase 8 moving a tree between windows.

**#6 — deterministic-failure disposition.** One symptom was root-caused
rather than absorbed, and it is recorded as **F-33** below because the
conclusion changes how T6 and T10 must capture. Nothing else recurred:
Observation 5's `scroll_view_layout_integration` access violation did not
appear in any run, and F-5's ordering was used as a matter of course.

**#7 — GUI evidence.** Two artifacts, of two different kinds, per the
start gate. Every build feeding a launch was
`cargo build --release --workspace` (F-21).

*Regression, shipped state.* T3's six frames re-captured on the T5 tree,
compared over the client interior with
[evidence/compare-frames.ps1](./evidence/compare-frames.ps1):

| Comparison | Result |
|---|---|
| `t5-after` vs `t5-baseline` (this session, pre-change) | 25 / 6 / 25 differing pixels on the three gallery frames, 0 on the three label-update frames — see F-33 |
| **`t5-after` vs the committed T3 [`after/`](./evidence/after/) set** | **0 of 827,904 and 0 of 224,224 — all six frames, byte-identical across two days and two builds** |
| `t5-after` re-captured on the tree with every throwaway reverted | 0, all six — so the reverted state is the committed state, verified by frame and not by `git status` alone |

The second row is the regression claim: **T5's build renders identically
to a frame set captured before T5 existed.** It is deliberately *not*
offered as a positive control — at `s = 1` every conversion is the
identity, so a build with no inbound seam at all would produce these same
frames. What it excludes is the class the identity hides nothing about: a
transposed axis, a wrong variable, a lost write. T3's N1 / N2 / N3
mutations are the standing evidence that this frame set reacts to exactly
that class while the test suite does not.

*Positive control, throwaway-declaration state.* The plan predicted the
number this task must move, so the observation was fixed before the work:
T4 measured aware + correction at **9 tiles per row**, "the pre-T5
signature", and [plan.md](./plan.md) §T10 records that it "must read **7**
again" once the inbound seam lands. Captured with T4's own probe script at
125%, frames in [evidence/t5-probe/](./evidence/t5-probe/):

| State | Source | Window / client | Tiles per row | Rendered content |
|---|---|---|---|---|
| **P1 — as landed** | throwaway V2 declaration only | 1000 x 750 / 982 x 703 | **7** | occupies 785.6 x 562.4 of the 982 x 703 client — 1/1.25 of it |
| **P2 — node cache throwaway-seeded to 120 DPI** | + `DipScale::from_dpi(120)` in the ten constructors | 1000 x 750 / 982 x 703 | **7** | **fills the client**, geometry 1.25x, text still soft |
| **P3 — mutation: the inbound division removed** | + rows 1, 2 and 2b unconverted | 1000 x 750 / 982 x 703 | **9** | as before the task |

Read as a set: **P1 → P3 is the mutation** and it moves the predicted
number in the predicted direction, so the 7 is produced by the inbound
seam and by nothing else. **P1 → P2 is the outbound control**: the same
correct 7-tile logical layout, drawn at 80% of the client when the node
cache is the identity and at 100% when it is not — which is the outbound
multiplication being exercised for the first time in the phase, and the
concrete demonstration that T6's walk is what is missing rather than a
conversion. P1 is the shipped state, and the 80% is **correct for T5
alone**: T6 owns the walk that writes the cache.

What this set does **not** show: crispness. P2's glyphs are visibly soft —
the DIP-sized rasterization surface stretched to a 1.25x Visual, which is
R-1's premise rendered rather than argued. That is T6's work and T10's
control A, and it is stated here so a reader of P2 does not mistake a
correct T5 for an incomplete T6.

*F-23's frames* are the second commit's, and a different claim again: a
**deliberate** behaviour change, 30,800 of 224,224 pixels on the two
post-click label-update frames and 0 on the four that do not reach the
drain with a `Shrink`-rooted tree.

**End-gate items from [plan.md](./plan.md) §T5.**

- *The 13-row call-site audit table* — above, assembled against the
  pre-registered enumeration.
- *Workspace green as a regression check only* —
  `cargo build -p wasamo-runtime` → `cargo build --workspace` →
  `cargo test --workspace` (the F-5 ordering, used as a matter of course):
  **32 test binaries, 0 failures**, runtime lib **462**, unchanged — T5
  adds no test, and the 7 edited call sites are literal changes inside
  existing ones. Per the owner-agreed downgrade this is a regression check
  and nothing more; the artifacts above are the evidence.
- `cargo fmt --all -- --check` and `git diff --check` clean.
- *Throwaways reverted* — `git status` clean of `wasamo-*` changes beyond
  the two commits, a repository-wide search for the probe markers finds
  nothing, and the reverted tree was **re-captured and compared** rather
  than trusted (row 3 of the regression table). The V2 declaration remains
  T9's to land.

### F-32 — the node-side scale cache is a parallel copy, and T1's table said it was not

T1's §T5 gate selection marks trap #3 non-applicable with the reason "No
parallel vector, index, or **cache** is added. The node-side scale cache
T5 introduces is written by nobody until T6 and has one writer thereafter;
it is covered as trap #5, not as parallel data." The sentence denies and
then names the same thing, and the two halves of the reason do different
work: "written by nobody until T6" is true and irrelevant — the trap is
about the *source's* mutators, not the copy's — and "covered as trap #5"
substitutes a re-trigger sentence for an enumeration.

The difference is not bookkeeping. Trap #5's artifact would have been
"any path that attaches a subtree without running the walk must call it",
which is what [handoff.md](./handoff.md) already carried from T1. Trap
#3's artifact is the table in the close gate above, and running it
surfaced two path classes neither the handoff nor [plan.md](./plan.md)
§T6 names: **`append_child` / `insert_child` / `replace_child` on an
already-attached tree, and the IR loader's conditional and `for` mutation
sites.** Both put a fresh node, holding the constructor identity, under a
window whose scale is not 1, and both are ordinary shipped paths rather
than future hazards. T6 must decide whether its walk covers them or
whether that is a stated limit.

This is the **fifth** phase-wide or table-level judgment narrowed by what
actually landed — trap #4 at T2 (F-12), the review lane at T3 (F-17),
trap #3 phase-wide at T3 (F-22), the review lane at T4 (F-25), and a
task-level trap selection here. *Disposition:* [plan.md](./plan.md) §T6
gains the walk-reach decision; [handoff.md](./handoff.md)'s scale-cache
re-trigger row gains the two path classes.

### F-33 — a committed frame set is not a later task's baseline, and one capture is not a baseline either

Found while establishing T5's regression control, and recorded because it
changes how **T6 and T10** must capture rather than because it affected
T5. [plan.md](./plan.md) §T10 control A already says "if a pre-change
frame is reused rather than re-captured, check the commit it was captured
at against the current surface first"; this is that instruction with a
measurement behind it, and the measurement says something the instruction
does not.

Measured, on the unmodified pre-T5 tree, over the client interior:

| Comparison | gallery frames | label-update frames |
|---|---|---|
| Three captures **within one process**, 2 s apart | 0 | — |
| Two captures, **different launches, same session** (runs 2 and 3) | **0** | 0 |
| **The session's first launch** against either of the other two | **149 / 75 / 149** and 124 / 69 / 124 | 0 |
| A settled capture against the **committed T3 set** (one day earlier) | 25 / 6 / 25 | 0 |

So the **first capture of a session was an outlier** by up to 149 of
827,904 pixels, at up to 13 per channel, while two later captures in the
same session agreed exactly.

**Where the differing pixels are, measured rather than described
(2026-07-29, owner-prompted).** The first version of this paragraph said
the differences were "confined to the two rows of tile-label glyphs" and
called them antialiasing. Both were **inferences from one comparison**,
and the first is **false**. Classified against the tile fill
(`#4f6272`) and the label colour, over the two same-code pairs:

| | same code, different day (25 px) | same code, session's first launch (149 px) |
|---|---|---|
| max channel delta | **1** | 13 |
| partial-coverage (antialiasing) pixels | 13 | 129 |
| fully-covered glyph-body pixels | 0 | 8 |
| tile-fill pixels | **0** | **0** |
| **coverage flips** (background ↔ covered) | **0** | **0** |
| direction | mixed (7 / 18) | **142 of 149 one-sided** |

Three things follow, and one correction:

- **Every differing pixel is a text pixel.** No tile fill, no button
  background, no backdrop pixel moves. The twelve pixels that fell
  outside the tile colour range are a single vertical stroke at
  `x = 830, y = 68…79`, `(90,90,90)` against `(91,91,91)` — a stem in the
  **toolbar button's** label.
- **The glyph geometry did not move.** Not one pixel flipped between
  background and covered, in either pair. A subpixel positional shift
  across 149 pixels would show flips at the edges. So the coverage mask
  is identical and only the *intensity* of already-covered pixels
  changed — one-sided in the larger pair, which reads as a level shift
  rather than jitter.
- **That independently kills the atlas hypothesis** a second time: an
  atlas-offset change moves the mask, and the mask is unchanged. What
  remains untested is how coverage becomes colour (blend, gamma,
  antialias-mode selection) or the capture itself. **Not claimed.**
- **The correction.** "Confined to tile-label glyphs" is wrong — a button
  label is in the set. That is not merely imprecise: the atlas hypothesis
  was made plausible partly by the reading "button labels are stable and
  tile labels are not, which fits later allocations landing wherever
  there is room". **That premise was never true**; it came from
  generalising the first comparison. The hypothesis was disproved on its
  own terms by the offset probe, so no conclusion changes, but one of the
  reasons for entertaining it was false from the start.

So the honest statement of what F-33 observes: **antialiased pixels
differ, and antialiasing is not established as the cause.**

**What this does and does not license — corrected at the independent
review (finding R-3), because the first version of this paragraph
contradicted the table above it.** It claimed that a baseline built from
two agreeing same-session captures yields exact equality "and T5 did".
That conflates two different comparisons: the settled baseline
`t5-baseline` and the post-change `t5-after` differ by **25 / 6 / 25**,
and the byte-identical result was `t5-after` against the **committed T3
set** of the previous day. Agreeing on one side does not make the
across-the-change comparison exact.

What the measurement supports, and nothing more:

- A session's first launch can be an outlier, so **one capture is not a
  baseline**.
- A committed frame set is not a later task's baseline either — the same
  commit reads 25 pixels apart across sessions.
- Therefore **neither a stale frame alone nor a fresh frame alone
  settles a comparison**, and a pre/post difference of this magnitude is
  **unresolved** rather than a pass or a fail.

The procedure that follows: agree **multiple captures on each side**, and
treat a residual pre/post difference as an open question — root-cause it
or state it — instead of reading it as a regression or waving it through
as noise. T5's own regression claim does not rest on this: it rests on the
byte-identical match against the committed T3 set (a comparison that *was*
exact) plus the source audit, and the mechanism behind the 25 pixels is
**not identified**.

**One hypothesis was tested and disproved rather than asserted** (T3's
derived discipline: measure the mechanism, do not infer it). The obvious
candidate was atlas packing — `draw_text` uses `BeginDraw`'s atlas offset
directly as the drawing origin, so a different packing could shift glyph
rasterization. `draw_text` was instrumented to print the offset and
surface size for every call, and **two launches produced byte-identical
output**: the packing is deterministic. The remaining candidate — the Mica
backdrop compositing whatever sits behind the window, which the capture
cannot control — is **not claimed**, because it was not measured.

**A fact for T6 came out of the disproved hypothesis, and it sharpens
risk R-3 in the direction that matters.** DD-M4-P1-002 §The rasterization
surface step 3 and [preamble.md](./preamble.md) R-3 both say the atlas
offset "is frequently `(0, 0)`, so an implementation that forgets the
conversion works most of the time". Measured on the gallery: the offsets
are `(1,2)`, `(19,2)`, `(68,2)`, `(125,2)`, `(199,2)`, `(255,2)`,
`(345,2)`, `(348,2)` … — they march across the atlas and **essentially
none is `(0, 0)`**. The trap is real and the qualification is generous:
on a UI with more than a couple of text nodes, omitting the origin
division is wrong almost everywhere rather than intermittently. *Disposition:*
[plan.md](./plan.md) §T6's atlas-origin bullet and
[preamble.md](./preamble.md) R-3.

### Plan-hypothesis re-audit (2026-07-29, in-gate — not owner-prompted)

Third run of the in-gate shape (T3 and T4 were the first two), and the
first run under the **proposition-first** rule T4's delta review produced
and [plan.md](./plan.md) §Task list preamble now carries. The rule is the
whole of the method here: T4's two passes searched for the *phrasing* of
each correction and missed the documents stating the same proposition in
other words, three times running. So each proposition below was written as
one sentence, the documents asserting it were enumerated **before**
searching, and the search was run against that list.

**The rule earned its place immediately.** Proposition Q5's third
asserting site is [architecture.md §12.4](../../../../docs/architecture.md#coordinate-spaces)
— a normative spec — and Q6's is the **ADR set's own verification-closure
item 3** plus [constraints §9](../requirements/constraints.md). A search
for "atlas" or for the wording of F-33 would have found the first; nothing
in F-33's phrasing appears in the second, which says "the commit it was
captured at is checked against the current surface first" and never uses
the word baseline.

| Proposition established by T5 | Asserting documents enumerated, then checked | Verdict |
|---|---|---|
| **Q1.** The `visual_rect` readback is divided by the **node's** cache, because it undoes the multiplication the node's own `sync_visuals` performed | §T5's open point; DD-002 row 9 (says only "÷ s" — under-specified, not falsified, and immutable) | **§T5 only.** Decision recorded above |
| **Q2.** Audit rows 5 and 6 use the scalar `to_physical` per component; no already-relative pair operation is added | §T5's open point; §T6 (the one sanctioned `factor()` use — unaffected); **T2's landed-surface table in this log**, which lists `relative_offset_to_physical` as serving "4, 5, 6" | **§T5**, plus a note: T2's table is now wrong for rows 5 and 6. Left standing as a **dated record of what T2 landed**, corrected forward by this entry — the same distinction T4's delta review drew for its own T2 entry (finding 3) |
| **Q3.** Every callback slot that carries a coordinate is DIP, and the three that carried `i32` become `f32` | §T5's open point; T1's F-3 (which assigned the decision here); [architecture.md §7.5](../../../../docs/architecture.md) (spells out only `resize_fn` and `key_down_fn`, both unchanged — **not** falsified); §12.3 (satisfied) | **§T5 only.** No spec edit needed, recorded rather than passed over |
| **Q4.** A derived copy's trap-#3 obligation is about the **source's** mutators, and the walk's two callers are not the only paths that put a fresh node under a scaled window | T1's §T5 gate table (**F-32**); §T6's walk bullet; [handoff.md](./handoff.md)'s scale-cache row; [preamble.md](./preamble.md) §Implementation gates trap #3 | **corrections in all four.** §T6 gains the walk-reach decision; the handoff row gains the two path classes; the preamble's trap-#3 narrowing gains T5 as a second site |
| **Q5.** The atlas origin is essentially never `(0, 0)` on a UI with more than a couple of text nodes | [preamble.md](./preamble.md) R-3; §T6's atlas bullet; **[architecture.md §12.4](../../../../docs/architecture.md#coordinate-spaces)**; DD-002 §The rasterization surface step 3 and §Technical risk re-evaluation | **corrections in the first two.** architecture.md is a normative spec and the correction is a **Moment 2 divergence for T12**, not an implementation task's edit — the same disposition F-28 took for [framing.md](../requirements/framing.md) at T4. **The ADR is not corrected and does not need to be**: "often `(0, 0)`" is a claim about the general case, a single-text-node UI really does get it, and the ADR's conclusion — write the division deliberately — is what the measurement strengthens |
| **Q6.** Checking the commit a reused frame was captured at is **not sufficient** to make it a baseline; the same commit produced frames 25 pixels apart across sessions and 149 apart on a session's first launch | §T10 control A; §T6's end gate ("local rendering unchanged at 100%"); §T3's end gate (historical, closed); **the ADR set's §Phase 1 verification closure item 3**; **[constraints §9](../requirements/constraints.md)**; [verification-environments.md](../../../../docs/notes/verification-environments.md) Observation 4 (T12 revises it anyway) | **corrections in §T6, §T10 and §T12.** The ADR statement and constraints §9 are **raised to the owner below** rather than edited: an implementation task does not choose an ADR's correction route (the T4 lesson), and constraints is an upstream agreement record |
| **Q7.** T5 does not close all thirteen audit rows itself — row 13 is T4's and row 7 is T6's | §T5's end gate (**already corrected at T4 by F-26**); **[preamble.md](./preamble.md) §Obligations carried, obligation 4**, which still reads "T5 closes against those 13 rows" | **correction — the implementation preamble.** F-26's fix reached the document that carries the task and not the one that summarises the obligation. That is S-3's shape a fourth time, caught here by the enumeration rather than by a reviewer |

Task-by-task verdicts, every entry, including the ones with nothing to
correct:

| Re-read | Verdict |
|---|---|
| §Task list preamble (gate-substitution table, commit rules, the propagation rule) | **correction**: the substitution table reads "T5 the call-site audit table". True and incomplete in the same way T4's row was — T5's real evidence is the audit table **plus** a discriminating capture the plan itself predicted. Row extended |
| §T5 | corrections: checklist ticked; the three open decisions recorded as taken; the two-commit landing recorded; F-23 landed as its own commit with its own frames |
| §T6 | **corrections — F-32** (the walk's reach: `append_child` / `insert_child` / `replace_child` on an attached tree and the IR loader's conditional / `for` mutation sites are shipped paths that put a fresh identity-scaled node under a scaled window, and §T6 names only `set_root` and T7's handler) and **F-33** (the measured atlas-offset distribution; and the baseline procedure its rendering gate depends on) |
| §T7 | **correction**: refreshing the node caches is not merely step 4 of DD-003's list but the discipline that keeps the parallel copy correct, so a T7 that reorders or short-circuits it breaks an invariant rather than skipping a step. Plus a confirmation folded in: step 3's "the nested `WM_SIZE` re-runs layout through T5's inbound seam" is now literally true and is a thing T7 can assert rather than describe |
| §T8 | **correction**: "Hold the *client* extent constant across the change" is now ambiguous in a way it was not before T5 landed, and the two readings are opposite. What must be held constant is the **DIP** client extent, which means the synthesised rectangle's *physical* client must move by the scale ratio. Read as "hold the physical client constant", the test would assert that layout results change |
| §T9 | no additional correction. T5 touches nothing T9 depends on: the declaration site, the one-shot guard, the feature list and the three-host rebuild are all unaffected by where a conversion happens |
| §T10 | **corrections — F-33** (control A's baseline procedure) and a measurement replacing a prediction: F-27 recorded that the aware-plus-correction state "must read 7 again once the inbound seam lands", and T5 measured **7**, so the plan can stop predicting it. The P2 state also gives T10 a stated expectation for what the same capture looks like once T6's walk lands |
| §T11 | no additional correction — owner-executed, and nothing T5 landed changes what the owner is asked to observe |
| §T12 | **correction**: two Moment 2 divergence items named now rather than discovered then — architecture.md §12.4's atlas-origin qualification, and the frame-reuse procedure that [verification-environments.md](../../../../docs/notes/verification-environments.md) Observation 4 and [constraints §9](../requirements/constraints.md) both state |
| [preamble.md](./preamble.md) §Implementation gates | **correction — F-32**: trap #3's narrowing gains T5's node scale cache as its second site |
| [preamble.md](./preamble.md) §Technical risks | **correction — F-33**: R-3 gains the measured offset distribution |
| [preamble.md](./preamble.md) §Obligations carried | **correction — Q7**: obligation 4's "T5 closes against those 13 rows" |
| [preamble.md](./preamble.md) review-lane table | no correction. T5's row is the one the table got right at drafting, and this task did not reach a class it does not already name |
| [preamble.md](./preamble.md) §The sequencing thesis | no correction. T5 is the *arithmetic* case the thesis was written for — every conversion executed and every one the identity — and the byte-identical frame set is that claim measured rather than argued. F-31's ordering qualification is untouched because T5 places no work relative to a message dispatch |
| [preamble.md](./preamble.md) §Verification closure | no correction. The task mapping is unchanged and item (2)'s qualification is T8's |
| [handoff.md](./handoff.md) | **corrections**: the scale-cache re-trigger row gains the two incremental attach classes, and three carry-forward rows land (row 9's divisor, the two conversions cancelling today, the walk's reach) |
| ADR set (preamble + DD-001 … DD-005) | **one owner question, no edit.** See below. Nothing else: DD-002's rows 9 and 12 are under-specified rather than wrong, and its "often `(0, 0)`" is a general claim T5 measured one instance of |

### Raised to the owner — the frame-reuse procedure

Not settled here, because it is a statement inside an **Accepted** ADR and
in an upstream requirements document, and T4's review established that an
implementation log is the wrong place to choose between annotating and
superseding.

The ADR set's [§Phase 1 verification closure](../decisions/preamble.md)
item 3 says: *"If a pre-change frame is reused rather than re-captured,
the commit it was captured at is checked against the current surface
first."* [constraints §9](../requirements/constraints.md) says the same in
Japanese, as a process premise carried from M3-Phase 8.

F-33 measures that the check is **not sufficient**: two captures at the
identical commit, in the same session and minutes apart, differed by 149
of 827,904 pixels when one of them was the session's first launch, and by
25 across two sessions a day apart. A reader following the procedure
literally would reuse a frame, find the commit unchanged, and report a
25-pixel regression that is not one.

Against the boundary T4's owner decision established — *supersede when a
reader implementing the original text would not obtain the shipped
behaviour; annotate when the decision stands and a statement around it was
too strong* — this looks like the **annotate** side: nothing is re-chosen,
the pair discipline is unaffected, and what is wrong is the sufficiency of
one check. But that is the owner's call, and there are three candidate
routes rather than two, because the operative correction can also live
only in [plan.md](./plan.md) §T6 / §T10 (where it already does) with
[verification-environments.md](../../../../docs/notes/verification-environments.md)
Observation 4 picking it up at T12 — which is the document later phases
actually read as procedure.

### Independent review disposition (Codex, 2026-07-29)

The full independent review the lane requires. Four findings: **three
major, one minor**, and **three of the four contradict a claim this log
made** — one of them the central argument of a decision the plan told T5 to
take, which was not merely under-stated but **backwards**. Each was
re-verified against the source or re-measured before acceptance; none was
taken on the reviewer's word.

| # | Finding | Verified | Disposition |
|---|---|---|---|
| R-1 | **major** — DD-002's audit table asserts "no coordinate enters or leaves outside these rows" while its row 2 names only `set_root`, and the closure was reached against a **14-site** extended table. The implementation is right; the *contract* is not | **Confirmed, and it is worse than the finding says.** T1 recorded the gap as F-1 and dispositioned it to T5 "as row 2b", explicitly declining to correct the immutable ADR. Re-checked from the code side: production `GetClientRect` has exactly two call sites and only `emit::flush_layout` is outside the table — so the extended enumeration is complete and the ADR's is not. **And the same gap is in the normative spec**: [architecture.md §12.3](../../../../docs/architecture.md#coordinate-spaces) states the inbound client-extent class as "at window attach and on every window-resize message", which omits the reactive drain's layout pass — a third site on the busiest path in the runtime | Raised to the owner below. The spec half is a **Moment 2 divergence item** and is added to [plan.md](./plan.md) §T12 |
| R-2 | **major** — decision B's reasoning is inverted: dividing the readback by each node's own cache is correct **only** when every node's scale equals the window's, while dividing by one window-level scale is correct for any mixture | **Confirmed, and the claim was wrong.** Worked through the recursion: `abs = off + vx` accumulates *parent-relative* readbacks, so the composited absolute position is `Σ(local_dip_i × scale_i)` while per-node division produces `Σ local_dip_i`. Those agree with `absolute_physical ÷ window_scale` — which is what `wnd_proc` hands in as the pointer — only if every `scale_i` is the window's. The reviewer's counterexample reproduces on paper: window scale 1, child cache 2, child local rect `x=10 w=10` renders at physical `[20,40)` and per-node division hit-tests `[10,20)`, so a pointer at physical 25 misses a widget it is over. **And the mixture is reachable by F-32's own path list** — a node attached to an already-attached tree keeps the constructor identity until a walk runs | **Implementation changed.** Both traversals now take the **traversal root's** scale and divide every readback by it, through a new `WidgetNode::visual_rect_dip`. Decision B is rewritten below with the corrected argument, and the source comment that carried the wrong one is replaced rather than left standing |
| R-3 | **major** — F-33's conclusion contradicts F-33's own table: the plan says a two-agreeing-capture baseline yielded exact equality "and T5 did", but `t5-baseline` vs `t5-after` is 25 / 6 / 25. The exact match was against the *committed* T3 set | **Confirmed, and the claim was wrong.** Re-ran the committed script: `t5-baseline` vs `t5-after` really is 25 / 6 / 25. Two separate results were conflated into one sentence — the procedure produced a *settled* baseline, and a *different* comparison produced the exact match | F-33's conclusion is narrowed to what was measured, in this log, [plan.md](./plan.md) §T6 / §T10 and [handoff.md](./handoff.md). The regression claim itself is untouched and stands on the other two legs: the byte-identical match against the committed T3 set, and the source audit |
| R-4 | **minor** — `compare-frames.ps1` prints "N frame(s) differ" and exits **0**, so a gate reading only the exit code reads a differing pair as success | **Confirmed by running it**: `$LASTEXITCODE` was unset on a pair with 25 / 6 / 25 differences | `exit 1` / `exit 0` added, and **verified to fire both ways** — 1 on the differing pair, 0 on the identical one. Same false-green family as F-21 and F-5, which is why it is worth the line |

**What the review confirmed independently**, so it is not re-argued here:
the rounding discipline (every production conversion goes through a named
operation; `factor()`'s only production use is T4's diagnostic; row 4
converts once on the DIP difference); the byte-identical scale-1 rendering
against the committed T3 set over the defined client interior; the
`WM_SIZE`-unsigned / pointer-signed split, checked against Microsoft's own
`WM_SIZE` and `WM_MOUSEMOVE` documentation, and that `as i16 as f32` is
lossless because every `i16` is exactly representable; **zero installers
for the six callback slots across the whole repository**, including that
the C header, the Zig binding and `rust-sys` all treat the type as opaque,
so the `f32` change is source-breaking only for a Rust-native caller that
does not exist; the F-23 fix's consistency and its isolated frame delta;
and the positive control's three tile counts read off the images.

**A note on the review's limit, and on mine.** R-2 is the finding that
matters, and the brief did ask about it — the question "does the reasoning
survive if two nodes hold different scales?" was one of the seven weak
claims named up front, and the answer came back "no, and it is backwards".
That is the second phase in a row where naming a weak claim produced the
sharpest result, and also the second where the author's own argument for a
*decision the plan asked to be argued* was the thing that failed. What the
brief did **not** name is R-3, a plain misreading of my own measurement
table, sitting two paragraphs below the table itself.

### Decision B, rewritten (T5 independent review finding R-2)

> **Read the last two paragraphs of this section with the first ones.**
> The argument below was itself corrected at the final review (S-2): the
> one-divisor form is **not** "correct for any mixture", it is correct for
> a traversal rooted at the window's root, and that is a **precondition on
> the public entry** rather than an invariant the runtime maintains.
> *(Pointer hoisted here at round 4, finding 8 — the correction previously
> arrived only after the superseded reasoning.)*

The original decision and its argument are withdrawn. They read: *"the
divisor is the node's own cache, because row 9 undoes row 4 — the same
variable by construction rather than by two variables agreeing."* The
premise is true of one node in isolation and does not survive the
traversal, which is where the value is actually used.

**Decided: every readback in a hit-test or hover traversal is divided by
the traversal root's scale**, through `WidgetNode::visual_rect_dip`.

The argument, stated over the traversal rather than over one node:

- A node's readback is its **parent-relative physical** offset, and the
  traversal accumulates those into an absolute position.
- The composited absolute position of a widget is
  `Σ(local_dip_i × scale_i)` — the sum of what each ancestor's
  `sync_visuals` actually wrote.
- The pointer arrives as `absolute_physical ÷ window_scale`.
- Dividing each term by its own `scale_i` before summing yields
  `Σ local_dip_i`, which equals the pointer's space **only if every
  `scale_i` is the window's**. Dividing every term by a single scale
  yields `Σ(local_physical_i) ÷ that scale` — the composited position, in
  the pointer's space — **for any mixture**.

So the choice is between an operation that is correct conditional on an
invariant the runtime cannot check, and one that is correct
unconditionally. The invariant is not hypothetical: F-32's own path list
— `append_child` / `insert_child` / `replace_child` on an attached tree,
and the IR loader's mutation sites — produces exactly the mixture, and
such a node is *already* rendered at the wrong size. With one divisor it
is at least hit-tested **where it actually is**, which is the question
hit-testing exists to answer; with per-node division it is rendered in one
place and hit-tested in another.

**Why the traversal root's scale and not the window's.** The traversal has
no window in hand — that is T1's carrier decision, and reopening it would
cost the public signatures and a `pub` export of `DipScale`, which DD-004
declines. The root's cache is where the walk starts, so the invariant
becomes **one point instead of one per node**.

**But "unconditional" is wrong, and the final review is right about it
(finding S-2).** The paragraphs above say the one-divisor form is correct
"for any mixture" and call the residual invariant single-point. That holds
**only for a traversal rooted at the window's root**. `hit_test_click` and
`update_hover` are `pub` and take the divisor from `self.scale`, so
entering on a *subtree* takes that subtree's cache while the pointer was
divided at `wnd_proc` by the **window's** — and `togglebutton_runtime_integration.rs`
does exactly that, calling `hit_test_click` on `built.root.children[1]`.
At a window scale of 1.25 with an unwalked subtree at 1, that entry
compares a `/1.25` pointer against a `/1` rectangle. The type cannot help:
`scale` is private, so an external caller has no way to supply the right
divisor even knowingly.

**What is true, stated at the size it is true at:** every *production*
caller enters on `WindowState::root_widget` — confirmed independently at
the final review — so the shipped path is correct, and the one-divisor
form is **strictly better than per-node** because it removes the per-node
requirement rather than because it removes all of them. The residual is a
**precondition on the public entry**, not an invariant the runtime
maintains, and it is carried forward as such. Observing it requires a
mixed-scale tree, which is T8's synthesised path.

**What did not change:** the pointer is still divided at `wnd_proc` by
`WindowState::scale`, the two conversions still cancel today, and at
`s = 1` this is the same arithmetic as before — which the re-captured
frame set confirms, since three of its six frames are taken after a click.

**A stated gap in the evidence, rather than an implied one.** The
correction is supported by the argument above and by a `s = 1` regression
set; **no test or capture exercises the mixed-scale case it exists for**.
It cannot be reached from the test suite — a mixed-scale tree has to be
built through `WidgetNode`s directly, which F-4 measured never routes a
coordinate through a window's scale — and the phase's synthesised
scale-change evidence is T8's. So this lands on reasoning plus a
no-regression check, and T8 is where it becomes observable.

### Post-review plan re-audit (2026-07-29, in-gate)

The pass T4 dropped: after a review's findings are dispositioned, re-read
the **task list as a whole** against what the review taught, rather than
only propagating the findings into the documents that carried them. Run
proposition-first, as the earlier pass was.

| Proposition established by the review | Asserting documents enumerated, then checked | Verdict |
|---|---|---|
| **P-A.** The readback's divisor is one per traversal, not one per node; the residual invariant is single-point | §T5's decision block; §T5's carrier bullet ("`hit_test_click_inner` and `update_hover_inner` read `self.scale`"); [handoff.md](./handoff.md)'s row; the T5 retrospective; DD-002 row 9 (says "÷ s" and names no divisor — **under-specified, not falsified**, and immutable) | **corrections in the first four.** DD-002 needs none, which is worth stating: the row was silent on exactly the point that turned out to matter, and silence is what let the wrong answer be recorded as a decision |
| **P-B.** A node the scale walk never reaches is not only rasterized at the identity — before this correction it was also hit-tested where it is not drawn | §T6's walk bullet (states the crispness bound only); §T5's direct-hosting limit; [handoff.md](./handoff.md)'s cache row | **correction — §T6 and the retrospective.** The one-divisor change removes the hit-test half, so T6's remaining obligation is the rendering half. Worth recording because it *narrows* T6's risk, and a later reader would otherwise re-derive a hazard that no longer exists |
| **P-C.** DD-002's 13-row enumeration is not the complete contract it claims to be | **the ADR set's own §Implementation gates** ("the seven coordinate-carrying paths … are the audit table"); [preamble.md](./preamble.md) §Implementation gates trap #1 and §Obligations carried 4; §T5's end gate; **[architecture.md §12.3](../../../../docs/architecture.md#coordinate-spaces)** | **corrections in the implementation preamble and §T12.** The ADR-side handling is the owner's; the spec side is Moment 2's. Note the ADR set's gates section traces the table to [constraints §4](../requirements/constraints.md)'s "seven coordinate-carrying paths", so the undercount is inherited from the requirements document rather than introduced at ADR time |
| **P-D.** An evidence script that prints a verdict without setting an exit code is a false-green generator | `compare-frames.ps1`; §T6's and §T10's gates, which now name it | **fixed and verified both ways.** No further site: T3's and T4's capture scripts throw on their own failure modes and are not used as pass/fail gates |

Tasks re-read against the review, with nothing to correct: **§T7** (the
`WM_DPICHANGED` ordering and its enumeration are untouched by any finding;
the walk obligation F-32 added still stands, and P-B narrows what a missed
walk costs without changing that the handler must run it); **§T9**;
**§T11**; **§T8**, whose two T5-driven corrections (the client-extent
reading, and 100 DPI) are unaffected — though note that P-A's mixed-scale
case is now *the* thing T8's synthesised change can observe and nothing
before it can, which is recorded in the retrospective as a stated evidence
gap rather than as a new T8 item, because §T8 already drives the handler
that produces the mixture; the implementation preamble's review-lane table,
§The sequencing thesis and §Verification closure; and the ADR set apart
from the owner question above.

### T6's landing site, read before hand-off

T4 set the precedent of reading the *next* task's landing site at the
source before handing over, on the ground that the plan names a task's
work but not its shape in the code. **This pass was initially skipped and
the owner asked for it** — the post-review re-audit above answers "what did
the review falsify in the task list", which is a different question from
"what does T6's landing site look like, and what has the plan not named".
Read end to end: [`text.rs`](../../../../wasamo-runtime/src/text.rs) in
full, the five `draw_text` call sites in
[`widget.rs`](../../../../wasamo-runtime/src/widget.rs) (`WidgetNode::text`,
`button_family`, `update_button_label`, `update_text_content`,
`update_text_style`), `window::set_root` as landed, and
`run_layout` / `run_layout_as_window_root` / `sync_visuals`.

Two open points the plan does not name. The first is the more serious and
it reaches T7 as well.

**F-34 — the walk writes the scale that the geometry pass has already
read, so "after the first layout" produces a tree drawn at 1/s.**

[plan.md](./plan.md) §T6 specifies the walk as
`apply_scale_recursive(…)` "called from `window::set_root` **after the
first layout**", and states — correctly, and for T3's one-geometry-pass
invariant — that "**the walk also writes no Composition geometry**". Put
those two together against what T5 landed and the sequence does not
close:

1. `set_root` converts the client extent and calls
   `run_layout_as_window_root`, which lays out **and calls
   `sync_visuals`**.
2. `sync_visuals` multiplies every offset and extent by **`self.scale`,
   the node cache** — which is still `DipScale::default()`, because the
   walk is the cache's only writer and has not run.
3. The walk then runs, updates every node's cache, rebuilds the text
   surfaces at the right resolution — and writes no geometry.
4. Nothing re-runs `sync_visuals`. **The Visual tree keeps the identity
   projection**: a correct DIP layout drawn at 1/s in the corner of the
   client area.

That is not a prediction: it is the state T5 photographed as **P1**
([evidence/t5-probe/](./evidence/t5-probe/)), where the tree occupies
785.6 × 562.4 of a 982 × 703 client. T5's record says that state is
"correct for T5 alone, because T6 owns the walk that writes the cache" —
which is true of T5 and **not** true of T6 as specified: the walk as
written updates the cache after the only pass that reads it.

**The same defect is in T7's step order**, and there it sits inside an
ordering DD-M4-P1-003 calls fixed and load-bearing: (1) update the scale,
(2) apply the OS rectangle via `SetWindowPos`, (3) the nested `WM_SIZE`
re-runs layout, (4) re-rasterize through the walk. Step 3's `sync_visuals`
reads the node caches; step 4 writes them.

**What this means for DD-003 is *not* settled here — withdrawn at the
final review (finding S-4).** This entry originally concluded "the ADR
needs no change", by reading DD-003 step 1's "update *the cached scale*"
as a collective term covering the per-node caches T1 introduced later. The
reviewer is right that the text does not support it: DD-003 names
`WindowState`'s field explicitly and *chose* that field as the storage, so
reading it as a collective for a carrier that did not exist is the
convenient reading rather than the textual one. Recorded as a withdrawal
rather than quietly rewritten, because it is the second time in this task
that an argument of mine for "no change needed" turned out to be built
backwards from the answer I wanted.

**The route depends on the shape T6 and T7 choose.** If only the cache
*write* moves into step 1 and the fallible surface rebuild stays at step
4, a dated annotation may cover it; if the whole walk moves, the fixed
order itself changes and that is a successor. **Decide the shape first,
then the record** — and the record is an owner decision either way.

**What T6 decides** (recorded as the choice, not pre-empted): run the
whole walk *before* the first layout, or split it so the cache write
precedes layout and the surface rebuild stays where it is. The walk has no
dependency on layout results either way — it rebuilds from
`WidgetData::Text { content, style }` and `ButtonData`'s retained
`label_text` / `label_style` / `label_size`, and `measure` is DIP and
scale-invariant (audit row 10). What T6 must **not** do is give the walk a
geometry write to fix the projection, which would break the invariant T3
established and the T5 audit rests on.

**A consequence for trap #6, which the plan arms but does not aim.** The
walk is O(text nodes) of WinRT-fallible calls, so a failure part-way
leaves **some nodes at the new scale and some at the old** — and because
`sync_visuals` reads that cache, the result is a tree drawn at two scales
rather than merely one whose text is stale. T6 states the failure policy
(`set_root` discards its layout `Result` today; DD-003 fixes log-and-survive
for the change path). Recorded because "a half-walked tree" is a state the
gate should name before it is met.

**F-35 — `TextRenderer::draw_text` is public, and the type T6 needs to
pass it is not.**

§T6 says "thread the scale into `draw_text`'s five call sites". The
signature it threads into is `pub fn draw_text` on `TextRenderer`, which
[`lib.rs`](../../../../wasamo-runtime/src/lib.rs) `pub use`-exports
alongside a public `get_text_renderer()`. So this is a **public
Rust-native signature change**, in the same class as T5's callback slots —
and `DipScale` is crate-private (`mod dip_scale;`), so it cannot appear
there without making the type public.

Audited rather than assumed: `draw_text` has **no caller outside
`widget.rs`** in the whole repository, so the change costs nothing today.
`get_text_renderer()` has 26 test call sites, but every one of them hands
the renderer to a `WidgetNode` constructor rather than drawing with it.

**T6 decides what crosses that boundary**, and the options are not
equivalent: a `u32` DPI keeps `DipScale` internal and hands the callee the
value D2D actually wants (T4's carrier reversal made `96 × s` exactly the
DPI); an `f32` factor is the `factor()` reach F-15's carry-forward names;
making `DipScale` public ships a scale type on the Rust-native surface
that DD-M4-P1-004 declined to give hosts. Naming it here so it is decided
rather than met mid-edit — which is exactly what T4 did for T5's three
open points.

### Final review disposition (Codex, 2026-07-29) — whole branch

The second and final independent review, over implementation, evidence and
documents together, and over whether the first round's dispositions landed.
Nine findings: **four major, four minor, one nit**, and **four of them are
my own claims being wrong** — including one where the tool I built to close
an owner's condition would have misled the very task it was written for.

| # | Finding | Verified | Disposition |
|---|---|---|---|
| S-1 | **major** — the drift/material rule can exit 0 on a real change. "Any real change moves geometry and therefore produces a full-contrast pixel" is false: a rasterization defect changes intensity without moving geometry, a sub-pixel error need not flip coverage, and contrast belongs to the edge, so "palette-independent" is wrong too | **Confirmed, and the claim was wrong — with the sharpest instance being the one I handed to T6.** A wrong D2D context DPI is exactly an intensity-only change, i.e. **T6's defining failure would have been classified as capture drift by a rule written for T6's gate.** The third counterexample is arithmetic: the gallery's own tile fill and page background differ by 32/46/55, so a *lower*-contrast pair anywhere gives a geometry move a delta under the threshold | **Default reverted to R-4's property: any difference exits non-zero.** The max delta stays, reported as **asymmetric evidence** — a large one proves a material change, a small one proves nothing — and the allowance becomes opt-in `-AllowDrift`, a judgement the runner records. Verified on five cases including that a material change stays material under `-AllowDrift`. Corrected in [plan.md](./plan.md) §T6 / §T10, [handoff.md](./handoff.md) and [evidence/README.md](./evidence/README.md), whose "non-zero on any difference" line the extension had also falsified |
| S-2 | **major** — R-2's "single-point invariant" does not hold at the public subtree entry: `hit_test_click` / `update_hover` take the divisor from the receiver, so entering on a subtree uses *its* scale while the pointer used the window's | **Confirmed, and the brief named this one — the answer came back that I had moved the hole rather than closed it.** `togglebutton_runtime_integration.rs:198` calls `hit_test_click` on `built.root.children[1]`, so the entry is used that way in this repository. `scale` is private, so an external caller cannot supply the right divisor even knowingly | Claim narrowed everywhere from "correct unconditionally" to **"correct for a traversal rooted at the window's root"**, and restated as a **precondition on the public entry** rather than an invariant the runtime maintains. Doc comments on both entries and on `visual_rect_dip` say so; carry-forward 1 rewritten. What survives is the real gain: one divisor needs the *root's* cache to be right instead of *every* node's |
| S-3 | **major** — R-2's disposition never reached the close-gate audit table: row 9 still recorded "`self.scale.pair_to_dip` at both `visual_rect` call sites" | **Confirmed.** `visual_rect` now has exactly one caller (`visual_rect_dip`) and that has two; the row described the first landing | Row 9 rewritten with what landed, and marked as twice-corrected. **This is the fifth instance in the phase of a correction reaching the prose and missing the artifact** — and the artifact here is the one the task's whole claim rests on |
| S-4 | **major** — F-34's "DD-003 needs no change" is not derivable from the ADR: it names `WindowState`'s field explicitly and chose that field as the storage; node caches did not exist | **Confirmed, and the reading was convenient rather than textual.** Recorded plainly because it is the second time in this task that an argument of mine for "no change needed" was built backwards from the answer I wanted | **Withdrawn.** The route now depends on the shape T6 and T7 choose — cache-write-only into step 1 may be an annotation, moving the whole walk changes the fixed order and is a successor. **Decide the shape first, then the record**, and the record is the owner's |
| S-5 | **minor** — the comparison enumerates only the left directory, so a frame added on the right passes | **Confirmed by running it** against a directory with an extra PNG | Both file *sets* are compared first; `EXTRA on the right` counts as material. Verified |
| S-6 | **minor** — the owner's R-1 decision did not reach the implementation preamble or the retrospective, and the retrospective's commit list omits four substantive commits while claiming to be at final branch state | **Confirmed at all three.** | Preamble and retrospective items 4, 6 and 7 updated to the decision as taken; the commit list completed. Recorded with the irony intact: `9dc090a` **wrote the termination rule for this exact class**, and the four omissions landed after it — a rule is not a substitute for running it |
| S-7 | **minor** — the callback-slot counts are wrong: six fields, **four** carry coordinates, **three** changed type (`resize_fn` was already `f32`) | **Confirmed against the source** | Corrected in [log.md](./log.md) (decision C and side-effect row 2), [plan.md](./plan.md) §T5 and [handoff.md](./handoff.md), with all three numbers stated rather than one |
| S-8 | **minor** — F-33's reclassification reached the log and the plan but not the handoff, which still called the drift "tile-label glyph antialiasing" | **Confirmed** — and that phrase is the one T5 had already established as *false* (a button label is in the set) | Handoff row restated: every differing pixel is a text pixel, no coverage flips, **antialiasing is not established as the cause** |
| S-9 | **nit** — 26 `get_text_renderer()` test sites, not 27 | **Confirmed by count** | Corrected in both places. The conclusion — no `draw_text` caller outside `widget.rs`, verified by the reviewer across the C header, the Zig binding and all examples — is unchanged |

Also corrected without a finding: the branch is **13** commits, not the 15
this task reported.

**What the review confirmed independently**, so it is not re-argued: the
F-23 fix is unconditionally right on the drain path, because
`flush_layout` only ever holds `WindowState::root_widget` and therefore
carries the same window-root contract as `set_root` and `WM_SIZE`; no
external `draw_text` caller exists anywhere, including the C header, the
Zig binding and every example; the evidence README's claims about which
set is current and which two frames are stale, checked against the images;
and an independent search of the Win32 geometry and message surface, the
Composition geometry and readback surface, clips, surfaces and
D2D/DirectWrite found **no unenumerated production conversion seam** other
than row 7, which is T6's.

**What the two rounds together say about this task's failure mode.** Six
of the thirteen findings across both reviews are the same shape: an
argument of mine that was sound about the thing in front of me and wrong
about the thing it was being applied to. Row 9 undoes row 4 — true of one
node, false across a traversal. Any real change moves geometry — true of
the changes I had measured, false of the class T6 will produce. The cached
scale is one thing — true when DD-003 was written, false after T1. In
each, the local reasoning was right and the **scope** of the claim was
not, and in each the phrase that gave it away was an absolute:
*unconditional*, *palette-independent*, *needs no change*.

### Post-final-review plan re-audit (2026-07-29, owner-prompted)

**Not run until the owner asked.** After the final review I propagated
S-1 … S-9 into the documents that carried them and stopped — which is
precisely the failure this log records against T4 ("only *propagation*
ran … the task list was never re-read as a whole against what the review
had taught"), committed by the task that recorded it. **Writing the rule
down is what I did instead of running it** — the same observation S-6
makes about the commit-list termination rule. Two instances in one task
is a pattern, not a slip.

Run proposition-first. Four propositions came out of the final review;
each written as a sentence, the asserting documents enumerated, then
checked.

| Proposition | Asserting documents enumerated, then checked | Verdict |
|---|---|---|
| **P-E.** A small pixel delta proves nothing, because an intensity-only *real* change exists — and it is T6's defining failure | §T6 end gate, §T10 control A, [handoff.md](./handoff.md), [evidence/README.md](./evidence/README.md) — all corrected in the S-1 pass; **§T6's end gate as a whole**, which that pass did not re-read | **correction — §T6's end gate.** See F-36 |
| **P-F.** The hit-test entries carry a precondition the type cannot enforce, and observing its violation needs a mixed-scale tree | §T5 (corrected), [handoff.md](./handoff.md) (corrected), **§T8** — which this log twice names as where the case becomes observable | **correction — §T8.** Its bullets do not own it. F-26's shape exactly: a claim assigned to a task whose checklist never received it |
| **P-G.** Whether DD-003's ordering needs a record depends on the shape T6 and T7 choose | §T7 (corrected), this log (corrected), **§T12's phase-end list** — the only place a decision owned by neither T6 nor T7 can live | **correction — §T12.** Nothing owned the record decision S-4 deferred |
| **P-H.** A rule written into a document is not self-executing | the retrospective (recorded there) | no plan correction — a learning about this task's conduct, not a constraint a later task should read as procedure |

Tasks re-read with nothing to correct, stated so the pass is auditable:
**§T9**, **§T11**, **§T10** beyond the S-1 correction already applied, the
implementation preamble's review-lane table, §The sequencing thesis,
§Verification closure and §Obligations carried, and the ADR set — whose
only open question is P-G's, now owned.

**F-36 — T6's end gate covers one third of what T6 does.**

§T6's end gate asks for "local rendering unchanged at 100%". Checked
against what T6 actually changes, at `s = 1`:

- **The D2D context DPI** becomes `96 × 1` = 96. **No-op.**
- **The atlas origin division** becomes `offset ÷ 1`. **No-op.**
- **The `ceil` surface allocation** is *not* a no-op: the measured gallery
  surfaces are `15.81 × 18.62`, `46.57 × 18.62`, `72.03 × 18.62` …, so
  `ceil` changes every one of them while the Visual keeps the exact `f32`
  extent — so surface and Visual **stop being the same size**, and the
  brush's mapping between them starts to matter.

**One qualification, because the first two drafts of this finding both
overreached.** The first said the gate was empty; the second said it
"does exercise the brush mapping and would catch a stretch". Neither is
supported: the runtime **never calls `SetStretch`**, so the behaviour
rests on `CompositionSurfaceBrush`'s WinRT default, which I have not
verified — and DD-002's "the default surface-brush stretch maps one texel
to one device pixel" is stated for the case where the two numbers agree,
which `ceil` is precisely what ends. Found by checking my own claim after
writing it into the plan, which is later than it should have been. **T6
verifies the default**; what this finding supports is only that the
allocation and its brush are the **one part of T6 the 100% gate can
reach**.

So the gate is not empty, but it reaches the allocation and **neither of
the two changes that actually buy crispness**. That is F-31's shape a second
time: a gate that passes while the task's deliverable is absent, because
the deliverable is unreachable at the scale the gate runs at.

**T5 demonstrated the technique that closes it, and it cost one line.** A
throwaway `SetProcessDpiAwarenessContext(PMv2)` in `runtime::init()`,
reverted before the task closes, makes the scaled path observable without
waiting for T9. T5 used it for the 9 → 7 tile control, and its **P2**
capture ([evidence/t5-probe/](./evidence/t5-probe/)) is already T6's
before-picture: correct geometry at 125% with visibly soft glyphs, which
is R-1's premise rendered rather than argued. T6 is not obliged to take
it — but "the frame at 100% is unchanged" is not evidence that text is
crisp, and T6 is the task where that distinction is the whole point.

*Disposition:* [plan.md](./plan.md) §T6's end gate.

**F-37 — §T8 does not own the mixed-scale case that T5 assigned to it.**

S-2's disposition states twice — in Decision B and in the carry-forward —
that a tree whose nodes hold different scales is where the hit-test
precondition can be violated, and that **observing it belongs to T8**,
because it needs a scale change driven through the handler. §T8's
checklist has six bullets and none is that. A T8 that closed every bullet
it was given would leave the case unobserved — the gap F-26 recorded when
audit row 13 had no owner.

*Disposition:* [plan.md](./plan.md) §T8 gains the bullet, stated as what
it must construct rather than as a reminder.

**F-38 — nothing owned the DD-003 record decision that S-4 deferred.
Withdrawn as first written (T5 re-review finding 9).**

S-4's disposition is "decide the shape first, then the record", and the
shape is T6's and T7's. F-38 then claimed the *record* decision was owned
by neither, "because both tasks will have closed before it can be
answered", and put it on §T12's phase-end list.

**That reasoning is wrong.** T7 is the task that *chooses* the ordering
shape, so the record can be decided at T7's own close — and §T7 already
places it with the owner there. "Owned by neither" was the convenient
reading, and it is this task's recurring error in its **third** instance:
local reasoning sound, scope of the claim not.

What survives is smaller and is what §T12 now carries: a **safety-net
audit** — confirm T7 closed the record question, and file it at phase end
only if it did not. The gap F-38 was reaching for is real (nothing was
written down anywhere), but the owner is T7, not the phase-end batch.

### Narrow re-review disposition (Codex, 2026-07-29) — the dispositions themselves

Third pass, requested by the owner, scoped to what changed since the final
review: whether S-1 … S-9 landed, whether F-36 / F-37 / F-38 stand, and
whether the `compare-frames.ps1` rewrite is sound. **Nine findings: three
major, six minor.** The production Rust diff in the range is doc comments
only, verified by the reviewer.

| # | Finding | Verified | Disposition |
|---|---|---|---|
| 1 | **major** — the withdrawn pass-rule survives as a *current* description in this log's §Making option (A)'s condition true and in the retrospective's item 10 (4), including `-Exact`, a switch the script no longer has | **Confirmed.** The log section reads as a standing description of how the tool works, not as a dated record | The log section is headed as **superseded**, pointing forward to §Final review disposition and the script header, and naming `-Exact`'s removal explicitly. The retrospective entry is rewritten to the asymmetric-evidence reading |
| 2 | **major** — the §T8 bullet F-37 added does **not** observe the hole it was written for. A mixed-scale tree is not the failure; the one-divisor form is correct for any mixture **while the traversal root's scale is the window's**. The failure needs the stale subtree to be the **receiver** | **Confirmed, and it is the same local-right / scope-wrong error the task keeps making — inside the finding that was itself correcting one.** Also confirmed: "T8 is the only task that can" is false, since T6 can reach it with the throwaway declaration §T6 now describes | Bullet rewritten to name the receiver, and the "only task" claim dropped. T8 keeps it because T8 drives the handler, not because nothing else could |
| 3 | **major** — the brush default is not unverified: `CompositionSurfaceBrush.Stretch` defaults to **`Uniform`** with alignment ratios 0.5, so a `ceil`-ed surface is scaled down and centred rather than padded | **Accepted on the reviewer's documentation citation, and consistent with what ships**: at `s = 1` today surface and Visual sizes are equal, so any stretch mode looks the same, which is why nothing has surfaced. Not measured here | §T6 turns from "verify the default" into **"set the stretch and alignment explicitly, and confirm by measurement"**. Two non-signals the reviewer supplied are recorded with it: removing `width.max(1.0)` produces no independent visible difference, and an omitted walk looks identical to constructor scale-1 surfaces at 100% |
| 4 | **minor** — two empty directories compare "identical" and exit 0 | **Confirmed by running it.** Pre-existing, not a rewrite regression | Rejected with exit 1 and a message naming which side was empty. Verified |
| 5 | **minor** — "a pixel moved across contrast" asserts a mechanism the measurement does not support; an intensity-only defect can exceed the bound too | **Confirmed** — and it is S-1's own counterexample pointed the other way | Verdict and header restated: a large delta proves only that the difference **exceeds the measured drift bound**, not what caused it |
| 6 | **minor** — S-2's correction reached the source comments and Decision B but not §T5's decision block or the handoff carry-forward, which still read "any mixture" and "single-point invariant" | **Confirmed at both** | Both rewritten to **precondition on the public entry**, with the receiver problem and the private `scale` stated |
| 7 | **minor** — the retrospective's commit list omits `48061b5` and `b61009e` while calling itself final branch state, and item 4 says all owner decisions are settled and then says R-1 is pending | **Confirmed. Third recurrence in this task** | Both fixed — and see the note below, because three recurrences is no longer a slip |
| 8 | **minor** — "four pointer slots" survives in the retrospective and in this log's proposition table; "tile-label glyph antialiasing" survives in §T10, so S-8's claim that the plan was already correct was false | **Confirmed at all three** | Corrected |
| 9 | **minor** — F-38's phase-end ownership is backwards: §T7 already places the record with the owner, and T7 is when the shape is known | **Confirmed** | F-38 withdrawn above; §T12 keeps a safety-net audit only |

**S-1 … S-9 verdicts from the reviewer:** S-3, S-4, S-5 and S-9 verified
as landed; S-1, S-2, S-7 and S-8 **partial**; S-6 **not fully landed**.
Four of nine dispositions incomplete.

**The pattern is now the finding, and it is mechanical rather than moral.**
Three rounds, and each round found the previous round's *dispositions*
incomplete — R-2's missed the audit table (S-3), S-1's and S-2's missed
the log and the handoff (re-review 1 and 6), S-8's asserted a correction
that had not been made. The common shape is that I correct **the site the
finding names** and then answer the propagation question from memory
rather than by enumeration. The proposition-first rule exists for exactly
this and I have been applying it to *findings from the work* while
handling *findings from a review* as a list of addresses.

The concrete carry-forward, and it is cheap: **a disposition is not done
until the corrected proposition has been searched for across the plan set,
the same way a finding is** — and the retrospective's commit list, which
has now drifted three times in one task, wants a check that reads
`git log` rather than a rule that says to.

### Round-4 review disposition (Codex, 2026-07-29)

The owner's merge gate is now explicit: **reviews continue until a round
returns zero major findings.** This round returned **six major and two
minor**, so the gate is not met by it. Every finding accepted; none argued
down. The range reviewed was one commit — the previous round's
disposition — which is the point: **the dispositions are what keep
failing, not the code.**

| # | Finding | Verified | Disposition |
|---|---|---|---|
| 1 | **major** — §T8's rewritten bullet asks T8 to assert that a **documented misuse** misbehaves, pinning a stated limit as a regression contract | **Confirmed.** `hit_test_click`'s doc comment already states the receiver precondition, so a test on the stale-subtree receiver would fix a known limit in place | Bullet re-aimed at the **legitimate** path: a descendant whose cached scale is not the window's, hit-tested **from the window root**, resolves to where the widget is composited. That is the property one divisor gives and per-node did not. The receiver case stays a stated limit with no test |
| 2 | **major** — "set stretch and alignment explicitly" does not specify values; `Fill` and `UniformToFill` satisfy it. The contract needs `CompositionStretch::None` with alignment ratios `0.0`. **And DD-002's own step 4 says the *default* brush maps one texel to one device pixel**, which its own step 1 (`ceil`) contradicts | **Confirmed, including the ADR half** — step 1 specifies `ceil(dip × s)` and step 4 asserts the two numbers agree, in the same section | §T6 gains the values and the obligation to **measure** them rather than inherit the citation. The DD-002 sentence is an incorrect implementation explanation under an unaffected decision — **raised to the owner below**, not annotated here |
| 3 | **major** — round-3 finding 5 was fixed in the script and **not** in §T6, §T10, the handoff or the README, which still said a large delta proves a material change; §T10 additionally generalised that a crispness change "alters coverage and lights up full-contrast deltas", contradicting the intensity-only concession two sentences above it | **Confirmed at all four** | All four corrected to *"outside the bound this phase measured, and nothing about what caused it"*. §T10's crispness sentence is replaced: crispness is a **glyph-shape judgement on the magnified pair**, and a pixel count cannot stand in for it in either direction |
| 4 | **major** — S-2's precondition correction reached the plan and the handoff but not the retrospective's carry-forward, which still read "correct for any mixture" and "single-point invariant" | **Confirmed** | Rewritten, with the receiver problem, the private `scale` and the production-caller fact stated |
| 5 | **major** — §T12's item contradicts itself: the head says T7 owns the record and phase-end audits, the body still says "owned by neither, because both will have closed" | **Confirmed** — I prepended the withdrawal and left the old body under it | Body rewritten. The item is now unambiguously *phase-end's to check, not to decide* |
| 6 | **major** — the commit list drifted a **fourth** time (`8f7803e` present only as an unhashed placeholder) while claiming final branch state, and the "mechanical check" I said was needed was another rule with no owner | **Confirmed** | **Built** as [evidence/list-task-commits.ps1](./evidence/list-task-commits.ps1), which reads `git log` and separates substantive from bookkeeping commits; run against this branch, it reports 15 substantive and 3 bookkeeping of 18. The retrospective's list is reconciled against its output |
| 7 | **minor** — "callback slot 6 個は DIP を運ぶ" survives in the retrospective's carry-forward | **Confirmed** | Corrected to six fields / four carrying / three retyped |
| 8 | **minor** — the superseded-in-place warnings sit *after* the superseded reasoning | **Confirmed** | Both hoisted to the top of their sections as block quotes |

**Raised to the owner — a second DD-002 annotation.** DD-002 §The
rasterization surface says, in step 1, that the surface is
`ceil(dip × s)` pixels, and in step 4 that "the visual's `Size` is the
*physical* size … and the surface is `dip × s` pixels, so the **default**
surface-brush stretch maps one texel to one device pixel". The two cannot
both hold: `ceil` is what makes them differ, and the default is `Uniform`
with 0.5 alignment, which scales and centres. **The decision is
unaffected** — allocate at device resolution, keep the brush one-to-one —
and what is wrong is the sentence explaining how it is achieved, which is
the annotate side of the boundary the owner set at T4. It is the second
annotation DD-002 would carry. T6 implements `None` + `0.0` regardless, so
nothing is blocked either way.

**Why this round was six.** Every one of the six is a **disposition that
did not land**, not a new defect in the work: four are corrections applied
to the site a finding named and not to the proposition's other homes, one
is a withdrawal prepended to text it contradicts, and one is a rule
written where a mechanism was needed. The pattern named after round 3 —
"I answer the propagation question from memory rather than by
enumeration" — reproduced at full strength in the very round that named
it. The one thing done differently here is finding 6's remedy: **a script
instead of a sentence.** Whether the same substitution is available for
the propagation problem is the open question this task hands forward.

### Round-5 review disposition (Codex, 2026-07-29)

**Seven major, one minor — the gate is not met.** Five of the seven are
again dispositions that did not land, including two that did not reach the
retrospective *for the fourth consecutive round*. That repetition is what
finally produced a mechanism rather than a resolution; it is stated first
because it is the useful part of this round.

**Why the retrospective keeps being missed, diagnosed rather than
deplored.** The propagation rule is "write the falsified proposition as a
sentence, enumerate the documents that assert it, then search". I have been
executing it — and searching **in English**, while
[t5.md](../retrospectives/t5.md) states every one of those propositions
**in Japanese**, per the project's language rule. A proposition search over
English phrasing cannot find 「最大差で判定する」 or 「残る不変条件」, no
matter how carefully the propositions are named. That is not carelessness;
it is a search over the wrong index, and it explains S-8, round-4 findings
4 and 7, and round-5 findings 6 and 7 as one cause.

**The correction, and it is mechanical:** the enumeration list for *every*
correction includes `retrospectives/t5.md` unconditionally, and the search
runs in both languages — or, more reliably, the retrospective is re-read
against the corrected proposition rather than searched. Recorded in the
retrospective's item 1 and carried forward.

| # | Finding | Verified | Disposition |
|---|---|---|---|
| 1 | **major** — `list-task-commits.ps1` prints a classification and exits zero regardless, so the list drifted a **fifth** time under the script built to stop that: `c75856e` without a hash, `63bfd55` absent | **Confirmed by running it.** "A tool that reports without comparing is the rule it replaced, with extra steps" | Rewritten to **read the retrospective**, extract the hashes it claims, and **exit 1 on any disagreement**. Run now, it reports both missing commits and fails. The bookkeeping classification is still a subject-line heuristic and is now **printed for eye-checking**, with the comparison — not the heuristic — as what fails |
| 2 | **major** — DD-002's second annotation is on the supersede side of this set's own boundary: a reader implementing step 4 gets the default brush and therefore scaled, centred glyphs | **Confirmed against the boundary as written.** My ground was "no option is re-chosen", and that is **not the test this repository uses** — the test is whether the original text yields the shipped behaviour, and it does not. Same error as T4's first disposition of I1 | **Raised to the owner** below. Not re-routed here: the owner approved the annotation, and choosing between annotate and supersede is explicitly not an implementation-log decision |
| 3 | **major** — the owner-approved second annotation reached DD-002 and **not** the ADR-set preamble's revision history | **Confirmed.** Third instance in this task of a correction reaching the carrier and not the summariser | Preamble row added, including a pointer to finding 2's open question so the record does not have to be found twice |
| 4 | **major** — `architecture.md` §12.4 carries a *determined* correction (the brush sentence) deferred to T12, unlike the atlas sentence, which is an empirical overstatement whose conclusion is unchanged | **Confirmed as a real distinction**, and it is sharper than the one I drew: the brush replacement value is already fixed by the owner-approved DD correction, so deferring it leaves a normative instruction that is known-wrong and known-how-to-fix | **Raised to the owner** below, with the atlas item explicitly *not* included |
| 5 | **major** — §T8's bullet is mathematically right but its **constructibility depends on a T6 decision that is open**: if T6's walk covers the incremental attach paths, no ordinary path produces a stale descendant and T8 needs a test seam; if not, a post-change `append_child` builds it | **Confirmed.** Also confirmed: the bullet said "nothing before T8 can exercise it" two lines above conceding T6 can reach the same tree | Bullet made explicitly conditional on T6's answer, with both branches named — including that the seam, if needed, is the `lib.rs::ffi` shape F-29 already identifies |
| 6 | **major** — the public-entry precondition never reached the retrospective's carry-forward, which still said "the residual invariant is guaranteed by T6" | **Confirmed** | Rewritten. T6 can guarantee that the *walk* starts at the root; it cannot guarantee **which node a caller passes**, which is the whole content of the precondition |
| 7 | **major** — the delta proposition never reached the retrospective either, leaving a heading that says "judge by max delta" over a spliced sentence carrying half the withdrawn claim | **Confirmed** — the splice is visible in the file | Heading and body rewritten |
| 8 | **minor** — §T12 says "Two … items" and lists four | **Confirmed** | Corrected to four |

### Owner decisions on the round-5 questions (2026-07-29)

**Both questions were raised because they came out of review findings, and
the owner's instruction is general: where a question I escalate originates
in a reviewer's finding, follow the reviewer's recommendation.** That is a
standing rule worth recording, not a one-off — it removes an escalation
whose only content is my own uncertainty about a call someone else has
already made with reasons.

So both went the reviewer's way:

1. **DD-002's step-4 mechanism clause is superseded, not annotated.**
   [DD-M4-P1-006](../decisions/dd-m4-p1-006-surface-brush-mapping-is-set-not-inherited.md)
   filed `Proposed`: the runtime sets `CompositionStretch::None` with
   alignment ratios `0.0`. The annotation approved earlier the same day is
   re-headed "Superseded in part … by DD-M4-P1-006" and its substance left
   standing, since what changed is the record's form. This is the set's
   **second supersede**, and — recorded because the pattern is the point —
   the second time in this task that "no option is re-chosen" was used as
   the test when the test is *whether a reader implementing the original
   text obtains the shipped behaviour*.
2. **`architecture.md` §12.4's brush sentence is corrected now**, in the
   same batch, rather than deferred to Moment 2. The distinction the
   reviewer drew is the operative one: a *determined* correction leaves a
   wrong instruction in the document external readers implement from,
   while an *unverified divergence* is exactly what T12 exists to
   reconcile. The atlas-origin overstatement in the same section stays
   Moment 2's, for that reason.

**A third instruction, on the commit list, changed the design rather than
the disposition.** The list had drifted five times, and my remedies had
been four successive rules and then a script that detected the drift. The
owner's instruction removes the class instead: **record only the hashes at
the point the code was fixed and tested**, so subsequent documentation
commits are not part of the list and cannot make it stale. The
retrospective now names three commits — the seams, the F-23 fix and the
R-2 correction — and the comparison script is **deleted**, because a tool
whose purpose has evaporated is the same "add a mechanism instead of
removing the problem" move this task kept making. It is also the more
accurate list: those three are the ones a bisect can land on.

**Superseded — the two questions as originally raised.**

1. **Should DD-002's brush correction be a successor rather than the
   annotation just approved?** The boundary this set established at T4 is
   *supersede when a reader implementing the original text would not
   obtain the shipped behaviour*. Step 4 says the **default** brush maps
   one-to-one; a reader who relies on it gets `Uniform` with 0.5
   alignment, i.e. scaled and centred glyphs, which is not what ships. By
   that test this is a supersede, and my "no option is re-chosen" ground
   was the wrong test — the same substitution T4's delta review caught on
   option I1. **Against:** what the successor would replace is one
   *explanatory clause*, not a choice; DD-005 exists because a clause
   changed the *behaviour* a reader would implement, and here the
   behaviour required — one texel to one device pixel — is stated
   correctly in the same step and in §The rounding contract. The record
   set gains a second successor either way. **No recommendation offered**:
   I have now argued this boundary wrongly twice in this task, and the
   argument I would make is the one that just failed.
2. **Should `architecture.md` §12.4's brush sentence be corrected now
   rather than at T12?** It is a normative spec stating a mechanism that
   is known-wrong *and* whose replacement is already fixed. Deferring an
   *unverified divergence* to Moment 2 is what T12 exists for; deferring a
   *determined* correction leaves a wrong instruction in the document
   external readers use. The atlas-origin sentence is **not** in this
   question — its conclusion is unaffected and its overstatement is
   empirical, so Moment 2 is the right home for it.

## T6 — Text-surface resolution + the re-rasterization walk

### Carry-over audit and responsibility re-audit (2026-07-30, before start gate)

Branch: `feat/m4-phase-1-t6`, created from `feat/m4-phase-1` at
`5e36ce5`.

The completed-task retrospectives and handoff leave five live T6-owned
obligations: consume `DipScale::surface_pixels` and remove its temporary
`dead_code` allowance; become the one writer of every node scale cache;
cover the incremental tree-mutation and IR conditional / `for` attach
paths F-32 found; preserve `sync_visuals` as the only Composition geometry
writer and `ButtonData.label_size` as the label-measurement source; and
produce GUI evidence that exercises scale != 1 rather than treating a 100%
frame as crispness evidence. The public subtree hit-test receiver
precondition, frame-drift mechanism, direct-Composition
`window_add_widget` path, and later-phase carry-forward rows remain stated
limits rather than T6 implementation work.

The plan's single fallible `apply_scale_recursive` hypothesis fails its own
side-effect test. If a surface recreation fails after earlier nodes have
already changed their caches, the next geometry pass projects one tree at
mixed scales. It also treats `set_root` and T7 as the whole reach even though
F-32 identified shipped incremental attach paths. The revised T6
responsibility is a prepare-then-commit primitive: rebuild stale brushes
without cache mutation, then write all caches in an infallible pass only
after preparation succeeds. It runs before initial layout and at every
layout entry, which covers inserted content at the point it first becomes
geometrically observable without teaching every mutator a second scale
rule. `window_add_widget` remains outside because it deliberately retains no
content root and runs no layout.

Two other planning choices are resolved before editing. Public
`TextRenderer::draw_text` remains a 96-DPI convenience wrapper; a
crate-private method takes raw `u32` DPI, so neither `DipScale` nor an
unstructured factor becomes public. The walk reads the existing fixed Text
extent and `ButtonData.label_size`; it does not call `measure`, add retained
measurement state, or write Composition geometry. The corresponding
changes are recorded in [plan.md](./plan.md) §T6 before the start gate is
selected.

### Start gate (recorded 2026-07-30, before production-code edits)

Review lane: **full independent review** — T6 changes the runtime rendering
path, the tree-wide cache commit boundary, and produces GUI-render evidence.
No trap is judged non-applicable after the responsibility re-audit.

| # | Applies | Reason and planned close artifact |
|---|---|---|
| 1 — semantic migration / call sites | **yes** | DD-002 audit row 7 changes and the internal scaled drawing entry has seven production call expressions (five existing paths plus the re-rasterization path's Text and Button-family arms). Close with the exact `rg` queries, every caller classified, and an explicit unchanged-row-10 check. |
| 2 — structural side effects | **yes** | The primitive changes text brushes and the cache that every geometry write reads. Enumerate brush replacement, cache commit, layout ordering, geometry non-effects, layout invalidation, child reach, and failure state. |
| 3 — parallel / derived data | **yes** | Each node cache is derived from `WindowState::scale`. Close by enumerating the authoritative scale mutator and all attach / layout paths, and show that cache commit occurs only inside the prepare-then-commit primitive after fallible work succeeds. |
| 4 — authored branch | **yes** | Reusing a Text node's fixed extent requires a size-shape branch, and rendering failures may gain a diagnostic branch. Add a direct test for each branch introduced; do not count an incidental GUI run. |
| 5 — carry-forward | **yes** | Preserve the single-writer rule, the no-geometry-write rule, and the scale-independent-measure premise. Record their evidence and re-trigger criteria, including a future scale-dependent `measure`. |
| 6 — deterministic failure | **yes** | Surface allocation, `BeginDraw`, `EndDraw`, brush creation, and brush installation are fallible across O(text nodes). A recurring failure is rooted and dispositioned; the all-or-no-cache-commit shape is verified rather than retried to green. |
| 7 — GUI positive control | **yes** | A 100% frame cannot exercise the D2D DPI or atlas-origin changes. Capture and inspect multiple 100% regression frames, a throwaway-PMv2 125% crispness pair, and an accepted-vs-default brush control with a deliberately non-proportional surface / Visual pair at integer and fractional device origins. |

The implementation approach is therefore constrained before editing: public
`draw_text` stays source-compatible; scaled drawing takes raw DPI inside the
crate; text extents are read rather than re-measured; no new retained
rendering state is added; no Composition geometry write leaves
`sync_visuals`; and cache writes occur only after the complete fallible
preparation succeeds.

### Implementation result and end gate (2026-07-30)

T6 implements the responsibility selected at the start gate. `TextRenderer`
now allocates each drawing surface with `DipScale::surface_pixels`, sets the
D2D context to the retained raw DPI, and converts `BeginDraw`'s physical atlas
offset back to DIP before drawing. `WidgetNode` creates every text surface
brush through one helper that sets `CompositionStretch::None` and both
alignment ratios to `0.0`. The public `draw_text` method remains the 96-DPI
entry; the runtime uses the crate-private `draw_text_at_dpi` entry.

The tree operation is the planned prepare-then-commit pair. The fallible pass
replaces stale Text and Button-family brushes from retained DIP state and does
not change a cache. The infallible pass then writes every node cache. `set_root`
runs it before detaching the previous root, and both layout entries run it
before building a layout tree, so a newly inserted child is normalized before
the first `sync_visuals` that can expose it.

#### Trap 1 — semantic call-site audit

Queries used:

```text
rg -n "draw_text\(" --glob "*.rs" .
rg -n "draw_text_at_dpi\(" wasamo-runtime/src
rg -n "CreateSurfaceBrushWithSurface" wasamo-runtime/src wasamo-runtime/tests
```

| Site | DPI source | Disposition |
|---|---|---|
| public `TextRenderer::draw_text` wrapper | `REFERENCE_DPI` | Existing Rust-native contract retained; the integration control is its only repository caller outside the wrapper itself. |
| Text constructor | identity | Construction precedes window ownership; normalized by the first attach/layout operation. |
| Button / ToggleButton constructor | identity | Same construction boundary. |
| Button label update | node cache | Reads DPI before the whole-node mutable borrow, then replaces the brush. |
| Text content update | node cache | Re-measures because content changed, then rasterizes at the already-attached node scale. |
| Text style update | node cache | Re-measures because style changed, then rasterizes at the already-attached node scale. |
| prepare pass, Text arm | target window scale | Reads the retained fixed extent; no measurement. |
| prepare pass, Button-family arm | target window scale | Reads `ButtonData.label_size`; no measurement. |

The seven internal call expressions are therefore all classified. The one
production `CreateSurfaceBrushWithSurface` expression is inside the accepted
mapping helper. Audit row 10 is unchanged: measurement still takes DIP and the
scale walk contains no `measure` call. `DipScale::surface_pixels` is now live,
so its two temporary `dead_code` allowances were removed.

#### Traps 2 and 3 — structural side effects and parallel data

| Effect / copy | Writer and ordering | End-gate result |
|---|---|---|
| drawing-surface pixel extent | `draw_text_at_dpi`, `ceil(dip × scale)`, minimum one pixel | Changed intentionally; Visual extent is not rounded. |
| D2D DPI and atlas origin | `draw_text_at_dpi`, after `BeginDraw` | Both sides of the physical/DIP boundary use the same `DipScale`. |
| surface-brush mapping | `create_text_surface_brush` | One production writer; `None` / `0.0` / `0.0`. |
| Text / Button brush | property-update path or prepare pass | A prepare failure may leave an already-prepared brush, but no geometry cache is partially committed; stale caches make the next pass retry. |
| node scale cache | `commit_scale_recursive` only | One assignment expression in the runtime, reached only after the complete fallible pass succeeds. |
| Composition geometry | `sync_visuals` only | The prepare and commit passes contain no `SetOffset` / `SetSize`; the repository search finds those writes only in `sync_visuals`. |
| retained DIP extent / layout invalidation | existing constructors and property updates only | The scale walk neither writes `SizeConstraint` / `label_size` nor calls `mark_layout_dirty_for`; row-10 scale-independent measurement is preserved. |

`WindowState::scale`, initialized from `GetDpiForWindow`, is the current
window authority; T7 will add its first mutation. Node constructors still
start at identity. Initial `set_root` explicitly applies the window value.
`append_child`, `insert_child`, `replace_child`, and the IR conditional / `for`
sites remain cache-neutral. The IR paths mark the owning window dirty and drain
through its layout entry. The direct Rust / ABI mutation APIs retain T3's
existing limit: they schedule no layout and wait for a later `WM_SIZE` or
size-affecting property write. In either case, the first layout that can assign
geometry applies the window root's scale recursively before `sync_visuals`.
The direct-Composition `window_add_widget` path retains no layout/content root
and remains the already-stated unsupported boundary.

#### Trap 4 — authored branches

`fixed_extent` has one accepted shape (`Fixed`, `Fixed`) and rejects either
non-fixed axis. `fixed_extent_accepts_only_two_fixed_axes` directly fires the
accepted branch and both rejection positions as pure Rust logic. The remaining
new failure exits are WinRT calls propagated with `?`; no diagnostic/reject
branch was added around them, and the OS surface is not mocked.

The mock-free `text_surface_mapping_integration` test reads live WinRT objects.
Its default-brush control observes `Uniform` / `0.5` / `0.5`; the production
Text brush observes `None` / `0.0` / `0.0`. A second test observes a
non-proportional ceil-source / exact-Visual pair and integer plus fractional
Visual origins. Both tests pass with `--test-threads=1`.

#### Trap 5 — carry-forward

The single cache writer, geometry-writer boundary, and scale-independent
measurement premise remain active. Re-run this audit if a new attach path can
reach geometry without `run_layout` / `run_layout_as_window_root`, if
measurement becomes scale-dependent, if a new text-bearing widget variant is
added, or if `window_add_widget` gains a retained content root. T7 consumes the
primitive; T8's stale-descendant control therefore requires the hidden test
seam recorded in the revised plan.

#### Trap 6 — deterministic failure and disposition

**F-39 — the 100% "unchanged" hypothesis was false.** Fresh parent and T6
captures differed materially in all six frames at 96 DPI (9,360–24,868 pixels,
maximum channel delta 220–249). This is not evidence that the D2D scale
boundary failed: DPI and atlas-origin conversion are identities at 96, while
whole-pixel `ceil` allocation and DD-M4-P1-006's mapping are still observable.
Here `96 DPI` is the runtime-effective path: because the final declaration had
not landed, DWM enlarged the complete frame on the 125% desktop. The reported
PNG measurements are therefore post-DWM physical pixels, although both sides
of each comparison pass through the same enlargement.
Removing only the three brush setters changed all six frames again
(9,327–25,197 pixels, maximum delta 217–234), and the live-object integration
test names the mechanism. The plan and end gate were corrected rather than
retrying toward the expected picture.

**F-40 — a comparison build reused an artifact directory across two source
trees.** After building the parent worktree into the shared target, a main-tree
workspace build reported the current runtime fresh. That makes a timestamped
DLL insufficient evidence, the same practical failure class as F-21. The
comparison worktree was removed; `cargo clean -p wasamo-runtime --release`
then forced the accepted source to compile before the release workspace build.
After the setter-removal mutation capture, the same clean/rebuild sequence was
run again. The six `t6-final` client interiors are byte-identical to the six
accepted `t6-100-after-b` interiors, proving the final DLL is not the named
setter-removal mutation artifact. That byte identity is not general
source-identity evidence: a render-neutral mutation could produce the same
frames and needs a structural/source artifact. Future cross-tree comparisons
must use separate target directories; mutation captures must finish with a
package clean and accepted rebuild, with the rebuild record—not the frame
alone—carrying source restoration.

#### Trap 7 — GUI evidence

[evidence/t6-analysis/README.md](./evidence/t6-analysis/README.md) records the
capture matrix, pixel counts, mutation, and assistant screenshot analysis. At
effective 120 DPI the same gallery is the positive control: the parent occupies
only `1 / 1.25` of the client and its 96-DPI glyph rasterization has grey edges
and uneven stems; its default brush adds a slight whole-surface shrink/centring,
not DWM bitmap stretching. T6 fills the client and shows narrower consistent
strokes and open counters. Three fresh captures on each side establish the
repeatability bounds. The 96-DPI default-brush mutation visibly centres and
scales the integer surface; the accepted brush holds the origin and unit
mapping. The branch-tip `t6-final/gallery-default.png` was inspected after the
forced clean build and shows the intended Gallery screen, not a blank or stale
host. Literal cross-monitor delivery remains T11's responsibility.

#### Verification

All commands ran on Windows from the T6 branch tip source:

- `cargo fmt --all -- --check` — green.
- `cargo build --release --workspace` after the forced package clean — green.
- `cargo build --workspace` — green.
- `cargo test --workspace -- --test-threads=1` — green.
- `cargo test -p wasamo-runtime --test text_surface_mapping_integration -- --test-threads=1` — 2 passed.
- `git diff --check` — green.

End-gate result: **passed**, subject to the required full independent review
after the T6 retrospective. No merge approval is implied.

### Independent review round 1 disposition (2026-07-30)

Result: **3 major, 3 minor**. The zero-major merge gate is not met.

| # | Severity | Finding | Disposition |
|---|---|---|---|
| R1 | major | T6's combined prepare+commit primitive does not yet supply DD-M4-P1-003 / T7's fixed order with both new-cache nested geometry and post-layout rasterization; its retry claim also has no authoritative target once `WindowState::scale` and the root cache diverge | **Confirmed; owner decision pending.** The plan already calls this a T6/T7 joint choice and reserves whether the cache commit moves into step 1 or the full walk moves. T6 cannot close while its public-to-crate primitive leaves that consumer unresolved. |
| R2 | major | The first 125% baseline changes both geometry cache and text surfaces, so it does not isolate the defining scaled-rasterization failure | **Confirmed and remediated.** Two fresh mutation captures hold geometry/cache at 120 DPI while forcing every text surface and D2D context to 96 DPI. The two runs are byte-identical; versus accepted `t6-after-b`, all six frames differ by 9,183–25,977 pixels with maximum channel delta 223–252. Both occupy the same client and retain the same rectangular geometry / 9×2 tile layout; only the text-surface path is mutated. |
| R3 | major | The new mock-free test binary has no recorded real `0x80070005` negative-path firing | **Confirmed; external evidence pending.** The helper is established, but the project rule requires this new test to be observed on a Compositor-unavailable environment rather than inheriting an earlier binary's observation. |
| R4 | minor | Direct tree-mutation APIs were incorrectly described as scheduling dirty layout | **Fixed.** Plan, log and retrospective now distinguish IR dirty/drain from direct APIs waiting for a later layout; T6 guarantees normalization before the first later geometry write, not immediate layout. |
| R5 | minor | The allocation checkbox was open and the public-wrapper caller statement predated the new integration control | **Fixed.** The checkbox is complete; the statement now distinguishes no production caller from the deliberate T6 test caller. |
| R6 | minor | The live test accepted any surface at least as large as the Visual rather than exact `ceil` | **Fixed.** It now asserts `SizeInt32.Width/Height == ceil(Visual.Size.X/Y)` before the non-proportional mapping controls. Targeted test remains 2/2 green. |

R2 evidence is
[evidence/t6-scaled-surface-identity-a/](./evidence/t6-scaled-surface-identity-a/)
and `-b/`. The temporary PMv2 declaration and identity-DPI renderer mutation
were removed, then `cargo clean -p wasamo-runtime --release` and
`cargo build --release --workspace` rebuilt the accepted source. R1 requires a
product-owner record decision; R3 requires an external environment whose
runtime actually returns `0x80070005`.

### Independent review round 2 intake and remediation plan (2026-07-30)

The owner supplied a second independent review derived against `97c24cf` and
independently checked against branch tip `e532c9f`. It reports **2 major and 11
minor at the reviewed base**. At the tip, its major 2 and minors 1–2 are the
already-landed R2/R4/R5/R6 remediation; its major 1 is the same unresolved R1,
with a concrete failure trace; and minors 3–11 are new or newly classified.
The zero-major gate therefore remains unmet.

The review's major-1 trace assumes one possible T7 caller order that has not
landed, but its root proposition is confirmed independently of that scenario:
production layout entries infer a target from a derived root copy, and the same
copy claims both geometry projection and successful text rasterization. A
fallible brush operation can therefore both block geometry and erase the only
retry marker. The chosen remediation follows the reviewer's recommendation:
production callers supply `WindowState::scale`; geometry and raster freshness
become separate facts; and surface failure no longer prevents layout. This
preserves DD-M4-P1-003's written order, so neither a successor nor a move of
the whole walk into step 1 is selected. T7 will defer the ordinary text refresh
inside its nested `WM_SIZE` and run it after `SetWindowPos` returns.

| Review item | Tip disposition before code remediation |
|---|---|
| Major 1 — inferred layout target and shared freshness cache | **Confirmed; remediation selected above.** This is first because it is the only unresolved major at the tip. |
| Major 2 — scaled evidence geometry confound | **Already fixed in `e532c9f`.** The scaled surface-identity mutation holds geometry/cache at 120 DPI. |
| Minor 1 — direct mutations do not dirty layout | **Already fixed in `e532c9f`.** |
| Minor 2 — open allocation checkbox | **Already fixed in `e532c9f`.** |
| Minor 3 — wildcard absorbs future text-bearing variants | **Confirmed; fix in this remediation.** Make the match exhaustive. |
| Minor 4 — `set_root` comment says the caller can discard a consumed `Box` | **Confirmed as a false comment and a wider ABI failure-contract question.** Correct the comment now; audit whether preserving a host handle on failure can be changed without redefining the experimental ABI ownership contract. |
| Minor 5 — text failure prevents geometry | **Confirmed; fix with the major-1 split.** Geometry runs and commits independently; raster failure stays retryable through its own marker. |
| Minor 6 — the 96-effective-DPI frames are DWM-scaled | **Confirmed; factual evidence correction required.** The pixels remain valid comparisons, but the description must distinguish process-effective DPI from final DWM frame scaling. |
| Minor 7 — parent `stretched-bitmap fringe` mechanism is wrong | **Confirmed; factual evidence correction required.** The old default brush slightly shrinks/centres; the dominant visible difference is lower glyph resolution. |
| Minor 8 — byte identity does not prove source identity | **Confirmed as a qualification to F-40.** It proves restoration from the tested mutation, not arbitrary source identity; render-neutral mutations need a structural/source artifact. |
| Minor 9 — public `draw_text` now returns a ceil-sized surface | **Confirmed; document the retained 96-DPI but changed storage-extent contract.** Callers supplying their own brush must set the one-to-one mapping if that is what they require. |
| Minor 10 — new layout-target/cache invariant absent from retrospective | **Confirmed; update item 10 after the code shape lands.** |
| Minor 11 — negative skip guard unobserved | **Confirmed; external evidence pending.** The fail-not-skip assertion closes the CI direction statically, so this review classifies it minor rather than round 1's major, but the AGENTS.md landing requirement still has to be discharged on a Compositor-unavailable environment. |

The two tip-only commits (`a26f213`, `e532c9f`) remain outside the supplied
review base and require a delta review together with this remediation before
the final zero-major verdict.

### R1 remediation result and repeated end-gate artifacts (2026-07-30)

Landed as `fad59e2` after the pre-code plan record `23e14c3`. `WidgetNode`
now holds two deliberately distinct derived facts: `scale` is the geometry /
hit-test projection cache; `raster_scale` is the DPI of the text brush actually
installed. Production window geometry receives `WindowState::scale`
explicitly, passes that one target through the complete `sync_visuals`
recursion, then commits every node geometry cache after successful sync. Text
refresh is a separate fallible recursion and advances a node's raster marker
only after `SetBrush` succeeds. The match is exhaustive over every current
`WidgetData` variant.

The three production boundaries compose those primitives as follows:

| Caller | Authoritative target | Geometry/cache | Text refresh |
|---|---|---|---|
| `window::set_root` | `state.scale` | refresh new tree before replacing the old root; initial geometry then uses the explicit target and commits it | pre-attach, so failure preserves the installed root |
| `wnd_proc::WM_SIZE` | the local copy of `state.scale` | `run_layout_as_window_root_at_scale` | separate call after geometry; T7 can suppress this call only for its nested `WM_SIZE` and run it at DD-003 step 4 |
| `emit::flush_layout` | `state.scale` | same explicit-target geometry entry | separate call after geometry, covering IR incremental attach and later layout after direct mutation |

Standalone Rust/test layout entries have no `WindowState`; they use their root
geometry cache as their target, run target-threaded geometry, commit it, then
refresh against the independent raster marker. Property writers rasterize at
the node geometry scale and update `raster_scale` only after brush installation.

**Structural side effects.** One `DipScale` marker is added per node; every
constructor initializes both markers to identity. No `SizeConstraint`, retained
text measurement, tree structure, registry/effect/signal state, reactive drain
accounting, or Composition geometry-writer site is added. `sync_visuals`
remains the only geometry writer; it now reads its explicit recursion target
rather than `self.scale`. The new T6 pre-attach failure is also checked at the C
ABI while the raw handle can still be restored, so that failure does not drop
the host's allocation or leave its handle dangling. The wider experimental
`set_root` ownership-on-error contract for pre-existing post-preparation WinRT
failures is not redefined by T6.

**Deterministic control.** The mock-free
`authoritative_geometry_scale_preserves_a_stale_raster_retry` test creates a
live Text surface at 96 DPI, runs geometry-only projection at 120 DPI, verifies
the surface remains the original size, then calls the standalone layout entry.
The geometry cache is already 120 DPI at that point, so the old shared-marker
implementation would skip. The corrected implementation consults the stale
raster marker and installs a surface whose exact integer size is the ceil of
the 120-DPI Visual. The T6 integration binary is now **3 passed** locally.

**GUI evidence.** A release workspace build preceded two fresh live six-frame
captures, `t6-r1-final-a/b`. The sets are byte-identical to each other and all
six client interiors are also byte-identical to the pre-remediation
`t6-final` set. Direct inspection confirmed a non-blank 9 × 2 gallery, the
click-created lightbox, and the `Counted 10 times` property-update frame with
all expected labels. The exact frame match is a success-path regression check;
the mock-free stale-raster control is the positive control for R1's
render-neutral state distinction.

Round-2 new-minor disposition after the remediation:

| Minor | Result |
|---|---|
| 3 wildcard match | **Fixed** — exhaustive current variants. |
| 4 consumed-Box comment / T6 preflight | **Fixed for the T6-added failure** — false comment removed; C ABI restores the raw handle if scale-aware preparation fails. Pre-existing later WinRT ownership semantics are unchanged. |
| 5 text failure blocks geometry | **Fixed** — geometry/cache and refresh are separate calls; raster failure cannot prevent the completed geometry pass. |
| 6 96-DPI evidence description | **Fixed** — evidence distinguishes runtime-effective 96 DPI from the post-DWM 125%-desktop frame. |
| 7 stretched-fringe mechanism | **Fixed** — lower raster resolution plus the default brush's slight shrink/centring replaces the incorrect DWM-stretch explanation. |
| 8 byte identity overclaim | **Fixed** — it excludes the named render-changing mutation; render-neutral changes require structural/source evidence. |
| 9 public `draw_text` storage contract | **Fixed** — rustdoc records 96-DPI inputs, ceil storage and caller brush-mapping responsibility. |
| 10 retrospective invariant | **Fixed** — item 10 records explicit authoritative target plus independent geometry/raster markers and the T7 consumer shape. |
| 11 negative skip firing | **Still pending external evidence.** Static fail-not-skip direction is present, but the project rule still requires one real Compositor-unavailable run before landing. |

R1 is code-remediated but not self-cleared: the branch still requires a delta
review over `a26f213..HEAD`, and the negative-path environment evidence remains
a non-review landing gate.

### Independent delta review intake and remediation start gate (2026-07-30)

The independent review of `97c24cf..d2084c7` reports **1 major and 2
minor**; the gate is not zero-major. The confirmed major is a T7 failure-path
planning hole rather than a missing T6 primitive: if `SetWindowPos` fails or
does not emit a nested `WM_SIZE`, no geometry pass consumes the new
`WindowState::scale`. The plan now requires an explicit step-3 fallback using
the current client rectangle before text refresh. The accepted success-path
order remains unchanged. The first minor is confirmed: T6's new ABI preflight
error branch restores raw ownership but no test fires the restoration. The
second minor is a confirmed wording error: geometry projection is fallible;
only the cache commit after it is infallible.

Before code remediation, the implementation-gate traps are selected as
follows:

| Trap | Applies | Reason / planned artifact |
|---|---|---|
| 1 — semantic migration | no | No enum, schema, IR, or variant changes. |
| 2 — structural side effects | **yes** | Enumerate the failure-state relationship among window scale, Visual geometry, geometry cache, raster marker, installed root, and incoming ABI allocation. |
| 3 — parallel data | **yes** | Correct the T7 fallback so authoritative scale and derived geometry cannot silently claim convergence; retain separate geometry and raster markers. |
| 4 — authored branch | **yes** | Extract the raw-ownership preflight transaction into pure generic logic and directly fire its error branch with a drop-counted allocation; T7's future fallback also gains an explicit direct-test requirement in the plan. |
| 5 — carry-forward | **yes** | Record the no-nested-geometry fallback and its trigger in T7's task list and the T6 retrospective. |
| 6 — deterministic failure | no | No recurring runtime/test failure initiated this remediation; the injected pure error is a branch control, not a rerolled failure. |
| 7 — GUI evidence | no, for this delta | The code delta is ownership-only and cannot alter rendering; the already-reviewed T6 frame sets remain the GUI artifact. |

Review lane remains **full independent review** because the containing T6
change is runtime-structural and GUI-render gated; trap #4 composes with that
review. The selected ABI shape centralises `Box::from_raw` / `Box::into_raw`
in a generic preflight transaction. Its test must prove that rejection returns
the identical live pointer, does not drop it, and permits exactly one later
destruction. `window::set_root` remains unreachable until preflight succeeds,
so the installed root is structurally untouched on this branch.

### Independent delta review remediation result (2026-07-30)

The three findings were remediated in `0f05bea` (pre-code plan / gate record)
and `1220b10` (ownership branch control and wording correction).

| Finding | Result |
|---|---|
| Major — missing geometry when `SetWindowPos` produces no successful nested pass | **Fixed in the T7 contract.** Completion is recorded only after successful nested geometry. Failure, a no-size-change success, or a failed nested projection enters a current-client-rectangle geometry fallback at explicit `WindowState::scale`; text refresh follows only successful geometry. T7 must directly fire this branch. |
| Minor — ABI ownership restoration untested | **Fixed.** `preflight_boxed_handle_restores_same_live_allocation_on_error` fires the extracted pure transaction's rejection branch. It observes the identical pointer, a live mutation, zero drops at rejection, and exactly one later caller destruction. The installed root is untouched because `window::set_root` is after the successful transaction arm. |
| Minor — geometry/cache operation called infallible | **Fixed.** Rustdoc now says fallible geometry projection followed by an infallible cache commit, independent from the fallible raster pass. |

**Close-gate artifacts for this delta.** The authoritative window scale is
unchanged. T7's derived geometry cache advances only after the entire fallible
Visual projection succeeds. Because `sync_visuals` performs WinRT writes
incrementally, a failed projection can leave a Visual prefix updated while all
node caches remain stale; a missing or failed nested pass therefore triggers a
whole-tree fallback, and a failed fallback leaves the possible partial Visual
state plus stale caches / raster markers logged rather than claiming atomicity.
Text refresh runs only after one whole-tree geometry pass succeeds. The
T6 ABI preflight mutates only the incoming text brushes/markers before attach;
on rejection it neither calls `window::set_root` nor drops the incoming
allocation. Its generic transaction is the only raw restore site, and its
direct branch test is the trap-#4 artifact. The fallback's re-trigger is any
`SetWindowPos` return without a recorded successful nested geometry pass; this
is carried in T7's task list and T6's retrospective.

Trap #6 became applicable during implementation: the test's first compile
deterministically failed because `Result::expect_err`
requires the `Ok` type (`Box<DropProbe>`) to implement `Debug`. This was not an
ownership failure. Replacing it with an explicit `match` removed the irrelevant
bound; the identical targeted command then passed. Final code-tip validation:

- `cargo test -p wasamo-runtime --lib -- --test-threads=1` — 464 passed;
- `cargo test -p wasamo-runtime --test text_surface_mapping_integration -- --nocapture --test-threads=1` — 3 passed;
- `cargo build --release --workspace` — green with existing warnings;
- `cargo test --workspace -- --test-threads=1` — green, 956 tests;
- `cargo fmt --all -- --check`, `git diff --check` — green.

The branch requires another independent delta review at this point in the
history. The separate real Compositor-unavailable skip firing remains the
external landing blocker.

### Final independent review disposition (2026-07-30)

The independent review of `97c24cf..eb3021e` returned **zero major and 3
minor**. The prior missing-step-3 major, ABI ownership branch control, and
fallibility wording were confirmed closed. The three final minors are
documentation corrections and are fixed after the reviewed tip:

1. the failure-state artifact now distinguishes incremental Visual writes from
   the all-traversal-success geometry-cache commit;
2. T7's fallback now explicitly converts the physical current client extent to
   DIP through `state.scale.pair_to_dip(...)` before passing both the DIP extent
   and same authoritative target to geometry-only layout; and
3. the evidence pointer names `t6-r1-final-a/b`, with `t6-final` retained as its
   byte-identical predecessor.

No source code or captured PNG changes in this disposition. The final narrow
review of `eb3021e..e3e878d` returned **zero major / zero minor** and confirmed
all three factual corrections without a new inconsistency. The review gate is
closed. The real Compositor-unavailable guard firing remains the sole external
landing blocker.

### External Compositor-unavailable evidence and T6 close (2026-07-30)

The owner ran the exact new test binary from the T6 branch in a Windows
PowerShell 7.6.4 session where runtime initialisation reached the established
Compositor-unavailable classification:

`cargo test -p wasamo-runtime --test text_surface_mapping_integration -- --nocapture --test-threads=1`

All three tests fired their named negative path rather than their Compositor
body:

| Test | Observed guard output |
|---|---|
| `authoritative_geometry_scale_preserves_a_stale_raster_retry` | `skipping geometry/raster scale separation: runtime compositor unavailable` |
| `ceil_surface_mapping_is_observed_at_integer_and_fractional_visual_origins` | `skipping text surface source/destination mapping: runtime compositor unavailable` |
| `text_surface_brush_overrides_the_scaled_centered_default` | `skipping text surface brush mapping: runtime compositor unavailable` |

The binary result was 3 passed, 0 failed, 0 ignored, 0 measured, and 0 filtered
out. This is the required actual firing of the substring-classified local skip
branch; the same helper statically asserts that this branch may not skip when
`GITHUB_ACTIONS` is set. Together with the local live-Compositor 3/3 run, the
full zero-major review, and the zero-major / zero-minor correction review, this
closes T6's final landing blocker. T6 is **done**. No merge or push is implied;
merging to `feat/m4-phase-1` still requires explicit owner approval.

## T7 — `WM_DPICHANGED` propagation

### Carry-over audit and responsibility re-audit (2026-07-30, before start gate)

Branch: `feat/m4-phase-1-t7`, created from `feat/m4-phase-1` at `1ff4cb1`
(the T6 merge commit).

The completed retrospectives and the handoff leave T7 four live obligations
and nothing else. It is the first mutator of `WindowState::scale`, so it is
where the derived node caches can first be left behind. It must consume T6's
split primitives in DD-M4-P1-003's written order — geometry through
`run_layout_as_window_root_at_scale` at an explicit target, raster through
`refresh_text_surfaces_recursive` against the independent marker — without
moving a step. It must not reuse `window::realize_dip_window_size` or its
`SWP_NOMOVE` flag set (handoff row for T4). And it owns DD-003's 13-row
structural side-effect enumeration as its close artifact, reading row 10's
clip sites from the source rather than from the ADR's wording, which names
Box where ZStack is (T1 F-2, re-verified at T3).

Everything else the audit surfaced is already owned elsewhere and is not T7
work: the PMv2 declaration is T9, synthetic `s != 1` assertions and the scale
accessor seam are T8, the literal monitor crossing is T11, `window_add_widget`
remains a stated content-boundary limit, and the frame-baseline and
target-isolation rules bind T10 / T12.

The task list itself survives the audit; five decisions it did not name are
added to [plan.md](./plan.md) §T7 before the gate is selected. Four of them
are only visible from the arm the handler sits beside, which is the reason the
re-audit read `wnd_proc` end to end rather than reading the task list: the
first `wnd_proc` re-entrancy with a live `GWLP_USERDATA` makes the arm's
*placement* a soundness decision; the nested refresh suppression is a
correctness property rather than step-order fidelity, because the landed arm
discards the geometry `Result` and refreshes unconditionally; `lParam` is a
raw `RECT*` from a message parameter and null is a reachable input; and the
handler synthesises no `resize_fn` call, which incidentally makes that slot
the only public observation of whether a nested `WM_SIZE` ran. The fifth is
that the step-3 verdict is pure logic whose failure states the OS cannot be
asked to produce.

### Start gate (recorded 2026-07-30, before production-code edits)

Review lane: **full independent review**. T7 changes the window procedure,
introduces re-entrancy through it, and is the first writer of the
authoritative scale — runtime-structural on all three counts. Trap #4
composes with that review rather than replacing it.

| # | Applies | Reason and planned close artifact |
|---|---|---|
| 1 — semantic migration / call sites | **no** | No enum, schema, IR or field type changes, and no existing traversal gains a case. The new step-3 verdict type is introduced with every consumer in the same commit and is matched exhaustively. The *caller* question that does exist — which callers of the geometry and raster primitives change discipline — is not a migration and is closed under traps 2/3 as an enumeration of all four call sites. |
| 2 — structural side effects | **yes** | The phase's primary side-effect surface. Close with DD-M4-P1-003's 13 rows, each stated `updated` or `verified unchanged` against the source, rows 9–13 verified rather than assumed. |
| 3 — parallel / derived data | **yes** | Added at the re-audit. `WindowState::scale` is authoritative and T7 is its first mutator; the per-node geometry cache and per-node `raster_scale` are both derived from it. Close by enumerating every consumer of the value step 1 writes and showing that neither derived copy can advance without a whole-tree projection having succeeded. |
| 4 — authored branch | **yes** | Three authored branches: the no-nested-geometry fallback, the null suggested rectangle, and the denial of step 4 after both projections fail. Each gets a test that fires it directly; the fallback's is a mock-free integration test, and the pure step-3 verdict is unit-tested over all three states so the arm no OS input can reach is still fired. |
| 5 — carry-forward | **yes** | The step 1 / step 2 order, the nested-refresh suppression, the fallback trigger, and the arm-placement soundness argument are all invariants a later task can trip — T9 makes the ordering defect producible for the first time, T8 synthesises the message, M4-Phase 2 touches this procedure. Record each with evidence and a re-trigger criterion. |
| 6 — deterministic failure | **yes** | Added at the re-audit. The change lives in a message loop behind a raw pointer, where the tempting dispositions are "it passed on retry" and "the nested message must have run". Any recurring failure is rooted rather than re-rolled, and *observed nested geometry* rather than *entry into `WM_SIZE`* is what the handler records. |
| 7 — GUI positive control | **no** | The handler's intended per-monitor path needs T9's declaration before the OS will drive it for a real monitor change. A pre-T9 capture at 100% therefore cannot distinguish that path's presence from its absence. The phase assigns synthetic scale-driven assertions to T8 and the literal monitor crossing plus its human-visible smoke to T11. No frame is captured here, and none is claimed. |

The approach is therefore constrained before editing: the arm sits above the
`&mut *state_ptr` borrow and holds no outer reference across `SetWindowPos`;
step 1 writes the scale and opens the nested-pass observation in one
primitive, so no path installs the marker without having committed the scale;
the nested `WM_SIZE` reports whether its projection *succeeded* and refreshes
no text; and step 4 runs only behind the extracted verdict.

### Implementation result and end gate (2026-07-30)

Landed as one code commit, `e63586e`. The handler and its four branch tests are
one commit rather than two because three of the branches the handler introduces
are authored failure paths, and a commit carrying them without the tests that
fire them is precisely the state trap #4 exists to refuse.

`wnd_proc` gains a `WM_DPICHANGED` arm **above** the `&mut *state_ptr` block.
`handle_dpi_changed` commits the scale and installs the nested-pass marker in
one primitive (`begin_scale_change`), applies the OS-suggested physical
rectangle (`apply_suggested_rectangle`), reads back what the re-entrant
`WM_SIZE` reported, falls back to `project_current_client_extent` when no
projection succeeded, and refreshes text only behind
`GeometryProgress::whole_tree_projected`. The `WM_SIZE` arm gains two
conditionals on the marker: it suppresses its ordinary text refresh and it
reports its projection's outcome.

#### Traps 2 and 3 — DD-M4-P1-003's 13 rows, closed against the source

The claim under test is "a scale change drags along exactly these, and nothing
else". Rows marked *verified unchanged* were checked by search, not by memory.

| Row | Effect | Result |
|---|---|---|
| 1 | `WindowState`'s cached scale | **updated**, first — `begin_scale_change` in [`window.rs`](../../../../wasamo-runtime/src/window.rs), the only mutator in the runtime beside the creation-time seed. |
| 2 | The window rectangle | **updated** from the OS suggestion via `SetWindowPos`, with `SWP_NOZORDER` + `SWP_NOACTIVATE` and **not** `SWP_NOMOVE`. Asserted by test 1, which moves as well as resizes. |
| 3 | The client extent | **updated** — arrives through the nested `WM_SIZE` and is divided at the T5 seam; when no `WM_SIZE` arrives, read from `GetClientRect` and divided by the same committed scale in the fallback. |
| 4 | Layout | **re-run** over the new DIP client extent, by the nested pass or the fallback. Per the 2026-07-29 DD-003 annotation, "identical results" holds of a *controlled* client extent; T8 preserves one, T11 does not. |
| 5 | Every widget Visual's offset and size | **updated** by `sync_visuals` at the explicit target. No handler-specific geometry code. |
| 6 | The ScrollView intermediate Visual | **updated**, same pass. |
| 7 | The Button label Visual | **updated**, same pass, with **no handler-specific code** — the assertion DD-003 row 7 says it is making. Verified structurally: `sync_visuals` is the only `SetOffset` / `SetSize` site in the runtime (T3's invariant, re-searched here), and it reaches the label through the Button / ToggleButton arm. Had T3 not moved that write, this row would have needed handler code and would have been the phase's silent bug. |
| 8 | Every text surface and its brush | **updated** by `refresh_text_surfaces_recursive` at step 4 — and **only behind a succeeded projection**, which is the one place this row's wording needed strengthening rather than implementing. |
| 9 | The root's `SetRelativeSizeAdjustment(1, 1)` | **verified unchanged.** `rg SetRelativeSizeAdjustment wasamo-runtime/src` returns exactly one site, inside `create` in [`window.rs`](../../../../wasamo-runtime/src/window.rs). It relates two physical quantities, so it is scale-independent, and no code path re-writes it. |
| 10 | `InsetClip` insets | **verified unchanged**, and **the ADR's site list is wrong here.** `rg CreateInsetClip` returns the constructors `WidgetNode::scroll_view`, `WidgetNode::grid` and `WidgetNode::zstack` — **not `WidgetNode::box_`**, which installs no clip. Independently, a search for every `Set*Inset` setter finds **no site in the repository**, so every inset is the constructor default of zero and zero is scale-invariant. The row's conclusion stands; the widgets it names do not. (T1 F-2 established this against DD-002 row 12 and dispositioned the correction only to T5; T3 F-18 predicted that a T7 reading the ADR wording would assert a site that does not exist.) |
| 11 | Signal registry, effect graph, binding state, widget pointers | **verified unchanged.** The diff introduces no `registry`, `reactive`, `emit::`, or `mark_layout_dirty_for` token — searched over the whole added diff, not over the functions the author remembered writing. The two matches are the words "reactive drain" inside a doc comment saying it must not be entered. |
| 12 | `MUTATION_CAP` / drain accounting | **verified unchanged**, by the same search. The handler enqueues nothing and never marks a window dirty, so `emit::flush_layout` is not reached and the drain is not entered. |
| 13 | Hover and press state | **verified unchanged.** No `mouse_down`, `update_hover` or `clear_hover` token in the diff, and no pointer message is synthesised. The pointer may end up over a different widget after the resize; the next real `WM_MOUSEMOVE` corrects it. |

**Trap #3, the derived copies.** `WindowState::scale` is authoritative and T7 is
its first mutator, so this is the first task where a derived copy can be left
behind. Every access in the runtime, enumerated by searching `.scale` across
`window.rs`, `emit.rs` and `abi.rs`:

| Site | Kind | Consequence of the change |
|---|---|---|
| `create`'s struct literal | seed | Unchanged. |
| `begin_scale_change` | **write** | The only mutation. Installs the marker in the same function. |
| `set_root` (pre-attach refresh, `pair_to_dip`, geometry target) | read x3 | Unchanged; runs before any change can occur. |
| `wnd_proc`'s single `let scale = state.scale` | read | Serves the `WM_SIZE` arm and all three pointer arms. Read *per entry*, so the nested pass sees the committed value. |
| `emit::flush_layout` (`pair_to_dip`, geometry target) | read x2 | Unchanged; a later drain projects at whatever the current scale is. |
| `abi::wasamo_window_set_root`'s T6 preflight | read | Unchanged. |
| `project_current_client_extent`, `refresh_text_at_new_scale` | read x2 | New; both read the committed value, and the fallback uses the *same* value as divisor and as projection target rather than two that happen to agree. |

The two derived copies — each node's `scale` (geometry) and `raster_scale`
(last-rasterized DPI) — advance only through `commit_scale_recursive` and
`refresh_text_surfaces_recursive` respectively, and T7 reaches neither except
through `run_layout_as_window_root_at_scale` and the step-4 call. So the
authoritative value cannot move without either every geometry cache following it
or the divergence being logged and left visible.

#### Trap 4 — authored branches, each fired directly

| Branch | Test that fires it |
|---|---|
| No successful nested projection, so the whole-tree fallback runs | `an_unchanged_suggested_rectangle_projects_through_the_fallback` — a suggested rectangle equal to the current one makes `SetWindowPos` succeed while dispatching no `WM_SIZE`, so `resize_fn` is not called and only the fallback can have produced target-scale geometry. |
| Null suggested rectangle: skip step 2, log, fall back | `a_null_suggested_rectangle_survives_and_still_projects` |
| No projection succeeded: deny step 4, log, leave everything stale | `two_failed_projections_leave_the_text_stale_without_diverging` |
| `GeometryProgress`'s three states | `window::tests::only_a_succeeded_projection_permits_step_four` and `neither_unsuccessful_state_is_progress` — pure logic, because a *failed* projection needs a layout error and a *succeeded* one a live Compositor. |
| Failed step-2 rectangle application (diagnostic only) | `a_failed_rectangle_application_reports_the_rectangle_and_the_consequence` |
| Failed client-rectangle read: verdict `Failed` plus diagnostic | `a_failed_client_rect_read_yields_no_projection_and_says_so`, with `a_successful_client_rect_read_yields_the_physical_extent` on the other arm |
| Failed step-4 re-rasterization (diagnostic only) | `a_failed_text_refresh_reports_that_geometry_already_converged` |

**The last three rows were added after the independent review, which was
right that they were missing.** The first end-gate table listed only the three
branches with observable behaviour and called them "each fired directly", while
three authored *diagnostic* arms sat untested — the exact shape trap #4 names.
The disposition took three steps.

**First, measurement (F-43): degenerate extents do not provoke failure.** A throwaway
probe called `SetWindowPos` on a live window with negative extents, an inverted
rectangle, zero, `i32::MIN` and `i32::MAX`; **every one returned `Ok`**. That
measurement says nothing universal about other failure conditions: the API
contracts report failure generically, `SetWindowPos` also documents cross-session
failure, and an invalid handle can make a direct mock-free probe fail even though
it is not a state this window procedure should manufacture. The refresh failure
remains a WinRT surface or brush failure. The implementation therefore does not
claim that the three failures are impossible or OS-unprovokable.

**Second, deleting them was rejected.** DD-M4-P1-003 §Failure handling explicitly
requires the failed `SetWindowPos` and failed re-rasterization consequences to be
logged and survived. `GetClientRect` is the implementation's fallback read, not
one of the two calls the ADR names; reporting its failure follows the same
resilient posture but is not attributed to an explicit three-item mandate. T4's
`realize_dip_window_size` also carries the same `SetWindowPos` diagnostic.

**Third, the first extraction was rejected by delta review and replaced.** A
`windows::core::Error` is not pure test data on Windows: constructing it calls
`RoOriginateErrorW`, and formatting it consults Windows error information. The
OS-bound call sites now render that value to an owned string immediately. Pure
functions — `finish_rectangle_application`, `client_extent_or_failure`, and
`finish_text_refresh` — receive only that string summary plus primitive local
data, own the failure branch, and dispatch the completed diagnostic to a supplied
sink. Their unit tests capture the sink in `Vec<String>` and assert one dispatch,
the OS summary, and the operational consequence. No Win32/WinRT type or call is
present on the tested path; production supplies `emit_runtime_diagnostic`.

The extraction also pulled the fallback's only arithmetic into the open, and
**that turned out to be uncovered**: mutation M6 swapped the two axes of the
client extent and **all four integration tests stayed green** while
`a_successful_client_rect_read_yields_the_physical_extent` failed. So the
extraction closed a real coverage gap rather than only satisfying the gate.

The failing tree is `VStack { Text, HStack { Box } }`, reached through the `.ui`
path because `WidgetNode::box_` is `pub(crate)`. It fails deterministically
rather than by injection: `measure_vstack` passes an infinite child height and
`measure_hstack` an infinite child width, so the childless `Box` is measured
against unbounded space on both axes and returns `LayoutError::BoxNoExtent`
(DD-M3-P2-005).

**The green suite was not taken as evidence.** Five mutations, each built and
run:

| # | Mutation | Result |
|---|---|---|
| M1 | Steps 1 and 2 inverted (`begin_scale_change` after `SetWindowPos`) | **4/4 still pass.** Predicted, and the measurement is the point — see F-41. |
| M2 | Nested-refresh suppression removed | `two_failed_projections...` **fails**; the other three pass. |
| M3 | Fallback removed | the two fallback tests **fail**; the nested-path and both-fail tests pass. |
| M4 | Step-4 permission gate removed | `two_failed_projections...` **fails**. A distinct mechanism from M2, caught independently. |
| M5 | `SWP_NOMOVE` inherited from the creation-time correction | `a_size_changing_suggested_rectangle...` **fails**. |
| M6 | The fallback's client-extent axes swapped (added at review remediation) | `a_successful_client_rect_read_yields_the_physical_extent` **fails**; **all four integration tests stay green**. |

M5 is worth stating plainly: [handoff.md](./handoff.md) predicted that inheriting
that flag would pin the window on every monitor crossing "while every test stays
green". It would have, with a suggested rectangle that only changed the size.
Moving the rectangle as well costs nothing and converts the hazard from a comment
into a failing test.

#### Trap 6 — what the measurements refused to confirm

**F-41 — the ordering defect is not observable in the final state, measured
(M1).** The task's own bullet asked for the step 1 / step 2 order to be encoded
structurally or shown by a falsifiable probe, and warned that "the enumeration
says the order is right" is the outcome to refuse. M1 is that probe, and it
falsifies the *stronger* reading of the design claim while confirming the weaker
one. Inverting the two steps leaves all four tests green, because the nested
`WM_SIZE` then finds no marker, is neither suppressed nor reported, and the
handler's fallback re-projects the whole tree at the committed scale — so the
**final** state is correct either way. What the inversion demonstrably still
produces is a geometry write at the stale factor and a text refresh at the stale
DPI, both overwritten by the fallback. **So the design does not make the ordering
defect impossible; it makes it transient instead of persistent.** That is a
weaker claim than "unconstructible" and it is the one the code supports.

**And "transient" is as far as the measurement reaches.** DD-M4-P1-003 predicts
that a stale-scale nested pass leaves the window "visibly wrong for one frame at
best", and it is tempting to restate that as what M1 showed. It is not: M1
established the wrong *intermediate state*, not a presented frame. The whole
handler runs inside a single message, and Composition commits on the dispatcher
tick, so the compositor may never see the intermediate projection at all.
Establishing whether a frame is presented would need a capture across the change,
which is T11's instrument and not available before T9 anyway.

**F-42 — a `.ui` attribute typo produces an empty widget with no error, and it
made a first-draft assertion vacuous.** The first version of these tests wrote
`Text { content: "..." }`. The DSL attribute is `text:`; `check::check` reported
**no error** and `has_errors()` was false, so the tests ran against `Text` nodes
whose content was the empty string and whose measured width was `0.0` — under
which `assert_close(after, before * 1.25)` compares `0.0` with `0.0` and passes.
Three of the four tests were caught only by the *surface pixel* assertion, whose
expected `ceil(0.0) == 0` disagreed with `surface_pixels`' one-pixel floor. The
defect is pre-existing `wasamoc` lenience, not T7's to fix, and it is not filed
as a T7 finding beyond this record — but the lesson generalises past this task:
**a `.ui`-driven test can be green and empty**, and an assertion of the form
"the value scaled by k" is satisfied by zero. Both `.ui` fixtures here now
assert non-degenerately.

What caught the vacuity was the *surface pixel* assertion, whose expected
`ceil(0.0)` disagreed with `surface_pixels`' one-pixel floor — i.e. a second,
differently-shaped fact about the witness. **That is three of the four tests, not
all four**: `two_failed_projections_leave_the_text_stale_without_diverging`
deliberately discards the Visual size and compares only the surface, because a
tree that never lays out has no projected geometry to read. Its discriminating
power does not depend on the witness being non-degenerate — a degenerate witness
still moves from a 19-pixel to a 24-pixel surface when the step-4 gate is
removed — so the fixture is sound and only this paragraph's earlier phrasing
("each test reads two facts") was wrong. Carried forward as: **an assertion that
scales a measured quantity needs a second fact of a different shape beside it,
or a witness proven non-degenerate.**

No failure was re-rolled and no test was retried to green. M2 through M5 were
expected to fail and did; M1 was expected to pass and did.

#### Trap 5 — carry-forward

Recorded in the T7 retrospective's item 10 and carried to
[handoff.md](./handoff.md) at phase close:

- **The handler holds no `&mut WindowState` across `SetWindowPos`.**
  *Re-trigger:* any task adding work to the `WM_DPICHANGED` arm, or moving it
  inside `wnd_proc`'s null-checked block. M4-Phase 2's event model touches this
  procedure next.
- **Step 4 is permitted only by a succeeded whole-tree projection**, and the
  nested `WM_SIZE` suppresses its own refresh to make that gate meaningful.
  *Re-trigger:* any new caller of `refresh_text_surfaces_recursive`, or any
  change making `measure` scale-dependent (which would also turn step 4's
  position from a free choice into a correctness constraint).
- **`resize_fn` fires from the `WM_SIZE` arm and nowhere else**, which is what
  makes it the discriminator between the nested path and the fallback.
  *Re-trigger:* the first host or ABI function to install a resize callback, or
  any task that synthesises one from the change path.
- **DD-M4-P1-003 row 10 names Box where the source has ZStack.** Closed here
  against the source for the third time (T1, T3, T7). *Re-trigger:* any task
  building an enumeration from the ADR's row-10 wording.

#### Verification

All commands on Windows, against the post-commit branch state:

- `cargo fmt --all -- --check` — green.
- `git diff --check` — green.
- `cargo test -p wasamo-runtime --lib -- --test-threads=1` — 466 passed (464 plus the two `GeometryProgress` tests).
- `cargo test -p wasamo-runtime --test dpi_change_propagation_integration -- --nocapture --test-threads=1` — 4 passed.
- `cargo clean`, then `cargo build -p wasamo-runtime`, `cargo build --release --workspace`, `cargo build --workspace`, `cargo test --workspace -- --test-threads=1` — recorded in the T7 retrospective item 3.

No GUI capture: trap #7 is non-applicable for the reason recorded at the start
gate, and no frame is claimed.

**End-gate result: passed**, subject to the required full independent review
after the T7 retrospective, and subject to one landing blocker that is not a
review finding: **the new test binary's Compositor-unavailable skip path has not
been observed firing.** The helper is the already-verified shared one, but T6's
round-1 R3 classified per-binary observation as the requirement, so this binary
needs one owner run on an environment where `wasamo_init` returns `0x80070005`.
No merge approval is implied. *(That blocker was closed the same day — see
§External Compositor-unavailable evidence below.)*

### External Compositor-unavailable evidence (2026-07-30)

The owner ran the T7 test binary in a Windows session where runtime
initialisation reached the established Compositor-unavailable classification:

`cargo test -p wasamo-runtime --test dpi_change_propagation_integration -- --nocapture --test-threads=1`

All four tests entered their named negative path rather than their Compositor
body:

| Test | Observed guard output |
|---|---|
| `a_null_suggested_rectangle_survives_and_still_projects` | `skipping DPI change with a null rectangle: runtime compositor unavailable` |
| `a_size_changing_suggested_rectangle_projects_through_the_nested_wm_size` | `skipping DPI change with a size-changing rectangle: runtime compositor unavailable` |
| `an_unchanged_suggested_rectangle_projects_through_the_fallback` | `skipping DPI change with no nested WM_SIZE: runtime compositor unavailable` |
| `two_failed_projections_leave_the_text_stale_without_diverging` | `skipping DPI change with unresolvable layout: runtime compositor unavailable` |

Result: 4 passed, 0 failed, 0 ignored, 0 measured, 0 filtered out.

**It was the same binary, not a rebuild.** Cargo reported
`Finished ... in 0.07s` and ran
`dpi_change_propagation_integration-108571e2c139caae.exe` — the artifact hash
from the local live-Compositor run. So the difference between the two runs is
the session's Compositor capability and nothing else, which is what makes this
a control on the guard rather than on a second build. The environment split is
the one [verification-environments.md](../../../../docs/notes/verification-environments.md)
records: the same machine yields `0x80070005` from `wasamo_init` in a session
without a usable desktop.

This is the required actual firing of the substring-classified local skip
branch, per [AGENTS.md §Testing rules](../../../../AGENTS.md) — a guard
verified only on the happy path is not verified — and per T6 round-1 R3's
finding that the observation is owed **per binary** rather than inherited from
the shared helper. The same helper statically asserts that this branch may not
skip when `GITHUB_ACTIONS` is set, so the CI direction is closed by
construction.

Together with the local live-Compositor 4/4 run, T7's sole landing blocker is
closed. **The full independent review remains outstanding**, and merge to
`feat/m4-phase-1` is still a separate owner-approval gate.

### Independent review disposition (2026-07-30)

The independent review of `feat/m4-phase-1..9bd17cb` returned **1 major and 4
minor**. The zero-major gate was not met. All five were confirmed and none was
argued down. The first dispositions below were found incomplete by delta review;
their second remediation is recorded after the start gate.

| # | Severity | Finding | Disposition |
|---|---|---|---|
| R1 | major | Three authored diagnostic branches ship untested — the failed `SetWindowPos`, the failed `GetClientRect`, and the failed step-4 refresh — while the end-gate table listed only the three behaviourally-observable branches and called them "each fired directly" | **Confirmed. First remediation incomplete; second remediation below.** The first extraction left WinRT FFI in unit tests and did not fire sink dispatch. The replacement moves the error to an owned summary at the OS boundary and has FFI-free pure functions own branch + dispatch. |
| R2 | minor | The M1 weakening still overclaims: "one visibly wrong frame" was not measured, only a wrong intermediate state; and "an undeclared process is never sent this message" is stronger than Microsoft documents | **Confirmed. First remediation incomplete; second remediation below.** The corrected implementation comment was accurate, but the start-gate row and retrospective still carried the stronger claims. |
| R3 | minor | "On an ordinary resize every `raster_scale` already equals the target" misses a subtree attached after a scale change, whose marker is still the constructor identity | **Confirmed. First remediation incomplete; second remediation below.** The plan named the subtree but incorrectly inferred harmless convergence, while the production comment retained the original claim. Both now describe this as pre-existing ordinary-resize behaviour outside T7's convergence proof. |
| R4 | minor | End-gate source line references drifted after the comment-only commit | **Confirmed. First remediation incomplete; second remediation below.** The drifted instances moved to function names, but row 10 retained three line numbers while the disposition claimed the whole class was fixed. All enumeration rows now use symbols. |
| R5 | minor | F-42's "each test reads two facts about the witness" is false of the failed-projection test, which reads only the surface | **Confirmed. First remediation incomplete; second remediation below.** The log was narrowed to three of four, but the test helper comment retained the universal claim. It now names the three-tests/one-test split and the fourth test's reason. |

**The reviewer verified rather than read** on the load-bearing claims —
re-running M1 through M5, re-deriving the `BoxNoExtent` path, checking the
aliasing by inspection of borrow lifetimes across `SetWindowPos`, and confirming
the T8 / T9 / T11 boundaries. Four findings include an over-strong record, but
R1 is not record-only: the first remediation also violated the unit-test FFI
boundary and left sink dispatch unfired. The corrected classification and the
recurrence boundary are recorded in the T7 retrospective.

Post-remediation verification, on the branch tip:

- `cargo fmt --all -- --check`, `git diff --check` — green.
- `cargo test -p wasamo-runtime --lib -- --test-threads=1` — **470 passed** (466 plus the four new diagnostic and extent tests).
- `cargo test -p wasamo-runtime --test dpi_change_propagation_integration -- --test-threads=1` — 4 passed.
- `cargo test --workspace -- --test-threads=1` — green.

The branch requires a **delta review** over the remediation commits before the
zero-major verdict stands. Merge remains a separate owner-approval gate.

### Delta-review remediation start gate (2026-07-31)

The delta review found that R1 was not closed: the four new tests were unit
tests with a hidden WinRT FFI dependency, and the three production diagnostic
dispatch branches still were not fired. It also found the same R2--R5 claims
left standing on sibling documentation surfaces. Before choosing the second
remediation shape, the implementation-gate catalog was re-read and classified:

| Trap | Applies | Reason / required close artifact |
|---|---|---|
| 1 — semantic migration | no | No enum, schema, IR variant, or field changes. |
| 2 — missed side effects | **yes** | Moving diagnostic decisions across the OS/pure boundary can lose the emitted message or the client-read failure verdict. Enumerate those effects at close. |
| 3 — parallel / derived drift | **yes**, documentation analogue | R2--R5 each survived on a sibling claim-bearing surface. Close with a proposition-based search and an explicit occurrence table, not a file-local edit. |
| 4 — authored branch | **yes** | This remediation exists to make each diagnostic decision and its sink dispatch fire in a test that has no Win32/WinRT FFI dependency. Record one test per branch and re-run the diagnostic-removal mutation. |
| 5 — carry-forward | **yes** | The value-level boundary must be stated narrowly: a pure test may receive a local error summary, but may not construct or format `windows::core::Error`. Record the rule and its re-trigger criterion in the T7 retrospective. |
| 6 — deterministic failure | **yes** | The delta review supplied two deterministic counterexamples: removing all three production emissions left 470 + 4 tests green, and M6 left all four integration tests green. Re-run both after the fix and disposition the result. |
| 7 — GUI evidence | no | The remediation changes diagnostics, tests, and claim accuracy; it does not deliver or alter GUI rendering. |

**Review lane:** full independent review. Although the immediate defect is
diagnostic/test focused, the correction moves the runtime's OS/pure boundary
and updates the T7 runtime structure, so the high-risk lane is the conservative
classification. The branch/test-focused trap-4 check composes with it.

### Delta-review remediation result and close gate (2026-07-31)

The second remediation replaces the first extraction rather than layering a
test-only exception onto it. `windows::core::Error` exists only above the
boundary; `RectangleSnapshot`, the owned error summary, `GeometryProgress`, and
the diagnostic sink are the complete input/output surface below it.

**Trap 2 — structural side effects and OS/pure call-site audit.** Query:
`rg -n "finish_rectangle_application|client_extent_or_failure|finish_text_refresh|emit_runtime_diagnostic|os_error_summary" wasamo-runtime/src/window.rs`.

| OS result site | Boundary conversion | Pure decision and side effects | Test |
|---|---|---|---|
| `apply_suggested_rectangle` / `SetWindowPos` | `os_error_summary(&result)` | `finish_rectangle_application` emits exactly one diagnostic on failure; fallback behaviour remains independent | `a_failed_rectangle_application_reports_the_rectangle_and_the_consequence` |
| `project_current_client_extent` / `GetClientRect` | `os_error_summary(&read)` | `client_extent_or_failure` emits exactly once and returns `GeometryProgress::Failed`; success preserves `(right-left, bottom-top)` | failure and success `client_extent_or_failure` tests |
| `refresh_text_at_new_scale` / recursive WinRT refresh | `os_error_summary(&refreshed)` | `finish_text_refresh` emits exactly once; the pre-existing no-root early return remains above the call | `a_failed_text_refresh_reports_that_geometry_already_converged` |

No new runtime state, tree structure, layout invalidation, callback, or error
channel was added. The diagnostic text is unchanged except for the already
reviewed client-read consequence sentence. The integration diff remains comment
only, so the owner's per-binary Compositor-unavailable observation remains valid.

**Trap 3 — proposition occurrence table.** Searches covered `implementation/`,
`retrospectives/t7.md`, `window.rs`, and the T7 integration test. Historical
finding rows may quote the rejected wording; they now label it as rejected.

| Proposition | Claim-bearing occurrences after remediation |
|---|---|
| R1: FFI-free diagnostic tests / failure reachability / ADR scope | `window.rs` boundary comment and tests; T7 trap-4 account; retrospective constraint. No unit test constructs or formats `windows::core::Error`; F-43 is limited to five live-HWND extent inputs; the ADR's explicit requirement is two diagnostics, with `GetClientRect` identified as implementation posture. |
| R2: intermediate projection, not a measured frame; per-monitor delivery only | `handle_dpi_changed` doc, T7 start-gate row 7, plan close note, and retrospective items 4 / downstream handoff all use the bounded statement. The immutable ADR remains a design prediction, explicitly distinguished from T7 evidence. |
| R3: ordinary resize is not T7 convergence evidence | Plan re-audit point 2 and the `WM_SIZE` comment both identify the call as pre-existing, potentially serving failed or newly attached nodes, and unconditional after a geometry failure. |
| R4: no T7 end-gate source line references | Rows 9 and 10 name `create`, `WidgetNode::scroll_view`, `grid`, `zstack`, and `box_`; older task-history line references elsewhere in the milestone log are outside this claim. |
| R5: witness count | Integration helper comment and F-42 both state three tests read geometry + surface and the failed-projection test reads surface only. |

**Trap 4 — branch tests.** The diagnostic functions receive only `Option<&str>`,
`RectangleSnapshot`, primitive DPI, a local sink, and (for the fallback) return
`GeometryProgress`. Their tested paths contain no Windows type or FFI call.

| Branch | Direct firing |
|---|---|
| Failed rectangle application + sink dispatch | `a_failed_rectangle_application_reports_the_rectangle_and_the_consequence` |
| Failed client read + sink dispatch + `Failed` verdict | `a_failed_client_rect_read_yields_no_projection_and_says_so` |
| Successful client read + extent arithmetic | `a_successful_client_rect_read_yields_the_physical_extent` |
| Failed text refresh + sink dispatch | `a_failed_text_refresh_reports_that_geometry_already_converged` |

Each failure test also asserts that the OS-bound summary text reaches the sink,
not only that a consequence substring was generated.

**Trap 5 — carry-forward.** Recorded in the T7 retrospective item 10. Re-trigger
when a task adds a diagnostic after a Win32/WinRT call: construct and format the
platform error only at the OS boundary, pass an owned local summary below it,
and make the pure decision own dispatch to its sink. A test that constructs or
formats `windows::core::Error` is not pure. The retrospective's universal-claim
enumeration remains a soft proposal unless a vision decision promotes it into a
project-wide required artifact.

**Trap 6 — deterministic mutations and disposition.** Both review
counterexamples were re-run against the replacement:

| Mutation | Observed result | Disposition |
|---|---|---|
| Remove all three `emit(&diagnostic)` calls from the pure decisions | The three named failure tests each failed at `diagnostics.len() == 1`; the three unrelated window tests passed | Trap #4 now distinguishes branch + dispatch from a string builder. Mutation restored. |
| M6: swap fallback extent axes | T7 integration 4/4 remained green; `a_successful_client_rect_read_yields_the_physical_extent` failed with `(750, 1000)` vs `(1000, 750)` | Original coverage finding remains valid. Mutation restored. |

Final restored-state verification:

- `cargo fmt --all -- --check` — green.
- `cargo test -p wasamo-runtime --lib -- --test-threads=1` — 470 passed.
- `cargo test -p wasamo-runtime --test dpi_change_propagation_integration -- --test-threads=1` — 4 passed.
- `cargo test --workspace -- --test-threads=1` — green; only the pre-existing no-linkable-target and linker-message warnings appeared.
- `git diff --check` — green.

Trap 7 remains non-applicable: no GUI rendering behaviour changed or is claimed.
The required full independent re-review remains a merge gate; this close artifact
does not self-certify that review.

### Merge approval (2026-07-31)

The paragraph above asks for one further independent round over the delta
remediation. **That round was not run.** The owner reviewed the state and
authorised the delta reviewer's fixes to be committed and the branch merged, so
the task-end merge gate was closed by explicit owner approval rather than by a
third review returning zero-major. Recorded here so a later reader does not
infer a review that did not happen.

What the merge does rest on, and what it does not:

- **Rests on:** two independent review rounds (round 1 over the whole branch,
  the delta review over the first remediation), both of whose findings are
  confirmed and remediated; the mutation evidence M1–M6 including the delta
  review's own diagnostic-removal counterexample; the owner-observed
  Compositor-unavailable guard firing; and green `cargo fmt --all -- --check`,
  `git diff --check`, `cargo test --workspace` on the final branch state.
- **Does not rest on:** a review of the second remediation itself
  (`ec5b852`, `4013870`). Those two commits are the delta reviewer's own work
  plus its record, verified by the test suite and by re-running the mutations,
  but not independently re-reviewed.

The residual is small and named: the second remediation moves an OS/pure
boundary and rewrites four unit tests, and its own close gate is self-reported.
If a later task finds a defect there, this is the commit range to look at first.

Phase-end (T12) still owns the phase → main gate, which is separate from this
one, and push remains a separate gate again.

---

## T8 — Windows integration evidence (mock-free, CI-gated, fail-not-skip)

### Carry-over audit and responsibility re-audit (2026-07-31, before start gate)

Branch: `feat/m4-phase-1-t8`, created from `feat/m4-phase-1` at `65f3f2b`
(the T7 merge commit).

The completed retrospectives, the handoff and the T7 close leave T8 six live
obligations and nothing else.

| Carried from | Obligation | Disposition here |
|---|---|---|
| T4 F-29 | `WindowState::scale` has no test-visible accessor; the integration tests are a separate crate | T8 adds the `#[doc(hidden)] pub` seam, returning a `u32` DPI so `DipScale` stays crate-private |
| T4 F-28 + T5 | The invariance claim holds of the **DIP client extent**, which the *outer* rectangle does not preserve | T8 synthesises the rectangle from a measured frame and a chosen client extent, and asserts the realised client |
| T2 F-13 + T4 review R-1 | 125 / 150 / 200% are not three equal probes, and every standard scaling has an exactly-representable factor | The matrix adds **100 DPI**, whose factor no `f32` holds exactly |
| T5 F-37 | The one-divisor traversal property needs a mixed-scale tree, which no legitimate path leaves constructible once geometry exists | T8 adds the second `#[doc(hidden)] pub` seam and drives the click through a real `WM_LBUTTONUP` |
| T6 round-1 R3 | The Compositor-unavailable skip path is owed **per binary** | T8 lands a new binary, so the observation is a landing blocker for an owner run |
| T7 F-42 | A `.ui`-driven test can be green and empty, and a "scaled by k" assertion is satisfied by zero | Every witness is asserted non-degenerate, and each carries a second fact of a different shape (surface pixels, row assignment) |

Everything else the audit surfaced is owned elsewhere and is not T8 work: the
PMv2 declaration and the three-host rebuild are T9; the assistant frame
captures, the re-derived capture coordinates and the runnable-set delivery are
T10; the literal monitor crossing is T11; the Moment 2 spec sync and the
`workflow.md` / "show it goes red" process questions are T12 and phase-end.
`lib.rs::window_add_widget` remains a stated content-boundary limit, and the
stale-*receiver* hit-test case remains a documented misuse with no test (T5's
decision, deliberately not pinned as a regression contract).

The task list itself survives the audit. Four things it did not name are added
to [plan.md](./plan.md) T8 before the gate is selected, and three of them are
visible only from working out what the assertions would actually read rather
than from re-reading the list:

- **F-44 — the plan's exactness claim is unreachable from the window T8 is
  handed.** The plan says T8 "chooses the rectangle, so it can assert equality
  rather than a tolerance". Preserving the DIP client extent means
  multiplying the *physical* client by `dpi / 96`, and the created window's
  client is 784 x 561 physical at 96 DPI (T4, measured): 561 x 1.25 is 701.25.
  There is no integer rectangle that preserves the DIP extent, at 120, 144,
  192 **or** 100 DPI, so a literal implementation would have asserted an
  approximate invariance while the plan claimed an exact one. One step fixes
  every factor at once — normalise the physical client to a **multiple of
  24** first, since `96 = 2^5 x 3` and the four DPIs contribute denominators
  4, 2, 1 and 24.
  *(Corrected at close, by mutation M5. The arithmetic here holds. The
  consequence stated with it — "so a literal implementation would have
  asserted an approximate invariance" — is **false of this fixture**: its
  per-tile assertions are insensitive to a sub-DIP change in the client
  extent, and it took a root-Visual readback to make the normalisation
  load-bearing. The close-gate trap-#4 entry records the sequence.)*
- **F-45 — the ADR's evidence item (2) is one claim, not two, until a
  discrete witness is added.** "The DIP layout results are unchanged" and
  "the Visual offsets and sizes moved by the ratio" are the same equation
  whenever the before-state is `s = 1`, because the only reading of a DIP
  layout result the runtime offers is a Visual read back and divided. The
  separating fact has to be **discrete**: the WrapPanel row assignment, which
  the plan's T10 already records as the 9-vs-7 signature. A wrong
  implementation that treats physical as logical moves the row count; a
  correct one cannot.
- **The non-client frame is measured, not predicted.** Below T9 the process
  is unaware, so the frame should be the 96-DPI one whatever a synthesised
  message claims — but that is a prediction about `WM_NCCALCSIZE`, so the
  frame is read as `GetWindowRect - GetClientRect` and the realised client is
  asserted afterwards.
- **Two seams, not one.** F-29 named the scale accessor; the mixed-scale
  hit-test bullet needs a second, to set one node's cached geometry scale
  stale after geometry exists. Both are public-surface additions and so are
  decisions, not implementation details.

**The binary is new rather than an extension of T7's.**
`dpi_change_propagation_integration.rs` fires authored branches and states in
its own header what it is and is not; a scale matrix and a hit-test property
landing inside it would blur both, and the ADR's evidence item (2) is easier
to cite as a named artifact. The cost — a re-opened per-binary
Compositor-unavailable observation, which is an owner run and a landing
blocker — is accepted rather than avoided, on the same terms T6 and T7 paid it.

### Start gate (recorded 2026-07-31, before production-code edits)

Review lane: **normal review** as classified by
[preamble.md](./preamble.md#review-lanes) ("T8 — test-only"). The
classification is re-checked rather than inherited, and it survives with one
qualification: T8 is not purely test-only, because it adds two
`#[doc(hidden)] pub` seams to the runtime crate. Neither is reachable from a
host through `wasamo.h`, neither changes production behaviour, and both are
of the shape [gates section 4](../../../procedures/implementation-gates.md)
leaves outside the high-risk classes. The trap-#4 branch/test-focused check
composes with it and is the substance of this task anyway.

| # | Applies | Reason and planned close artifact |
|---|---|---|
| 1 — semantic migration / call sites | **yes** | No enum or schema changes, but two new `#[doc(hidden)] pub` items land on the runtime's public surface, and the question the trap asks — who else can reach this, and what does each caller mean by it — is live for exactly that reason. Close with a call-site audit of both seams: every caller, its classification, and the reason no production path may acquire one. |
| 2 — structural side effects | **yes** | The client normalisation drives a real `SetWindowPos` and therefore a real `WM_SIZE` before the change, and the hit-test fixture drives a real `WM_LBUTTONUP` that also updates hover and enqueues a signal. Enumerate what each synthesised message drags along, so a test does not silently depend on an effect it did not intend. |
| 3 — parallel / derived data | **yes** | The node geometry-scale seam exists precisely to put a derived copy out of sync with its source. Close by stating which copy is poked, which primitive normally owns it, and why the poke cannot leak into a production path. |
| 4 — authored branch | **yes** | The gate's substance. Every assertion must fire directly and be shown to go red against a deliberately wrong implementation; the mutation table is the artifact, not the passing run. |
| 5 — carry-forward | **yes** | The one-divisor traversal property, the two seams, and the multiple-of-24 normalisation rule are all invariants a later task can trip — T9 makes the real OS drive this path, M4-Phase 2 replaces the readback with layout-derived rectangles. Record each with evidence and a re-trigger criterion. |
| 6 — deterministic failure | **yes**, low expectation | Real windows, real messages and a live Compositor, where "it passed the second time" is the tempting disposition. Carried so any recurring failure is rooted rather than re-rolled; nothing here is expected to be flaky. |
| 7 — GUI positive control | **no** | T8's evidence is headless runtime state read back off live Composition objects, not a rendered frame. The assistant frame captures and their positive-control pairs are T10's, and the human-visible smoke is T11's. No frame is captured here and none is claimed. |

The approach is therefore constrained before editing: both seams return or
take a `u32` DPI so `DipScale` stays crate-private; the client extent is
normalised to a multiple of 24 and the realised value asserted before any
synthesised message is sent; the invariance witness is the discrete row
assignment rather than a number the ratio assertion already reads; and no
assertion is recorded as evidence until a mutation has been shown to break it.

### Implementation result and end gate (2026-07-31)

Landed as one code commit, `0ebf8ae`. The two seams and the tests that consume
them are one commit rather than two because a seam without its consumer is
dead `#[doc(hidden)] pub` surface, and a commit carrying one is the state
trap #1 exists to refuse.

`wasamo-runtime/tests/dpi_scale_matrix_integration.rs` holds three tests;
`ffi::__window_scale_dpi_for_test` and
`WidgetNode::__set_geometry_scale_dpi_for_test` are the seams.

#### Trap 1 — the two new public items, and every caller

Query: `rg -n "__window_scale_dpi_for_test|__set_geometry_scale_dpi_for_test" --type rust`.

| Item | Callers | Classification |
|---|---|---|
| `ffi::__window_scale_dpi_for_test` | `dpi_scale_matrix_integration.rs` x3 (one per test) | Read-only. Returns `(*window).scale.dpi()`. Cannot change runtime state, so no production path can be harmed by its existence; the reason it is a seam rather than a `pub` field is that widening the field would put the scale factor on a `pub use`-exported type and ship the host-visible surface DD-M4-P1-004 declines. |
| `WidgetNode::__set_geometry_scale_dpi_for_test` | `dpi_scale_matrix_integration.rs` x1 | **Write, and deliberately drift-producing.** It writes the derived geometry-scale copy without the projection that owns it — exactly what trap #3 exists to prevent — because the property under test is what survives that drift. Zero production callers, asserted by the query; the doc comment states that it must not acquire one. |

Both take or return a `u32` DPI, so `DipScale` stays crate-private. That is
the same resolution T6 reached for
`WidgetNode::__run_layout_as_window_root_at_dpi_for_test`, which sits
immediately above the new one and is the third member of this family alongside
`ffi::__install_owning_thread_for_test` and its siblings. **Nothing new
crosses the C ABI**: neither symbol is `extern "C"`, neither appears in
`wasamo.h`, and `rg -n "scale" bindings/ examples/` returns no host reference.

#### Trap 2 — what the synthesised messages drag along

The tests drive three real messages. Each is enumerated rather than assumed,
because a test that depends on an effect it did not intend is a test that
reports the wrong thing when that effect moves.

| Message | Dispatched by | Effects the test relies on | Effects it must not rely on |
|---|---|---|---|
| `WM_SIZE` (normalisation) | the test's own `SetWindowPos` before any change | Re-layout at the *current* scale, so the "before" readback is the chosen client extent rather than `wasamo_load_ui`'s | It also runs the ordinary post-layout text refresh; the surface assertion compares before against after, so a refresh on either side is harmless |
| `WM_DPICHANGED` | `SendMessageW` | Steps 1-5 of DD-M4-P1-003, including the nested `WM_SIZE` its `SetWindowPos` dispatches | Which of the nested pass or the fallback produced the geometry — T7 owns that discrimination and T8 asserts only the result |
| `WM_LBUTTONUP` | `SendMessageW` | `hit_test_click` after the pointer crosses the inbound seam | It also clears `mouse_down`, calls `update_hover(down=false)` (which may start a colour animation) and enqueues a `clicked` signal. None is asserted; the signal is never drained, because no message loop runs |

The normalisation is what makes the "before" state a number this file chose.
Without it every assertion below would be about `wasamo_load_ui`'s 784 x 561
client, which is the rectangle F-44 shows cannot be preserved.

#### Trap 3 — the derived copy, poked on purpose

`WindowState::scale` is authoritative; each node's `scale` is the derived
geometry copy, written only by `commit_scale_recursive` from inside a
successful projection (T6/T7). `__set_geometry_scale_dpi_for_test` writes one
node's copy and nothing else — not its children, not its `raster_scale`, not
its Visual. That the Visual is untouched is asserted rather than stated: the
test re-reads the node's rectangle after the poke and requires it to be
bit-identical to the pre-poke one.

The drift is the subject, not an accident. The property is that a hit-test
traversal divides every readback by the **traversal root's** scale, so the
mixture is survivable; per-node division would place the node at
`physical / its own scale`, which is its composited position multiplied by the
window's factor.

#### Trap 4 — every assertion fired, and every test shown to go red

Seven mutations, each built and run. Five are production-code mutations; M5
and the `factor_is_exact` probe are fixture mutations, which is the right
shape for claims about the *fixture's* discriminating power.

| # | Mutation | Result |
|---|---|---|
**Rows M1, M3, M4 and M6 below were re-run against the landed fixture at the
round-2 review (finding MAJOR 1), and their recorded panic sites had gone
stale.** M1–M4 were run *before* M5 added the root-Visual readback, and that
readback sits above `row_shape` and `assert_scaled` in the test body, so on the
landed fixture it is what fires first. The mutations still fail — the same
mutation, the same test, a different assertion. This is the M5 correction
reaching the prose and not the table, i.e. round 1's R1 one level in, and it is
recorded as the sequence rather than silently rewritten.

| # | Mutation | Result **as landed** | Result when first run (pre-M5 fixture) |
|---|---|---|---|
| M1 | The inbound client-extent seam removed from the `WM_SIZE` arm (physical treated as DIP) | `dip_layout_...` **fails** at the root assertion: `(0, 0, 1125, 750)` against `(0, 0, 900, 600)`. The other two pass | `row_shape` read `(9, 2)` against `(7, 2)` |
| M2 | `visual_rect_dip` divides by `self.scale` instead of the traversal root's | `a_stale_descendant_...` **fails** on the stale case (`0` clicks against `1`) while its **control click passes**, so the failure is the divisor and not the coordinates. The other two pass | unchanged — this test has no root assertion |
| M3 | `sync_visuals` writes sizes at `DipScale::IDENTITY` | `dip_layout_...` **fails** at the root assertion: `(0, 0, 720, 480)` against `(0, 0, 900, 600)` | tile 0 width read `88.0` against `110.0` |
| M4 | `surface_pixels` truncates instead of ceiling | `dip_layout_...` **fails**: the first label's surface reads `(24, 23)` against `(25, 24)`. Unshadowed — the root assertion does not read surfaces | unchanged |
| M5 | The fixture's client extent set to 785 x 480 — not a multiple of 24 | Discussed below; **first run passed**, which is the finding | — |
| M6 | M3 with the matrix restricted to 100 DPI | `dip_layout_...` **fails** at 100 DPI, at the root assertion | recorded only as "fails at 100 DPI" |
| M7 | `begin_scale_change` does not commit the scale | `a_created_windows_...` **fails** (`96` against `144`) and `dip_layout_...` with it; the mixed-scale test passes, because a change that never happens is internally consistent | unchanged |

**Shadowed is not dead, and the difference is measured** (added at the round-2
review). The gate's own standard is that **each assertion** fires directly, so
"M1 fires something else now" leaves `row_shape` and the per-tile
`assert_scaled` without a demonstration. Two further runs supply one:

| Run | Result |
|---|---|
| M1 with the `after_root` assertion shadowed (`if false`) | `row_shape` **fires**: `(9, 2)` against `(7, 2)` — the 9-vs-7 signature, live on the landed fixture |
| M3 with `after_root` **and** the root `assert_scaled` shadowed | per-tile **fires**: `dpi=120 tile 0 w`, `88.0` against `110.0` |

So both assertions are reachable and discriminating; what the table had wrong
was which one a given mutation reaches first. **One consequence is not
repaired by that measurement and is recorded instead** (finding MINOR 3): the
*after*-state `row_shape` assertion is implied by the conjunction of the ratio
assertion and the *before*-state row assertion, so no mutation can fire it
alone. It is kept for legibility, not as evidence; see
[plan.md](./plan.md) §T8 re-audit point 3 and `TILES_PER_ROW`'s doc.

Every mutation and every shadowing edit was restored; the final state is the
committed one and the suite is green on it.

**M5 is the one that changed the implementation, and it changed it by
falsifying the reasoning that produced F-44.** F-44's arithmetic is right —
no integer rectangle preserves the DIP extent from 784 x 561 at any matrix
DPI. The *consequence* recorded at the start gate was that the per-tile
geometry assertions therefore could not be equalities, and the first run of
M5 refuted it: at 785 x 480, where the recovered DIP width is 784.8 rather
than 785, **every per-tile assertion still passed**. WrapPanel tiles are
start-packed at a fixed cross-size, so they do not move when the client extent
shifts by a fraction of a DIP. The fixture was therefore not testing the
client extent at all in its continuous half.

The fix is one readback: the **root** node's Visual, which under the
window-root `Fill` override *is* the client rectangle. With it added, the same
785 x 480 run fails at `981.0` against an expected `981.25`, and the
normalisation is load-bearing rather than tidy. Recorded as the sequence it
was rather than as the conclusion: the finding's premise survived, its stated
consequence did not, and only running the mutation separated them.

**A second measurement came out of the same run and is not a rounding
issue at all.** With the client at 784 x 561, the 192 DPI case asks for a
1568 x 1122 client and the window realises 1568 x **1014** — the monitor's
maximum track size. `set_client_extent`'s realised-client assertion is what
turns that into a named failure instead of a silent one, and it is a real
constraint on any later scale fixture: **the 200% target must fit the
display.** 720 x 480 asks for 1440 x 960 and gets it on the development
machine.

**And the exactness split is measured, not defensive.** Forcing
`factor_is_exact` to `true` fails at 100 DPI on the root height: `500.0` read
against `499.99997` expected. The *runtime* produced exactly 500; the test's
naive expectation `480 x f32(100/96)` is the imprecise number, because the DIP
extent the runtime laid out into is `500 / f32(100/96)` = 480.00003 rather
than 480. So the invariance holds at 100 DPI and its naive restatement does
not — an `f32` property, not a conversion-boundary one.

#### Trap 5 — carry-forward

Recorded in the T8 retrospective's item 10 and carried to
[handoff.md](./handoff.md) at phase close:

- **A scale-invariance fixture normalises its physical client to a multiple
  of 24 and asserts the realised value.** *Evidence:* 96 = 2^5 x 3, so a
  multiple of 24 makes 100 / 120 / 144 / 192 DPI all produce integer targets;
  and 784 x 561 fails at 192 DPI on the monitor's max track size rather than
  on rounding. *Re-trigger:* any later task that asserts layout invariance
  across a scale change — T10's control B, T11's literal form, M4-Phase 8's
  second window.
- **A fixture can be insensitive to the very input it claims to hold
  constant.** *Evidence:* mutation M5 — start-packed WrapPanel tiles did not
  move when the client extent shifted by 0.2 DIP, so the continuous half of
  the invariance claim was reading nothing about the client extent until the
  root Visual was added. *Re-trigger:* any assertion of the form "input X was
  preserved, so output Y is unchanged" where Y is not demonstrably a function
  of X.
- **`__set_geometry_scale_dpi_for_test` has zero production callers and must
  keep zero.** *Evidence:* it writes the derived geometry copy outside the
  projection that owns it, which is the drift trap #3 exists to prevent.
  *Re-trigger:* any production path that needs to set a node's cached scale
  — which would mean a second writer beside `commit_scale_recursive`, the
  thing T6's boundary exists to prevent.
- **The one-divisor traversal property now has a test, and it is the
  *descendant* case only.** *Evidence:* `a_stale_descendant_...`, shown to
  fail under M2. The stale-*receiver* case — entering `hit_test_click` on a
  subtree whose cache is not the window's — remains a documented misuse with
  no test, deliberately (T5's decision: pinning it would fix a stated limit as
  a regression contract). *Re-trigger:* M4-Phase 2's option H3, hit rectangles
  cached from layout, deletes the property and its test together.

#### Trap 6 — deterministic failure

No failure was re-rolled and no test was retried to green. Every mutation
result above was the expected one except M5's first run, which was expected to
fail and passed — the disposition is the fixture change recorded under trap #4,
not a re-run. The three tests were run repeatedly across seven mutation cycles
on a live Compositor with no intermittent result.

#### Verification

All commands on Windows, against the post-commit branch state:

- `cargo fmt --all -- --check` — green.
- `git diff --check` — green.
- `cargo test -p wasamo-runtime --test dpi_scale_matrix_integration -- --test-threads=1` — 3 passed.
- `cargo build --release --workspace` — green.
- `cargo test --workspace -- --test-threads=1` — green; 35 test binaries, no `FAILED`, runtime unit tests still 470.

No GUI capture: trap #7 is non-applicable for the reason recorded at the start
gate, and no frame is claimed.

**The stated limits**, recorded here and in the test's own module header so a
reader of either finds them:

1. A synthesised `WM_DPICHANGED` proves the handling path. It does **not**
   prove that crossing a real monitor boundary delivers the same message with
   a usable suggested rectangle. That half is T11's, and neither alone
   discharges AC7's third requirement.
2. The exact-invariance assertion holds **because this file preserves the DIP
   client extent**, which choosing the rectangle is necessary and not
   sufficient for: the chosen physical client must give an **integer** target
   at every DPI in the matrix (not the same as the DIP extent being
   recoverable bit-for-bit, which fails at 100 DPI), the realised value must
   be asserted, and the quantity asserted must be **sensitive to** that extent
   at the precision the claim needs. The OS's suggested
   rectangle preserves the **outer** rectangle instead, so on the real path
   the DIP layout input moves by a DIP or two and invariance is approximate.
   T11 is where that shows, and it must not read as a failure.
3. The process has not declared awareness yet, so `GetDpiForWindow` reports 96
   here and the creation-time half of the cached-scale test is `96 == 96`
   until T9. What is live now is the post-change half. The test asserts
   `os_dpi == 96` explicitly, so T9 will land on a failing assertion rather
   than on a silently degenerate one.

**End-gate result: passed**, subject to the review lane and to one landing
blocker that is not a review finding: **this binary's Compositor-unavailable
skip path has not been observed firing.** The helper is the already-verified
shared one, but T6's round-1 R3 classified the observation as owed per binary,
so this binary needs one owner run on an environment where `wasamo_init`
returns `0x80070005`. CI is a separate gate again: the same helper statically
refuses to skip when `GITHUB_ACTIONS` is set, so the fail-not-skip direction is
closed by construction, but the actual CI run id belongs to the phase-end
batch. No merge approval is implied.

### External Compositor-unavailable evidence (2026-07-31)

The owner ran the T8 test binary in a Windows session where runtime
initialisation reached the established Compositor-unavailable
classification:

`cargo test -p wasamo-runtime --test dpi_scale_matrix_integration -- --nocapture --test-threads=1`

All three tests entered their named negative path rather than their
Compositor body:

| Test | Observed guard output |
|---|---|
| `a_created_windows_cached_scale_is_the_dpi_the_os_reports` | `skipping cached window scale: runtime compositor unavailable` |
| `a_stale_descendant_scale_still_hit_tests_where_the_widget_is` | `skipping mixed-scale hit test: runtime compositor unavailable` |
| `dip_layout_is_invariant_while_every_visual_moves_by_the_ratio` | `skipping DIP invariance across the scale matrix: runtime compositor unavailable` |

Result: 3 passed, 0 failed, 0 ignored, 0 measured, 0 filtered out.

**It was the same binary, not a rebuild.** Cargo reported
`Finished ... in 0.07s` and ran
`dpi_scale_matrix_integration-911796c0ce6f8da6.exe`, the artifact the clean
rebuild recorded in the T8 retrospective's item 3 had produced; the working
tree was clean over the whole interval between that rebuild and this run, the
only commits in it being documentation. So the difference between this run and
the local live-Compositor 3/3 run is the session's Compositor capability and
nothing else, which is what makes it a control on the guard rather than on a
second build.

**Two corrections to that argument, both from the round-2 review (NIT 1).**
*(i)* "The only such artifact on disk" was offered as the strong half and is
**not an identity argument at all**: cargo's filename hash is derived from
package, target and profile metadata, not from source, so a mutated build
writes the same filename — as the round-2 reviewer's own M1 and M3 runs
demonstrated. It says which file was executed, not what was in it. The weight
therefore rests on the clean tree plus cargo's freshness verdict, which is the
half the phase has three findings against (F-5 / F-21 / F-40) — mitigated
here, but not more than that, by the fact that the whole-archive path those
findings concern is the `wasamo.dll` link, which an integration test binary
does not take. *(ii)* The clean-tree half is stated **for that interval** and
must not be read as standing: the round-2 review applied and reverted seven
mutations, which rebuilt this artifact (mtime 5:08 → 6:23). The owner's
observation is a past event and is unaffected; the argument's shelf life is
not.

*(iii)* **A content hash was prescribed here as "the cheap fix" and that
prescription is wrong — measured at the round-3 review, finding MINOR B.**
This repository's debug artifacts are **not bit-reproducible**: rebuilt from
an unchanged source tree they hash differently every time. Five builds, five
values — the reviewer's three and two more taken here by touching the test
file's mtime and rebuilding:
`DEEDD907…`, `9E1C632D…`, `98E873BC…`, `EC2FAB51…`, `E017EF34…`. So a hash
recorded in a document is unverifiable even by its author, and worse as a
*prescription*: a later task following it would record a number nobody can
check, and a later reviewer comparing hashes would get a **false positive**
for "the binary changed" — a fresh instance of the false-green /
false-alarm class this phase already carries three findings about
(F-5 / F-21 / F-40). The recorded value is therefore withdrawn rather than
kept with a caveat.

**What does work, and is what should have been written**: hash *the same
file twice*, before and after the run being attested, and require the two to
agree. That measures identity across an interval, which is the actual claim,
and it assumes nothing about the build being deterministic. `LastWriteTime`
read at the same two points is cheaper and sufficient for the same purpose.

This is the required actual firing of the substring-classified local skip
branch, per [AGENTS.md §Testing rules](../../../../AGENTS.md) — a guard
verified only on the happy path is not verified — and per T6 round-1 R3's
finding that the observation is owed **per binary** rather than inherited from
the shared helper. The same helper statically asserts that this branch may not
skip when `GITHUB_ACTIONS` is set, so the CI direction is closed by
construction and the run id itself belongs to the phase-end batch.

T8's sole landing blocker is closed. Normal review before merge remains
outstanding, and merge to `feat/m4-phase-1` is a separate owner-approval gate.

### Independent review disposition (2026-07-31)

The independent review of `feat/m4-phase-1..e80fb7a` returned **1 major and 1
nit**. The zero-major gate was not met. Both were confirmed and neither was
argued down.

**The reviewer verified rather than read** on the load-bearing claims: it
re-ran the binary against a live Compositor (3/3), re-derived M1 and M2 from
their descriptions and observed the predicted failures — root extent
`1125 × 750` against `900 × 600`, and the stale-descendant click at `0`
against `1` — re-ran the workspace suite and both doc gates, and restored its
mutations. It pushed back on three things this task had flagged as possibly
fragile (`row_shape`'s exact-equality grouping, `send_click`'s signed `i16`
packing, `set_client_extent`'s use of a pre-change frame) and on the review
lane, finding all four sound for the fixture and input range as landed.

| # | Severity | Finding | Disposition |
|---|---|---|---|
| R1 | major | The F-44 correction did not reach the documents that assert the same proposition in other words. `implementation/preamble.md`'s verification-closure item (2) and `plan.md` §T8's positive-control bullet both still say T8 can assert equality **because** it chooses the rectangle, which is exactly what M5 showed to be insufficient — while the F-44 correction record two screens above the second one, and the test's `CLIENT_W` doc, say the opposite. The document contradicts itself | **Confirmed.** Corrected at **six** sites; the enumeration is below and it found four the review did not name. The ADR side is enumerated too, with one row raised to the owner rather than corrected here |
| R2 | nit | `__window_scale_dpi_for_test`'s safety contract says only "a live `WasamoWindow` pointer" while the body dereferences a raw pointer immediately | **Confirmed.** Now states non-null, aligned, valid for a shared read for the duration of the call, not valid after `wasamo_window_destroy`, not retained, and owning-thread-only. Contract clarification, not an implementation defect |

**R1 is the sixth occurrence of this shape in the phase** (T4 R-2, T5 R-2 and
its close-gate row 9, T7 R2–R5, and the T4 preamble's own four sites), and the
fifth time the corrective the plan already carries — *write the falsified
proposition as one sentence first, then enumerate the documents that assert
it, then search* — was **run and still under-scoped**. What went wrong this
time is narrower than "did not enumerate": the enumeration was run against the
finding's *name* (F-44) rather than against the proposition, so it visited the
two places that say "F-44" and stopped. The plan's rule says to enumerate the
asserting documents, and `implementation/preamble.md` is on the standing list
it gives. It was not opened.

**The falsified proposition, as one sentence:** *"Because T8 synthesises the
message, it chooses the rectangle, and that is what lets it assert equality
rather than a tolerance."*

**The corrected proposition:** choosing the rectangle is **necessary and not
sufficient**. Three conditions carry the equality, and only the first is about
synthesising: (a) the message is synthesised, so the rectangle is chosen;
(b) the chosen **physical** client is one the DIP extent is exactly
recoverable from at every DPI in the matrix — a multiple of 24, since
`96 = 2^5 × 3` — and its **realised** value is asserted rather than assumed,
because a requested rectangle the display cannot honour is silently not the
one applied; (c) the quantity asserted is a **function of** that extent, which
T8's per-tile geometry is not and its root Visual is.

**Occurrence table**, enumerated before searching and then checked by
`rg "chooses the rectangle|choosing the rectangle|synthesises the message and|controlled client extent|preserves the client extent|because this file chooses"`
over `process/milestone-4/phase-1/`, `docs/` and `wasamo-runtime/tests/`:

| Site | Asserts it? | Named by the review? | Action |
|---|---|---|---|
| `implementation/preamble.md` §Verification closure item (2) | yes | yes | Corrected in place: the "necessary and not sufficient" clause and the three conditions |
| `implementation/plan.md` §T8 positive-control bullet | yes | yes | Corrected in place, same three conditions |
| `implementation/plan.md` §T8 stated-limit bullet | yes | **no** | Corrected: the second stated limit now points at the three conditions rather than at "T8 preserves the client extent" alone |
| `wasamo-runtime/tests/dpi_scale_matrix_integration.rs` module header, limit 2 | yes | **no** | Corrected. The review read the `CLIENT_W` doc — which was right — and not the header eight lines above it, which asserted the uncorrected form |
| `implementation/log.md` §T8 end gate, stated limit 2 | yes | **no** | Corrected, same wording as the header |
| `retrospectives/t8.md` §後続タスク / オーナーへ共有すること | yes | **no** | Corrected |
| `implementation/plan.md` §T8 re-audit point 1 (the F-44 record) | states the correction | — | Already correct; this is the site the contradiction was *against* |
| `implementation/log.md` §T8 carry-over audit, F-44 bullet | states the correction | — | Already correct |
| `dpi_scale_matrix_integration.rs` `CLIENT_W` doc | states the correction | — | Already correct |

**The ADR side, enumerated with its verdict rather than swept in.** The review
recommended listing it, and listing it is what separates the two rows:

| ADR site | Verdict |
|---|---|
| [DD-M4-P1-003 §Context](../decisions/dd-m4-p1-003-dpi-change-propagation.md)'s 2026-07-29 annotation — "exact invariance is a property of a **controlled client extent** … the integration test preserves the client extent and therefore still asserts equality rather than a tolerance" | **Survives — reason restated at round 2 (MINOR 1), because the original one-liner was ambiguous in the way that invites the opposite verdict.** "Controlled" has two readings. Under *chosen by the test* it is necessary-not-sufficient exactly as "chosen" is, and 785 × 480 falsifies it. Under *held at the same DIP value* it is sufficient — and 785 × 480 is then **not** a controlled extent, because its DIP extent moves from 785 to 784.8, which is the entire content of M5. The annotation's own contrast fixes the second reading: it is set against the OS-suggested rectangle, whose defect is that it **moves** the client extent. The sentence is sound under the reading its context establishes. Recorded because "checked and it holds" and "did not look" are different facts — and so is "checked, and said why in a way that does not survive being read the other way" |
| Same annotation's **revision-history summary** (`dd-m4-p1-003-…md` §Revision history, 2026-07-29) — "it holds of a controlled client extent, not of the OS-suggested rectangle" | **Survives, same reading.** Added at round 2 (MINOR 2): it is *derived prose* summarising the body annotation, which is the documentation form of trap #3 — the trap T8 declared applicable and then closed only over the node-side derived copy. Recorded rather than corrected: the summary says what the body says |
| [decisions/preamble.md](../decisions/preamble.md) §Revision log, 2026-07-29 row — "exact invariance is a property of a controlled client extent" | **Survives, same reading.** Added at round 2 (MINOR 2), same derived-prose class. Also recorded: the round-1 query `controlled client extent` cannot match `a *controlled* client extent`, so markdown emphasis inside a phrase is a way a proposition search silently under-reports |
| This log's own §T7 structural side-effect table, row 4 — "'identical results' holds of a *controlled* client extent; T8 preserves one, T11 does not" | **Survives**, and its wording is the better one: "preserves" is the held-constant reading spelled out, so it does not depend on how "controlled" is taken. **Added at round 3 (NIT A), and the reason it was missed twice is the finding**: round 2 raised it and the round-2 disposition left it out with no verdict, because it sits in a *T7* artifact and "is a historical record correctable here?" was treated as a reason to defer rather than a question to answer. Enumerating it costs one row and answering it costs one sentence; deferring it cost two rounds. It is also the exact instance of the markdown-emphasis blind spot the row above records — `*controlled*` again |
| [decisions/preamble.md](../decisions/preamble.md) §Phase 1 verification closure item 2's 2026-07-29 qualification — "the unchanged-results half is exact **because the test synthesises the message and therefore holds the client extent constant**" | **Asserts the falsified causation**, in the `therefore`. **Raised to the owner, not corrected here.** No decision or option is re-chosen, so by the boundary T4 established — *supersede when a reader implementing the original text would not obtain the shipped behaviour; annotate when the decision still produces it and only a statement around it was too strong* — this is at most a third dated annotation, and the annotate route on an Accepted record is the owner's call, as it was at T4. The implementation-side documents now carry the corrected form, so nothing downstream reads the ADR for this |

Post-remediation verification, on the branch tip:

- `cargo fmt --all -- --check`, `git diff --check` — green.
- `cargo test -p wasamo-runtime --test dpi_scale_matrix_integration -- --test-threads=1` — 3 passed.
- `cargo test --workspace -- --test-threads=1` — green.

The remediation is documentation plus one doc comment; no test or production
logic changed, so the owner's per-binary Compositor-unavailable observation
and the mutation evidence both remain valid. The branch requires a **delta
review** over the remediation commit before the zero-major verdict stands.
Merge remains a separate owner-approval gate.

### Independent review round 2 disposition (2026-07-31)

A **full** independent review rather than a delta review over the round-1
remediation, on owner instruction. The reason it is worth the extra pass is
recorded because it is a process judgment, not a preference: T7 merged on
round 1 plus a delta review and left its second remediation independently
unreviewed as a named residual, and round 1's major here was a
documentation-propagation defect whose fix touched six files — the shape a
diff-scoped review reads least well.

Result: **2 major, 5 minor, 2 nit.** The zero-major gate was not met. Eight of
the nine are confirmed; **one minor is partially pushed back**, with the reason
below.

**The reviewer verified rather than read.** It re-ran the binary against a live
Compositor (3/3, not skipped), re-derived M1 and M3 from their descriptions and
observed panics the table did not predict, forced `factor_is_exact` to `true`
and observed the 100 DPI failure, searched `bindings/` and `examples/` for the
scale surface, ran the workspace suite and both doc gates, enumerated the
proposition's occurrence sites independently of the table, and restored every
mutation to a clean tree. Every claim below marked *measured* was re-measured
here as well before dispositioning.

| # | Severity | Finding | Disposition |
|---|---|---|---|
| MAJOR 1 | major | The mutation table's M1 / M3 / M4 / M6 rows record panic sites from the pre-M5 fixture. On the landed fixture the root-Visual assertion M5 added fires first, so `row_shape` and the per-tile `assert_scaled` have no mutation demonstrating them — while the start gate's standard is "each assertion fires directly, not incidentally" | **Confirmed, and re-measured.** M1 reproduces at the root assertion (`(0,0,1125,750)` against `(0,0,900,600)`), M3 likewise. Table rewritten with as-landed **and** first-run columns. Two shadowing runs added to close the gap the finding names: with `after_root` disabled M1 fires `row_shape` at `(9,2)` against `(7,2)`, and with the root `assert_scaled` also disabled M3 fires per-tile at `88.0` against `110.0`. Both assertions are therefore live; what was wrong was the recorded panic site |
| MAJOR 2 | major | The round-1 remediation's proposition (b) — the physical client must be one the DIP extent is "exactly recoverable from at every DPI in the matrix" — is **false at 100 DPI**, and the same close artifact records the counter-measurement (480.00003) three screens below | **Confirmed.** This is the phase's recurring over-strong claim arriving *inside the commit that fixes an over-strong claim*, which is the part worth recording rather than the wording. (b) now separates two facts that the phrase "exact" was carrying at once: a multiple of 24 gives an **integer physical target** at all four DPIs (rational arithmetic), while bit-for-bit recovery of the DIP extent holds at three (`f32`). Corrected at all five sites the review enumerated |
| MINOR 1 | minor | The verdict "DD-M4-P1-003's annotation survives" is wrong, because its reason — "It says *controlled* client extent" — leans on a word that is necessary-not-sufficient exactly as "chosen" is; 785 × 480 is a controlled extent that does not give exact invariance | **Partially pushed back; the verdict stands, the reasoning does not.** "Controlled" carries two readings and the disposition named neither, which is the real defect. Under *chosen by the test* the reviewer is right and the sentence is falsified. Under *held at the same DIP value* it is sufficient, and 785 × 480 is not a controlled extent in that sense — its DIP extent moves from 785 to 784.8, which is the whole content of M5. The annotation's own contrast fixes the second reading: it is set against the OS-suggested rectangle, whose defect is precisely that it **moves** the client extent. So the ADR sentence is sound and the disposition's one-line reason was ambiguous in the way that invites the reviewer's reading. Reason replaced with the explicit reading; **no ADR change** |
| MINOR 2 | minor | The ADR-side enumeration stopped at two body sites and missed two revision-log summaries of the same annotation — derived prose, which is the documentation form of trap #3 that T8 declared applicable | **Confirmed.** Both added to the table with verdicts. The reviewer also names why the search missed them: the query `controlled client extent` does not match `a *controlled* client extent` with markdown emphasis inside it |
| MINOR 3 | minor | The discrete witness is not independent of the ratio assertion: `row_shape` is computed from the same array `assert_scaled` reads, and a partition by equal `Y` is invariant under positive scaling, so `row_shape(after)` follows from the ratio assertion plus `row_shape(before)` | **Confirmed, and it is the most substantive of the nine.** F-45's *problem* stands — the two halves of evidence item (2) are one equation at `s = 1` — but the answer T8 gave does not separate them. What does is on the **input** side: the realised physical client against a computed target, and the root Visual against this file's constants. The after-state row assertion is kept for **legibility** and is now labelled as such. "Not a number derived from the Visual geometry" is withdrawn |
| MINOR 4 | minor | "the per-tile geometry is **not** a function of that extent" is over-strong and contradicts the same file, which uses per-tile `Y` offsets to compute the extent-sensitive row assignment | **Confirmed.** The measurement was always "insensitive below about a DIP"; the distillation widened it. Replaced with "sensitive to that extent at the precision the claim needs" at all three sites |
| MINOR 5 | minor | The 9-vs-7 signature degenerates at 100 DPI: a physical-as-logical implementation lays out into 750 DIP, and `floor((750+12)/100)` is 7 — the same count a correct implementation gives | **Confirmed by arithmetic here.** Recorded on `TILES_PER_ROW` and in the plan. The discrete witness discriminates at 120 / 144 / 192 and is blind at 100, where only the root and ratio assertions catch M1 |
| NIT 1 | nit | "the only such artifact on disk" is not an identity argument — the filename hash is metadata-derived and survives a source change, as the reviewer's own mutations demonstrated | **Confirmed**, and it compounds: the reviewer's mutation cycle rebuilt that binary, so "the working tree has been clean since" is no longer true of the present. Both corrected, and the guard evidence now carries a file hash |
| NIT 2 | nit | "exact" is used for two different things in one file — integer targets, and `f32` representability | **Confirmed**; it is MAJOR 2's root cause. `CLIENT_W`'s doc now says "integer targets" |

**Round 1's dispositions, re-judged:** R1 (the F-44 propagation) is **not
closed** — the correction reached six sites, but it introduced MAJOR 2, the
ADR verdict in MINOR 1 was under-specified, the ADR enumeration was incomplete
(MINOR 2), and the same M5 correction never reached the mutation table
(MAJOR 1). R2 (the safety contract) is **closed**, neither under- nor
over-corrected. The reviewer pushed back on nothing from round 1 and
independently reached the same conclusion on all four items round 1 had
pushed back on.

**What this round costs the task's own corrective, and what survives it.**
The round-1 retrospective proposed: *propagate by proposition, not by the
finding's name; if the enumeration starts at `rg` it has already failed.*
Round 2 shows that corrective is necessary and **not sufficient** — the
enumeration this time was written by hand and still (i) stopped at the prose
and never asked whether the *close artifact's own table* asserted the
proposition, and (ii) restated the proposition in a form that was itself too
strong. So the corrective is extended rather than replaced: **the enumeration
must include the artifacts the task produced, not only the documents it
inherited; and the corrected proposition is a claim like any other, so it gets
the same "is this true of every member of its set" check the original failed.**

Post-remediation verification, on the branch tip:

- `cargo fmt --all -- --check`, `git diff --check` — green.
- `cargo test -p wasamo-runtime --test dpi_scale_matrix_integration -- --test-threads=1` — 3 passed.
- `cargo test --workspace -- --test-threads=1` — green.

No test logic and no production logic changed in this remediation: the diff is
prose, doc comments, and one assertion message. The owner's per-binary
Compositor-unavailable observation and the mutation evidence therefore both
remain valid. A further independent round is required before the zero-major
verdict stands; merge remains a separate owner-approval gate.

### Independent review round 3 disposition — delta over the round-2 remediation (2026-07-31)

**Zero-major reached.** No new major; both round-2 majors closed. The round
returned **2 minor and 1 nit**, all three confirmed and remediated here, and it
**accepted the one push-back** T8 made.

**The reviewer re-derived rather than accepted.** It independently reproduced
both shadowing runs — M1 with `after_root` disabled firing `row_shape` at
`(9, 2)` against `(7, 2)`, and M3 with the root `assert_scaled` also disabled
firing per-tile at `88.0` against `110.0` — matching the recorded values
exactly. It checked MAJOR 2's replacement proposition against every member of
its set (24 is the lcm of the four denominators; `dpi / 96` is dyadic exactly
when `3 | dpi`, giving three of four) and found it neither over- nor
under-stated. And it produced a counter-measurement of its own, below.

| # | Round-2 finding | Round-3 verdict | Action |
|---|---|---|---|
| MAJOR 1 | mutation table recorded pre-M5 panic sites | **closed** — shadowing runs independently reproduced | — |
| MAJOR 2 | "exactly recoverable at every DPI" false at 100 | **closed** — replacement checked over its whole set | — |
| MINOR 1 | the "controlled" verdict's reasoning | **closed; the push-back is accepted** — see below | Reasoning stands as written |
| MINOR 2 | ADR enumeration missed two revision logs | **closed**, one residual → NIT A | Third row added |
| MINOR 3 | the discrete witness is not independent | **not closed** — the *replacement* claim is over-strong | Corrected, below |
| MINOR 4 | "function of" too strong | **closed** | — |
| MINOR 5 | 9-vs-7 degenerates at 100 DPI | **closed** | — |
| NIT 1 | filename hash is not identity | **not closed** — the prescription added with the fix does not work | Corrected, below |
| NIT 2 | "exact" used in two senses | **closed** | — |

**Over-closed: none.** In particular the reviewer endorsed keeping the
after-state row assertion for legibility rather than deleting it, on three
grounds: deleting it makes an M1-class failure present as a list of `f32`
mismatches with the phase's own 9-vs-7 signature absent from the output; the
doc *and* the assertion message both now say it is redundant and not evidence,
so the misreading path is closed; and a redundant assertion costs no runtime.

#### MINOR 1 — the push-back was accepted, and the reason narrows the residual

The reviewer withdrew its own finding after checking the annotation's context:
the contrast is drawn against the OS-suggested rectangle, whose defect is that
it **moves** the client extent, so the axis is preserved-vs-moved and not
chosen-vs-not; and the very next sentence glosses "controlled" as
"**preserves** the client extent". It also confirmed independently that the
held-constant reading is genuinely sufficient — identical DIP input into a pure
layout gives identical DIP output, and physical is then `dip × factor` against
a before of `dip × 1` — and that 100 DPI is not a counter-example but a case
where the antecedent fails.

**One thing it added is worth more than the finding it withdrew.** The reviewer
read the ADR body directly and took the wrong reading, so the ambiguity is
demonstrably in the ADR sentence and not only in T8's summary of it — n = 1,
measured. That does not make an ADR edit T8's to do. It is folded into the
question already with the owner about `decisions/preamble.md` item 2: **if**
that annotation is written, the cheapest disambiguation is to say
"controlled (held at the same DIP value)" in the same breath.

#### MINOR 3 — the replacement separator was over-strong in two ways

The withdrawal held: the round-2 correction is right that `row_shape(after)`
follows from the ratio assertion plus the pinned before-state, and the
reviewer confirmed it is carried through to the assertion message itself. What
did not hold is what replaced it — "**what separates them is the input side**:
the realised client, **and the root Visual**, … nothing read off the
post-change tree can stand in for them".

1. **The sentence disqualifies one of its own two members.** `after_root` is
   read off the post-change tree. That is the same structure as the claim just
   withdrawn, one member over.
2. **`after_root` is partly implied.** `before_root` is pinned to the
   constants and `assert_scaled` covers `.2` / `.3`, and at the three exact
   DPIs `720 × factor` is exactly `target_w` — so the size components follow
   there. What stays independent is the **offset** components, which no
   `assert_scaled` touches, and the **100 DPI** case, where the exact tuple is
   strictly stronger than a tolerated ratio.
3. **"Separates" claims the wrong thing.** F-45's problem is that the runtime
   offers one reading of a DIP layout *result*. An input-side assertion does
   not supply a second one; it guarantees the experiment held the input it
   claims to have held. That is the right role and a narrower one.

Corrected at both sites. `realised_client` alone carries the "no ratio
assertion touches it" claim — it comes from `GetClientRect`, not from the
Visual tree.

#### NIT 1 — the fix was right and the prescription attached to it was not

The two corrections stand. The **content hash prescribed beside them does
not**: this repository's debug artifacts are not bit-reproducible. Measured
over five builds of an unchanged source tree — three by the reviewer, two here
by touching the test file's mtime and rebuilding — **five distinct SHA-256
values**. So the recorded hash was unverifiable by its own author, and as a
prescription it would have had later tasks record uncheckable numbers and
later reviewers read a rebuild as "the binary changed": a false-alarm
generator, in a phase already carrying three findings about false signals from
build artifacts.

Withdrawn and replaced with what measures the actual claim: **hash the same
file twice, before and after the run being attested, and require agreement** —
identity across an interval, assuming nothing about determinism.
`LastWriteTime` at the same two points is cheaper and sufficient. Carried
forward, because a later task designing evidence around reproducible builds
would be building on sand.

#### The pattern the reviewer named, and what it changes

**A remediation has introduced a fresh over-strong claim in three consecutive
rounds** — round 1 produced MAJOR 2, round 2 produced MINOR 3's replacement
separator and NIT 1's hash prescription. The magnitude is shrinking
(major → minor → minor) and the round-2 corrective demonstrably worked where
it was pointed: MAJOR 2's replacement was checked over its whole set and
survived. It did not reach the claims that were *not* the corrected
proposition — a separator description and a procedural recommendation.

So the corrective is widened again, and this is the third widening:
**the all-members check applies to everything a remediation commit newly
asserts, not only to the proposition being corrected.** Recorded in the T8
retrospective.

Post-remediation verification, on the branch tip:

- `cargo fmt --all -- --check`, `git diff --check` — green.
- `cargo test -p wasamo-runtime --test dpi_scale_matrix_integration -- --test-threads=1` — 3 passed.
- `cargo test --workspace -- --test-threads=1` — green, 35 binaries.

This remediation changes prose and doc comments only; no assertion, tolerance,
comparison or control flow moved. The owner's per-binary
Compositor-unavailable observation and the mutation evidence remain valid, and
the reviewer independently confirmed that reading of the round-2 diff before
relying on it.

---

## T9 — Declare Per-Monitor-Aware V2 + three-host rebuild

### Carry-over audit and responsibility re-audit (2026-07-31, before start gate)

Branch: `feat/m4-phase-1-t9`, created from `feat/m4-phase-1` at `1d38222`
(the T8 merge commit).

The completed retrospectives, [handoff.md](./handoff.md) and the T8 close
leave T9 the obligations below and nothing else.

| Carried from | Obligation | Disposition here |
|---|---|---|
| T8 retrospective | `a_created_windows_cached_scale_is_the_dpi_the_os_reports` asserts `os_dpi == 96` and will **fail**, deliberately, the moment the declaration lands | Discharged, and **under-scoped by the handoff** — see F-47. The breakage is 5 tests across 2 binaries, not 1 assertion in 1 binary |
| T4 (§T9 `Cargo.toml` note) | The declaration symbols are measured available; the two *query* symbols the effective-level assertion needs are **not** exercised | Closed by measurement at this re-audit, and widened to a fourth symbol the tolerated-failure binary needs |
| T1 F-9 / F-10 | The declaration goes **below** the `RUNTIME.get().is_some()` early return; the existing one-shot is sufficient and no new guard is added | Taken as landed. T1 verified both halves against the source; nothing since has moved `init`'s prologue |
| preamble obligation 6 / risk R-8 | All three hosts rebuilt and **run**, with no manifest asset and no build-system edit — the falsifier for DD-001's declarative-host boundary claim, which must not be inferred from "we did not edit them" | This task's own artifact |
| T3 F-21 (handoff row) | A host-package build relinks `wasamo.dll` around the **stale uplifted** rlib, silently and green with a fresh timestamp | Every host run in this task is preceded by `cargo build --release --workspace` |
| preamble §Implementation gates | Trap #4 is re-decided here explicitly, as a firing test or as a stated limit with its reason — never as an inherited "non-applicable" | Re-decided, and the pre-authorised stated-limit escape is **rejected** — see the third bullet below |
| T6 round-1 R3 | The Compositor-unavailable skip path is owed **per binary** | Two new binaries land, so two observations are owed; both are a landing blocker for one owner run |

Everything else the audit surfaced is owned elsewhere and is not T9 work: the
assistant frame captures, the positive-control pairs, the re-derived capture
coordinates and the runnable-set delivery are T10; the literal monitor
crossing is T11; the Moment 2 spec sync, the `verification-environments.md`
Observation 4 revision and the `AGENTS.md` build-ordering correction are T12.
`lib.rs::window_add_widget` remains a stated content-boundary limit, and the
stale-*receiver* hit-test case remains a documented misuse with no test.

#### What the re-audit measured before choosing an approach

The task list survives, and **five things it did not name are added to
[plan.md](./plan.md) §T9 before the gate is selected**. Four are measured
rather than reasoned, with a throwaway probe in `runtime::init()` — the
declaration and an `eprintln!` of its result, nothing else — reverted before
this gate closed. The probe's own output, recorded because the rest depends
on it: `PROBE: SetProcessDpiAwarenessContext -> Ok(())`, and
`GetDpiForWindow` then reports **120** on the development machine where it
had reported 96.

- **F-46 — the diagnostic channel the task is told to use is erased by the
  success path of the function that is told to use it.** DD-001 §Failure
  handling and [abi_spec §4.1](../../../../docs/abi_spec.md) both put the
  tolerated-failure disclosure in the thread-local last-error string. But
  `abi.rs::wasamo_init` calls `clear_last_error()` on its **`Ok`** arm, after
  `runtime::init()` has returned, so a diagnostic written inside
  `runtime::init()` is wiped before any host can read it. The naive
  implementation — write it and stop — compiles, passes every existing test,
  and ships a runtime that contradicts a normative spec section that landed
  at Moment 1. Found by reading the landing site; predicted by nothing.
  Removing the clear is the wrong fix: every `clear_last_error()` site in
  `abi.rs` is a success-path clear, and that convention is what makes
  abi_spec's "valid until the next ABI call" true. Moving it to the
  function's entry keeps the convention and stops the function discarding its
  own output. *Disposition:* [plan.md](./plan.md) §T9, as its own task item
  with a falsification build attached.
- **F-47 — the fixture breakage is one proposition across two binaries, and
  the handoff named one assertion in one file.** The proposition is *every
  DPI fixture in this phase assumes the creation-time scale is 1*. Measured
  with the probe in place, `cargo test --workspace --no-fail-fast`:

  | Binary | Result | Mechanism |
  |---|---|---|
  | `dpi_change_propagation_integration.rs` | **3 of 4 fail** | `CHANGED_DPI = 120` **is** this machine's DPI, so the change it drives is a no-op: the ratio assertions read a factor of 1 against an expected 1.25, and `assert_ne!(before_pixels, after_pixels)` fails because no re-rasterization was needed |
  | `dpi_scale_matrix_integration.rs` | **2 of 3 fail** | `os_dpi == REFERENCE_DPI` reads 120 against 96; and `row_shape(before)` reads **(5, 3)** against **(7, 2)**, because a 720-physical client at 120 DPI is 576 DIP and `floor((576 + 12) / 100)` is 5 |
  | everything else — 33 binaries, **962** tests | green | No other fixture routes a coordinate through a window's scale (T1 finding F-4, still exactly true) |

  **The test count was wrong in the first draft of this row and is corrected
  here** (T9 independent review, minor 5). It read **523**, which is a
  `wasamo-runtime` *package* figure standing beside a *workspace* binary count.
  Recounted: `cargo test --workspace` runs 974 tests today; subtracting this
  task's five additions (two new binaries with one test each, three new unit
  tests) gives 969 at probe time, and subtracting the seven tests in the two
  affected binaries gives **962** in the other 33. Two of those seven also
  passed. The conclusion the row supports — everything outside the two DPI
  binaries was green — is unchanged; the number attached to it was
  mis-scoped, in the same table whose whole point is counting.

  T8's handed-forward prediction of its own mechanism is exact. What it did
  not reach is T7's binary, which no task named and which fails harder. The
  general shape is the phase's recurring failure seen from the other side: a
  correction propagated by the **name** of the assertion rather than by the
  **proposition** underneath it stops at the file the name is in.
  *Disposition:* [plan.md](./plan.md) §T9 re-generalisation item.
- **The trap-#4 escape hatch is rejected on measurement, not taken on
  precedent.** [preamble.md](./preamble.md#implementation-gates)
  pre-authorises a stated limit "if that branch cannot be fired by a test
  because process DPI awareness is a one-shot per process". It can be fired:
  awareness is one-shot per **process**, and a test binary is a process, so a
  binary that declares its own awareness before `wasamo_init` runs anywhere
  in it makes the runtime take the real `ERROR_ACCESS_DENIED` on the shipped
  path. The pre-authorisation is exactly the shape of inherited
  non-applicability the gate exists to prevent, arriving with the gate's own
  signature on it. The residual limit is real but narrower, and is recorded
  at the close gate rather than in place of the test.
- **All four HiDpi symbols compile against the landed feature list.**
  Measured by a throwaway test target naming `SetProcessDpiAwarenessContext`,
  `GetWindowDpiAwarenessContext`, `AreDpiAwarenessContextsEqual` and
  `DPI_AWARENESS_CONTEXT_SYSTEM_AWARE`; it compiled and passed, and was
  deleted. The fourth is the one no earlier task had a reason to name — the
  tolerated-failure binary needs a *second* awareness level to pre-declare.
  `windows` is a plain `[dependencies]` entry with no separate
  dev-dependency, so a test target and the library resolve one feature list
  and the measurement covers both. **No `Cargo.toml` edit**, and nothing for
  T12's §4.5 re-sync to pick up on this account.
- **T4's creation-time correction stops being unreachable.** F-31 recorded
  that at `s = 1` a size-preserving `SetWindowPos` dispatches no `WM_SIZE` at
  all, so the ordering question T4's placement decision answers had no answer
  to get wrong before T9. This task is what gives it one, on every host on a
  scaled monitor, at a point where the window is half-constructed. That is a
  structural side effect T9 owns enumerating; "T4 argued it structurally" is
  the reason to expect it to hold, not evidence that it did.

**Two new test binaries, and the cost is accepted rather than worked around.**
DD-001's verification names two facts — the level in force, and what happens
when the declaration does not take effect — and the second needs a process
whose awareness was set before `wasamo_init`, which would make the first pass
for the wrong reason. They cannot share a process. Folding either into
`dpi_scale_matrix_integration.rs` was rejected on the terms T8 used to reject
the symmetric move into T7's binary: an ADR evidence line is easier to cite as
a named artifact than as a test buried in a file about something else. The
cost — two re-opened per-binary Compositor-unavailable observations, an owner
run and a landing blocker — is paid, as T6, T7 and T8 each paid it.

### Start gate (recorded 2026-07-31, before production-code edits)

Review lane: **full independent review**, as
[preamble.md §Review lanes](./preamble.md#review-lanes) assigns it
("process-wide platform posture + the diagnostic branch (trap #4 folded
in)"). Re-checked rather than inherited, and the re-audit strengthens rather
than qualifies it: the task also edits a shipped ABI entry point's
error-reporting path (F-46) and rewrites the assertions of the phase's two
existing DPI evidence binaries (F-47), so the lane would have been raised here
had it not already been full. The trap-#4 branch/test-focused check composes
with it per [gates §4](../../../procedures/implementation-gates.md).

| # | Applies | Reason and planned close artifact |
|---|---|---|
| 1 — semantic migration / call sites | **yes** | No enum or schema change, but `runtime::init` gains a call whose effect is process-global, and `wasamo_init`'s error-reporting contract changes. The trap's question — who else reaches this, and what does each caller mean by it — is live for both: `runtime::init()` has a second caller in `lib.rs::init` that does **not** clear last-error, and every `wasamo_*` entry point shares the thread-local the diagnostic now occupies. Close with a call-site audit of `runtime::init`, of `clear_last_error`'s success-path convention, and of the four new HiDpi symbols. |
| 2 — structural side effects | **yes** | The declaration is one line whose blast radius is the whole process: `GetDpiForWindow`, the non-client frame, `realize_dip_window_size`'s nested `WM_SIZE` (unreachable until now, F-31), text surface allocation, and real OS-delivered `WM_DPICHANGED`. Close with an enumeration of each and how it was verified, including the `0x80070005` near-miss against `tests/common/mod.rs`'s Compositor-unavailable string match. |
| 3 — parallel / derived data | **yes** | Narrow but real, and inherited non-applicability is what this phase keeps getting wrong here (F-22, F-32). The declaration creates a second reading of "what DPI is this process at": the OS's effective per-window answer, and every fixture constant derived from `REFERENCE_DPI`. The re-generalisation is what re-synchronises them; close by stating which constants stop being facts about the machine. |
| 4 — authored branch | **yes** | The gate's named substance. One authored branch lands (record the diagnostic / record nothing). Close with the pure-logic unit tests on the selector **and** the end-to-end binary that takes the real `ERROR_ACCESS_DENIED`, plus the falsification build for F-46's `clear_last_error()` move, plus the mutation re-run showing T7's and T8's generalised fixtures still go red. |
| 5 — carry-forward | **yes** | The declaration site and its one-shot, the entry-clear convention, and the "establish the before-state, never inherit it" fixture rule are invariants later tasks can trip — T10 captures frames against this posture, M4-Phase 8 puts a second window on a second monitor. Record each with evidence and a re-trigger criterion. |
| 6 — deterministic failure | **yes**, low expectation | Real windows, a live Compositor, and for the first time a *real* OS-driven scale path. Any recurring failure is rooted rather than re-rolled. |
| 7 — GUI positive control | **no** | T9 rebuilds and runs the three hosts, but the artifact is the boundary claim — that they build and run with no manifest asset and no build-system edit — not a rendered frame. Process survival plus the effective-level assertion is what that claim needs. The rendered evidence and its positive-control pairs are T10's and are not claimed here. |

The approach is therefore constrained before editing: the declaration goes
below the existing one-shot with no new guard; the diagnostic selection is a
free function over the call's `Result` so trap #4's artifact does not depend
on an OS outcome; `wasamo_init` clears at entry rather than on success; no
branch anywhere assumes scale 1 on failure; the two new binaries stay separate
processes; and no re-generalised assertion is recorded as evidence until a
mutation has been shown to break it.

### Implementation result and end gate (2026-07-31)

Landed as **one code commit**. The declaration, the `wasamo_init` entry-clear,
the two new binaries and the two re-generalised fixtures do not separate into
buildable, honestly-testable intermediate states: the declaration alone leaves
five tests red across two binaries, and the entry-clear alone is an ABI edit
with no consumer. Splitting them would put a knowingly-red tree in history to
satisfy a default the commit rules already exempt for exactly this case.

Production diff: `wasamo-runtime/src/runtime.rs` (the declaration, the
diagnostic selector, three unit tests), `wasamo-runtime/src/abi.rs`
(`wasamo_init`'s clear moves to entry), and two doc-comment corrections in
`dip_scale.rs` and `window.rs`. Test diff: two new binaries and the two
re-generalised DPI fixtures.

#### Trap 1 — call-site audit

The claim being checked is "every path that reaches the declaration, or that
reads the thread-local it now writes, is enumerated and classified".

| Call site | What it is | Classification and verification |
|---|---|---|
| `runtime::init` → `declare_per_monitor_aware_v2()` | The only production caller | **Must declare.** Below the `RUNTIME.get().is_some()` early return, above every WinRT initialisation. Verified by the level readback in `dpi_awareness_declaration_integration.rs`, and by mutation T9-M3 (moved above the guard → the second-init readback fires) |
| `abi::wasamo_init` → `runtime::init` | The ABI entry point | **Must clear on entry.** It is the only caller that clears the thread-local, and it used to do so *after* `runtime::init` returned. Verified by mutation T9-M2 |
| `lib::init` → `runtime::init` | The Rust-native entry point | **Correct unchanged, and the reason is recorded rather than assumed.** It never cleared the thread-local, so the diagnostic reaches a Rust host without any edit. It also does not *set* one on failure — that asymmetry is pre-existing (it returns `Result`, so the error is the return value) and is not widened here |
| `declaration_diagnostic` | Pure selector | Two callers: `declare_per_monitor_aware_v2` and the unit tests. No production path reads the string other than through `wasamo_last_error_message` |
| The rest of `abi.rs` | The other `clear_last_error()` sites | **Unchanged, and counted rather than asserted** — the first draft of this row said "all 24 are success-path clears" and that is false. `abi.rs` has **25** occurrences: 1 definition and **24** call sites. Of the 24, **1** is `wasamo_init`'s (the one that moves), **21** are success-path clears, and **2** are inside the `#[cfg(test)]` module and are not ABI paths at all. **Of the 21, 20 are in ABI entry points and one is in the private helper `finish_stack`** — corrected after the independent review (nit 1), which caught the first draft calling all 21 entry points. The 21 are what make abi_spec §4.1's "valid until the next ABI call on that thread" true, so the diagnostic survives exactly until the host's next ABI call — the documented lifetime, not a new one. `wasamo_init` is now the single entry-clear and its comment says why |

Symbols: `SetProcessDpiAwarenessContext` and
`DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` in production;
`GetWindowDpiAwarenessContext`, `GetThreadDpiAwarenessContext`,
`AreDpiAwarenessContextsEqual`, `DPI_AWARENESS_CONTEXT_SYSTEM_AWARE` and
`DPI_AWARENESS_CONTEXT_UNAWARE` in tests. All resolve against the landed
`Win32_UI_HiDpi` feature; **no `Cargo.toml` edit**.

#### Trap 2 — structural side effects of a process-wide posture flip

The declaration is one line and the enumeration is what says what it drags
along. Each row states how it was verified, not that it was considered.

| Effect | Verified how |
|---|---|
| `GetDpiForWindow` answers with the monitor's DPI instead of 96 | Measured: 120 on the development machine, in the throwaway probe and again in all three hosts |
| The non-client frame scales by its own DPI-indexed metrics | Measured indirectly: the hosts' client is 982 × 703 inside a 1000 × 750 outer, and `frame_thickness` is read live on every call in the T8 fixture rather than derived from a DPI |
| `window::realize_dip_window_size` stops being a no-op and the nested `WM_SIZE` F-31 recorded as **unreachable** becomes reachable | **Inferred, not measured, and the first draft of this row said "measured"** (T9 independent review, minor 6). The inference is T4's measured message set plus a correction that is no longer size-preserving. What the evidence offered — three hosts creating and rendering without incident, the suite green with every window created at 120 DPI — **cannot distinguish "the nested dispatch happened and `wnd_proc` was safely inert" from "no nested dispatch happened"**, so it is a no-early-crash signal and nothing more. **No test observes it**: `resize_fn` can only be installed after `window::create` returns, so the creation-time nested pass has no witness by construction. T4's placement decision therefore still rests on its structural argument, exactly as T4 said it must |
| Text surfaces are allocated at `ceil(dip × s)` — larger, and rebuilt on every change | The T7 and T8 surface assertions still fire; mutation T8-M4's contract (`ceil`, not truncate) is unchanged |
| The OS begins delivering `WM_DPICHANGED` for real | Not exercised here — T11's monitor crossing is what shows it. Stated rather than implied |
| **Near-miss**: `ERROR_ACCESS_DENIED` is `0x80070005`, which is the exact string `tests/common/mod.rs` matches to decide "Compositor unavailable" | Safe, and safe by a conjunct this task does not own: the helper checks `status == WASAMO_ERR_RUNTIME` first, and on that arm `wasamo_init` overwrites the diagnostic with `"wasamo_init: {e}"`. Recorded as a carry-forward, because a future change that reports a declaration failure through the *status* would make the helper skip every Compositor test on a machine whose host declared its own awareness |
| An **unaware observer cannot measure an aware window's rectangle** (finding F-48) | Measured, by getting it wrong first — see below |

#### F-48 — the observer's own awareness is part of the measurement

The first three-host run reported `outer=800x600` for every host and would
have supported the conclusion that T4's DIP-size correction never ran. It is
an artifact of the **probe**: a DPI-unaware process asking `GetWindowRect`
about an aware window is answered in *virtualized* coordinates — the real
rectangle divided by the system scale — so 1000 × 750 is handed back as
800 × 600. Declaring the probe Per-Monitor-Aware V2 and changing nothing else
turns the same three readings into `outer=1000x750  client=982x703`.

Two things follow, and the second is the one that matters beyond this task.

- The rectangle numbers in the host artifact are only meaningful because the
  observer declares V2. `GetDpiForWindow` and
  `GetWindowDpiAwarenessContext` are **not** virtualized, so the level
  readback was correct in both runs; it is the coordinates that moved.
- **Every assistant-side measurement of a Wasamo window from now on inherits
  this**, which puts it squarely in T10's path: a capture or coordinate
  derivation performed by an unaware tool reads a consistent, plausible, and
  wrong rectangle. Carried forward with a re-trigger criterion rather than
  left in this task's prose.

The general shape is the phase's own lesson arriving from a new direction: the
first reading was not noise and was not obviously wrong — it was internally
consistent and off by exactly the scale factor, which is the signature this
whole phase exists to remove.

#### Trap 3 — the constants that stopped being facts about the process

The declaration creates a second reading of "what DPI is this process at", and
the fixtures held the old one. What was re-synchronised:

| Constant / assumption | Was | Is |
|---|---|---|
| `dpi_change_propagation_integration::CHANGED_DPI = 120` | A step up from an assumed 96 baseline | Deleted. The target is `changed_dpi(window)` = twice the window's **committed** creation DPI, so the factor is exactly 2 on every machine |
| `dpi_change_propagation_integration::CHANGED_FACTOR` | `120 / 96` | `2.0`, and exact at every magnitude |
| `dpi_scale_matrix_integration`'s before-state | Inherited from creation, assumed `s = 1` | Established by `normalise_to_reference_baseline`, which synthesises one `WM_DPICHANGED` to 96 realising the chosen physical client, and **asserts both halves** |
| `a_created_windows_cached_scale_is_the_dpi_the_os_reports`'s `os_dpi == 96` | The assertion whose stated job was to fail at T9 | Replaced by the awareness-level precondition. No number works: 96 is correct on a 100% monitor, so `os_dpi != 96` would redden a correct build on CI |
| `CLIENT_W`'s "the created client is 784 × 561 physical at 96 DPI" | A fact about the created window | Kept as a fact about a 96-DPI display, with the note that since T9 the created client is not a fixed number at all |

The two fixtures are now **posture-independent**, which mutation T9-M4
measured rather than claimed: with the declaration deleted entirely, all four
T7 tests and two of three T8 tests still pass, and the two that fail are the
awareness assertions that should. Before this task the same mutation would
have been indistinguishable from the shipped state.

#### Trap 4 — the authored branch, and every claim shown to be falsifiable

One authored branch ships: record the diagnostic, or record nothing. It is
fired in both directions by unit tests on `declaration_diagnostic`, and the
recording direction is fired end-to-end, in a real process, against a real
`ERROR_ACCESS_DENIED`, by
`dpi_awareness_tolerated_failure_integration.rs`.

**T9's own mutations.** Every row was run; none is predicted.

| # | Mutation | Result |
|---|---|---|
| T9-M1 | `declaration_diagnostic` always returns `None` | **2 of 3 unit tests fail** (the `Ok` one passes, correctly), and `a_host_that_already_declared...` **fails** at the disclosure `expect`. The declaration binary passes — the level is unaffected, which is the point of separating them |
| T9-M2 | `wasamo_init` clears on the `Ok` arm — i.e. F-46 unfixed, the shape a literal reading of the plan would have shipped | `a_host_that_already_declared...` **fails**, reading `None` where a diagnostic is required. **This is the falsification build the plan demanded**: the claim "the clear must move or the disclosure is unreachable" is measured, not argued |
| T9-M3 | The declaration moved **above** the `RUNTIME.get().is_some()` early return | `a_windows_effective_awareness_context...` **fails** on the second-init readback, with exactly the predicted symptom: the process takes `ERROR_ACCESS_DENIED` against its own earlier correct declaration and reports it. The level in force stays V2 and the call still returns `WASAMO_OK`, so **without the readback this mutation is invisible** |
| T9-M4 | The declaration removed entirely (the pre-T9 posture) | Declaration binary **fails** (`unaware: true`); tolerated-failure binary **fails**; T7's four and T8's other two **pass**, which is the posture-independence claim above, measured |

**T1's F-10 was a structural claim with nothing observing it.** "The existing
one-shot is sufficient; T9 adds no new guard" was verified at T1 against the
source and then carried for eight tasks with no readback. T9-M3 is what turns
it into a falsifiable one, and the readback had to be *added* for the
mutation to have anything to break — which is the "if you say the structure
protects it, run a mutation that breaks the structure" discipline finding its
gap rather than confirming its absence.

**Inherited mutations, re-run against the re-generalised fixtures.** The
hazard the plan named is that this task defangs the phase's central evidence
while calling it a fixture fix. Six were re-run; all still fire, and one fires
harder.

| # | Mutation | Result on the generalised fixture |
|---|---|---|
| T8-M1 | Inbound client-extent seam removed from the `WM_SIZE` arm | `dip_layout_...` **fails** (unchanged) |
| T8-M2 | `visual_rect_dip` divides by each node's own scale | `a_stale_descendant_...` **fails**, `0` clicks against `1`, control click passing (unchanged) |
| T8-M3 | `sync_visuals` writes sizes at `DipScale::IDENTITY` | `dip_layout_...` **fails** (unchanged) |
| T8-M7 | `begin_scale_change` does not commit the scale | **All three** T8 tests fail and **three of four** T7 tests fail, on the 120-DPI development machine. **Qualified after the independent review (major 3): that breadth is a property of this machine, not of the fixtures.** `normalise_to_reference_baseline` asserts the committed scale is 96, which on a 96-DPI display is the value the window was created with — so it passes whether or not the commit happened. Re-measured **by this author rather than taken from the review**, with the creation DPI forced to 96: the mixed-scale test **passes** under M7, exactly as it did at T8. "Broader, not narrower" was true of the run and false as a claim about the change |
| T7-M3 | The fallback removed | The two fallback tests **fail**; the nested-path and both-fail tests pass (unchanged) |
| T7-M5 | `SWP_NOMOVE` inherited from the creation-time correction | `a_size_changing_suggested_rectangle...` **fails** (unchanged) |

Not re-run, stated rather than implied: T8-M4 (`surface_pixels` truncates),
T8-M5 (the non-multiple-of-24 client), T8-M6 (M3 restricted to 100 DPI),
T7-M1, T7-M2, T7-M4 and T7-M6. Each is about arithmetic or a branch the
generalisation does not touch — the generalisation changed *which DPI is
targeted* and *how the before-state is reached*, and the six re-run are the
ones whose sensitivity depends on either. That is a judgment, and it is
recorded as one.

#### The propagation pass — added after the independent review, which is itself the finding

**This section did not exist when the end gate was first recorded, and the
plan's end gate listed it as owed** ("the propagation pass with its enumerated
asserting sites"). What existed was the *output* — two corrected doc comments —
and one line in the retrospective naming them. That is precisely the abstract
"checked" [gates §2](../../../procedures/implementation-gates.md) refuses: a
reviewer cannot audit an enumeration that was never written down.

Worse, the pass had **missed a site**, and the reviewer found it where the
plan's own falsifiable test says it will be looked for: *"falsified if a
reviewer again finds an asserting site the pass never visited."* The missed
site is `window.rs`'s `realize_dip_window_size` doc comment — **in the same
file as a site the pass did correct**. The pass propagated by the *name* T9
(it edited the comment whose text mentioned T9) rather than by the
proposition. That is the phase's signature failure for the ninth time,
occurring inside the remediation written for it, which is the third time a
remediation has done that.

The enumeration, written now and audited against ground truth:

**Proposition P1** — *"the process has not declared DPI awareness, so the OS
reports 96 for every window and every scale factor is 1."*

| Asserting site | Verdict |
|---|---|
| `tests/dpi_scale_matrix_integration.rs` header, stated limit 3 | **Corrected** — rewritten to say the messages are synthesised by choice, not by necessity |
| `tests/dpi_change_propagation_integration.rs` header | **Corrected** — same |
| `tests/dpi_scale_matrix_integration.rs`, test 1's assertion and doc | **Corrected** — replaced, and the replacement qualified again after review major 1 |
| `src/dip_scale.rs`, `IDENTITY` doc | **Corrected** — "until T9, on every process" was present tense and false |
| `src/window.rs`, `WM_DPICHANGED` handler doc | **Corrected** — re-tensed |
| `src/window.rs`, `realize_dip_window_size` doc, the nested-`WM_SIZE` sentence | **MISSED by the pass; corrected at review remediation.** Asserted "no `WM_SIZE` is dispatched at all, so this placement is unverifiable until the awareness declaration lands" — the identical proposition the trap-2 row declares falsified |
| `src/window.rs`, same doc comment, the no-guard paragraph | **MISSED by the pass; corrected at review remediation.** "a branch that no test can fire until the declaration lands" |
| `src/window.rs`, `realize_dip_window_size` failure note | **No edit needed**, checked: conditional in both directions ("an aware process… while an unaware one…"), true whatever the posture |
| `plan.md` §Task list intro; `preamble.md` §The sequencing thesis | **No edit needed**, checked: scoped to what T2–T8 landed into, and historically accurate |
| `docs/abi_spec.md` §4.1; `docs/architecture.md` §12 | **No edit needed**, checked: both conditional ("including none, where the effective DPI is 96") |
| `log.md` §T1–§T8, `retrospectives/t1..t8.md` | **Must not be edited** — historical record of what was true when written |

**Proposition P2** — *"a created window's scale is 1, so the creation-time
correction is an identity and dispatches no `WM_SIZE`."*

| Asserting site | Verdict |
|---|---|
| `src/window.rs`, `realize_dip_window_size` doc | **MISSED; corrected** — the same site, which is why one proposition-first pass over both propositions would have caught it |
| `preamble.md` R-9 | **Updated** — the residual it named is discharged, with the measurement |
| `plan.md` §T4 "Inertness holds"; `preamble.md` F-31 paragraph | **No edit needed**, checked: both are records of what T4 measured, correctly tensed |
| `handoff.md`, `realize_dip_window_size` row | **No edit needed**, checked: a re-trigger criterion, not a claim about the current posture |

**What the miss says about the method.** Enumerating *documents* was not
enough, because the missed site is a doc comment inside a source file, and the
first pass's search was seeded from hits for the string `T9` and for "unaware".
The sentence carrying the proposition contains neither. The rule the plan
already states — write the proposition, then enumerate what asserts it, then
search — was followed for the first two steps and abandoned at the third, where
the enumeration was allowed to become whatever the search returned. **The
enumeration has to name the source files by responsibility before the search
runs**, or the search silently defines the scope.

#### Trap 5 — carry-forward

Recorded in [handoff.md](./handoff.md) with re-trigger criteria: the
declaration site and its one-shot; `wasamo_init`'s entry-clear and the
diagnostic's lifetime; the `0x80070005` collision with the test harness's
Compositor-unavailable match; the "establish the before-state, never inherit
it" fixture rule; and F-48's unaware-observer trap, which T10 consumes
immediately.

#### Trap 6 — deterministic failure

No flaky or recurring failure. The five red tests the probe produced were
deterministic, reproduced on every run, root-caused to one proposition
(F-47), and fixed rather than re-rolled. The one misleading measurement
(F-48) was likewise deterministic and was resolved by finding the cause, not
by re-running it.

#### The three-host rebuild — DD-M4-P1-001's boundary falsifier

Preceded by `cargo build -p wasamo-runtime --release` and
`cargo build --release --workspace`, per F-5 and F-21; the C and Zig build
directories were **deleted** first, so neither reused a cached artifact.

- `examples/counter-c` — `cmake -S . -B build` + `cmake --build build --config Release`: clean, `counter.exe` produced.
- `counter-rust` — built by the workspace release build.
- `examples/counter-zig` — `zig build` after removing `zig-out` and `.zig-cache`: clean.

**No manifest asset and no build-system edit.** Audited rather than asserted:
no `.rc` or manifest **source file** anywhere under `examples/`, no `dpiAware`
in any of them, and the only match for "manifest" is `CARGO_MANIFEST_DIR`, a
cargo path variable. `git log` puts the last change to `examples/` at
`f3ccaef` (M3-Phase 8) — before this phase opened.

**One correction to how that was first worded, and the replacement is measured
rather than inferred** (independent review, nit 5). The first draft said "no
`RT_MANIFEST` anywhere under `examples/`", which is false of the *built*
binary. Checked at the byte level rather than reasoned about, across all three:

| Host binary | embedded manifest | `dpiAware` | `dpiAwareness` |
|---|---|---|---|
| `counter-c/build/Release/counter.exe` | **yes** (MSVC's default `asInvoker` trustInfo manifest) | no | no |
| `target/release/counter-rust.exe` | no | no | no |
| `counter-zig/zig-out/bin/counter-zig.exe` | no | no | no |

So one of the three does carry an embedded manifest — the toolchain's, not the
host's — and none of the three carries a DPI element. The claim DD-M4-P1-001
makes is that no host ships a manifest *asset of its own* or gains a build
step, and that survives.

**The first replacement wording for this was also over-strong and is not what
is written above.** It said the absence of `dpiAware` is "what the level
readback proves". It is not: a host manifest declaring V2 would produce the
identical V2 reading, which is the stated limit already recorded for the
declaration binary. What actually establishes it is the byte-level check above,
plus T1's and T4's measurements of these same hosts as **unaware** before T9 —
which no host-side declaration could have permitted. Recorded because a false
universal in the middle of an audit table is what this phase keeps shipping,
and because catching one in the correction for another is the pattern the
reviewer named.

All three launched and were asked what level is in force in their own process,
by an observer declared Per-Monitor-Aware V2 (F-48):

```
counter-c      level=PER_MONITOR_AWARE_V2   GetDpiForWindow=120  outer=1000x750  client=982x703
counter-rust   level=PER_MONITOR_AWARE_V2   GetDpiForWindow=120  outer=1000x750  client=982x703
counter-zig    level=PER_MONITOR_AWARE_V2   GetDpiForWindow=120  outer=1000x750  client=982x703
```

The probe is [evidence/probe-t9-hosts.ps1](./evidence/probe-t9-hosts.ps1),
landed after the independent review (nit 4) pointed out that three pasted lines
with no recorded source cannot be re-audited — which matters more than usual
here, because F-48 means the numbers depend on a property of the probe itself.
Re-run from the committed script, it reproduces the three lines above exactly.

**Why the level readback and not just "they ran".** F-9 recorded that
1000 × 750 is *also* what an unaware process produces, because DWM stretches
the logical rectangle by the same factor — so the rectangle alone is satisfied
by a build that declares nothing, and "all three hosts still build and run" is
a claim three unaware processes would also satisfy. The level is what makes
the run a falsifier. The rectangle is a second fact of a different shape
beside it, and it independently agrees with T4's throwaway-probe measurement
of `982 × 703`, taken two days earlier through a different route.

#### Stated limits

1. **One process observes one declaration outcome.** A process that watched
   the runtime declare successfully can never watch it fail, and the
   tolerated-failure binary can never watch it succeed. The two halves of the
   branch are two artifacts and no run asserts both. What does not inherit
   this is `declaration_diagnostic`, which is why the selection was extracted.
2. **The declaration binary shows the level is in force, not that the runtime
   put it there.** In that process nothing else declares anything, so the
   runtime is the only candidate — a property of the fixture. The sibling
   binary is what distinguishes "declared" from "found already declared".
3. **The OS is still not what drives `WM_DPICHANGED` in any test here.** T9
   makes the OS capable of it; T11's monitor crossing is what shows it. The
   preamble's obligation-5 limit is unchanged by this task.
4. **The host artifact is not GUI evidence.** It says the three hosts build,
   run, and carry the declared level. It says nothing about what they
   rendered; that is T10's, with the positive-control pairs.

#### Local gates

- `cargo build -p wasamo-runtime` → `cargo build --workspace` → `cargo test --workspace --no-fail-fast`: **37 test binaries, every one `ok`**, 0 failed. Up from 35 binaries at T8; the two new ones are this task's.
- `cargo build --release --workspace`: clean.
- `cargo fmt --all -- --check` and `git diff --check`: clean.
- The throwaway probes — the declaration `eprintln!` and the four-symbol
  compile check — are reverted; `git status` carries only the intended diff.

#### Landing blocker — **closed 2026-07-31 by owner run, after one round of repair**

Both binaries print their named skip line and pass on a session where
`wasamo_init` returns `0x80070005`:

```
running 1 test
skipping effective DPI awareness level: runtime compositor unavailable
test a_windows_effective_awareness_context_is_per_monitor_aware_v2 ... ok

running 1 test
skipping tolerated declaration failure: runtime compositor unavailable
test a_host_that_already_declared_keeps_its_level_and_is_told_ours_did_not_take ... ok
```

**Which build ran is established rather than assumed**, and the obvious check
does not establish it. Cargo's test-executable filename hash
(`...-1b13d4adc4b6727c.exe`) is derived from package / target / profile
metadata, **not from content**, so it was byte-identical across the F-49 repair
and cannot distinguish the two versions — a freshness signal that looks
authoritative and is not, in the family of F-5 and F-21. What does establish
it is the output itself: the pre-repair code panicked at its `expect` **before**
reaching `run_on_owning_runtime_thread_or_skip`, so it could not print
`skipping tolerated declaration failure` under any circumstances. That line is
in the run. The repaired code is what executed.

**The first run of this gate is what produced F-49** — the observation was
owed twice, was run once, failed once, and closed on the second run against
the repair. Recorded that way rather than as "closed", because a gate that
found a defect on first execution is the evidence that the gate is worth
having, and this phase has three prior instances where it found nothing.

**It also verified the repair in its target environment**, which the local
simulation could only approximate: the simulation forced the pre-declaration
to lose on a machine with a live Compositor, so it exercised the assertion
path; this run exercised the skip path on a machine where the awareness was
already set *and* the Compositor was absent. Both halves of the repair are now
measured, on the environments that distinguish them.

---

*The paragraphs below are the pre-run record, kept as written.*

Two new binaries, so the per-binary Compositor-unavailable observation is owed
twice (T6 round-1 R3). **Open at the time of writing.** The run is the
owner's, on a session where `wasamo_init` returns `0x80070005`, and the fix
container if a guard did not fire is `feat/m4-phase-1-t9` before merge — an
additive commit there, not a follow-up task. The expected result is that
`a_windows_effective_awareness_context_is_per_monitor_aware_v2` and
`a_host_that_already_declared_keeps_its_level_and_is_told_ours_did_not_take`
each print their named skip line and pass.

**One of the two has a wrinkle worth stating in advance rather than
discovering during the run.** The tolerated-failure binary calls
`SetProcessDpiAwarenessContext` *before* the skip decision is reached, because
the pre-declaration must precede `wasamo_init`. That call does not need a
Compositor and will succeed on a session that has none, so the test still
reaches the helper and still skips — but it means the binary does OS work
outside the guard, which no other binary in the suite does.

#### F-49 — the wrinkle was real, the prediction attached to it was wrong, and the owner run is what said so

**The paragraph above is preserved as written and is falsified in its second
half.** Identifying the hazard ("this binary does OS work outside the guard")
was correct. Predicting it was harmless — "that call does not need a
Compositor and will succeed on a session that has none" — was **wrong, and it
was reasoned rather than measured**, in a paragraph whose whole purpose was to
say what might go wrong.

**Owner run, 2026-07-31.** On the guard-verification session,
`a_host_that_already_declared_keeps_its_level_and_is_told_ours_did_not_take`
**failed** rather than skipping:

```
panicked at dpi_awareness_tolerated_failure_integration.rs:92:10:
the test host declares its own awareness first; nothing has set it yet:
Error { code: HRESULT(0x80070005), message: "アクセスが拒否されました。" }
```

The pre-declaration returned `ERROR_ACCESS_DENIED` — the process's awareness
had **already been set, before a line of test code ran**. So the binary failed
on exactly the environment where every other binary skips, which is the defect
[AGENTS.md §Testing rules](../../../../AGENTS.md) requires this observation to
catch. It caught it on the first run.

**The cause is environmental and that much is established rather than
inferred**: cargo reported `Finished in 0.06s` and ran
`...-1b13d4adc4b6727c.exe`, the **same on-disk artifact** as every local run,
so the difference between passing here and failing there is the session and
not the build. **The mechanism is not identified and is not claimed.** The
owner notes the account is a non-administrator, which is a plausible line —
per-user AppCompat layers live in `HKCU` and need no elevation — but nothing
here measures it, and `SetProcessDpiAwarenessContext` needs no privilege of
its own. It is left as an open observation.

**The fix does not depend on identifying the mechanism, which is the point.**
The test's premise was written as a claim about *code ordering* — "this binary
holds exactly one test, so the ordering is not a race but a sequence" — and
that is true and irrelevant: it establishes that no *test code* set the
awareness, and says nothing about the OS, the loader, or a compatibility shim.
The `expect` turned an assumption about the environment into a hard failure.
The corrected shape:

- the pre-declaration is attempted and **its result is discarded**;
- the awareness actually in force is **read back** before `wasamo_init`;
- **every assertion moved behind the skip guard**, where the rest of the suite
  already puts them;
- the premise is asserted from the readback (the level must not already be V2,
  or deferring and overriding look the same), and property 3 is stated against
  whatever level was in force rather than against `SYSTEM_AWARE` specifically
  — because *who won the race* was never what the test is about.

**Verified in both directions rather than assumed.** The owner's condition was
simulated locally by setting `DPI_AWARENESS_CONTEXT_UNAWARE` immediately before
the test's own pre-declaration, so that pre-declaration loses with
`ERROR_ACCESS_DENIED` exactly as it did on the owner's session: the test
**passes**, and passes by asserting rather than by skipping, since the local
Compositor is live. So the correction is not merely "no longer fatal" — the
test is *correct* in the world where something else declared first. And the
branch still fires: mutations T9-M1 and T9-M2 were re-run against the
restructured test and both still turn it red.

**What this says about the earlier work.** The start gate's trap-#2 row
enumerated what the declaration drags in process-wide and got the near-miss
against `tests/common/mod.rs` right. This is the same class one step further
out — an OS-global property that something *outside the process's own code*
can already have decided — and the task reasoned about it instead of measuring
it, in the one place it had explicitly noticed the risk. **Noticing a hazard
and then predicting it away is worse than not noticing it**, because the
prediction is what stops the check from being run.

### T9 independent review — disposition (2026-07-31)

Full independent review per the lane table. The reviewer re-ran four of T9's
own mutations and reproduced F-47 from the pre-T9 fixtures.

**Every finding acted on below was re-verified by this author before the
correction was written**, rather than accepted on the reviewer's account —
because the corrections themselves make claims, and a correction built on an
unverified report is the same defect one level out. Specifically: major 1 was
reproduced (seed replaced by `DipScale::IDENTITY`, declaration removed → the
test fails **only** at `is_v2` while `cached == os_dpi` passes); major 3 was
re-measured (creation DPI forced to 96 → the mixed-scale test passes under
T8-M7, as at T8); nit 1 was recounted by walking each of the 24 call sites back
to its enclosing function (20 `extern "C"` entry points, 1 private helper, 2 in
`#[cfg(test)]`, 1 `wasamo_init`); nit 5 was checked at the byte level in all
three built host binaries. Nit 3 was confirmed by reading the two spec
sections. The rest are corrections to prose whose ground truth is the diff.

| # | Severity | Finding | Disposition |
|---|---|---|---|
| MAJOR 1 | major | "The creation-time half stopped being degenerate at T9" is false on a 96-DPI runner. `cached == os_dpi` is satisfied by a seeding that ignores the OS entirely, because `DipScale::IDENTITY.dpi()` **is** 96, and the awareness precondition is independently true there | **Confirmed, and reproduced independently.** With `WindowState`'s seed replaced by `DipScale::IDENTITY` and the declaration removed, the test fails **only** at the `is_v2` assertion; `cached == os_dpi` passes. On a real 96-DPI runner `is_v2` is true, so the test would be green with the seeding broken. The doc comment is rewritten to say what the precondition does buy and what it does not, and the module header gains a stated limit: **the seeding path has no CI coverage from this test, and none is available** — at 96 DPI "seeded from the OS" and "seeded from the identity" are the same number |
| MAJOR 2 | major | The propagation pass produced no enumeration artifact, and missed an asserting site — `realize_dip_window_size`'s doc comment, in the same file as a site the pass did correct | **Confirmed on both halves.** The end gate listed the enumeration as owed and shipped only the output; a §The propagation pass section is added with both propositions and every asserting site classified. The two missed sentences are corrected. This is the phase's signature failure for the **ninth** time and the **third** time inside a remediation for it, which is the finding that matters more than the two comments |
| MAJOR 3 | major | "Broader, not narrower" for T8-M7 is a 120-DPI result presented as a property of the fixtures. `normalise_to_reference_baseline`'s scale assertion is `96 == 96` on a 96-DPI runner and passes whether or not the commit happened | **Confirmed.** The mutation row is qualified to say the breadth belongs to this machine, and the vacuity is recorded as a stated limit on the helper and in the module header. The claim the header does keep — that the *arithmetic* is machine-independent — survives, because the DIP client is 720 × 480 either way |
| MINOR 4 | minor | `bindings/c/wasamo.h` still documents last-error as "the most recent non-OK status", which T9 carves out. It is the only place a C or Zig host learns the semantics, and it is named by no phase task | **Confirmed and fixed here.** One paragraph mirroring abi_spec §4.1's carve-out. Mechanical transcription of an Accepted DD, so it is not a spec change under the retrospective's item-2 rule. The reviewer independently checked all six example hosts and confirmed none reads last-error on an OK status |
| MINOR 5 | minor | "the workspace's other 523 tests" pairs a `wasamo-runtime` *package* count with a *workspace* binary count | **Confirmed, recounted, corrected to 962.** `cargo test --workspace` runs 974 today; minus T9's five additions and the seven tests in the two affected binaries. The conclusion is unchanged; the number was mis-scoped in the table whose point is counting |
| MINOR 6 | minor | "Now measurable, and measured" for the creation-time nested `WM_SIZE` is backed by an absence of failure. Nothing observes the dispatch — `resize_fn` can only be installed after `create` returns | **Confirmed.** The trap-2 row is rewritten to say *inferred*, from T4's measured message set plus a correction that is no longer size-preserving, and to state that the green suite is a no-early-crash signal only. The same correction is made in the `realize_dip_window_size` doc comment |
| MINOR 7 | minor | "each test puts the window into the before-state it assumes" is false — 2 of 3 do, and the next sentence concedes it | **Confirmed and fixed**: "the two tests that measure a scale change" |
| NIT 1 | nit | Of the 21 success-path clears, one is in the private helper `finish_stack`, not an entry point | **Confirmed** (read the enclosing function) **and corrected**: 20 entry points + 1 private helper |
| NIT 2 | nit | "like every other entry point in this file" — `wasamo_last_error_message` clears nowhere and `wasamo_shutdown` has no status arm | **Confirmed and narrowed** to "the status-returning entry points" |
| NIT 3 | nit | architecture.md §12 and abi_spec §4.1 say "first act"; the declaration is the first *OS-touching* act and does not run on a second `wasamo_init` | **Confirmed.** Normative-spec wording is Moment 2's, so it is filed as a **fifth T12 divergence item** rather than edited here. The retrospective's "no spec change" was right about §4.1's diagnostic contract and silent about this clause |
| NIT 4 | nit | The three-host artifact is three pasted lines with no recorded probe source, which matters because F-48 made the numbers depend on a property of the probe | **Confirmed and fixed**: [evidence/probe-t9-hosts.ps1](./evidence/probe-t9-hosts.ps1), with the F-48 reason in its header. Re-run from the committed script it reproduces the three lines exactly |
| NIT 5 | nit | "no `RT_MANIFEST` anywhere under `examples/`" is false of the built exe — MSVC embeds a default manifest | **Confirmed and corrected** to "no `.rc` or manifest *source file*". The substance is unchanged and is what the level readback proves: the toolchain's default manifest carries no `dpiAware` |

**What the reviewer checked and found sound**, recorded because coverage is
part of the artifact: all four T9 mutations re-run and reproduced exactly;
F-47 reproduced from the pre-T9 fixtures; the placement argument and its
readback; the `clear_last_error()` move audited across all 24 call sites and
every `runtime::init` caller and all six example hosts; the `0x80070005`
near-miss analysis and both its conjuncts; the two new binaries shown unable
to pass for a wrong reason; the T7 re-generalisation shown to weaken nothing;
the three-host artifact accepted as a genuine falsifier; and the trap-#4
disposition, including the judgment that rejecting the pre-authorised stated
limit was correct. The reviewer's objection there was that limits were
**missing** (majors 1 and 3), not that a stated one was wrong.

**Post-remediation verification, on the final branch state** (the
[retrospectives.md](../../../procedures/retrospectives.md) item-3 rule that a
remediation landing after the retrospective invalidates the recorded gate run):

- `cargo fmt --all -- --check` and `git diff --check` — both exit 0.
- `cargo test --workspace --no-fail-fast` — **37 binaries, 974 tests, 0 failed.**
- `git status --porcelain` — empty; every mutation and probe reverted.
- The remediation changes **no executable code**: a diff of `wasamo-runtime/`
  filtered to non-comment lines is empty. Every source change is a doc comment
  or a comment, plus one comment block in `bindings/c/wasamo.h`. So the clean
  rebuild recorded above is not stale in the way item 3 guards against — but the
  suite was re-run against the final state anyway rather than argued about.

**The pattern, stated rather than left implicit.** Three of the four
substantive findings — majors 1 and 3, minor 6 — are the same defect: a claim
that is true of the run that produced it and false as a claim about the
change. Major 1 and major 3 are both instances of a sharper version: **an
assertion that discriminates on the 120-DPI development machine and is vacuous
at 96 DPI**, presented as a property of the test. The phase has recorded this
class eight times before T9 and it arrived twice more here, both inside work
whose stated purpose was to make fixtures machine-independent. The corrective
this task adds to the record: **when a fixture is claimed to be
machine-independent, the claim has to be evaluated at each machine the fixture
will actually run on** — for this repository that is 96 (CI) and 120 (dev), and
an assertion is only as strong as its weakest one.

---

## T10 — Assistant GUI evidence (positive controls A, B, and C's path form)

### Carry-over audit and responsibility re-audit (2026-08-01, before start gate)

Task branch `feat/m4-phase-1-t10`, cut from `feat/m4-phase-1` = `1b3ee59`.

**Carry-over into T10, read out of the T1–T9 retrospectives and
[handoff.md](./handoff.md) rather than out of §T10 alone.** Nine items name
T10 as a consumer; each is listed with what it obliges here.

| From | Proposition | Obligation on T10 |
|---|---|---|
| F-5 / F-21 (T1, T3) | A host-package build relinks `wasamo.dll` around a stale uplifted rlib, silently and green | `cargo build -p wasamo-runtime --release` then `cargo build --release --workspace` before **every** capture |
| F-40 (T6) | Two source trees sharing one cargo target directory make cargo report the wrong one fresh | The base-commit tree gets its own `CARGO_TARGET_DIR`; every mutation run ends with a package clean and an accepted-source rebuild |
| F-33 (T5) | A committed frame set is not a baseline, and one capture is not a baseline | Re-capture both sides in the session that compares them; agree ≥2 captures per side; `compare-frames.ps1` exits non-zero on any difference and its delta classifies nothing |
| F-28 / P3 (T4) | The client rectangle does not scale by `s` | Control B is driven from a controlled **client** rectangle, not a controlled outer one |
| T8 | A DIP client that is a multiple of 24 is an integer physical size at 96 / 120 / 144 / 192 | Control B's target is 960 × 576 DIP |
| F-27 (T4) / T5 | Rows 1 and 3 of the three-state table share a rectangle, rows 1 and 2 share a tile count; the third row's 9 is the pre-T5 signature and must now read 7 | The measurement check reports the **pair**, and does not inherit 9 |
| F-34 (T5/T6) | Between T5 and T6 the tree occupies 1/1.25 of the client, and after T6 it must not | A capture that still looks small after T6 is a defect, not a known intermediate |
| F-48 (T9) | An unaware observer reads an aware window's rectangle in virtualized coordinates | Every measuring script declares PMv2 **and says so in the artifact** |
| F-49 (T9) | A call that arranges OS state may fail; the arrangement is not the same fact as the result | The scripts also **read the level back** and print it, rather than trusting the call |

**Responsibility re-audit — what T10 should be, not what §T10 said.**
Three of §T10's items did not survive contact with the current state of the
phase, and the plan was revised before any capture was designed.

1. **The window-measurement bullet was written as T10's closure of risk
   R-9, and R-9 has been closed since T9.** The preamble records it closed;
   the plan did not. What is genuinely open is narrower and is now stated
   narrowly: T9 probed the three **counter** hosts, and tiles-per-row is a
   **gallery** property, so the gallery's signature has only ever been taken
   under a *throwaway* declaration (T4, T5). T10 takes it against the landed
   one. Saying "T10 closes R-9" would have been the phase's own recurring
   defect — a claim wider than its object — in the task whose whole content
   is evidence.
2. **Control A's "before" was never defined**, and the two candidate
   referents answer different questions. The phase-level pair (base commit
   vs branch tip) is what risk R-1 is about; the posture pair (one binary,
   declaration allowed vs denied) is what isolates the declaration. Both are
   run, and neither is described as doing the other's work.
3. **Controls B and C assumed a display-scale change this machine cannot
   make.** Measured rather than assumed — see the start gate below. B is
   restructured onto a mechanism that exists; C is raised to the owner
   instead of being quietly downgraded or quietly forced.

**Measurements taken before the plan was revised** (feasibility, not
evidence — the evidence captures come after the start gate):

- `gallery-rust.exe` at the branch tip, probed from a PMv2 harness that
  reads its own level back: no environment override gives `level=PMV2
  GetDpiForWindow=120 outer=1000x750 client=982x703`, matching T4's and
  T9's independent measurements. With `__COMPAT_LAYER=DPIUNAWARE` in the
  child environment it gives `level=UNAWARE GetDpiForWindow=96
  outer=1000x750 client=980x701`. The window comes up in both, so the
  AppCompat shim gives a scale-1 run of the **shipped bytes** and
  simultaneously exercises DD-M4-P1-001's tolerated-declaration-failure
  path in a real host.
- Display scale is not changeable here. One monitor, RDP session on
  `Microsoft Remote Display Adapter`, physical 2452 × 1291 against logical
  1962 × 1033.
  `DisplayConfigGetDeviceInfo(DISPLAYCONFIG_DEVICE_INFO_GET_DPI_SCALE)`
  returns `ERROR_GEN_FAILURE` (31) for the one active source.
  `HKCU\Control Panel\Desktop` carries `Win8DpiScaling = 0`, no `LogPixels`,
  and no `PerMonitorSettings` subkey. `SPI_GETLOGICALDPIOVERRIDE` reads 0.
  A **no-op** `SPI_SETLOGICALDPIOVERRIDE(0)` returns TRUE — recorded as
  "the entry point exists", **not** as "a real change would work". The
  distinction is F-49's: arranging OS state and succeeding at it are
  different facts, and this task will not write the second one down until
  something measures it.

### Start gate (recorded 2026-08-01, before any capture or script)

Review lane: **full independent review**, as
[preamble.md §Review lanes](./preamble.md#review-lanes) assigns it
("GUI-render evidence"). Re-checked rather than inherited and unchanged: the
task's entire deliverable is rendered frames and the claims drawn off them.

| # | Applies | Reason and planned close artifact |
|---|---|---|
| 1 — semantic migration / call sites | **no** | T10 lands no production code, no enum, no schema and no traversal, so there is no call site to classify. The analogous "did the enumeration cover everything" question is real here but lands on prose, and is carried by the trap-#2 row below rather than being smuggled in under this one. |
| 2 — structural side effects | **yes**, in trap #3's documentation sense | No runtime state changes, but T10 produces derived prose that restates measurements owned by §T4, §T5, §T9 and the ADR set — exactly the second-source-of-truth analogue. Close by citing the owning document for every inherited number and marking which numbers this task measured itself. |
| 3 — parallel / derived data | **yes** | The phase has narrowed an inherited "no" here twice (F-22 at T3, F-32 at T5) and is not inheriting a third. The live instance is documentary: `evidence/README.md` is a parallel index of which frame set evidences which claim, and it goes stale the moment a set is added without updating it — which is what made T3's `after/` a trap. Close by updating the README in the same commit as the frames. |
| 4 — authored branch | **no** | No reject, diagnostic or size branch is authored. The scripts' own error paths are not product branches. **Not inherited**: re-decided here, and it is a real "no" rather than a formality because T10 adds no code the product runs. |
| 5 — carry-forward | **yes** | Two carriers are already visible: the re-derived capture coordinates that T12 turns into `verification-environments.md` Observation 4, and the runnable-set delivery that T11 consumes. Record each with a re-trigger criterion. |
| 6 — deterministic failure | **yes** | Named in the gate line because this task's failure mode is *re-shooting*. A capture that disagrees with a number recorded at T4 / T5 / T9 is a deterministic failure to root-cause; F-33 measured a real drift band, and the temptation is to file any disagreement into it. The rule for this task: a disagreement is rooted, and the drift band is only invoked for differences whose shape was measured to match it. |
| 7 — GUI positive control | **yes** | The gate's substance. Every control ships with the run that shows it can fail: A with the posture pair and the phase-level pair, B with the inbound-seam mutation whose signature T5 measured (9 tiles against the accepted 7). |

The approach is constrained before any script is written: every harness
declares PMv2 and prints its own readback; the base tree builds under a
separate `CARGO_TARGET_DIR`; controls A2 and B use `__COMPAT_LAYER` rather
than a source mutation, so only control B's falsification run is a mutation
build at all; and control C is raised to the owner rather than resolved by
the assistant on the owner's own desktop.

### Implementation result and end gate (2026-08-01)

No production code. The task lands two evidence scripts, nine frame sets with
their measurement records, the analysis README, the R-7 coordinate artifact,
and the plan/log revisions. The frames, numbers and the assistant's reading of
them are in
[evidence/t10-analysis/README.md](./evidence/t10-analysis/README.md) and are
cited rather than restated here (trap #3's documentation analogue — this
section is derived prose and must not become a second source of truth for
numbers the analysis owns).

**Build discipline before every capture** (F-5, F-21): `cargo build -p
wasamo-runtime --release` then `cargo build --release --workspace`, including
before the mutation capture and again before the restoration capture. The base
worktree built under its own `CARGO_TARGET_DIR`, never the repo's (F-40).

#### Trap #2 / #3 — the derived-prose enumeration

The claim: **every number in T10's prose is either measured by T10 or cited to
the task that owns it.** The sites that carry numbers, and which they are:

| Site | Numbers it carries | Disposition |
|---|---|---|
| `evidence/t10-analysis/README.md` | all of T10's own measurements | Owner. Everything else points here |
| `evidence/t10-capture-coordinates.md` | T10's measurements + Observation 4's falsified clauses | Owner of the *proposal*; explicitly not normative, and says so |
| `evidence/README.md` | one identifying line per set | Index only — no measurement is restated beyond the set's identity |
| `plan.md` §T10 | the feasibility measurements taken before the revision | Recorded once, at the head of the section, as the reason the controls are shaped as they are |
| this log section | none of the frame numbers | Cites the analysis README |

`evidence/README.md` is the parallel structure trap #3 names, and it went
stale once already in this phase — T3's `after/` is 30,800 pixels out of date
for two frames and the README is the only thing that says so. Updated in the
same commit as the frames, with a new statement that **frame shape is part of
a set's identity**: every pre-T10 set is a six-frame window-rectangle capture
and every T10 set is a single client-rectangle capture, so the two cannot be
compared at all.

#### Trap #6 — the disagreement that was rooted rather than re-shot

One number disagreed with the phase's record and it was chased rather than
filed under drift: **three repeatability pairs came out byte-identical**,
where T5 finding F-33 measured 25 differing pixels a day apart and 149 on a
session's first launch. A "better than expected" result is exactly the shape
that gets waved through.

The root cause is that these are a **different measurement**, not a better
one. F-33's captures are window-rectangle captures whose outer frame is
alpha-blended against whatever is behind it, taken across sessions; T10's are
client-rectangle captures of a topmost window taken minutes apart in one
session. The difference is in what was photographed, and it retires nothing:
the baseline discipline (re-capture both sides in the comparing session, agree
two captures per side) is what produced these numbers, not something they
license skipping. Recorded in the analysis README as a measurement rather than
as a claim about capture stability in general.

No test failed and no capture was re-run to green. Every frame in the evidence
directory is the first capture taken under its stated conditions.

#### Trap #7 — the controls, and the run that shows each can fail

| Control | The pair | The falsifier |
|---|---|---|
| **A1** crispness, phase level | `t10-base-*` (base `80d79c4`) vs `t10-aware-*` | The base build is the wrong implementation, in the only sense R-1 cares about: it is the phase before the phase |
| **A2** crispness, posture | `t10-unaware-*` vs `t10-aware-*`, **same executable** | The `__COMPAT_LAYER` run *is* the negative side — a build whose declaration was refused |
| **B** logical layout invariance | `t10-unaware-*` vs `t10-aware-*` at an equal 1200 × 720 physical client | `t10-mutation-inbound` — T5's inbound seam removed at all three sites: eleven tiles per row instead of nine, the tenth clipped by the right edge and the eleventh entirely outside the client; the toolbar's right group off the window; the status bar gone; 92,805 differing pixels |
| **C** path form | `t10-control-c` — three legs across an owner-driven 125% → 150% → 125% change, the window untouched after the first leg so the rectangle is the OS's | The counterfactual, built at the independent review: upscaling `1-before` by 1.2 — what a 120-DPI surface stretched by the Visual would give — puts the status region at mid/saturated 3.5–4.4 with max horizontal gradient 146–185, against the real `2-changed`'s **0.71 and 223**. That is T6's `t6-scaled-surface-identity` signature, and the intended result and the look-alike are far apart |
| Measurement check | `t10-shipped-created` and `t10-unaware-created` — outer 1000 × 750 in both, client 982 × 703 against 980 × 701, **7** tiles per row in both | The rectangle separates nothing (T1 measured 1000 × 750 unaware too) and **neither does the tile count**, which is 7 on both rows at this client size; the awareness readback beside them is what does |

**Control B's substitution, stated as what it is.** The `s = 1` side is an
unaware process on the same 125% desktop, not a 100% monitor. For risk R-2 —
a missed conversion site is wrong exactly at `s ≠ 1` — that is the comparison
needed, and it is *stronger* than two monitors because the DIP extent is equal
by construction (960 × 576 on both sides, residual `0x0`) rather than
approximately. It is **weaker** in that it says nothing about monitor-to-
monitor delivery. That half is T11's and is not claimed here.

**A2's second result.** A host whose declaration was refused comes up, lays
out and renders. Every prior artifact for DD-M4-P1-001's tolerated-failure
path is headless; this is the first rendered one. It is a by-product of the
mechanism control B needed, not a control the plan asked for, and it is
recorded as a measurement rather than promoted to an evidence line.

#### Trap #4 — the start gate's "no" was right about the product and silent about the instrument

Recorded at the independent review (finding N4), and it is the **fourth** time
this phase has narrowed an inherited or too-broad gate judgment after F-12
(T2), F-22 (T3) and F-32 (T5).

The start gate marked trap #4 non-applicable because "the scripts' own error
paths are not product branches". That is true and it is the wrong scope:
**this task's evidence *is* the scripts' output**, so an authored branch in a
harness is exactly trap #4's shape, one level out from the product. The
concrete instance: `capture-t10-controls.ps1`'s measure-and-adjust loop could
fall out of its bound with `$iterations = 11` and still record "**reached** in
11 iteration(s)" — asserting success on the one path where the target was
missed, beside a residual line that would have said otherwise. Never executed;
every capture converged in 1.

Fixed by making non-convergence throw: a frame at a client size other than the
target is not comparable against one that hit it, so refusing the capture is
the honest outcome. Three further branches are named rather than predicted
harmless — the PMv2-readback abort, the new occlusion abort, and the
`GetDpiForWindow == 0` guard in the control-C harness. All three fail closed,
so their failure mode is a refused run, not a false artifact.

The sibling script already had this habit: `capture-t10-control-c.ps1` named
its own unexercised DPI-change branch instead of predicting it harmless, and
the analysis README records the moment the owner's run closed that gap. The
gate judgment and the code disagreed inside one task.

#### Trap #5 — carry-forward

| Item | Where | Re-trigger criterion |
|---|---|---|
| The re-derived capture coordinates, with Observation 4's falsified clauses and a draft replacement | [evidence/t10-capture-coordinates.md](./evidence/t10-capture-coordinates.md) | T12 consumes it. Re-derive above 125%, on any change to the window's non-client treatment (M5's custom title bar is already a handoff item), or for any host whose window is not created through `window::create` |
| `__COMPAT_LAYER=DPIUNAWARE` gives a scale-1 run of the shipped bytes | this section + the analysis README | Any later phase needing an `s = 1` reference without a mutation build. It also refuses the declaration, so it doubles as a live exercise of DD-M4-P1-001's failure path — and that is a *side effect*, so a phase that changes the failure handling must re-check what this posture then produces |
| The T11 runnable set is staged and verified | `C:\Users\devuser\dev\wasamo-t11-delivery\` | T11 |

#### Preamble obligation 7 — the runnable set, and a correction to its wording

Staged at `C:\Users\devuser\dev\wasamo-t11-delivery\` (outside the repository;
binaries are not committed):

| File | Bytes | SHA-256 |
|---|---:|---|
| `gallery-zig.exe` | 1,842,176 | `9E0500093747E7854ADC28251E7C4CF80D7A8603960838EE45A63BC43AA7DA83` |
| `wasamo.dll` | 690,176 | `F743AB82B05796D91AB72BC5D6E6634B2EE222102F3F8330168A77D1512B13FF` |
| `wasamo-t11-gallery.zip` | 897,887 | `476C743EB289595D9A98408219C8D0BB8952A1CEA1E94A7C325A1DE78085C4A9` |

**The obligation's phrasing is wrong for the host that should be delivered,
and the correction is recorded rather than quietly applied.** Obligation 7
says "host executable + `wasamo.dll` + compiled `.uic`". `gallery-rust` bakes
an **absolute build-machine path** to its `.uic` in through
`env!("WASAMO_GALLERY_IR")`, so it cannot run from a copied directory on
another machine at all; `gallery-c` and `gallery-zig` **embed** the IR and
load it through `WASAMO_LOAD_MEMORY`, so their set is two files and there is
no `.uic` to ship. The Zig host was chosen for that reason. The gallery rather
than the counter because wrapped tiles and many text runs are what make a
layout change and a rasterization change visible to a human.

**The staged copy was launched from its delivery directory, not from the build
tree** — `t10-delivery-check/` records `PER_MONITOR_AWARE_V2`,
`GetDpiForWindow=120`, outer 1000 × 750, client 982 × 703 and 7 tiles per row,
matching the Rust host at the same created size. "The files were copied" and
"the delivered thing runs" are different facts.

#### Control C — captured, on a human-driven scale change

**Withdrawn: "two frames across a live display-scale change are not obtainable
on this machine".** That was written as a measurement and it is not one. What
was measured is that three *programmatic* routes are unavailable; **the
Settings UI was never tried**, and nothing in the probe run bears on it. The
claim is the phase's own recurring defect — a statement wider than its object
— arriving in the task whose entire content is evidence, and one paragraph
after this log records that defect as the thing to watch for.

The overreach had a second half worth separating, because it is not the same
error. The bullet was also read as "the assistant must *cause* the change",
and that constraint is nowhere in the plan:
[preamble.md](./preamble.md)'s verification-closure item (5) says the
assistant **captures** the path, and the human half of the split it names is
T11's **cross-monitor** form, not the scale change. A human changing Scale in
Settings while
[evidence/capture-t10-control-c.ps1](./evidence/capture-t10-control-c.ps1)
polls `GetDpiForWindow` is control C as specified, needs no undocumented API,
and is not a substitution requiring an owner trade-off decision at all. The
first framing invented a dilemma out of an unexamined assumption and then
asked the owner to resolve it.

What the probe run does establish, unchanged: one monitor, an RDP session on
`Microsoft Remote Display Adapter`,
`DisplayConfigGetDeviceInfo(DISPLAYCONFIG_DEVICE_INFO_GET_DPI_SCALE)`
returning `ERROR_GEN_FAILURE` for the only active source, `Win8DpiScaling = 0`
with no `LogPixels` and no `PerMonitorSettings` key. The one remaining route
is `SPI_SETLOGICALDPIOVERRIDE`, undocumented, which would rescale the owner's
live desktop; a **no-op** call returns TRUE and that is recorded as "the entry
point exists", not as "a real change works". A cross-process synthesised
`WM_DPICHANGED` is not a substitute — its `LPARAM` is a pointer to the
suggested `RECT` and Windows does not marshal it across a process boundary, so
the host would dereference the sender's address in its own space; T8's
synthesis works because it is in-process.

`SPI_SETLOGICALDPIOVERRIDE` therefore stops being the question. It remains
the only *programmatic* route found, it remains unmeasured beyond a no-op
returning TRUE, and it is not needed if a human can reach the Scale control.

**T10 does not close AC7's third requirement either way**, and did not before
this task either: [preamble.md](./preamble.md) already assigns the literal
cross-monitor form to T11, and obligation 5 already says neither half
discharges it alone. Control C's path form and T11's monitor crossing are two
different paths through the same handler, not two names for one.

[evidence/capture-t10-control-c.ps1](./evidence/capture-t10-control-c.ps1)
captures three legs — before, changed, restored — polling `GetDpiForWindow`
and raising the host with `HWND_TOPMOST` immediately before each capture, so
it never takes the keyboard or fights the Settings window for focus.

**Captured 2026-08-01 03:16, owner-driven: 125% -> 150% -> 125%.** The scale
control is reachable in this session, which settles the question the withdrawn
claim had made unanswerable. Numbers, frames and the reading of them are in
[evidence/t10-analysis/README.md](./evidence/t10-analysis/README.md); the two
results that change what the phase knows:

- **Crispness survives the change.** Every earlier crispness frame in this
  phase is at the window's *creation* scale. The 144-DPI status run is
  natively rasterized — the failure it rules out is T6's
  `t6-scaled-surface-identity` signature, geometry following the new scale
  with the text surface left at the old resolution.
- **The round trip is byte-identical**: `3-restored` matches `1-before` over
  all 864,000 client pixels.

**The DIP extent returned exactly (960 x 576 at all three legs) and that is
recorded as an observation, not as a property.** §T10 asserts element order
and wrap structure and explicitly not bit-exact positions, because the OS
chooses the rectangle here and the non-client frame moves by its own
DPI-indexed metrics (18 x 47 -> 22 x 56). Outer went 1218 -> 1462 against
`1218 x 1.2 = 1461.6` and 767 -> 920 against `920.4` — up in width, down in
height — and the client that fell out divided by 1.5 exactly. Whether Windows
computes the suggested rectangle to preserve the client extent or the rounding
landed well, **this run does not distinguish**: 1461 would have given 959.33
DIP. Writing it as a guarantee is the defect this task already withdrew once.

**Two things it does not close.** No intermediate frame was taken — the
harness waits about 1.5 s after detecting the change — so T7's open question
about whether a stale intermediate projection is presented stays with T11. And
a display-setting change is a second path through the handler, not the monitor
crossing; T11 owns the literal form.

#### The non-client decomposition, measured directly

Taken with a separate probe because
[evidence/t10-capture-coordinates.md](./evidence/t10-capture-coordinates.md)
needs the *inset* breakdown and not only the frame totals, and because
deriving top-inset from the total would have been arithmetic where a
measurement was available. An 800 × 600 DIP gallery window, left / top /
right / bottom in physical pixels:

- declared PMv2, `GetDpiForWindow` 120: **9 / 38 / 9 / 9**
- `__COMPAT_LAYER=DPIUNAWARE`, `GetDpiForWindow` 96: **10 / 39 / 10 / 10**

The consequence is a correction to what
[evidence/compare-frames.ps1](./evidence/compare-frames.ps1)'s defaults
(`InsetX 12`, `InsetTop 44`, `InsetBottom 12`) mean. **Stated from one basis,
because the first version of this paragraph mixed two** (independent review
finding N2): the 96-DPI figure the defaults were designed against is a **top
inset of 31** — `SM_CYCAPTION` 23 plus an 8-px border — not the 39 that is
the frame's total *height*. The paragraph above gives its own reason for
taking a direct probe — "deriving top-inset from the total would have been
arithmetic where a measurement was available" — and then made exactly that
conflation one sentence later.

| | top inset | side inset | top margin | side margin |
|---|---:|---:|---:|---:|
| 96 DPI (the design basis) | 31 | 8 | 13 | 4 |
| 120 DPI, aware (measured) | 38 | 9 | 6 | 3 |
| 120 DPI desktop, unaware (measured) | 39 | 10 | 5 | 2 |

The defaults still exclude the whole frame in every row — the correction goes
the safe way — but the margin shrinks as DPI rises while the constant does
not, so a capture above 125% needs them re-derived rather than inherited.

#### Local gates

- `cargo fmt --all -- --check` — exit 0.
- `git diff --check` — exit 0.
- `cargo test --workspace --no-fail-fast` — **37 binaries, 974 tests, 0
  failed**, unchanged from T9's post-commit state. Run against the restored
  tree, after the mutation's `git checkout`, `cargo clean -p wasamo-runtime
  --release` and accepted-source rebuild.
- `git status --porcelain wasamo-runtime/` — empty. The only mutation this
  task made is reverted, and the restored build's frame is byte-identical to
  the pre-mutation one over all 864,000 client pixels.

### Independent review disposition (2026-08-01)

Full independent review of the whole branch. **1 major, 5 minor, 6 nits, all
confirmed**, none disputed. The reviewer re-measured every quantitative claim
independently — re-implementing the pixel comparator with `LockBits`, counting
tile-fill runs, template-matching the magnified crops back to their sources,
running the host under both postures, and running the suite — and **every
number reproduced exactly**. It reported zero instances of "evidence a wrong
implementation would also produce" and zero instances of "a hazard named and
then dismissed by an unmeasured prediction".

**Every finding is in the prose describing the evidence, not in the evidence.**
For a task whose entire deliverable is claims drawn off frames, that is the
relevant surface rather than a mitigation.

| # | Severity | Finding | Disposition |
|---|---|---|---|
| J1 | **major** | The **trap #7 close artifact still said control C was "not captured"**, with `—` in its falsifier column, and the section heading still read "pending". Control C is documented 100 lines further down *in the same file*, in `evidence/README.md`, in the analysis README, and the plan checkbox was flipped in the same commit. A reviewer directed at the gate artifact would conclude control C was open. Also: `capture-t10-control-c.ps1` was a materially new approach with new branches and landed with **no gate re-record**. | **Fixed.** Row rewritten with the artifact and its falsifier; heading corrected; the trap #4 section above records the gate delta the control-C harness should have carried. |
| N1 | minor | The mutation frame's description did not match the frame. Measured tile-fill runs at `y = 100`: nine complete tiles then `1140..1199` — **the tenth** clipped at 60 of 110 px, with the eleventh entirely outside the client. The prose said "the eleventh clipped by the right edge". `Scroll down` retains 118 of 130 px, not "cut in half". `log.md` said "11 clipped tiles per row", which disagreed with the README. | **Fixed** in both documents, with the run positions recorded. Re-measured independently before editing. |
| N2 | minor | The `compare-frames.ps1` inset-margin arithmetic was wrong in two documents in two different ways, and they disagreed. `log.md` said `InsetTop 44` "was chosen against 39", but 39 is the 96-DPI frame's total **height**; the top inset is **31**. On the "39" reading the margin *grew* 5 → 6, contradicting the next sentence. `t10-capture-coordinates.md` compared rows on one basis and columns on another. | **Fixed.** Both now state one basis with a table: 96 DPI 31/8 → margins 13/4; 120 DPI aware 38/9 → 6/3; unaware here 39/10 → 5/2. This matters beyond tidiness because the coordinates file is what T12 folds into normative `verification-environments.md`. |
| N3 | minor | The unaware "tiles per row = 7" in the coordinates table was **neither measured by T10 nor cited** — it came from T4's three-state table, taken before T5/T6/T7 — at the one cell the responsibility re-audit says needed re-taking against the landed declaration. It breached this task's own trap-#2 claim that "every number in T10's prose is either measured by T10 or cited to the task that owns it". The table header also spanned `gallery-rust` and `gallery-zig` across a column only ever run on `gallery-rust`. | **Fixed by measuring it**: `t10-unaware-created/` is a new `-Unaware` capture at the created size, counted at **7**. Header scoped. A by-product worth recording: **the tile count no longer separates the two postures at this size** — both read 7 — so the awareness readback and the 982-vs-980 client are the only discriminators left. The trap #7 row now says so. |
| N4 | minor | `capture-t10-controls.ps1`'s measure-and-adjust loop could exit unconverged and still record "reached in 11 iteration(s)". Never executed. The start gate marked trap #4 non-applicable on grounds that are right about the product and silent about the instrument, which *is* this task's evidence. | **Fixed**: non-convergence throws. Recorded as the phase's fourth narrowed gate judgment in the trap #4 section above. Three further branches named rather than predicted harmless. |
| N5 | minor | Post-control-C staleness: the analysis README said "every frame here was captured by `capture-t10-controls.ps1`" when three came from the other script; `log.md` said "two evidence scripts, nine frame sets" against three and twelve; the retrospective's count was one short even before control C. | **Fixed** in all three. |
| T1 | nit | The crispness prose described "the dot of the `i`" in a crop that contains no lowercase `i`, and a hyphen "smearing across its neighbours" that is space-separated. The paired aware claim was accurate and measurable. | **Fixed**, and replaced with the reviewer's measurement: the hyphen is a two-row bar with no saturated pixel in the base and a single row with saturated pixels in the aware frame. |
| T2 | nit | The crop rectangles were recorded nowhere — `magnify-crop.ps1` prints them to the console only — so the fairness of the comparison that carries control A was not auditable from the committed artifacts. The reviewer recovered them by patch-matching and confirmed the comparison **is** fair. | **Fixed**: rectangles recorded beside each pair, including the note that the 144-DPI crop is the same region in DIP and therefore prints 1.2× larger. |
| T3 | nit | `status-unaware-posture-5x.png` was committed and never referenced. | **Fixed**: referenced, and it earns its place — it shows the softness is the posture rather than the vintage of the code. |
| T4 | nit | Neither harness verified that the photographed region belonged to the target window. `CopyFromScreen` takes a screen rectangle; the control-C capture interleaves with a human driving Settings and is the phase's highest-occlusion-risk shot. | **Fixed**: both harnesses check four interior client points with `WindowFromPoint` and refuse to record otherwise. **The already-committed frames do not carry the line** — their freedom from occlusion rests on inspection, by the author and by the reviewer, and that is stated rather than implied. The guard was verified not to false-positive and a capture taken with it is byte-identical to `t10-aware-a`, so it is capture-neutral and the committed set stands. |
| T5 | nit | `capture-t10-control-c.ps1` would read a `GetDpiForWindow` of 0 — window gone, process alive — as a scale change. Reasoning-only; guarded in practice by the per-iteration `HasExited` check. | **Fixed**: 0 throws. |
| T6 | nit | `handoff.md`'s three new T10 rows sit inside a table broken by a stray blank line introduced at `dedd327` on 2026-07-28, **before T10** — everything after it renders as literal pipe text. Pre-existing, but T10's trap-#5 artifact points at those rows. | **Fixed**: blank line removed. Pre-existing defect, repaired because this task's own close artifact is unreadable without it. |

**The review also strengthened three claims the task had stated more weakly
than the evidence allowed**, and those are folded in rather than left in the
review:

- **Control B's layouts are not merely equivalent but bit-identical in
  position.** Toolbar button runs are the same six intervals across aware,
  unaware and base; tile runs match; the status bar's top edge is `y = 685`
  in all three.
- **Control C's invariance holds in DIP, measured.** At 144 DPI the tiles sit
  at `x 18..149`, pitch 150 — 12 DIP origin, 88 DIP width, 100 DIP pitch,
  identical to the 120-DPI legs' 15..124, pitch 125. The status bar is 42
  physical rows at 144 DPI and 35 at 120, both 28 DIP.
- **Control C's crispness claim has a falsifier after all.** It shipped
  without one. The reviewer constructed the counterfactual — `1-before`
  upscaled 1.2×, which is what a 120-DPI surface stretched by the Visual
  would look like — and measured mid/saturated 3.5–4.4 with max horizontal
  gradient 146–185 against the real frame's 0.71 and 223. Recorded in the
  trap #7 row.

**One observation the review made that the task should have made itself**:
`t10-control-c/1-before.png` is byte-identical to `t10-aware-a/gallery-client.png`
captured 2.5 hours earlier, at a different screen position, from a separate
process launch. That is a stronger repeatability datum than the three pairs the
analysis README cites, and it supersedes that section's explanation ("taken
minutes apart in one session"). Folded into the README.

**Not verified by the reviewer, and left unverified**: the `cargo clean -p
wasamo-runtime --release` "9 files, 6.9 MiB" figure, which is not retroactively
checkable. It is a console reading recorded at the time.

---

## T11 — Owner human-visible smoke (positive control C, literal form)

### Carry-over audit and responsibility re-audit (2026-08-01, before start gate)

Task branch `feat/m4-phase-1-t11`, cut from `feat/m4-phase-1` = `ea1215e`.

**Carry-over into T11**, read out of the T4–T10 retrospectives and
[handoff.md](./handoff.md) rather than out of §T11 alone.

| From | Proposition | Obligation on T11 |
|---|---|---|
| Obligation 7 (T10) | The runnable set is staged and was launched from its delivery directory, not from the build tree | T11 observes the delivered copy. The owner has transferred and extracted it, so the observing machine needs no repository, no toolchain and no build |
| T10 (handoff) | `gallery-rust` / `counter-rust` bake an absolute build-machine path to their `.uic` and cannot run from a copied directory | The delivered host is `gallery-zig.exe` + `wasamo.dll`, and nothing else is expected to start there |
| T10 (handoff) | `__COMPAT_LAYER=DPIUNAWARE` in a child's environment gives a scale-1 run of the **shipped bytes** | The positive control is the same executable under that variable. No mutation build, and **the delivered files are not replaced** — identical bytes is precisely what stops a difference being attributed to a different build |
| F-49 (T9) | Arranging OS state and succeeding at it are different facts | The unaware posture is **read back** — Task Manager's DPI Awareness column, with both processes visible — not inferred from having set the variable |
| F-28 (T4) | The client rectangle does not scale by `s`; the non-client frame moves by its own DPI-indexed metrics | The invariant asserted is element order and wrap structure — tiles per row — never a bit-exact wrap position |
| T8 stated limit 2 | On a real crossing the **OS** chooses the rectangle, so logical invariance is approximate | The same, and T10 control B's exact-DIP result is **not** transferable here: that path chose the rectangle by measure-and-adjust, this one does not |
| F-33 (T5) | A committed frame set is not a baseline | Every T11 frame comes from one session on one machine, and none is compared against a committed set. The observing machine is not the development machine, so its non-client metrics are its own and are not checked against §T4's |
| F-48 (T9) | An unaware observer reads an aware window's rectangle in virtualized coordinates | T11 takes **no coordinate measurement**. The one number recorded per monitor is a tile count, which is not virtualized. Recorded so that "F-48 does not bite here" is a decision rather than an omission |
| T7 F-34, forwarded through T10 | Whether a stale intermediate projection is ever presented as a frame during the change is still open | **Scoped out of T11 at start** — re-audit point 1 |

**Responsibility re-audit — what T11 should be, not what §T11 said.**

1. **The intermediate-projection question is removed from T11's observation
   list, before the observation rather than after it.** T10's retrospective
   forwarded it here on the ground that its own harness waits about 1.5 s after
   detecting the change and therefore cannot speak to it. The owner's objection
   on reading the draft checklist is the correct one and is the recorded
   reason: during a drag the window is **in motion**, so the eye tracks it
   instead of fixating, and the artifact in question is one or two composited
   frames — on the order of 16–33 ms at 60 Hz — which is below what a human can
   consciously resolve in either direction. An item that cannot produce a
   positive observation can still produce a **negative** one, and "watched for
   it, did not see it" would be exactly the claim-wider-than-its-object defect
   this phase has now withdrawn twice (§T4 F-31, §T10 control C). The question
   is not dropped: it goes to [handoff.md](./handoff.md) with the two
   instrument classes that could answer it — frame-level capture, where a
   positive sighting is conclusive and a null result says nothing, or an
   in-process observation of whether a frame is committed inside the handler at
   all, which is the shape F-34 actually poses. Neither is a human-smoke
   instrument, and T11 lands no production code. **The screen-recording option
   was offered with that asymmetry stated and the owner declined it**, so the
   record reads "not captured", not "captured and saw nothing".
2. **§T11's three bullets are a settled-state observation and carry no
   positive control.** [AGENTS.md §Testing rules](../../../../AGENTS.md)
   requires the owner's human-visible smoke to separate the intended behaviour
   from a coincidental look-alike, and here the look-alike is known precisely:
   a DPI-unaware window is bitmap-stretched to the same physical size on the
   destination monitor, so **size separates nothing** — T1 measured it, and
   rows 1 and 3 of §T10's three-state table share a rectangle for the same
   reason. What separates them is glyph sharpness and the process's declared
   level. The control is therefore the same executable run twice, once under
   `__COMPAT_LAYER=DPIUNAWARE`, compared **side by side inside one frame on one
   monitor**, so the comparison does not rest on two capture events.
3. **A leg the plan did not have: the control has to *agree* somewhere too.**
   With the external display at 100% the conversion is the identity, so the
   aware and unaware runs should be indistinguishable there and differ on the
   150% panel. A control that differs on both monitors is satisfied by any two
   differing things; one that differs on exactly the scaled panel is satisfied
   by the posture. That is why the scale pair is **chosen** — internal 150%,
   external 100%, both owner-adjustable — rather than inherited from whatever
   the desks happened to be set to (150% / 125% when the question was asked).
4. **The pointer path is added to the observation.** After the crossing,
   clicking a gallery tab and hovering a button checks that the inbound
   conversion follows the window's new scale in a composited window. T8 already
   drives a click through a real `WM_LBUTTONUP` at a synthesised scale, so this
   is not the only evidence for that seam — but it is the only one where the
   coordinate comes from a real device across a real crossing.
5. **The protocol is written down because the instrument is a human.** Every
   earlier GUI gate in this phase had a script, and the script was itself the
   record of what was done. T11's equivalent is
   [evidence/t11-owner-smoke/protocol.md](./evidence/t11-owner-smoke/protocol.md),
   in Japanese because the owner executes it. Without it the only record of the
   procedure would be the verdict, and a verdict cannot be audited against a
   procedure that was never written down.

**Environment, as arranged before the run** (recorded now, so a later reading
knows what was intended rather than inferring it from the frames): laptop
internal panel at 150%, external display at 100%, both adjustable; the delivery
set already transferred by `scp` and extracted; frames returned the same way for
assistant analysis. The observing machine is **not** the development machine.

### Start gate (recorded 2026-08-01, before the observation)

Review lane: **full independent review**. §T11 is absent from
[preamble.md §Review lanes](./preamble.md#review-lanes) — the table stops at
T10 — so the lane is assigned here rather than inherited, by the reasoning T10
recorded: the deliverable is rendered evidence and the claims drawn off it, and
zero production code lowers the risk *of the change*, not the risk this task
carries, which is that the recorded verdict outruns what was seen. T11 also
carries one half of AC7's third requirement.

| # | Applies | Reason and planned close artifact |
|---|---|---|
| 1 — semantic migration / call sites | **no** | No code, no enum, no schema, no traversal. Re-decided rather than inherited: the delivered binary is run unmodified, so there is not even a build |
| 2 — structural side effects | **yes**, in trap #3's documentation sense | The close record restates numbers owned by §T4, §T8 and §T10. Close by citing the owning document for every inherited number and marking which numbers this task observed itself |
| 3 — parallel / derived data | **yes** | [evidence/README.md](./evidence/README.md) is a parallel index of which frame set evidences which claim, and it goes stale the moment a set is added without it. The `t11-owner-smoke/` row lands in the same commit as the frames |
| 4 — authored branch | **no** | No reject, diagnostic or size branch, and no script is authored — the harness is a human plus the built-in capture tool |
| 5 — carry-forward | **yes** | Two carriers are already visible: the intermediate-projection question, which goes to [handoff.md](./handoff.md) with the instrument classes that could answer it, and whatever the crossing shows about approximate logical invariance, which is M4-Phase 8's input for per-window differing scale |
| 6 — deterministic failure | **yes** | This task's failure mode is **re-dragging**. A disagreement with a recorded number — tiles per row above all — is root-caused, not repeated until it reads better. The second half is specific to a human instrument: a "did not observe" is recorded only for observations the procedure can actually produce, which is why re-audit point 1 removes one item entirely rather than letting it be answered weakly |
| 7 — GUI positive control | **yes** | The gate's substance. The control is the unaware twin, side by side in one frame, expected to differ on the 150% panel and to agree on the 100% one. Its own can-fail property is that agreement leg: if the two differ everywhere, the difference is not attributable to the posture |

Constraints fixed before the run: the delivered files are neither modified nor
replaced; the posture is read back rather than assumed; the window is not
resized at any point, because its physical size and its non-client frame are
both part of what is observed; every frame comes from the one session; and no
number is compared against the development machine's.

### Observation result and end gate (2026-08-03)

No production code, no build on the observing machine. The task lands eleven
owner-captured frames, six magnified crops, two analysis scripts, the analysis
README, and the plan / log / handoff revisions. Frames, numbers and the reading
of them are in
[evidence/t11-owner-smoke/README.md](./evidence/t11-owner-smoke/README.md); this
section records the gate, not the analysis.

**Executed by the owner on 2026-08-03** against the T10 delivery set, launched
from its delivery directory on a laptop whose internal panel was set to 150% and
external display to 100% before the host started. The observing machine is not
the development machine and no number is compared against §T4's.

**What the owner attests, as distinct from what the frames show.** Four facts
come from the owner and are recorded as attestation: which window in each pair
was launched normally and which from the shell carrying
`__COMPAT_LAYER=DPIUNAWARE`; that the side-by-side windows were narrowed by hand
because two full-size windows do not fit on one 150% panel; that only two
`gallery-zig.exe` instances were running; and that the `Win+Shift+arrow` leg was
performed after the drag legs. Everything else below is read off the frames.

**The result, in one line each.** Logical layout preserved across the crossing —
7 tiles per row on both monitors, wrap structure and element order identical, and
the content band's DIP width differing by 1.3 DIP, which is F-28's residual and
did not move a wrap position. Non-client scaled with the window and visibly not
by `s` (band width ×1.5026, band top ×1.452). Round trip returned the same frame
— 12 differing pixels of 1,036,642, all four corner radii. Pointer path followed
the new scale — the Favorites tab took a click on the destination monitor. The
non-modal `Win+Shift+arrow` delivery agrees with the modal drag delivery to
within F-33's text-intensity drift. Positive control fired, and its agreement
leg fired too: at 100% the aware and unaware runs are indistinguishable, at 150%
and 175% they are not, and the aware side is flat across all three scales while
the unaware side degrades monotonically.

#### Trap #2 / #3 — inherited numbers cited, not restated as this task's

| Number used | Owner |
|---|---|
| The ~1.6 DIP client residual and its `GetSystemMetricsForDpi` decomposition | §T4 and [handoff.md](./handoff.md); T11 measures its own 1.3 DIP on another machine and does not re-derive the mechanism |
| The 13-per-channel text-intensity drift band | §T5 finding F-33; T11's own differences are 1 and 8 per channel and are compared to it, not merged into it |
| `__COMPAT_LAYER=DPIUNAWARE` gives a scale-1 run of the shipped bytes | §T10; T11 uses the mechanism and adds the first Task Manager readback of both postures |
| DD-M4-P1-004's outer-rectangle claim | The ADR; T11's captured bounds corroborate its shape and are explicitly not a second measurement |
| The gallery's 7-tiles-per-row signature | §T10 (`t10-shipped-created/`), taken at 125% on the development machine; T11's 7 is its own reading at 150% and 100% |

[evidence/README.md](./evidence/README.md) gains the `t11-owner-smoke/` row in
this commit, which is trap #3's close artifact — the index is a parallel source
of truth about which set evidences which claim and goes stale the moment a set
lands without it.

#### Trap #6 — three disagreements, all rooted, none re-shot

The gate line for this task named re-dragging as its failure mode. Three
comparisons came back non-zero and each was root-caused rather than repeated:

| Disagreement | Root cause | Disposition |
|---|---|---|
| The drag round trip is not pixel-identical: 8,323 of 1,053,162 differ, max delta 100 | Every differing pixel is on the outermost one or two pixels of the frame or at a corner radius. The window sat at a different desktop position on the way back, and a rounded corner blends with what is behind it | Not a render difference. Reported with the inset that isolates it (12 pixels at inset 4, all corners) rather than by widening the inset until it reads zero |
| The 100% control pair has identical statistics but 17,712 differing pixels | The two windows were sized by hand: content bands 515 px against 522 px. The tile grid lands on different sub-pixel origins | The claim was narrowed to what was measured — the *statistics* agree — and the earlier "identical" wording was corrected before it reached a document |
| `10` vs `2` (same monitor, two launches) differ in 3,885 pixels | Max per-channel delta is **1**, confined to text rows | F-33's intensity-only drift, an order below its measured band. Recorded with its number, not filed as "drift" without one |

#### Trap #5 — carry-forward

| Item | Where | Re-trigger criterion |
|---|---|---|
| Whether a stale intermediate projection is presented as a frame during a scale change | [handoff.md](./handoff.md) | T7 F-34's question, removed from T11 at the start gate as unanswerable by a human instrument. Re-triggers for any task that can run frame-level capture or observe commit boundaries in-process |
| The toolbar overlaps rather than wrapping or clipping when the client is too narrow | [handoff.md](./handoff.md) | Observed at 100%, 150% and 175% alike, so width-driven and not this phase's. Re-triggers at M4-Phase 2 (layout / event work) or whenever a host is run at a client narrower than its content |
| A positive control needs a leg where it must **agree** | this section, F-50 | Every later control built as "A differs from B" |
| A list-based readback is evidence about the rows it shows | this section, F-51 | Any later use of Task Manager, a process list, or any sorted/scrolled UI as an artifact |

#### Trap #7 — the positive control, and how it could have failed

The control is the same executable run twice, and it ships with three ways to
come out wrong rather than one. It could have shown **no** difference at 150%
(the posture would then not be what separates the runs); it could have shown a
difference at **100%** (the metric would then be measuring window identity,
position or capture rather than rasterization); and the aware side could have
degraded with the scale factor (R-1's claim would then be false). None did.
`6-taskmgr-dpi.png` is the readback that keeps "we set the variable" and "the
process is unaware" separate facts (F-49).

#### Findings

**F-50 — a positive control needs an agreement leg, not only a difference
leg.** §T11 as planned had the control differing on the scaled panel and said
nothing about where it must *not* differ. A control of the form "A differs from
B" is satisfied by any two differing things — a different build, a different
window size, a different capture. What makes the difference attributable to the
posture is that the same pair is **indistinguishable** where the posture cannot
matter, which here is the 100% monitor, where every conversion in this phase is
the identity. The leg was added at the start gate for this reason and the scale
pair was chosen to make it available; it then also validated the measurement,
because a metric that separated the two runs at 100% would have been measuring
something else. *Re-trigger:* any later control expressed as a difference.

**F-51 — a sorted, scrolled list is evidence about the rows it shows, not about
the set.** The Task Manager readback was very nearly written up as "two
`gallery-zig.exe` processes were running". The list is sorted by Description and
scrolled about a fifth of the way down, and rows sharing a Description are
contiguous, so a third instance immediately above the visible top row cannot be
excluded from the image. The correct claim is about the two rows the artifact
shows; that only two existed is the owner's attestation and is labelled as one.
This is F-48 and F-49's family — the instrument reporting something narrower
than what the reader assumes — arriving in a UI screenshot rather than in an API
result. *Re-trigger:* any artifact that is a view onto a list.

**F-52 — a deviation from an owner-executed protocol is scoped, not just
noted.** Two full-size windows do not fit side by side on one 150% panel, so the
owner narrowed them, and the pairs show 5 tiles per row against the protocol's
"do not resize". The reflex answers are both wrong: discarding the frames throws
away the control, and using them for everything would put layout claims on
frames whose layout was altered by hand. What the close does instead is say
which claims each artifact can still carry — the narrowed pairs support the
crispness claims only, and every layout claim rests on the six frames that were
not resized. *Re-trigger:* any owner- or human-executed procedure whose
execution differs from what was written. The protocol being committed
([evidence/t11-owner-smoke/protocol.md](./evidence/t11-owner-smoke/protocol.md))
is what makes the deviation visible at all; had the procedure lived only in
chat, the frames would have looked like the plan.

#### Owner's reading

Kept separate from the analysis above, because it is the thing T11 exists for
and the pixel work is its corroboration, not the other way round. The owner's
own three statements, rendered from the Japanese:

- text was properly legible on the destination monitor — no sense of blur;
- nothing felt broken about the layout;
- **with the two runs side by side, the unaware one's blur was visible to the
  eye.**

The third is the one that cannot be obtained any other way. Every quantitative
result in this task is a proxy for a glyph-shape judgement, and the judgement
is a human one; the numbers say the two runs' edges are shaped differently and
by how much, and a person looking at them says which one is worse. Both were
run, and they agree.

#### End gate

Owner verdict recorded — the attested facts above plus the owner's own reading
in §Owner's reading; frames committed with their [evidence/README.md](./evidence/README.md)
row in the same commit; the positive control's result recorded including the
agreement leg; the two new observations triaged to
[handoff.md](./handoff.md). `cargo test --workspace` is not re-run for this
task and is not claimed: nothing in the workspace changed, and the observing
machine has no toolchain. Full independent review before merge.
