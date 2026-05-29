### DD-M2-P1-001 — Existence and shape of the cdylib shim

**Status:** Accepted

**Context:**
The rlib filename collision was resolved in M1 by removing the rlib
crate-type from `wasamo-runtime` entirely (DD-P7-002 post-M1 note).
That works but is a *workaround*: it discards the in-tree mechanism
for letting Rust dev tools link against the runtime's internal API.
The Phase 2-5 visual-check examples were collateral. A3 asks for the
*proper* fix, not just continued absence of the symptom.

**Options:**

Option A — Two-crate split: `wasamo-runtime` (rlib-only) + `wasamo-dll` (cdylib-only shim) (recommended)
- `wasamo-runtime` becomes rlib-only and houses all runtime logic
  (including the `#[no_mangle] pub extern "C"` ABI symbol definitions
  it already contains).
- A new minimal crate `wasamo-dll` is cdylib-only, depends on
  `wasamo-runtime`, and forces the C ABI symbols through to the
  cdylib output (mechanism = DD-M2-P1-005).
- The shim crate's `[lib].name = "wasamo"` preserves the
  `wasamo.dll` / `wasamo.dll.lib` filenames the public ABI artifact
  is named under.

  - What you gain: Structurally separates "DLL build product" from
    "Rust library." The rlib filename now derives from
    `wasamo-runtime` (DD-M2-P1-002), which is distinct from the safe
    wrapper's `wasamo` rlib. Collision class is *eliminated by
    construction*, not just unmanifested. Future Rust-side dev tools
    (a resurrected `phase4_visual_check`, a benchmark harness, a
    fuzz target) can depend on `wasamo-runtime` directly without
    touching the shim or the C ABI.
  - What you give up: One additional workspace member with a
    near-trivial source file. A small amount of build-script glue
    (DD-M2-P1-005). Marginal.

Option B — Rename the safe wrapper's crate name instead (e.g. `wasamo-rs`)
- Keep `wasamo-runtime` cdylib-only as it is now.
- Restore `rlib` to its `crate-type` and accept that *both* this and
  the safe wrapper produce an rlib — but the safe wrapper's package
  name changes so its rlib is `libwasamo_rs.rlib`, no collision.

  - What you gain: No new crate. Smallest workspace delta.
  - What you give up: The user-facing Rust crate ships under a
    non-obvious name. DD-P7-002 already evaluated this option (its
    "Option B") and rejected it: "`wasamo-rs` for the Rust binding
    to a Rust framework reads as a workaround." Re-considering it
    here would re-open a DD-P7-002 decision, which is a higher bar
    than this phase warrants. Also keeps two rlibs in flight (one
    from `wasamo-runtime`, one from the safe wrapper), which is the
    structural smell A3 is meant to remove.

Option C — Status quo: keep `wasamo-runtime` cdylib-only, no shim
- Do nothing structural. A3 is "discharged" by pointing at the M1
  resolution: rlib was removed, collision is gone.

  - What you gain: Zero work. A3 read literally ("no longer share an
    rlib filename") is already true.
  - What you give up: A3 read in spirit (per its DD-P7-002 origin
    note: "the proper long-term fix is a cdylib-shim crate") is
    *not* discharged. The mechanism that prevents collision is
    "we deleted one of the colliding rlibs," not a structural
    separation; reintroducing any Rust dev tool that wants the
    runtime's internal API would re-create the collision. The
    plan's framing of A3 as "the cdylib-shim split" makes this
    option a misread of the criterion.

**Decision:** Option A — Accepted (2026-05-03).

---
