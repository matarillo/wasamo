// Phase 4 visual check: Text + Button widgets, hover/press states, WM_SIZE re-layout.
//
// What to verify:
//   - A Title text label and two Buttons (Increment / Reset) are visible on the Mica background.
//   - Hovering a Button lightens its background; pressing darkens it.
//   - Clicking a Button prints the new counter value to stdout.
//   - Resizing the window re-runs layout; the VStack reflows to the new width.

use std::sync::{Arc, Mutex};
use wasamo_runtime::{
    Alignment, ButtonStyle, TextRenderer, TypographyStyle, WidgetNode,
};

fn main() -> windows::core::Result<()> {
    wasamo_runtime::init()?;
    let compositor = wasamo_runtime::get_compositor();
    let renderer = TextRenderer::new(compositor)?;

    let window_w = 640.0f32;
    let window_h = 480.0f32;

    // ── Build widget tree ────────────────────────────────────────────────────

    let mut root = WidgetNode::vstack(compositor, 12.0, 24.0, Alignment::Center)?;

    let counter = Arc::new(Mutex::new(0u32));

    let title = WidgetNode::text(
        compositor,
        &renderer,
        "Count: 0",
        TypographyStyle::Title,
    )?;
    root.append_child(title)?;

    let mut increment_btn = WidgetNode::button(
        compositor,
        &renderer,
        "Increment",
        ButtonStyle::Accent,
    )?;

    let mut reset_btn = WidgetNode::button(
        compositor,
        &renderer,
        "Reset",
        ButtonStyle::Default,
    )?;

    let counter_inc = counter.clone();
    increment_btn.set_clicked(move || {
        let mut c = counter_inc.lock().unwrap();
        *c += 1;
        println!("Count: {}", *c);
    });

    let counter_reset = counter.clone();
    reset_btn.set_clicked(move || {
        let mut c = counter_reset.lock().unwrap();
        *c = 0;
        println!("Count: {}", *c);
    });

    root.append_child(increment_btn)?;
    root.append_child(reset_btn)?;

    root.run_layout(window_w, window_h)?;

    // ── Attach root to window ────────────────────────────────────────────────

    let mut window =
        wasamo_runtime::window_create("Phase 4 Visual Check", window_w as i32, window_h as i32)?;
    wasamo_runtime::window_add_widget(&window, &root)?;

    // ── Event callbacks ──────────────────────────────────────────────────────
    //
    // Safety: root outlives window (declared before it on the stack; Rust drops
    // in reverse order). The callbacks are only invoked inside wasamo_runtime::run(),
    // which completes before root is dropped.

    let root_ptr: *mut WidgetNode = root.as_mut();
    let compositor_ptr: *const _ = compositor;

    window.resize_fn = Some(Box::new(move |w, h| {
        let _ = unsafe { (*root_ptr).run_layout(w, h) };
    }));

    window.mouse_move_fn = Some(Box::new(move |x, y| {
        let _ = unsafe { (*root_ptr).update_hover(&*compositor_ptr, x, y, false) };
    }));

    window.mouse_leave_fn = Some(Box::new(move || {
        let _ = unsafe { (*root_ptr).clear_hover(&*compositor_ptr) };
    }));

    window.mouse_down_fn = Some(Box::new(move |x, y| {
        unsafe { (*root_ptr).hit_test_click(x, y) };
    }));

    wasamo_runtime::window_show(&window);
    wasamo_runtime::run();
    Ok(())
}
