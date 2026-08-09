//! Mock-free Windows integration evidence for M4-Phase 2 T7's focus
//! mechanism, driven through real window messages and the real window
//! procedure — the demonstration
//! [framing.md](../../process/milestone-4/phase-2/requirements/framing.md)
//! agreement 5 asks for: the semantics `docs/dsl_spec.md` §4.19 and
//! `docs/architecture.md` §13 state must be shown running, not only
//! pinned by `focus_core.rs`'s unit tests.
//!
//! # Why this file survives T7's retirement of the spike scaffolding
//!
//! Before T6, no `.ui` syntax could say "this container is a group" or
//! "this subtree is modal", so the pre-ADR spike supplied both roles
//! through a test-side override map keyed by pre-order index, and every
//! assertion here went through that map plus a bespoke projection call.
//! T6 landed the authored `focus-group` / `modal-scope` attributes
//! (DD-M4-P2-005) and T7 landed the runtime behaviour that acts on them
//! (`focus::sync_scopes_to_tree`, `arrow_on_key`, `dismiss_on_key`), so
//! the override map now stands in for something that exists:
//! [`FIXTURE_UI`] authors both annotations directly, and the fixture
//! drives them through the production `.ui` -> `wasamoc` -> IR -> loader
//! -> real window procedure path, reading every result back through the
//! seams `modal_scope_integration.rs` already established
//! (`ffi::__focus_path_for_test`, `WidgetNode::__focus_role_for_test`,
//! `__button_focused_for_test`, `__arranged_rect_for_test`) rather than
//! through `crate::focus`, which is private — the phase's cross-task
//! obligation is no new ABI function
//! (`process/milestone-4/phase-2/requirements/constraints.md` §2).
//!
//! # What this file is for, and what it is not
//!
//! `modal_scope_integration.rs` already covers group traversal and modal
//! entry/exit as separate fixtures, each isolating one concern. This
//! file's value after the re-point is the **combination**: Tab order
//! across a tree that has both a `focus-group` and a `modal-scope` at
//! once, arrows moving inside the group, and a click that enters the
//! scope followed by an Escape that restores out of it — all inside one
//! tree, end to end. Three tests carry that:
//!
//! - [`tab_order_covers_the_group_and_the_ungrouped_stops_on_the_real_tree`]
//!   — Tab visits the group once and the two ungrouped Buttons, in
//!   declaration order, wrapping back to the group; the unmaterialised
//!   scope contributes nothing.
//! - [`arrows_move_inside_the_group_and_group_memory_survives_a_visit_outside`]
//!   — arrow keys move within the group and wrap; leaving the group by
//!   Tab and returning by Shift+Tab lands on the remembered member, not
//!   the first.
//! - [`a_thumbnail_click_enters_the_modal_scope_and_escape_restores_it`]
//!   — clicking a thumbnail opens the (conditionally materialised) scope
//!   through the production drain (DD-M4-P2-004 "presence is the
//!   entry"), entry lands on the scope's first stop, Tab is confined to
//!   it, and Escape's authored `dismiss` handler removes the subtree and
//!   restores the focus captured at entry.
//!
//! **A fourth test is deleted, not carried over.** The pre-repoint file
//! had `the_widget_kind_alone_cannot_express_the_group_or_the_scope`,
//! which projected the same tree with **no** annotations and asserted
//! every Button was its own stop — a measurement of the gap
//! DD-M4-P2-005 had to close. That gap is closed: the annotations exist
//! and [`FIXTURE_UI`] authors them, so there is no longer an unannotated
//! variant of this tree to build. The property that test was really
//! guarding — that an *unannotated* subtree does not confine traversal —
//! is the spike's S-3 leg, and it is pinned for the production path by
//! `modal_scope_integration.rs`'s
//! `a_present_but_unannotated_subtree_does_not_confine`, which this file
//! does not duplicate.
//!
//! # What "landed on the right widget" means here
//!
//! Every assertion goes through [`assert_focused_stop`], which checks
//! three things together: the retained focus record names the expected
//! path, the node there paints the focus indicator, and — read back
//! through the C ABI, not trusted from tree position — that node's own
//! label is the expected one. A wrong traversal lands the record on a
//! *different* real widget and reports a different label; a bare path
//! assertion could not tell the two cases apart.
//!
//! # Helpers copied from `modal_scope_integration.rs`
//!
//! Cargo compiles every `tests/*.rs` file as its own crate
//! (`modal_scope_integration.rs`'s own module header records this
//! convention), so nothing in that file is importable here.
//! `lower_ui_to_ir`, `load_window`, `client_extent`, `display_limits`,
//! `window_rect`, `frame_thickness`, `send_dpi_change_to_client`,
//! `normalise_to_reference_baseline`, `click_and_drain`, `key_and_drain`,
//! `last_error`, `label_of`, `node_at_path`, `assert_focused_stop`,
//! `send_key`, `send_tab`, and `send_shift_tab` are copied verbatim from
//! there.
//!
//! # The client stays small (M4-Phase 1 T8 finding)
//!
//! Every fixture normalises to 96 DPI at an explicitly chosen 360x240
//! physical client — the same ceiling every other integration test file
//! in this crate records (a hosted CI desktop failed a larger request
//! twice). Every click coordinate is derived from
//! `__arranged_rect_for_test()` multiplied by the scale factor the
//! runtime **committed** at that baseline, never from a hand-worked-out
//! constant.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::ffi::{c_void, CStr};
use std::ptr;

