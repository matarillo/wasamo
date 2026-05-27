### DD-M2-P6-012 — Re-entrancy and safety-guard placement principle

**Status: Accepted (2026-05-10)**

#### Context

DD-M2-P6-001 (Option D) specifies the **observable** post-divergence
ABI contract: while the runtime is in `Diverged`, every `wasamo_*`
call except `wasamo_runtime_destroy` must behave as a no-op returning
`WASAMO_ERR_REACTIVE_DIVERGED`. The runtime additionally carries
several re-entrancy-sensitive states defined across DD-M2-P6-001 and
DD-M2-P6-005:

- `Diverged` — terminal absorbing state; rejects all but destroy.
- `IN_DRAIN` — Phase 1 mutation convergence loop is active;
  structure-changing ABI returns `WASAMO_ERR_REENTRANT_LOAD`.
- `IN_OBSERVER_CALLBACK` — Phase 3 post-commit observer drain is
  active; state-mutating ABI returns `WASAMO_ERR_OBSERVER_MUTATION`.
- UI-thread confinement — non-UI threads reach the runtime and
  receive `WASAMO_ERR_WRONG_THREAD`.

What none of these DDs specify is the **architectural rule for where
these guards must be enforced in the call stack**: at the ABI boundary
(every exported `wasamo_*` function checks the relevant states at
entry), at the internal state-machine layer (the runtime's mutation,
read, and structure-changing primitives check, with the ABI as a
thin pass-through), or at both layers as deliberate defense in depth.
The current M2 implementation places guards case-by-case —
`check_not_diverged` lives at the ABI layer (`abi.rs`); the `IN_DRAIN`
and `Diverged` checks in `drain_if_outermost` live at the internal
layer (`emit.rs`); the overall pattern is implicit, not documented as
a rule.

The Phase 5 retrospective surfaced a concrete instance of the cost
of leaving the rule implicit. A code path entered through the Win32
message loop reached internal runtime state without crossing a
`wasamo_*` exported function, and therefore bypassed the
`check_not_diverged` guard. The local fix (add the missing check) is
straightforward and orthogonal to this DD; **the architectural issue
surfaced by the bug is that the codebase had no stated invariant the
implementer could have consulted to know whether that entry path
required a guard, nor a convention that would have made the omission
visible at review time.**

This is not a single-bug retrospective item. The same omission shape
recurs whenever a new entry path to runtime state is added that is
not a `wasamo_*` ABI function:

- M3 timer callbacks dispatched from a Win32 timer message.
- M3 async-I/O completions delivered on the UI thread.
- M3+ window-procedure subclassing for additional message types.
- Any future re-entrancy state layered on top of the existing four,
  whether that state is introduced in M3 or beyond.

Without a placement principle, each new state and each new entry
path becomes an independent local decision, and a missed case is
silent. A runtime that should have returned an error instead
executes against state that violates an invariant — with no
guarantee the violation surfaces as a recoverable ABI error rather
than a panic, an assertion failure, or silent state corruption that
manifests later as an apparently unrelated bug.

Re-entrancy and guard placement therefore belong to the same
category of architectural rule as the drain-transaction commit
discipline (DD-M2-P6-001) and the UI-thread-confinement contract
(DD-M2-P6-005): a global runtime invariant whose enforcement
strategy must be stated, not left to per-call-site discretion.
Establishing the principle now — before M3 introduces both new
re-entrancy states and new non-ABI entry paths — converts a class
of implementation oversights into a structural invariant that can
be reviewed and verified uniformly.

#### Options

**Option A — ABI-boundary guards as the single source of
enforcement.** Every exported `wasamo_*` function checks every
relevant runtime state at entry; internal modules trust that no
caller reaches them in a disallowed state. Non-ABI entry paths
(Win32 callback thunks, message-loop reactions, future timer / I/O
completions) must invoke the same guard helpers explicitly before
touching runtime state.

- Pro: enforcement responsibility is concentrated at a small,
  finite, auditable set of entry points; new ABI functions inherit
  a copy-paste-guarded entry pattern; the rule "ABI = guard,
  internal = trust" is easy to state and to review.
- Con: non-ABI entry paths constitute a separate category that must
  remember to invoke the same guard helpers; the Phase 5
  retrospective bug landed in exactly that category. The principle
  reduces but does not eliminate the implementer's responsibility
  on those paths — it only makes the responsibility explicit and
  named.

**Option B — Internal-state-machine guards as the single source of
enforcement.** The runtime's own mutation, read, and structure-
changing primitives check state and refuse work. The ABI layer is a
thin pass-through that forwards arguments and translates internal
refusals to `WasamoStatus` codes. Non-ABI entry paths inherit
guards automatically because every path to state goes through the
same primitives.

