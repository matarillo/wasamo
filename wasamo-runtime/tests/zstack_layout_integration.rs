//! Mock-free Windows-only runtime evidence for M3-Phase 6 T3.
//!
//! Drives ZStack through the production compiler -> textual IR -> runtime
//! loader -> live Composition Visual tree path. This covers the ZStack half
//! of ADR Phase 6 verification closure items (3) and (4): runtime loader
//! materialisation preserves direct child count / document order, the ZStack
//! Visual carries an outer-bounds clip, child Visuals carry no clips, and a
//! production-root `VStack { ZStack { ... } }` shape stays non-collapsed.
//!
//! The skip-guard is reused byte-for-byte from the Phase 5 Grid integration
//! pattern: local `0x80070005` from `wasamo_init` skips, GitHub Actions
//! fails rather than silently skipping.

#![cfg(windows)]

use std::ffi::CStr;

use wasamo_runtime::ffi;
use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Composition::{ContainerVisual, Visual};

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<zstack-layout-integration>";
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

fn init_runtime_or_skip(test_name: &str) -> Option<()> {
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
                "{test_name} cannot skip on GitHub Actions: \
                 runtime compositor unavailable ({msg:?})"
            );
            eprintln!("skipping {test_name}: runtime compositor unavailable ({msg:?})");
            return None;
        }
    }
    assert_eq!(
        status,
        ffi::WASAMO_OK,
        "wasamo_init failed: {:?}",
        last_error()
    );
    Some(())
}

fn visual_of(widget: &WidgetNode) -> Visual {
    widget.visual.cast().expect("cast SpriteVisual to Visual")
}

fn visual_offset(v: &Visual) -> (f32, f32) {
    let off = v.Offset().unwrap_or(Vector3 {
        X: 0.0,
        Y: 0.0,
        Z: 0.0,
    });
    (off.X, off.Y)
}

fn visual_size(v: &Visual) -> (f32, f32) {
    let sz = v.Size().unwrap_or(Vector2 { X: 0.0, Y: 0.0 });
    (sz.X, sz.Y)
}

fn assert_close(actual: f32, expected: f32, label: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.01,
        "{label}: expected {expected}, got {actual} (delta {delta})"
    );
}

fn assert_same_visual(actual: &Visual, expected: &Visual, label: &str) {
    assert_eq!(
        actual.as_raw(),
        expected.as_raw(),
        "{label}: Visual identity mismatch"
    );
}

fn assert_zstack_visual_contract(zstack: &WidgetNode, label: &str) {
    assert_eq!(
        zstack.children.len(),
        3,
        "{label}: ZStack must have three direct runtime children"
    );

    let z_visual = visual_of(zstack);
    assert!(
        z_visual.Clip().is_ok(),
        "{label}: ZStack outer Visual must carry the DD-M3-P6-002 InsetClip"
    );

    let z_container: ContainerVisual = zstack
        .visual
        .cast()
        .expect("cast ZStack SpriteVisual to ContainerVisual");
    let children = z_container.Children().expect("ZStack visual children");
    assert_eq!(
        children.Count().expect("visual child count"),
        3,
        "{label}: live Visual child collection must match WidgetNode child count"
    );
    let visual_children: Vec<Visual> = children
        .First()
        .expect("ZStack VisualCollection iterator")
        .collect();
    assert_eq!(
        visual_children.len(),
        3,
        "{label}: VisualCollection iterator must expose all direct child Visuals"
    );

    for (i, visual) in visual_children.iter().enumerate() {
        let expected = visual_of(&zstack.children[i]);
        assert_same_visual(
            visual,
            &expected,
            &format!("{label}: child Visual order index {i}"),
        );
        assert!(
            visual.Clip().is_err(),
            "{label}: ZStack child {i} Visual must not carry a per-child clip"
        );
    }
}

const ZSTACK_ROOT_SRC: &str = r#"component ZStackRoot inherits Window {
    ZStack {
        Text { text: "bottom" }
        Text { text: "middle" }
        Text {
            h-align: end
            v-align: start
            text: "top"
        }
    }
}"#;

const VSTACK_ZSTACK_SRC: &str = r#"component VStackZStackRoot inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "header" }
        ZStack {
            Text { text: "bottom" }
            Text { text: "middle" }
            Text {
                h-align: end
                v-align: start
                text: "top"
            }
        }
    }
}"#;

#[test]
fn zstack_rooted_fixture_preserves_live_visual_order_and_clip() {
    if init_runtime_or_skip("ZStack-rooted layout integration test").is_none() {
        return;
    }

    let ir = lower_ui_to_ir(ZSTACK_ROOT_SRC);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    let mut built =
        build_widget_tree(&component, compositor, text_renderer).expect("build_widget_tree failed");

    built
        .root
        .run_layout_as_window_root(300.0, 200.0)
        .expect("run_layout_as_window_root failed");

    let root = built.root.as_ref();
    let z_visual = visual_of(root);
    let (zx, zy) = visual_offset(&z_visual);
    let (zw, zh) = visual_size(&z_visual);
    assert_close(zx, 0.0, "ZStack root x");
    assert_close(zy, 0.0, "ZStack root y");
    assert_close(zw, 300.0, "ZStack root width");
    assert_close(zh, 200.0, "ZStack root height");
    assert_zstack_visual_contract(root, "ZStack-rooted fixture");
}

#[test]
fn zstack_vstack_root_fixture_pins_production_root_shape() {
    if init_runtime_or_skip("VStack-rooted ZStack layout integration test").is_none() {
        return;
    }

    let ir = lower_ui_to_ir(VSTACK_ZSTACK_SRC);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    let mut built =
        build_widget_tree(&component, compositor, text_renderer).expect("build_widget_tree failed");

    assert_eq!(
        built.root.children.len(),
        2,
        "VStack root must have two children (Button + ZStack)"
    );

    built
        .root
        .run_layout_as_window_root(300.0, 200.0)
        .expect("run_layout_as_window_root failed");

    let zstack = &built.root.children[1];
    let z_visual = visual_of(zstack);
    let (zx, zy) = visual_offset(&z_visual);
    let (zw, zh) = visual_size(&z_visual);
    assert_close(zx, 0.0, "ZStack offset x within VStack");
    assert!(
        zy > 0.0,
        "ZStack must sit below the Button within the VStack; got offset y {zy}"
    );
    assert_close(zw, 300.0, "ZStack width = VStack allocation");
    assert!(
        zh > 0.0,
        "ZStack outer Visual height must be > 0 in the production root shape; got {zh}"
    );
    assert_close(
        zy + zh,
        200.0,
        "ZStack bottom = window bottom (Fill/Fill child receives remaining height)",
    );
    assert_zstack_visual_contract(zstack, "VStack-rooted fixture");
}
