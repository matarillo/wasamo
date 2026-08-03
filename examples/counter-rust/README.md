# counter-rust — Hello Counter (Rust)

A minimal counter application written in Rust. The widget tree, state,
binding, and click handler all live in
[`examples/counter/counter.ui`](../counter/counter.ui); `build.rs`
compiles that file to Wasamo IR via `wasamoc` at build time, and
`main.rs` hands the IR to `wasamo_load_ui` through the raw `wasamo-sys`
binding. No imperative widget construction, no `wasamo_set_property`
calls — A1/A2 (DD-M2-P6-008) are structurally satisfied.

## What it does

- Opens an 800 × 600 window titled "Counter" from the DSL-side
  `title: "Counter"` declaration.
- Displays a title-size text label reading "Count: 0".
- Shows an accent-style "Increment" button below the label.
- Clicking Increment updates the label to "Count: N" via the reactive
  binding declared in `counter.ui`.

## Build

Prerequisites: a release build of `wasamo.dll` / `wasamo.dll.lib` from
the repo root, and the Visual Studio 2022 Build Tools (MSVC linker).
`wasamoc` is built automatically as a workspace-internal build
dependency of this crate.

```bat
rem From the repo root:
cargo build --release -p wasamo-runtime
cargo build --release --workspace
cargo build --release -p counter-rust
```

The resulting executable is at
`target/release/counter-rust.exe` and requires `wasamo.dll` on the
`PATH` or in the same directory to run.

## See also

- [counter-c](../counter-c/README.md) — same example in C
- [counter-zig](../counter-zig/README.md) — same example in Zig
- [docs/abi_spec.md](../../docs/abi_spec.md) — C ABI specification
