//! Mock-free Windows integration evidence for M4-Phase 2 T8's runtime half of
//! `key-down("<key>")` — `focus::key_down_on_key` mapping the vk and picking
//! the walk's start, then `WidgetNode::deliver_key_down` walking the
//! ancestor chain — built from real `.ui` source through `wasamoc` -> IR ->
//! the loader, driven by real `WM_KEYDOWN` messages through the real window
//! procedure. Nothing is mocked.
//!
//! # What each fixture is for
//!
//! - [`key_down_fires_on_the_root_box_via_the_traversal_root_start`]
//!   (Fixture 1) — a `Box` with a `key-down("ArrowLeft")` handler fires with
//!   nothing focused: the walk's start is
//!   `focus_core::FocusTree::traversal_root` (the tree root, since no modal
//!   scope is entered — `docs/dsl_spec.md` §4.19 "or, when nothing is
//!   focused, at the innermost modal focus scope"), not "no dispatch at
//!   all". The `Box` carrying the handler must be the component's own
//!   root: `hit::dispatch_chain` walks upward from the start path toward
//!   the root, never down into descendants, so a handler on a *descendant*
//!   of the root could never fire from this start.
//! - [`key_down_reaches_an_ancestor_containers_handler`] (Fixture 2) — the
//!   ancestor walk: a container carries the handler, focus is on a Button
//!   inside it (which carries none), and the key fires the container's
//!   handler — the same "target, then ancestors" walk DD-M4-P2-001 gives
//!   clicks, now exercised for `key-down`.
//! - [`key_down_first_match_on_the_focused_node_consumes_before_any_ancestor`]
//!   (Fixture 3) — the focused node's own handler runs and consumes; its
//!   ancestor's handler for the identical key must not also run. Without
//!   this control, "the walk works" (Fixture 2) would be indistinguishable
//!   from "the ancestor always runs regardless of what the target itself
//!   carries".
//! - [`key_down_arrow_two_legs_inside_and_outside_a_focus_group`] (Fixture 4)
//!   — the plan's named arrow evidence, both legs: `key-down("ArrowLeft")`
//!   fires when focus is outside any `focus-group`, and does not fire while
//!   focus is inside one (the runtime keeps the arrow for group movement,
//!   `docs/dsl_spec.md` §4.19's key table) — with the second leg's group
//!   movement itself asserted, so "did not fire" is distinguished from
//!   "nothing happened at all".
//! - [`key_down_escape_two_legs_with_and_without_an_entered_modal_scope`]
//!   (Fixture 5) — `key-down("Escape")` fires with no modal scope entered
//!   and does not fire while one is (the scope converts Escape to a
//!   dismissal request instead, `focus::dismiss_on_key`) — the scope's own
//!   removal and focus restoration are the agreement leg that shows Escape
//!   was still consumed by *something* in the second leg, not merely lost.
//! - [`the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot`]
//!   (Fixture 6) — the two legs that pin where the new consumption arm sits
//!   relative to the host key slot: a recognised key with no handler
//!   anywhere on the chain reaches the slot, and a key an authored handler
//!   consumed does not. Every other fixture here reads handler *effects*,
//!   so without the second leg the arm's placement is unpinned in one
//!   direction — measured, not assumed.
//! - [`a_key_down_handlers_state_write_re_lays_out_through_the_same_drain_clicks_use`]
//!   (Fixture 7) — a handler's state write both updates a bound `Text` and
//!   materialises a new conditional subtree whose own layout (a non-`None`
//!   `__arranged_rect_for_test()`) only exists once `emit::flush_layout`
//!   (Phase 2) has actually run — the same drain a click-triggered
//!   structural mutation goes through in `modal_scope_integration.rs`.
//!
//! # `send_key` vs `key_and_drain`
//!
//! An inline handler's state write drains its reactive Effects
//! **synchronously** (`widget.rs`'s `run_clicked_handlers` doc comment, next
//! to `hit_test_click`; `run_signal_handlers`'s doc comment, shared by
//! `key-down` and `dismiss`), so a plain [`send_key`] (`SendMessageW`) is
//! enough to observe a bound `Text.content` change or a focus-record read
//! that needs no layout. What synchronous drain does **not** include is
//! `emit::flush_layout` (Phase 2) — the actual layout pass, and
//! `focus::sync_scopes_to_tree`'s modal-scope reconciliation that runs
//! inside it — which only executes at the message-loop boundary
//! `wasamo_run` reaches after `DispatchMessageW`. [`key_and_drain`] posts the
//! key, posts a synthesised `WM_QUIT`, then pumps `wasamo_runtime::run()` to
//! reach that boundary. Each fixture below states which it uses and why.
//!
//! # Helpers copied from `modal_scope_integration.rs`
//!
//! Cargo compiles every `tests/*.rs` file as its own crate, so nothing in
//! that file is importable here (a convention its own module header already
//! records, itself continuing the chain from `focus_traversal_integration.rs`
//! / `focus_identity_integration.rs`). `lower_ui_to_ir`, `load_window`,
//! `client_extent`, `display_limits`, `window_rect`, `frame_thickness`,
//! `send_dpi_change_to_client`, `normalise_to_reference_baseline`,
//! `click_and_drain`, `key_and_drain`, `last_error`, `label_of`,
//! `node_at_path`, `assert_focused_stop`, `send_key`, `send_tab`, and
//! `install_key_slot_recorder` are copied verbatim from
//! `modal_scope_integration.rs` (`send_click` is omitted: no fixture here
//! needs a synchronous, non-drained click). This file's own addition is
//! [`read_counter`] (copied in spirit from `event_routing_integration.rs`'s
//! helper of the same name) and the seven fixtures themselves.
//!
//! # The client stays small (M4-Phase 1 T8 finding)
//!
//! Every fixture normalises to 96 DPI at an explicitly chosen 360x240
//! physical client — the same ceiling every other integration test file in
//! this crate records (a hosted CI desktop failed a larger request twice).

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::cell::RefCell;
use std::ffi::{c_void, CStr};
use std::ptr;
use std::rc::Rc;

