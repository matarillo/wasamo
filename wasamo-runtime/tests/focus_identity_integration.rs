//! Mock-free Windows integration evidence for M4-Phase 2 T7's coordinate
//! system (`focus_core::FocusState::remap`, `focus::WindowFocus::rebase`,
//! `focus::rebase_to_current_tree`) and for its click-landing fix
//! (`focus_core::FocusTree::focus_landing`, closing CF-T6-5), built from
//! real `.ui` source through `wasamoc` -> IR -> the loader, driven by real
//! `WM_LBUTTONUP` messages through the real window procedure. Nothing is
//! mocked.
//!
//! # Fixture 1 — a retained focus record survives a structural removal
//! that shifts ids
//!
//! `FocusId` is the pre-order index of `FocusProjection`'s walk over the
//! **whole** widget tree (every node, not only stops), rebuilt fresh per
//! operation; `WindowFocus` retains one across messages. T5's
//! `discard_stale_focus` only ever caught the case where a retained id
//! fell *out of range* after a structural removal — the case where a
//! removal upstream of the retained id shrinks the tree by fewer nodes
//! than the id's own value, leaving the id still *in range* but now naming
//! a *different* node, was recorded as unfireable through T5's own two
//! call sites (CF-T5-1's in-range half). This fixture fires exactly that
//! branch: a conditional subtree with two Buttons sits *before* four
//! trailing Buttons in declaration order, so removing it shifts every
//! trailing Button's id down by the removed count while leaving each one's
//! *identity* — which real widget it is — unchanged.
//!
//! # Fixture 2 — a click inside a `focus-group` focuses the clicked member
//!
//! Before T7, `focus::focus_on_click` built its landing from
//! `FocusTree::tab_stops` plus a nearest-stop search — and a `focus-group`
//! container contributes *itself* to `tab_stops`, with its members not
//! appearing at all. So a click on a widget inside a group moved focus to
//! the **group container**, not the clicked member (CF-T6-5). This fixture
//! clicks two different members of an authored `focus-group` container and
//! asserts each click lands on the member actually clicked.
//!
//! # Helpers copied from `focus_traversal_integration.rs` and
//! `event_routing_integration.rs`
//!
//! Cargo compiles every `tests/*.rs` file as its own crate, so nothing in
//! either file is importable here. `lower_ui_to_ir`, `load_window`,
//! `client_extent`, `display_limits`, `window_rect`, `frame_thickness`,
//! `send_dpi_change_to_client`, `normalise_to_reference_baseline`,
//! `send_click`, `last_error`, `label_of`, `node_at_path`, and
//! `assert_focused_stop` are copied verbatim from
//! `focus_traversal_integration.rs`. `click_and_drain` is copied verbatim
//! from `event_routing_integration.rs` — Fixture 1 needs it so the drain's
//! Phase 2 (`emit::flush_layout`, where `focus::rebase_to_current_tree`
//! runs) actually executes; a plain `send_click` only reaches Phase 1's
//! synchronous Effect drain, never Phase 2 (see the module header of
//! either source file for the full "why" of that split). This file's own
//! additions are [`pre_order_focus_id`] and [`count_nodes`] — a test-side
//! replica of `FocusProjection::project`'s pre-order walk, needed because
//! no ABI seam exposes a raw `FocusId` (the phase's "no new ABI function"
//! obligation) — and the two fixtures themselves.
//!
//! # The client stays small (M4-Phase 1 T8 finding)
//!
//! Every fixture normalises to 96 DPI at an explicitly chosen 360x240
//! physical client — the same ceiling every other integration test file in
//! this crate records (a hosted CI desktop failed a larger request twice).
//! Every click coordinate is derived from `__arranged_rect_for_test()`
//! multiplied by the scale factor the runtime **committed** at that
//! baseline, never from a hand-worked-out constant, and the geometric
//! relationship each fixture depends on is asserted before the point is
//! used.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::ffi::{c_void, CStr};
use std::ptr;

