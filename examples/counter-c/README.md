# counter-c — Hello Counter (C)

A minimal counter application written in C against the Wasamo C ABI.
This is the **M1 host-imperative** shape: the widget tree is constructed
by hand through the `WASAMO_EXPERIMENTAL` constructor layer because
`wasamoc` is parser-only in M1. The M2 form — `wasamoc` lowering
`counter.ui` to host-language code — is future work.

The equivalent declarative source lives at
[`examples/counter/counter.ui`](../counter/counter.ui); run
`wasamoc check examples/counter/counter.ui` to verify it still parses.

## What it does

- Opens an 800 × 600 Mica window titled "Counter".
- Displays a title-size text label reading "Count: 0".
- Shows an accent-style "Increment" button below the label.
- Clicking Increment updates the label to "Count: N" and re-lays out
  the widget tree so the new text width is reflected immediately.

## Build

Prerequisites: Visual Studio 2022 Build Tools (C compiler + linker),
CMake ≥ 3.21, and a release build of `wasamo.dll` / `wasamo.dll.lib`
from the repo root.

```bat
rem From the repo root:
cargo build --release --workspace

cmake -S examples/counter-c -B build/counter-c
cmake --build build/counter-c --config Release
```

The resulting `build/counter-c/Release/counter.exe` requires
`wasamo.dll` on the `PATH` or in the same directory to run.

## See also

- [counter-rust](../counter-rust/README.md) — same example in Rust
- [counter-zig](../counter-zig/README.md) — same example in Zig
- [docs/abi_spec.md](../../docs/abi_spec.md) — C ABI specification