use wasamo_runtime::{ffi, WidgetNode};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_LEFT, VK_RIGHT, VK_TAB};
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

// ── Copied verbatim from `modal_scope_integration.rs` ───────────────────────

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<key-down-integration>";
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
/// `focus::dismiss_on_key` / `focus::key_down_on_key` can run a handler
/// whose state write removes or materialises a subtree synchronously
/// (Phase 1), but layout / `sync_scopes_to_tree`'s exit-restoration only
/// runs at Phase 2, which needs the message-loop boundary this pumps.
/// `lparam` is `0`: nothing in `window.rs`'s `WM_KEYDOWN` arm reads the
/// repeat count / scan code / extended-key bits, only `wparam` (the
/// virtual-key code) — the same simplification `focus_traversal_integration.rs`'s
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

/// A real `WM_KEYDOWN` for an arbitrary virtual-key code, with no modifier
/// state manipulated.
unsafe fn send_key(hwnd: HWND, vk: u16) -> LRESULT {
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(vk as usize), LPARAM(0))
}

/// A real `WM_KEYDOWN(VK_TAB)` with no Shift held.
unsafe fn send_tab(hwnd: HWND) -> LRESULT {
    send_key(hwnd, VK_TAB.0)
}

/// Install a recorder in the window's **host key slot** and hand back the
/// list it appends to. `window.rs`'s `WM_KEYDOWN` arm calls `key_down_fn`
/// only after traversal, arrow routing, dismissal, and the authored
/// `key-down` walk have all declined the key, so what this list contains
/// after a message is the direct answer to "did anything upstream consume
/// it" — installing a recorder in a pre-existing `pub` field is not
/// shipping an installer (`focus_traversal_integration.rs`'s module header
/// states this in full; `process/milestone-4/phase-2/requirements/constraints.md`
/// §2 is about a *host-facing* installer, which this is not).
unsafe fn install_key_slot_recorder(window: *mut ffi::WasamoWindow) -> Rc<RefCell<Vec<u16>>> {
    let seen: Rc<RefCell<Vec<u16>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    (*window).key_down_fn = Some(Box::new(move |vk| sink.borrow_mut().push(vk)));
    seen
}