use wasamo_runtime::{ffi, WidgetNode};

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, PostMessageW, PostQuitMessage, SendMessageW,
    SM_CXMAXTRACK, SM_CXSCREEN, SM_CYMAXTRACK, SM_CYSCREEN, WM_DPICHANGED, WM_LBUTTONUP,
};

/// The reference DPI every fixture normalises to before measuring. See
/// `hit_resolution_integration.rs`'s module header (Phase 1 T9 finding
/// F-47) for why this is reached deliberately rather than assumed.
const REFERENCE_DPI: u32 = 96;

/// The physical (== DIP, since no fixture here changes scale) client every
/// fixture normalises to. See this file's module header, "The client stays
/// small".
const CLIENT_W: i32 = 360;
const CLIENT_H: i32 = 240;

// ── Copied verbatim from `focus_traversal_integration.rs` / `event_routing_integration.rs` ──

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<focus-identity-integration>";
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

unsafe fn load_window(ir: &str) -> *mut ffi::WasamoWindow {
    let mut window: *mut ffi::WasamoWindow = ptr::null_mut();
    let status = ffi::wasamo_load_ui(
        ffi::WASAMO_LOAD_MEMORY,
        ir.as_ptr() as *const c_void,
        ir.len(),
        &mut window as *mut *mut ffi::WasamoWindow,
    );
    assert_eq!(status, ffi::WASAMO_OK, "{:?}", window);
    assert!(!window.is_null(), "wasamo_load_ui must return a window");
    window
}

unsafe fn client_extent(hwnd: HWND) -> (i32, i32) {
    let mut rect = RECT::default();
    GetClientRect(hwnd, &mut rect).expect("GetClientRect");
    (rect.right - rect.left, rect.bottom - rect.top)
}

/// Diagnostic-only, printed into a failing assertion so a CI runner that
/// cannot honour a requested client explains itself instead of failing
/// silently on a rectangle nobody chose (M4-Phase 1 T8/T9 precedent).
unsafe fn display_limits() -> (i32, i32, i32, i32) {
    (
        GetSystemMetrics(SM_CXSCREEN),
        GetSystemMetrics(SM_CYSCREEN),
        GetSystemMetrics(SM_CXMAXTRACK),
        GetSystemMetrics(SM_CYMAXTRACK),
    )
}

unsafe fn window_rect(hwnd: HWND) -> RECT {
    let mut rect = RECT::default();
    GetWindowRect(hwnd, &mut rect).expect("GetWindowRect");
    rect
}

/// The non-client frame, as `outer - client` on each axis. Measured live
/// rather than derived from a DPI table.
unsafe fn frame_thickness(hwnd: HWND) -> (i32, i32) {
    let outer = window_rect(hwnd);
    let (cw, ch) = client_extent(hwnd);
    (
        (outer.right - outer.left) - cw,
        (outer.bottom - outer.top) - ch,
    )
}

/// Send a real `WM_DPICHANGED` whose suggested rectangle realises
/// `(client_w, client_h)` physical client pixels at the window's current
/// position.
unsafe fn send_dpi_change_to_client(hwnd: HWND, dpi: u32, client_w: i32, client_h: i32) {
    let (frame_w, frame_h) = frame_thickness(hwnd);
    let outer = window_rect(hwnd);
    let suggested = RECT {
        left: outer.left,
        top: outer.top,
        right: outer.left + client_w + frame_w,
        bottom: outer.top + client_h + frame_h,
    };
    let wparam = WPARAM(((dpi as usize) << 16) | dpi as usize);
    let result = SendMessageW(
        hwnd,
        WM_DPICHANGED,
        wparam,
        LPARAM(&suggested as *const RECT as isize),
    );
    assert_eq!(
        result.0, 0,
        "DD-M4-P1-003 step 5: the handler returns LRESULT(0) whatever happened"
    );
}

