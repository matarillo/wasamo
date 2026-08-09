//! Mock-free Windows integration evidence for M4-Phase 2 T7's modal-scope
//! seam (`focus::sync_scopes_to_tree`), arrow-key routing
//! (`focus::arrow_on_key`), and Escape-as-dismissal (`focus::dismiss_on_key`)
//! — built from real `.ui` source through `wasamoc` -> IR -> the loader,
//! driven by real `WM_KEYDOWN` / `WM_LBUTTONUP` messages through the real
//! window procedure. Nothing is mocked.
//!
//! # What each fixture is for
//!
//! - [`entry_is_driven_by_the_production_seam`] (Fixture 1) — a scope that
//!   materialises later (an `if` going true) is entered through
//!   `sync_scopes_to_tree`'s call site in `emit::flush_layout`: focus lands
//!   on the scope's first stop, and Tab / Shift+Tab cycle only within it —
//!   `docs/architecture.md` §13.4's confinement, exercised through the real
//!   drain rather than by calling `focus_core` directly.
//! - [`entry_at_initial_build`] (Fixture 2) — a scope present with no `if`
//!   at all is entered through `sync_scopes_to_tree`'s *other* call site,
//!   `window::set_root`, with no input at all — DD-M4-P2-004's "a scope in
//!   the initial tree is entered at startup".
//! - [`exit_restores_and_restoration_beats_succession`] (Fixture 3) —
//!   closing a scope by removing its subtree restores the node that was
//!   focused *before the scope opened*, not the domain's first stop, which
//!   is why the fixture deliberately focuses a button that is not the
//!   first stop (see the fixture's own doc comment for why that choice is
//!   load-bearing).
//! - [`a_present_but_unannotated_subtree_does_not_confine`] (Fixture 4) —
//!   the spike's S-3 leg, named explicitly by `plan.md`: a subtree that
//!   materialises without `modal-scope: true` is an ordinary container, so
//!   `sync_scopes_to_tree`'s entry walk (which filters on
//!   `FocusRole::ModalScope`) must not enter it at all.
//! - [`escapes_two_legs`] (Fixture 5) — Escape is consumed by
//!   `dismiss_on_key` exactly when a scope is entered (whether or not it
//!   carries a `dismiss` handler) and falls through to the host key slot
//!   otherwise.
//! - [`arrow_keys_two_legs`] (Fixture 6) — arrow keys are consumed by
//!   `arrow_on_key` while focus is inside a `focus-group`, and reach the
//!   host key slot everywhere else — the agreement leg without which
//!   "consumed" would be indistinguishable from "did nothing".
//! - [`exit_with_no_restore_target_leaves_focus_unset`] (Fixture 7) —
//!   `sync_scopes_to_tree`'s exit branch with `restore_to == None`
//!   (M4-Phase 2 T7 remediation, B2): a scope entered while nothing was
//!   focused captures `None`, and closing it must leave focus unset rather
//!   than falling to the domain's first stop. Reuses Fixture 2's "entered
//!   with nothing focused" shape and adds the missing leg, closing it.
//!
//! # Helpers copied from `focus_identity_integration.rs` and
//! `focus_traversal_integration.rs`
//!
//! Cargo compiles every `tests/*.rs` file as its own crate, so nothing in
//! either file is importable here (a convention `focus_identity_integration.rs`'s
//! own module header already records). `lower_ui_to_ir`, `load_window`,
//! `client_extent`, `display_limits`, `window_rect`, `frame_thickness`,
//! `send_dpi_change_to_client`, `normalise_to_reference_baseline`,
//! `send_click`, `click_and_drain`, `last_error`, `label_of`, `node_at_path`,
//! and `assert_focused_stop` are copied verbatim from
//! `focus_identity_integration.rs`. `send_key`, `send_tab`,
//! `send_shift_tab`, and `install_key_slot_recorder` are copied verbatim
//! from `focus_traversal_integration.rs`. This file's own addition is
//! [`key_and_drain`] — a `WM_KEYDOWN` analogue of `click_and_drain`, needed
//! so the drain's Phase 2 (`emit::flush_layout`, where
//! `focus::sync_scopes_to_tree` runs) actually executes for a key-triggered
//! state write, the same reason `click_and_drain` exists for a
//! click-triggered one — and the six fixtures themselves.
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