// ── This file's own addition ────────────────────────────────────────────

/// Read a `Text` widget's rendered content and parse it as the `i32`
/// counter it is bound to (`"\{root.<state>}"`). Copied in spirit from
/// `event_routing_integration.rs`'s helper of the same name:
/// `__text_content_for_test` reads the `content: String` field directly,
/// which an inline handler's `Signal::set` writes synchronously (this
/// file's module header) — no layout pass is needed for this read to be
/// current. Panics with `what` in the message if the node is not a `Text`
/// or its content does not parse: either would mean the fixture's binding
/// is broken, not that the counter legitimately holds some value.
fn read_counter(node: &WidgetNode, what: &str) -> i32 {
    let content = node
        .__text_content_for_test()
        .unwrap_or_else(|| panic!("{what}: node must be a Text widget bound to the counter"));
    content
        .parse::<i32>()
        .unwrap_or_else(|e| panic!("{what}: Text content {content:?} did not parse as i32 ({e})"))
}

// ── Fixture 1 — fires via the traversal_root start ──────────────────────────

/// The handler's own `Box` is the component's **root** — see this
/// function's own doc comment (and this file's module header) for why that
/// placement is load-bearing, not incidental: `hit::dispatch_chain` walks
/// upward from the start path, never down, so only a handler *at or above*
/// the start can ever run.
const ROOT_BOX_KEY_DOWN_UI: &str = r#"component KeyDownFiresOnRootBoxViaTraversalRoot inherits Window {
    state hits: i32 = 0
    Box {
        key-down("ArrowLeft") => { root.hits += 1; }
        Text { text: "\{root.hits}" }
    }
}"#;

/// Fixture 1: nothing is focused when the window opens, so
/// `focus::key_down_on_key`'s start comes from
/// `focus_core::FocusTree::traversal_root` — the tree root, since no modal
/// scope is entered — not "no dispatch at all"
/// (`docs/dsl_spec.md` §4.19 "or, when nothing is focused, at the innermost
/// modal focus scope"). The precondition ("nothing focused") is asserted
/// explicitly before the key is sent, so the fixture states which start
/// path the walk took rather than assuming it.
///
/// Uses [`send_key`]: the handler's state write updates the bound `Text`
/// child's content synchronously (this file's module header), and nothing
/// here needs a layout pass.
#[test]
fn key_down_fires_on_the_root_box_via_the_traversal_root_start() {
    run_on_owning_runtime_thread_or_skip(
        "key-down: fires via the traversal_root start",
        move || {
            let ir = lower_ui_to_ir(ROOT_BOX_KEY_DOWN_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F1 baseline");

                // The precondition this fixture is about.
                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "fixture stopped discriminating: this fixture is specifically about the \
                     traversal_root start, which only applies when nothing is focused"
                );
                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.__kind_name_for_test(),
                        "Box",
                        "fixture stopped discriminating: the component's root must actually be \
                         the Box carrying the handler, or the traversal_root start names \
                         something else entirely"
                    );
                    assert_eq!(
                        read_counter(&root.children[0], "hits before any key"),
                        0,
                        "fixture stopped discriminating: hits must start at 0"
                    );
                }

                send_key(hwnd, VK_LEFT.0);

                let hits_after = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    read_counter(&root.children[0], "hits after ArrowLeft")
                };

                ffi::wasamo_window_destroy(window);

                assert_eq!(
                    hits_after, 1,
                    "the root Box's own key-down(\"ArrowLeft\") handler must have run exactly \
                     once, dispatched from the traversal_root start with nothing focused"
                );
            }
        },
    );
}