use wasamo_runtime::{ffi, WidgetNode};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardState, SetKeyboardState, VK_ESCAPE, VK_RIGHT, VK_SHIFT, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, PostMessageW, PostQuitMessage, SendMessageW,
    SM_CXMAXTRACK, SM_CXSCREEN, SM_CYMAXTRACK, SM_CYSCREEN, WM_DPICHANGED, WM_KEYDOWN,
    WM_LBUTTONUP,
};

/// The reference DPI this fixture normalises to before measuring. See
/// `hit_resolution_integration.rs`'s module header (Phase 1 T9 finding
/// F-47) for why this is reached deliberately rather than assumed.
const REFERENCE_DPI: u32 = 96;

/// The physical (== DIP, since no fixture here changes scale) client this
/// fixture normalises to. See this file's module header, "The client
/// stays small".
const CLIENT_W: i32 = 360;
const CLIENT_H: i32 = 240;

// ── Copied verbatim from `modal_scope_integration.rs` ───────────────────

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
/// through the production message loop instead of by direct `SendMessageW`.
/// Posts the click, then a synthesised `WM_QUIT`, then pumps
/// `wasamo_runtime::run()` — the same function `wasamo_run` (abi_spec)
/// wraps — until it returns. Needed whenever a click's handler mutates
/// state structurally: the removal/materialisation happens synchronously
/// inside the click's handler (Phase 1's synchronous Effect drain), but
/// `emit::flush_layout` — Phase 2, where `focus::sync_scopes_to_tree` runs
/// — only executes at the message-loop boundary this pumps.
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

/// A `WM_KEYDOWN` analogue of [`click_and_drain`], for the same reason —
/// `focus::dismiss_on_key` can run a `dismiss` handler whose state write
/// removes the scope's subtree synchronously (Phase 1), but
/// `sync_scopes_to_tree`'s exit-restoration only runs at Phase 2, which
/// needs the message-loop boundary this pumps. `lparam` is `0`: nothing in
/// `window.rs`'s `WM_KEYDOWN` arm reads the repeat count / scan code /
/// extended-key bits, only `wparam` (the virtual-key code).
unsafe fn key_and_drain(hwnd: HWND, vk: u16) {
    PostMessageW(hwnd, WM_KEYDOWN, WPARAM(vk as usize), LPARAM(0))
        .expect("PostMessageW(WM_KEYDOWN)");
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
/// than about an index or a path. Works for `ToggleButton` too: both share
/// the same button data and the same property id.
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

/// A real `WM_KEYDOWN` for an arbitrary virtual-key code, with no modifier
/// state manipulated.
unsafe fn send_key(hwnd: HWND, vk: u16) -> LRESULT {
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(vk as usize), LPARAM(0))
}

/// A real `WM_KEYDOWN(VK_TAB)` with no Shift held.
unsafe fn send_tab(hwnd: HWND) -> LRESULT {
    send_key(hwnd, VK_TAB.0)
}