/// Put a freshly created window into the before-state every fixture below
/// assumes: cached scale [`REFERENCE_DPI`], physical (and therefore DIP)
/// client `(client_w, client_h)`. Both halves are asserted rather than
/// assumed — see `hit_resolution_integration.rs`'s module header for why.
unsafe fn normalise_to_reference_baseline(
    window: *mut ffi::WasamoWindow,
    client_w: i32,
    client_h: i32,
    what: &str,
) {
    let hwnd = (*window).hwnd;
    send_dpi_change_to_client(hwnd, REFERENCE_DPI, client_w, client_h);
    let limits = display_limits();
    assert_eq!(
        client_extent(hwnd),
        (client_w, client_h),
        "{what}: the realised client extent must be the requested one, or every \
         assertion below is about a rectangle nobody chose; \
         requested_client=({client_w},{client_h}), \
         display=(screen {}x{}, max_track {}x{})",
        limits.0,
        limits.1,
        limits.2,
        limits.3,
    );
    assert_eq!(
        ffi::__window_scale_dpi_for_test(window),
        REFERENCE_DPI,
        "{what}: the baseline scale must be the reference one, or the factors \
         below are taken against the developer's monitor"
    );
}

/// A real `WM_LBUTTONUP` at a **physical** client position, delivered
/// synchronously via `SendMessageW`. Suitable for a click that only needs
/// `focus_on_click` / `hit_test_click`'s synchronous dispatch, not
/// `emit::flush_layout`'s Phase 2.
unsafe fn send_click(hwnd: HWND, x: f32, y: f32) {
    let packed = ((y.round() as i32 as u32) << 16) | (x.round() as i32 as u32 & 0xFFFF);
    SendMessageW(
        hwnd,
        WM_LBUTTONUP,
        WPARAM(0),
        LPARAM(packed as i32 as isize),
    );
}

/// A real `WM_LBUTTONUP` at a **physical** client position, delivered
/// through the production message loop instead of by direct `SendMessageW`.
/// Posts the click, then a synthesised `WM_QUIT`, then pumps
/// `wasamo_runtime::run()` — the same function `wasamo_run` (abi_spec)
/// wraps — until it returns. Fixture 1 needs this: the conditional
/// subtree's removal happens synchronously inside the click's handler
/// (Phase 1's synchronous Effect drain), but `emit::flush_layout` — Phase
/// 2, where `focus::rebase_to_current_tree` runs — only executes at the
/// message-loop boundary this pumps.
///
/// `PostQuitMessage`'s quit flag is only consulted once the thread's
/// regular message queue is empty (documented Win32 ordering), so the
/// posted click is always retrieved, dispatched through `wnd_proc`, and
/// drained by `run`'s post-`DispatchMessageW` call before the loop sees the
/// quit and returns.
unsafe fn click_and_drain(hwnd: HWND, x: f32, y: f32) {
    let packed = ((y.round() as i32 as u32) << 16) | (x.round() as i32 as u32 & 0xFFFF);
    PostMessageW(
        hwnd,
        WM_LBUTTONUP,
        WPARAM(0),
        LPARAM(packed as i32 as isize),
    )
    .expect("PostMessageW(WM_LBUTTONUP)");
    PostQuitMessage(0);
    wasamo_runtime::run();
}

/// Read the last-error message through the C ABI.
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

/// Read a Button-family node's label through the C ABI
/// (`wasamo_get_property`, `PROP_BUTTON_LABEL = 1`). Reading the label back
/// makes "it landed on the widget I meant" a claim about a widget rather
/// than about an index or a path — `focus_traversal_integration.rs`'s
/// module header states the same reasoning.
fn label_of(node: &WidgetNode) -> String {
    const PROP_BUTTON_LABEL: u32 = 1;
    let widget = node as *const WidgetNode as *mut ffi::WasamoWidget;
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

/// Descend `root` by `path` (child indices, root-relative — the same shape
/// `ffi::__focus_path_for_test` returns) and return the node at the end.
fn node_at_path<'a>(root: &'a WidgetNode, path: &[usize]) -> &'a WidgetNode {
    let mut node: &WidgetNode = root;
    for &i in path {
        node = node.children[i].as_ref();
    }
    node
}

