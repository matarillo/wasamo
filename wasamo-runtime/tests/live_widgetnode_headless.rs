//! Windows-only headless integration test for DD-M2-P6-011 evidence.
//!
//! This crosses the production path that the pure-logic tests deliberately
//! avoid: `.ui` source is lowered by `wasamoc`, parsed by the runtime IR
//! loader, and built into live `WidgetNode`s backed by a real Compositor /
//! TextRenderer. No visible window or message loop is required.

#![cfg(windows)]

use std::ffi::CStr;

use wasamo_runtime::ffi;
use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};

const PROP_TEXT_CONTENT: u32 = 3;

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<live-widgetnode-headless>";
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

unsafe fn read_string_property(widget: *mut ffi::WasamoWidget, property_id: u32) -> String {
    let mut value = ffi::WasamoValue {
        tag: ffi::WASAMO_VALUE_NONE,
        payload: ffi::WasamoValuePayload { v_i32: 0 },
    };
    let status = ffi::wasamo_get_property(widget, property_id, &mut value);
    assert_eq!(status, ffi::WASAMO_OK, "wasamo_get_property failed");
    assert_eq!(value.tag, ffi::WASAMO_VALUE_STRING);

    let view = value.payload.v_string;
    assert!(
        !view.ptr.is_null(),
        "string property pointer must be non-null"
    );
    let bytes = std::slice::from_raw_parts(view.ptr as *const u8, view.len);
    std::str::from_utf8(bytes)
        .expect("property string must be UTF-8")
        .to_owned()
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

#[test]
fn string_binding_reaches_live_widgetnode_property_state() {
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
                "live WidgetNode headless test cannot skip on GitHub Actions: \
                 runtime compositor unavailable ({msg:?})"
            );
            eprintln!(
                "skipping live WidgetNode headless test: runtime compositor unavailable ({msg:?})"
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

    let src = r#"component StringBinding inherits Window {
    state label: string = "Ready"
    VStack {
        Text { text: "State: \{root.label}" }
    }
}"#;
    let ir = lower_ui_to_ir(src);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let rt_compositor = wasamo_runtime::get_compositor();
    let rt_text = wasamo_runtime::get_text_renderer();
    let mut built =
        build_widget_tree(&component, rt_compositor, rt_text).expect("build_widget_tree failed");

    assert_eq!(built.root.children.len(), 1);
    let text = built.root.children[0].as_mut() as *mut _;
    let content = unsafe { read_string_property(text, PROP_TEXT_CONTENT) };

    assert_eq!(content, "State: Ready");
}