use std::cell::RefCell;
use std::ffi::{c_void, CStr};
use std::ptr;
use std::rc::Rc;

use wasamo_runtime::{ffi, WidgetNode};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardState, SetKeyboardState, VK_DOWN, VK_ESCAPE, VK_LEFT, VK_RIGHT,
    VK_SHIFT, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, PostMessageW, PostQuitMessage, SendMessageW,
    SM_CXMAXTRACK, SM_CXSCREEN, SM_CYMAXTRACK, SM_CYSCREEN, WM_DPICHANGED, WM_KEYDOWN,
    WM_LBUTTONUP,
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

// ── Copied verbatim from `focus_identity_integration.rs` ───────────────────

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<modal-scope-integration>";
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

/// This file's own addition: a `WM_KEYDOWN` analogue of [`click_and_drain`],
/// for the same reason — `focus::dismiss_on_key` can run a `dismiss`
/// handler whose state write removes the scope's subtree synchronously
/// (Phase 1), but `sync_scopes_to_tree`'s exit-restoration only runs at
/// Phase 2, which needs the message-loop boundary this pumps. `lparam` is
/// `0`: nothing in `window.rs`'s `WM_KEYDOWN` arm reads the repeat
/// count / scan code / extended-key bits, only `wparam` (the virtual-key
/// code) — the same simplification `focus_traversal_integration.rs`'s
/// `send_key` already makes for the synchronous case.
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
/// than about an index or a path.
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

// ── Copied verbatim from `focus_traversal_integration.rs` ──────────────────

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

/// Install a recorder in the window's **host key slot** and hand back the
/// list it appends to. `window.rs`'s `WM_KEYDOWN` arm calls `key_down_fn`
/// only after traversal, arrow routing, and dismissal have all declined the
/// key, so what this list contains after a message is the direct answer to
/// "did the focus machinery consume it" — installing a recorder in a
/// pre-existing `pub` field is not shipping an installer
/// (`focus_traversal_integration.rs`'s module header states this in full;
/// `process/milestone-4/phase-2/requirements/constraints.md` §2 is about a
/// *host-facing* installer, which this is not).
unsafe fn install_key_slot_recorder(window: *mut ffi::WasamoWindow) -> Rc<RefCell<Vec<u16>>> {
    let seen: Rc<RefCell<Vec<u16>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    (*window).key_down_fn = Some(Box::new(move |vk| sink.borrow_mut().push(vk)));
    seen
}

// ── Fixture 1 / 3's shared shape ────────────────────────────────────────────

/// Two outside Buttons, then a conditionally-materialised `modal-scope`
/// container with two Buttons of its own and a `dismiss` handler. Clicking
/// "outside-b" both focuses it (a click's own focus-before-dispatch order,
/// `window.rs`'s `WM_LBUTTONUP` arm) and, through its own handler, opens
/// the scope — which is exactly what lets Fixture 3 read "focus a named
/// Button outside the scope, then open it" off one click rather than two.
const SCOPE_UI: &str = r#"component ModalScopeFixture inherits Window {
    state open: bool = false
    VStack {
        spacing: 0
        padding: 0
        Button { text: "outside-a" }
        Button { text: "outside-b" clicked => { root.open = true; } }
        if open {
            VStack {
                spacing: 0
                padding: 0
                modal-scope: true
                dismiss => { root.open = false; }
                Button { text: "in-a" }
                Button { text: "in-b" }
            }
        }
    }
}"#;

// ── Fixture 1 — entry driven by the production seam ─────────────────────────