/// The paired read-back: the retained record names `expected_path`, the
/// node there paints the indicator, and its label (read through the C
/// ABI, not trusted from tree position) is `expected_label`.
///
/// `previous_path` is deliberately not offered here (unlike
/// `focus_traversal_integration.rs`'s copy of this helper): after a
/// structural mutation, a *path* from before the mutation can itself name
/// a different node afterward — the same shift this file's Fixture 1 is
/// about — so a "the old path stopped painting" check would silently
/// assert something about whichever node the old path happens to name in
/// the *new* tree, not about the node that used to be there.
unsafe fn assert_focused_stop(
    window: *mut ffi::WasamoWindow,
    expected_path: &[usize],
    expected_label: &str,
) {
    assert_eq!(
        ffi::__focus_path_for_test(window),
        Some(expected_path.to_vec()),
        "the retained focus record must name {expected_path:?} ({expected_label:?})"
    );
    let root = (*window).root_widget.as_ref().unwrap();
    let node = node_at_path(root, expected_path);
    assert_eq!(
        node.__button_focused_for_test(),
        Some(true),
        "the node the record names ({expected_path:?}) must paint the focus indicator"
    );
    assert_eq!(
        label_of(node),
        expected_label,
        "the focused node's own label must be {expected_label:?}"
    );
}

// ── This file's own helpers ─────────────────────────────────────────────

/// Every widget node's `FocusId`, replicated from `focus.rs`'s `walk`: a
/// plain pre-order count over `WidgetNode::children`, with no regard to
/// widget kind or focus role — the identical order
/// `FocusProjection::project` assigns ids in (`focus.rs`'s own doc
/// comment: "FocusId is the pre-order index of the walk... by
/// construction"). No ABI seam exposes a raw `FocusId` — that would be a
/// second surface the phase's "no new ABI function" obligation forbids —
/// so this is the only way this fixture can name the exact numeric id a
/// path names, which the fixture's own count-arithmetic assertion needs.
///
/// Panics if `target_path` does not exist in `root`'s subtree, which would
/// be a fixture authoring error rather than something to route around.
fn pre_order_focus_id(root: &WidgetNode, target_path: &[usize]) -> usize {
    fn walk(
        node: &WidgetNode,
        path: &[usize],
        target: &[usize],
        next_id: &mut usize,
    ) -> Option<usize> {
        let id = *next_id;
        *next_id += 1;
        if path == target {
            return Some(id);
        }
        for (i, child) in node.children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(i);
            if let Some(found) = walk(child.as_ref(), &child_path, target, next_id) {
                return Some(found);
            }
        }
        None
    }
    let mut next_id = 0usize;
    walk(root, &[], target_path, &mut next_id)
        .unwrap_or_else(|| panic!("no widget node at path {target_path:?}"))
}

/// The number of nodes in the subtree rooted at `node`, including `node`
/// itself — the same count a structural removal of this subtree takes off
/// the tree's total, and what Fixture 1's "the removal must shrink the
/// tree by fewer nodes than the retained id's value" arithmetic needs.
fn count_nodes(node: &WidgetNode) -> usize {
    1 + node
        .children
        .iter()
        .map(|c| count_nodes(c.as_ref()))
        .sum::<usize>()
}

// ── Fixture 1 — a retained id survives a shifting structural removal ───────

