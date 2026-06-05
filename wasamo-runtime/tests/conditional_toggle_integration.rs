//! Mock-free Windows-only runtime evidence for M3-Phase 6 T5.
//!
//! Drives conditional rendering through the production compiler -> textual IR
//! -> runtime loader -> live Widget/Visual tree path. The fixtures pin
//! structural present/absent mutation, declared sibling order, no-op
//! re-evaluation, structural teardown, and same-drain initialisation of
//! freshly-created subtree bindings.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use wasamo_runtime::ffi;
use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Composition::{ContainerVisual, Visual};
static DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn test_lock() -> &'static Mutex<()> {
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<conditional-toggle-integration>";
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
    let diff = (actual - expected).abs();
    assert!(
        diff <= 0.5,
        "{label}: expected {expected}, got {actual} (diff {diff})"
    );
}

fn child_texts(parent: &WidgetNode) -> Vec<String> {
    parent
        .children
        .iter()
        .map(|child| {
            child
                .__text_content_for_test()
                .expect("child must be Text")
                .to_owned()
        })
        .collect()
}

fn assert_visual_order(parent: &WidgetNode, label: &str) {
    let container: ContainerVisual = parent
        .visual
        .cast()
        .expect("cast parent SpriteVisual to ContainerVisual");
    let children = container.Children().expect("parent visual children");
    assert_eq!(
        children.Count().expect("visual child count") as usize,
        parent.children.len(),
        "{label}: VisualCollection count must match WidgetNode children"
    );
    let visual_children: Vec<Visual> = children
        .First()
        .expect("VisualCollection iterator")
        .collect();
    for (i, visual) in visual_children.iter().enumerate() {
        assert_eq!(
            visual.as_raw(),
            visual_of(&parent.children[i]).as_raw(),
            "{label}: child Visual order index {i}"
        );
    }
}

fn assert_order(parent: &WidgetNode, expected: &[&str], label: &str) {
    assert_eq!(child_texts(parent), expected, "{label}: widget child order");
    assert_visual_order(parent, label);
}

unsafe extern "C" fn noop_signal_handler(
    _sender: *mut ffi::WasamoWidget,
    _args: *const ffi::WasamoValue,
    _arg_count: usize,
    _user_data: *mut std::ffi::c_void,
) {
}

unsafe extern "C" fn count_destroy(_user_data: *mut std::ffi::c_void) {
    DESTROY_COUNT.fetch_add(1, Ordering::SeqCst);
}

fn connect_destroy_counted_signal(widget: &mut WidgetNode) -> u64 {
    unsafe {
        ffi::__add_signal_registration_for_test(
            widget as *mut WidgetNode as *mut ffi::WasamoWidget,
            "clicked",
            Some(noop_signal_handler),
            std::ptr::null_mut(),
            Some(count_destroy),
        )
    }
}

const ORDER_SRC: &str = r#"component ConditionalOrder inherits Window {
    state first: bool = true
    state second: bool = true
    ZStack {
        Text { text: "bottom" }
        if first {
            Text { text: "first" }
        }
        if second {
            Text { text: "second" }
        }
        Text { text: "top" }
    }
}"#;

#[test]
fn conditional_toggle_preserves_declared_visual_order_and_disposes_registry() {
    // Marshalled onto the runtime-owning thread (Observation 5 step 1): the
    // Compositor is created and used on one thread. See tests/common/mod.rs.
    run_on_owning_runtime_thread_or_skip(
        "conditional declared-order integration test",
        move || {
            let _guard = test_lock().lock().expect("test lock poisoned");
            DESTROY_COUNT.store(0, Ordering::SeqCst);

            let ir = lower_ui_to_ir(ORDER_SRC);
            let component = parse_ir(&ir).expect("parse_ir failed");
            let compositor = wasamo_runtime::get_compositor();
            let text_renderer = wasamo_runtime::get_text_renderer();
            let mut built = build_widget_tree(&component, compositor, text_renderer)
                .expect("build_widget_tree failed");

            assert_order(
                &built.root,
                &["bottom", "first", "second", "top"],
                "initial both-present state",
            );

            let first_token = connect_destroy_counted_signal(&mut built.root.children[1]);
            assert!(first_token != 0, "signal token must be non-zero");
            assert_eq!(
                ffi::__signal_tokens_for_test(
                    built.root.children[1].as_mut() as *mut WidgetNode as *mut ffi::WasamoWidget,
                    "clicked",
                ),
                vec![first_token],
                "test registration must be visible before subtree removal"
            );

            assert!(built.__set_bool_state_for_test("first", false));
            assert_order(
                &built.root,
                &["bottom", "second", "top"],
                "preceding conditional removed while following conditional remains",
            );
            assert_eq!(
                DESTROY_COUNT.load(Ordering::SeqCst),
                1,
                "widget_destroy must sever registry entries in the absent subtree"
            );

            assert!(built.__set_bool_state_for_test("first", false));
            assert_order(
                &built.root,
                &["bottom", "second", "top"],
                "false-to-false re-evaluation is a no-op",
            );

            assert!(built.__set_bool_state_for_test("first", true));
            assert_order(
                &built.root,
                &["bottom", "first", "second", "top"],
                "re-presented preceding conditional returns to declared slot",
            );

            assert!(built.__set_bool_state_for_test("first", true));
            assert_order(
                &built.root,
                &["bottom", "first", "second", "top"],
                "true-to-true re-evaluation is a no-op",
            );

            assert!(built.__set_bool_state_for_test("second", false));
            assert_order(
                &built.root,
                &["bottom", "first", "top"],
                "following conditional removed after preceding conditional is present",
            );
        },
    );
}

