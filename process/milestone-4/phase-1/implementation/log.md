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