/// Declaration order: a conditional subtree ("cond-a", "cond-b") first,
/// then four trailing Buttons, then a `Box` (not a Button — clicking it
/// must not itself move focus, `docs/dsl_spec.md` §4.19 "clicking
/// background never clears focus") whose handler closes the conditional.
///
/// Pre-order `FocusId`s before the click (every widget node, not only
/// stops): root VStack=0, the conditional's own VStack=1, cond-a=2,
/// cond-b=3, trail-0=4, trail-1=5, trail-2=6, trail-3=7, the closing
/// Box=8. Removing the conditional subtree takes 3 nodes off the tree
/// (ids 1-3), so every trailing Button's id shifts down by 3: trail-1's
/// id 5 becomes 2, landing exactly where "cond-b" used to be numerically
/// — the collision a naive reuse of the old numeric id would produce.
const SHIFTING_REMOVAL_UI: &str = r#"component FocusIdentitySurvivesShiftingRemoval inherits Window {
    state open: bool = true
    VStack {
        spacing: 0
        padding: 0
        if open {
            VStack {
                spacing: 0
                padding: 0
                Button { text: "cond-a" }
                Button { text: "cond-b" }
            }
        }
        Button { text: "trail-0" }
        Button { text: "trail-1" }
        Button { text: "trail-2" }
        Button { text: "trail-3" }
        Box { aspect: 4:1 fill: #336699cc clicked => { root.open = false; } }
    }
}"#;