/// `docs/architecture.md` §13.4: entry "pushes the scope onto the
/// per-window stack..., captures the currently focused node..., and moves
/// focus to the scope's first stop." This fixture drives that through the
/// real reactive drain rather than by calling `focus_core` or
/// `sync_scopes_to_tree` directly: a click opens the scope, and the
/// assertions are read back through the C ABI / the retained focus record.
#[test]
fn entry_is_driven_by_the_production_seam() {
    run_on_owning_runtime_thread_or_skip(
        "modal scope: entry driven by the production seam",
        move || {
            let ir = lower_ui_to_ir(SCOPE_UI);
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

                let (outside_a_rect, outside_b_rect) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        2,
                        "fixture stopped discriminating: `open` starts false, so only the \
                         two outside Buttons must be present before the click"
                    );
                    assert_eq!(label_of(root.children[0].as_ref()), "outside-a");
                    assert_eq!(label_of(root.children[1].as_ref()), "outside-b");
                    (
                        root.children[0].__arranged_rect_for_test(),
                        root.children[1].__arranged_rect_for_test(),
                    )
                };
                let outside_a_rect = outside_a_rect.expect("outside-a must be laid out");
                let outside_b_rect = outside_b_rect.expect("outside-b must be laid out");
                for (label, rect) in [("outside-a", outside_a_rect), ("outside-b", outside_b_rect)]
                {
                    assert!(
                        rect.width > 0.0 && rect.height > 0.0,
                        "fixture stopped discriminating: {label}'s rectangle {rect:?} must be \
                         non-degenerate"
                    );
                }

                // Establish the before-state: focus outside-a, and read it
                // back by label — the requirement this brief names
                // explicitly ("focus somewhere outside, asserted by
                // label").
                let (a_cx, a_cy) = (
                    (outside_a_rect.x + outside_a_rect.width / 2.0) * factor,
                    (outside_a_rect.y + outside_a_rect.height / 2.0) * factor,
                );
                send_click(hwnd, a_cx, a_cy);
                assert_focused_stop(window, &[0], "outside-a");

                // Click outside-b: focuses it first (the click's own
                // focus-before-dispatch order), then its handler sets
                // `open = true`. `click_and_drain` is required from here:
                // the scope's materialisation and entry both need Phase 2.
                let (b_cx, b_cy) = (
                    (outside_b_rect.x + outside_b_rect.width / 2.0) * factor,
                    (outside_b_rect.y + outside_b_rect.height / 2.0) * factor,
                );
                click_and_drain(hwnd, b_cx, b_cy);

                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        3,
                        "the click's handler must have materialised the scope's subtree"
                    );
                    let scope = root.children[2].as_ref();
                    assert_eq!(
                        scope.__focus_role_for_test(),
                        "modal-scope",
                        "fixture stopped discriminating: the materialised container must \
                         actually carry FocusRole::ModalScope"
                    );
                    assert_eq!(label_of(scope.children[0].as_ref()), "in-a");
                    assert_eq!(label_of(scope.children[1].as_ref()), "in-b");
                }

                // Entry moved focus to the scope's first stop.
                assert_focused_stop(window, &[2, 0], "in-a");

                // Tab / Shift+Tab cycle only between in-a and in-b — never
                // the outside Buttons. Four forward Tabs alternate; the
                // path never leaves the scope's two indices.
                for expected in [
                    ([2usize, 1usize], "in-b"),
                    ([2, 0], "in-a"),
                    ([2, 1], "in-b"),
                    ([2, 0], "in-a"),
                ] {
                    send_tab(hwnd);
                    assert_focused_stop(window, &expected.0, expected.1);
                }
                // Shift+Tab: backward within the same confined domain.
                send_shift_tab(hwnd);
                assert_focused_stop(window, &[2, 1], "in-b");
                send_shift_tab(hwnd);
                assert_focused_stop(window, &[2, 0], "in-a");

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── Fixture 2 — entry at the initial build ──────────────────────────────────

const INITIAL_SCOPE_UI: &str = r#"component ModalScopeAtInitialBuild inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "outside" }
        VStack {
            spacing: 0
            padding: 0
            modal-scope: true
            Button { text: "scope-a" }
            Button { text: "scope-b" }
        }
    }
}"#;

