//! Cross-crate round-trip test for the IR loader (DD-M2-P6-006).
//!
//! Covers the seam between `wasamoc::emit` (compiler-side serialization,
//! DD-M2-P6-002 grammar) and `wasamo_runtime::ir_loader::parse_ir`
//! (runtime-side deserialization). Asserts that an IR text emitted by the
//! compiler from the canonical `examples/counter/counter.ui` parses back
//! into a structurally-equal `IrComponent`.
//!
//! The build/widget-tree side requires a live Compositor and is exercised
//! by the Phase 5-pattern GUI checkpoint on the
//! `exp/m2-p6-ir-loader-checkpoint` branch.

use std::path::PathBuf;
use wasamo_runtime::ir_loader::parse_ir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn counter_ui() -> PathBuf {
    workspace_root().join("examples").join("counter").join("counter.ui")
}

fn build_counter_ir() -> wasamo_ir::IrComponent {
    use wasamoc::{check, lexer, lower, parser};
    let path = counter_ui();
    let src = std::fs::read_to_string(&path).expect("counter.ui not found");
    let path_str = path.to_string_lossy().to_string();
    let tokens = lexer::tokenize(&src, &path_str).expect("lex failed");
    let ast = parser::parse(&tokens, &path_str).expect("parse failed");
    let result = check::check(&ast, &path_str);
    assert!(!result.has_errors(), "check errors: {:?}", result.diagnostics);
    lower::lower(&ast, &result.namespace)
}

fn emit_counter_ir() -> String {
    use wasamoc::emit;
    emit::emit(&build_counter_ir())
}

#[test]
fn counter_ui_emit_then_parse_yields_equal_ir() {
    let original = build_counter_ir();
    let text = emit_counter_ir();
    let parsed = parse_ir(&text).expect("parse_ir failed");
    assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");
}

#[test]
fn parsed_counter_has_state_count_i32_zero() {
    let parsed = parse_ir(&emit_counter_ir()).expect("parse_ir failed");
    assert_eq!(parsed.states.len(), 1);
    let s = &parsed.states[0];
    assert_eq!(s.name, "count");
    assert_eq!(s.ty, wasamo_ir::IrType::I32);
    assert_eq!(s.default, wasamo_ir::IrLiteral::Int(0));
}

#[test]
fn parsed_counter_has_text_binding_and_clicked_handler() {
    let parsed = parse_ir(&emit_counter_ir()).expect("parse_ir failed");

    // Walk the tree to find the Text node's binding and the Button's
    // clicked handler. The tree shape is VStack { Text {...}, Button {...} }
    // (component-level props live on the VStack root after lowering).
    let vstack = &parsed.root;
    assert_eq!(vstack.widget_type, "VStack");

    let text_node = vstack
        .children
        .iter()
        .find(|c| c.widget_type == "Text")
        .expect("counter root must contain a Text child");
    assert!(
        text_node.bindings.iter().any(|b| b.prop_name == "text"),
        "Text node must declare a `bind text = ...`"
    );

    let button_node = vstack
        .children
        .iter()
        .find(|c| c.widget_type == "Button")
        .expect("counter root must contain a Button child");
    let clicked = button_node
        .handlers
        .iter()
        .find(|h| h.signal == "clicked")
        .expect("Button must declare an `on clicked` handler");

    use wasamo_ir::{CompoundOp, HandlerExpr};
    assert_eq!(
        clicked.expr,
        HandlerExpr::CompoundAssign {
            op: CompoundOp::Add,
            lhs: "count".into(),
            rhs: Box::new(HandlerExpr::IntLit(1)),
        }
    );
}
