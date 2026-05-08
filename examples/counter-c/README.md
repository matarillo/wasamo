# counter-c — Hello Counter (C)

A minimal counter application written in C against the Wasamo C ABI.
The widget tree, state, binding, and click handler all live in
[`examples/counter/counter.ui`](../counter/counter.ui); CMake invokes
`wasamoc` to compile that file to Wasamo IR at build time, then generates
`counter_uic.h` embedding the IR bytes as a `static const unsigned
char[]`. `main.c` hands the blob to `wasamo_load_ui` via
`WASAMO_LOAD_MEMORY`. No imperative widget construction, no
`wasamo_set_property` calls — A1/A2 (DD-M2-P6-008) are structurally
satisfied.

## What it does

- Opens an 800 × 600 window (default title "Wasamo" — DSL-side
  `title: "Counter"` is currently dropped by the runtime; tracked in
  [docs/notes/dsl-grammar.md Q2](../../docs/notes/dsl-grammar.md)).
- Displays a title-size text label reading "Count: 0".
- Shows an accent-style "Increment" button below the label.
- Clicking Increment updates the label to "Count: N" via the reactive
  binding declared in `counter.ui`.

## Build

Prerequisites: Visual Studio 2022 Build Tools (C compiler + linker),
CMake ≥ 3.21, and a release build of `wasamo.dll` / `wasamo.dll.lib` and
`wasamoc.exe` from the repo root.

```bat
rem From the repo root:
cargo build --release --workspace

cmake -S examples/counter-c -B build/counter-c
cmake --build build/counter-c --config Release
```

The CMake configure step verifies that `wasamoc.exe` is present at
`target/release/wasamoc.exe`; see CLAUDE.md "Build ordering
requirements" if you hit a missing-binary error.

The resulting `build/counter-c/Release/counter.exe` requires
`wasamo.dll` on the `PATH` or in the same directory to run.

## See also

- [counter-rust](../counter-rust/README.md) — same example in Rust
- [counter-zig](../counter-zig/README.md) — same example in Zig
- [docs/abi_spec.md](../../docs/abi_spec.md) — C ABI specification
