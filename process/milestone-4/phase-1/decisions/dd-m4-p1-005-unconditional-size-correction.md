# DD-M4-P1-005 — The DIP window-size correction is unconditional

**Status:** Accepted
**Phase:** M4-Phase 1
**AC:** None directly. This record exists to resolve a conflict between
two Accepted decisions that surfaced during implementation
(AC7's first and third requirements own the decisions themselves).

**Supersedes:** the conditional clause of
[DD-M4-P1-003](./dd-m4-p1-003-dpi-change-propagation.md) §Initial scale
acquisition, option I1 — *"and **if the scale is not 1** apply
`size × s` with `SetWindowPos`"*. **Nothing else in DD-M4-P1-003 is
superseded**: create-then-correct, the flash-free reasoning, the
`WM_DPICHANGED` step ordering, the failure posture and the enumeration
all stand as written.

## Context

DD-M4-P1-003 was drafted before any of this phase's code existed, and
option I1's sentence carries a conditional that reads as incidental:
create at the requested numbers, read the DPI, *and if the scale is not
1*, correct. Implementation (T4) shipped the correction **unconditional**
and recorded the departure in the implementation log as a narrowing.

The T4 independent review, and then its delta review, rejected that
handling in two steps, and both are worth stating because the second is
the reason this record exists rather than an annotation.

- The difference is **observable**. A size-preserving `SetWindowPos`
  still dispatches `WM_WINDOWPOSCHANGING` and `WM_GETMINMAXINFO` —
  measured, in T4's own probe, on the very run that was cited to argue
  the change was invisible. They reach `wnd_proc` before `GWLP_USERDATA`
  is installed and go to `DefWindowProcW`, so no Wasamo runtime state
  and no Wasamo-exposed callback observes them; a native host that
  installs a `WH_CALLWNDPROC` hook on the thread can. "Invisible to the
  runtime" is not "invisible".
- The condition is **part of what option I1 is**. A reader implementing
  I1 as written does not produce the shipped behaviour, which is the
  operative test for a changed decision rather than a clarified one. And
  choosing between DD-M4-P1-001's unconditional-machinery property and
  DD-M4-P1-003's conditional clause is itself a choice, made in an
  implementation log where no decision belongs.

[process/README.md](../../../README.md) makes `decisions/` immutable
under the supersede rule, so the correction lands here.

## Decision dependency summary

Consumes [DD-M4-P1-001](./dd-m4-p1-001-dpi-awareness-declaration.md)
§Failure handling (the unconditional-machinery property). Supersedes one
clause of [DD-M4-P1-003](./dd-m4-p1-003-dpi-change-propagation.md).
Provides nothing new downstream — the shipped behaviour is already what
T5 through T12 build on.

## The conflict

Two Accepted decisions disagree about whether the correction may branch.

**DD-M4-P1-001 §Failure handling** tolerates a failed awareness
declaration, and the property that makes that safe is structural, stated
there in terms: *"the conversion machinery is unconditional… The runtime
never asks 'did my declaration succeed'… **There is no second code path
to keep correct**, and the scaled path is exercised on every machine
including 100% ones."* That is not decoration. It is the whole reason
option F3 — tolerate silently and assume scale 1 — was rejected as *"the
one option that can be wrong."*

**DD-M4-P1-003 option I1** introduces exactly such a second path: a
branch whose taken arm runs only when the scale is not 1.

The conflict was invisible at drafting time because both readings
produce identical behaviour in the world the ADRs were written for. It
becomes real at T9, when the declaration lands and the scale stops being
1 — which is also the first moment either arm could be tested.

## Options

- **N1 — Unconditional.** `window::create` always calls `SetWindowPos`
  with `window_size_to_physical(width, height)`.
  - What you gain: **no second code path**, which is what DD-001's
    tolerance argument relies on. The path every window creation takes
    is the same path at every scale, so it is exercised on every machine
    including the 100% CI runners, and there is no arm that first
    executes at T9 on the one seam all three example hosts and both
    public window-create entries pass through. The behaviour is also
    what already shipped and what the phase's evidence was gathered
    against.
  - What you give up: one `SetWindowPos` per window creation that
    changes nothing, and two window messages dispatched at scale 1 that
    a guarded implementation would not send. Both are real and neither
    is observable to the runtime; a host with a message hook can see the
    messages.
- **N2 — Restore the guard** (`if scale.factor() != 1.0`).
  - What you gain: DD-M4-P1-003 stands exactly as written, with no
    supersede and no new record. The syscall and the two messages
    disappear at 100%.
  - What you give up: an authored branch that **no test can fire in
    either direction before T9**, on `window::create`. This is precisely
    the failure the phase's implementation gates arm trap #4 against,
    and T4 measured why it is worse here than it looks: at scale 1 a
    size-preserving `SetWindowPos` dispatches no `WM_SIZE` at all, so
    the ordering question the correction's placement answers has no
    answer to get wrong before T9 either. A guard would add a second
    unexercised dimension to a site that is already unverifiable.
    Rejected on merit.
- **N3 — Keep the guard but drive it from a test seam** so both arms are
  exercisable before T9.
  - Rejected on proportionality. It manufactures test surface to defend
    a branch that exists for no product reason — the guard buys a
    syscall, not a behaviour — and DD-M4-P1-003's own §Verification
    already places the synthesised-scale work at T8 through the real
    handler rather than through injected state. Adding a seam here to
    justify a branch inverts that.

## Comparison

The decision turns on which of two costs is permanent. N1 costs one
redundant syscall per window creation, forever but bounded and
unobservable to the product. N2 costs an untested branch on the runtime's
single window-creation path, and that cost does not expire — it converts
into a defect the first time someone edits either arm believing the other
is equivalent, which is the class trap #4 exists to name. Under the
product-merit prior, an unconditional path that is always exercised beats
a conditional one that is never exercised.

The tie-breaker points the same way and is not needed to decide it:
DD-M4-P1-001's structural argument is load-bearing for a decision
(tolerate a failed declaration) that has already been accepted, while
I1's conditional is a clause inside an option's description that carries
no argument of its own. Where a stated property and an unargued clause
conflict, the property wins.

## Recommendation

**N1 — the correction is unconditional.** DD-M4-P1-003 option I1 is
superseded to read: *create the HWND at the requested numbers, read
`GetDpiForWindow`, and apply `size × s` with `SetWindowPos`* — with no
condition on the scale.

**Nothing else changes.** The correction still lives inside
`window::create` (not at the ABI entry point), still runs before the
`GWLP_USERDATA` install, still uses
`SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE`, and still logs and
survives a failure. `AdjustWindowRectExForDpi` remains available for a
future client-size form. The `WM_DPICHANGED` path is untouched and must
**not** inherit `SWP_NOMOVE`.

## Verification

No new verification surface. The claim being made is the absence of a
branch, which is checked by reading `window::realize_dip_window_size` —
it contains no conditional — and by the T4 close gate's trap-#4 artifact
recorded in [implementation/log.md](../implementation/log.md) §T4. The
two messages dispatched at scale 1 are recorded there as measured, in
the trap-#2 structural side-effect enumeration.

## Forward-compat exposure

1. **Author-controlled window position / size** (M4-Phase 8 / AC11) —
   would revisit DD-M4-P1-003's I2 (determine the monitor before
   creating), at which point the correction may disappear rather than
   become conditional. This record does not stand in the way.
