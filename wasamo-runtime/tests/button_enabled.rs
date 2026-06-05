//! Mock-free Windows-only integration test for M3-Phase 1 T6 evidence
//! (DD-M3-P1-005): toggling `Button.enabled` through the C ABI flips the
//! widget's background brush colour on a live `WidgetNode` backed by a real
//! Compositor.
//!
//! The test bypasses the binding pipeline (T8 lands the per-type writer
//! seam) and drives the property setter directly via `wasamo_set_property`
//! using `WASAMO_VALUE_BOOL`, so it exercises:
//!   - `PropertyValue::Bool` round-tripping through the ABI conversion arms,
//!   - `Button` widget-setter dispatch for `PROP_BUTTON_ENABLED`,
//!   - the brush swap performed by `update_button_enabled`, and
//!   - click-handler suppression when disabled.

#![cfg(windows)]

use std::cell::Cell;
use std::ffi::CStr;
use std::rc::Rc;

use wasamo_runtime::ffi;
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Color;
use windows::UI::Composition::{CompositionColorBrush, Visual};

const PROP_BUTTON_ENABLED: u32 = 5;

fn last_error() -> Option<String> {
    let ptr = ffi::wasamo_last_error_message();
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_str()
                .expect("last-error must be UTF-8")
                .to_owned(),
        )
    }
}

fn runtime_compositor_unavailable(msg: Option<&str>) -> bool {
    msg.is_some_and(|m| m.contains("0x80070005"))
}

fn github_actions() -> bool {
    std::env::var_os("GITHUB_ACTIONS").is_some()
}

fn read_button_color(widget: &WidgetNode) -> Color {
    let brush = widget
        .visual
        .Brush()
        .expect("button visual must have a brush");
    let color_brush: CompositionColorBrush = brush
        .cast()
        .expect("button visual brush must be a CompositionColorBrush");
    color_brush.Color().expect("read brush colour")
}

fn make_bool_value(b: bool) -> ffi::WasamoValue {
    ffi::WasamoValue {
        tag: ffi::WASAMO_VALUE_BOOL,
        payload: ffi::WasamoValuePayload {
            v_bool: if b { 1 } else { 0 },
        },
    }
}

#[test]
fn button_enabled_property_flips_visual_and_suppresses_click() {
    // No shared keep-alive helper needed: this binary has a single Compositor
    // test, so no later test can reuse a Compositor whose apartment was torn
    // down — the crash tests/common/mod.rs guards against cannot occur here.
    let _ = unsafe {
        windows::Win32::System::WinRT::RoInitialize(
            windows::Win32::System::WinRT::RO_INIT_SINGLETHREADED,
        )
    };

    let status = ffi::wasamo_init();
    if status == ffi::WASAMO_ERR_RUNTIME {
        let msg = last_error();
        if runtime_compositor_unavailable(msg.as_deref()) {
            assert!(
                !github_actions(),
                "Button.enabled headless test cannot skip on GitHub Actions: \
                 runtime compositor unavailable ({msg:?})"
            );
            eprintln!(
                "skipping Button.enabled headless test: runtime compositor unavailable ({msg:?})"
            );
            return;
        }
    }
    assert_eq!(
        status,
        ffi::WASAMO_OK,
        "wasamo_init failed: {:?}",
        last_error()
    );

    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    let mut button = WidgetNode::button(
        compositor,
        text_renderer,
        "Click me",
        wasamo_runtime::ButtonStyle::Default,
    )
    .expect("button construction must succeed");

    // Initial state: enabled=true, brush is the Default/Normal hue, click
    // dispatch fires the registered callback.
    let click_count = Rc::new(Cell::new(0u32));
    {
        let counter = Rc::clone(&click_count);
        button.set_clicked(move || {
            counter.set(counter.get() + 1);
        });
    }
    let initial_color = read_button_color(&button);

    // `WidgetNode::button` does not size the background visual itself — that
    // normally happens during the layout pass owned by the host window. The
    // test bypasses windowing, so manually pin the visual at (0,0)/(100,40)
    // so `hit_test_click` has a non-empty rect to land inside.
    let bg_vis: Visual = button
        .visual
        .cast()
        .expect("cast bg SpriteVisual to Visual");
    bg_vis
        .SetOffset(Vector3 {
            X: 0.0,
            Y: 0.0,
            Z: 0.0,
        })
        .expect("set bg offset");
    bg_vis
        .SetSize(Vector2 { X: 100.0, Y: 40.0 })
        .expect("set bg size");

    let widget_ptr = (&mut *button) as *mut WidgetNode as *mut ffi::WasamoWidget;
    button.hit_test_click(50, 20);
    assert_eq!(
        click_count.get(),
        1,
        "click on an enabled button must invoke the registered callback"
    );

    // Read back PROP_BUTTON_ENABLED via the ABI: tag must be BOOL, value true.
    let mut value = ffi::WasamoValue {
        tag: ffi::WASAMO_VALUE_NONE,
        payload: ffi::WasamoValuePayload { v_i32: 0 },
    };
    let status = unsafe { ffi::wasamo_get_property(widget_ptr, PROP_BUTTON_ENABLED, &mut value) };
    assert_eq!(status, ffi::WASAMO_OK);
    assert_eq!(value.tag, ffi::WASAMO_VALUE_BOOL);
    assert_eq!(unsafe { value.payload.v_bool }, 1);

    // Disable via ABI write.
    let disabled = make_bool_value(false);
    let status = unsafe { ffi::wasamo_set_property(widget_ptr, PROP_BUTTON_ENABLED, &disabled) };
    assert_eq!(
        status,
        ffi::WASAMO_OK,
        "wasamo_set_property(PROP_BUTTON_ENABLED, false) failed: {:?}",
        last_error()
    );

    // Visual flip: brush colour now matches BUTTON_DISABLED_COLOR (flat grey,
    // see widget.rs `update_button_enabled`).
    let disabled_color = read_button_color(&button);
    assert_ne!(
        (
            disabled_color.A,
            disabled_color.R,
            disabled_color.G,
            disabled_color.B
        ),
        (
            initial_color.A,
            initial_color.R,
            initial_color.G,
            initial_color.B
        ),
        "disabled brush colour must differ from the enabled-state colour"
    );
    assert_eq!(
        (
            disabled_color.A,
            disabled_color.R,
            disabled_color.G,
            disabled_color.B
        ),
        (0x40, 0x80, 0x80, 0x80),
        "disabled brush colour must be the documented flat grey"
    );

    // Click suppression: hit-testing the same pixel must NOT invoke the
    // callback while disabled.
    button.hit_test_click(50, 20);
    assert_eq!(
        click_count.get(),
        1,
        "click on a disabled button must not invoke the callback"
    );

    // Re-enable via ABI; brush returns to the enabled-state colour.
    let enabled = make_bool_value(true);
    let status = unsafe { ffi::wasamo_set_property(widget_ptr, PROP_BUTTON_ENABLED, &enabled) };
    assert_eq!(status, ffi::WASAMO_OK);
    let restored_color = read_button_color(&button);
    assert_eq!(
        (
            restored_color.A,
            restored_color.R,
            restored_color.G,
            restored_color.B
        ),
        (
            initial_color.A,
            initial_color.R,
            initial_color.G,
            initial_color.B
        ),
        "re-enabling must restore the original enabled-state brush colour"
    );

    // Click dispatch resumes once re-enabled.
    button.hit_test_click(50, 20);
    assert_eq!(
        click_count.get(),
        2,
        "click on a re-enabled button must invoke the callback again"
    );
}
