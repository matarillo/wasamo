# DD-M4-P1-003 — Initial scale acquisition and DPI-change propagation

**Status:** Accepted
**Phase:** M4-Phase 1
**AC:** AC7, third requirement ("handle DPI changes across monitors")

## Context

Once [DD-M4-P1-001](./dd-m4-p1-001-dpi-awareness-declaration.md) makes
the OS report real per-monitor DPI and
[DD-M4-P1-002](./dd-m4-p1-002-coordinate-space-and-conversion-boundary.md)
defines what a scale factor is used for, two questions remain: **where
does the scale come from at window birth**, and **what happens when it
changes**. `wnd_proc` handles no `WM_DPICHANGED` today
([constraints §1](../requirements/constraints.md)).

The decision has a second consumer beyond AC7. M4-Phase 8 puts a second
window on the desktop, potentially on a monitor at a different scale.
Where the scale is *held* decides whether that is an additive change or
a structural rebuild, and the framing explicitly assigns this DD the
responsibility of **not blocking per-window scale**
([framing](../requirements/framing.md) §含まないもの).

One property inherited from DD-002 shapes everything below and is worth
stating before the options: **layout results in DIP are invariant under
a scale change.** The engine never receives a scale factor, `measure`
returns DIP, and authored values are DIP — so a DPI change invalidates
the *physical projection* and the *rasterization*, and nothing else. It
does not re-decide a single layout number. That is why the propagation
below is as small as it is, and it is what the integration test asserts.

**Qualified in part (2026-07-29, M4-Phase 1 T4; owner-approved).** The
*mechanism* above is unaffected — the engine still never receives a
scale factor, so the layout **function** is scale-invariant. What is too
strong is the consequence "it does not re-decide a single layout
number", and the same over-reach appears in §Structural side-effect
enumeration row 4 and §Verification. Layout is handed the **client**
extent, and the client extent is not preserved when the *window's*
logical size is: the non-client frame scales by its own DPI-indexed
system metrics rather than by `s`. Measured on the M4-Phase 1
development machine — an 800 × 600 DIP outer request yields 784 × 561
DIP of client at 96 DPI and 785.6 × 562.4 DIP at 120 DPI, because
`SM_CXSIZEFRAME` / `SM_CXPADDEDBORDER` / `SM_CYCAPTION` step 4/4/23 →
4/5/29. So a wrap position sitting near a line-break boundary may
legitimately move across a real scale change, and exact invariance is a
property of a **controlled client extent**, not of the OS-suggested
rectangle. Nothing decided in this DD changes: the suggested rectangle
is still applied and the step ordering is unchanged. The integration
test preserves the client extent and therefore still asserts equality
rather than a tolerance. Measurement, reasoning and the correction to
the phase plan:
[implementation/log.md](../implementation/log.md) §T4 (finding F-28 and
independent review finding R-2).

## Decision dependency summary

Consumes DD-002's space definition and its re-rasterization walk (the
walk is specified there; this DD is its second caller). Provides the
per-window scale that DD-002's conversion sites read. Close artifact:
the structural side-effect enumeration below (implementation gate).

## Sub-issues

- **Where the scale is held.**
- **How the initial scale is acquired**, and how a DIP-denominated
  window size is realised given that the DPI is not known until the
  window exists.
- **What `WM_DPICHANGED` does, in what order.**
- **Invalidation granularity.**
- **Failure handling.**

## Where the scale is held

### Options

- **S1 — No storage; call `GetDpiForWindow` at each point of use.**
  - What you gain: no cached value can go stale.
  - What you give up: a syscall inside the mouse-move path, and — the
    real objection — there is then **no moment at which "the scale
    changed" is observed**. The re-rasterization walk needs an event to
    hang off; with S1 the runtime would have to detect the change by
    comparing against a remembered value, which is S2 with the storage
    left implicit. Rejected on merit.