2. **A client-size window attribute** (M4-Phase 8 / AC11) — arrives
   through `AdjustWindowRectExForDpi` on the same unconditional path.
3. **A future cost concern** — if one redundant `SetWindowPos` per
   window ever matters (many windows created per second, which no M4
   phase plans), the fix is to compare the computed physical size against
   the requested one *inside* `realize_dip_window_size` rather than to
   reintroduce a scale test at the call site. Recorded so the next reader
   does not re-derive N2 from the cost side.

## Technical risk re-evaluation

- **Superseding a clause rather than a decision** risks reading as
  license to supersede wording generally. The boundary applied here: a
  reader implementing the original text does not obtain the shipped
  behaviour. Annotations that fail that test — such as the
  layout-invariance qualification added to DD-M4-P1-003 §Context on the
  same day — stay annotations.
- **The record is filed after the code shipped**, not before. Honest
  framing: the implementation was right and its *justification* was
  filed in the wrong document. Nothing is being retrofitted to match
  code that was not thought through; T4's start gate recorded the
  no-branch decision with its reason before an approach was chosen, and
  what was missing was the recognition that the reason belonged in a
  decision record.
- **This is the ADR set's first supersede**, and it establishes by use a
  distinction the process documents do not name — *superseded* (an
  option is re-chosen) versus *qualified* (a decision stands and a
  statement around it is corrected). Whether
  [workflow.md](../../../procedures/workflow.md)'s status vocabulary
  needs that distinction is a process question, carried to the
  M4-Phase 1 phase-end batch rather than decided here.

## Revision history

- 2026-07-29: Initial draft (Status: Proposed). Filed on owner approval
  of the successor route, following the T4 delta review's finding that
  an in-place annotation could not carry a changed option. The owner has
  approved the **substance** (keep the correction unconditional; file a
  successor rather than restore the guard); the `Accepted` flip awaits
  their review of this text.
- 2026-07-29: Accepted flip following owner review of this text; no
  change requested to the recommendation, its options, or the stated
  supersede scope (option I1's conditional clause and nothing else).
