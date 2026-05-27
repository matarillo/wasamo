### DD-P4-006 — Button clicked callback model

**Status:** Accepted

**Context:**
When a button is clicked, the host code must be notified. The callback
must work for both the Rust-native API (examples, `bindings/rust`) and
the C ABI (Phase 6 `wasamo.h`). The design should not introduce a
second, incompatible mechanism.

**Options:**

Option A — Store `Box<dyn Fn()>` on the Rust side; C ABI adds a separate setter
- `ButtonNode` (the internal type tracking button state) stores a
  `clicked_fn: Option<Box<dyn Fn()>>`.
- Rust-native callers pass a closure: `button.set_clicked(|| { ... })`.
- The C ABI Phase 6 function will be:
  `wasamo_button_set_clicked(widget, cb: unsafe extern "C" fn(*mut c_void), userdata: *mut c_void)`
  which wraps the C function pointer + userdata into a `Box<dyn Fn()>`.
- What you gain: Rust API is ergonomic; the C ABI wrapper (added in Phase 6)
  is a thin adapter; single internal dispatch path.
- What you give up: Phase 4 ships only the Rust-native setter; the C ABI
  wrapper is added in Phase 6 when all other ABI functions are finalized.

Option B — Only expose C ABI function pointer / userdata from the start
- What you gain: Forces early ABI thinking.
- What you give up: Rust-native callers must write `unsafe extern "C"` blocks
  for simple closures; awkward; misaligns with the Rust binding layer planned
  in Phase 7.

**Decision:** Option A — `Box<dyn Fn()>` internally; C ABI adapter in Phase 6.

---
