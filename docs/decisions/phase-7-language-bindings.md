# Phase 7 — Language Bindings: Architecture Decisions

**Phase:** 7 (Language bindings — C / Rust / Zig)
**Date:** 2026-04-30
**Status:** Agreed (2026-04-30)

## Context

Phase 7's acceptance criterion is derived from
[VISION §7 M1](../../VISION.md#7-roadmap--milestones) and
[ROADMAP M1](../../ROADMAP.md#m1-proof-of-concept):
**"Hello Counter runs in three languages: C, Rust, and Zig."**
Phase 7 produces the **bindings** that Phase 8 consumes; Phase 8 then
writes the actual `counter` apps in each language.

The C ABI is already shaped and shipped in Phase 6
([`bindings/c/wasamo.h`](../../bindings/c/wasamo.h),
[`docs/abi_spec.md`](../abi_spec.md), Agreed). On the C side Phase 7
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

The ROADMAP Phase 7 task list ([../../ROADMAP.md L172-L180](../../ROADMAP.md#L172-L180))
has seven items. Per
[Pre-doc discipline](./README.md#pre-doc-discipline) those are
working hypotheses; this ADR revisits them against the acceptance
criterion. The decisions below are sequenced so that DD-P7-001
(Rust binding architecture) determines the shape of the rest.

---

### DD-P7-001 — Rust binding architecture

**Status:** Agreed

**Context:**
Phase 8's Rust "Hello Counter" needs *some* Rust API to drive the
runtime. There are three natural shapes, and the choice determines
whether M1's "C ABI verified in three languages" claim is real or
hollow.

**Options:**

Option A — Rlib path only (no FFI; what Phase 4/5 examples already do)
The Rust example links the `wasamo` rlib statically and calls Rust
types directly. No `wasamo-sys`, no safe wrapper.

- What you gain: Zero new crates. Phase 4/5 examples already
  demonstrate this works. Smallest delta from current state.
- What you give up: M1's acceptance criterion is "C ABI verified in
  three languages." Rust never crosses the C ABI in this option, so
  the C-Rust-Zig triplet collapses to "C and Zig exercise the C ABI;
  Rust separately exercises a Rust-native rlib." That is a weaker
  validation than the milestone claims. Also leaves the rlib's
  unmarked stability story unresolved.

Option B — `wasamo-sys` (raw FFI) + safe wrapper (recommended)
Two new crates:
- `wasamo-sys` — raw `extern "C"` declarations matching `wasamo.h`,
  links dynamically to `wasamo.dll` via `wasamo.dll.lib`.
- A safe wrapper crate (name TBD — see DD-P7-002) — translates the
  C ABI into idiomatic Rust (`Result<_, WasamoError>`, RAII handles,
  closure-capable callbacks).

Phase 8's Rust counter consumes the safe wrapper, which consumes
`wasamo-sys`, which calls into `wasamo.dll`. The rlib Rust-native
API is **not** removed but is repositioned (see DD-P7-002).

- What you gain: Rust genuinely traverses the C ABI on the same
  path C and Zig do. M1's three-language claim is real. The safe
  wrapper is also the artifact that proves the C ABI is *usable*
  from a memory-safe language — a non-trivial check on DD-P6-003
  (callback contract) and DD-P6-007 (memory ownership).
- What you give up: Two new crates, plus the crate-name collision
  problem (DD-P7-002). More moving parts in CI.

Option C — Hybrid: keep the rlib path **and** add sys+safe
Both A and B coexist. Phase 4/5 visual-check examples continue to
use the rlib for development convenience; Phase 8 Hello Counter
goes through sys+safe.

- What you gain: No regression of existing examples. Validates the
  C ABI without disturbing the dev-loop ergonomics that the rlib
  path gives internal contributors.
- What you give up: Two parallel Rust APIs to the same runtime,
  with overlapping type names and divergent stability stories.
  The rlib's "is this experimental or not?" question doesn't go
  away, it gets harder.

**Recommendation:** **Option B.** M1's acceptance criterion only
holds water if Rust crosses the C ABI like the other two languages.
Option A produces a hollow check; Option C carries Option B's costs
without retiring Option A's confusion. The price of B is bounded
(two crates, one CI step, one naming decision) and the wins
are durable: every sys/safe binding pair we ship later (Swift, Go,
.NET) reuses the contract this Rust pair will pin down.

**Implication for Phase 4/5 visual-check examples:** they continue
to compile against the rlib. They are dev-internal and not part of
the M1 acceptance surface. DD-P7-002 documents this explicitly.

---

### DD-P7-002 — `wasamo` rlib status and crate naming

**Status:** Agreed

**Context:**
If DD-P7-001 = B, two derived questions arise:

- The cdylib is named `wasamo` and emits `wasamo.dll`. The safe
  wrapper would naturally also want to be called `wasamo` on
  crates.io (it is the user-facing crate). Two crates cannot share
  a name in one workspace, and even if we publish one and not the
  other, the workspace today already has a `wasamo` rlib path that
  Phase 4/5 examples consume.
- The rlib's Rust-native API has no experimental marker. If we are
  about to publish a safe wrapper as the *real* Rust face of wasamo,
  the rlib's standing relative to it must be stated.

**Options:**

Option A — Rename the runtime crate to `wasamo-runtime`; safe wrapper takes `wasamo` (recommended)
- `wasamo` runtime crate → `wasamo-runtime` (cdylib + rlib;
  cdylib still emits `wasamo.dll` via `[lib].name = "wasamo"`).
- New `wasamo-sys` crate (raw FFI).
- New `wasamo` crate (safe wrapper) at `bindings/rust/`.
- Phase 4/5 examples that use the rlib update their dependency
  from `wasamo` to `wasamo-runtime` and are explicitly documented
  as "internal dev examples; not part of the public Rust surface."
- The rlib's Rust-native API is treated as **internal/experimental**
  (`#![doc(hidden)]` on the public re-exports it currently has, or
  a `WASAMO_INTERNAL` cargo feature gate). It is **not** retired —
  removing it would gut Phase 4/5 dev-loop infrastructure — but it
  is documented in `architecture.md` as not the supported Rust API.

- What you gain: Public Rust API ships under the obvious name
  (`wasamo`). The two Rust paths (rlib for dev, sys+safe for hosts)
  are clearly distinguished by crate name. M1's "experimental"
  qualifier applies to both the C experimental layer and the rlib
  path uniformly.
- What you give up: One crate rename, touching `Cargo.toml`,
  Phase 4/5 examples' deps, and `architecture.md` §1.

Option B — Safe wrapper takes a different name (e.g. `wasamo-rs`); cdylib keeps `wasamo`
No runtime rename. Safe wrapper crate is `wasamo-rs` or similar.

- What you gain: No rename of the runtime crate.
- What you give up: The user-facing Rust crate ships under a
  non-obvious name. Reads as "the Rust binding of the Rust
  framework," which is awkward when the framework is *primarily*
  the Rust crate. A future `wasamo-py` / `wasamo-go` naming pattern
  for sister bindings would make sense, but `wasamo-rs` for the
  Rust binding to a Rust framework reads as a workaround.

Option C — Retire the rlib entirely; runtime crate becomes cdylib-only
The Phase 4/5 examples are rewritten to use `wasamo-sys`+safe.

- What you gain: Single Rust API. No name collision (the runtime
  crate has no public Rust surface, just the DLL).
- What you give up: Rewriting the visual-check examples is
  pure churn — they exist to verify Win32/WinRT integration, not
  to demo the public API. They predate Phase 6's C ABI. Forcing
  them through the C ABI for no acceptance-criterion reason is
  exactly the kind of "implement the task list literally" the
  pre-doc discipline warns against.

**Recommendation:** **Option A.** Rename runtime crate to
`wasamo-runtime`; let `wasamo` be the safe wrapper's name. Reposition
the rlib's Rust-native API as internal/experimental in
`architecture.md`. Phase 4/5 examples remain on the rlib path.

The `[lib].name = "wasamo"` setting in `wasamo-runtime/Cargo.toml`
preserves `wasamo.dll` / `wasamo.dll.lib` filenames, so the C ABI
artifact is unaffected.

---

### DD-P7-003 — Experimental layer surfacing in bindings

**Status:** Agreed

**Context:**
[abi_spec.md §5](../abi_spec.md) marks roughly half the C ABI surface
`WASAMO_EXPERIMENTAL` (the all-at-once widget constructors and
`wasamo_button_set_clicked`). Bindings must propagate this marker
in language-idiomatic ways, otherwise hosts learn the experimental
boundary only by reading the C header.

**Options:**

Option A — Module split: `wasamo::experimental` (Rust); equivalent in Zig (recommended)
- Rust: stable-core C ABI → safe wrapper at the crate root
  (`wasamo::Window`, `wasamo::Widget`, `wasamo::Value`, etc.).
  Experimental constructors → `wasamo::experimental` submodule
  (`wasamo::experimental::button`, `::vstack`, etc.) with a
  module-level docstring stating the M1 stability story.
- Zig: same shape — `wasamo.zig` exposes stable-core types at the
  top level; experimental constructors live in `wasamo.experimental`
  namespace.
- C: header inherits `WASAMO_EXPERIMENTAL` markers as-is. No
  separate header; the marker is the boundary signal.

- What you gain: `use wasamo::experimental::*` is a visible signal
  in source — code review and grep both see it. Symmetric across
  Rust and Zig. Costs nothing if a future `wasamoc`-codegen path
  retires the experimental layer (the module empties out, the
  stable-core API is unaffected).
- What you give up: Slightly more module-organization work in the
  wrapper crate. Negligible.

Option B — Cargo feature flag (`features = ["experimental"]`)
Experimental constructors are gated behind a non-default cargo
feature.

- What you gain: Hosts that don't enable the feature cannot
  accidentally call experimental functions.
- What you give up: For M1, **every** host needs the experimental
  layer (M1 `wasamoc` is parser-only, the stable core has no
  tree-construction primitive). Defaulting it off means defaulting
  every M1 host to a broken state. Defaulting it on makes the
  feature flag decorative. Cargo features add CI matrix complexity
  for no M1 protection.

Option C — Same crate, no separation; rely on docstrings
Experimental and stable wrappers sit side by side at the crate
root, distinguished only by `#[doc = "EXPERIMENTAL — ..."]`.

- What you gain: Smallest implementation cost.
- What you give up: Source-level visibility of the boundary is
  weak. `vstack(...)` and `Window::run(...)` look identical at the
  call site. The C side took pains to mark its experimental
  surface; bindings should match.

**Recommendation:** **Option A.** Module split is the cheapest way
to make the experimental boundary structurally visible in source,
and it survives both DSL-codegen-to-Rust and DSL-IR-to-runtime as
the M2 path (the experimental module either empties out or is
retained as a hand-builder escape hatch).

---

### DD-P7-004 — Phase 7 scope: Hello-Counter-minimal

**Status:** Agreed

**Context:**
The Phase 6 stable core has 13 functions; the experimental layer
adds 6 more. A "complete" binding wraps all of them. A
"Hello-Counter-sufficient" binding wraps only what `examples/counter`
will actually call. The acceptance criterion explicitly references
Hello Counter, not full ABI coverage.

**Options:**

Option A — Bind only what Hello Counter needs (recommended)
The Rust safe wrapper and Zig wrapper expose: lifecycle (init/run/
shutdown/quit), window create/show/destroy, the four experimental
constructors (`text`/`button`/`vstack`/`hstack`), `window_set_root`,
`button_set_clicked`, and `set_property`/`get_property` for at
least Button label and Text content. Other ABI entries (observers,
generic signal connect/disconnect, value packing for non-Counter
types) are added only if Phase 8 demonstrates they are needed.

- What you gain: Smallest binding surface. Phase 7 stays scoped.
  Anything not used by Phase 8 is by definition unverified, so
  binding it speculatively in Phase 7 is busywork that has to be
  re-checked anyway when a real consumer appears.
- What you give up: Hosts wanting more than Counter can do are
  blocked until a follow-up. Acceptable — M1 does not promise a
  complete binding, only enough binding for the milestone demo.

Option B — Full ABI coverage in M1
Wrap every `wasamo.h` entry in Rust and Zig.

- What you gain: Hosts have a complete surface from day one.
- What you give up: Phase 7 scope balloons. Most of the surface
  has no test consumer in M1 — observers and generic signal
  connect/disconnect are not used by Hello Counter, so wrapping
  them produces unverified code in the binding crate.

**Recommendation:** **Option A.** "Phase 7 produces what Phase 8
exercises" is the disciplined scope. Add to the bindings *during*
Phase 8 if a need surfaces; document the unbound entries in
`CONTRIBUTING.md` as "open for contribution." This also aligns
the binding's experimental surface area with what abi_spec §5.1
already commits to verifying in M1.

---

### DD-P7-005 — Zig binding strategy

**Status:** Agreed (with CI-driven fallback clause — see note below)

**Context:**
Zig has two natural strategies for consuming a C ABI:
`@cImport("wasamo.h")` (which translates the header at build time
via `zig translate-c`), or hand-written `extern` declarations.

**Options:**

Option A — `@cImport` over `wasamo.h` (recommended)
- `bindings/zig/wasamo.zig` does
  `const c = @cImport({ @cInclude("wasamo.h"); });` and re-exports
  Zig-flavored wrappers (slices for strings, tagged unions for
  `WasamoValue`, error sets for `WasamoStatus`).
- The build system points Zig at `bindings/c/` for the header and
  links `wasamo.dll.lib`.

- What you gain: No duplication. Header changes propagate
  automatically. Zig's `translate-c` is well-suited to a small,
  clean header like `wasamo.h`.
- What you give up: `@cImport` builds depend on Zig's bundled
  Clang. CI needs Zig + Windows SDK. The translated names live
  in a `c` namespace; the wrapper has to re-export with idiomatic
  shapes (acceptable, this is the wrapper's job).

Option B — Hand-written `extern` block
Mirror `wasamo.h` as Zig `extern fn` and `extern struct`
declarations.

- What you gain: No `@cImport` toolchain dependency. Drift can be
  CI-checked the same way Phase 6 already checks the Rust side.
- What you give up: A second source of truth for the same ABI.
  Phase 6 made `wasamo.h` the canonical artifact precisely to
  avoid two-source-of-truth setups (DD-P6-006 rejected `cbindgen`
  for the same reason). Hand-writing the Zig `extern` block here
  re-introduces the same problem on the Zig side.

**Recommendation:** **Option A.** `@cImport` is the Zig idiom, and
`wasamo.h` is exactly the kind of small, idiomatic-C header it
handles cleanly. CI grows by one Zig install. Drift is impossible
by construction.

**Agreement note (2026-04-30):** Adopted Option A on the
understanding that GitHub-hosted CI is the first place Zig
`@cImport` against `wasamo.h` is exercised end-to-end (the local
SSH dev box does not currently have a Zig toolchain installed; it
can be added later if needed for local iteration). If CI surfaces
a `translate-c` failure or a Windows-SDK header-resolution issue
that cannot be cleanly resolved, fall back to Option B
(hand-written `extern` block) is acceptable. The choice will be
re-evaluated on concrete CI evidence rather than speculatively.

---

### DD-P7-006 — C bindings layout and CMake sample shape

**Status:** Agreed (CMake build verifiable locally — see note below)

**Context:**
The C side already has `bindings/c/wasamo.h` and a smoke-test TU
(`bindings/c/smoke.c`) that CI builds with MSVC and clang-cl
(Phase 6). Phase 7 adds a "CMake sample" — i.e., a buildable
C consumer that a binding-author would copy as a starting point.
The shape of this sample affects how `bindings/c/` is organized.

**Options:**

Option A — `bindings/c/` holds header + import-lib copy + CMake template; sample lives in Phase 8 (recommended)
- `bindings/c/wasamo.h` (already present)
- `bindings/c/wasamo.dll.lib` — produced by the runtime build,
  copied into `bindings/c/` by the build script (or referenced
  by relative path from the workspace target dir)
- `bindings/c/CMakeLists.txt` — a template `add_library` /
  `target_include_directories` / `target_link_libraries` block,
  documented as "copy this into your project."
- The actual `examples/counter-c/` sample (Phase 8 work) consumes
  this template via `add_subdirectory` or `find_package`.

- What you gain: The reusable surface (header + import lib +
  CMake snippet) lives at a single, advertised location.
  Phase 7 produces the contract; Phase 8 produces the demo that
  exercises it.
- What you give up: A build engineer wanting to verify "does the
  CMake template actually work?" must wait for Phase 8. Mitigated
  by extending the existing CI smoke test to drive the CMake
  template (already builds and links a TU; this just changes the
  driver from "raw cl.exe" to "cmake --build").

Option B — Sample C app lives in `bindings/c/sample/` (Phase 7); Phase 8 just adds Counter
A standalone "minimal CMake consumer" sample inside `bindings/c/`,
distinct from `examples/counter-c/`.

- What you gain: Phase 7 has its own buildable artifact, not just
  a template.
- What you give up: Two C samples (`bindings/c/sample/` and
  `examples/counter-c/`) doing similar things. The `bindings/c/`
  one risks bit-rotting once `examples/counter-c/` becomes the
  real demo. ROADMAP Phase 8 already lists `examples/counter-c/`
  with a README; that is the real Phase 8 sample.

**Recommendation:** **Option A.** Phase 7 ships the *contract*
(header, import lib, CMake template, CI proof that all three link).
Phase 8 ships the *demo* (`examples/counter-c/`) consuming that
contract. Extend the existing smoke-test CI step to also drive a
CMake build of the same TU, so the template is CI-verified before
Phase 8 starts.

**Agreement note (2026-04-30):** The local SSH dev box has CMake
available at
`C:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe`
(bundled with the VS 2026 Community install). With the appropriate
PATH / `VCINSTALLDIR` environment set up (e.g. via
`vcvars64.bat`), the CMake template should be buildable locally
before pushing to CI. Confirming local buildability is part of the
implementation step for this item.

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

If the above decisions are agreed, the Phase 7 task list in
[ROADMAP.md](../../ROADMAP.md#L172-L180) is revised to reflect the
crate rename and the scope split:

- [ ] `docs/decisions/phase-7-language-bindings.md` — owner agreement (this doc)
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

Per session-end agreement (recorded in project memory), each item
above lands as a separate commit.