/// DD-M4-P2-004: "a scope in the initial tree is entered at startup" —
/// recorded there as behaviour rather than an accident. No `if`, no click,
/// no key: the scope is present from the first build, so
/// `window::set_root`'s `sync_scopes_to_tree` call is the only seam that
/// could have entered it, and this fixture reads the result back
/// immediately after `load_window` returns, before any input at all.
#[test]
fn entry_at_initial_build() {
    run_on_owning_runtime_thread_or_skip("modal scope: entry at initial build", move || {
        let ir = lower_ui_to_ir(INITIAL_SCOPE_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;

            // No `normalise_to_reference_baseline` before this read-back:
            // the point is that entry has already happened by the time
            // `load_window` returns, with nothing done to the window yet.
            {
                let root = (*window).root_widget.as_ref().expect("content root");
                assert_eq!(root.children.len(), 2, "outside Button + scope container");
                assert_eq!(label_of(root.children[0].as_ref()), "outside");
                assert_eq!(
                    root.children[1].__focus_role_for_test(),
                    "modal-scope",
                    "fixture stopped discriminating: the second child must actually carry \
                     FocusRole::ModalScope"
                );
                assert_eq!(label_of(root.children[1].children[0].as_ref()), "scope-a");
            }
            assert_focused_stop(window, &[1, 0], "scope-a");

            // Confinement holds from startup too: Tab never reaches
            // "outside".
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F2 baseline");
            send_tab(hwnd);
            assert_focused_stop(window, &[1, 1], "scope-b");
            send_tab(hwnd);
            assert_focused_stop(window, &[1, 0], "scope-a");

            ffi::wasamo_window_destroy(window);
        }
    });
}

// ── Fixture 3 — exit restores, and restoration beats succession ────────────

/// Continues Fixture 1's shape: the same [`SCOPE_UI`], focusing "outside-a"
/// first (the before-state), then clicking "outside-b" (which focuses it
/// and opens the scope in the same click — Fixture 1's own doc comment).
/// The scope's `restore_to` capture therefore names **outside-b**, not
/// outside-a.
///
/// **Why outside-b, not outside-a, is what makes this a restoration test.**
/// The domain's first stop (what structural succession would answer,
/// `focus_core::FocusTree::initial_focus`) is "outside-a" — the first
/// declared stop in the whole tree. If this fixture captured outside-a as
/// the restore target too, a runtime that answered with succession instead
/// of restoration would pass by coincidence. Capturing outside-b instead —
/// the *second* declared stop, deliberately not the first — makes the two
/// answers disagree, so only the correct rule (restoration precedence,
/// `docs/architecture.md` §13.4) can pass this assertion.
#[test]
fn exit_restores_and_restoration_beats_succession() {
    run_on_owning_runtime_thread_or_skip(
        "modal scope: exit restores, restoration beats succession",
        move || {
            let ir = lower_ui_to_ir(SCOPE_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F3 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                assert_eq!(ffi::__focus_path_for_test(window), None);

                let outside_b_rect = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    root.children[1].__arranged_rect_for_test()
                }
                .expect("outside-b must be laid out");
                assert!(outside_b_rect.width > 0.0 && outside_b_rect.height > 0.0);

                // Click outside-b: focuses it (the restore target the scope
                // will capture), then opens the scope, whose entry moves
                // focus to in-a.
                let (b_cx, b_cy) = (
                    (outside_b_rect.x + outside_b_rect.width / 2.0) * factor,
                    (outside_b_rect.y + outside_b_rect.height / 2.0) * factor,
                );
                click_and_drain(hwnd, b_cx, b_cy);
                assert_focused_stop(window, &[2, 0], "in-a");

                // Escape: the authored `dismiss` handler sets `open =
                // false`, the subtree is removed, and Phase 2 reconciles
                // focus.
                key_and_drain(hwnd, VK_ESCAPE.0);

                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        2,
                        "the dismiss handler's own state write ('root.open = false') must \
                         have removed the scope's subtree synchronously"
                    );
                }
                // Restoration: back on outside-b, not outside-a (the
                // domain's first stop — see this fixture's own doc
                // comment for why the two must differ here).
                assert_focused_stop(window, &[1], "outside-b");

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── Fixture 4 — a present but unannotated subtree does not confine ─────────

/// Identical to [`SCOPE_UI`] with `modal-scope: true` (and therefore
/// `dismiss`, which is admitted only beside it) removed — the spike's S-3
/// leg, named explicitly by `plan.md`.
///
/// **Why a conditional-subtree-removal test cannot stand in for this leg.**
/// Every other fixture in this file exercises a subtree *disappearing*
/// (Fixture 3's Escape, or simply never opening). None of those can show
/// whether a subtree that *appears* without the annotation was silently
/// entered anyway: entry and removal are different seams
/// (`sync_scopes_to_tree`'s step 5 vs. its exit steps 3/4), and a
/// container this loop's `FocusRole::ModalScope` filter correctly skips
/// produces no observable difference from one it wrongly entered *unless*
/// something is asserted about the moment right after it appears — which
/// is exactly what this fixture does and a removal-only fixture cannot.
const UNANNOTATED_UI: &str = r#"component PresentButUnannotatedSubtreeDoesNotConfine inherits Window {
    state open: bool = false
    VStack {
        spacing: 0
        padding: 0
        Button { text: "outside-a" }
        Button { text: "outside-b" clicked => { root.open = true; } }
        if open {
            VStack {
                spacing: 0
                padding: 0
                Button { text: "in-a" }
                Button { text: "in-b" }
            }
        }
    }
}"#;

