### DD-P6-002 — Signal model

**Status:** Accepted

**Context:**
A "signal" in wasamo is a named, typed event a component can declare
and emit (e.g. `Button.clicked`). The C ABI must let a host
language register a callback to receive emissions. The model must
work for both component-declared signals (DSL `signal foo(i32)`)
and built-in widget signals (`Button.clicked`), and must survive
deferred question (a): whether DSL inline handlers run host-side or
runtime-side. (Inline handlers are a separate path — they are bodies
of code, not host callbacks. Signals are the host-callback path.)

**Options:**

Option A — String-keyed, untyped payload (recommended)
`wasamo_signal_connect(widget, "clicked", callback, user_data, &out_token)`
where `callback` has signature
`void (*)(WasamoWidget*, const WasamoValue* args, size_t arg_count, void* user_data)`.
`WasamoValue` is a tagged union over the M1 property-type set
(i32, f64, bool, string-view, widget-handle).

- What you gain: One signature handles every signal regardless of
  arity or types. Codegen (M2) can produce typed wrappers on top
  without ABI changes. Untyped payload + string key is the
  GTK / GLib idiom — well-understood by C-ABI binding authors.
- What you give up: Runtime type-check cost (small) and the ergonomic
  loss of compile-time mismatch detection at the C boundary. Both
  are recovered by generated bindings in Rust/Zig/Swift.

Option B — Per-signal typed C function pointers
`wasamo_button_set_clicked(button, void (*)(WasamoButton*, void*), user_data)`,
one per widget×signal pair.

- What you gain: Compile-time type safety at the C boundary. Cheaper
  per-emission (no value packing).
- What you give up: Does **not** scale to component-declared signals
  — a `signal foo(i32, string)` defined in `.ui` cannot have a
  hand-written `wasamo_*_set_foo` because the runtime has no static
  knowledge of it. Forces all DSL-declared signals into a separate
  mechanism, fragmenting the model. This is exactly the M1
  experimental shape (`wasamo_button_set_clicked` already exists)
  that the framing wants to keep **out** of the stable core.

Option C — Single global dispatch
`wasamo_set_signal_dispatcher(callback)` — one host-side router for
all signals from all widgets, with `(widget, signal_name, args)` in
the payload.

- What you gain: Smallest ABI surface (one register call).
- What you give up: Per-widget connection lifetime is awkward —
  disconnecting one connection means the dispatcher must keep its
  own table. Hostile to bindings that want per-callback ownership
  (Rust closures, Swift `@escaping`). Doesn't compose with multiple
  bindings in the same process.

**Recommendation:** **Option A.** The string-key + tagged-value
model is the only one that uniformly handles built-in and
component-declared signals, survives deferred (a)/(b), and is the
established C-ABI idiom for this shape of problem. Option B's
`wasamo_button_set_clicked` form is preserved in the **M1
experimental** layer for Phase 8 simplicity but is explicitly not
part of the stable core. Option C is rejected for ownership
reasons.

---
