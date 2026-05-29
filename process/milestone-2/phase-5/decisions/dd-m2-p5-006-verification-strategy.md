### DD-M2-P5-006 — Verification strategy

**Status:** Accepted

**Context:**
[docs/notes/headless-verification.md](../notes/headless-verification.md)
records the M2 stance: do not build a general-purpose headless
backend; cover pure-logic surfaces with phase-specific test fixtures;
GUI-observable behaviour is verified by manual exercise on a visible
desktop. Phase 5 is the trigger phase that note flagged for
re-evaluation: "Phase 5 着手時に reactive 経路の検証が unit test
単独で覆えるか再評価". This DD answers the trigger.

The reactive engine has a large pure-logic surface (Signal storage,
dependency tracker, dirty-set, drain loop, evaluator wiring) and a
small Visual-Layer-bound surface (the bound widget actually renders
the new text). Phase 4 established a precedent (Slot/Children mirror
test pattern, [CLAUDE.md](../../CLAUDE.md) testing rule's optional
mirror clause); this DD decides how far to lean on it for Phase 5.

**Options:**

Option A — Pure-logic only; no new mirrors; GUI manual confirms end-to-end (recommended)
- Test surface: Signal `get`/`set`, Effect creation/disposal,
  dependency-graph mutation across re-runs, dirty-set drain loop
  (including iteration-cap), `with_batched_writes` deferral,
  `BindingEvalContext` over `HandlerExpr` (read-only mode rejects
  writes; reads register dependencies). Effects are tested with
  closure bodies that record observable side effects into a
  test-side `Vec` — no widget property writes in unit tests.
- The Phase 4 Slot/Children mirror pattern is **not** extended;
  Phase 5 does not introduce widget-tree mutation that would need
  it. The Effect closures that, in production, write through
  `set_property` are stubbed in tests with closures that push to a
  log Vec.
- GUI verification: at Phase 5 close, run the M1 counter example
  through a Phase-5-aware code path (Phase 6 is not yet present,
  so a small experimental harness wires a `Signal` to a Text
  widget by hand) on a visible desktop and confirm `count++`
  updates the label. Recorded as Phase 5 GUI checkpoint in the
  m2-plan; A2 acceptance is fully confirmed at Phase 6 close
  (counter.ui-driven).

- What you gain: Stays inside [CLAUDE.md](../../CLAUDE.md) testing
  rules without further interpretation. Test fixtures are narrow
  and phase-local. No mirror struct to maintain. The pure-logic
  surface is large enough that a "binding evaluator over Signal +
  Effect with deferred drain" suite gives high confidence; the
  remaining GUI-observable bit (the widget actually re-renders)
  is exercised by the manual checkpoint and by Phase 6 e2e.
- What you give up: A2 is not closed by unit tests alone. The
  manual GUI checkpoint at Phase 5 close is the close criterion;
  CI green is necessary but not sufficient.
- **Technical risk: Low.** The test surface is pure Rust;
  closures-as-side-effect-loggers is a standard pattern. The
  manual checkpoint is the same shape as Phase 6's manual GUI
  verification — owner runs counter on RDP / physical desktop and
  observes the click → label update.

Option B — Extend the Phase 4 mirror pattern to cover bound widget property writes
- Add a test-only mirror of `WidgetNode` (or a narrow sub-struct)
  that supports `set_property` and records writes; bind an Effect
  to the mirror; assert the mirror's recorded write set after a
  Signal write + drain.

- What you gain: Tests demonstrate "Effect ran and wrote the
  property" rather than "Effect ran and incremented our counter
  closure" — closer in shape to the production path.
- What you give up: A new mirror that has to track `WidgetNode`'s
  property storage shape, drift risk against production, and
  maintenance burden as M3 adds property types. The Effect closure
  in production calls the same `set_property` function by name;
  testing through a mirror tests the bridging code, not the
  reactive engine. Diminishing-returns — the bridging code is one
  line per binding (Effect's closure is ~3 lines).
- **Technical risk: Low–medium.** The risk is mirror drift, the
  same issue that motivates the [CLAUDE.md](../../CLAUDE.md) rule
  to prefer extracting free functions over mirrors.

Option C — Build a "no-Compositor" runtime mode and integration-test through it
- Per [headless-verification.md (ii)](../notes/headless-verification.md):
  introduce a runtime mode where `WidgetNode` is fully constructed
  but no Compositor / Visual / DirectWrite is created. Tests
  exercise full property write → reactive drain → property store
  end-to-end without the OS surface.

- What you gain: Higher-fidelity tests; A2-shaped verification in
  CI (modulo the actual rendering bit).
- What you give up: A "Visual on / Visual off" two-mode runtime
  is exactly what
  [headless-verification.md](../notes/headless-verification.md)'s
  long-form analysis rejected for M2 — DD-V-001-era posture
  (no abstraction over Visual Layer) and infrastructure cost.
  Building it as a Phase 5 sub-task expands scope significantly
  and would need its own ADR.
- **Technical risk: Medium.** The risk is scope creep into a
  separate runtime mode that has its own design surface (which
  events fire, how the message loop is faked, which OS-bound
  paths short-circuit). Out of M2-Phase 5 budget; if needed,
  a separate vision decision record is the right shape.

**Recommendation:** **Option A.**

Pure-logic test fixtures are sufficient for the engine surface and
align with [CLAUDE.md](../../CLAUDE.md) testing rules without
further reinterpretation. The Phase 4 mirror pattern was used
sparingly (Slot/Children — small enough state to mirror without
drift risk); Phase 5 does not present a target small enough to
mirror cleanly without dragging in `WidgetNode`'s full property
storage. The GUI manual checkpoint at Phase 5 close, plus full A2
verification at Phase 6 close, completes the verification chain.

The binding-evaluator integration with `HandlerExpr` is itself
pure-logic-testable: a `BindingEvalContext` with mock storage,
fake signals, and assertion that read calls are tracked correctly.
This is where the Phase 5 testing lift is concentrated, and it is
done entirely without OS-bound types.

`headless-verification.md` is **not** updated to flag a new
trigger; the M2-stance "do not build a headless backend" survives
Phase 5 by virtue of Option A working.

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are post-1.0 hot-reload CI verification and post-1.0
binding-conformance test (Swift / Go community track).

- Option A leaves the door open to building a headless mode later
  if a foreseeable future event (hot reload in CI, binding
  conformance tests) demands it; the engine internals don't lock
  in any assumption that prevents a later headless mode.
- Option B's mirror pattern has the same forward-compat
  property; the difference is in the M2 cost, not the M3+ cost.
- Option C builds the headless mode now, which has the same
  long-term value but pulls scope forward into Phase 5 without an
  M2 driver.

This axis reinforces Option A: defer infrastructure that has no
M2 driver; the runtime architecture remains amenable to a
headless mode if a real driver appears.

**Technical-risk re-evaluation:** Option A's risk is the smallest;
the test fixtures are narrow and phase-local. Option B's mirror
drift is bounded but real. Option C's scope is out of phase
budget. Risk reinforces Option A.

---
