### DD-M2-P3-002 — Coexistence with `wasamo_signal_connect`

**Status:** Accepted

**Context:**
Inline handlers in the DSL and host-registered listeners through
`wasamo_signal_connect` (DD-P6-002) are two ways to react to the
same widget signal. A button has `clicked => { root.count += 1 }`
in `counter.ui`; an instrumentation host might also call
`wasamo_signal_connect(button, "clicked", on_click_metric, …)` to
log every click. The runtime must define what happens on click.

The question is conceptual, not pick-an-option-from-three: are
inline handlers the *same path* as host listeners (consume one
shared listener list), or are they a *separate path* that fires
alongside the listener list?

**Options:**

Option A — Single list, inline handler enqueued first
- The runtime treats the inline handler as a synthetic
  `wasamo_signal_connect` registration done at widget construction.
  Click fires every entry in the listener list in registration
  order; the inline handler entry is always first because it is
  registered first.

- What you gain: One mechanism. Host can introspect/disconnect the
  inline handler if `wasamo_signal_connect` returns a token.
  Conceptually clean if you accept "inline handler is just a
  built-in subscriber".
- What you give up: Surfaces the inline handler as a token the host
  can disconnect — a footgun ("why does my counter stop
  incrementing?"). Mixes two distinct artifacts (DSL-author intent
  vs host-author observation) into one orderable list. Forces a
  decision on whether the inline handler appears in the host's
  listener enumeration.
- **Technical risk: Low.** Pure mechanism choice; ordering rules
  are stable. The risk is design-quality, not technical.

Option B — Separate paths; inline runs first, listeners run after (recommended)
- The runtime stores the inline handler IR on the widget directly.
  On signal fire, the runtime first evaluates the inline handler in
  the interpreter (DD-M2-P3-001 = A), then iterates the host
  listener list registered via `wasamo_signal_connect` and dispatches
  each.
- The inline handler is **not** a token returned to the host and is
  **not** disconnectable from the host side. It is part of the DSL
  contract for the component.

- What you gain: Each path expresses what it actually is — inline =
  DSL-author intent, listener list = host-side observation. Hosts
  see a coherent "inline first, then me" ordering: any state change
  the handler causes is visible to the host listener that fires next
  on the same click. Aligns with the natural DSL reading: `clicked
  => { root.count += 1; }` is the component's own response to its
  own button being clicked, not a subscription a stranger should be
  able to revoke.
- What you give up: Two pieces of state on the widget rather than
  one (inline-handler slot + listener list). Slight asymmetry
  between built-in widget signals (always potentially have an inline
  handler) and component-declared signals (in M2 scope, never have
  an inline handler at the declaration site, only at instantiation
  sites — but `counter.ui` does not instantiate sub-components, so
  this gap is invisible at M2).
- **Technical risk: Low.** Two clearly-separated lookups on emit;
  documented ordering ("inline before host"). Each path is small.
  Less risk than Option A because the host cannot accidentally
  disconnect DSL-author code.

Option C — Separate paths; listeners run first, inline after
- Same as B but reversed order: host listeners observe pre-handler
  state, then the handler runs.

- What you gain: Hosts can implement "observe what happened, then
  let DSL react" patterns.
- What you give up: Counterintuitive: the DSL author wrote the
  handler as the component's response, and observation typically
  watches the *result* of a response. Reversing the order forces
  every host listener to remember it sees the pre-state. No M2
  acceptance criterion benefits from this order; Counter's host
  doesn't even use `wasamo_signal_connect`.
- **Technical risk: Low** (same as B); design-quality risk is
  higher because the order is the surprising one.

**Recommendation:** **Option B.**

Inline handlers and host listeners are different artifacts at
different layers; treating them as a single list (Option A) compresses
two roles into one and creates the disconnect-the-handler footgun.
Between B and C, B's order ("DSL response first, then host
observation") matches the natural reading of the DSL and is the
order any host listener would assume by default if not told
otherwise.

The order rule is **documented in `architecture.md` §6 (or the M2
revision thereof) as a runtime contract**, so the M2-Phase 5
reactive engine and any future host-listener author can rely on it.

**Out of scope:** What happens when a future DSL feature lets the
inline handler `return false` or otherwise short-circuit further
emission. M2 has no such mechanism in `counter.ui`. If/when added,
the contract here becomes "inline handler runs; if it requests
short-circuit, host listeners are skipped" — a non-breaking
addition.

**Technical-risk re-evaluation:** All three options are
mechanically straightforward; the risk axis is design-quality
rather than implementability. Option B is the lowest design risk
because it isolates two unrelated concerns and uses the natural
order. Risk reinforces the recommendation.

---