/// `docs/architecture.md` §13.3 / DD-M4-P2-003 "The focused node
/// disappearing", extended by M4-Phase 2 T7 to the case that section's own
/// mechanism (`discard_stale_focus`) could not detect: a retained id that
/// survives a removal *in range*, but now names a different node.
///
/// Focuses "trail-1" (a trailing Button, unrelated to the subtree about to
/// be removed), then clicks the `Box` — whose handler removes the
/// conditional subtree that precedes every trailing Button in declaration
/// order — through [`click_and_drain`], so `emit::flush_layout`'s Phase 2
/// (where `focus::rebase_to_current_tree` runs) actually executes. The
/// retained record must still name "trail-1" afterward, at its shifted
/// path, and the fixture pins the exact `FocusId` arithmetic that makes
/// this the in-range case rather than the out-of-range one T5 already
/// handled.
#[test]
fn a_retained_focus_record_survives_a_structural_removal_that_shifts_ids() {
    run_on_owning_runtime_thread_or_skip(
        "T7 F1: retained id survives a shifting structural removal",
        move || {
            let ir = lower_ui_to_ir(SHIFTING_REMOVAL_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F1 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "M4-Phase 1 F-47: establish the initial focus state by reading it back"
                );

                // Before-state: the conditional subtree is present, and
                // every label is where this fixture's doc comment says it
                // is.
                let (conditional_removed_count, trail_1_id_before, trail_1_rect, box_rect) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        6,
                        "fixture stopped discriminating: `open` starts true, so the \
                         conditional VStack, four trailing Buttons and the closing Box \
                         must all be present before the click"
                    );
                    let conditional = root.children[0].as_ref();
                    assert_eq!(conditional.child_count(), 2);
                    assert_eq!(label_of(conditional.children[0].as_ref()), "cond-a");
                    assert_eq!(label_of(conditional.children[1].as_ref()), "cond-b");
                    assert_eq!(label_of(root.children[1].as_ref()), "trail-0");
                    assert_eq!(label_of(root.children[2].as_ref()), "trail-1");
                    assert_eq!(label_of(root.children[3].as_ref()), "trail-2");
                    assert_eq!(label_of(root.children[4].as_ref()), "trail-3");

                    let removed = count_nodes(conditional);
                    assert_eq!(
                        removed, 3,
                        "fixture stopped discriminating: the conditional subtree (itself \
                         plus cond-a and cond-b) must be exactly 3 nodes"
                    );
                    let trail_1_id = pre_order_focus_id(root, &[2]);
                    assert_eq!(
                        trail_1_id, 5,
                        "fixture stopped discriminating: trail-1's pre-order FocusId must \
                         be 5 (root=0, conditional VStack=1, cond-a=2, cond-b=3, \
                         trail-0=4, trail-1=5), or the arithmetic this fixture pins below \
                         is not about the case it claims"
                    );
                    (
                        removed,
                        trail_1_id,
                        root.children[2].__arranged_rect_for_test(),
                        root.children[5].__arranged_rect_for_test(),
                    )
                };
                // The load-bearing relationship: the removal must shrink
                // the tree by *fewer* nodes than the retained id's value,
                // or the retained id would fall out of range — the case
                // T5's `discard_stale_focus` already handled — rather than
                // survive in range but pointing at a different node.
                assert!(
                    trail_1_id_before > conditional_removed_count,
                    "fixture stopped discriminating: trail-1's id ({trail_1_id_before}) must \
                     exceed the removed node count ({conditional_removed_count}), or this is \
                     the out-of-range case rather than the in-range one CF-T5-1 named"
                );

                let trail_1_rect = trail_1_rect.expect("trail-1 must be laid out");
                let box_rect = box_rect.expect("the closing Box must be laid out");
                assert!(
                    trail_1_rect.width > 0.0 && trail_1_rect.height > 0.0,
                    "fixture stopped discriminating: trail-1's rectangle {trail_1_rect:?} \
                     must be non-degenerate"
                );
                assert!(
                    box_rect.width > 0.0 && box_rect.height > 0.0,
                    "fixture stopped discriminating: the closing Box's rectangle \
                     {box_rect:?} must be non-degenerate"
                );
                assert!(
                    trail_1_rect.y + trail_1_rect.height <= box_rect.y,
                    "fixture stopped discriminating: trail-1 {trail_1_rect:?} and the \
                     closing Box {box_rect:?} must be disjoint"
                );

                // Focus trail-1 with a plain click (no drain needed: this
                // click's handler does nothing structural).
                let (trail_1_cx, trail_1_cy) = (
                    (trail_1_rect.x + trail_1_rect.width / 2.0) * factor,
                    (trail_1_rect.y + trail_1_rect.height / 2.0) * factor,
                );
                send_click(hwnd, trail_1_cx, trail_1_cy);
                assert_focused_stop(window, &[2], "trail-1");

                // Click the closing Box through `click_and_drain`, so
                // `emit::flush_layout`'s Phase 2 runs. The Box is not
                // focusable, so this click must not itself move focus
                // (`docs/dsl_spec.md` §4.19) — only its handler's
                // structural removal is under test.
                let (box_cx, box_cy) = (
                    (box_rect.x + box_rect.width / 2.0) * factor,
                    (box_rect.y + box_rect.height / 2.0) * factor,
                );
                click_and_drain(hwnd, box_cx, box_cy);

                // After the click: the conditional subtree is gone, and
                // every trailing Button (and the Box) shifted one slot
                // left in `root.children`.
                let (trail_1_id_after, new_path) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        5,
                        "the Box's own state write ('root.open = false') must have removed \
                         the conditional subtree synchronously, or this fixture is not \
                         exercising a removal at all"
                    );
                    assert_eq!(
                        label_of(root.children[1].as_ref()),
                        "trail-1",
                        "trail-1 must now be at index 1 (shifted down from 2)"
                    );
                    (pre_order_focus_id(root, &[1]), vec![1usize])
                };
                assert_eq!(
                    trail_1_id_after, 2,
                    "trail-1's new pre-order FocusId (root=0, trail-0=1, trail-1=2) must be \
                     2, landing exactly where 'cond-b' used to be numerically — the \
                     collision a naive reuse of the old id would produce"
                );
                assert_ne!(
                    trail_1_id_before, trail_1_id_after,
                    "trail-1's id must actually have shifted, or this fixture is not \
                     exercising the coordinate-system hazard it claims to"
                );

                // The claim itself: the retained record still names
                // trail-1, by label, at its new (shifted) path — not
                // "cond-b" (gone), not "trail-2" (what the old path [2]
                // now names), and not `None` (the out-of-range fallback).
                assert_eq!(ffi::__focus_path_for_test(window), Some(new_path));
                assert_focused_stop(window, &[1], "trail-1");

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── Fixture 2 — a click inside a `focus-group` focuses the clicked member ──

