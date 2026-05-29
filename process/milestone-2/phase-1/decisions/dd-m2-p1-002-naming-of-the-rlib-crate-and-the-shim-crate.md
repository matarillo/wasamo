### DD-M2-P1-002 — Naming of the rlib crate and the shim crate

**Status:** Accepted

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

**Decision:** Option A — Accepted (2026-05-03).

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