#[test]
fn a_present_but_unannotated_subtree_does_not_confine() {
    run_on_owning_runtime_thread_or_skip(
        "modal scope: a present but unannotated subtree does not confine",
        move || {
            let ir = lower_ui_to_ir(UNANNOTATED_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F4 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                assert_eq!(ffi::__focus_path_for_test(window), None);

                let outside_b_rect = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    root.children[1].__arranged_rect_for_test()
                }
                .expect("outside-b must be laid out");
                assert!(outside_b_rect.width > 0.0 && outside_b_rect.height > 0.0);

                let (b_cx, b_cy) = (
                    (outside_b_rect.x + outside_b_rect.width / 2.0) * factor,
                    (outside_b_rect.y + outside_b_rect.height / 2.0) * factor,
                );
                click_and_drain(hwnd, b_cx, b_cy);

                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        3,
                        "the container must have materialised"
                    );
                    assert_eq!(
                        root.children[2].__focus_role_for_test(),
                        "container",
                        "fixture stopped discriminating: without `modal-scope: true` the \
                         materialised node must carry the plain FocusRole::Container, not \
                         FocusRole::ModalScope, or this fixture is not exercising the \
                         unannotated case at all"
                    );
                }

                // The claim: entry did not happen. Focus is still where the
                // click itself left it (outside-b), not jumped to in-a.
                assert_focused_stop(window, &[1], "outside-b");

                // Tab still reaches the outside Buttons — there was never
                // any confinement to begin with. The whole tree is one
                // flat domain: outside-a, outside-b, in-a, in-b, wrapping.
                send_tab(hwnd);
                assert_focused_stop(window, &[2, 0], "in-a");
                send_tab(hwnd);
                assert_focused_stop(window, &[2, 1], "in-b");
                send_tab(hwnd);
                assert_focused_stop(window, &[0], "outside-a");
                send_tab(hwnd);
                assert_focused_stop(window, &[1], "outside-b");

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── Fixture 5 — Escape's two (or three) legs ────────────────────────────────

const ESCAPE_LEGS_UI: &str = r#"component EscapeConsumptionDependsOnAnEnteredScope inherits Window {
    state open: bool = false
    VStack {
        spacing: 0
        padding: 0
        Button { text: "trigger" clicked => { root.open = true; } }
        if open {
            VStack {
                spacing: 0
                padding: 0
                modal-scope: true
                dismiss => { root.open = false; }
                Button { text: "in-a" }
            }
        }
    }
}"#;

/// A scope with `modal-scope: true` and **no** `dismiss` handler at all —
/// the third leg, present from the initial build so no click is needed to
/// enter it.
const ESCAPE_NO_DISMISS_UI: &str = r#"component EscapeConsumedWithNoDismissHandlerDoesNotClose inherits Window {
    VStack {
        spacing: 0
        padding: 0
        VStack {
            spacing: 0
            padding: 0
            modal-scope: true
            Button { text: "only-stop" }
        }
    }
}"#;

