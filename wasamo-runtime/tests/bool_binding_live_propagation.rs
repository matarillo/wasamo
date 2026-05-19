//! Mock-free Windows-only integration test for M3-Phase 1 T13 evidence:
//! the ADR §Verification item 3 unified chain
//! `.ui → load → click → state → bound widget property`.
//!
//! Unlike [`button_enabled.rs`] — which bypasses the binding pipeline by
//! design and drives `wasamo_set_property(PROP_BUTTON_ENABLED, …)` directly
//! to exercise the widget-setter / visual-flip / click-suppression slice —
//! this test runs the full pipeline end-to-end:
//!
//!   `wasamoc` lowers a Phase 1 bool-binding fixture to IR; `parse_ir`
//!   reconstructs the `IrComponent`; `build_widget_tree` constructs a live
//!   `Button` with a `Signal<bool>`-backed binding and the inline `clicked`
//!   handler `{ root.ready = false; }`; a `hit_test_click` invocation
//!   triggers the inline handler synchronously, the handler writes to the
//!   `Signal<bool>` via `HandlerEvalContext::set_bool`, `Signal::set`
//!   drains the dirty binding effect immediately (BATCH_DEPTH == 0), and
//!   the binding effect calls `widget_write_property_bool` on the same
//!   `Button` — flipping `btn.enabled` to false in one synchronous chain.
//!
//! This is the unified chain that ADR §Verification item 3 calls for, and
//! that the original T6 widget-setter slice does not cover.

#![cfg(windows)]

use std::ffi::CStr;

use wasamo_runtime::ffi;
use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Composition::Visual;

const PROP_BUTTON_ENABLED: u32 = 5;

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<bool-binding-live-propagation>";
    let tokens = lexer::tokenize(src, path).expect("lex failed");
    let ast = parser::parse(&tokens, path).expect("parse failed");
    let checked = check::check(&ast, path);
    assert!(
        !checked.has_errors(),
        "check errors: {:?}",
        checked.diagnostics
    );
    emit::emit(&lower::lower(&ast, &checked.namespace))
}

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

unsafe fn read_bool_property(widget: *mut ffi::WasamoWidget, property_id: u32) -> bool {
    let mut value = ffi::WasamoValue {
        tag: ffi::WASAMO_VALUE_NONE,
        payload: ffi::WasamoValuePayload { v_i32: 0 },
    };
    let status = unsafe { ffi::wasamo_get_property(widget, property_id, &mut value) };
    assert_eq!(status, ffi::WASAMO_OK, "wasamo_get_property failed");
    assert_eq!(
        value.tag,
        ffi::WASAMO_VALUE_BOOL,
        "PROP_BUTTON_ENABLED must read back as WASAMO_VALUE_BOOL"
    );
    let v = unsafe { value.payload.v_bool };
    v != 0
}

#[test]
fn bool_binding_propagates_state_write_through_inline_handler_to_widget_property() {
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
                "bool-binding live-propagation test cannot skip on GitHub Actions: \
                 runtime compositor unavailable ({msg:?})"
            );
            eprintln!(
                "skipping bool-binding live-propagation test: \
                 runtime compositor unavailable ({msg:?})"
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

    // Minimal in-test variant of examples/bool-demo/bool-demo.ui: only the
    // bool binding + inline handler are kept, no window chrome / theming /
    // sibling Text label, so the test focuses on the chain ADR item 3
    // demands. `Button` as the component root is the same shape the
    // wasamoc lower-tests use (e.g. lower::tests::
    // bool_state_ident_in_prop_bind_lowered_to_bool_prop_read_binding).
    let src = r#"component BoolBinding inherits Window {
    state ready: bool = true
    Button {
        text: "Click me"
        enabled: ready
        clicked => { root.ready = false; }
    }
}"#;

    let ir = lower_ui_to_ir(src);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    let mut built =
        build_widget_tree(&component, compositor, text_renderer).expect("build_widget_tree failed");

    // The Button is the component's root widget.
    let button: &mut WidgetNode = built.root.as_mut();

    // `build_widget_tree` does not run a layout pass; `WidgetNode::button`
    // leaves the background SpriteVisual sized (0,0). Pin it so
    // `hit_test_click` lands inside — same workaround `button_enabled.rs`
    // uses for the same reason.
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

    let widget_ptr = button as *mut WidgetNode as *mut ffi::WasamoWidget;

    // The binding's initial run during `build_widget_tree` reads
    // `ready = true` and writes through `widget_write_property_bool`, so
    // the widget reads back as enabled before any click happens.
    assert!(
        unsafe { read_bool_property(widget_ptr, PROP_BUTTON_ENABLED) },
        "binding from `state ready: bool = true` must drive \
         PROP_BUTTON_ENABLED to true at build time"
    );

    // Click → inline handler `root.ready = false` runs synchronously inside
    // `hit_test_click_inner` against `HandlerEvalContext`. `Signal<bool>::
    // set` then drains the dirty binding effect immediately (BATCH_DEPTH ==
    // 0), and the binding effect calls `widget_write_property_bool(
    // PROP_BUTTON_ENABLED, false)` on the same Button. The closure in the
    // effect holds the widget by raw `WidgetId`, not by Rust reference, so
    // it can mutate the live `Button` while the outer `&mut WidgetNode`
    // borrow is still active inside `hit_test_click`. By the time
    // `hit_test_click` returns, the widget reads back as disabled — the
    // end-to-end `.ui → load → click → state → bound widget property`
    // chain ADR §Verification item 3 calls for.
    button.hit_test_click(50, 20);

    assert!(
        !unsafe { read_bool_property(widget_ptr, PROP_BUTTON_ENABLED) },
        "after click, the bool binding must propagate \
         `state ready = false` to PROP_BUTTON_ENABLED"
    );
}
