//! Mock-free Windows-only layout integration evidence for M3-Phase 3 T8.
//!
//! These tests drive the production path end-to-end for the WrapPanel
//! layout primitive: `.ui` source is lowered by `wasamoc`, parsed by the
//! runtime IR loader, built into live `WidgetNode`s backed by a real
//! Compositor / TextRenderer, laid out, and then read back from the
//! resulting SpriteVisual tree.
//!
//! Two fixtures discharge ADR verification closure evidence item 4
//! (CI-gated Compositor pipeline) — see
//! `process/milestone-3/phase-3/decisions/preamble.md §Phase 3 verification
//! closure`:
//!
//! - **wrap_path_fixture_lays_out_multi_line_thumbnails** — primary
//!   positive control. A WrapPanel of six 1:1 Boxes with
//!   `item-cross-size: 88; item-spacing: 12; line-spacing: 12` inside a
//!   300×400 parent. Three children fit per line (88×3 + 12×2 = 288 ≤
//!   300); the fourth would exceed the bound (288 + 12 + 88 = 388), so
//!   the layout wraps to two lines of three. Asserts WrapPanel outer
//!   dimensions, per-child 88×88 size, and per-child line/column
//!   assignment.
//! - **oversized_child_fixture_paints_visible_overflow** — visible-
//!   overflow regulation. A WrapPanel with one `aspect: 4:1` Box and
//!   `item-cross-size: 50` inside a 100-wide parent. The child measures
//!   to (200, 50) — its main extent exceeds the parent main bound — and
//!   per DD-M3-P3-005 oversized-line Option A the WrapPanel does not
//!   grow to accommodate it. The test asserts that the WrapPanel
//!   rectangle stays at main-axis = 100, the child's resolved rectangle
//!   has width = 200, `x + width > 100` (visible overflow surfaces to
//!   the Visual tree), and the WrapPanel's visual has no `Clip` property
//!   installed by Phase 3 code — i.e. the spec'd "visible overflow,
//!   parent clips" convention is preserved end-to-end.
//!
//! Skip-guard inherits the Phase 2 T11 / Phase 1 T6 / T13 pattern: fail
//! (not skip) on the GitHub Actions runner; locally skip on
//! `0x80070005` (E_ACCESSDENIED) from `wasamo_init`. The guard's
//! triggering half was verified against an SSH dev box where
//! `wasamo_init` returns `0x80070005`; see
//! `docs/notes/m3-phase-3/t8-step-end-retrospective.md`.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Composition::Visual;

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<wrap-panel-layout-integration>";
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

fn visual_rect(widget: &WidgetNode) -> (f32, f32, f32, f32) {
    let visual: Visual = widget.visual.cast().expect("cast SpriteVisual to Visual");
    let offset = visual.Offset().unwrap_or(Vector3 {
        X: 0.0,
        Y: 0.0,
        Z: 0.0,
    });
    let size = visual.Size().unwrap_or(Vector2 { X: 0.0, Y: 0.0 });
    (offset.X, offset.Y, size.X, size.Y)
}

fn assert_close(actual: f32, expected: f32, label: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.01,
        "{label}: expected {expected}, got {actual} (delta {delta})"
    );
}

