# counter-rust — Hello Counter (Rust)

A minimal counter application written in Rust using the `wasamo` safe
wrapper crate. This is the **M1 host-imperative** shape: the widget tree
is constructed by hand through the `wasamo::experimental` module because
`wasamoc` is parser-only in M1. The M2 form — `wasamoc` lowering
`counter.ui` to Rust code — is future work.

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

Prerequisites: a release build of `wasamo.dll` / `wasamo.dll.lib` from
the repo root, and the Visual Studio 2022 Build Tools (MSVC linker).

```bat
rem From the repo root:
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
