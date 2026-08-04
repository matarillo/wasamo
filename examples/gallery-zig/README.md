# gallery-zig - Photo Gallery (Zig)

The Photo Gallery target application written in Zig. The widget tree, state,
binding, and click handlers all live in
[`examples/gallery/gallery.ui`](../gallery/gallery.ui). `build.zig` invokes
`wasamoc` to compile that file to Wasamo IR and exposes the result to
`main.zig` as the anonymous import `gallery_uic`. `@embedFile` inlines the
IR bytes at compile time, and `main.zig` hands the blob to `wasamo_load_ui`
via `WASAMO_LOAD_MEMORY`.

The host performs no imperative widget construction and no
`wasamo_set_property` calls.

## What It Does

- Opens the integrated Photo Gallery declared by the shared DSL file.
- Shows the `ToggleButton` tab band, thumbnail placeholders, scroll
  controls, status strip, and lightbox placeholder surface.
- Uses the same embedded `.uic` loading path as the counter Zig example.

## Build

Prerequisites: Zig 0.16.0, a release build of `wasamo.dll`,
`wasamo.dll.lib`, and `wasamoc.exe` from the repo root, and the Visual
Studio 2022 Build Tools.

```bat
rem From the repo root:
cargo build --release --workspace

cd examples\gallery-zig
zig build ^
    -Dwasamo-lib=../../target/release/wasamo.dll.lib ^
    -Dwasamo-zig=../../bindings/zig/wasamo.zig ^
    -Dwasamoc=../../target/release/wasamoc.exe ^
    -Doptimize=ReleaseSafe
```

The resulting `zig-out/bin/gallery-zig.exe` requires `wasamo.dll` on the
`PATH` or in the same directory to run.

## See Also

- [gallery-c](../gallery-c/README.md) - same example in C
- [gallery-rust](../gallery-rust/README.md) - same example in Rust
- [docs/abi_spec.md](../../docs/abi_spec.md) - C ABI specification