- Pro: every path to runtime state is guarded regardless of how it
  was reached; a new entry path cannot bypass the guard because
  there is no guarded layer to bypass — the primitives themselves
  refuse.
- Con: error-reporting context (which `wasamo_*` function was
  called, which argument was bad, which name failed to resolve)
  must be threaded into the primitives or attached out of band;
  guard awareness spreads across `reactive`, `emit`, `registry`,
  and `window` rather than concentrating at one layer.

**Option C — Defense-in-depth at both layers.** ABI-boundary guards
provide diagnostic context and short-circuit obvious violations;
internal-state-machine guards catch any path that reached the
primitives without crossing the ABI. The two layers are
intentionally redundant; their per-state coverage is specified
explicitly so that neither layer assumes the other handled a state
it did not.

- Pro: structural protection against both shapes of failure — the
  Phase 5 retrospective bug shape (missed ABI guard) and any
  future internal refactor that moves work between layers;
  diagnostic context preserved at the ABI layer.
- Con: the same check is written twice; per-layer subset of
  states-to-check must be specified to avoid the "each side assumed
  the other handled it" failure mode; the duplication itself
  carries an audit cost.

**Option D — Compile-time-typed guard tokens.** Introduce a
zero-cost type (e.g. `LiveAccess`, `MutationAccess`) that is
constructible only via the guard helper, and require it as a
parameter on every primitive that touches the relevant state.
Code that reaches a primitive without first acquiring the token
fails to compile.

- Pro: omissions become compile errors rather than runtime bugs;
  the Phase 5 retrospective bug shape (call path reaches state
  without guard) is structurally impossible.
- Con: pervasive API change across `reactive`, `emit`, `registry`,
  and `window`; significant ergonomic cost on every internal
  caller; M2-late introduction collides with the M2-to-M3
  transition.

#### Recommendation

**Choose Option C — role-specified defense in depth.** The accepted
guard-placement principle is:

- The **ABI boundary is the diagnostic boundary**. Exported
  `wasamo_*` functions perform the relevant UI-thread, Diverged,
  `IN_DRAIN`, and `IN_OBSERVER_CALLBACK` checks before mutating or
  structurally changing runtime state. These checks own caller-facing
  `WasamoStatus` return values and last-error messages because that
  layer knows the public function name, argument context, and lifecycle
  exception being applied.
- The **internal runtime boundary is the invariant boundary**.
  Internal entry points that may be reached without crossing an
  exported ABI function must refuse or suppress work when the runtime
  state would make that work invalid. In M2 this is concretely
  represented by `emit::drain_if_outermost()` suppressing re-entrant
  drains while `IN_DRAIN` is set and suppressing all drain phases after
  `RuntimeHealth::Diverged`.
- **Runtime-owned non-ABI entry paths are first-class runtime entries**,
  not exceptions to the rule. The Win32 message-loop path in
  `lib.rs::run()` and future M3 timer / async-I/O / additional
  window-procedure paths must enter runtime state through an internal
  invariant boundary rather than relying on ABI-only guards they do not
  cross.
- **Cleanup / destroy paths remain explicit exceptions.** Any operation
  that is allowed after `Diverged` must be named at its entry boundary
  and documented as a lifecycle exception; the exception does not imply
  general permission to touch runtime state after divergence.

Option A is rejected because ABI-only enforcement still leaves the
Phase 5 omission shape as a per-entry-path obligation: every non-ABI
entry must remember to call an ABI-shaped guard helper even though it
does not cross the ABI. Option B is rejected because moving all guards
into internal primitives would either lose ABI diagnostic precision or
force public-call context through otherwise local runtime APIs. Option
D is not required for M2 acceptance: typed guard tokens are the
strongest structural answer, but their blast radius is disproportionate
to the Phase 7 acceptance need now that Option C gives both a
diagnostic boundary and an invariant boundary.

DD-012 acceptance therefore updates `docs/architecture.md` with this
principle as a global runtime invariant. Implementation alignment is
scoped to ensuring existing M2 paths match the accepted rule and adding
focused guard-placement tests; broader tokenisation or callback-surface
redesign is not part of this DD.

#### Forward-compat exposure

Low-medium after acceptance. The placement rule is now explicit before
M3 adds timer, async-I/O, and additional Win32 message handling, so
those new surfaces inherit the Option C responsibility split instead
of re-deciding guard placement locally.

Residual exposure remains in two places. First, each new non-ABI entry
path must name the internal invariant boundary it crosses; omission is
now review-visible but not compile-time impossible. Second, typed guard
tokens remain a M3+ revisit trigger if the number of internal entry
paths grows enough that runtime checks and review discipline no longer
provide sufficient structural confidence.

---
