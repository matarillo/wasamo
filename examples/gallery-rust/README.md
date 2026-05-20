# gallery-rust — M3-Phase 2 gallery seed

A minimal Rust host for the first gallery sub-screen. The UI lives in
[`examples/gallery/gallery.ui`](../gallery/gallery.ui); `build.rs`
compiles it to Wasamo IR via the workspace `wasamoc` crate, then
`main.rs` loads the generated IR through `wasamo_load_ui`.

The fixture shows a single `Box` with `aspect: 16:9`, a translucent
blue fill, and a centered `Text` placeholder. Later M3 phases can grow
`examples/gallery/` sub-screen by sub-screen.

```powershell
cargo build --release -p gallery-rust
Start-Process .\target\release\gallery-rust.exe
```
