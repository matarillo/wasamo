# Workspace Layout — Open Questions

**Status:** Live note — open question
**Origin:** DD-M2-P1-004 (cdylib-shim placement decision, 2026-05-03)

## Background

M2-Phase 1 added `wasamo-dll/` as a top-level workspace member (sibling
of `wasamo-runtime/` and `wasamoc/`). That decision was taken with Option
A (top-level) on the understanding that a possible future workspace
reorganisation — grouping all crates under a `crates/` directory — was
not scoped to that phase. This note records the open question.

## Open question: adopt `crates/` root directory?

Some Rust monorepos group all crates that are not examples or bindings
under a `crates/` directory:

```
wasamo/
├── crates/
│   ├── wasamo-runtime/
│   ├── wasamo-dll/
│   └── wasamoc/
├── bindings/
│   ├── c/
│   ├── rust/
│   ├── rust-sys/
│   └── zig/
└── examples/
    └── counter*/
```

This is a recognised pattern in the Rust ecosystem (used by crates
such as `bevy`, `zed`, and others). It distinguishes "core library
crates" from `bindings/` and `examples/` by directory semantics rather
than by convention alone.

### Arguments for adopting `crates/`

- Cleaner workspace root: at M2 the top-level already has
  `wasamo-runtime/`, `wasamo-dll/`, `wasamoc/` as crates, plus
  `bindings/`, `examples/`, `docs/`, `Cargo.toml`, `ROADMAP.md`, etc.
  If M2 adds more crates (reactive engine, future DSL crates), the root
  grows further.
- `crates/` is a well-understood signal to contributors navigating the
  repo: "the implementation lives here."

### Arguments against (current standing)

- All existing crates are at the top level; moving them is mechanical
  but broad churn (Cargo.toml members list, path dependencies, any
  IDE/CI path references).
- No concrete pain today. Premature reorganisation is the kind of work
  pre-doc discipline discourages.
- The split between `crates/` (internal) and `bindings/` (public
  surface) is meaningful only if the language bindings are not
  themselves "implementation." For wasamo, the Rust safe wrapper
  (`bindings/rust/`) *is* the public API — its placement under
  `bindings/` is accurate. A `crates/` grouping would cover only the
  runtime-side crates (`wasamo-runtime`, `wasamo-dll`, `wasamoc`).

### Suggested trigger

Revisit when — and only when — the workspace root becomes genuinely
difficult to navigate, or when a new phase adds enough crates to push
the top-level past ~6 non-bindings/non-examples entries. At that point
the cost of the migration is justified by the organisational gain.

If a migration is decided, it should be its own commit (no other
changes) with an ADR entry in `docs/decisions/` recording the rationale
(not merely "we wanted crates/"; specifically which organisational
problem the migration solves).
