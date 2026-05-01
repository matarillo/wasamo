# counter-zig — Hello Counter (Zig)

A minimal counter application written in Zig using the `wasamo` Zig
binding. This is the **M1 host-imperative** shape: the widget tree is
constructed by hand through the `wasamo.experimental` namespace because
`wasamoc` is parser-only in M1. The M2 form — `wasamoc` lowering
`counter.ui` to Zig code — is future work.

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

Prerequisites: Zig 0.16.0 (install via `winget install -e --id zig.zig`),
a release build of `wasamo.dll` / `wasamo.dll.lib` from the repo root,
and the Visual Studio 2022 Build Tools.

```bat
rem From the repo root:
cargo build --release --workspace

zig build -p . ^
    --wasamo-lib target/release/wasamo.dll.lib ^
    --wasamo-zig bindings/zig/wasamo.zig ^
    -Doptimize=ReleaseSafe
```

The resulting `bin/counter-zig.exe` requires `wasamo.dll` on the
`PATH` or in the same directory to run.

## See also

- [counter-c](../counter-c/README.md) — same example in C
- [counter-rust](../counter-rust/README.md) — same example in Rust
- [docs/abi_spec.md](../../docs/abi_spec.md) — C ABI specification
