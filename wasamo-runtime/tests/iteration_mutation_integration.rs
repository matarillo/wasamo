//! Mock-free Windows-only runtime evidence for M3-Phase 7 T7.
//!
//! Covers reactive `for` range mutation through the production
//! wasamoc -> textual IR -> runtime loader -> live Widget/Visual tree path.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use wasamo_ir::{
    ControlFlowNode, HandlerExpr, IrComponent, IrLiteral, IrMember, IrNode, IrState, IrStateType,
    IrType,
};
use wasamo_runtime::ffi;
use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Composition::{ContainerVisual, Visual};

static DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);
static DESTROY_LOG: OnceLock<Mutex<Vec<usize>>> = OnceLock::new();
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn test_lock() -> &'static Mutex<()> {
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn destroy_log() -> &'static Mutex<Vec<usize>> {
    DESTROY_LOG.get_or_init(|| Mutex::new(Vec::new()))
}

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<iteration-mutation-integration>";
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

fn build(src: &str) -> wasamo_runtime::ir_loader::BuiltUi {
    let ir = lower_ui_to_ir(src);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    build_widget_tree(&component, compositor, text_renderer).expect("build_widget_tree failed")
}

fn build_memory_ir(component: &IrComponent) -> wasamo_runtime::ir_loader::BuiltUi {
    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    build_widget_tree(component, compositor, text_renderer).expect("build_widget_tree failed")
}

fn text_children(parent: &WidgetNode) -> Vec<String> {
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

fn text_children_from(parent: &WidgetNode, start: usize) -> Vec<String> {
    parent.children[start..]
        .iter()
        .map(|child| {
            child
                .__text_content_for_test()
                .expect("child must be Text")
                .to_owned()
        })
        .collect()
}

fn child_ptr(parent: &WidgetNode, index: usize) -> *const WidgetNode {
    parent.children[index].as_ref() as *const WidgetNode
}

fn visual_of(widget: &WidgetNode) -> Visual {
    widget.visual.cast().expect("cast SpriteVisual to Visual")
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

unsafe extern "C" fn log_destroy(user_data: *mut std::ffi::c_void) {
    let tag = user_data as usize;
    destroy_log()
        .lock()
        .expect("destroy log poisoned")
        .push(tag);
}

fn connect_destroy_counted_signal(widget: &mut WidgetNode) {
    let token = unsafe {
        ffi::__add_signal_registration_for_test(
            widget as *mut WidgetNode as *mut ffi::WasamoWidget,
            "clicked",
            Some(noop_signal_handler),
            std::ptr::null_mut(),
            Some(count_destroy),
        )
    };
    assert_ne!(token, 0, "signal token must be non-zero");
}

fn connect_destroy_logged_signal(widget: &mut WidgetNode, tag: usize) {
    let token = unsafe {
        ffi::__add_signal_registration_for_test(
            widget as *mut WidgetNode as *mut ffi::WasamoWidget,
            "clicked",
            Some(noop_signal_handler),
            tag as *mut std::ffi::c_void,
            Some(log_destroy),
        )
    };
    assert_ne!(token, 0, "signal token must be non-zero");
}

#[test]
fn reactive_for_tail_append_reset_remove_preserves_order_and_prefix_identity() {
    run_on_owning_runtime_thread_or_skip("iteration reactive range mutation", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");
        DESTROY_COUNT.store(0, Ordering::SeqCst);
        destroy_log().lock().expect("destroy log poisoned").clear();

        let src = r#"component IterMutation inherits Window {
    state show: bool = true
    state labels: string[] = ["A", "B"]
    VStack {
        Text { text: "before" }
        if show {
            Text { text: "maybe" }
        }
        for label in labels {
            Text { text: label }
        }
        Text { text: "after" }
    }
}"#;
        let mut built = build(src);

        assert_eq!(
            text_children(&built.root),
            ["before", "maybe", "A", "B", "after"]
        );
        assert_visual_order(&built.root, "initial");
        let before_ptr = child_ptr(&built.root, 0);
        let maybe_ptr = child_ptr(&built.root, 1);
        let a_ptr = child_ptr(&built.root, 2);
        let b_ptr = child_ptr(&built.root, 3);
        connect_destroy_counted_signal(&mut built.root.children[3]);

        assert!(built
            .__set_string_list_state_for_test("labels", vec!["A".into(), "B".into(), "C".into()]));
        assert_eq!(
            text_children(&built.root),
            ["before", "maybe", "A", "B", "C", "after"],
            "tail append must insert before following static sibling"
        );
        assert_visual_order(&built.root, "after append");
        assert_eq!(child_ptr(&built.root, 0), before_ptr);
        assert_eq!(child_ptr(&built.root, 1), maybe_ptr);
        assert_eq!(child_ptr(&built.root, 2), a_ptr);
        assert_eq!(child_ptr(&built.root, 3), b_ptr);
        let c_ptr = child_ptr(&built.root, 4);
        connect_destroy_logged_signal(&mut built.root.children[3], 3);
        connect_destroy_logged_signal(&mut built.root.children[4], 4);

        assert!(built
            .__set_string_list_state_for_test("labels", vec!["X".into(), "Y".into(), "Q".into(),]));
        assert_eq!(
            text_children(&built.root),
            ["before", "maybe", "X", "Y", "Q", "after"],
            "same-length reset must update retained item bindings in place"
        );
        assert_visual_order(&built.root, "after same-length reset");
        assert_eq!(child_ptr(&built.root, 0), before_ptr);
        assert_eq!(child_ptr(&built.root, 1), maybe_ptr);
        assert_eq!(child_ptr(&built.root, 2), a_ptr);
        assert_eq!(child_ptr(&built.root, 3), b_ptr);
        assert_eq!(child_ptr(&built.root, 4), c_ptr);
        assert_eq!(
            DESTROY_COUNT.load(Ordering::SeqCst),
            0,
            "same-length reset must not remove retained subtrees"
        );

        assert!(built.__set_string_list_state_for_test("labels", vec!["Z".into()]));
        assert_eq!(
            text_children(&built.root),
            ["before", "maybe", "Z", "after"],
            "tail removal must remove generated children before following static sibling"
        );
        assert_visual_order(&built.root, "after tail removal");
        assert_eq!(child_ptr(&built.root, 0), before_ptr);
        assert_eq!(child_ptr(&built.root, 1), maybe_ptr);
        assert_eq!(child_ptr(&built.root, 2), a_ptr);
        assert_eq!(
            DESTROY_COUNT.load(Ordering::SeqCst),
            1,
            "tail removal must dispose registry entries in the removed subtree"
        );
        assert_eq!(
            destroy_log()
                .lock()
                .expect("destroy log poisoned")
                .as_slice(),
            &[4, 3],
            "tail removal must dispose generated subtrees tail-first"
        );

        assert!(built.__set_bool_state_for_test("show", false));
        assert_eq!(text_children(&built.root), ["before", "Z", "after"]);
        assert_visual_order(&built.root, "after conditional removal through splice seam");

        assert!(built.__set_bool_state_for_test("show", true));
        assert_eq!(
            text_children(&built.root),
            ["before", "maybe", "Z", "after"]
        );
        assert_visual_order(
            &built.root,
            "after conditional reinsert through splice seam",
        );
    });
}