/// `docs/dsl_spec.md` §4.19 / `docs/architecture.md` §13.5: Escape is
/// consumed by the innermost entered scope and never reaches the host key
/// slot; with no scope entered, the same key reaches the slot. Uses the
/// `WindowState::key_down_fn` recorder technique
/// `focus_traversal_integration.rs`'s Fixture 5 uses (this file's own
/// `install_key_slot_recorder`, copied from there).
///
/// - **Leg B (no scope present)**: Escape reaches the slot.
/// - **Leg A (a scope entered)**: Escape is consumed — the `dismiss`
///   handler runs (the subtree is removed) and the slot never sees it.
/// - **Leg C (a scope with no `dismiss` handler)**: Escape is still
///   consumed — the scope does not close, because nothing closes it, but
///   the key is still the scope's, not the propagation walk's / the host
///   slot's (`docs/dsl_spec.md` §4.19 "Writing no handler means the scope
///   does not close by dismissal").
#[test]
fn escapes_two_legs() {
    run_on_owning_runtime_thread_or_skip("modal scope: Escape's two legs", move || {
        // Legs A and B share one window: B (no scope) runs first, then a
        // click opens the scope, then A (scope entered) runs.
        {
            let ir = lower_ui_to_ir(ESCAPE_LEGS_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F5 A/B baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                let seen = install_key_slot_recorder(window);

                // Leg B: no scope present. Escape must reach the slot, via
                // the synchronous path (no state changes, no drain needed).
                assert_eq!(ffi::__focus_path_for_test(window), None);
                let escape_result = send_key(hwnd, VK_ESCAPE.0);
                assert_eq!(
                    seen.borrow().as_slice(),
                    &[VK_ESCAPE.0],
                    "with no scope present, Escape must reach the host key slot"
                );
                assert_eq!(
                    escape_result.0, 0,
                    "the fallthrough LRESULT happens to be 0 here too (Step 0's measurement \
                     in focus_traversal_integration.rs); the recorder above is what actually \
                     discriminates"
                );

                // Open the scope.
                let trigger_rect = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    root.children[0].__arranged_rect_for_test()
                }
                .expect("trigger must be laid out");
                let (tx, ty) = (
                    (trigger_rect.x + trigger_rect.width / 2.0) * factor,
                    (trigger_rect.y + trigger_rect.height / 2.0) * factor,
                );
                click_and_drain(hwnd, tx, ty);
                assert_focused_stop(window, &[1, 0], "in-a");

                // Leg A: a scope is entered. Escape must be consumed — the
                // recorder must not grow — and the dismiss handler's
                // removal must actually have run, which needs
                // `key_and_drain` for Phase 2.
                let seen_before_escape = seen.borrow().len();
                key_and_drain(hwnd, VK_ESCAPE.0);
                assert_eq!(
                    seen.borrow().len(),
                    seen_before_escape,
                    "with a scope entered, Escape must not reach the host key slot: it saw \
                     {:?}",
                    seen.borrow()
                );
                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        1,
                        "the dismiss handler must have removed the scope's subtree"
                    );
                }

                ffi::wasamo_window_destroy(window);
            }
        }

        // Leg C: a scope with no `dismiss` handler still consumes Escape,
        // and does not close.
        {
            let ir = lower_ui_to_ir(ESCAPE_NO_DISMISS_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F5 C baseline");

                assert_focused_stop(window, &[0, 0], "only-stop");
                let seen = install_key_slot_recorder(window);

                let escape_result = send_key(hwnd, VK_ESCAPE.0);
                assert!(
                    seen.borrow().is_empty(),
                    "Escape must be consumed by the entered scope even with no `dismiss` \
                     handler to run — it must not reach the host key slot. Saw {:?}",
                    seen.borrow()
                );
                assert_eq!(escape_result.0, 0);

                // The scope did not close: still present, still entered,
                // focus unchanged.
                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(root.children.len(), 1, "the scope must still be present");
                    assert_eq!(root.children[0].__focus_role_for_test(), "modal-scope");
                }
                assert_focused_stop(window, &[0, 0], "only-stop");

                ffi::wasamo_window_destroy(window);
            }
        }
    });
}

// ── Fixture 6 — arrow keys' two legs ────────────────────────────────────────

const ARROW_GROUP_UI: &str = r#"component ArrowKeysInsideAndOutsideAGroup inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "outside" }
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

