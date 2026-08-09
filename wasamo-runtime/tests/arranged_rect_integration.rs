//! Mock-free Windows-only integration evidence for M4-Phase 2 T1.
//!
//! DD-M4-P2-002 "one walk, two stores": `WidgetNode::sync_visuals` now
//! writes, per node, both the physical Composition geometry (`Visual.Offset`
//! / `Visual.Size`) and a node-side `arranged_rect` retained as an absolute
//! DIP rectangle. This file pins the properties that make the second store
//! trustworthy as a hit-testing geometry source: every laid-out node gets
//! one, it agrees with what the live Visual tree actually paints, it is DIP
//! rather than physical, an unlaid-out node has none, a ScrollView-scrolled
//! child's rectangle reflects the position it is actually painted at (not
//! the pre-scroll position), and the `clips_children` predicate agrees with
//! the live `Visual.Clip()` for every widget kind.
//!
//! Skip-guard follows the established pattern: local `0x80070005` from
//! `wasamo_init` skips, GitHub Actions fails rather than silently skipping
//! (CLAUDE.md §Testing rules).

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::collections::BTreeSet;

use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::{
    Alignment, ButtonStyle, DipRect, SizeConstraint, TypographyStyle, WidgetNode,
};

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Composition::Visual;

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<arranged-rect-integration>";
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

fn arranged_rect_or_panic(node: &WidgetNode, label: &str) -> DipRect {
    node.__arranged_rect_for_test()
        .unwrap_or_else(|| panic!("{label}: arranged_rect must be Some after layout"))
}

// ── Shared fixture for (a) and (b): a nested tree with non-zero parent
// offsets at every level ──────────────────────────────────────────────────
//
// VStack(padding, spacing non-zero) -> HStack(padding non-zero) ->
// [Button, Button, Rectangle]. `Box` is not used here even though the task
// description's example names it: `WidgetNode::box_` is crate-private (see
// `dpi_change_propagation_integration.rs`), so it is not reachable from this
// external test crate without going through the IR loader. `Rectangle` is a
// directly-constructible `pub` leaf with an equally arbitrary Fixed size, so
// it serves the same role (an innermost leaf whose absolute rect can be
// hand-computed) without adding IR-loader machinery to a test that is not
// about Box specifically.
const VSTACK_PADDING: f32 = 20.0;
const VSTACK_SPACING: f32 = 10.0;
const HSTACK_PADDING: f32 = 15.0;
const HSTACK_SPACING: f32 = 6.0;
const RECT_W: f32 = 50.0;
const RECT_H: f32 = 30.0;
const WINDOW_W: f32 = 500.0;
const WINDOW_H: f32 = 300.0;

fn build_nested_tree(
    compositor: &windows::UI::Composition::Compositor,
    renderer: &wasamo_runtime::TextRenderer,
) -> Box<WidgetNode> {
    let mut root = WidgetNode::vstack(
        compositor,
        VSTACK_SPACING,
        VSTACK_PADDING,
        Alignment::Leading,
    )
    .expect("create VStack root");
    let mut hstack = WidgetNode::hstack(
        compositor,
        HSTACK_SPACING,
        HSTACK_PADDING,
        Alignment::Leading,
    )
    .expect("create HStack");
    hstack
        .append_child(
            WidgetNode::button(compositor, renderer, "A", ButtonStyle::Default)
                .expect("create Button A"),
        )
        .expect("append Button A");
    hstack
        .append_child(
            WidgetNode::button(compositor, renderer, "BB", ButtonStyle::Default)
                .expect("create Button B"),
        )
        .expect("append Button B");
    hstack
        .append_child(
            WidgetNode::rectangle(
                compositor,
                SizeConstraint::Fixed(RECT_W),
                SizeConstraint::Fixed(RECT_H),
            )
            .expect("create Rectangle"),
        )
        .expect("append Rectangle");
    root.append_child(hstack).expect("append HStack");
    root
}

/// Recursively assert `arranged_rect` is `Some` for every node in the tree.
fn assert_every_node_has_rect(node: &WidgetNode, label: &str) {
    assert!(
        node.__arranged_rect_for_test().is_some(),
        "{label}: arranged_rect must be Some after layout"
    );
    for (i, child) in node.children.iter().enumerate() {
        assert_every_node_has_rect(child, &format!("{label}.child[{i}]"));
    }
}

