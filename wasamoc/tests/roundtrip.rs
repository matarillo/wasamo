use std::path::PathBuf;

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

fn bool_demo_ui() -> PathBuf {
    workspace_root()
        .join("examples")
        .join("bool-demo")
        .join("bool-demo.ui")
}

/// Run `wasamoc build` on a .ui fixture and return the emitted IR text.
fn build_ui(path: PathBuf) -> String {
    let src = std::fs::read_to_string(&path).expect(".ui fixture not found");
    let path_str = path.to_string_lossy().to_string();
    build_src(&src, &path_str)
}

fn build_src(src: &str, path: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let tokens = lexer::tokenize(src, path).expect("lex failed");
    let ast = parser::parse(&tokens, path).expect("parse failed");
    let result = check::check(&ast, path);
    assert!(
        !result.has_errors(),
        "check errors: {:?}",
        result.diagnostics
    );
    let comp = lower::lower(&ast, &result.namespace);
    emit::emit(&comp)
}

fn build_counter() -> String {
    build_ui(counter_ui())
}

fn build_bool_demo() -> String {
    build_ui(bool_demo_ui())
}

#[test]
fn counter_ui_produces_valid_ir_header() {
    let ir = build_counter();
    assert!(
        ir.starts_with(";wasamo-ir v0\n"),
        "IR must start with header line, got:\n{}",
        ir
    );
}

#[test]
fn counter_ui_contains_state_node() {
    let ir = build_counter();
    assert!(
        ir.contains("state count: i32 = 0"),
        "missing state declaration, got:\n{}",
        ir
    );
}

#[test]
fn counter_ui_contains_reactive_text_binding() {
    let ir = build_counter();
    assert!(
        ir.contains("bind text =") && ir.contains("prop-read count"),
        "missing reactive text binding, got:\n{}",
        ir
    );
}

#[test]
fn counter_ui_contains_clicked_handler() {
    let ir = build_counter();
    assert!(
        ir.contains("on clicked {"),
        "missing clicked handler, got:\n{}",
        ir
    );
    assert!(
        ir.contains("compound-assign += count 1"),
        "missing compound-assign in handler, got:\n{}",
        ir
    );
}

#[test]
fn counter_ui_component_structure() {
    let ir = build_counter();
    assert!(
        ir.contains("component Counter inherits Window {"),
        "got:\n{}",
        ir
    );
    assert!(
        ir.contains("node Window {") || ir.contains("node VStack {"),
        "got:\n{}",
        ir
    );
}

#[test]
fn bool_demo_ui_contains_bool_binding_and_handler() {
    let ir = build_bool_demo();
    assert!(
        ir.contains("component BoolDemo inherits Window {"),
        "got:\n{}",
        ir
    );
    assert!(
        ir.contains("state ready: bool = true"),
        "missing bool state declaration, got:\n{}",
        ir
    );
    assert!(
        ir.contains("bind enabled =") && ir.contains("bool-prop-read ready"),
        "missing Button.enabled bool binding, got:\n{}",
        ir
    );
    assert!(
        ir.contains("on clicked {") && ir.contains("(assign ready false)"),
        "missing clicked handler bool assignment, got:\n{}",
        ir
    );
}

#[test]
fn key_down_handler_surface_emits_parenthesised_argument() {
    // M4-Phase 2 T8 (DD-M4-P2-005): `.ui` -> IR text for a `key-down`
    // handler round-trips its string argument into the parenthesised
    // `on key-down("<key>") { ... }` IR text form.
    let ir = build_src(
        r#"component KeyDemo inherits Window {
            state selected_index: i32 = 0
            Box {
                modal-scope: true
                key-down("ArrowLeft")  => { root.selected_index -= 1; }
                key-down("ArrowRight") => { root.selected_index += 1; }
            }
        }"#,
        "<key-down-roundtrip>",
    );
    assert!(
        ir.contains(r#"on key-down("ArrowLeft") {"#),
        "missing key-down(\"ArrowLeft\") handler, got:\n{}",
        ir
    );
    assert!(
        ir.contains(r#"on key-down("ArrowRight") {"#),
        "missing key-down(\"ArrowRight\") handler, got:\n{}",
        ir
    );
    assert!(
        ir.contains("(compound-assign -= selected_index 1)"),
        "missing compound-assign in ArrowLeft handler, got:\n{}",
        ir
    );
    assert!(
        ir.contains("(compound-assign += selected_index 1)"),
        "missing compound-assign in ArrowRight handler, got:\n{}",
        ir
    );
}

#[test]
fn togglebutton_surface_emits_literal_and_binding_forms() {
    let ir = build_src(
        r#"component ToggleDemo inherits Window {
            state selected: bool = true
            HStack {
                ToggleButton {
                    text: "Literal"
                    checked: false
                }
                ToggleButton {
                    text: "Binding"
                    style: accent
                    enabled: true
                    checked: selected
                    clicked => { selected = false; }
                }
            }
        }"#,
        "<togglebutton-roundtrip>",
    );
    assert!(
        ir.contains("node ToggleButton {"),
        "missing ToggleButton node, got:\n{}",
        ir
    );
    assert!(
        ir.contains("prop checked = false"),
        "missing literal checked prop, got:\n{}",
        ir
    );
    assert!(
        ir.contains("bind checked = (bool-prop-read selected)"),
        "missing checked bool binding, got:\n{}",
        ir
    );
    assert!(
        ir.contains("prop style = accent") && ir.contains("prop enabled = true"),
        "missing carried Button attrs, got:\n{}",
        ir
    );
}
