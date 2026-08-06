//! M4-Phase 2 pre-ADR spike: the **mechanism fixture**.
//!
//! [framing.md](../../process/milestone-4/phase-2/requirements/framing.md)
//! agreement 5 requires the focus semantics to be demonstrated on something
//! that runs, not only on paper. The unit tests in `focus_core` do that for
//! the traversal logic; this binary does it for the part the unit tests
//! cannot reach — that the core's input can be built from a **real widget
//! tree**, produced by the production `.ui` -> `wasamoc` -> IR -> runtime
//! path, against a live Compositor.
//!
//! # This fixture is not a widget
//!
//! It composes existing material only (`VStack` / `HStack` / `Button` /
//! `ToggleButton` / `Box`). No official widget is created, nothing here is
//! spelled in `docs/dsl_spec.md`, and the group / modal annotations are
//! supplied by the test as an override map rather than by any `.ui` syntax —
//! because no such syntax exists yet, which is the measurement (see
//! `focus_spike`'s module docs). The group spelling is M5's decision and this
//! fixture must not create a fait accompli.
//!
//! # What "landed on the right widget" means here
//!
//! Focus ids are indices into a projection, so asserting on them alone would
//! be circular — it would test the projection against itself. Every assertion
//! therefore reads the **Button label back through the C ABI** from the widget
//! the focus id maps to. A wrong traversal lands on a different real widget
//! and reports a different string.

#![cfg(windows)]

use std::collections::BTreeMap;
use std::ffi::CStr;

use wasamo_runtime::__focus_spike::{
    project, Arrow, FocusRole, FocusState, Projection, TabDirection,
};
use wasamo_runtime::ffi;
use wasamo_runtime::ir_loader::{build_widget_tree, parse_ir};

mod common;

const PROP_BUTTON_LABEL: u32 = 1;

/// The A-shaped fixture: a toolbar whose three tabs form one group, a
/// single view-toggle stop, two thumbnails, and a lightbox subtree that is
/// only reachable once entered.
// `r##` because the `fill` colour literal contains a `#`.
const FIXTURE_UI: &str = r##"component FocusFixture inherits Window {
    VStack {
        HStack {
            ToggleButton { text: "All" }
            ToggleButton { text: "Albums" }
            ToggleButton { text: "Favorites" }
        }
        Button { text: "ViewToggle" }
        HStack {
            Button { text: "Thumb0" }
            Button { text: "Thumb1" }
        }
        Box {
            fill: #101820cc
            VStack {
                Button { text: "Prev" }
                Button { text: "Next" }
            }
        }
    }
}"##;

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<focus-mechanism-fixture>";
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

/// Read a Button-family widget's label through the C ABI.
fn label_of(widget: *mut ffi::WasamoWidget) -> String {
    let mut value = ffi::WasamoValue {
        tag: ffi::WASAMO_VALUE_NONE,
        payload: ffi::WasamoValuePayload { v_i32: 0 },
    };
    let status = unsafe { ffi::wasamo_get_property(widget, PROP_BUTTON_LABEL, &mut value) };
    assert_eq!(
        status,
        ffi::WASAMO_OK,
        "wasamo_get_property(label) failed: {:?}",
        unsafe { last_error() }
    );
    assert_eq!(value.tag, ffi::WASAMO_VALUE_STRING);
    let view = unsafe { value.payload.v_string };
    let bytes = unsafe { std::slice::from_raw_parts(view.ptr as *const u8, view.len) };
    std::str::from_utf8(bytes)
        .expect("label must be UTF-8")
        .to_owned()
}

