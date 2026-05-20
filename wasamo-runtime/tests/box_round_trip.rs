//! IR text round-trip evidence for the Box widget (M3-Phase 2 ADR
//! §Phase 2 verification closure item 2, T10).
//!
//! This file makes the emit ↔ load boundary executable for the canonical
//! Phase 2 fixture
//! `Box { aspect: 16:9; fill: #00000080; Text { text: "Photo 12" } }`.
//! Two halves of the round-trip:
//!
//! 1. **Emit → parse_ir (pure logic, cross-crate).** `wasamoc::emit::emit`
//!    is fed into `wasamo_runtime::ir_loader::parse_ir`; the resulting
//!    `IrComponent` is asserted to carry `IrLiteral::Ratio { num: 16, den: 9 }`
//!    and `IrLiteral::Color(0x80_00_00_00)` per DD-M3-P2-002 / DD-M3-P2-003.
//!    The in-crate fixtures `wasamoc::emit::box_phase2_ir_text_emit_fixture`
//!    (T5) and `wasamo_runtime::ir_loader::box_phase2_load_side_fixture` (T7)
//!    each assert one direction; T10 joins them so the same fixture string
//!    survives both directions of the IR text grammar end to end.
//!
//! 2. **build_node materialisation (Windows-only, CI-gated).** The parsed
//!    `IrComponent` is fed to `ir_loader::build_widget_tree`, which
//!    transitively invokes `build_node`. The resulting `WidgetData::Box`
//!    is read back through `WidgetNode::__box_state_for_test` and asserted
//!    to carry the Box-internal domain types — `IrLiteral::*` do not
//!    survive into runtime state, per DD-M3-P2-002 / DD-M3-P2-003 variant
//!    strategy Option A. This is the "loaded runtime state" gate the ADR
//!    requires.
//!
//! T11 (Windows-runtime layout integration test) reuses the same skip-guard
//! pattern (`wasamo_init` failing with `0x80070005` ⇒ skip locally, fail on
//! GitHub Actions per CLAUDE.md "Testing rules") and the same
//! `__box_state_for_test` accessor for the `fill`-side observation.

use std::ffi::CStr;

use wasamo_ir::{IrComponent, IrLiteral};
use wasamo_runtime::ffi;
use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir, IrLoadError};

const PHASE2_FIXTURE: &str =
    r#"component C inherits W { Box { aspect: 16:9 fill: #00000080 Text { text: "Photo 12" } } }"#;

fn lower_and_emit(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<box-round-trip>";
    let tokens = lexer::tokenize(src, path).expect("lex failed");
    let ast = parser::parse(&tokens, path).expect("parse failed");
    let checked = check::check(&ast, path);
    assert!(
        !checked.has_errors(),
        "wasamoc check errors: {:?}",
        checked.diagnostics
    );
    emit::emit(&lower::lower(&ast, &checked.namespace))
}

fn parse_phase2_fixture() -> IrComponent {
    let ir_text = lower_and_emit(PHASE2_FIXTURE);
    parse_ir(&ir_text).expect("parse_ir failed")
}

/// Cross-crate join for the emit-side and parse-level halves of ADR §Phase 2
/// verification closure item 2. The two in-crate fixtures
/// (`wasamoc::emit::box_phase2_ir_text_emit_fixture` and
/// `wasamo_runtime::ir_loader::box_phase2_load_side_fixture`) each assert one
/// direction with hand-written reference strings; here we feed the actual
/// `wasamoc::emit::emit` output into `parse_ir`, closing the loop.
#[test]
fn box_phase2_emit_parses_back_to_ir_literal_variants() {
    let comp = parse_phase2_fixture();

    assert_eq!(comp.root.widget_type, "Box");

    let aspect = comp
        .root
        .props
        .iter()
        .find(|p| p.name == "aspect")
        .expect("aspect prop survives emit → parse_ir");
    let fill = comp
        .root
        .props
        .iter()
        .find(|p| p.name == "fill")
        .expect("fill prop survives emit → parse_ir");
    assert_eq!(aspect.value, IrLiteral::Ratio { num: 16, den: 9 });
    assert_eq!(fill.value, IrLiteral::Color(0x80_00_00_00));

    assert_eq!(comp.root.children.len(), 1);
    assert_eq!(comp.root.children[0].widget_type, "Text");
}

/// Defense-in-depth: DD-M3-P2-001 requires `ir_loader` to reject a Box with
/// more than one child, even for IR text not produced by `wasamoc` (which
/// the `check` pass would have rejected earlier). The in-crate
/// `malformed_box_with_two_children` (T7) exercises the same gate at the
/// `parse_ir` level; we re-state it here so T10's checklist item 4 has an
/// observable test owned by the round-trip file.
#[test]
fn box_phase2_two_children_rejected_at_parse_ir() {
    let malformed = ";wasamo-ir v0\n\
                     component C inherits W {\n\
                         node Box {\n\
                             node Text {}\n\
                             node Text {}\n\
                         }\n\
                     }";
    let err = parse_ir(malformed).expect_err("two-child Box must reject");
    match err {
        IrLoadError::Validate(ref m) => {
            assert!(
                m.contains("Box") && m.contains("at most one child"),
                "unexpected Validate message: {m}"
            );
        }
        other => panic!("expected IrLoadError::Validate, got {other:?}"),
    }
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

/// Build-node materialisation half of ADR §Phase 2 verification closure
/// item 2. Drives the full emit → parse_ir → build_widget_tree chain and
/// asserts the runtime `WidgetData::Box` carries Box-internal domain types
/// rather than the IR-level `IrLiteral::Ratio` / `IrLiteral::Color`.
/// `build_widget_tree` needs a live `Compositor`, so this test follows the
/// CLAUDE.md "Testing rules" Windows-only mock-free integration pattern:
/// fail (not skip) on GitHub Actions if the runtime Compositor is missing,
/// skip with a printed reason on local dev boxes without WinRT capability.
#[cfg(windows)]
#[test]
fn box_phase2_build_node_materialises_box_internal_state() {
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
                "box round-trip materialisation test cannot skip on GitHub Actions: \
                 runtime compositor unavailable ({msg:?})"
            );
            eprintln!(
                "skipping box round-trip materialisation test: runtime compositor unavailable ({msg:?})"
            );
            return;
        }
    }
    assert_eq!(
        status,
        ffi::WASAMO_OK,
        "wasamo_init failed: {:?}",
        last_error()
    );

    let component = parse_phase2_fixture();
    let compositor = wasamo_runtime::get_compositor();
    let renderer = wasamo_runtime::get_text_renderer();
    let built =
        build_widget_tree(&component, compositor, renderer).expect("build_widget_tree failed");

    let state = built
        .root
        .__box_state_for_test()
        .expect("built root must be a Box");
    assert_eq!(
        state,
        (Some((16, 9)), Some(0x80_00_00_00)),
        "WidgetData::Box must carry the Box-internal Ratio / Color materialised \
         from IrLiteral::Ratio / IrLiteral::Color (DD-M3-P2-002 / DD-M3-P2-003)"
    );

    assert_eq!(
        built.root.children.len(),
        1,
        "Box must have built its single Text child"
    );
}