- **S2 — A field on `WindowState`.** Seeded at creation, updated in the
  `WM_DPICHANGED` handler.
  - What you gain: one owner, one update site, and the update site is
    exactly where the change is announced. `WindowState` is already the
    per-window aggregate (HWND, root visual, callbacks, root widget),
    so the scale sits with the things it applies to. **It is already
    per-window**, so M4-Phase 8's "each window can be on a differently
    scaled monitor" needs no structural change — a second `WindowState`
    carries a second scale and the conversion sites, which already read
    through the window, keep working.
  - What you give up: the value must be updated before anything reads
    it during a change — an ordering obligation, addressed below and
    load-bearing.
- **S3 — A process-global scale.**
  - Rejected on merit: it is correct only while exactly one window
    exists, and M4-Phase 8 is four phases away. Adopting it would
    guarantee a teardown, which is the outcome the framing charged this
    DD with avoiding.

### Recommendation

**S2 — a `DipScale` field on `WindowState`.** `GetDpiForWindow` remains
the single source of truth for the value; `WindowState` caches it.

## Initial scale acquisition

`GetDpiForWindow(hwnd)` immediately after `CreateWindowExW` returns, and
before any layout runs — so `set_root`'s first pass and every Visual
write it performs already use the real scale.

There is a genuine chicken-and-egg problem underneath: by
[DD-M4-P1-004](./dd-m4-p1-004-unit-contract-and-spec-wording.md),
`wasamo_window_create`'s `width` / `height` are **DIP**, but the DPI is
not knowable until a window exists on a monitor.

### Options

- **I1 — Create, then correct.** Create the HWND at the requested
  numbers, read `GetDpiForWindow`, and if the scale is not 1 apply
  `size × s` with `SetWindowPos`.
  - What you gain: it works with `CW_USEDEFAULT` placement, where the
    monitor is the OS's choice and therefore unknown in advance. And
    **there is no visible flash**, because `wasamo_window_create` and
    `wasamo_window_show` are separate ABI calls — the correction lands
    while the window is still hidden. That property is a consequence of
    the existing ABI shape, not an assumption about timing.
  - What you give up: two window-geometry operations instead of one, and
    a transient period where the HWND's size is wrong. Invisible, but
    real if a future path ever queries geometry between the two.
- **I2 — Determine the monitor first.** `MonitorFromPoint` /
  `MonitorFromWindow` plus `GetDpiForMonitor`, then create at the
  correct physical size in one step.
  - Rejected on merit: with `CW_USEDEFAULT` the runtime does not know
    where the window will be placed, so it would be predicting the OS's
    placement decision — and being wrong would produce exactly the
    correction I1 performs, plus a wrong guess. It becomes attractive
    only once the window's position is author-controlled, which is
    M4-Phase 8 / AC11 territory.
- **I3 — Leave `width` / `height` as physical pixels.** Rejected by
  DD-004's unit contract; also a worse product (a host cannot express
  "an 800-point-wide window" at all).

### Recommendation

**I1**, with `AdjustWindowRectExForDpi` available if the correction is
ever expressed in terms of a desired *client* size. It is not needed for
the semantics DD-004 states, because `width` / `height` denote the
**outer window rectangle** — which is what they have always denoted,
since they are passed straight to `CreateWindowExW`.

**Superseded in part (2026-07-29, M4-Phase 1 T4, by
[DD-M4-P1-005](./dd-m4-p1-005-unconditional-size-correction.md)).**
I1's wording above conditions the correction on the scale — "if the
scale is not 1 apply `size × s`". **That clause is superseded: the
correction is unconditional.** Nothing else in this section changes —
create-then-correct, the `CW_USEDEFAULT` reasoning, the flash-free
property and `AdjustWindowRectExForDpi`'s availability all stand. As
implemented the correction is unconditional,
and the difference is real rather than a matter of implementation shape:
at a scale of 1 the size-preserving `SetWindowPos` still dispatches
`WM_WINDOWPOSCHANGING` and `WM_GETMINMAXINFO`, which a guarded
implementation would not. (Both reach `wnd_proc` before `GWLP_USERDATA`
is installed and go straight to `DefWindowProcW`, so nothing in the
runtime and no host can observe them — but the message stream differs,
and that is enough to make this a departure from the text rather than a
reading of it.)

