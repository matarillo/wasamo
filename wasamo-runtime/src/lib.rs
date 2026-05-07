mod abi;
mod emit;
pub mod handler;
pub mod ir_loader;
mod layout;
pub(crate) mod reactive;
mod registry;
mod runtime;
mod text;
mod widget;
mod window;

pub use layout::{Alignment, SizeConstraint, WidgetKind};
pub use text::{TextRenderer, TypographyStyle};
pub use widget::{ButtonStyle, WidgetNode};
pub use window::WindowState;

use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG};

// ── Rust-native API (used by examples and future bindings) ───────────────────

pub fn init() -> windows::core::Result<()> {
    runtime::init()
}

pub fn window_create(
    title: &str,
    width: i32,
    height: i32,
) -> windows::core::Result<Box<WindowState>> {
    window::create(title, width, height)
}

pub fn window_show(state: &WindowState) {
    window::show(state);
}

pub fn get_compositor() -> &'static windows::UI::Composition::Compositor {
    &runtime::get().compositor
}

pub fn get_text_renderer() -> &'static TextRenderer {
    &runtime::get().text_renderer
}

pub fn window_add_widget(
    window: &WindowState,
    widget: &WidgetNode,
) -> windows::core::Result<()> {
    use windows::core::Interface;
    use windows::UI::Composition::Visual;
    let child_visual: Visual = widget.visual.cast()?;
    window.root.Children()?.InsertAtTop(&child_visual)?;
    Ok(())
}

/// Install `root` as the window's content tree, taking ownership of the
/// subtree. Performs an initial layout pass against the window's current
/// client size and registers the root for click hit-testing and
/// resize-driven re-layout.
///
/// Counterpart to `window_add_widget` — the latter merely attaches a Visual
/// to the Composition tree without putting the widget into the window's
/// `root_widget` slot, so it is suitable only for direct visual hosting (no
/// layout, no hit-test). IR-loader-driven trees should use `window_set_root`.
pub fn window_set_root(
    window: &mut WindowState,
    root: Box<WidgetNode>,
) -> windows::core::Result<()> {
    window::set_root(window, root)
}

pub fn run() {
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
            // Drain queued callback emissions between message dispatches —
            // the message-loop iteration boundary is a "no host code is
            // currently inside a wasamo_* call" point (abi_spec §6).
            emit::drain_if_outermost();
        }
    }
}

// C ABI surface (wasamo.h) lives in `abi.rs`. The `mod abi;` declaration above
// is sufficient to register the `#[no_mangle] pub extern "C"` functions for
// linkage; nothing else needs to be re-exported here.
