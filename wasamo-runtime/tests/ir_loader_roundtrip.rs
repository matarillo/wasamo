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
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn counter_ui() -> PathBuf {
    workspace_root()
        .join("examples")
        .join("counter")
        .join("counter.ui")
}

fn build_counter_ir() -> wasamo_ir::IrComponent {
    use wasamoc::{check, lexer, lower, parser};
    let path = counter_ui();
    let src = std::fs::read_to_string(&path).expect("counter.ui not found");
    let path_str = path.to_string_lossy().to_string();
    let tokens = lexer::tokenize(&src, &path_str).expect("lex failed");
    let ast = parser::parse(&tokens, &path_str).expect("parse failed");
    let result = check::check(&ast, &path_str);
    assert!(
        !result.has_errors(),
        "check errors: {:?}",
        result.diagnostics
    );
    lower::lower(&ast, &result.namespace)
}

fn emit_counter_ir() -> String {
    use wasamoc::emit;
    emit::emit(&build_counter_ir())
}

fn gallery_ui() -> PathBuf {
    workspace_root()
        .join("examples")
        .join("gallery")
        .join("gallery.ui")
}

fn build_gallery_ir() -> wasamo_ir::IrComponent {
    use wasamoc::{check, lexer, lower, parser};
    let path = gallery_ui();
    let src = std::fs::read_to_string(&path).expect("gallery.ui not found");
    let path_str = path.to_string_lossy().to_string();
    let tokens = lexer::tokenize(&src, &path_str).expect("lex failed");
    let ast = parser::parse(&tokens, &path_str).expect("parse failed");
    let result = check::check(&ast, &path_str);
    assert!(
        !result.has_errors(),
        "check errors: {:?}",
        result.diagnostics
    );
    lower::lower(&ast, &result.namespace)
}

fn build_string_binding_ir() -> wasamo_ir::IrComponent {
    use wasamoc::{check, lexer, lower, parser};
    let src = r#"component StringBinding inherits Window {
    state label: string = "Ready"
    VStack {
        Text { text: "State: \{root.label}" }
    }
}"#;
    let tokens = lexer::tokenize(src, "<string-binding>").expect("lex failed");
    let ast = parser::parse(&tokens, "<string-binding>").expect("parse failed");
    let result = check::check(&ast, "<string-binding>");
    assert!(
        !result.has_errors(),
        "check errors: {:?}",
        result.diagnostics
    );
    lower::lower(&ast, &result.namespace)
}

fn build_bool_binding_ir() -> wasamo_ir::IrComponent {
    use wasamoc::{check, lexer, lower, parser};
    let src = r#"component BoolBinding inherits Window {
    state ready: bool = false
    Button {
        enabled: ready
        clicked => { root.ready = true; }
    }
}"#;
    let tokens = lexer::tokenize(src, "<bool-binding>").expect("lex failed");
    let ast = parser::parse(&tokens, "<bool-binding>").expect("parse failed");
    let result = check::check(&ast, "<bool-binding>");
    assert!(
        !result.has_errors(),
        "check errors: {:?}",
        result.diagnostics
    );
    lower::lower(&ast, &result.namespace)
}

fn build_zstack_ir() -> wasamo_ir::IrComponent {
    use wasamoc::{check, lexer, lower, parser};
    let src = r#"component ZStackDemo inherits Window {
    ZStack {
        Box { fill: #336699cc }
        Text {
            slot.h-align: end
            slot.v-align: start
            text: "caption"
        }
        Box { fill: #993366cc }
    }
}"#;
    let tokens = lexer::tokenize(src, "<zstack>").expect("lex failed");
    let ast = parser::parse(&tokens, "<zstack>").expect("parse failed");
    let result = check::check(&ast, "<zstack>");
    assert!(
        !result.has_errors(),
        "check errors: {:?}",
        result.diagnostics
    );
    lower::lower(&ast, &result.namespace)
}

fn build_iteration_ir() -> wasamo_ir::IrComponent {
    use wasamoc::{check, lexer, lower, parser};
    let src = r#"component IterationDemo inherits Window {
    state labels: string[] = ["S01", "S02"]
    WrapPanel {
        for label, i in labels {
            Text { text: "\{label} #\{i}" }
        }
    }
}"#;
    let tokens = lexer::tokenize(src, "<iteration>").expect("lex failed");
    let ast = parser::parse(&tokens, "<iteration>").expect("parse failed");
    let result = check::check(&ast, "<iteration>");
    assert!(
        !result.has_errors(),
        "check errors: {:?}",
        result.diagnostics
    );
    lower::lower(&ast, &result.namespace)
}