// ── Fixture 2 — the ancestor walk ────────────────────────────────────────

const ANCESTOR_CONTAINER_KEY_DOWN_UI: &str = r#"component KeyDownReachesAnAncestorContainer inherits Window {
    state hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.hits}" }
        VStack {
            spacing: 0
            padding: 0
            key-down("ArrowLeft") => { root.hits += 1; }
            Button { text: "member" }
        }
    }
}"#;

/// Fixture 2: a container carries the `key-down("ArrowLeft")` handler;
/// focus is on a Button inside it that carries none. The key must still
/// fire the container's handler — `docs/dsl_spec.md` §4.19's "target, then
/// ancestors" walk, the same one DD-M4-P2-001 gives clicks.
///
/// Uses [`send_key`]: same reasoning as Fixture 1.
#[test]
fn key_down_reaches_an_ancestor_containers_handler() {
    run_on_owning_runtime_thread_or_skip("key-down: reaches an ancestor container", move || {
        let ir = lower_ui_to_ir(ANCESTOR_CONTAINER_KEY_DOWN_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F2 baseline");

            assert_eq!(ffi::__focus_path_for_test(window), None);
            send_tab(hwnd);
            assert_focused_stop(window, &[1, 0], "member");

            send_key(hwnd, VK_LEFT.0);

            let hits_after = {
                let root = (*window).root_widget.as_ref().unwrap();
                read_counter(&root.children[0], "hits after ArrowLeft")
            };

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                hits_after, 1,
                "the inner VStack's key-down(\"ArrowLeft\") handler must have run exactly once \
                 for a key sent while its Button member is focused"
            );
        }
    });
}

// ── Fixture 3 — first match consumes before any ancestor ────────────────────

const FIRST_MATCH_CONSUMES_UI: &str = r#"component KeyDownFirstMatchConsumesBeforeAnyAncestor inherits Window {
    state inner_hits: i32 = 0
    state outer_hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.inner_hits}" }
        Text { text: "\{root.outer_hits}" }
        VStack {
            spacing: 0
            padding: 0
            key-down("ArrowLeft") => { root.outer_hits += 1; }
            Button { text: "member" key-down("ArrowLeft") => { root.inner_hits += 1; } }
        }
    }
}"#;

/// Fixture 3: both the focused Button and its ancestor container carry
/// `key-down("ArrowLeft")` handlers. Only the Button's own (the first
/// match on the walk) must run — the control without which Fixture 2's
/// "the walk works" would be indistinguishable from "the ancestor always
/// runs regardless of what the target itself carries".
///
/// Uses [`send_key`]: same reasoning as Fixture 1.
#[test]
fn key_down_first_match_on_the_focused_node_consumes_before_any_ancestor() {
    run_on_owning_runtime_thread_or_skip("key-down: first match consumes", move || {
        let ir = lower_ui_to_ir(FIRST_MATCH_CONSUMES_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F3 baseline");

            assert_eq!(ffi::__focus_path_for_test(window), None);
            send_tab(hwnd);
            assert_focused_stop(window, &[2, 0], "member");

            send_key(hwnd, VK_LEFT.0);

            let (inner_after, outer_after) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    read_counter(&root.children[0], "inner_hits after ArrowLeft"),
                    read_counter(&root.children[1], "outer_hits after ArrowLeft"),
                )
            };

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                inner_after, 1,
                "the focused Button's own key-down(\"ArrowLeft\") handler must have run exactly \
                 once"
            );
            assert_eq!(
                outer_after, 0,
                "the ancestor container's identical handler must not also run: the focused \
                 Button's own handler running must consume the key"
            );
        }
    });
}

// ── Fixture 4 — arrows' two legs ─────────────────────────────────────────

