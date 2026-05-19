# bool-demo-rust — M3-Phase 1 bool binding proof

A minimal Rust host for the M3-Phase 1 `bool` scalar binding surface.
The UI lives in
[`examples/bool-demo/bool-demo.ui`](../bool-demo/bool-demo.ui);
`build.rs` compiles it to Wasamo IR via the workspace `wasamoc` crate,
then `main.rs` loads the generated IR through `wasamo_load_ui`.

The fixture declares `state ready: bool = true`, binds
`Button.enabled` to `ready`, and sets `ready = false` from the button's
`clicked` handler. Launching the host should show an enabled accent
button; after one click it becomes disabled and visibly grey.

```powershell
cargo build --release -p bool-demo-rust
target\release\bool-demo-rust.exe
```
