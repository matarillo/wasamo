//! Mock-free Windows-only layout integration evidence for M3-Phase 5 T4.
//!
//! Drives the production path end-to-end for the Grid layout primitive
//! (DD-M3-P5-001 / DD-M3-P5-002 / DD-M3-P5-004 / DD-M3-P5-005): `.ui`
//! source is lowered by `wasamoc`, parsed by the runtime IR loader, built
//! into live `WidgetNode`s backed by a real Compositor / TextRenderer,
//! laid out, and then read back from the resulting Composition Visual
//! tree.
//!
//! Discharges ADR Phase 5 verification closure **evidence item (4)** —
//! see `process/milestone-3/phase-5/decisions/preamble.md §Phase 5
//! verification closure` for the (a)-(d) assertion menu. Two fixtures:
//!
//! - **grid_rooted_fixture_lays_out_cells_through_visual_tree** —
//!   window-root Fill/Fill path. A Grid is the component root with mixed
//!   fixed + weighted-star tracks on both axes; driven through
//!   `WidgetNode::run_layout_as_window_root` (the Phase 4 T6 entry
//!   point).
//! - **grid_vstack_root_fixture_pins_production_root_shape** —
//!   production-root-shape carry-forward per
//!   `../requirements/constraints.md §1` (Phase 4 T6 fix class). A
//!   `VStack { Button + Grid }` shape matching the gallery / counter /
//!   bool-demo `.ui` production root family; guards against the Phase 4
//!   T6 runtime-boundary collapse (a Fill Grid child of a Shrink VStack
//!   root collapsing to zero extent).
//!
//! Both fixtures assert:
//!   (a) Grid's resolved rectangle matches the parent allocation;
//!   (b) each Cell content Visual offset matches the resolved cell
//!       rectangle origin (parent-relative offsets per
//!       `docs/architecture.md §6.5`);
//!   (c) Grid's outer Visual has a **non-null** clip — the `InsetClip`
//!       from DD-M3-P5-005 (clip-presence assertion);
//!   (d) each Cell content Visual has `Visual.Clip = null` — clip-absence
//!       regression guard, symmetric with the Phase 3 T8 WrapPanel and
//!       Phase 4 ScrollView precedents.
//!
//! Skip-guard (`init_runtime_or_skip`) is reused verbatim from the
//! Phase 2 T11 / Phase 3 T8 / Phase 4 T4 pattern: fail (not skip) on the
//! GitHub Actions runner; locally skip on `0x80070005` (E_ACCESSDENIED)
//! from `wasamo_init`. Phase 5 introduces no separate runtime capability
//! path — the guard fires on the same Compositor-unavailable surface
//! earlier phases use. The byte-identical guard was verified to fire
//! (test FAILS, not skips) on the SSH dev box where `wasamo_init`
//! returns `0x80070005` in the Phase 4 T4 verification; the Phase 5
//! re-confirmation per CLAUDE.md §Testing rules is the T4 checklist
//! item-4 owner/environment gate.
//!
//! Unbounded star-axis runtime fixture (ADR Phase 5 verification closure
//! item (4), third sub-bullet) is **downgraded to pure-logic coverage**
//! per the closure's escape clause, mirroring the Phase 4 ScrollView T4
//! disposition. Every DSL parent in the Phase 5 widget catalog (VStack /
//! HStack / Box / WrapPanel / ScrollView / window root) passes a finite
//! cell to its Grid child at arrange time, and DSL-authored `.ui` cannot
//! set a Grid's `width` / `height` (so a Grid is always Fill/Fill and
//! never reaches `measure_grid`'s Shrink branch with an unbounded probe).
//! There is therefore no ergonomic `.ui` / IR-level fixture that can
//! reach `resolve_axis_tracks` with an unbounded star axis. The
//! pure-logic `grid_arrange_unbounded_star_axis_errors` and
//! `grid_resolve_unbounded_star_axis_errors` tests in
//! `wasamo-runtime::layout::tests` (T2) already pin that branch — see
//! the T4 step-end retrospective Item 10 and log.md T4 Decisions entry
//! for the disposition record.

#![cfg(windows)]

use std::ffi::CStr;

use wasamo_runtime::ffi;
use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Composition::Visual;

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<grid-layout-integration>";
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
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.01,
        "{label}: expected {expected}, got {actual} (delta {delta})"
    );
}