const ARROW_TWO_LEGS_UI: &str = r#"component KeyDownArrowTwoLegsInsideAndOutsideAFocusGroup inherits Window {
    state outside_hits: i32 = 0
    state group_hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.outside_hits}" }
        Text { text: "\{root.group_hits}" }
        Button { text: "outside" key-down("ArrowLeft") => { root.outside_hits += 1; } }
        HStack {
            spacing: 0
            padding: 0
            focus-group: true
            Button { text: "member-a" key-down("ArrowLeft") => { root.group_hits += 1; } }
            Button { text: "member-b" key-down("ArrowLeft") => { root.group_hits += 1; } }
        }
    }
}"#;

/// Fixture 4: the plan's named arrow evidence, both legs.
/// `key-down("ArrowLeft")` fires when focus is outside any `focus-group`
/// (Leg A) and must not fire while focus is inside one (Leg B) — the
/// runtime keeps the arrow for group movement instead
/// (`docs/dsl_spec.md` §4.19's key table). Both group members also carry
/// the identical handler, so if routing failed to intercept the key in Leg
/// B, `group_hits` would tick — and Leg B's own focus assertion confirms
/// group movement *did* happen, so "did not fire" is distinguished from
/// "nothing happened at all" (the control the plan names explicitly).
///
/// Uses [`send_key`] throughout: no structural mutation, and every
/// assertion (counters, the retained focus record) is synchronous state
/// with no layout dependency.
#[test]
fn key_down_arrow_two_legs_inside_and_outside_a_focus_group() {
    run_on_owning_runtime_thread_or_skip("key-down: arrow two legs", move || {
        let ir = lower_ui_to_ir(ARROW_TWO_LEGS_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F4 baseline");

            assert_eq!(ffi::__focus_path_for_test(window), None);
            {
                let root = (*window).root_widget.as_ref().expect("content root");
                assert_eq!(
                    root.children[3].__focus_role_for_test(),
                    "group",
                    "fixture stopped discriminating: the HStack must actually carry \
                     FocusRole::Group"
                );
            }

            // Leg A: outside the group.
            send_tab(hwnd);
            assert_focused_stop(window, &[2], "outside");
            let result = send_key(hwnd, VK_LEFT.0);
            assert_eq!(
                result.0, 0,
                "the fallthrough LRESULT happens to be 0 here too; the counters below are what \
                 actually discriminate"
            );
            let (outside_after_a, group_after_a) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    read_counter(&root.children[0], "outside_hits after leg A"),
                    read_counter(&root.children[1], "group_hits after leg A"),
                )
            };
            assert_focused_stop(window, &[2], "outside");

            // Leg B: inside the group.
            send_tab(hwnd);
            assert_focused_stop(window, &[3, 0], "member-a");
            send_key(hwnd, VK_LEFT.0);
            // Group movement itself: Arrow::Prev from the first member wraps
            // backward to the last (`arrow_direction`'s doc comment; the
            // same wrap `modal_scope_integration.rs`'s `arrow_keys_two_legs`
            // pins for a 3-member group).
            assert_focused_stop(window, &[3, 1], "member-b");
            let (outside_after_b, group_after_b) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    read_counter(&root.children[0], "outside_hits after leg B"),
                    read_counter(&root.children[1], "group_hits after leg B"),
                )
            };

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                outside_after_a, 1,
                "leg A: the \"outside\" Button's own key-down(\"ArrowLeft\") handler must have \
                 run exactly once, focus being outside any group"
            );
            assert_eq!(
                group_after_a, 0,
                "leg A: no group member's handler must have run"
            );
            assert_eq!(
                outside_after_b, outside_after_a,
                "leg B: the \"outside\" Button is no longer focused, so its handler must not \
                 have run again"
            );
            assert_eq!(
                group_after_b, 0,
                "leg B: with focus inside the group, ArrowLeft must be consumed by group \
                 movement before the authored key-down walk ever runs — neither member's \
                 identical handler must have run, even though the key visibly did something \
                 (moved focus to member-b, asserted above)"
            );
        }
    });
}