/// A `focus-group: true` `HStack` (the annotation's spelling landed at T6;
/// see `tests/focus_annotation_integration.rs`) with three Button members.
const GROUP_MEMBER_CLICK_UI: &str = r#"component FocusGroupClickFocusesTheClickedMember inherits Window {
    VStack {
        spacing: 0
        padding: 0
        HStack {
            spacing: 0
            padding: 0
            focus-group: true
            Button { text: "member-a" }
            Button { text: "member-b" }
            Button { text: "member-c" }
        }
    }
}"#;

/// CF-T6-5: before M4-Phase 2 T7, `focus::focus_on_click` built its
/// landing from `FocusTree::tab_stops` + a nearest-stop search, and a
/// `focus-group` contributes *itself* to `tab_stops` — its members do not
/// appear there at all — so a click on a member moved focus to the group
/// **container**. `focus_core::FocusTree::focus_landing` fixes this by
/// walking the click's ancestor chain against node roles directly, so a
/// member (a `Stop`) is reached before its enclosing group ever is.
///
/// Two legs: clicking the **second** member ("member-b") must focus
/// exactly that member, not the group container (`root.children[0]`,
/// path `[0]`) and not any other member. The **first** member
/// ("member-a") is then clicked too — the agreement leg that
/// distinguishes "the click landed on the member I clicked" from "it
/// always lands on the same one" (e.g. a bug that always resolved to the
/// group's first or remembered member regardless of target).
#[test]
fn a_click_inside_a_focus_group_focuses_the_clicked_member() {
    run_on_owning_runtime_thread_or_skip(
        "T7 F2: a click on a focus-group member focuses that member",
        move || {
            let ir = lower_ui_to_ir(GROUP_MEMBER_CLICK_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F2 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "M4-Phase 1 F-47: establish the initial focus state by reading it back"
                );

                let (member_a_rect, member_b_rect, group_role) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(root.children.len(), 1, "fixture must have one HStack child");
                    let group = root.children[0].as_ref();
                    assert_eq!(group.child_count(), 3, "the group must have three members");
                    assert_eq!(label_of(group.children[0].as_ref()), "member-a");
                    assert_eq!(label_of(group.children[1].as_ref()), "member-b");
                    assert_eq!(label_of(group.children[2].as_ref()), "member-c");
                    (
                        group.children[0].__arranged_rect_for_test(),
                        group.children[1].__arranged_rect_for_test(),
                        group.__focus_role_for_test(),
                    )
                };
                assert_eq!(
                    group_role, "group",
                    "fixture stopped discriminating: the HStack must actually carry \
                     FocusRole::Group, or a click landing on it would prove nothing about \
                     group members specifically"
                );
                let member_a_rect = member_a_rect.expect("member-a must be laid out");
                let member_b_rect = member_b_rect.expect("member-b must be laid out");
                for (label, rect) in [("member-a", member_a_rect), ("member-b", member_b_rect)] {
                    assert!(
                        rect.width > 0.0 && rect.height > 0.0,
                        "fixture stopped discriminating: {label}'s rectangle {rect:?} must \
                         be non-degenerate"
                    );
                }
                assert!(
                    member_a_rect.x + member_a_rect.width <= member_b_rect.x,
                    "fixture stopped discriminating: member-a {member_a_rect:?} and \
                     member-b {member_b_rect:?} must be disjoint (an HStack lays out \
                     left-to-right)"
                );

                // Leg 1: click the second member.
                let (b_cx, b_cy) = (
                    (member_b_rect.x + member_b_rect.width / 2.0) * factor,
                    (member_b_rect.y + member_b_rect.height / 2.0) * factor,
                );
                send_click(hwnd, b_cx, b_cy);
                assert_focused_stop(window, &[0, 1], "member-b");

                // Leg 2 (the agreement leg): click the first member, and
                // it — not member-b again — must be the new landing.
                let (a_cx, a_cy) = (
                    (member_a_rect.x + member_a_rect.width / 2.0) * factor,
                    (member_a_rect.y + member_a_rect.height / 2.0) * factor,
                );
                send_click(hwnd, a_cx, a_cy);
                assert_focused_stop(window, &[0, 0], "member-a");

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}