#[test]
fn handler_collection_append_is_observable_before_click_returns() {
    run_on_owning_runtime_thread_or_skip("iteration handler collection append", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let src = r#"component IterHandlerAppend inherits Window {
    state labels: string[] = ["A"]
    VStack {
        Button {
            text: "Add"
            clicked => { labels = labels.append("B"); }
        }
        for label in labels {
            Text { text: label }
        }
    }
}"#;
        let mut built = build(src);
        built
            .root
            .run_layout_as_window_root(300.0, 200.0)
            .expect("run_layout_as_window_root failed");

        assert_eq!(text_children_from(&built.root, 1), ["A"]);
        let button_visual = visual_of(&built.root.children[0]);
        let (button_x, button_y) = visual_offset(&button_visual);
        let (button_w, button_h) = visual_size(&button_visual);
        assert!(
            button_w > 0.0 && button_h > 0.0,
            "Button must have a non-zero hit-test rect; got ({button_w}, {button_h})"
        );
        built.root.hit_test_click(
            (button_x + button_w / 2.0) as i32,
            (button_y + button_h / 2.0) as i32,
        );
        assert_eq!(
            text_children_from(&built.root, 1),
            ["A", "B"],
            "handler append must drain the for-loop effect before hit_test_click returns"
        );
    });
}

#[test]
fn empty_drop_last_is_equal_value_and_does_not_dirty_range() {
    run_on_owning_runtime_thread_or_skip("iteration empty drop-last no dirty", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let src = r#"component IterEmptyDrop inherits Window {
    state labels: string[] = []
    VStack {
        Text { text: "before" }
        for label in labels {
            Text { text: label }
        }
        Text { text: "after" }
    }
}"#;
        let built = build(src);
        let before_ptr = child_ptr(&built.root, 0);
        let after_ptr = child_ptr(&built.root, 1);

        assert!(
            !built.__set_string_list_state_for_test("labels", Vec::new()),
            "equal empty collection write must return false"
        );
        assert_eq!(text_children(&built.root), ["before", "after"]);
        assert_eq!(child_ptr(&built.root, 0), before_ptr);
        assert_eq!(child_ptr(&built.root, 1), after_ptr);
        assert_visual_order(&built.root, "after empty drop-last equivalent");
    });
}

