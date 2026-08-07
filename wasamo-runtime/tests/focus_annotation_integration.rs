//! Mock-free Windows-only runtime evidence for M4-Phase 2 T6.
//!
//! Drives the authored `focus-group` / `modal-scope` annotations
//! (dsl_spec §4.19, DD-M4-P2-005 A1) through the production compiler ->
//! textual IR -> runtime loader -> live Widget tree path, and asserts
//! that the authored annotation reaches the loaded node as its focus
//! role (`WidgetNode::__focus_role_for_test`).
//!
//! This is the one thing only a built tree can show, and it is the
//! whole of this test's job: group traversal, per-group memory, modal
//! entry/exit/restore, and the Escape-to-`dismiss` conversion carry no
//! evidence here — they are M4-Phase 2 T7's.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<focus-annotation-integration>";
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

// Root VStack with four direct children, each exercising one branch of
// `WidgetNode::focus_role`'s widened container arm:
//   [0] HStack   — focus-group: true               -> "group"
//   [1] Box      — modal-scope: true                -> "modal-scope"
//   [2] VStack   — no annotation (control)          -> "container"
//   [3] ZStack   — both annotations at once         -> "modal-scope" (precedence)
// Every annotated container also carries a Button child, to show the
// Button-family arm is untouched by an ancestor's annotation.
const SRC: &str = r#"component FocusAnnotationIntegration inherits Window {
    VStack {
        HStack {
            focus-group: true
            Button { text: "A" }
            Button { text: "B" }
        }
        Box {
            modal-scope: true
            Button { text: "C" }
        }
        VStack {
            Button { text: "D" }
        }
        ZStack {
            focus-group: true
            modal-scope: true
            Button { text: "E" }
        }
    }
}"#;

#[test]
fn authored_focus_annotation_reaches_the_loaded_node_as_its_focus_role() {
    // Marshalled onto the runtime-owning thread (Observation 5 step 1): the
    // Compositor is created and used on one thread. See tests/common/mod.rs.
    run_on_owning_runtime_thread_or_skip("focus annotation integration test", move || {
        let ir = lower_ui_to_ir(SRC);
        let component = parse_ir(&ir).expect("parse_ir failed");
        let compositor = wasamo_runtime::get_compositor();
        let text_renderer = wasamo_runtime::get_text_renderer();
        let built = build_widget_tree(&component, compositor, text_renderer)
            .expect("build_widget_tree failed");

        // ── Before-state: establish the tree shape this test is about to
        // index into, rather than inheriting it from the source above. ──
        assert_eq!(
            built.root.__kind_name_for_test(),
            "VStack",
            "root must be the outer VStack"
        );
        assert_eq!(
            built.root.children.len(),
            4,
            "root VStack must have exactly its four declared children"
        );
        assert_eq!(built.root.children[0].__kind_name_for_test(), "HStack");
        assert_eq!(built.root.children[1].__kind_name_for_test(), "Box");
        assert_eq!(built.root.children[2].__kind_name_for_test(), "VStack");
        assert_eq!(built.root.children[3].__kind_name_for_test(), "ZStack");
        assert_eq!(
            built.root.children[0].children.len(),
            2,
            "focus-group HStack must have its two declared Button children"
        );
        assert_eq!(
            built.root.children[1].children.len(),
            1,
            "modal-scope Box must have its one declared Button child"
        );
        assert_eq!(
            built.root.children[2].children.len(),
            1,
            "unannotated VStack must have its one declared Button child"
        );
        assert_eq!(
            built.root.children[3].children.len(),
            1,
            "both-annotated ZStack must have its one declared Button child"
        );
        assert_eq!(
            built.root.children[0].children[0].__kind_name_for_test(),
            "Button",
            "focus-group HStack's first child must be the Button under test"
        );

        // ── The assertions this test exists for ──────────────────────
        assert_eq!(
            built.root.children[0].__focus_role_for_test(),
            "group",
            "focus-group: true must reach the loaded node as FocusRole::Group"
        );
        assert_eq!(
            built.root.children[1].__focus_role_for_test(),
            "modal-scope",
            "modal-scope: true must reach the loaded node as FocusRole::ModalScope"
        );
        assert_eq!(
            built.root.children[2].__focus_role_for_test(),
            "container",
            "an unannotated sibling container must read FocusRole::Container"
        );
        assert_eq!(
            built.root.children[0].children[0].__focus_role_for_test(),
            "stop",
            "a Button inside an annotated container must still read FocusRole::Stop \
                 (the Button-family arm of focus_role is unchanged)"
        );
        assert_eq!(
            built.root.children[3].__focus_role_for_test(),
            "modal-scope",
            "a container carrying both annotations at once must read \
                 FocusRole::ModalScope — modal-scope wins over focus-group \
                 (DD-M4-P2-005 forward-compat note: the both-at-once case is \
                 expressible under A1 and untested in M4; confinement is the \
                 stronger property)"
        );
    });
}
