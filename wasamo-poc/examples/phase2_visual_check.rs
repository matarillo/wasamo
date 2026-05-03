// Phase 2 visual check: open a window with Visual Layer attached and render
// a solid-colour SpriteVisual to confirm the Compositor pipeline is working.
//
// Expected result:
//   - Window opens with Mica backdrop (Win11) or plain background (Win10)
//   - A blue (Windows accent #0078D4) square is visible in the upper-left area
//   - Closing the window exits the process cleanly

use windows::{
    Foundation::Numerics::Vector3,
    UI::{
        Color,
        Composition::Visual,
    },
    core::Interface,
};

fn main() -> windows::core::Result<()> {
    wasamo_runtime::init()?;

    let window = wasamo_runtime::window_create("Phase 2 — Visual Layer Check", 640, 480)?;

    // Add a solid-colour SpriteVisual to confirm the Visual Layer is live.
    let compositor = wasamo_runtime::get_compositor();
    let sprite = compositor.CreateSpriteVisual()?;
    sprite.cast::<Visual>()?.SetSize(windows::Foundation::Numerics::Vector2 { X: 200.0, Y: 200.0 })?;
    sprite.cast::<Visual>()?.SetOffset(Vector3 { X: 40.0, Y: 40.0, Z: 0.0 })?;
    let brush = compositor.CreateColorBrushWithColor(Color { A: 255, R: 0x00, G: 0x78, B: 0xD4 })?;
    sprite.SetBrush(&brush)?;

    window
        .root
        .Children()?
        .InsertAtTop(&sprite.cast::<Visual>()?)?;

    wasamo_runtime::window_show(&window);
    wasamo_runtime::run();
    Ok(())
}