/// Cross-check `arranged_rect` against an independently derived value:
/// accumulate `Visual.Offset()` down the tree from `parent_abs_dip` and
/// divide by `scale_factor` (1.0 at 96 DPI), and compare against the DIP
/// `Visual.Size()`. This proves the two stores agree.
///
/// Only valid for trees without a `ScrollView`: a ScrollView interposes a
/// non-`WidgetNode` intermediate Visual (`content_visual`) between the
/// ScrollView's own Visual and its content child's Visual, so summing
/// `WidgetNode.children` parent-relative offsets alone would skip the
/// intermediate's translation. Test (d) covers that kind directly.
fn assert_rect_matches_visual_chain(node: &WidgetNode, parent_abs_dip: (f32, f32), label: &str) {
    let rect = arranged_rect_or_panic(node, label);

    let visual = visual_of(node);
    let (rel_x, rel_y) = visual_offset(&visual);
    let (phys_w, phys_h) = visual_size(&visual);
    // scale_factor == 1.0 at 96 DPI (this fixture's fixed target).
    let abs_x = parent_abs_dip.0 + rel_x;
    let abs_y = parent_abs_dip.1 + rel_y;

    assert_close(
        rect.x,
        abs_x,
        &format!("{label}: x matches accumulated Visual.Offset"),
    );
    assert_close(
        rect.y,
        abs_y,
        &format!("{label}: y matches accumulated Visual.Offset"),
    );
    assert_close(
        rect.width,
        phys_w,
        &format!("{label}: width matches Visual.Size"),
    );
    assert_close(
        rect.height,
        phys_h,
        &format!("{label}: height matches Visual.Size"),
    );

    for (i, child) in node.children.iter().enumerate() {
        assert_rect_matches_visual_chain(child, (abs_x, abs_y), &format!("{label}.child[{i}]"));
    }
}

#[test]
fn every_laid_out_node_retains_its_absolute_dip_rectangle() {
    run_on_owning_runtime_thread_or_skip(
        "every laid-out node retains its absolute DIP rectangle",
        move || {
            let compositor = wasamo_runtime::get_compositor();
            let renderer = wasamo_runtime::get_text_renderer();
            let mut root = build_nested_tree(compositor, renderer);

            root.run_layout_as_window_root(WINDOW_W, WINDOW_H)
                .expect("run_layout_as_window_root failed");

            // (1) Every node in the tree has a rectangle.
            assert_every_node_has_rect(&root, "root");

            // (2) Cross-check: accumulated Visual.Offset chain agrees with the
            // node-side store, for every node.
            assert_rect_matches_visual_chain(&root, (0.0, 0.0), "root");

            // (3) Hand-computed absolute expectation for an innermost child, so
            // a store that recorded the parent-relative offset (rather than
            // absolute) would fail on a *value*, not only on the cross-check
            // above (which would also happen to fail, but this pins the
            // expected number independently).
            //
            // Button A is the first child of the HStack, itself the sole child
            // of the VStack root. Its absolute position does not depend on any
            // measured text width (unlike its siblings, whose x position
            // depends on Button A's measured width) — only on the two
            // containers' padding:
            //   x = 0 (root) + VSTACK_PADDING + HSTACK_PADDING
            //   y = 0 (root) + VSTACK_PADDING + HSTACK_PADDING
            // A parent-relative store would instead read HSTACK_PADDING alone
            // (15.0), which is a different, checkable number.
            let hstack = &root.children[0];
            let button_a = &hstack.children[0];
            let button_a_rect = arranged_rect_or_panic(button_a, "button_a");
            let expected_xy = VSTACK_PADDING + HSTACK_PADDING;
            assert_close(
                button_a_rect.x,
                expected_xy,
                "button_a: hand-computed absolute x",
            );
            assert_close(
                button_a_rect.y,
                expected_xy,
                "button_a: hand-computed absolute y",
            );

            // The Rectangle (third child of the HStack) is the deepest leaf on
            // this branch and gets the same treatment for its y and size, which
            // do not depend on the buttons' measured widths (HStack's cross
            // axis is height, and Leading alignment places every non-Fill-height
            // child at the same y regardless of its own extent).
            let rect_node = &hstack.children[2];
            let rect_rect = arranged_rect_or_panic(rect_node, "rectangle");
            assert_close(
                rect_rect.y,
                expected_xy,
                "rectangle: hand-computed absolute y",
            );
            assert_close(rect_rect.width, RECT_W, "rectangle: hand-computed width");
            assert_close(rect_rect.height, RECT_H, "rectangle: hand-computed height");
        },
    );
}

