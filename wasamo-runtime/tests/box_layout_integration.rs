//! Mock-free Windows-only layout integration evidence for M3-Phase 2 T11.
//!
//! This drives the production path for an aspect-fixed Box with a Text child:
//! `.ui` source is lowered by `wasamoc`, parsed by the runtime IR loader,
//! built into live `WidgetNode`s backed by a real Compositor / TextRenderer,
//! laid out inside a known 800x800 parent extent, and then read back from the
//! resulting `SpriteVisual` tree.
//!
//! `fill` is observed through the Box-internal test accessor and the
//! `SpriteVisual` brush, never through `wasamo_get_property`, because Phase 2
//! keeps `fill` out of the `PropertyValue` / ABI value surface per
//! DD-M3-P2-003 / DD-M3-P2-004.

#![cfg(windows)]

use std::ffi::CStr;

use wasamo_runtime::ffi;
use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Color;
use windows::UI::Composition::{CompositionColorBrush, Visual};

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<box-layout-integration>";
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

fn visual_rect(widget: &WidgetNode) -> (f32, f32, f32, f32) {
    let visual: Visual = widget.visual.cast().expect("cast SpriteVisual to Visual");
    let offset = visual.Offset().unwrap_or(Vector3 {
        X: 0.0,
        Y: 0.0,
        Z: 0.0,
    });
    let size = visual.Size().unwrap_or(Vector2 { X: 0.0, Y: 0.0 });
    (offset.X, offset.Y, size.X, size.Y)
}

fn read_box_brush_color(widget: &WidgetNode) -> Color {
    let brush = widget
        .visual
        .Brush()
        .expect("Box with fill must have a brush");
    let color_brush: CompositionColorBrush = brush
        .cast()
        .expect("Box fill brush must be a CompositionColorBrush");
    color_brush.Color().expect("read Box fill brush colour")
}

fn assert_close(actual: f32, expected: f32, label: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.01,
        "{label}: expected {expected}, got {actual} (delta {delta})"
    );
}

#[test]
fn aspect_box_with_text_child_lays_out_and_paints_fill() {
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
                "Box layout integration test cannot skip on GitHub Actions: \
                 runtime compositor unavailable ({msg:?})"
            );
            eprintln!(
                "skipping Box layout integration test: runtime compositor unavailable ({msg:?})"
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

    let src = r#"component BoxLayout inherits Window {
    Box {
        aspect: 16:9
        fill: #336699cc
        Text { text: "Photo 12" }
    }
}"#;

    let ir = lower_ui_to_ir(src);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    let mut built =
        build_widget_tree(&component, compositor, text_renderer).expect("build_widget_tree failed");

    let root = built.root.as_mut();
    assert_eq!(
        root.__box_state_for_test(),
        Some((Some((16, 9)), Some(0xcc_33_66_99))),
        "Box-internal state must carry aspect/fill outside the PropertyValue surface"
    );
    assert_eq!(root.children.len(), 1, "Box must have one Text child");

    root.run_layout(800.0, 800.0).expect("run_layout failed");

    let (box_x, box_y, box_w, box_h) = visual_rect(root);
    assert_close(box_x, 0.0, "Box visual x");
    assert_close(box_y, 0.0, "Box visual y");
    assert_close(box_w, 800.0, "Box visual width");
    assert_close(box_h, 450.0, "Box visual height");

    let fill = read_box_brush_color(root);
    assert_eq!(
        (fill.A, fill.R, fill.G, fill.B),
        (0xcc, 0x33, 0x66, 0x99),
        "Box fill must materialise as the SpriteVisual colour brush"
    );

    let (text_x, text_y, text_w, text_h) = visual_rect(&root.children[0]);
    assert!(
        text_w > 0.0 && text_h > 0.0,
        "Text child must retain a non-empty intrinsic visual size"
    );
    assert_close(text_x, (box_w - text_w) / 2.0, "Text child centred x");
    assert_close(text_y, (box_h - text_h) / 2.0, "Text child centred y");
}