// ── Fixture 5 — Escape's two legs ────────────────────────────────────────

const ESCAPE_TWO_LEGS_UI: &str = r#"component KeyDownEscapeTwoLegsWithAndWithoutAnEnteredModalScope inherits Window {
    state open: bool = false
    state outside_hits: i32 = 0
    state inside_hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.outside_hits}" }
        Text { text: "\{root.inside_hits}" }
        Button { text: "trigger" key-down("Escape") => { root.outside_hits += 1; } clicked => { root.open = true; } }
        if open {
            VStack {
                spacing: 0
                padding: 0
                modal-scope: true
                dismiss => { root.open = false; }
                Button { text: "in-a" key-down("Escape") => { root.inside_hits += 1; } }
            }
        }
    }
}"#;

/// Fixture 5: `key-down("Escape")` fires with no modal scope entered (Leg
/// A) and must not fire while one is (Leg B) — `focus::dismiss_on_key`
/// converts Escape into a dismissal request instead
/// (`docs/dsl_spec.md` §4.19). The scope's own handler (`dismiss`) removes
/// its subtree and focus is restored to "trigger" — the agreement leg that
/// shows Escape was still consumed by *something* in Leg B, not merely
/// lost, distinguishing "the scope's own mechanism took it" from "nothing
/// claimed it at all".
///
/// Leg A uses [`send_key`] (no structural mutation). Opening the scope and
/// Leg B both use the drained variants ([`click_and_drain`] /
/// [`key_and_drain`]): opening materialises the scope's subtree and enters
/// it (`sync_scopes_to_tree`'s entry step, Phase 2); closing removes that
/// subtree and restores focus (its exit step, also Phase 2).
#[test]
fn key_down_escape_two_legs_with_and_without_an_entered_modal_scope() {
    run_on_owning_runtime_thread_or_skip("key-down: Escape two legs", move || {
        let ir = lower_ui_to_ir(ESCAPE_TWO_LEGS_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F5 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            assert_eq!(ffi::__focus_path_for_test(window), None);
            send_tab(hwnd);
            assert_focused_stop(window, &[2], "trigger");

            // Leg A: no scope present.
            send_key(hwnd, VK_ESCAPE.0);
            let (outside_after_a, inside_after_a) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    read_counter(&root.children[0], "outside_hits after leg A"),
                    read_counter(&root.children[1], "inside_hits after leg A"),
                )
            };
            assert_eq!(
                outside_after_a, 1,
                "leg A: \"trigger\"'s own key-down(\"Escape\") handler must have run exactly \
                 once with no scope present"
            );
            assert_eq!(inside_after_a, 0);

            // Open the scope: a click both focuses "trigger" (already
            // focused, a no-op here) and runs its `clicked` handler, which
            // sets `open = true`.
            let trigger_rect = {
                let root = (*window).root_widget.as_ref().expect("content root");
                root.children[2].__arranged_rect_for_test()
            }
            .expect("trigger must be laid out");
            let (tx, ty) = (
                (trigger_rect.x + trigger_rect.width / 2.0) * factor,
                (trigger_rect.y + trigger_rect.height / 2.0) * factor,
            );
            click_and_drain(hwnd, tx, ty);
            {
                let root = (*window).root_widget.as_ref().expect("content root");
                assert_eq!(
                    root.children.len(),
                    4,
                    "the click's handler must have materialised the scope's subtree"
                );
                assert_eq!(
                    root.children[3].__focus_role_for_test(),
                    "modal-scope",
                    "fixture stopped discriminating: the materialised container must actually \
                     carry FocusRole::ModalScope"
                );
            }
            assert_focused_stop(window, &[3, 0], "in-a");

            // Leg B: a scope is entered. Escape must be consumed by
            // `dismiss_on_key`, not reach the authored key-down walk.
            key_and_drain(hwnd, VK_ESCAPE.0);

            let (outside_after_b, inside_after_b) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    read_counter(&root.children[0], "outside_hits after leg B"),
                    read_counter(&root.children[1], "inside_hits after leg B"),
                )
            };
            {
                let root = (*window).root_widget.as_ref().expect("content root");
                assert_eq!(
                    root.children.len(),
                    3,
                    "the scope's own `dismiss` handler must have removed its subtree — the \
                     agreement leg showing Escape was consumed by the modal path, not simply \
                     lost"
                );
            }
            // Restoration: focus returns to "trigger", the node that was
            // focused when the scope was entered.
            assert_focused_stop(window, &[2], "trigger");

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                outside_after_b, outside_after_a,
                "leg B: \"trigger\" is not focused while the scope is entered, so its handler \
                 must not have run again"
            );
            assert_eq!(
                inside_after_b, 0,
                "leg B: \"in-a\"'s own key-down(\"Escape\") handler must never have run — \
                 dismiss_on_key must have consumed Escape ahead of the authored key-down walk"
            );
        }
    });
}

