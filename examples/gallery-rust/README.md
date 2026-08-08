# gallery-rust — Photo Gallery (Rust)

A minimal Rust host for the gallery sub-screen. The UI lives in
[`examples/gallery/gallery.ui`](../gallery/gallery.ui); `build.rs`
compiles it to Wasamo IR via the workspace `wasamoc` crate, then
`main.rs` loads the generated IR through `wasamo_load_ui`.

The Phase 3 sub-screen is a `WrapPanel` of eighteen uniform 1:1 `Box`
thumbnails, generated through a `for` over the `labels` state, with
`item-cross-size: 88; item-spacing: 12;
line-spacing: 12` — the same WrapPanel attribute set the ADR
verification closure pins for the canonical sub-screen positive
control and the CI integration fixture. On the default 800×600
window's ~784-wide client area the line breaker fits seven
thumbnails per line, wrapping the eighteen thumbnails across three
lines — the visible smoke for the M3-Phase 3 WrapPanel
primitive. Later M3 phases grow `examples/gallery/` further
sub-screen by sub-screen. The gallery is now also the M4-Phase 2
interaction consumer: clicking a thumbnail opens a lightbox that
confines the keyboard and steps or closes with Left/Right/Esc.

```powershell
cargo build --release -p gallery-rust
Start-Process .\target\release\gallery-rust.exe
```
