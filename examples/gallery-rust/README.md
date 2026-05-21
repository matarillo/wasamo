# gallery-rust — M3 gallery host

A minimal Rust host for the gallery sub-screen. The UI lives in
[`examples/gallery/gallery.ui`](../gallery/gallery.ui); `build.rs`
compiles it to Wasamo IR via the workspace `wasamoc` crate, then
`main.rs` loads the generated IR through `wasamo_load_ui`.

The Phase 3 sub-screen is a `WrapPanel` of eight uniform 1:1 `Box`
thumbnails with `item-cross-size: 120; item-spacing: 16;
line-spacing: 16`. On an 800×600 default window the line breaker
fits five thumbnails on the first line and wraps the remaining
three onto a second line — the visible smoke for the M3-Phase 3
WrapPanel primitive. Later M3 phases grow `examples/gallery/`
further sub-screen by sub-screen.

```powershell
cargo build --release -p gallery-rust
Start-Process .\target\release\gallery-rust.exe
```
