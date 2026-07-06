//! Mock-free Windows-only runtime evidence for M3-Phase 8 T4.
//!
//! Drives `ToggleButton.checked` through the production compiler -> textual IR
//! -> runtime loader -> live `WidgetNode` path. The tests pin runtime
//! defaulting, bool-binding propagation, alpha-style exclusion handlers, and
//! the inherited `enabled: false` click-suppression contract.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};
use wasamo_runtime::WidgetNode;

use windows::core::Interface;
use windows::Foundation::Numerics::{Vector2, Vector3};
use windows::UI::Color;
use windows::UI::Composition::{CompositionColorBrush, Visual};

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<togglebutton-runtime-integration>";
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

fn build_ui(src: &str) -> wasamo_runtime::ir_loader::BuiltUi {
    let ir = lower_ui_to_ir(src);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let compositor = wasamo_runtime::get_compositor();
    let text_renderer = wasamo_runtime::get_text_renderer();
    build_widget_tree(&component, compositor, text_renderer).expect("build_widget_tree failed")
}

fn read_widget_color(widget: &WidgetNode) -> Color {
    let brush = widget
        .visual
        .Brush()
        .expect("widget visual must have a brush");
    let color_brush: CompositionColorBrush = brush
        .cast()
        .expect("widget brush must be a CompositionColorBrush");
    color_brush.Color().expect("read brush colour")
}

fn rgba(color: Color) -> (u8, u8, u8, u8) {
    (color.A, color.R, color.G, color.B)
}

fn pin_hit_rect(widget: &WidgetNode) {
    let visual: Visual = widget.visual.cast().expect("cast SpriteVisual to Visual");
    visual
        .SetOffset(Vector3 {
            X: 0.0,
            Y: 0.0,
            Z: 0.0,
        })
        .expect("set visual offset");
    visual
        .SetSize(Vector2 { X: 120.0, Y: 40.0 })
        .expect("set visual size");
}

#[test]
fn togglebutton_default_false_and_literal_checked_drive_distinct_visuals() {
    run_on_owning_runtime_thread_or_skip(
        "ToggleButton default/literal checked integration test",
        move || {
            let built = build_ui(
                r#"component ToggleDefaults inherits Window {
    HStack {
        ToggleButton { text: "All" }
        ToggleButton { text: "Recent" }
        ToggleButton { text: "Favorites" checked: true }
    }
}"#,
            );

            let off = &built.root.children[0];
            let also_off = &built.root.children[1];
            let on = &built.root.children[2];
            assert_eq!(
                off.__togglebutton_checked_for_test(),
                Some(false),
                "absent checked must materialize runtime default false"
            );
            assert_eq!(
                also_off.__togglebutton_checked_for_test(),
                Some(false),
                "a second absent checked value must also default false"
            );
            assert_eq!(
                on.__togglebutton_checked_for_test(),
                Some(true),
                "literal checked=true must materialize on the runtime node"
            );
            assert_eq!(
                rgba(read_widget_color(off)),
                rgba(read_widget_color(also_off)),
                "unchecked ToggleButtons should share the same default background"
            );
            assert_ne!(
                rgba(read_widget_color(off)),
                rgba(read_widget_color(on)),
                "checked visual cue must differ from the default background"
            );
        },
    );
}

#[test]
fn togglebutton_bool_state_flip_reaches_checked_visual() {
    run_on_owning_runtime_thread_or_skip(
        "ToggleButton checked binding propagation integration test",
        move || {
            let built = build_ui(
                r#"component ToggleBinding inherits Window {
    state selected: bool = false
    state unrelated: bool = false
    ToggleButton {
        text: "Albums"
        checked: selected
    }
}"#,
            );

            assert_eq!(built.root.__togglebutton_checked_for_test(), Some(false));
            let before = read_widget_color(&built.root);

            assert!(
                built.__set_bool_state_for_test("unrelated", true),
                "test seam must update the unrelated state"
            );
            assert_eq!(
                rgba(before),
                rgba(read_widget_color(&built.root)),
                "unrelated state writes must not repaint the checked visual"
            );

            assert!(
                built.__set_bool_state_for_test("selected", true),
                "test seam must update the selected state"
            );

            assert_eq!(built.root.__togglebutton_checked_for_test(), Some(true));
            assert_ne!(
                rgba(before),
                rgba(read_widget_color(&built.root)),
                "state write must repaint the checked visual"
            );
        },
    );
}

#[test]
fn togglebutton_alpha_exclusion_click_leaves_exactly_one_checked() {
    run_on_owning_runtime_thread_or_skip(
        "ToggleButton alpha exclusion integration test",
        move || {
            let mut built = build_ui(
                r#"component ToggleTabs inherits Window {
    state all: bool = true
    state albums: bool = false
    state favorites: bool = false
    HStack {
        ToggleButton {
            text: "All"
            checked: all
            clicked => { root.all = true; root.albums = false; root.favorites = false; }
        }
        ToggleButton {
            text: "Albums"
            checked: albums
            clicked => { root.all = false; root.albums = true; root.favorites = false; }
        }
        ToggleButton {
            text: "Favorites"
            checked: favorites
            clicked => { root.all = false; root.albums = false; root.favorites = true; }
        }
    }
}"#,
            );

            assert_eq!(checked_children(&built.root), vec![true, false, false]);

            let albums = built.root.children[1].as_mut();
            pin_hit_rect(albums);
            albums.hit_test_click(60, 20);

            assert_eq!(
                checked_children(&built.root),
                vec![false, true, false],
                "clicking Albums must check Albums and clear its siblings"
            );
        },
    );
}

#[test]
fn disabled_togglebutton_suppresses_click_like_button() {
    run_on_owning_runtime_thread_or_skip(
        "ToggleButton disabled click-suppression integration test",
        move || {
            let mut built = build_ui(
                r#"component DisabledToggle inherits Window {
    state selected: bool = false
    ToggleButton {
        text: "Disabled"
        enabled: false
        checked: selected
        clicked => { root.selected = true; }
    }
}"#,
            );

            assert_eq!(built.root.__button_enabled_for_test(), Some(false));
            assert_eq!(built.root.__togglebutton_checked_for_test(), Some(false));

            pin_hit_rect(&built.root);
            built.root.hit_test_click(60, 20);

            assert_eq!(
                built.root.__togglebutton_checked_for_test(),
                Some(false),
                "disabled ToggleButton must not fire clicked handlers"
            );
        },
    );
}

#[test]
fn disabled_checked_togglebutton_shows_disabled_not_checked_color() {
    run_on_owning_runtime_thread_or_skip(
        "ToggleButton disabled checked colour-priority integration test",
        move || {
            let built = build_ui(
                r#"component DisabledCheckedToggle inherits Window {
    ToggleButton {
        text: "Disabled"
        enabled: false
        checked: true
    }
}"#,
            );

            assert_eq!(built.root.__button_enabled_for_test(), Some(false));
            assert_eq!(built.root.__togglebutton_checked_for_test(), Some(true));
            assert_eq!(
                rgba(read_widget_color(&built.root)),
                (0x40, 0x80, 0x80, 0x80),
                "disabled colour must win over the checked visual cue"
            );
        },
    );
}

fn checked_children(root: &WidgetNode) -> Vec<bool> {
    root.children
        .iter()
        .map(|child| {
            child
                .__togglebutton_checked_for_test()
                .expect("child must be ToggleButton")
        })
        .collect()
}
