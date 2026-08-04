# counter-zig — Hello Counter (Zig)

A minimal counter application written in Zig. The widget tree, state,
binding, and click handler all live in
[`examples/counter/counter.ui`](../counter/counter.ui); `build.zig`
invokes `wasamoc` to compile that file to Wasamo IR and exposes the
result to `main.zig` as the anonymous import `counter_uic`. `@embedFile`
inlines the IR bytes at compile time, and `main.zig` hands the
`(pointer, length)` blob to `wasamo_load_ui` via `WASAMO_LOAD_MEMORY`
through the raw `wasamo.c` extern surface. No imperative widget
construction, no `wasamo_set_property` calls — A1/A2 (DD-M2-P6-008) are
structurally satisfied.

## What it does

- Opens an 800 × 600 window titled "Counter" from the DSL-side
  `title: "Counter"` declaration.
- Displays a title-size text label reading "Count: 0".
- Shows an accent-style "Increment" button below the label.
- Clicking Increment updates the label to "Count: N" via the reactive
  binding declared in `counter.ui`.

## Build

Prerequisites: Zig 0.16.0 (install via `winget install -e --id zig.zig`),
a release build of `wasamo.dll` / `wasamo.dll.lib` and `wasamoc.exe`
from the repo root, and the Visual Studio 2022 Build Tools.

```bat
rem From the repo root:
cargo build --release --workspace

cd examples\counter-zig
zig build ^
    -Dwasamo-lib=../../target/release/wasamo.dll.lib ^
    -Dwasamo-zig=../../bindings/zig/wasamo.zig ^
    -Dwasamoc=../../target/release/wasamoc.exe ^
    -Doptimize=ReleaseSafe
```

The build invokes `wasamoc.exe` to compile `counter.ui`, then embeds
the IR via `@embedFile`. `wasamoc.exe` must exist at the configured
path — see CLAUDE.md "Build ordering requirements".

The resulting `zig-out/bin/counter-zig.exe` requires `wasamo.dll` on
the `PATH` or in the same directory to run.

## See also

- [counter-c](../counter-c/README.md) — same example in C
- [counter-rust](../counter-rust/README.md) — same example in Rust
- [docs/abi_spec.md](../../docs/abi_spec.md) — C ABI specification