#[test]
fn the_retained_rectangle_is_dip_not_physical() {
    run_on_owning_runtime_thread_or_skip(
        "the retained rectangle is DIP, not physical",
        move || {
            let compositor = wasamo_runtime::get_compositor();
            let renderer = wasamo_runtime::get_text_renderer();

            // Two independently-built but structurally/numerically identical
            // trees, laid out at the same DIP window size but different DPI. Two
            // trees rather than a second pass over one tree: `sync_visuals` reads
            // `self.scale` implicitly through the two run_layout* entry points'
            // caller-supplied target, and building fresh keeps each tree's
            // Composition state unambiguous about which pass produced it.
            let mut root_96 = build_nested_tree(compositor, renderer);
            root_96
                .run_layout_as_window_root(WINDOW_W, WINDOW_H)
                .expect("96 DPI layout failed");

            let mut root_144 = build_nested_tree(compositor, renderer);
            // This seam needs no window and no particular monitor: it drives
            // geometry-only layout at an explicit DPI directly on a standalone
            // WidgetNode tree, so it introduces no desktop-range dependency.
            root_144
                .__run_layout_as_window_root_at_dpi_for_test(WINDOW_W, WINDOW_H, 144)
                .expect("144 DPI layout failed");

            assert_dip_unchanged_and_physical_scaled(&root_96, &root_144, "root");
        },
    );
}

/// Recursively assert that `arranged_rect` is bit-for-bit unchanged between
/// the 96 DPI and 144 DPI trees (the claim under test: the store is DIP, so
/// the same DIP window size produces the same rectangle regardless of DPI —
/// a store that instead wrote physical values would fail this at 144 DPI,
/// where every value would read 1.5x larger), while the live Visual's
/// physical `Size()` really did grow by 1.5x (the positive control: the 144
/// DPI pass was not a no-op).
fn assert_dip_unchanged_and_physical_scaled(
    node_96: &WidgetNode,
    node_144: &WidgetNode,
    label: &str,
) {
    let rect_96 = arranged_rect_or_panic(node_96, &format!("{label} (96 DPI)"));
    let rect_144 = arranged_rect_or_panic(node_144, &format!("{label} (144 DPI)"));

    assert_close(
        rect_144.x,
        rect_96.x,
        &format!("{label}: x unchanged across DPI"),
    );
    assert_close(
        rect_144.y,
        rect_96.y,
        &format!("{label}: y unchanged across DPI"),
    );
    assert_close(
        rect_144.width,
        rect_96.width,
        &format!("{label}: width unchanged across DPI"),
    );
    assert_close(
        rect_144.height,
        rect_96.height,
        &format!("{label}: height unchanged across DPI"),
    );

    let visual_96 = visual_of(node_96);
    let visual_144 = visual_of(node_144);
    let (w_96, h_96) = visual_size(&visual_96);
    let (w_144, h_144) = visual_size(&visual_144);
    assert_close(
        w_144,
        w_96 * 1.5,
        &format!("{label}: Visual width scaled by 1.5"),
    );
    assert_close(
        h_144,
        h_96 * 1.5,
        &format!("{label}: Visual height scaled by 1.5"),
    );

    assert_eq!(
        node_96.children.len(),
        node_144.children.len(),
        "{label}: the two trees must have identical shape"
    );
    for (i, (c96, c144)) in node_96
        .children
        .iter()
        .zip(node_144.children.iter())
        .enumerate()
    {
        assert_dip_unchanged_and_physical_scaled(c96, c144, &format!("{label}.child[{i}]"));
    }
}

#[test]
fn an_attached_but_unlaid_out_subtree_has_no_rectangle() {
    run_on_owning_runtime_thread_or_skip(
        "an attached but unlaid-out subtree has no rectangle",
        move || {
            let compositor = wasamo_runtime::get_compositor();

            let mut root =
                WidgetNode::vstack(compositor, 0.0, 0.0, Alignment::Leading).expect("create root");
            root.append_child(
                WidgetNode::rectangle(
                    compositor,
                    SizeConstraint::Fixed(10.0),
                    SizeConstraint::Fixed(10.0),
                )
                .expect("create first Rectangle"),
            )
            .expect("append first Rectangle");

            root.run_layout_as_window_root(100.0, 100.0)
                .expect("initial layout failed");
            assert!(
                root.children[0].__arranged_rect_for_test().is_some(),
                "the already-laid-out child must have a rectangle"
            );

            // Append a freshly created widget without running layout again.
            // The `None` here is the intended failure mode, asserted rather
            // than assumed: a node that has never been through `sync_visuals`
            // must not appear hit-testable at some leftover or default rect.
            root.append_child(
                WidgetNode::rectangle(
                    compositor,
                    SizeConstraint::Fixed(10.0),
                    SizeConstraint::Fixed(10.0),
                )
                .expect("create second Rectangle"),
            )
            .expect("append second Rectangle");
            assert_eq!(
                root.children[1].__arranged_rect_for_test(),
                None,
                "an attached-but-unlaid-out node must have no arranged_rect"
            );

            root.run_layout_as_window_root(100.0, 100.0)
                .expect("second layout failed");
            assert!(
                root.children[1].__arranged_rect_for_test().is_some(),
                "the newly appended child must gain a rectangle once layout runs"
            );
        },
    );
}

