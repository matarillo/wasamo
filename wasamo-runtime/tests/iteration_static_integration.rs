//! Mock-free Windows-only runtime evidence for M3-Phase 7 T6.
//!
//! Covers static `for` materialisation through the production
//! wasamoc -> textual IR -> runtime loader -> live WidgetNode path.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::sync::{Mutex, OnceLock};

use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Composition::Visual;

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn test_lock() -> &'static Mutex<()> {
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<iteration-static-integration>";
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

#[test]
fn static_for_materialises_initial_children_and_loop_local_bindings() {
    run_on_owning_runtime_thread_or_skip("iteration static materialisation", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let src = r#"component IterStatic inherits Window {
    state labels: string[] = ["A", "B"]
    VStack {
        Text { text: "before" }
        for label, i in labels {
            Text { text: "\{label}#\{i}" }
        }
        Text { text: "after" }
    }
}"#;
        let ir = lower_ui_to_ir(src);
        let component = parse_ir(&ir).expect("parse_ir failed");
        let compositor = wasamo_runtime::get_compositor();
        let text_renderer = wasamo_runtime::get_text_renderer();
        let built = build_widget_tree(&component, compositor, text_renderer)
            .expect("build_widget_tree failed");

        assert_eq!(
            child_texts(&built.root),
            ["before", "A#0", "B#1", "after"],
            "static siblings and generated children must share declared order"
        );
    });
}

#[test]
fn static_for_materialises_i32_item_and_index_bindings() {
    run_on_owning_runtime_thread_or_skip("iteration static i32 materialisation", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let src = r#"component IterStaticI32 inherits Window {
    state nums: i32[] = [10, 20]
    WrapPanel {
        for n, i in nums {
            Text { text: "\{n}@\{i}" }
        }
    }
}"#;
        let ir = lower_ui_to_ir(src);
        let component = parse_ir(&ir).expect("parse_ir failed");
        let compositor = wasamo_runtime::get_compositor();
        let text_renderer = wasamo_runtime::get_text_renderer();
        let built = build_widget_tree(&component, compositor, text_renderer)
            .expect("build_widget_tree failed");

        assert_eq!(child_texts(&built.root), ["10@0", "20@1"]);
    });
}

#[test]
fn static_for_empty_initial_collection_materialises_zero_children() {
    run_on_owning_runtime_thread_or_skip("iteration empty static materialisation", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let src = r#"component IterEmpty inherits Window {
    state labels: string[] = []
    VStack {
        Text { text: "before" }
        for label in labels {
            Text { text: label }
        }
        Text { text: "after" }
    }
}"#;
        let ir = lower_ui_to_ir(src);
        let component = parse_ir(&ir).expect("parse_ir failed");
        let compositor = wasamo_runtime::get_compositor();
        let text_renderer = wasamo_runtime::get_text_renderer();
        let built = build_widget_tree(&component, compositor, text_renderer)
            .expect("build_widget_tree failed");

        assert_eq!(
            child_texts(&built.root),
            ["before", "after"],
            "empty initial collection must leave the declared for slot live but materialise no children"
        );
    });
}

#[test]
fn static_for_composes_with_preceding_conditional_slot_offsets() {
    run_on_owning_runtime_thread_or_skip("iteration static conditional offsets", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        for (condition, expected) in [
            (true, vec!["before", "maybe", "A", "B", "after"]),
            (false, vec!["before", "A", "B", "after"]),
        ] {
            let src = format!(
                r#"component IterConditionalOffset inherits Window {{
    state show: bool = {condition}
    state labels: string[] = ["A", "B"]
    VStack {{
        Text {{ text: "before" }}
        if show {{
            Text {{ text: "maybe" }}
        }}
        for label in labels {{
            Text {{ text: label }}
        }}
        Text {{ text: "after" }}
    }}
}}"#
            );
            let ir = lower_ui_to_ir(&src);
            let component = parse_ir(&ir).expect("parse_ir failed");
            let compositor = wasamo_runtime::get_compositor();
            let text_renderer = wasamo_runtime::get_text_renderer();
            let built = build_widget_tree(&component, compositor, text_renderer)
                .expect("build_widget_tree failed");

            assert_eq!(
                child_texts(&built.root),
                expected,
                "condition={condition} must preserve declared order"
            );
        }
    });
}

#[test]
fn static_for_under_zstack_preserves_child_carried_placement() {
    run_on_owning_runtime_thread_or_skip("iteration static ZStack placement", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let src = r#"component IterZStack inherits Window {
    state labels: string[] = ["A", "B"]
    ZStack {
        for label in labels {
            Text {
                slot.h-align: end
                slot.v-align: start
                text: label
            }
        }
    }
}"#;
        let ir = lower_ui_to_ir(src);
        let component = parse_ir(&ir).expect("parse_ir failed");
        let compositor = wasamo_runtime::get_compositor();
        let text_renderer = wasamo_runtime::get_text_renderer();
        let mut built = build_widget_tree(&component, compositor, text_renderer)
            .expect("build_widget_tree failed");

        assert_eq!(child_texts(&built.root), ["A", "B"]);
        built
            .root
            .run_layout_as_window_root(300.0, 200.0)
            .expect("run_layout_as_window_root failed");

        for (index, child) in built.root.children.iter().enumerate() {
            let visual = visual_of(child);
            let (x, y) = visual_offset(&visual);
            let (w, h) = visual_size(&visual);
            assert!(
                w > 0.0 && h > 0.0,
                "generated ZStack child {index} must have a non-zero measured size; got ({w}, {h})"
            );
            assert_close(y, 0.0, "generated ZStack v-align=start offset");
            assert_close(x + w, 300.0, "generated ZStack h-align=end right edge");
        }
    });
}
