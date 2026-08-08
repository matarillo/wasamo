# gallery-c - Photo Gallery (C)

The Photo Gallery target application written in C against the Wasamo C ABI.
The widget tree, state, binding, and click handlers all live in
[`examples/gallery/gallery.ui`](../gallery/gallery.ui). CMake invokes
`wasamoc` to compile that file to Wasamo IR at build time, generates
`gallery_uic.h` embedding the IR bytes, and `main.c` hands the blob to
`wasamo_load_ui` via `WASAMO_LOAD_MEMORY`.

The host performs no imperative widget construction and no
`wasamo_set_property` calls.

## What It Does

- Opens the integrated Photo Gallery declared by the shared DSL file.
- Shows the tab band, thumbnail grid, and status strip declared in
  `gallery.ui`, whose lightbox opens on a thumbnail click, confines the
  keyboard, and responds to Esc and Left/Right.
- Uses the same embedded `.uic` loading path as the counter C example.

## Build

Prerequisites: Visual Studio 2022 Build Tools, CMake 3.21 or newer, and a
release build of `wasamo.dll`, `wasamo.dll.lib`, and `wasamoc.exe` from the
repo root.

```bat
rem From the repo root:
cargo build --release --workspace

cmake -S examples/gallery-c -B build/gallery-c
cmake --build build/gallery-c --config Release
```

The resulting `build/gallery-c/Release/gallery-c.exe` requires `wasamo.dll`
on the `PATH` or in the same directory to run.

## See Also

- [gallery-rust](../gallery-rust/README.md) - same example in Rust
- [gallery-zig](../gallery-zig/README.md) - same example in Zig
- [docs/abi_spec.md](../../docs/abi_spec.md) - C ABI specification