// ── (d): ScrollView content retains the position it is painted at ──────────

// Mirrors the established `scroll_view_layout_integration.rs` fixture shape:
// a ScrollView (component root, Fill/Fill viewport) with a bound `offset-y`
// over a WrapPanel of eight 50x50 Boxes (`item-cross-size: 50`). Two boxes
// per line (viewport width 100) -> four lines -> content height 200.
const SCROLL_FIXTURE_SRC: &str = r#"component ScrollRectFixture inherits Window {
    state scroll_y: i32 = 0
    ScrollView {
        offset-y: scroll_y
        WrapPanel {
            item-cross-size: 50
            item-spacing: 0
            line-spacing: 0
            Box { aspect: 1:1 fill: #336699cc }
            Box { aspect: 1:1 fill: #336699cc }
            Box { aspect: 1:1 fill: #336699cc }
            Box { aspect: 1:1 fill: #336699cc }
            Box { aspect: 1:1 fill: #336699cc }
            Box { aspect: 1:1 fill: #336699cc }
            Box { aspect: 1:1 fill: #336699cc }
            Box { aspect: 1:1 fill: #336699cc }
        }
    }
}"#;

const SCROLL_VIEWPORT_W: f32 = 100.0;
const SCROLL_VIEWPORT_H: f32 = 100.0;
// Content height (4 lines x 50) is 200, so a mid-range scroll_y of 50 sits
// strictly inside [0, max_offset = 200 - 100 = 100]: the clamp is a no-op,
// so `applied == scroll_y` exactly, with no need to read the clamped value
// back or recompute max_offset from the WrapPanel geometry.
const SCROLL_Y: i32 = 50;

#[test]
fn a_scrollview_child_retains_the_position_it_paints_at() {
    run_on_owning_runtime_thread_or_skip(
        "a ScrollView child retains the position it paints at",
        move || {
            let ir = lower_ui_to_ir(SCROLL_FIXTURE_SRC);
            let component = parse_ir(&ir).expect("parse_ir failed");
            let compositor = wasamo_runtime::get_compositor();
            let text_renderer = wasamo_runtime::get_text_renderer();
            let mut built = build_widget_tree(&component, compositor, text_renderer)
                .expect("build_widget_tree failed");

            assert!(
                built.__set_i32_state_for_test("scroll_y", SCROLL_Y),
                "__set_i32_state_for_test must find scroll_y"
            );
            built
                .root
                .run_layout_as_window_root(SCROLL_VIEWPORT_W, SCROLL_VIEWPORT_H)
                .expect("run_layout_as_window_root failed");

            let scroll_view = built.root.as_ref();
            assert_eq!(
                scroll_view.children.len(),
                1,
                "ScrollView must have exactly one content child (WrapPanel)"
            );
            let content = &scroll_view.children[0];

            let scroll_rect = arranged_rect_or_panic(scroll_view, "ScrollView");
            let content_rect = arranged_rect_or_panic(content, "content (WrapPanel)");

            // The relation under test: the content child's retained rectangle
            // is the position it is actually painted at (viewport-relative,
            // shifted by the applied scroll offset) — not the unscrolled
            // position. `arrange_scroll_view` arranges the content at
            // `(x, y - applied)` absolute, and DD-M4-P2-002 retains that same
            // `computed.offset` un-subtracted, so the content's y must sit
            // exactly `SCROLL_Y` above the ScrollView's own y.
            assert_close(
                content_rect.y,
                scroll_rect.y - SCROLL_Y as f32,
                "content y must be ScrollView y minus the applied scroll offset",
            );
        },
    );
}

// ── (e): clips_children agrees with the live Visual for every widget kind ──

// `Box`, `WrapPanel`, `ScrollView`, `Grid`, and `ZStack` constructors are
// crate-private (`WidgetNode::box_` et al.), so this file cannot call them
// directly; they are pulled out of one IR-loader-built tree instead. `Cell`
// is IR-only (never materialises as a `WidgetData` variant), so the Grid's
// single `Text` content child threads straight through.
const ALL_KINDS_SRC: &str = r#"component AllWidgetKinds inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Box {
            aspect: 1:1
        }
        WrapPanel {
        }
        ScrollView {
            Box { aspect: 1:1 }
        }
        Grid {
            columns: 1*
            rows: 1*
            Cell { row: 0 column: 0 Text { text: "g" } }
        }
        ZStack {
        }
    }
}"#;

#[test]
fn the_clip_predicate_agrees_with_the_live_visual_for_every_widget_kind() {
    run_on_owning_runtime_thread_or_skip(
        "clips_children agrees with the live Visual for every widget kind",
        move || {
            let ir = lower_ui_to_ir(ALL_KINDS_SRC);
            let component = parse_ir(&ir).expect("parse_ir failed");
            let compositor = wasamo_runtime::get_compositor();
            let text_renderer = wasamo_runtime::get_text_renderer();
            let mut built = build_widget_tree(&component, compositor, text_renderer)
                .expect("build_widget_tree failed");

            let mut kinds: Vec<Box<WidgetNode>> = Vec::new();

            // Pull the five IR-loader-only kinds out of the VStack root, in
            // document order (Box, WrapPanel, ScrollView, Grid, ZStack).
            // Always removing index 0 keeps the remaining indices valid as
            // the Vec shrinks. The strings below are construction-time notes
            // only, used in the panic message if `remove_child` itself
            // fails — they are not trusted as the pulled node's identity;
            // `__kind_name_for_test` below is the source of truth for that,
            // since a loader reordering would make a positional label lie
            // while `remove_child`/`Clip()` still silently agreed with each
            // other.
            for construction_note in ["Box", "WrapPanel", "ScrollView", "Grid", "ZStack"] {
                let child = built.root.remove_child(0).unwrap_or_else(|e| {
                    panic!("remove_child(0) for {construction_note} failed: {e:?}")
                });
                kinds.push(child);
            }
            // The VStack root, now emptied of children, stands in for the
            // VStack kind itself.
            kinds.push(built.root);

            kinds.push(
                WidgetNode::rectangle(
                    compositor,
                    SizeConstraint::Fixed(10.0),
                    SizeConstraint::Fixed(10.0),
                )
                .expect("create Rectangle"),
            );
            kinds.push(
                WidgetNode::hstack(compositor, 0.0, 0.0, Alignment::Leading)
                    .expect("create HStack"),
            );
            kinds.push(
                WidgetNode::text(compositor, text_renderer, "x", TypographyStyle::Body)
                    .expect("create Text"),
            );
            kinds.push(
                WidgetNode::button(compositor, text_renderer, "x", ButtonStyle::Default)
                    .expect("create Button"),
            );
            kinds.push(
                WidgetNode::toggle_button(
                    compositor,
                    text_renderer,
                    "x",
                    ButtonStyle::Default,
                    true,
                    false,
                )
                .expect("create ToggleButton"),
            );

            // What a new `WidgetData` variant compile-forces is its
            // *classification* — `clips_children` and `__kind_name_for_test`
            // are exhaustive, so neither compiles until the new kind is
            // named. What it does not force is *coverage here*: Rust cannot
            // enumerate the variants of an enum at run time, so this table
            // is maintained by hand. The set-equality assertion below is
            // what turns that into a checkable claim — it fails if an entry
            // is not the kind it was pulled out as (e.g. a loader reordering
            // that makes `remove_child(0)` pull the wrong sibling), and it
            // fails if the expected set is edited without adding the
            // instance.
            let actual_kinds: BTreeSet<&str> =
                kinds.iter().map(|n| n.__kind_name_for_test()).collect();
            let expected_kinds: BTreeSet<&str> = [
                "Rectangle",
                "VStack",
                "HStack",
                "Text",
                "Button",
                "ToggleButton",
                "Box",
                "WrapPanel",
                "ScrollView",
                "Grid",
                "ZStack",
            ]
            .into_iter()
            .collect();
            assert_eq!(
                actual_kinds, expected_kinds,
                "expected one instance of every WidgetData kind, identified by \
                 __kind_name_for_test rather than by construction position"
            );

            for node in &kinds {
                let label = node.__kind_name_for_test();
                let visual = visual_of(node);
                assert_eq!(
                    node.__clips_children_for_test(),
                    visual.Clip().is_ok(),
                    "{label}: clips_children() must agree with the live Visual.Clip()"
                );
            }
        },
    );
}