#[test]
fn counter_ui_emit_then_parse_yields_equal_ir() {
    let original = build_counter_ir();
    let text = emit_counter_ir();
    let parsed = parse_ir(&text).expect("parse_ir failed");
    assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");
}

#[test]
fn gallery_ui_emits_and_validates_through_runtime_loader() {
    // DD-M3-P6-008 T7b re-verification: the host-surface migration rewrote the
    // runtime validator / loader that T7's GUI screenshot depended on. This
    // exercises the *real* gallery IR (root ZStack + lightbox conditional +
    // WrapPanel/ScrollView slices) through `parse_ir`, whose `validate()` pass
    // is exactly the surface T7b changed — proving the gallery still *loads*
    // at the validate level headlessly, not merely that it compiles. The Win32
    // GUI smoke (live Compositor render) remains T8's owner-visible step.
    let original = build_gallery_ir();
    let text = wasamoc::emit::emit(&original);
    let parsed =
        parse_ir(&text).expect("gallery IR must parse and validate through the runtime loader");
    assert_eq!(
        parsed, original,
        "gallery round-trip mismatch\nIR text:\n{text}"
    );
    // Host attributes live on the host surface, never squatted on the root.
    assert!(parsed.host_props.iter().any(|p| p.name == "title"));
    assert!(!parsed
        .root
        .props
        .iter()
        .any(|p| matches!(p.name.as_str(), "title" | "backdrop" | "theme")));
}

#[test]
fn parsed_counter_has_state_count_i32_zero() {
    let parsed = parse_ir(&emit_counter_ir()).expect("parse_ir failed");
    assert_eq!(parsed.states.len(), 1);
    let s = &parsed.states[0];
    assert_eq!(s.name, "count");
    assert_eq!(s.ty, wasamo_ir::IrStateType::Scalar(wasamo_ir::IrType::I32));
    assert_eq!(s.default, wasamo_ir::IrLiteral::Int(0));
}

#[test]
fn parsed_counter_has_text_binding_and_clicked_handler() {
    let parsed = parse_ir(&emit_counter_ir()).expect("parse_ir failed");

    // Walk the tree to find the Text node's binding and the Button's
    // clicked handler. The tree shape is VStack { Text {...}, Button {...} }
    // while component-level host props live on IrComponent.host_props.
    assert!(parsed.host_props.iter().any(|p| p.name == "title"));
    let vstack = &parsed.root;
    assert_eq!(vstack.widget_type, "VStack");
    assert!(!vstack.props.iter().any(|p| p.name == "title"));

    let text_node = vstack
        .widget_children()
        .find(|c| c.widget_type == "Text")
        .expect("counter root must contain a Text child");
    assert!(
        text_node.bindings.iter().any(|b| b.prop_name == "text"),
        "Text node must declare a `bind text = ...`"
    );

    let button_node = vstack
        .widget_children()
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

#[test]
fn bool_state_binding_emits_and_parses_bool_productions() {
    let original = build_bool_binding_ir();
    let text = wasamoc::emit::emit(&original);
    assert!(
        text.contains("state ready: bool = false"),
        "bool state must emit `bool` type and `false` literal\n{text}"
    );
    assert!(
        text.contains("(bool-prop-read ready)"),
        "bool state binding must emit BoolPropRead form\n{text}"
    );
    assert!(
        text.contains("(assign ready true)"),
        "bool literal in handler RHS must emit as `true`\n{text}"
    );
    let parsed = parse_ir(&text).expect("parse_ir failed");
    assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");

    // T5 acceptance: round-trip reconstructs the bool state declaration
    // and `BoolPropRead { path: "ready" }`.
    assert_eq!(parsed.states.len(), 1);
    let state = &parsed.states[0];
    assert_eq!(state.name, "ready");
    assert_eq!(
        state.ty,
        wasamo_ir::IrStateType::Scalar(wasamo_ir::IrType::Bool)
    );
    assert_eq!(state.default, wasamo_ir::IrLiteral::Bool(false));

    let button = &parsed.root;
    assert_eq!(button.widget_type, "Button");
    let binding = button
        .bindings
        .iter()
        .find(|b| b.prop_name == "enabled")
        .expect("Button must declare a `bind enabled = ...`");
    assert_eq!(
        binding.expr,
        wasamo_ir::HandlerExpr::BoolPropRead {
            path: "ready".into()
        }
    );

    // Handler RHS `BoolLit(true)` survives the round-trip too.
    let clicked = button
        .handlers
        .iter()
        .find(|h| h.signal == "clicked")
        .expect("Button must declare an `on clicked` handler");
    assert_eq!(
        clicked.expr,
        wasamo_ir::HandlerExpr::Assign {
            lhs: "ready".into(),
            rhs: Box::new(wasamo_ir::HandlerExpr::BoolLit(true)),
        }
    );
}

#[test]
fn zstack_emit_then_parse_preserves_direct_children_and_order() {
    let original = build_zstack_ir();
    let text = wasamoc::emit::emit(&original);
    assert!(text.contains("node ZStack {"), "got: {text}");
    assert!(
        !text.contains("kind_payload"),
        "ZStack must not emit a kind payload\n{text}"
    );
    let parsed = parse_ir(&text).expect("parse_ir failed");
    assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");

    assert_eq!(parsed.root.widget_type, "ZStack");
    assert_eq!(
        parsed.root.children.len(),
        3,
        "ZStack direct child count must survive emit -> parse"
    );
    let child_types: Vec<_> = parsed
        .root
        .widget_children()
        .map(|child| child.widget_type.as_str())
        .collect();
    assert_eq!(
        child_types,
        ["Box", "Text", "Box"],
        "ZStack document order must survive emit -> parse"
    );
}

#[test]
fn iteration_emit_then_parse_preserves_for_member_and_collection_state() {
    let original = build_iteration_ir();
    let text = wasamoc::emit::emit(&original);
    assert!(text.contains("state labels: string[] = [\"S01\", \"S02\"]"));
    assert!(text.contains("for label, i in labels"));
    assert!(text.contains("(item-read label)"));
    assert!(text.contains("(index-read i)"));

    let parsed = parse_ir(&text).expect("parse_ir failed");
    assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");
}

fn build_wrap_panel_ir() -> wasamo_ir::IrComponent {
    use wasamoc::{check, lexer, lower, parser};
    let src = r#"component WrapDemo inherits Window {
    WrapPanel {
        item-cross-size: 96
        item-spacing: 8
        line-spacing: 12
        Box { aspect: 1:1 }
        Box { aspect: 1:1 }
        Box { aspect: 1:1 }
    }
}"#;
    let tokens = lexer::tokenize(src, "<wrap-panel>").expect("lex failed");
    let ast = parser::parse(&tokens, "<wrap-panel>").expect("parse failed");
    let result = check::check(&ast, "<wrap-panel>");
    assert!(
        !result.has_errors(),
        "check errors: {:?}",
        result.diagnostics
    );
    lower::lower(&ast, &result.namespace)
}