const ZSTACK_PLACEMENT_SRC: &str = r#"component ConditionalZStackPlacement inherits Window {
    state open: bool = false
    ZStack {
        Text { text: "base" }
        if open {
            Text {
                h-align: end
                v-align: start
                text: "edge"
            }
        }
    }
}"#;

#[test]
fn conditional_zstack_reinsert_uses_declared_placement_metadata() {
    // Marshalled onto the runtime-owning thread (Observation 5 step 1): the
    // Compositor is created and used on one thread. See tests/common/mod.rs.
    run_on_owning_runtime_thread_or_skip(
        "conditional ZStack placement integration test",
        move || {
            let _guard = test_lock().lock().expect("test lock poisoned");

            let ir = lower_ui_to_ir(ZSTACK_PLACEMENT_SRC);
            let component = parse_ir(&ir).expect("parse_ir failed");
            let compositor = wasamo_runtime::get_compositor();
            let text_renderer = wasamo_runtime::get_text_renderer();
            let mut built = build_widget_tree(&component, compositor, text_renderer)
                .expect("build_widget_tree failed");

            built
                .root
                .run_layout_as_window_root(300.0, 200.0)
                .expect("initial run_layout_as_window_root failed");
            assert_eq!(
                built.root.children.len(),
                1,
                "initial false condition must not materialise the aligned child"
            );

            assert!(built.__set_bool_state_for_test("open", true));
            built
                .root
                .run_layout_as_window_root(300.0, 200.0)
                .expect("open run_layout_as_window_root failed");
            assert_eq!(
                built.root.children.len(),
                2,
                "open=true must insert the aligned child"
            );
            let edge = visual_of(&built.root.children[1]);
            let (x, y) = visual_offset(&edge);
            let (w, h) = visual_size(&edge);
            assert!(
                w > 0.0 && h > 0.0,
                "aligned child must have a non-zero measured size; got ({w}, {h})"
            );
            assert_close(y, 0.0, "dynamic ZStack v-align=start offset");
            assert_close(
                x + w,
                300.0,
                "dynamic ZStack h-align=end right edge after insert",
            );

            assert!(built.__set_bool_state_for_test("open", false));
            assert!(built.__set_bool_state_for_test("open", true));
            built
                .root
                .run_layout_as_window_root(300.0, 200.0)
                .expect("re-open run_layout_as_window_root failed");
            let edge = visual_of(&built.root.children[1]);
            let (x, y) = visual_offset(&edge);
            let (w, _) = visual_size(&edge);
            assert_close(y, 0.0, "dynamic ZStack v-align=start offset after reinsert");
            assert_close(
                x + w,
                300.0,
                "dynamic ZStack h-align=end right edge after reinsert",
            );
        },
    );
}

const DRAIN_SRC: &str = r#"component ConditionalDrain inherits Window {
    state open: bool = false
    state enabled_state: bool = true
    VStack {
        Text { text: "before" }
        if open {
            Button {
                text: "inside"
                enabled: enabled_state
            }
        }
        Text { text: "after" }
    }
}"#;

fn read_button_enabled(button: &WidgetNode) -> bool {
    button
        .__button_enabled_for_test()
        .expect("child must be Button")
}

#[test]
fn conditional_toggle_drains_fresh_subtree_effects_before_return() {
    // Marshalled onto the runtime-owning thread (Observation 5 step 1): the
    // Compositor is created and used on one thread. See tests/common/mod.rs.
    run_on_owning_runtime_thread_or_skip("conditional drain integration test", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let ir = lower_ui_to_ir(DRAIN_SRC);
        let component = parse_ir(&ir).expect("parse_ir failed");
        let compositor = wasamo_runtime::get_compositor();
        let text_renderer = wasamo_runtime::get_text_renderer();
        let built = build_widget_tree(&component, compositor, text_renderer)
            .expect("build_widget_tree failed");

        assert_eq!(
            built.root.children.len(),
            2,
            "initial false condition must not materialise the Button subtree"
        );

        assert!(built.__set_bool_state_for_test("open", true));
        assert_eq!(
            built.root.children.len(),
            3,
            "open=true must synchronously insert the conditional Button"
        );
        assert!(
            read_button_enabled(&built.root.children[1]),
            "fresh Button.enabled binding must run before the open setter returns"
        );

        assert!(built.__set_bool_state_for_test("open", false));
        assert_eq!(
            built.root.children.len(),
            2,
            "open=false must synchronously remove the conditional Button"
        );

        assert!(built.__set_bool_state_for_test("enabled_state", false));
        assert!(built.__set_bool_state_for_test("open", true));
        assert_eq!(
            built.root.children.len(),
            3,
            "re-open must rebuild the conditional Button"
        );
        assert!(
            !read_button_enabled(&built.root.children[1]),
            "freshly rebuilt Button must observe the latest enabled_state \
         before the open setter returns"
        );
    });
}
