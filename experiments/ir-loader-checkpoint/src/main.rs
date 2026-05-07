//! DD-M2-P6-006 GUI checkpoint harness.
//!
//! Counterpart to the Phase 5 reactive-spike (`exp/m2-p5-reactive-checkpoint`,
//! commit `fdc1545`): same pattern, same widget tree shape, but driven by
//! `wasamoc`-produced IR text fed through `wasamo_runtime::ir_loader` rather
//! than imperative Rust calls.
//!
//! Pass criteria: clicking "Increment" updates the visible label through the
//! reactive path, with no host-side `set_property` invocation. The per-frame
//! flow:
//!
//!   1. WM_LBUTTONUP → `WidgetNode::hit_test_click_inner`
//!   2. → inline_handlers fires the `(compound-assign += count 1)` HandlerExpr
//!      against the active `SignalRegistry`
//!   3. → `Signal::set` enqueues the dependent text-binding Effect
//!   4. → message-loop drain runs the binding, which writes the formatted
//!      `"Count: N"` back through `widget_write_property` →
//!      `WidgetNode::set_property(PROP_TEXT_CONTENT, ...)`
//!
//! This branch is the verification artifact for DD-M2-P6-006 only;
//! production wiring of the same path through the C ABI is DD-M2-P6-008.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Compile examples/counter/counter.ui to normative IR text.
    let ir_text = compile_counter_ui()?;
    println!("=== wasamoc-produced IR ===\n{ir_text}");

    // 2. Initialise wasamo runtime (compositor, dispatcher queue, etc.).
    wasamo_runtime::init()?;

    // 3. Create the window.
    let window = wasamo_runtime::window_create("Counter (IR loader checkpoint)", 320, 240)?;

    // 4. Parse IR + build widget tree via the production loader.
    let comp = wasamo_runtime::ir_loader::parse_ir(&ir_text)
        .map_err(|e| format!("parse_ir: {e}"))?;
    let compositor = wasamo_runtime::get_compositor();
    let renderer = wasamo_runtime::get_text_renderer();
    let built = wasamo_runtime::ir_loader::build_widget_tree(&comp, compositor, renderer)
        .map_err(|e| format!("build_widget_tree: {e}"))?;

    // 5. Attach root to window.
    wasamo_runtime::window_add_widget(&window, &built.root)?;

    // 6. Show window. The widget tree must outlive the message loop, but
    //    Box<WidgetNode> would normally drop at end of `main`'s scope — leak
    //    it explicitly for the harness lifetime (process exits when window
    //    closes, so the OS reclaims everything). Mirrors how the Phase 5
    //    spike harness handled the same constraint.
    let _root_leak = Box::leak(built.root);
    wasamo_runtime::window_show(&window);

    // 7. Run the message loop. Returns when the window is closed.
    wasamo_runtime::run();
    Ok(())
}

fn compile_counter_ui() -> Result<String, Box<dyn std::error::Error>> {
    use wasamoc::{check, emit, lexer, lower, parser};
    let path = counter_ui_path()?;
    let src = std::fs::read_to_string(&path)?;
    let path_str = path.to_string_lossy().to_string();
    let tokens = lexer::tokenize(&src, &path_str)
        .map_err(|e| format!("lex: {e:?}"))?;
    let ast = parser::parse(&tokens, &path_str)
        .map_err(|e| format!("parse: {e:?}"))?;
    let result = check::check(&ast, &path_str);
    if result.has_errors() {
        return Err(format!("check errors: {:?}", result.diagnostics).into());
    }
    let comp = lower::lower(&ast, &result.namespace);
    Ok(emit::emit(&comp))
}

fn counter_ui_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // CARGO_MANIFEST_DIR = .../experiments/ir-loader-checkpoint
    // Climb two levels to the workspace root, then descend into examples/counter.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest
        .parent()
        .and_then(|p| p.parent())
        .ok_or("cannot resolve workspace root")?;
    Ok(workspace.join("examples").join("counter").join("counter.ui"))
}