The reason is
[DD-M4-P1-001 §Failure handling](./dd-m4-p1-001-dpi-awareness-declaration.md):
tolerating a failed awareness declaration is safe *because the
conversion machinery is unconditional and there is no second code path
to keep correct*. A guard here would be a branch that no test can fire
until the declaration lands — on the one path both public window-create
entries and all three example hosts take — which is precisely the
untested-authored-branch failure the phase's implementation gates are
armed against.

**This annotation is not itself sufficient, and the first draft of it
claimed otherwise.** It said the decision "(create, then correct) is
unchanged" and that a clause inside an option's description was being
narrowed rather than a decision changed. The T4 delta review rejected
that: the condition is part of **what option I1 is**, a reader
implementing I1 as written would not produce the shipped behaviour, and
adjudicating a conflict between two Accepted DDs is itself a new choice.
[process/README.md](../../../README.md) makes `decisions/` immutable
under the **supersede rule**, and the precedent this disposition leaned
on — [doc-system.md](../../../cross-milestone/decisions/doc-system.md)'s
"Superseded in part" block — *points at a successor record* (DD-V-026)
rather than standing in for one. The citation was to the annotation's
shape while omitting the thing it exists to reference.

**The successor is
[DD-M4-P1-005](./dd-m4-p1-005-unconditional-size-correction.md)**, filed
2026-07-29 on owner approval, which compares the unconditional form
against restoring the guard and supersedes this clause alone. Reasoning
and the measurement:
[implementation/log.md](../implementation/log.md) §T4 (independent
review finding R-3, delta review finding 1).

**Recorded caveat:** the DPI observed immediately after creation is the
DPI of the monitor the OS chose. If the window is subsequently moved,
`WM_DPICHANGED` arrives and the ordinary path handles it. There is no
separate "the initial value was provisional" case to design — the change
path *is* the correction.

## `WM_DPICHANGED`: what happens, in what order

The order is fixed by the ADR because it is load-bearing, not
incidental.

1. **Update `WindowState`'s scale first**, from `HIWORD(wParam)`.
2. **Apply the OS-suggested rectangle** (`lParam`, a `RECT*`) with
   `SetWindowPos(..., SWP_NOZORDER | SWP_NOACTIVATE)`.
3. **The nested `WM_SIZE` performs the re-layout.** `SetWindowPos`
   dispatches `WM_SIZE` **synchronously, before it returns**, and the
   existing `WM_SIZE` arm already converts the client extent to DIP
   (DD-002 site 1) and re-runs layout with the visual writes scaled.
   That is why step 1 must precede step 2: **if the scale were updated
   after `SetWindowPos`, the nested `WM_SIZE` would lay out and project
   with the stale scale**, and the window would be visibly wrong for
   one frame at best. This is the single most likely ordering defect in
   the phase and is stated so the implementation is written against it.
4. **Re-rasterize text surfaces** through DD-002's walk, at the new
   scale.
5. Return `LRESULT(0)`.

**Whether to apply the suggested rectangle** was a real choice.
Ignoring it keeps the window at the same *physical* size across a
monitor change, which means its *logical* size changes — a window that
grows and shrinks as it crosses monitors, and text that stays the same
number of device pixels while everything around it rescales. Applying it
preserves the logical size, which is the behaviour every DPI-aware
Windows application has and the one positive control C is looking for.
**Apply it.**

**Why step 4 can follow step 3 safely.** Because layout is
scale-invariant (§Context), re-rasterization does not change any node's
`SizeConstraint::Fixed(w, h)` — `measure` returns the same DIP values at
any scale — so it cannot invalidate the layout that step 3 just
computed. Ordering re-rasterization after re-layout is therefore a
choice about when the brush and the visual's new physical size land
together, not a correctness constraint. Stating the reason matters: if a
future change makes `measure` scale-dependent (explicit hinting, a
snapped metric), this ordering stops being free and must be re-derived.

**Not handled this phase:** `WM_GETDPISCALEDSIZE`, which V2 delivers
before the change so a process can propose its own new size. Wasamo has
no reason to override the OS's suggestion — the suggestion preserves
logical size, which is exactly what the DIP contract wants. Recorded as
forward exposure, not as an omission.

## Structural side-effect enumeration