#[test]
fn wrap_path_fixture_lays_out_multi_line_thumbnails() {
    // Marshalled onto the runtime-owning thread (Observation 5 step 1): the
    // Compositor is created and used on one thread. See tests/common/mod.rs.
    run_on_owning_runtime_thread_or_skip("WrapPanel wrap-path integration test", move || {
        // Six 1:1 Boxes with item-cross-size: 88 → each child measures 88×88.
        // Parent main-axis bound = 300; with item-spacing: 12 three boxes
        // fit per line (88×3 + 12×2 = 288 ≤ 300) and a fourth would exceed
        // the bound (288 + 12 + 88 = 388). Two lines of three with
        // line-spacing: 12 → outer cross = 88×2 + 12 = 188.
        let src = r#"component WrapLayout inherits Window {
    WrapPanel {
        item-cross-size: 88
        item-spacing: 12
        line-spacing: 12
        Box {
            aspect: 1:1
            fill: #336699cc
            Text { text: "1" }
        }
        Box {
            aspect: 1:1
            fill: #336699cc
            Text { text: "2" }
        }
        Box {
            aspect: 1:1
            fill: #336699cc
            Text { text: "3" }
        }
        Box {
            aspect: 1:1
            fill: #336699cc
            Text { text: "4" }
        }
        Box {
            aspect: 1:1
            fill: #336699cc
            Text { text: "5" }
        }
        Box {
            aspect: 1:1
            fill: #336699cc
            Text { text: "6" }
        }
    }
}"#;

        let ir = lower_ui_to_ir(src);
        let component = parse_ir(&ir).expect("parse_ir failed");
        let compositor = wasamo_runtime::get_compositor();
        let text_renderer = wasamo_runtime::get_text_renderer();
        let mut built = build_widget_tree(&component, compositor, text_renderer)
            .expect("build_widget_tree failed");

        let root = built.root.as_mut();
        assert_eq!(root.children.len(), 6, "WrapPanel must have six children");

        // Parent extent: 300 main × 400 cross. Two lines × 88 + 12 = 188 cross
        // fits within 400; main axis is bounded by the parent (Fill-width
        // WrapPanel resolves to the parent's main bound = 300).
        root.run_layout(300.0, 400.0).expect("run_layout failed");

        let (wp_x, wp_y, wp_w, wp_h) = visual_rect(root);
        assert_close(wp_x, 0.0, "WrapPanel visual x");
        assert_close(wp_y, 0.0, "WrapPanel visual y");
        assert_close(wp_w, 300.0, "WrapPanel outer main (parent bound)");
        assert_close(wp_h, 188.0, "WrapPanel outer cross (2 lines × 88 + 12)");

        let expected = [
            // (x, y) per child — line 0: column 0/1/2; line 1: column 0/1/2.
            (0.0_f32, 0.0_f32),     // child 0: col 0, line 0
            (100.0_f32, 0.0_f32),   // child 1: col 1 (88 + 12)
            (200.0_f32, 0.0_f32),   // child 2: col 2 (88 + 12 + 88 + 12)
            (0.0_f32, 100.0_f32),   // child 3: col 0, line 1 (88 + 12)
            (100.0_f32, 100.0_f32), // child 4: col 1, line 1
            (200.0_f32, 100.0_f32), // child 5: col 2, line 1
        ];

        for (i, &(ex, ey)) in expected.iter().enumerate() {
            let (cx, cy, cw, ch) = visual_rect(&root.children[i]);
            assert_close(cw, 88.0, &format!("child {i} width"));
            assert_close(ch, 88.0, &format!("child {i} height"));
            assert_close(cx, ex, &format!("child {i} x"));
            assert_close(cy, ey, &format!("child {i} y"));
        }
    });
}

#[test]
fn oversized_child_fixture_paints_visible_overflow() {
    // Marshalled onto the runtime-owning thread (Observation 5 step 1): the
    // Compositor is created and used on one thread. See tests/common/mod.rs.
    run_on_owning_runtime_thread_or_skip("WrapPanel oversized-child integration test", move || {
        // One Box with aspect 4:1 and item-cross-size: 50 → intrinsic
        // (200, 50). Parent main-axis bound = 100; per DD-M3-P3-005
        // oversized-line Option A the WrapPanel does not grow to fit.
        let src = r#"component WrapOverflow inherits Window {
    WrapPanel {
        item-cross-size: 50
        Box {
            aspect: 4:1
            fill: #336699cc
        }
    }
}"#;

        let ir = lower_ui_to_ir(src);
        let component = parse_ir(&ir).expect("parse_ir failed");
        let compositor = wasamo_runtime::get_compositor();
        let text_renderer = wasamo_runtime::get_text_renderer();
        let mut built = build_widget_tree(&component, compositor, text_renderer)
            .expect("build_widget_tree failed");

        let root = built.root.as_mut();
        assert_eq!(root.children.len(), 1, "WrapPanel must have one child");

        // Parent extent: 100 main × 200 cross. The oversized child does not
        // expand the WrapPanel main axis.
        root.run_layout(100.0, 200.0).expect("run_layout failed");

        let (_, _, wp_w, wp_h) = visual_rect(root);
        assert_close(wp_w, 100.0, "WrapPanel main stays at parent bound");
        assert_close(wp_h, 50.0, "WrapPanel cross == single line × 50");

        let (cx, _, cw, ch) = visual_rect(&root.children[0]);
        assert_close(cw, 200.0, "oversized child width (aspect 4:1 × 50)");
        assert_close(ch, 50.0, "oversized child height (item-cross-size)");
        assert!(
            cx + cw > wp_w,
            "expected visible overflow: child end {} should exceed WrapPanel end {}",
            cx + cw,
            wp_w
        );

        // Phase 3 installs no clip surface on the WrapPanel's visual —
        // the spec'd "visible overflow, parent clips" convention is
        // preserved through to the Visual tree. `Visual::Clip()` returns
        // an error when no clip is installed; an Ok(_) here would mean
        // Phase 3 code accidentally set one.
        let visual: Visual = root.visual.cast().expect("cast SpriteVisual to Visual");
        assert!(
            visual.Clip().is_err(),
            "WrapPanel must not install a clip surface (Phase 3 visible-overflow rule)"
        );
    });
}
