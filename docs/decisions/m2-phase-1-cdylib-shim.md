# M2-Phase 1 — cdylib-shim cleanup: Architecture Decisions

**Phase:** M2-Phase 1 (cdylib-shim cleanup)
**Date:** 2026-05-03
**Status:** Agreed (2026-05-03)

## Context

M2 acceptance criterion **A3** (see
[ROADMAP.md M2](../../ROADMAP.md#m2-foundation),
[m2-plan.md](../plans/m2-plan.md#frozen-agreement)):

> `wasamo-runtime` and the `wasamo` safe wrapper no longer share an
> rlib filename through the cdylib-shim split; the post-M1 cleanup
> flagged in [DD-P7-002](./phase-7-language-bindings.md) is discharged.

The post-M1 implementation note in DD-P7-002 records the symptom and
the planned shape of the long-term fix. [`architecture.md §11.4`](../architecture.md)
sketches the same shape:

> A cdylib-shim crate (`wasamo-dll`) that depends on `wasamo-runtime`
> (rlib-only, renamed to `wasamo_runtime`) will restore the separation
> cleanly — `wasamo-dll` emits `wasamo.dll` without an rlib, so no
> collision with the safe wrapper's rlib. The Phase 2-5 dev examples
> can be re-introduced under a `wasamo-poc` workspace once that
> refactor is complete.

That sketch is at the level of *what to do*, not *how to do it*. This
ADR resolves the design questions that come up when actually doing it,
in the order they constrain each other. DD-M2-P1-001 (does the shim
exist at all?) gates the rest; DD-M2-P1-002 (naming) follows; the
remaining three are subordinate but each is a real fork.

The acceptance lens is narrow: A3 is a *structural* criterion. Pre-doc
discipline says the phase is done when the rlib-collision class is
gone, not when every conceivable cleanup has happened. Decisions below
are framed against that lens; speculative scope (e.g. resurrecting
Phase 2-5 examples) is treated as out-of-scope unless A3 demands it.

---

### DD-M2-P1-001 — Existence and shape of the cdylib shim

**Status:** Agreed

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

**Decision:** Option A — Agreed (2026-05-03).

---

### DD-M2-P1-002 — Naming of the rlib crate and the shim crate

**Status:** Agreed

**Context:**
Once Option A is taken, two crates need names. The cdylib's
*filename* must remain `wasamo.dll` (DD-P6-007 ABI artifact name);
that is fixed by `[lib].name = "wasamo"` in whichever crate emits the
cdylib. Crate package names and rlib filenames are the free variables.

The constraint is that no two rlibs in the workspace's dependency
graph share an output filename. The safe wrapper's package is
`wasamo` and produces `libwasamo.rlib` — that is the user-facing name
fixed by DD-P7-002 and is not on the table.

**Options:**

Option A — `wasamo-runtime` keeps its name (rlib-only, `[lib].name = "wasamo_runtime"`); shim is `wasamo-dll` (recommended)
- `wasamo-runtime` package: rlib-only, `[lib].name = "wasamo_runtime"`,
  output `libwasamo_runtime.rlib`. Distinct from
  the safe wrapper's `libwasamo.rlib`.
- `wasamo-dll` package: cdylib-only, `[lib].name = "wasamo"`, output
  `wasamo.dll` / `wasamo.dll.lib`.

  - What you gain: `wasamo-runtime` retains the name DD-P7-002 chose
    for it; that name's call sites (architecture.md, DD-P7-002 post-M1
    note, m2-plan) stay valid without further rename churn. Rlib
    filename `libwasamo_runtime.rlib` is unambiguous and matches the
    package name (cargo idiom). `wasamo-dll` reads as "the DLL-build
    crate," which is what it is.
  - What you give up: One `[lib].name` change in `wasamo-runtime`'s
    Cargo.toml (currently `"wasamo"`, becomes `"wasamo_runtime"`).
    The change is contained: no public symbol moves; `wasamo.dll`
    filename now derives from the *shim's* `[lib].name`, not this
    one.
    Additionally, `wasamo-dll`'s `[lib].name = "wasamo"` is a
    deliberate deviation from the cargo convention that lib name
    matches package name (transformed: `wasamo-dll` → `wasamo_dll`).
    The deviation is justified by DD-P6-007: `wasamo.dll` is the
    public C ABI artifact name. See the note on convention deviation
    below.

Option B — Rename `wasamo-runtime` to e.g. `wasamo-core`, call the shim `wasamo-runtime`
- The shim crate is `wasamo-runtime` (cdylib-only, `[lib].name = "wasamo"`).
- The rlib crate becomes `wasamo-core`.

  - What you gain: The crate that emits `wasamo.dll` is named
    `wasamo-runtime`, which arguably matches the public mental model
    ("the runtime is a DLL").
  - What you give up: Renames a crate that was just renamed in
    Phase 7. Every architecture.md, DD-P7-002, m2-plan, ROADMAP, and
    git-history reference to `wasamo-runtime` now means a different
    thing, or has to be re-disambiguated. The naming gain is
    aesthetic; the churn is real.

Option C — Distinct rlib name only (no rename of either crate); only `[lib].name` changes
- `wasamo-runtime` package: rlib-only, `[lib].name = "wasamo_runtime"`.
  *No new crate.* Reintroduce `rlib` to `crate-type` and *also* keep
  cdylib in the same crate, with `[lib].name = "wasamo"` for the
  cdylib only.

  - What you gain: No new workspace member.
  - What you give up: cargo does not let a single `[lib]` table
    declare two `[lib].name` values. The cdylib and rlib outputs of
    one crate share `[lib].name` and therefore the filename stem.
    This option is structurally not expressible. Listed for
    completeness; rejected on feasibility.

**Decision:** Option A — Agreed (2026-05-03).

**Note on naming convention deviation:**
Setting `[lib].name = "wasamo"` on a package named `wasamo-dll`
deviates from the cargo convention that a package's lib name mirrors
its package name (hyphens to underscores: `wasamo-dll` → `wasamo_dll`).
This deviation is deliberate and bounded:

1. `[lib].name` is the cargo-documented mechanism for controlling
   the output filename of a cdylib. Using it here is idiomatic for
   cdylib crates that must expose a product-branded artifact.
2. The justification for the specific name `wasamo` is DD-P6-007:
   the public C ABI artifact is named `wasamo.dll`; changing it
   would break all downstream consumers' build scripts and import
   libraries.
3. The deviation is confined to the shim crate, which has no
   public Rust-library surface of its own. It is a build-product
   crate, not a library crate that other Rust packages would
   depend on by name.

The convention deviation and its rationale are documented at the
point of deviation: `wasamo-dll/Cargo.toml` carries a comment
referencing this ADR and DD-P6-007. The crate responsibilities
table in `architecture.md §1` will note the `[lib].name` override
explicitly.

---

### DD-M2-P1-003 — Phase 2-5 dev examples: resurrect now, or defer?

**Status:** Agreed

**Context:**
The M1 resolution to the rlib collision deleted
`phase2_window_check`, `phase3_layout_check`, `phase4_visual_check`,
`phase5_visual_check`. They depended on the runtime's internal Rust
API (`Runtime`, `WindowState`, `WidgetNode`, `TextRenderer`, …),
which was reachable only through the rlib. Once DD-M2-P1-001 Option A
lands, that reachability is restored.

A3's literal text says nothing about these examples. The architecture.md
§11.4 "long-term fix" paragraph says they "*can* be re-introduced
under a `wasamo-poc` workspace once that refactor is complete."

**Options:**

Option A — Resurrect under a new workspace dir (e.g. `wasamo-poc/`) in this phase
Option B — Defer; ship the structural split alone (recommended)
Option C — Drop them permanently

**Decision:** Option B on the main branch — Agreed (2026-05-03).

After the main-branch portion of M2-Phase 1 lands, an experimental
branch (`exp/m2-p1-poc-examples`) will be created to attempt
resurrecting the Phase 2-5 examples under `wasamo-poc/`. The branch
serves as a validation that the shim structure actually enables
in-workspace rlib consumers and as a reference for any future formal
resurrection. It is not merged to main unless a concrete acceptance
criterion demands it.

---

### DD-M2-P1-004 — Workspace location of the shim crate

**Status:** Agreed

**Context:**
The workspace currently has crates at top level (`wasamo-runtime/`,
`wasamoc/`) and grouped under `bindings/` and `examples/`. The shim
crate needs a home. A secondary question arose during review: should
the project adopt a `crates/` root directory to follow a pattern used
by some larger Rust workspaces?

**Options:**

Option A — Top-level `wasamo-dll/` (recommended)
- Sibling of `wasamo-runtime/`. Top-level placement matches the
  other "produces a build artifact at the project's name level"
  crates.

  - What you gain: Discoverability; consistent with `wasamo-runtime/`
    and `wasamoc/`.
  - What you give up: One more entry at the workspace root. The
    root listing is not yet so crowded that one more matters.

Option B — Nested under `wasamo-runtime/dll-shim/`
  - What you gain: Physical containment expresses the dependency.
  - What you give up: Unusual in cargo workspaces; inversion of the
    dependency direction (shim depends on runtime, not vice versa).

Option C — `crates/wasamo-dll/`
- Introduce a `crates/` directory and put the shim there.

  - What you gain: Conventional in some Rust monorepos; signals
    "internal crates" vs. `bindings/` and `examples/`.
  - What you give up: Inconsistent with the existing layout; either
    requires moving all other crates into `crates/` (broad churn
    out of M2-Phase 1 scope) or leaves `wasamo-dll` as the sole
    resident of a new directory (asymmetry). Deciding this now
    entangles a workspace-layout open question with an unrelated
    phase.

**Decision:** Option A — Agreed conditionally (2026-05-03). The
`crates/` pattern (Option C) would only make sense if all crates
migrated together. That is a separate workspace-layout decision,
not part of this phase. The open question — whether a future
`crates/` reorganisation is warranted — is recorded in
[`docs/notes/workspace-layout.md`](../notes/workspace-layout.md)
as a live note. If the project reaches the point where it decides to
adopt `crates/`, M2-Phase 1's placement of `wasamo-dll/` can be
addressed in the same migration commit.

---

### DD-M2-P1-005 — How the shim re-exports the C ABI symbols

**Status:** Agreed

**Context:**
The `#[no_mangle] pub extern "C"` symbols defined in
`wasamo-runtime` need to end up exported from the cdylib produced by
`wasamo-dll`. Three mechanisms exist.

**Options:**

Option A — Whole-archive link of the rlib via build.rs (recommended)
- `wasamo-dll/build.rs` emits a MSVC-specific whole-archive link
  argument to force all rlib symbols into the cdylib output.
- `wasamo-dll/src/lib.rs` is minimal (a crate-level `extern crate`
  or empty body as implementation determines).

  - What you gain: Zero per-symbol maintenance. New ABI symbols in
    `wasamo-runtime` automatically appear in `wasamo.dll`. Standard
    Rust cdylib-shim pattern.
  - What you give up: One MSVC-specific link arg in build.rs.
    Verified on the local SSH dev box before pushing to CI
    (`dumpbin /exports wasamo.dll` shows all current ABI symbols).

Option B — Per-symbol re-export from the shim's `lib.rs`
  - What you give up: Per-symbol maintenance burden proportional to
    ABI growth (M2-Phase 4 adds tree-mutation primitives — exactly
    the wrong scaling). Requires stripping `#[no_mangle]` from the
    rlib side, defeating its usefulness as a direct dev dependency.

Option C — `#[used]` annotations on each symbol in `wasamo-runtime`
  - What you give up: Non-idiomatic for functions in stable Rust;
    documented whole-archive is the standard cdylib-shim mechanism.

**Decision:** Option A — Agreed (2026-05-03). SSH dev box verification
(cargo build + `dumpbin /exports`) is required before pushing to CI.

---

### DD-M2-P1-006 — Build-order edge between cdylib shim and final binaries

**Status:** Agreed (2026-05-03)

**Context:**
After implementing DD-M2-P1-001..005 in a working tree,
`cargo clean && cargo build --release --workspace` reproducibly failed
with `LNK1181: cannot open input file 'wasamo.dll.lib'`. Diagnosis: the
cargo dependency graph had no edge between `wasamo-dll` (cdylib
producer of `wasamo.dll.lib`) and the final binaries (`counter-rust`
etc.) that consume it via `bindings/rust-sys`'s `#[link]`. Cargo
parallelised them, and the linker for `counter-rust` ran before the
cdylib finished. The `#[link]` attribute alone does not create a build
order edge — cargo only orders crates that appear in some `dependencies`
table.

**Options:**

Option A — Add `wasamo-dll` to `[dependencies]` of `bindings/rust-sys/Cargo.toml` (recommended)
- One edge covers every binary that links the C ABI (all Rust hosts
  go through `rust-sys`).
- Verified locally: `cargo clean && cargo build --release --workspace`
  succeeds; `dumpbin /exports target/release/wasamo.dll` shows all 19
  ABI symbols; `cargo run -p counter-rust --release` works
  end-to-end.
- What you give up: cargo emits `warning: the package wasamo
  provides no linkable target` (rust-lang/cargo#6313) for every
  build, because a cdylib has no Rust-linkable surface and `rust-sys`
  is a normal Rust crate. Accepted as a deferred / open issue —
  recorded in [`docs/notes/cdylib-shim-build-graph.md`](../notes/cdylib-shim-build-graph.md)
  with explicit re-evaluation triggers.

Option B — `[build-dependencies] wasamo-dll` or `artifact = "cdylib"`
  - What you give up: `[build-dependencies]` triggers host-target
    double build → filename collision on `wasamo.dll`. The
    `artifact`/`-Z bindeps` mechanism is unstable on stable Rust and
    has had similar collision behaviour in tested forms. Not
    actionable today.

Option C — Add `wasamo-dll` to `[dependencies]` of each Rust binary individually
  - What you give up: Fragile — every new Rust binary added to the
    workspace would silently regress to LNK1181 if the maintainer
    forgot the extra line. Centralising the edge in `rust-sys` (which
    every Rust host already depends on) is strictly safer.

**Decision:** Option A — Agreed (2026-05-03). The `no linkable target`
warning is accepted as a known wart, not a settled end-state; the
note records re-evaluation triggers (cargo making the warning a hard
error; a second cdylib-only build-order dependency appearing; a real
need to consume `wasamo-dll`'s Rust surface). If any trigger fires,
revisit this DD.

---

## Out of scope (for M2-Phase 1; recorded explicitly)

- **Resurrecting Phase 2-5 dev examples on main.** Mechanism enabled
  by this phase; experimental branch created after main lands; formal
  resurrection deferred (DD-M2-P1-003).
- **Renaming any public crate (`wasamo`, `wasamoc`, `wasamo-sys`).**
  DD-P7-002's naming is settled; this phase does not re-open it.
- **Changes to `wasamo.h`, ABI symbol names, or DLL filename.** All
  preserved by construction.
- **Adding new ABI symbols.** A4 (M2-Phase 4) territory.
- **Workspace-wide `crates/` reorganisation.** Recorded as an open
  question in [`docs/notes/workspace-layout.md`](../notes/workspace-layout.md).

## Summary of agreed decisions

| ID | Topic | Decision |
|---|---|---|
| DD-M2-P1-001 | Cdylib shim existence/shape | Option A — two-crate split (`wasamo-runtime` rlib-only + `wasamo-dll` cdylib shim) |
| DD-M2-P1-002 | Naming | Option A — keep `wasamo-runtime` name; shim = `wasamo-dll`; `[lib].name = "wasamo_runtime"` on rlib, `"wasamo"` on cdylib |
| DD-M2-P1-003 | Phase 2-5 examples | Option B (main) — defer; experimental branch `exp/m2-p1-poc-examples` after main lands |
| DD-M2-P1-004 | Shim location | Option A — top-level `wasamo-dll/`; `crates/` question deferred to `docs/notes/workspace-layout.md` |
| DD-M2-P1-005 | ABI symbol propagation | Option A — `+whole-archive` via build.rs; local SSH dev box verification required |
| DD-M2-P1-006 | Build-order edge for cdylib consumers | Option A — add `wasamo-dll` to `[dependencies]` of `bindings/rust-sys/Cargo.toml`; `no linkable target` warning accepted as deferred (see `docs/notes/cdylib-shim-build-graph.md`) |

## Agreed M2-Phase 1 task list

### Main branch

- [x] `docs/decisions/m2-phase-1-cdylib-shim.md` — owner agreement
      (this doc; status "Agreed")
- [x] `docs/notes/workspace-layout.md` — new live note: workspace
      layout open question (`crates/` migration) per DD-M2-P1-004
- [ ] `wasamo-runtime/Cargo.toml`: `[lib].name = "wasamo_runtime"`,
      `crate-type = ["rlib"]`. Comment update.
- [ ] New `wasamo-dll/` crate: `Cargo.toml`
      (`[lib] name = "wasamo" crate-type = ["cdylib"]`), `build.rs`
      with MSVC `/WHOLEARCHIVE:wasamo_runtime` link arg, `src/lib.rs`.
      Workspace `Cargo.toml` `members += ["wasamo-dll"]`.
- [ ] `bindings/rust-sys/build.rs` and any other consumer: verify
      cdylib build output path is unchanged; update if needed.
- [ ] `bindings/rust-sys/Cargo.toml`: add
      `wasamo-dll = { path = "../../wasamo-dll" }` to `[dependencies]`
      to create the build-order edge (DD-M2-P1-006). Accept the
      `no linkable target` warning per the linked note.
- [ ] `docs/notes/cdylib-shim-build-graph.md` — new live note
      recording the `no linkable target` deferral and re-evaluation
      triggers (DD-M2-P1-006).
- [ ] Local verification: `cargo clean && cargo build --release --workspace`;
      `dumpbin /exports target/release/wasamo.dll` shows all 19
      `wasamo_*` symbols; `cargo run -p counter-rust` works
      end-to-end.
- [ ] `docs/architecture.md`: update §1 workspace layout (add
      `wasamo-dll/`) and crate responsibilities table (`wasamo-runtime`
      = rlib, `wasamo-dll` = cdylib with `[lib].name` convention note);
      replace §11.4 to reflect structural resolution.
- [ ] `docs/plans/m2-plan.md` Progress: tick M2-Phase 1, link this
      ADR.
- [ ] CHANGELOG.md: add entry for the cdylib-shim split.

### Experimental branch (after main lands)

- [ ] Create branch `exp/m2-p1-poc-examples` from the M2-Phase 1 tip.
- [ ] Recover Phase 2-5 examples from git history; place under
      `wasamo-poc/`; add to workspace. Update their `wasamo` dep to
      `wasamo-runtime`.
- [ ] Verify they compile and run on the SSH dev box.
- [ ] Do not merge to main; branch serves as resurrection reference.

No CI update is needed (no new language or build system; existing
`cargo build --release --workspace` covers the new crate per
CLAUDE.md CI rule). No unit tests are added (build plumbing; no pure
logic surface; correctness verified by local cdylib build + dumpbin
check plus CI Windows runner).