/// `docs/dsl_spec.md` §4.19's key table: arrow keys go to the runtime while
/// focus is inside a `focus-group`, and to the propagation walk (here,
/// nothing claims it, so the host key slot / `DefWindowProc`) otherwise.
/// Uses the same recorder technique as [`escapes_two_legs`].
///
/// - **Leg A (focus inside the group)**: Right moves focus between
///   members, wrapping at the ends; Left moves backward; the slot never
///   sees either.
/// - **Leg B (focus outside any group)**: the identical key reaches the
///   slot, and focus does not move — the agreement leg without which
///   "consumed" would be indistinguishable from "did nothing"
///   (`plan.md`'s own framing of this requirement).
#[test]
fn arrow_keys_two_legs() {
    run_on_owning_runtime_thread_or_skip("modal scope: arrow keys' two legs", move || {
        let ir = lower_ui_to_ir(ARROW_GROUP_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F6 baseline");

            assert_eq!(ffi::__focus_path_for_test(window), None);
            {
                let root = (*window).root_widget.as_ref().expect("content root");
                assert_eq!(root.children[1].__focus_role_for_test(), "group");
            }

            let seen = install_key_slot_recorder(window);

            // Tab twice: first to "outside" (the domain's first stop),
            // second into the group, landing on its first member (no
            // memory yet).
            send_tab(hwnd);
            assert_focused_stop(window, &[0], "outside");
            send_tab(hwnd);
            assert_focused_stop(window, &[1, 0], "member-a");
            assert!(
                seen.borrow().is_empty(),
                "Tab must not reach the host key slot; saw {:?}",
                seen.borrow()
            );

            // Leg A: arrows inside the group move focus and wrap, never
            // reaching the slot.
            for expected in [
                ([1usize, 1usize], "member-b"),
                ([1, 2], "member-c"),
                ([1, 0], "member-a"), // wraps forward past the last member
            ] {
                let result = send_key(hwnd, VK_RIGHT.0);
                assert_eq!(result.0, 0, "an arrow the group handles must be consumed");
                assert_focused_stop(window, &expected.0, expected.1);
            }
            let result = send_key(hwnd, VK_LEFT.0);
            assert_eq!(result.0, 0);
            assert_focused_stop(window, &[1, 2], "member-c"); // wraps backward
            assert!(
                seen.borrow().is_empty(),
                "arrows consumed by group movement must never reach the host key slot; saw \
                 {:?}",
                seen.borrow()
            );

            // Leg B: leave the group (Tab wraps to "outside", the domain's
            // only other stop) and send the identical key again.
            send_tab(hwnd);
            assert_focused_stop(window, &[0], "outside");
            assert!(
                seen.borrow().is_empty(),
                "precondition: still nothing recorded"
            );

            send_key(hwnd, VK_RIGHT.0);
            assert_eq!(
                seen.borrow().as_slice(),
                &[VK_RIGHT.0],
                "outside any group, the identical arrow key must reach the host key slot"
            );
            // And it must not have moved focus — nothing in the runtime
            // has any meaning for this key here.
            assert_focused_stop(window, &[0], "outside");

            // Up/Down are accepted too (`arrow_direction`'s doc comment:
            // both axes are accepted regardless of the group's own
            // layout). Confirm one more consumption inside the group to
            // pin that, without re-running the whole leg-A sequence.
            //
            // Re-entering the group lands on the **remembered** member —
            // member-c, where leg A's `VK_LEFT` wrap left it
            // (`focus_core.rs`'s `a_group_remembers_the_member_that_was_focused`
            // is the pure-logic pin for this same rule) — not the first
            // member, so the landing below is member-c, not member-a.
            send_tab(hwnd);
            assert_focused_stop(window, &[1, 2], "member-c");
            let seen_before_down = seen.borrow().len();
            let down_result = send_key(hwnd, VK_DOWN.0);
            assert_eq!(
                down_result.0, 0,
                "VK_DOWN must be accepted as Arrow::Next too"
            );
            // Next from the last member wraps forward to the first.
            assert_focused_stop(window, &[1, 0], "member-a");
            assert_eq!(seen.borrow().len(), seen_before_down);

            ffi::wasamo_window_destroy(window);
        }
    });
}

// ── Fixture 7 — exit with no restore target leaves focus unset ─────────────

