use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn counter_ui() -> PathBuf {
    workspace_root().join("examples").join("counter").join("counter.ui")
}

/// Run `wasamoc build` on counter.ui and return the emitted IR text.
fn build_counter() -> String {
    use wasamoc::{check, emit, lower, lexer, parser};

    let path = counter_ui();
    let src = std::fs::read_to_string(&path).expect("counter.ui not found");
    let path_str = path.to_string_lossy().to_string();

    let tokens = lexer::tokenize(&src, &path_str).expect("lex failed");
    let ast = parser::parse(&tokens, &path_str).expect("parse failed");
    let result = check::check(&ast, &path_str);
    assert!(!result.has_errors(), "check errors: {:?}", result.diagnostics);
    let comp = lower::lower(&ast, &result.namespace);
    emit::emit(&comp)
}

#[test]
fn counter_ui_produces_valid_ir_header() {
    let ir = build_counter();
    assert!(ir.starts_with(";wasamo-ir v0\n"), "IR must start with header line, got:\n{}", ir);
}

#[test]
fn counter_ui_contains_state_node() {
    let ir = build_counter();
    assert!(ir.contains("state count: i32 = 0"), "missing state declaration, got:\n{}", ir);
}

#[test]
fn counter_ui_contains_reactive_text_binding() {
    let ir = build_counter();
    assert!(
        ir.contains("bind text =") && ir.contains("prop-read count"),
        "missing reactive text binding, got:\n{}", ir
    );
}

#[test]
fn counter_ui_contains_clicked_handler() {
    let ir = build_counter();
    assert!(ir.contains("on clicked {"), "missing clicked handler, got:\n{}", ir);
    assert!(
        ir.contains("compound-assign += count 1"),
        "missing compound-assign in handler, got:\n{}", ir
    );
}

#[test]
fn counter_ui_component_structure() {
    let ir = build_counter();
    assert!(ir.contains("component Counter inherits Window {"), "got:\n{}", ir);
    assert!(ir.contains("node Window {") || ir.contains("node VStack {"), "got:\n{}", ir);
}