The implementation gate's close artifact. What a scale change drags
along, listed so the implementation audits against it rather than from
memory. Rows marked *unchanged* are assertions, not omissions.

1. `WindowState`'s cached scale — updated first (the ordering
   constraint above).
2. The **window rectangle** — set from the OS suggestion via
   `SetWindowPos`.
3. The **client extent** — arrives through the nested `WM_SIZE` and is
   converted to DIP at the seam.
4. **Layout** — re-run over the new DIP client extent. Its results are
   expected to be *identical* when the logical size is preserved; that
   expectation is the integration-test control, not an assumption.
5. **Every widget Visual's offset and size** — rewritten by
   `sync_visuals` at the new scale.
6. The **ScrollView intermediate Visual's** offset and size — same pass;
   the scroll translation `−applied_y` is DIP and is multiplied at the
   write, not accumulated.
7. The **Button label Visual's** offset and size — covered because
   DD-002 moved that write into the sync pass. Had it stayed at
   construction, this row would have been the phase's silent bug.
8. **Every text rasterization surface and its surface brush** —
   re-created by DD-002's walk at the new scale: `Text` nodes and
   Button labels alike.
9. The root's **`SetRelativeSizeAdjustment(1, 1)`** — *unchanged*. It
   relates two physical quantities and is scale-independent.
10. **`InsetClip` insets** (ScrollView, Grid, Box) — *unchanged*: all
    are zero, and zero is scale-invariant. Re-check if a non-zero inset
    is ever introduced.
11. **Signal registry, effect graph, binding state, widget pointers** —
    *unchanged*. A scale change mutates no tree structure, creates and
    destroys no node, and enqueues no signal. It must not enter the
    reactive drain at all.
12. **`MUTATION_CAP` / drain accounting** — *untouched*, following from
    row 11.
13. **Hover and press state** — *unchanged*: no pointer message is
    synthesised. (The pointer may end up over a different widget after
    the resize; the next real `WM_MOUSEMOVE` corrects it. Synthesising
    one is out of scope and belongs with M4-Phase 2's event model if it
    is ever wanted.)

## Invalidation granularity

**Confirmed as-is: the whole window is invalidated.** The current
policy ([layout-engine.md §3.4](../../../../docs/notes/layout-engine.md))
dirties the entire window on a size-affecting change, and a scale change
is the most window-wide event there is — every Visual's projection and
every text surface changes. Sub-tree granularity would buy nothing here
even if it existed.

This is an affirmation, not a decision to revisit §3.4. Its own trigger
(re-layout cost becoming a problem at ~1,000 nodes) is unrelated to DPI
and remains deferred.

## Failure handling

`SetWindowPos` can fail; surface re-creation is WinRT-fallible
(`CreateDrawingSurface`, `BeginDraw`, brush creation).

**Log and survive**, matching the runtime's existing resilient posture
for layout and rendering. A failed re-rasterization leaves a text
surface at the old resolution — visibly blurry until the next change,
and honest about it. A failed `SetWindowPos` leaves the window at the
old rectangle. Neither tears down the window, and `wnd_proc` returns
`LRESULT(0)` regardless, as the message contract requires.

The runtime is **not** put into the `Diverged` state: that state exists
for reactive-engine divergence, and a failed WinRT geometry call is
neither reactive nor unrecoverable. Nothing here creates a new
error path for `wasamo_run` to report, which is why the M3-Phase 2
residual (a layout-time runtime error code) stays non-applicable
([constraints §先行フェーズの残件の点検](../requirements/constraints.md)).

## Verification

- **Windows integration test (mock-free, CI-gated, fail-not-skip).**
  After creating a window, the cached scale equals `GetDpiForWindow`.
  Then, driving a scale change through the handler: **the layout's DIP
  results are unchanged** while the Visual offsets and sizes have moved
  by the scale ratio. The first half is the positive control — it is
  what distinguishes a correct implementation from one that treats
  physical pixels as logical (which would change the DIP results and,
  visibly, the WrapPanel line count).