const SCOPE_NO_PRIOR_FOCUS_UI: &str = r#"component ModalScopeEnteredWithNothingFocusedExitLeavesFocusUnset inherits Window {
    state open: bool = true
    VStack {
        spacing: 0
        padding: 0
        Button { text: "outside" clicked => { root.open = false; } }
        if open {
            VStack {
                spacing: 0
                padding: 0
                modal-scope: true
                Button { text: "scope-a" }
                Button { text: "scope-b" }
            }
        }
    }
}"#;

/// [`entry_at_initial_build`] (Fixture 2) already reaches "a scope entered
/// with nothing focused before it": the initial build has no predecessor
/// input at all, so the restore target `enter_modal` captures there is
/// `None` by construction — there is no way anything could have been
/// focused before the window's first build. What Fixture 2 never does is
/// close the scope; this fixture does, and asserts that
/// `sync_scopes_to_tree`'s exit branch with `restore_to == None`
/// (`focus.rs` step 3) leaves focus unset rather than falling to the
/// domain's first stop.
///
/// **`None` here is the correct answer, not a degenerate case.**
/// `docs/dsl_spec.md` §4.19: entry "remembers the focused widget", which
/// may be nothing; DD-M4-P2-004: entry "captures the restore target: the
/// widget focused at that moment, possibly none". Restoration takes
/// precedence over structural succession (`docs/architecture.md` §13.4),
/// so "nothing was captured" must not be silently reinterpreted as "fall
/// through to the domain's first stop" — that would swap the exit branch
/// for the structural-succession branch whenever the captured target
/// happens to be `None`, which is exactly the branch confusion this
/// fixture exists to catch.
///
/// The scope closes through "outside", a Button declared *outside* the
/// confined scope: clicking it does not itself move focus (the click's own
/// focus resolution is filtered out by confinement, the same rule
/// [`a_present_but_unannotated_subtree_does_not_confine`]'s neighbouring
/// fixtures in this file exercise directly), but its `clicked` handler
/// still runs and removes the scope's subtree — pointer dispatch is
/// confined by an authored scrim, not by the scope itself
/// (DD-M4-P2-004 §"What containment covers"), and this fixture authors no
/// scrim.
#[test]
fn exit_with_no_restore_target_leaves_focus_unset() {
    run_on_owning_runtime_thread_or_skip(
        "modal scope: exit with no restore target leaves focus unset",
        move || {
            let ir = lower_ui_to_ir(SCOPE_NO_PRIOR_FOCUS_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;

                // No `normalise_to_reference_baseline` before this
                // read-back, matching `entry_at_initial_build`: the point
                // is that entry — and its `None` capture — has already
                // happened by the time `load_window` returns, before any
                // input at all.
                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(root.children.len(), 2, "outside Button + scope container");
                    assert_eq!(label_of(root.children[0].as_ref()), "outside");
                    assert_eq!(
                        root.children[1].__focus_role_for_test(),
                        "modal-scope",
                        "fixture stopped discriminating: the second child must actually \
                         carry FocusRole::ModalScope"
                    );
                    assert_eq!(label_of(root.children[1].children[0].as_ref()), "scope-a");
                }
                assert_focused_stop(window, &[1, 0], "scope-a");

                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F7 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                let outside_rect = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    root.children[0].__arranged_rect_for_test()
                }
                .expect("outside must be laid out");
                assert!(
                    outside_rect.width > 0.0 && outside_rect.height > 0.0,
                    "fixture stopped discriminating: outside's rectangle {outside_rect:?} \
                     must be non-degenerate"
                );

                // Click "outside" through `click_and_drain`: the scope's
                // removal needs Phase 2 for `sync_scopes_to_tree`'s exit
                // step to run.
                let (ox, oy) = (
                    (outside_rect.x + outside_rect.width / 2.0) * factor,
                    (outside_rect.y + outside_rect.height / 2.0) * factor,
                );
                click_and_drain(hwnd, ox, oy);

                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        1,
                        "the handler's own state write ('root.open = false') must have \
                         removed the scope's subtree synchronously, or this fixture is not \
                         exercising a removal at all"
                    );
                }

                // The claim: focus is unset — not fallen to "outside" (the
                // domain's first stop, what structural succession would
                // answer) and not still naming "scope-a" (gone).
                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "closing a scope that captured no restore target must leave focus \
                     unset, not fall through to the domain's first stop"
                );

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}