unsafe fn last_error() -> Option<String> {
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

/// The label of whatever the focus state currently points at. `None` for
/// "nothing focused"; panics if focus landed on a non-Button node, which
/// would itself be a traversal defect.
fn focused_label(projection: &Projection, state: &FocusState) -> Option<String> {
    state
        .focused()
        .map(|id| label_of(projection.widget(id) as *mut ffi::WasamoWidget))
}

/// The focus id of the Button-family node carrying `label`.
///
/// Reads every candidate's label through the ABI rather than trusting a
/// position, because the projection's ids are a function of the tree the IR
/// loader actually built — not of the shape the `.ui` source suggests.
fn id_of_label(projection: &Projection, label: &str) -> usize {
    (0..projection.tree.len())
        .find(|&id| {
            projection.tree.role(id) == FocusRole::Stop
                && label_of(projection.widget(id) as *mut ffi::WasamoWidget) == label
        })
        .unwrap_or_else(|| panic!("no Button-family node labelled {label:?}"))
}

/// Build the live tree, then **derive** the annotation targets from it.
///
/// An earlier version hard-coded the pre-order indices from reading the `.ui`
/// source, and they were wrong by one: `build_widget_tree` returns the root
/// `VStack` itself rather than a separate Window node, so the group override
/// landed on a Button and the modal override on the wrong container. Two of
/// the four tests still passed, because a `VStack` annotated as a modal scope
/// still contains the two buttons the containment assertion looks for.
///
/// Deriving the ids from the built tree removes the class: the group is
/// whatever contains the first tab, and the scope is the `Box` two levels
/// above the first lightbox button. Both are asserted to be non-focusable
/// containers before being annotated, so a structural change to the fixture
/// fails loudly instead of silently annotating a Button.
struct Fixture {
    built: wasamo_runtime::ir_loader::BuiltUi,
    overrides: BTreeMap<usize, FocusRole>,
    group: usize,
    scope: usize,
}

fn build_fixture() -> Fixture {
    let ir = lower_ui_to_ir(FIXTURE_UI);
    let component = parse_ir(&ir).expect("parse_ir failed");
    let mut built = build_widget_tree(
        &component,
        wasamo_runtime::get_compositor(),
        wasamo_runtime::get_text_renderer(),
    )
    .expect("build_widget_tree failed");

    let (group, scope) = {
        let bare = project(&mut built.root, &BTreeMap::new());
        let all = id_of_label(&bare, "All");
        let group = bare.tree.parent(all).expect("the tabs have a parent");

        let prev = id_of_label(&bare, "Prev");
        let inner = bare.tree.parent(prev).expect("Prev has a parent");
        let scope = bare
            .tree
            .parent(inner)
            .expect("the lightbox VStack has a parent");

        assert_eq!(
            bare.tree.role(group),
            FocusRole::Container,
            "the group annotation must land on a container, not on a widget that \
             is already a stop"
        );
        assert_eq!(bare.tree.role(scope), FocusRole::Container);
        (group, scope)
    };

    let mut overrides = BTreeMap::new();
    overrides.insert(group, FocusRole::Group);
    overrides.insert(scope, FocusRole::ModalScope);
    Fixture {
        built,
        overrides,
        group,
        scope,
    }
}

#[test]
fn the_core_traverses_a_tree_built_through_the_production_ui_path() {
    common::run_on_owning_runtime_thread_or_skip(
        "the_core_traverses_a_tree_built_through_the_production_ui_path",
        || {
            let mut fixture = build_fixture();
            let projection = project(&mut fixture.built.root, &fixture.overrides);

            // The derived annotations landed where they were meant to: the
            // group holds the three tabs, the scope holds the two lightbox
            // buttons. Read through the ABI, not asserted on ids.
            assert_eq!(projection.tree.role(fixture.group), FocusRole::Group);
            assert_eq!(projection.tree.role(fixture.scope), FocusRole::ModalScope);
            let members: Vec<String> = projection
                .tree
                .group_members(fixture.group)
                .into_iter()
                .map(|id| label_of(projection.widget(id) as *mut ffi::WasamoWidget))
                .collect();
            assert_eq!(members, vec!["All", "Albums", "Favorites"]);

            let mut state = FocusState::default();
            state.set_focus(&projection.tree, projection.tree.initial_focus(&state));

            // Tab order over the real tree, read back by label. The three tabs
            // are one stop, and the lightbox contributes nothing.
            let mut seen = vec![focused_label(&projection, &state).expect("initial focus")];
            for _ in 0..3 {
                state.set_focus(
                    &projection.tree,
                    projection.tree.tab(&state, TabDirection::Forward),
                );
                seen.push(focused_label(&projection, &state).expect("focus after Tab"));
            }
            assert_eq!(
                seen,
                vec!["All", "ViewToggle", "Thumb0", "Thumb1"],
                "Tab visits one stop per group and never enters the unentered modal subtree"
            );

            // One more Tab wraps to the group, which resolves to its first
            // member because nothing inside it has been focused yet.
            state.set_focus(
                &projection.tree,
                projection.tree.tab(&state, TabDirection::Forward),
            );
            assert_eq!(focused_label(&projection, &state).as_deref(), Some("All"));
        },
    );
}

#[test]
fn arrows_move_inside_the_group_on_the_real_tree() {
    common::run_on_owning_runtime_thread_or_skip(
        "arrows_move_inside_the_group_on_the_real_tree",
        || {
            let mut fixture = build_fixture();
            let projection = project(&mut fixture.built.root, &fixture.overrides);

            let mut state = FocusState::default();
            state.set_focus(&projection.tree, projection.tree.initial_focus(&state));
            assert_eq!(focused_label(&projection, &state).as_deref(), Some("All"));

            for expected in ["Albums", "Favorites", "All"] {
                let outcome = projection.tree.arrow(&state, Arrow::Next);
                state.apply_arrow(&projection.tree, outcome);
                assert_eq!(
                    focused_label(&projection, &state).as_deref(),
                    Some(expected)
                );
            }

            // Leaving the group and coming back lands on the remembered member,
            // not on the first — the roving memory, on a real tree.
            let outcome = projection.tree.arrow(&state, Arrow::Next);
            state.apply_arrow(&projection.tree, outcome);
            assert_eq!(
                focused_label(&projection, &state).as_deref(),
                Some("Albums")
            );
            state.set_focus(
                &projection.tree,
                projection.tree.tab(&state, TabDirection::Forward),
            );
            assert_eq!(
                focused_label(&projection, &state).as_deref(),
                Some("ViewToggle")
            );
            state.set_focus(
                &projection.tree,
                projection.tree.tab(&state, TabDirection::Backward),
            );
            assert_eq!(
                focused_label(&projection, &state).as_deref(),
                Some("Albums"),
                "re-entering the group restored the remembered member"
            );
        },
    );
}

#[test]
fn a_thumbnail_click_enters_the_modal_scope_and_esc_restores_it() {
    common::run_on_owning_runtime_thread_or_skip(
        "a_thumbnail_click_enters_the_modal_scope_and_esc_restores_it",
        || {
            let mut fixture = build_fixture();
            let projection = project(&mut fixture.built.root, &fixture.overrides);
            let lightbox = fixture.scope;

            let mut state = FocusState::default();
            // Focus the thumbnail the way a click would.
            let thumb1 = id_of_label(&projection, "Thumb1");
            assert!(
                projection.tree.tab_stops(&state, 0).contains(&thumb1),
                "Thumb1 must be reachable by Tab before the scope is entered"
            );
            state.set_focus(&projection.tree, Some(thumb1));

            assert!(
                state.enter_modal(&projection.tree, lightbox),
                "the lightbox must carry the scope annotation for entry to be accepted"
            );
            assert_eq!(focused_label(&projection, &state).as_deref(), Some("Prev"));

            // Containment: Tab cycles Prev/Next and never reaches the gallery.
            let mut inside = Vec::new();
            for _ in 0..4 {
                state.set_focus(
                    &projection.tree,
                    projection.tree.tab(&state, TabDirection::Forward),
                );
                inside.push(focused_label(&projection, &state).expect("focus inside the scope"));
            }
            assert_eq!(inside, vec!["Next", "Prev", "Next", "Prev"]);

            // Esc: the core names the scope; the caller decides what closing
            // means. Here closing is exiting, and the focus captured at entry
            // comes back.
            assert_eq!(projection.tree.esc_target(&state), Some(lightbox));
            state.exit_modal(&projection.tree);
            assert_eq!(
                focused_label(&projection, &state).as_deref(),
                Some("Thumb1"),
                "closing the lightbox restored the thumbnail that opened it"
            );
        },
    );
}

#[test]
fn the_widget_kind_alone_cannot_express_the_group_or_the_scope() {
    common::run_on_owning_runtime_thread_or_skip(
        "the_widget_kind_alone_cannot_express_the_group_or_the_scope",
        || {
            let mut fixture = build_fixture();
            // The same tree with **no** annotations: what today's `.ui` can say.
            let bare = project(&mut fixture.built.root, &BTreeMap::new());
            let state = FocusState::default();
            let labels: Vec<String> = bare
                .tree
                .tab_stops(&state, 0)
                .into_iter()
                .map(|id| label_of(bare.widget(id) as *mut ffi::WasamoWidget))
                .collect();
            assert_eq!(
                labels,
                vec![
                    "All",
                    "Albums",
                    "Favorites",
                    "ViewToggle",
                    "Thumb0",
                    "Thumb1",
                    "Prev",
                    "Next"
                ],
                "without authored annotations every Button is its own stop and the \
                 lightbox is tabbable -- the gap DD-M4-P2-005 must close"
            );
        },
    );
}