- **The stated limit, recorded with the test and in the ADR.** A
  synthesised `WM_DPICHANGED` exercises the handling path; it **does not
  prove** that crossing a real monitor boundary delivers the same
  message with a usable suggested rectangle. That half is discharged by
  the owner's human-visible smoke (framing positive control C, literal
  form). Neither is claimed to close AC7's third requirement alone.
- **Assistant-captured positive control C (path form).** Two frames
  across a display-setting scale change on the development machine at
  125%, launched, captured, and analysed — showing text still crisp and
  the logical layout unchanged after the change.

## Forward-compat exposure

1. **Per-window differing scale** — M4-Phase 8. Additive by
   construction: the scale is already per `WindowState`.
2. **`WM_GETDPISCALEDSIZE`** — available if a phase ever wants to
   propose its own post-change size (author-specified window sizing,
   AC11 / M4-Phase 8).
3. **Author-controlled window position / size** — M4-Phase 8 / AC11;
   would make I2 (determine the monitor before creating) worth
   revisiting, since the placement would then be known.
4. **A synthesised pointer update after a scale change** — M4-Phase 2's
   event model, if hover correctness across a resize turns out to
   matter.
5. **Sub-tree invalidation** — deferred with its own unrelated trigger.
6. **A scale-dependent `measure`** (explicit hinting, snapped metrics) —
   M5 text-quality wave; would make step 4's ordering a correctness
   constraint rather than a free choice, which is why the reason is
   recorded above rather than just the order.

## Technical risk re-evaluation

- **The step 1 / step 2 ordering** is the highest-probability defect
  here: it produces one visibly wrong frame, is easy to write the wrong
  way round, and is invisible at 100%. Named in the ADR and asserted
  indirectly by the integration test's post-change Visual assertions.
- **The nested synchronous `WM_SIZE`** means the handler is re-entrant
  through the message loop. `WindowState` is accessed through a raw
  pointer from `GWLP_USERDATA` and the runtime is single-threaded, so
  this is not a data race — but the re-entrancy is real and the scale
  must be committed before it happens.
- **Synthesised-message testing over-claiming** is framing risk R6. The
  mitigation is the recorded split above; the ADR must not, and does
  not, say the integration test discharges the cross-monitor
  requirement.
- **Re-rasterizing the whole tree on every attach** (DD-002's R-a,
  whose second caller this DD is) is bounded work at gallery N on a
  once-per-window event. Not an axis this phase optimises; the benefit
  — the change path runs on every startup rather than only on a rare
  event — is worth more here than the cost.
- **Owner-visible verification needs a second machine.** Framing risk
  R3: the environment exists, the scheduling does not. The implementation
  plan front-loads the delivery of a runnable set (host executable +
  `wasamo.dll` + compiled `.uic`) to the laptop, so the only thing
  waiting on the owner at phase end is one observation.

## Revision history

- 2026-07-28: Initial draft (Status: Proposed).
- 2026-07-28: Accepted flip following owner approval of the phase slate; no
  change requested to the recommendations or their comparisons.
- 2026-07-29: Two dated annotations added in place, body unchanged, on
  owner approval after the M4-Phase 1 T4 independent review. §Context's
  layout-invariance consequence is **qualified** (it holds of a
  controlled client extent, not of the OS-suggested rectangle, because
  the non-client frame does not scale by `s`); §Initial scale
  acquisition's option I1 is annotated because the shipped correction is
  unconditional.
- 2026-07-29 (same day, T4 delta review): the second annotation's
  framing is **corrected**. It originally read "Narrowed … the decision
  this section makes is unchanged", and that was wrong — the condition
  is part of what option I1 *is*, and adjudicating the DD-001 / DD-003
  conflict is a new choice. The §Context qualification is unaffected by
  this correction: it changes no option and stands as an annotation.
- 2026-07-29: **option I1's conditional clause is superseded** by
  [DD-M4-P1-005](./dd-m4-p1-005-unconditional-size-correction.md)
  (Accepted 2026-07-29). The correction is unconditional. **This DD's own
  `Status` stays `Accepted`** — one clause of one option is replaced and
  every decision it makes stands, so the record is superseded *in part*
  in the sense [doc-system.md](../../../cross-milestone/decisions/doc-system.md)
  already uses, not retired.