/// Initialise the runtime; return `Some(())` on success, `None` if the
/// Compositor is locally unavailable (in which case the caller returns
/// from the test, skipping it). Fails the test on GitHub Actions —
/// CI must surface a missing Compositor as a failure per CLAUDE.md
/// §Testing rules; the skip is a developer-laptop convenience only.
fn init_runtime_or_skip(test_name: &str) -> Option<()> {
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
                "{test_name} cannot skip on GitHub Actions: \
                 runtime compositor unavailable ({msg:?})"
            );
            eprintln!("skipping {test_name}: runtime compositor unavailable ({msg:?})");
            return None;
        }
    }
    assert_eq!(
        status,
        ffi::WASAMO_OK,
        "wasamo_init failed: {:?}",
        last_error()
    );
    Some(())
}

// With default (stretch / stretch) alignment each Cell's single content
// child fills its resolved rectangle, so the content Visual's
// parent-relative offset equals the cell origin and its size equals the
// cell extent — independent of the Box's own intrinsic size. Both
// fixtures use the same three Cells: (row 0, col 0), (row 0, col 1), and
// (row 1, col 0) spanning two columns.

/// (b)/(c)/(d) shared assertions over a built Grid `WidgetNode` whose
/// three Box content children are at `children[0..3]`. `expected` carries
/// each child's expected parent-relative `(x, y, w, h)`.
fn assert_grid_cells(grid: &WidgetNode, expected: &[(f32, f32, f32, f32)]) {
    assert_eq!(
        grid.children.len(),
        expected.len(),
        "Grid must flatten exactly {} Cell content children",
        expected.len()
    );

    // (c) Grid's outer Visual carries the DD-M3-P5-005 outer-bounds clip.
    let grid_visual = visual_of(grid);
    assert!(
        grid_visual.Clip().is_ok(),
        "Grid outer Visual must carry the outer-bounds clip (DD-M3-P5-005 InsetClip)"
    );

    for (i, &(ex, ey, ew, eh)) in expected.iter().enumerate() {
        let cv = visual_of(&grid.children[i]);

        // (b) Cell content Visual offset == resolved cell origin.
        let (cx, cy) = visual_offset(&cv);
        assert_close(
            cx,
            ex,
            &format!("cell {i} content Visual x (resolved cell origin)"),
        );
        assert_close(
            cy,
            ey,
            &format!("cell {i} content Visual y (resolved cell origin)"),
        );

        // Size pinned too: stretch alignment expands the content to the
        // full resolved cell extent (span included for cell 2).
        let (cw, ch) = visual_size(&cv);
        assert_close(
            cw,
            ew,
            &format!("cell {i} content Visual width (cell extent)"),
        );
        assert_close(
            ch,
            eh,
            &format!("cell {i} content Visual height (cell extent)"),
        );

        // (d) Cell content Visual must NOT carry a clip — per-cell
        // clipping is out of scope; overflow is contained only at Grid's
        // outer-bounds clip (symmetric with the Phase 3 WrapPanel and
        // Phase 4 ScrollView clip-absence guards).
        assert!(
            cv.Clip().is_err(),
            "cell {i} content Visual must NOT carry a clip (DD-M3-P5-005 — clip lives on Grid's outer Visual only)"
        );
    }
}

// ── Fixture 1: Grid-rooted (window-root Fill/Fill path) ─────────────────────