// ── Fixture 6 — the authored walk's two legs against the host key slot ─────

const HOST_KEY_SLOT_LEGS_UI: &str = r#"component TheAuthoredWalkAndTheHostKeySlot inherits Window {
    state hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        key-down("ArrowRight") => { root.hits += 1; }
        Text { text: "\{root.hits}" }
        Button { text: "only" }
    }
}"#;

/// Fixture 6: the two legs that pin **where** `focus::key_down_on_key` sits
/// relative to the host key slot in `window.rs`'s `WM_KEYDOWN` arm — the
/// CF-T5-5 tripwire's own shape, applied to the arm this task adds.
///
/// - **Leg A (no handler for the key)**: `ArrowLeft` is a recognised key
///   (`wasamo_ir::RECOGNISED_KEY_NAMES`) that nothing on the chain declares
///   a handler for. `WidgetNode::deliver_key_down` returns `false`, so the
///   key must reach the host key slot — the agreement leg proving the new
///   consumption arm did not start swallowing keys nothing claims.
/// - **Leg B (a handler runs)**: `ArrowRight` has a handler on the chain.
///   It must run **and** the key must **not** reach the slot. Without this
///   leg the arm's placement is unpinned in one direction: moving
///   `key_down_on_key` behind the slot, or dropping its early `return`,
///   leaves every other fixture in this file green, because they all read
///   handler effects rather than what the slot saw. Measured — the lead ran
///   that mutation before writing this leg and the whole suite stayed
///   green.
///
/// The `LRESULT` cannot discriminate the two legs — T5 measured it as 0
/// either way over eight candidate keys — so the recorder is what does.
///
/// Uses [`send_key`]: the handler's state write reaches the bound `Text`
/// synchronously (this file's module header), and no layout pass is needed.
#[test]
fn the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot() {
    run_on_owning_runtime_thread_or_skip("key-down: the host key slot's two legs", move || {
        let ir = lower_ui_to_ir(HOST_KEY_SLOT_LEGS_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F6 baseline");

            let seen = install_key_slot_recorder(window);

            assert_eq!(ffi::__focus_path_for_test(window), None);
            send_tab(hwnd);
            assert_focused_stop(window, &[1], "only");
            assert!(
                seen.borrow().is_empty(),
                "precondition: nothing recorded before the keys under test"
            );

            // Leg A — no handler for ArrowLeft anywhere on the chain.
            let unmatched_result = send_key(hwnd, VK_LEFT.0);
            let after_leg_a = seen.borrow().clone();

            // Leg B — the ancestor VStack's ArrowRight handler runs.
            let matched_result = send_key(hwnd, VK_RIGHT.0);
            let after_leg_b = seen.borrow().clone();
            let hits_after = {
                let root = (*window).root_widget.as_ref().unwrap();
                read_counter(&root.children[0], "hits after ArrowRight")
            };

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                after_leg_a,
                vec![VK_LEFT.0],
                "leg A: a recognised key with no handler anywhere on the chain must reach the \
                 host key slot; saw {after_leg_a:?}"
            );
            assert_eq!(
                hits_after, 1,
                "leg B stopped discriminating: the ArrowRight handler must actually have run, \
                 or 'the slot did not see it' would be satisfied by a key that went nowhere"
            );
            assert_eq!(
                after_leg_b, after_leg_a,
                "leg B: a key an authored handler consumed must NOT reach the host key slot — \
                 the recorder must not have grown past leg A's entry; saw {after_leg_b:?}"
            );
            assert_eq!(
                unmatched_result.0, 0,
                "the fallthrough LRESULT happens to be 0 here too (T5's measurement over eight \
                 candidate keys); the recorder above is what actually discriminates"
            );
            assert_eq!(matched_result.0, 0, "the consuming arm returns LRESULT(0)");
        }
    });
}