#[test]
fn reactive_for_zstack_tail_append_uses_child_carried_placement() {
    run_on_owning_runtime_thread_or_skip("iteration reactive ZStack range mutation", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let src = r#"component IterZStackMutation inherits Window {
    state labels: string[] = ["A"]
    ZStack {
        Text { text: "base" }
        for label in labels {
            Text {
                h-align: end
                v-align: start
                text: label
            }
        }
    }
}"#;
        let built = build(src);
        assert_eq!(text_children(&built.root), ["base", "A"]);
        assert!(built.__set_string_list_state_for_test("labels", vec!["A".into(), "B".into()]));
        assert_eq!(text_children(&built.root), ["base", "A", "B"]);
        assert_visual_order(&built.root, "after ZStack append");

        let mut root = built.root;
        root.run_layout_as_window_root(300.0, 200.0)
            .expect("run_layout_as_window_root failed");
        for (index, child) in root.children.iter().enumerate().skip(1) {
            let visual = visual_of(child);
            let (x, y) = visual_offset(&visual);
            let (w, h) = visual_size(&visual);
            assert!(
                w > 0.0 && h > 0.0,
                "generated ZStack child {index} must have non-zero size; got ({w}, {h})"
            );
            assert_close(y, 0.0, "generated ZStack v-align=start offset");
            assert_close(x + w, 300.0, "generated ZStack h-align=end right edge");
        }
    });
}

#[test]
fn large_breadth_tail_append_converges_beyond_mutation_cap() {
    run_on_owning_runtime_thread_or_skip("iteration breadth cap convergence", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let src = r#"component IterLargeAppend inherits Window {
    state labels: string[] = []
    WrapPanel {
        for label in labels {
            Text { text: label }
        }
    }
}"#;
        let built = build(src);
        assert_eq!(ffi::__runtime_health_for_test(), "Healthy");
        assert!(
            !ffi::__reactive_divergence_diagnostics_present_for_test(),
            "fresh runtime must start without divergence diagnostics"
        );

        let gallery_labels: Vec<String> = (0..8).map(|i| format!("G{i:02}")).collect();
        assert!(built.__set_string_list_state_for_test("labels", gallery_labels));
        assert_eq!(
            built.root.children.len(),
            8,
            "gallery-scale collection write must materialise every generated child"
        );
        assert_eq!(
            text_children(&built.root).last().map(String::as_str),
            Some("G07")
        );
        assert_eq!(ffi::__runtime_health_for_test(), "Healthy");
        assert!(
            !ffi::__reactive_divergence_diagnostics_present_for_test(),
            "gallery-scale write must not trip divergence diagnostics"
        );

        let large_labels: Vec<String> = (0..64).map(|i| format!("S{i:02}")).collect();
        assert!(built.__set_string_list_state_for_test("labels", large_labels));
        assert_eq!(
            built.root.children.len(),
            64,
            "one breadth-heavy collection write must materialise every generated child"
        );
        assert_eq!(
            text_children(&built.root).last().map(String::as_str),
            Some("S63")
        );
        assert_eq!(ffi::__runtime_health_for_test(), "Healthy");
        assert!(
            !ffi::__reactive_divergence_diagnostics_present_for_test(),
            "breadth-heavy write must not trip divergence diagnostics"
        );
    });
}

#[test]
fn staged_for_insert_build_failure_leaves_tree_unchanged() {
    run_on_owning_runtime_thread_or_skip("iteration staged failure rollback", move || {
        let _guard = test_lock().lock().expect("test lock poisoned");

        let component = IrComponent {
            name: "IterStageFailure".into(),
            base: "Window".into(),
            host_props: Vec::new(),
            host_bindings: Vec::new(),
            states: vec![IrState {
                name: "labels".into(),
                ty: IrStateType::Collection(IrType::Str),
                default: IrLiteral::List(Vec::new()),
            }],
            root: IrNode {
                widget_type: "WrapPanel".into(),
                props: Vec::new(),
                bindings: Vec::new(),
                handlers: Vec::new(),
                children: vec![IrMember::ControlFlow(ControlFlowNode::For {
                    binder: "label".into(),
                    index_binder: None,
                    collection: HandlerExpr::ListPropRead {
                        path: "labels".into(),
                        elem: IrType::Str,
                    },
                    body: vec![IrMember::Widget(IrNode {
                        widget_type: "NotAWidget".into(),
                        props: Vec::new(),
                        bindings: Vec::new(),
                        handlers: Vec::new(),
                        children: Vec::new(),
                        kind_payload: None,
                    })],
                })],
                kind_payload: None,
            },
        };

        let built = build_memory_ir(&component);
        assert_eq!(built.root.children.len(), 0);
        assert!(built.__set_string_list_state_for_test("labels", vec!["A".into()]));
        assert_eq!(
            built.root.children.len(),
            0,
            "staging failure must leave the materialised tree unchanged"
        );
        assert_visual_order(&built.root, "after staged build failure");
    });
}