// Grid as the component root with mixed fixed + weighted-star tracks on
// both axes. Driven through `run_layout_as_window_root`, which forces the
// root Fill/Fill so the window client rect determines the Grid viewport.
const GRID_ROOT_SRC: &str = r#"component GridRoot inherits Window {
    Grid {
        columns: 100 1*
        rows: 50 1*
        Cell { row: 0 column: 0 Box { fill: #336699cc } }
        Cell { row: 0 column: 1 Box { fill: #993366cc } }
        Cell { row: 1 column: 0 column-span: 2 Box { fill: #66993399 } }
    }
}"#;

const GRID_ROOT_W: f32 = 300.0;
const GRID_ROOT_H: f32 = 200.0;

#[test]
fn grid_rooted_fixture_lays_out_cells_through_visual_tree() {
    if init_runtime_or_skip("Grid-rooted layout integration test").is_none() {
        return;
    }

    let ir = lower_ui_to_ir(GRID_ROOT_SRC);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    let mut built =
        build_widget_tree(&component, compositor, text_renderer).expect("build_widget_tree failed");

    // Drive the production window-root entry point (Phase 4 T6).
    built
        .root
        .run_layout_as_window_root(GRID_ROOT_W, GRID_ROOT_H)
        .expect("run_layout_as_window_root failed");

    let root = built.root.as_ref();

    // (a) Grid's resolved rectangle equals the window allocation.
    let grid_visual = visual_of(root);
    let (gx, gy) = visual_offset(&grid_visual);
    let (gw, gh) = visual_size(&grid_visual);
    assert_close(gx, 0.0, "Grid outer x");
    assert_close(gy, 0.0, "Grid outer y");
    assert_close(gw, GRID_ROOT_W, "Grid outer width = window allocation");
    assert_close(gh, GRID_ROOT_H, "Grid outer height = window allocation");

    // Columns `100 1*` @ width 300 → [100, 200]; boundaries [0, 100, 300].
    // Rows    `50 1*`  @ height 200 → [50, 150]; boundaries [0, 50, 200].
    let expected = [
        (0.0, 0.0, 100.0, 50.0),   // cell 0: (row 0, col 0)
        (100.0, 0.0, 200.0, 50.0), // cell 1: (row 0, col 1)
        (0.0, 50.0, 300.0, 150.0), // cell 2: (row 1, col 0, col-span 2)
    ];
    assert_grid_cells(root, &expected);
}

// ── Fixture 2: VStack { Button + Grid } (production root shape) ──────────────

const VSTACK_GRID_SRC: &str = r#"component VStackGridRoot inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button {
            text: "header"
        }
        Grid {
            columns: 100 1*
            rows: 50 50
            Cell { row: 0 column: 0 Box { fill: #336699cc } }
            Cell { row: 0 column: 1 Box { fill: #993366cc } }
            Cell { row: 1 column: 0 column-span: 2 Box { fill: #66993399 } }
        }
    }
}"#;

const VSTACK_GRID_W: f32 = 300.0;
const VSTACK_GRID_H: f32 = 200.0;

#[test]
fn grid_vstack_root_fixture_pins_production_root_shape() {
    if init_runtime_or_skip("VStack-rooted Grid layout integration test").is_none() {
        return;
    }

    let ir = lower_ui_to_ir(VSTACK_GRID_SRC);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    let mut built =
        build_widget_tree(&component, compositor, text_renderer).expect("build_widget_tree failed");

    {
        let root = built.root.as_mut();
        assert_eq!(
            root.children.len(),
            2,
            "VStack root must have two children (Button + Grid)"
        );
    }

    // Drive the production WinRT-bound window-root entry point. Prior to
    // the Phase 4 T6 fix a Fill Grid child of a Shrink VStack root would
    // collapse to a zero-extent outer Visual; `run_layout_as_window_root`
    // forces the VStack root Fill/Fill so the Grid receives the remaining
    // viewport height after the Button's Fixed height.
    built
        .root
        .run_layout_as_window_root(VSTACK_GRID_W, VSTACK_GRID_H)
        .expect("run_layout_as_window_root failed");

    let grid = &built.root.children[1];

    // (a) Grid's resolved rectangle matches the VStack allocation: full
    // width, and a non-zero remaining height (the regression gate for
    // the Phase 4 T6 collapse class — a zero height would mean the Shrink
    // VStack root collapsed its Fill Grid child).
    let grid_visual = visual_of(grid);
    let (gx, gy) = visual_offset(&grid_visual);
    let (gw, gh) = visual_size(&grid_visual);
    assert_close(gx, 0.0, "Grid offset x within VStack");
    assert!(
        gy > 0.0,
        "Grid must sit below the Button within the VStack; got offset y {gy}"
    );
    assert_close(
        gw,
        VSTACK_GRID_W,
        "Grid width = VStack allocation (full width)",
    );
    assert!(
        gh > 0.0,
        "regression gate (Phase 4 T6 fix class): Grid outer Visual height \
         must be > 0 when the production WidgetNode::run_layout_as_window_root \
         drives a VStack-rooted fixture; got {gh}. A zero height indicates \
         the Shrink VStack root collapsed its Fill Grid child.",
    );

    // Columns `100 1*` @ width 300 → boundaries [0, 100, 300].
    // Rows `50 50` (fixed) → boundaries [0, 50, 100], deterministic
    // regardless of the variable Grid outer height the VStack allocates.
    let expected = [
        (0.0, 0.0, 100.0, 50.0),   // cell 0: (row 0, col 0)
        (100.0, 0.0, 200.0, 50.0), // cell 1: (row 0, col 1)
        (0.0, 50.0, 300.0, 50.0),  // cell 2: (row 1, col 0, col-span 2)
    ];
    assert_grid_cells(grid, &expected);
}