/// A real `WM_KEYDOWN(VK_TAB)` with Shift held, following the read-back-
/// before-send, restore-after discipline `focus_traversal_integration.rs`'s
/// module header records (M4-Phase 1 F-49).
unsafe fn send_shift_tab(hwnd: HWND) -> LRESULT {
    let mut table = [0u8; 256];
    GetKeyboardState(&mut table).expect("GetKeyboardState");
    let saved = table;
    table[VK_SHIFT.0 as usize] |= 0x80;
    SetKeyboardState(&table).expect("SetKeyboardState");
    assert!(
        (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0,
        "VK_SHIFT's high bit must actually read back set before a Shift+Tab \
         assertion is allowed to rely on it"
    );
    let result = send_key(hwnd, VK_TAB.0);
    SetKeyboardState(&saved).expect("SetKeyboardState restore");
    result
}

// ── This fixture's tree ─────────────────────────────────────────────────

/// The A-shaped fixture: a toolbar whose three tabs form one authored
/// `focus-group`, a single ungrouped view-toggle stop, two thumbnails
/// (one wired to open the lightbox), and a lightbox subtree that
/// materialises only on click and carries the authored `modal-scope`
/// annotation plus its `dismiss` handler.
///
/// **This fixture is not a shipped widget.** It composes existing
/// material only (`VStack` / `HStack` / `Button` / `ToggleButton`); no
/// official widget is created and nothing here is spelled in
/// `docs/dsl_spec.md` beyond the `focus-group` / `modal-scope` /
/// `dismiss` attributes T6 already authored there.
const FIXTURE_UI: &str = r#"component FocusMechanismFixture inherits Window {
    state lightbox_open: bool = false
    VStack {
        spacing: 0
        padding: 0
        HStack {
            spacing: 0
            padding: 0
            focus-group: true
            ToggleButton { text: "All" }
            ToggleButton { text: "Albums" }
            ToggleButton { text: "Favorites" }
        }
        Button { text: "ViewToggle" }
        HStack {
            spacing: 0
            padding: 0
            Button { text: "Thumb0" }
            Button { text: "Thumb1" clicked => { root.lightbox_open = true; } }
        }
        if lightbox_open {
            VStack {
                spacing: 0
                padding: 0
                modal-scope: true
                dismiss => { root.lightbox_open = false; }
                Button { text: "Prev" }
                Button { text: "Next" }
            }
        }
    }
}"#;

// ── Test 1 — Tab order over the group and the ungrouped stops ──────────

