// Phase 5 visual check: compositor independence verification.
//
// What to verify:
//   - Button hover/press transitions animate smoothly via ColorKeyFrameAnimation.
//   - A magenta synthetic SpriteVisual oscillates continuously in the top-right corner.
//   - Press [B] to block the app thread for ~2 s: the synthetic visual keeps animating
//     and Mica continues to render — compositor runs on its own thread (DD-P5-006).
//   - No animation toggle or property-change animation API is present (DD-V-001).

use std::sync::{Arc, Mutex};
use wasamo::{Alignment, ButtonStyle, TextRenderer, TypographyStyle, WidgetNode};
use windows::{
    core::{Interface, HSTRING},
    Foundation::{Numerics::{Vector2, Vector3}, TimeSpan},
    UI::{
        Color,
        Composition::{
            AnimationIterationBehavior, CompositionAnimation, CompositionObject,
            Vector3KeyFrameAnimation,
        },
    },
};

fn main() -> windows::core::Result<()> {
    wasamo::init()?;
    let compositor = wasamo::get_compositor();
    let renderer = TextRenderer::new(compositor)?;

    let window_w = 640.0_f32;
    let window_h = 480.0_f32;

    // ── Widget tree (same as Phase 4) ────────────────────────────────────────

    let mut root = WidgetNode::vstack(compositor, 12.0, 24.0, Alignment::Center)?;

    let counter = Arc::new(Mutex::new(0u32));

    let title = WidgetNode::text(compositor, &renderer, "Count: 0", TypographyStyle::Title)?;
    root.append_child(title)?;

    let mut increment_btn =
        WidgetNode::button(compositor, &renderer, "Increment", ButtonStyle::Accent)?;
    let mut reset_btn =
        WidgetNode::button(compositor, &renderer, "Reset", ButtonStyle::Default)?;

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

    // ── Window ───────────────────────────────────────────────────────────────

    let mut window =
        wasamo::window_create("Phase 5 Visual Check", window_w as i32, window_h as i32)?;
    wasamo::window_add_widget(&window, &root)?;

    // ── Synthetic SpriteVisual (DD-P5-006) ───────────────────────────────────
    //
    // A 32×32 magenta square that continuously oscillates horizontally in the
    // top-right corner, driven by a looping Vector3KeyFrameAnimation on Offset.
    // Because the animation runs on the compositor thread, it keeps moving even
    // when the app thread is blocked (press [B] to verify).

    let synth = compositor.CreateSpriteVisual()?;
    let synth_brush =
        compositor.CreateColorBrushWithColor(Color { A: 255, R: 230, G: 0, B: 180 })?;
    synth.SetBrush(&synth_brush)?;

    let synth_vis: windows::UI::Composition::Visual = synth.cast()?;
    synth_vis.SetSize(Vector2 { X: 32.0, Y: 32.0 })?;

    // Oscillate between x = window_w - 50 and x = window_w - 90, y = 16.
    let x0 = window_w - 50.0;
    let x1 = window_w - 90.0;
    let sy = 16.0_f32;
    synth_vis.SetOffset(Vector3 { X: x0, Y: sy, Z: 0.0 })?;

    // ~2-second looping Vector3KeyFrameAnimation on the Offset property.
    let anim: Vector3KeyFrameAnimation = compositor.CreateVector3KeyFrameAnimation()?;
    anim.InsertKeyFrame(0.0_f32, Vector3 { X: x0, Y: sy, Z: 0.0 })?;
    anim.InsertKeyFrame(0.5_f32, Vector3 { X: x1, Y: sy, Z: 0.0 })?;
    anim.InsertKeyFrame(1.0_f32, Vector3 { X: x0, Y: sy, Z: 0.0 })?;
    anim.SetDuration(TimeSpan { Duration: 20_000_000 })?; // 2 s = 20 000 000 × 100 ns
    anim.SetIterationBehavior(AnimationIterationBehavior::Forever)?;

    let comp_anim: CompositionAnimation = anim.cast()?;
    let synth_obj: CompositionObject = synth_vis.cast()?;
    synth_obj.StartAnimation(&HSTRING::from("Offset"), &comp_anim)?;

    // Attach synthetic visual directly to the window root (DD-P5-006 "pub root" hook).
    window.root.Children()?.InsertAtTop(&synth_vis)?;

    // ── Event callbacks ──────────────────────────────────────────────────────
    //
    // Safety: root outlives window (declared before it; Rust drops in reverse order).
    // Callbacks are only invoked inside wasamo::run(), which completes first.

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
        let _ = unsafe { (*root_ptr).update_hover(&*compositor_ptr, x, y, true) };
        unsafe { (*root_ptr).hit_test_click(x, y) };
    }));

    window.mouse_up_fn = Some(Box::new(move |x, y| {
        let _ = unsafe { (*root_ptr).update_hover(&*compositor_ptr, x, y, false) };
    }));

    // [B]: block the app thread for ~2 s to verify compositor independence.
    window.key_down_fn = Some(Box::new(move |vk| {
        if vk == 0x42 {
            println!("[B] pressed – blocking app thread for 2 s …");
            std::thread::sleep(std::time::Duration::from_secs(2));
            println!("[B] done – compositor should have kept animating");
        }
    }));

    wasamo::window_show(&window);
    wasamo::run();
    Ok(())
}