#[test]
fn wrap_panel_emit_then_parse_yields_equal_ir() {
    // M3-Phase 3 T4 round-trip: WrapPanel + the three kebab-case attribute
    // props (`item-cross-size`, `item-spacing`, `line-spacing`) traverse
    // `wasamoc::emit::emit` and `wasamo_runtime::ir_loader::parse_ir`
    // without changing the `IrComponent` shape. Phase 3 introduces no new
    // emit/parse grammar; this test pins that the existing generic
    // `node IDENT { prop IDENT = <int> ... }` form already covers the
    // WrapPanel surface end-to-end.
    let original = build_wrap_panel_ir();
    let text = wasamoc::emit::emit(&original);
    assert!(text.contains("node WrapPanel {"), "got: {}", text);
    assert!(text.contains("prop item-cross-size = 96"), "got: {}", text);
    assert!(text.contains("prop item-spacing = 8"), "got: {}", text);
    assert!(text.contains("prop line-spacing = 12"), "got: {}", text);
    let parsed = parse_ir(&text).expect("parse_ir failed");
    assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");
}

#[test]
fn string_state_binding_emits_and_parses_str_prop_read() {
    let original = build_string_binding_ir();
    let text = wasamoc::emit::emit(&original);
    assert!(
        text.contains("(str-prop-read label)"),
        "String state interpolation must emit StrPropRead form\n{text}"
    );
    let parsed = parse_ir(&text).expect("parse_ir failed");
    assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");

    let text_node = parsed
        .root
        .widget_children()
        .find(|c| c.widget_type == "Text")
        .unwrap();
    let binding = text_node
        .bindings
        .iter()
        .find(|b| b.prop_name == "text")
        .unwrap();
    assert_eq!(
        binding.expr,
        wasamo_ir::HandlerExpr::Interpolation(vec![
            wasamo_ir::InterpolationPart::Literal("State: ".into()),
            wasamo_ir::InterpolationPart::Expr(wasamo_ir::HandlerExpr::StrPropRead {
                path: "label".into(),
            }),
        ])
    );
}