// ── Fixture 7 — the handler's state write re-lays out through the drain ────

const RE_LAYOUT_VIA_DRAIN_UI: &str = r#"component KeyDownHandlerReLaysOutThroughTheSameDrainClicksUse inherits Window {
    state shown: bool = false
    state hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Button { text: "trigger" key-down("ArrowLeft") => { root.hits += 1; root.shown = true; } }
        if shown {
            Text { text: "\{root.hits}" }
        }
    }
}"#;

/// Fixture 7: the handler's state write both increments a bound counter
/// and flips a conditional to materialise a new `Text`. The `Text`'s
/// *content* would already be correct after Phase 1's synchronous Effect
/// drain alone (this file's module header), but its *layout*
/// (`__arranged_rect_for_test()`) only exists once `emit::flush_layout`
/// (Phase 2) has actually run a layout pass over the now-larger tree — the
/// same drain a click-triggered structural mutation goes through
/// (`modal_scope_integration.rs`'s fixtures). Asserting both together is
/// what makes this fixture prove the drain path, not merely that
/// `key_and_drain` didn't crash.
#[test]
fn a_key_down_handlers_state_write_re_lays_out_through_the_same_drain_clicks_use() {
    run_on_owning_runtime_thread_or_skip(
        "key-down: state write re-lays out via the drain",
        move || {
            let ir = lower_ui_to_ir(RE_LAYOUT_VIA_DRAIN_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F7 baseline");

                assert_eq!(ffi::__focus_path_for_test(window), None);
                send_tab(hwnd);
                assert_focused_stop(window, &[0], "trigger");
                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                    root.children.len(),
                    1,
                    "fixture stopped discriminating: `shown` starts false, so only \"trigger\" \
                     must be present before the key"
                );
                }

                key_and_drain(hwnd, VK_LEFT.0);

                let (children_after, hits_after, revealed_rect) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    (
                        root.children.len(),
                        root.children
                            .get(1)
                            .map(|t| read_counter(t, "hits after reveal")),
                        root.children
                            .get(1)
                            .and_then(|t| t.__arranged_rect_for_test()),
                    )
                };

                ffi::wasamo_window_destroy(window);

                assert_eq!(
                    children_after, 2,
                    "the handler's `root.shown = true` must have materialised the conditional \
                 Text, or this fixture is not exercising a re-layout at all"
                );
                assert_eq!(
                    hits_after,
                    Some(1),
                    "the materialised Text's bound content must reflect the same handler's \
                 `root.hits += 1`"
                );
                let revealed_rect = revealed_rect.expect(
                    "the materialised Text's arranged rect must be populated by the drain's \
                          layout pass (emit::flush_layout, Phase 2) — this is what key_and_drain \
                          is required for, unlike Fixtures 1-3's plain send_key",
                );
                assert!(
                    revealed_rect.width > 0.0 && revealed_rect.height > 0.0,
                    "the materialised Text's rectangle {revealed_rect:?} must be non-degenerate, \
                 confirming a real layout pass positioned it rather than merely inserting a \
                 zeroed node"
                );
            }
        },
    );
}
