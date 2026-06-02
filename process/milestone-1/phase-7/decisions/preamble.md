# Phase 7 — Language Bindings: Architecture Decisions

**Phase:** 7 (Language bindings — C / Rust / Zig)
**Date:** 2026-04-30
**Status:** Accepted (2026-04-30)

## Context

Phase 7's acceptance criterion is derived from
[VISION §7 M1](../../VISION.md#7-roadmap--milestones) and
[process/_roadmap.md M1](../../../_roadmap.md#m1-proof-of-concept):
**"Hello Counter runs in three languages: C, Rust, and Zig."**
Phase 7 produces the **bindings** that Phase 8 consumes; Phase 8 then
writes the actual `counter` apps in each language.

The C ABI is already shaped and shipped in Phase 6
([`bindings/c/wasamo.h`](../../bindings/c/wasamo.h),
[`docs/abi_spec.md`](../../../../docs/abi_spec.md), Accepted). On the C side Phase 7
adds only sample-build infrastructure, not new ABI. The substantive
work is on the Rust and Zig wrapper sides.

Two pre-existing facts complicate the Rust side and motivate most of
this ADR:

1. The `wasamo` crate is configured `crate-type = ["cdylib", "rlib"]`
   ([`wasamo/Cargo.toml`](../../wasamo/Cargo.toml#L7)). Phase 4/5
   examples (`phase4_visual_check`, `phase5_visual_check`) consume the
   `wasamo` rlib **directly**, calling `Runtime`, `Window`, `Button`,
   etc. as Rust types. Those names overlap with what a host-side
   "safe Rust wrapper" would naturally want to call itself.
2. The rlib's Rust-native API (`Runtime::init`, `WindowState`, widget
   constructors) is currently unmarked. Functionally it is equivalent
   to the `WASAMO_EXPERIMENTAL` C-ABI layer — both are imperative
   builders that exist because M1 `wasamoc` is parser-only and host
   code must construct trees by hand. But the rlib path has no
   experimental marker and no documented stability story.

The roadmap Phase 7 task list ([process/_roadmap.md M1](../../../_roadmap.md#m1-proof-of-concept))
has seven items. Per
[Pre-doc discipline](../../../README.md) those are
working hypotheses; this ADR revisits them against the acceptance
criterion. The decisions below are sequenced so that DD-P7-001
(Rust binding architecture) determines the shape of the rest.

---

## Implementation-pattern sketches (no decision; for the implementer)

The following are not ADR-level decisions but should be sketched
here so the implementation step does not re-discover them.

### Callback trampolines (Rust safe wrapper)

The C ABI takes `(fn, user_data, destroy_fn)` triples. The safe
wrapper accepts a Rust closure (`FnMut`) and:

```rust
fn connect_clicked<F: FnMut() + 'static>(&self, f: F) -> Connection {
    let boxed: Box<dyn FnMut()> = Box::new(f);
    let raw: *mut c_void = Box::into_raw(Box::new(boxed)) as *mut c_void;
    extern "C" fn trampoline(_w: *mut WasamoWidget, _args: *const WasamoValue,
                              _n: usize, ud: *mut c_void) {
        let f = &mut **(ud as *mut Box<dyn FnMut()>);
        f();
    }
    extern "C" fn drop_box(ud: *mut c_void) {
        unsafe { drop(Box::from_raw(ud as *mut Box<dyn FnMut()>)); }
    }
    // wasamo_signal_connect(..., trampoline, raw, drop_box, &mut token);
    todo!()
}
```

`destroy_fn` is what makes this leak-free; that is the binding-side
justification for DD-P6-003.

### `!Send` / `!Sync` markers

The C ABI is strict UI-thread-affinity (DD-P6-004). The safe wrapper
must mark every handle type `!Send` and `!Sync` so the Rust
borrow-checker prevents accidental cross-thread sends:

```rust
pub struct Window {
    raw: *mut sys::WasamoWindow,
    _not_send: PhantomData<*const ()>,
}
```

(The `*const ()` PhantomData makes it `!Send + !Sync` automatically.)

### `WasamoValue` in safe Rust

The C `WasamoValue` tagged union maps to a Rust enum:

```rust
pub enum Value<'a> {
    None,
    I32(i32),
    F64(f64),
    Bool(bool),
    String(&'a str),     // borrows for callback duration only
    Widget(&'a Widget),  // ditto
}
```

Callback parameters are `&[Value<'_>]` with a lifetime tied to the
closure invocation. Hosts wanting to retain a string copy it inside
the closure. This matches DD-P6-007 (memory ownership) exactly:
the runtime owns the storage, the closure borrows for its duration.

---

## Summary of recommended decisions

| ID | Topic | Recommendation |
|---|---|---|
| DD-P7-001 | Rust binding architecture | Option B — `wasamo-sys` + safe wrapper |
| DD-P7-002 | Crate naming / rlib status | Option A — rename runtime to `wasamo-runtime`; rlib path = internal/dev |
| DD-P7-003 | Experimental layer in bindings | Option A — `wasamo::experimental` module split; same in Zig |
| DD-P7-004 | Phase 7 scope | Option A — Hello-Counter-minimal |
| DD-P7-005 | Zig binding strategy | Option A — `@cImport` over `wasamo.h` |
| DD-P7-006 | C sample shape | Option A — header + import lib + CMake template; demo in Phase 8 |

## Revised Phase 7 ROADMAP task list (proposed)

If the above decisions are Accepted, the Phase 7 task list in
[process/_roadmap.md](../../../_roadmap.md#m1-proof-of-concept) is revised to reflect the
crate rename and the scope split:

- [ ] `process/milestone-1/phase-7/decisions/preamble.md` — owner agreement (this doc)
- [ ] Workspace: rename runtime crate `wasamo` → `wasamo-runtime`;
      `[lib].name = "wasamo"` keeps `wasamo.dll` / `wasamo.dll.lib`
      filenames stable. Update Phase 4/5 examples' `Cargo.toml` deps.
- [ ] `wasamo-sys` crate: raw `extern "C"` bindings to `wasamo.h`;
      `build.rs` links `wasamo.dll.lib`; coverage = Hello Counter
      minimum (DD-P7-004).
- [ ] `wasamo` (safe wrapper) crate at `bindings/rust/`: stable-core
      surface at crate root; `wasamo::experimental` for the
      experimental constructors and `button_set_clicked`. `!Send`
      handles, closure-capable callbacks via trampoline+drop.
- [ ] `bindings/zig/wasamo.zig`: `@cImport(wasamo.h)` + Zig-idiomatic
      wrappers (slices, error sets, tagged unions); same module
      split as Rust.
- [ ] `bindings/c/CMakeLists.txt` template; CI extended to build the
      existing smoke TU through CMake (MSVC + clang-cl).
- [ ] `CONTRIBUTING.md` documents how to add a binding (sys/safe
      pair pattern; experimental module convention; what coverage
      level is expected per phase).
- [ ] `docs/architecture.md` bindings section: crate layout updated;
      rlib path documented as internal/dev-only; experimental module
      convention recorded.
- [ ] CI: Zig install step; CMake build step; both link against
      `wasamo.dll.lib`.
