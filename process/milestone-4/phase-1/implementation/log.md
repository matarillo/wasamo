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