#[test]
fn tab_order_covers_the_group_and_the_ungrouped_stops_on_the_real_tree() {
    run_on_owning_runtime_thread_or_skip(
        "focus mechanism: Tab order over the group and the ungrouped stops",
        move || {
            let ir = lower_ui_to_ir(FIXTURE_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F1 baseline");

                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "nothing is focused at window open (docs/dsl_spec.md §4.19)"
                );
                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        3,
                        "fixture stopped discriminating: `lightbox_open` starts false, so the \
                         scope must not be materialised yet"
                    );
                    assert_eq!(
                        root.children[0].__focus_role_for_test(),
                        "group",
                        "fixture stopped discriminating: the tabs container must actually \
                         carry FocusRole::Group"
                    );
                }

                // Tab visits one stop per group and the two ungrouped
                // Buttons, in declaration order; the unmaterialised scope
                // contributes nothing.
                send_tab(hwnd);
                assert_focused_stop(window, &[0, 0], "All");
                send_tab(hwnd);
                assert_focused_stop(window, &[1], "ViewToggle");
                send_tab(hwnd);
                assert_focused_stop(window, &[2, 0], "Thumb0");
                send_tab(hwnd);
                assert_focused_stop(window, &[2, 1], "Thumb1");

                // One more Tab wraps to the group, landing on its first
                // member.
                send_tab(hwnd);
                assert_focused_stop(window, &[0, 0], "All");

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── Test 2 — arrows inside the group, group memory across a Tab out ────

#[test]
fn arrows_move_inside_the_group_and_group_memory_survives_a_visit_outside() {
    run_on_owning_runtime_thread_or_skip(
        "focus mechanism: arrows inside the group, group memory across a Tab out and back",
        move || {
            let ir = lower_ui_to_ir(FIXTURE_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F2 baseline");

                send_tab(hwnd);
                assert_focused_stop(window, &[0, 0], "All");

                // Arrow-Right steps through the group's members and wraps.
                let result = send_key(hwnd, VK_RIGHT.0);
                assert_eq!(result.0, 0, "an arrow the group handles must be consumed");
                assert_focused_stop(window, &[0, 1], "Albums");

                let result = send_key(hwnd, VK_RIGHT.0);
                assert_eq!(result.0, 0);
                assert_focused_stop(window, &[0, 2], "Favorites");

                let result = send_key(hwnd, VK_RIGHT.0);
                assert_eq!(result.0, 0);
                assert_focused_stop(window, &[0, 0], "All");

                let result = send_key(hwnd, VK_RIGHT.0);
                assert_eq!(result.0, 0);
                assert_focused_stop(window, &[0, 1], "Albums");

                // Leaving the group and coming back lands on the
                // remembered member, not on the first — the roving
                // memory, on a real tree driven end to end.
                send_tab(hwnd);
                assert_focused_stop(window, &[1], "ViewToggle");
                send_shift_tab(hwnd);
                assert_focused_stop(window, &[0, 1], "Albums");

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── Test 3 — a thumbnail click enters the scope, Escape restores it ────

#[test]
fn a_thumbnail_click_enters_the_modal_scope_and_escape_restores_it() {
    run_on_owning_runtime_thread_or_skip(
        "focus mechanism: a thumbnail click enters the modal scope, Escape restores it",
        move || {
            let ir = lower_ui_to_ir(FIXTURE_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F3 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                assert_eq!(ffi::__focus_path_for_test(window), None);

                let thumb1_rect = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        3,
                        "the scope must not be materialised yet"
                    );
                    root.children[2].children[1].__arranged_rect_for_test()
                }
                .expect("Thumb1 must be laid out");
                assert!(thumb1_rect.width > 0.0 && thumb1_rect.height > 0.0);

                // Click Thumb1: focuses it first (the click's own
                // focus-before-dispatch order), then its `clicked` handler
                // sets `lightbox_open = true`. `click_and_drain` is
                // required from here: the scope's materialisation and its
                // entry (DD-M4-P2-004 "presence is the entry") both need
                // Phase 2.
                let (tx, ty) = (
                    (thumb1_rect.x + thumb1_rect.width / 2.0) * factor,
                    (thumb1_rect.y + thumb1_rect.height / 2.0) * factor,
                );
                click_and_drain(hwnd, tx, ty);

                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        4,
                        "the click's handler must have materialised the scope's subtree"
                    );
                    let scope = root.children[3].as_ref();
                    assert_eq!(
                        scope.__focus_role_for_test(),
                        "modal-scope",
                        "fixture stopped discriminating: the materialised container must \
                         actually carry FocusRole::ModalScope"
                    );
                    assert_eq!(label_of(scope.children[0].as_ref()), "Prev");
                    assert_eq!(label_of(scope.children[1].as_ref()), "Next");
                }

                // Entry moved focus to the scope's first stop.
                assert_focused_stop(window, &[3, 0], "Prev");

                // Containment: Tab cycles Prev/Next and never reaches the
                // rest of the tree — the group, ViewToggle, the
                // thumbnails.
                send_tab(hwnd);
                assert_focused_stop(window, &[3, 1], "Next");
                send_tab(hwnd);
                assert_focused_stop(window, &[3, 0], "Prev");
                send_tab(hwnd);
                assert_focused_stop(window, &[3, 1], "Next");
                send_tab(hwnd);
                assert_focused_stop(window, &[3, 0], "Prev");

                // Escape: the authored `dismiss` handler runs, removing
                // the scope's subtree, and exit restoration writes back
                // the focus captured at entry — Thumb1, the widget the
                // opening click landed on, not the domain's first stop.
                key_and_drain(hwnd, VK_ESCAPE.0);

                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        3,
                        "the dismiss handler's own state write \
                         ('root.lightbox_open = false') must have removed the scope's \
                         subtree synchronously"
                    );
                }
                assert_focused_stop(window, &[2, 1], "Thumb1");

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}
