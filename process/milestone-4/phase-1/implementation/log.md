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
| 6 | `wasamo_widget_insert_child` (ABI structural mutation) | inserted button's label was placed; the button itself was not | **pre-existing class, not introduced here.** That entry point neither marks layout dirty nor drains, so the inserted node's *own* Visual already had no offset or size until the next layout pass. T3 makes the label match the button instead of floating at (16, 8) over a zero-sized background |
| 7 | `ir_loader` conditional / `for`-range mutations | as above | **preserved.** Both sites call `mark_layout_dirty_for` after the mutation, so the drain's layout phase places the new subtree. Exercised by the `gallery-lightbox` frame, whose three buttons are constructed *after* the tree was attached |
| 8 | Visual parenting / Z-order (`bg_container.Children().InsertAtTop`) | at construction | **unchanged** — only the two geometry writes moved |
| 9 | The node's `SizeConstraint::Fixed` pair | derived from `(lw, lh)` in both writers | **unchanged**, and still per axis (F-10): `Fixed(lw + BUTTON_PAD_H * 2.0)` and `Fixed(lh + BUTTON_PAD_V * 2.0)` in each |
| 10 | Hit-testing / hover (`visual_rect`) | reads the **node's** visual, never the label's | **unchanged** |
| 11 | `update_button_style` / `update_button_enabled` / `update_toggle_button_checked` | touch the background brush only | **unchanged** |
| 12 | `draw_text`'s surface size at construction (`lw.max(1.0)`) | — | **unchanged.** F-14 removes that clamp at T6, not here |
| 13 | Per-pass cost | — | **changed**: two extra WinRT property writes per Button-family node per layout pass. Bounded — the gallery has nine — on an event that is a resize or a property write |

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
| `labelupdate-clicked-twice` | the same write again, at a third label width | **0** of 224,480 |

**What the pair does and does not discriminate**, stated rather than
implied. It **does** show that the relocated write lands, for both widget
kinds, on all three paths that reach a Button (first layout, drain
re-layout, post-attach construction), and at three different label widths
on the update path — N1 and N2 are the proof that a frame in this set goes
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

### F-21 — a host-package build does not rebuild the runtime

Found while running the N2 mutation, and recorded because it is a
**false-negative generator for every GUI evidence gate in this phase**,
not a T3 detail.

`cargo build --release -p gallery-rust` completes without recompiling
`wasamo-runtime`. The host reaches the runtime through `wasamo-sys`,
which links `wasamo.dll` by a build-script link-search path rather than
by a cargo dependency edge, so a source change in `wasamo-runtime` is not
in the host package's dependency graph and does not trigger a rebuild.
The launched host then loads the **previous** `wasamo.dll` from
`target/release`.

Measured: the first N2 run — `ToggleButton` dropped from the sync arm —
was built that way and produced a gallery frame **identical to the
unmutated build**, which briefly read as "the mutation does not fire".
Rebuilt with `cargo build --release --workspace`, the same mutation
removed exactly the three tab labels. The mechanism is adjacent to F-5
but distinct: F-5 is a link failure from a cold directory, this is a
silent staleness with a green build.

*Disposition:* every capture in this phase is preceded by
`cargo build --release --workspace`, folded into [plan.md](./plan.md) §T6,
§T9 and §T10 and into preamble R-1b; carried to
[handoff.md](./handoff.md) alongside F-5, and folded into T12's existing
[AGENTS.md §Build ordering](../../../../AGENTS.md) correction, which today
describes only the `wasamoc` ordering and says nothing about the runtime
cdylib.

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
| §T9 | **correction — F-21**: the three-host rebuild is the artifact for DD-001's boundary claim, and a host-package build would run it against a pre-T9 DLL |
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
- **F-21 — a host-package build does not rebuild the runtime.** Recorded
  in full above. *Disposition:* [plan.md](./plan.md) §T6, §T9, §T10, §T12;
  preamble R-1b; [handoff.md](./handoff.md).
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

