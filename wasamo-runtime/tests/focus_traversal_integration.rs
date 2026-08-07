//! Mock-free Windows integration evidence for M4-Phase 2 T5 stage 2
//! ([DD-M4-P2-003](../../process/milestone-4/phase-2/decisions/dd-m4-p2-003-focus-model-and-traversal.md)):
//! per-window focus state and Tab / Shift+Tab traversal over a real widget
//! tree built from real `.ui` source through `wasamoc` -> IR -> the loader,
//! driven by real `WM_KEYDOWN` / `WM_LBUTTONUP` / `WM_SETFOCUS` /
//! `WM_KILLFOCUS` messages through the real window procedure (`window.rs`'s
//! `WM_KEYDOWN` arm and `WM_LBUTTONUP` arm, and the fallthrough that both
//! `WM_SETFOCUS` and `WM_KILLFOCUS` take because neither has an arm at
//! all), with live widget state read back afterwards. Nothing is mocked.
//!
//! # The two read-backs, and why both are asserted together
//!
//! - `WidgetNode::__button_focused_for_test() -> Option<bool>` — `None`
//!   for a non-Button-family widget, otherwise whether that node paints
//!   the focus indicator. The **painted** state.
//! - `ffi::__focus_path_for_test(window) -> Option<Vec<usize>>` — the
//!   window's **retained** focus record (`WindowFocus`, projected back to
//!   a tree path).
//!
//! `focus.rs`'s `move_focus` doc comment states the invariant these two
//! are supposed to satisfy: it is "the single primitive" that writes both,
//! which is what keeps the retained record and the painted flag from being
//! edited apart — the same shape T4's `HoverState` gives hover
//! (implementation-gates trap #3). Every fixture below therefore reads
//! both after every step where either could have changed, on the node the
//! record now names **and** on the node it previously named, and — for
//! every fixture that names *which* stop focus landed on — reads that
//! node's label back through the C ABI too
//! (`tests/focus_mechanism_fixture.rs`'s technique and its own reasoning
//! for it: a path is checked against the same tree walk the implementation
//! used, so it alone cannot show focus landed on a *real* widget the
//! fixture meant).
//!
//! # Step 0 — the fallthrough `LRESULT` measurement
//!
//! The T5 start gate recorded an unmeasured negative prediction
//! ([log.md §T5 fact 6](../../process/milestone-4/phase-2/implementation/log.md#t5--per-window-focus-state-and-tab-traversal)):
//! "an unconsumed key's fallthrough to `DefWindowProc` cannot be
//! distinguished by the returned `LRESULT`, because `DefWindowProcW`
//! returns 0 for `WM_KEYDOWN`." Measured with a throwaway probe before any
//! fixture below was written (deleted afterwards, not part of this file):
//! for every candidate — `VK_F1` (0x70), `VK_F10` (0x79), `VK_APPS`
//! (0x5D), `VK_ESCAPE` (0x1B), `VK_RETURN` (0x0D), `VK_SPACE` (0x20),
//! `VK_LEFT` (0x25), and `'A'` (0x41) — both a `SendMessageW(WM_KEYDOWN)`
//! through the real `wnd_proc` (nothing installed to consume it) and a
//! direct `DefWindowProcW` call for the same message returned
//! `LRESULT(0)`. **The prediction is confirmed**, not merely unmeasured
//! anymore: every value is 0 today, so the `LRESULT` alone cannot
//! distinguish "traversal correctly declined this key" from "some
//! unrelated path also happened to return 0".
//!
//! **The host key slot is what discriminates instead.**
//! `WindowState::key_down_fn` is a `pub` field, uninstalled in production
//! (nothing in the runtime, the ABI, the bindings or the examples writes
//! it — the T5 start gate measured that), and `window.rs`'s `WM_KEYDOWN`
//! arm reaches it **only after** `focus::traverse_on_key` has declined the
//! key. Installing a recorder there for the length of one fixture
//! therefore makes "was this key consumed by traversal" a directly
//! observable fact rather than an inference from a return value that is 0
//! either way: a consumed key never reaches the recorder, an unclaimed one
//! always does. This installs no API — the field and its `FnMut(u16)`
//! signature already exist, and
//! `process/milestone-4/phase-2/requirements/constraints.md` §2's rule is
//! about shipping a *host-facing* installer, which a test writing a
//! pre-existing `pub` field is not.
//! [`tab_is_consumed_by_traversal_while_an_unclaimed_key_is_not`]
//! (Fixture 5) uses both: the slot for consumption, and the `LRESULT`
//! compared against a **live** direct `DefWindowProcW` call for the same
//! message (a measured comparison, not a hard-coded `0`) for the
//! return-path half, whose limit that fixture's own doc comment states.
//!
//! # Step 0b — the Shift-state measurement
//!
//! `Shift+Tab` needs `GetKeyState(VK_SHIFT)` to read as held while a
//! synthesized `WM_KEYDOWN` is processed inside `wnd_proc`. Measured with
//! a second throwaway probe (also deleted, not part of this file),
//! following the M4-Phase 1 F-49 discipline (try, discard the result, and
//! read the actual state back rather than assume it worked): `GetKeyboardState`
//! + setting the `VK_SHIFT` entry's high bit + `SetKeyboardState`, then
//! `GetKeyState(VK_SHIFT)` read back with its high bit set — confirmed —
//! **and** confirmed to actually reach `wnd_proc`'s own `GetKeyState`
//! call specifically, not merely this thread in general: a three-stop
//! window Tabbed twice to its second stop, then a real
//! `WM_KEYDOWN(VK_TAB)` sent with the Shift bit held moved focus
//! **backward** to the first stop (the direction only Shift produces —
//! forward would have gone to the third). [`send_shift_tab`] below bakes
//! this exact read-back-before-send, restore-after discipline into one
//! helper so every fixture's Shift+Tab call sits behind it automatically.
//!
//! # What each fixture is for
//!
//! - [`nothing_is_focused_when_a_window_opens_and_window_activation_does_not_change_it`]
//!   (Fixture 1) — `docs/dsl_spec.md` §4.19 "Nothing is focused when a
//!   window opens" (walked over the **whole** tree, not spot-checked) and
//!   "Losing and regaining the window's activation does not change which
//!   widget is focused" — the discriminating half, since a runtime that
//!   never focused anything would also pass the first half.
//! - [`tab_walks_the_stops_in_declaration_order_and_wraps_at_both_ends`]
//!   (Fixture 2) — declaration order and the wrap at both ends, over a
//!   tree where "declaration order of stops" and "declaration order of
//!   children" are two different lists.
//! - [`a_disabled_button_is_skipped_by_traversal_and_a_button_disabled_while_focused_loses_it_at_the_next_tab`]
//!   (Fixture 3) — a disabled stop is skipped (paired with an agreement
//!   leg, or "skipped" would be indistinguishable from "unreachable"), and
//!   DD-M4-P2-003's named boundary: disabling the *focused* stop through
//!   the binding path applies the successor rule lazily, at the next
//!   traversal, landing on the domain's first stop rather than an eager
//!   successor.
//! - [`a_click_focuses_the_nearest_focusable_widget_at_or_above_the_target_and_a_background_click_changes_nothing`]
//!   (Fixture 4) — a click focuses a Button; a click on a non-focusable
//!   widget with no focusable ancestor leaves the previously focused stop
//!   **unchanged and still painted** (the pair that distinguishes
//!   "unchanged" from "cleared"); a click outside every widget does the
//!   same. **The "at or above" leg is not built** — see the fixture's own
//!   doc comment for the measured reason (T3 start-gate finding 1: a
//!   Button-family node is always a layout leaf, so it can never have a
//!   hit-testable descendant to click on instead of itself).
//! - [`tab_is_consumed_by_traversal_while_an_unclaimed_key_is_not`]
//!   (Fixture 5) — Step 0's licensed assertion shape, plus the branch
//!   `focus::traverse_on_key`'s doc comment names explicitly: Tab is
//!   consumed even when the tree has no focusable widget at all.
//! - [`a_focus_record_naming_a_removed_stop_is_cleared_rather_than_read_back_stale`]
//!   (Fixture 6) — a click on a stop whose own handler removes the
//!   conditional subtree it lives in must not leave the retained focus
//!   record naming a node the next projection cannot explain
//!   (`docs/architecture.md` §13.3, DD-M4-P2-003); the record reads back
//!   `None` and the next Tab lands on the domain's first surviving stop.
//! This is also the regression fixture for a measured crash: before
//! `focus::discard_stale_focus` existed, the read-back right after the
//! click panicked with `index out of bounds`, since the retained id
//! outlived the projection that produced it.
//!
//! # Helpers copied from `event_routing_integration.rs`
//!
//! Cargo compiles every `tests/*.rs` file as its own crate, so nothing in
//! that file is importable here. `lower_ui_to_ir`, `load_window`,
//! `client_extent`, `display_limits`, `window_rect`, `frame_thickness`,
//! `send_dpi_change_to_client`, `normalise_to_reference_baseline`, and
//! `send_click` below are copied verbatim (per that file's own stated
//! copying convention, itself continuing the chain from
//! `hit_resolution_integration.rs`) from `event_routing_integration.rs`.
//! This file's own additions are the three key-message senders
//! (`send_key`, `send_tab`, `send_shift_tab`), the label / path readers
//! (`label_of`, `node_at_path`, `last_error`), the whole-tree hover-style
//! walk (`assert_no_button_family_node_paints_focused`), the paired
//! read-back assertion (`assert_focused_stop`), and the containment
//! mirror `dip_contains` — `hit` is a private module (`mod hit;` in
//! `lib.rs`), so `hit::contains` is unreachable from this crate and is
//! mirrored locally instead, with the same left/top-inclusive,
//! right/bottom-exclusive edge convention `hit.rs`'s own doc comment
//! states (the same copy `hover_transition_integration.rs` already made
//! for the same reason).
//!
//! # The client stays small (M4-Phase 1 T8 finding)
//!
//! Every fixture normalises to 96 DPI at an explicitly chosen 360x240
//! physical client — the same ceiling `event_routing_integration.rs`'s and
//! `hit_resolution_integration.rs`'s module headers record (a hosted CI
//! desktop failed a larger request twice). No fixture in this file ever
//! changes scale, so 360x240 is also each fixture's only client extent.
//! Every click coordinate is derived from `__arranged_rect_for_test()`
//! (the layout-derived DIP rectangle T1/T2 store) multiplied by the scale
//! factor the runtime **committed** at that baseline, never from a
//! hand-worked-out constant, and the geometric relationship each fixture
//! depends on (containment, disjointness, "this point is outside every
//! widget") is asserted before the point is used — a layout change that
//! broke the assumption then fails the fixture loudly instead of silently
//! testing nothing.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::cell::RefCell;
use std::ffi::{c_void, CStr};
use std::ptr;
use std::rc::Rc;

use wasamo_runtime::{ffi, DipRect, WidgetNode};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardState, SetKeyboardState, VK_SHIFT, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, GetClientRect, GetSystemMetrics, GetWindowRect, SendMessageW, SM_CXMAXTRACK,
    SM_CXSCREEN, SM_CYMAXTRACK, SM_CYSCREEN, WM_DPICHANGED, WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONUP,
    WM_SETFOCUS,
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

// ── Copied verbatim from `event_routing_integration.rs` ────────────────────

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<focus-traversal-integration>";
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
/// synchronously via `SendMessageW`. Suitable for every click in this file:
/// none needs a queued host-listener callback, only `hit_test_click`'s and
/// `focus_on_click`'s synchronous dispatch.
unsafe fn send_click(hwnd: HWND, x: f32, y: f32) {
    let packed = ((y.round() as i32 as u32) << 16) | (x.round() as i32 as u32 & 0xFFFF);
    SendMessageW(
        hwnd,
        WM_LBUTTONUP,
        WPARAM(0),
        LPARAM(packed as i32 as isize),
    );
}

// ── This file's own helpers ─────────────────────────────────────────────

/// A real `WM_KEYDOWN` for an arbitrary virtual-key code, with no modifier
/// state manipulated. Suitable for `VK_TAB` with no Shift ([`send_tab`])
/// and for any key traversal does not claim.
unsafe fn send_key(hwnd: HWND, vk: u16) -> LRESULT {
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(vk as usize), LPARAM(0))
}

/// A real `WM_KEYDOWN(VK_TAB)` with no Shift held. Reaches `window.rs`'s
/// `WM_KEYDOWN` arm, which calls `focus::traverse_on_key` before the
/// (uninstalled) host key slot.
unsafe fn send_tab(hwnd: HWND) -> LRESULT {
    send_key(hwnd, VK_TAB.0)
}

/// A real `WM_KEYDOWN(VK_TAB)` with Shift held, following the Step 0b
/// discipline this file's module header records: `SetKeyboardState` is
/// applied and then **read back** through `GetKeyState` before the message
/// is sent, rather than assumed to have taken effect (M4-Phase 1 F-49).
/// The previous keyboard state is restored unconditionally afterwards —
/// every fixture in this binary runs on the same single owning thread
/// (`common::run_on_owning_runtime_thread_or_skip`'s module header), so a
/// state left set here would still be set for the next fixture's plain
/// [`send_tab`] otherwise.
unsafe fn send_shift_tab(hwnd: HWND) -> LRESULT {
    let mut table = [0u8; 256];
    GetKeyboardState(&mut table).expect("GetKeyboardState");
    let saved = table;
    table[VK_SHIFT.0 as usize] |= 0x80;
    SetKeyboardState(&table).expect("SetKeyboardState");
    assert!(
        (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0,
        "Step 0b (M4-Phase 1 F-49 read-back): VK_SHIFT's high bit must actually read back \
         set before a Shift+Tab assertion is allowed to rely on it"
    );
    let result = send_key(hwnd, VK_TAB.0);
    SetKeyboardState(&saved).expect("SetKeyboardState restore");
    result
}

/// Read the last-error message through the C ABI, the same helper
/// `tests/focus_mechanism_fixture.rs` and `tests/common/mod.rs` each keep
/// their own copy of (neither is importable here — the former is a
/// separate `tests/*.rs` crate, the latter's copy is private).
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
/// (`wasamo_get_property`, `PROP_BUTTON_LABEL = 1` — `widget.rs`'s
/// constant of that name, redeclared locally since it is not `pub` at the
/// crate root). `tests/focus_mechanism_fixture.rs`'s module header states
/// why this matters more than it looks: a focus id / path is checked
/// against the same tree walk the implementation used, so it alone cannot
/// show focus landed on a *real* widget the fixture meant. Reading the
/// label back makes "it landed on the widget I meant" a claim about a
/// widget rather than about an index.
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
/// Panics on an out-of-range index, which would itself be a fixture
/// authoring error rather than something to route around.
fn node_at_path<'a>(root: &'a WidgetNode, path: &[usize]) -> &'a WidgetNode {
    let mut node: &WidgetNode = root;
    for &i in path {
        node = node.children[i].as_ref();
    }
    node
}

/// Walk `root`'s whole subtree (including `root` itself, via
/// `WidgetNode::for_each_ptr`) and assert every Button-family node's
/// painted focus indicator is `false`. Returns how many Button-family
/// nodes were visited, so a caller can assert the walk was not vacuous —
/// Fixture 1's "walk the tree and check them all; do not spot-check one".
fn assert_no_button_family_node_paints_focused(root: &WidgetNode, what: &str) -> usize {
    let mut visited = 0usize;
    root.for_each_ptr(&mut |ptr| {
        let node = unsafe { &*ptr };
        if let Some(focused) = node.__button_focused_for_test() {
            visited += 1;
            assert!(
                !focused,
                "{what}: a {} node must not paint the focus indicator when nothing is \
                 focused",
                node.__kind_name_for_test()
            );
        }
    });
    visited
}

/// The paired read-back this task's spec requires after every step where
/// either half of the focus pair could have changed: the retained record
/// names `expected_path`, the node there paints the indicator, its label
/// (read through the C ABI, not trusted from its tree position) is
/// `expected_label`, and — when `previous_path` is `Some` — the node the
/// record *previously* named has stopped painting
/// (implementation-gates trap #3, the same shape
/// `hover_transition_integration.rs` pins for hover). Pass `previous_path:
/// None` both for "this is the very first focus in the fixture" and for
/// "focus is unchanged from an already-asserted state" — the two cases
/// this helper cannot tell apart from the arguments alone, and does not
/// need to: the caller already knows which one it means.
unsafe fn assert_focused_stop(
    window: *mut ffi::WasamoWindow,
    expected_path: &[usize],
    expected_label: &str,
    previous_path: Option<&[usize]>,
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
    if let Some(prev) = previous_path {
        let prev_node = node_at_path(root, prev);
        assert_eq!(
            prev_node.__button_focused_for_test(),
            Some(false),
            "the previously focused node {prev:?} must have stopped painting the \
             indicator"
        );
    }
}

/// Install a recorder in the window's **host key slot** and hand back the
/// list it appends to.
///
/// `window.rs`'s `WM_KEYDOWN` arm calls `key_down_fn` only after
/// `focus::traverse_on_key` has declined the key ("routing consumes ahead
/// of that slot without installing it"), so what this list contains after
/// a message is the direct answer to "did traversal consume it" — the
/// question Step 0's measurement showed the returned `LRESULT` cannot
/// answer, because it is 0 either way. See this file's module header for
/// why writing this pre-existing `pub` field from a test is not the
/// shipped installer `constraints.md` §2 keeps out of this phase.
///
/// The closure is dropped with the window, so nothing outlives the
/// fixture that installed it.
unsafe fn install_key_slot_recorder(window: *mut ffi::WasamoWindow) -> Rc<RefCell<Vec<u16>>> {
    let seen: Rc<RefCell<Vec<u16>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    (*window).key_down_fn = Some(Box::new(move |vk| sink.borrow_mut().push(vk)));
    seen
}

/// Local mirror of `hit::contains`'s edge convention — **left/top
/// inclusive, right/bottom exclusive** — copied because `hit` is a private
/// module (`mod hit;` in `lib.rs`) and unreachable from this crate. The
/// same copy `hover_transition_integration.rs` already made for the same
/// reason.
fn dip_contains(rect: DipRect, x: f32, y: f32) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

// ── F1 — nothing focused at open; activation does not move it ──────────────

const NOTHING_FOCUSED_AT_OPEN_UI: &str = r#"component NothingFocusedAtOpen inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "alpha" }
        HStack {
            spacing: 0
            padding: 0
            ToggleButton { text: "beta" }
            Button { text: "gamma" }
        }
    }
}"#;

/// `docs/dsl_spec.md` §4.19 "Nothing is focused when a window opens" and
/// "Losing and regaining the window's activation does not change which
/// widget is focused" (`docs/architecture.md` §13.3 keeps Win32 activation
/// separate from internal focus: `window.rs`'s `wnd_proc` has no arm for
/// `WM_SETFOCUS` (0x0007) or `WM_KILLFOCUS` (0x0008) at all, so neither
/// message can touch `state.focus` — both simply fall through to
/// `DefWindowProcW`).
///
/// The first half (nothing focused at open) walks the **whole** tree
/// rather than spot-checking one Button-family node: three exist here
/// (`alpha`, `beta`, `gamma`), at two different nesting depths, so a walk
/// that only checked `root.children[0]` would miss a wrongly-painted
/// `beta` or `gamma`. **The second half is the discriminating one**: a
/// runtime that never focuses anything at all would also pass every
/// assertion before the Tab press. This fixture Tabs once, sends
/// `WM_KILLFOCUS` then `WM_SETFOCUS`, and asserts the focused stop is
/// exactly where it was.
#[test]
fn nothing_is_focused_when_a_window_opens_and_window_activation_does_not_change_it() {
    run_on_owning_runtime_thread_or_skip(
        "F1: nothing focused at open, activation is inert",
        move || {
            let ir = lower_ui_to_ir(NOTHING_FOCUSED_AT_OPEN_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F1 baseline");

                // Establish the initial focus state by reading it back (M4-Phase 1
                // F-47), rather than assuming a freshly loaded window starts unfocused.
                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "docs/dsl_spec.md §4.19: nothing is focused when a window opens"
                );
                let visited = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_no_button_family_node_paints_focused(root, "F1 initial state")
                };
                assert_eq!(
                    visited, 3,
                    "fixture stopped discriminating: this fixture's tree must contain \
                     exactly the three Button-family nodes (alpha, beta, gamma) the walk \
                     is meant to cover, or the walk above is not exercising 'check them \
                     all'"
                );

                // WM_SETFOCUS / WM_KILLFOCUS on a window that never had keyboard focus:
                // still must not create one.
                SendMessageW(hwnd, WM_SETFOCUS, WPARAM(0), LPARAM(0));
                SendMessageW(hwnd, WM_KILLFOCUS, WPARAM(0), LPARAM(0));
                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "WM_SETFOCUS / WM_KILLFOCUS with nothing focused must not create a focus"
                );

                // Tab once: focus moves to the first stop (alpha).
                let tab_result = send_tab(hwnd);
                assert_eq!(tab_result.0, 0, "Tab must be consumed (LRESULT(0))");
                assert_focused_stop(window, &[0], "alpha", None);

                // The discriminating half: losing and regaining activation must not
                // move the focused stop.
                SendMessageW(hwnd, WM_KILLFOCUS, WPARAM(0), LPARAM(0));
                SendMessageW(hwnd, WM_SETFOCUS, WPARAM(0), LPARAM(0));
                assert_focused_stop(window, &[0], "alpha", None);

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── F2 — Tab walks declaration order and wraps at both ends ────────────────

const TAB_ORDER_UI: &str = r#"component TabOrderWithInterleavedNonStops inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Text { text: "header" }
        Button { text: "stop-one" }
        Box { aspect: 4:1 fill: #33669988 }
        Button { text: "stop-two" }
        Text { text: "footer" }
        Button { text: "stop-three" }
    }
}"#;

/// `docs/dsl_spec.md` §4.19: "Tab / Shift+Tab move focus in declaration
/// order, wrapping at both ends; the first Tab lands on the first stop."
///
/// The `Text` / `Box` widgets are interleaved between the three Button
/// stops so that "declaration order of stops" and "declaration order of
/// children" are two different lists: `root.children` has six entries,
/// but the stop order is only `[1, 3, 5]`. A traversal that (wrongly)
/// walked `root.children` directly rather than through
/// `FocusTree::tab_stops` would visibly disagree with the assertions
/// below (landing on `header`'s or the `Box`'s position, neither of which
/// is a Button-family node at all) rather than coincidentally agree.
#[test]
fn tab_walks_the_stops_in_declaration_order_and_wraps_at_both_ends() {
    run_on_owning_runtime_thread_or_skip("F2: declaration order and wrap", move || {
        let ir = lower_ui_to_ir(TAB_ORDER_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F2 baseline");

            assert_eq!(
                ffi::__focus_path_for_test(window),
                None,
                "M4-Phase 1 F-47: establish the initial focus state by reading it back"
            );

            // First Tab lands on the first stop, not the second.
            send_tab(hwnd);
            assert_focused_stop(window, &[1], "stop-one", None);

            // Second Tab: the second stop.
            send_tab(hwnd);
            assert_focused_stop(window, &[3], "stop-two", Some(&[1]));

            // Third Tab: the third (last) stop.
            send_tab(hwnd);
            assert_focused_stop(window, &[5], "stop-three", Some(&[3]));

            // Forward wrap: from the last stop back to the first.
            send_tab(hwnd);
            assert_focused_stop(window, &[1], "stop-one", Some(&[5]));

            // Shift+Tab: backward wrap from the first stop to the last.
            send_shift_tab(hwnd);
            assert_focused_stop(window, &[5], "stop-three", Some(&[1]));

            // Shift+Tab again: the second-to-last stop.
            send_shift_tab(hwnd);
            assert_focused_stop(window, &[3], "stop-two", Some(&[5]));

            ffi::wasamo_window_destroy(window);
        }
    });
}

// ── F3 — a disabled stop is skipped; disabling while focused is lazy ───────

/// The difference leg's tree. `beta`'s `enabled: false` is a **literal on
/// a `ToggleButton`**, not a plain `Button` — see the fixture's own doc
/// comment (CF-2) for why.
const DISABLED_MIDDLE_STOP_UI: &str = r#"component DisabledMiddleStopIsSkipped inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "alpha" }
        ToggleButton { text: "beta" enabled: false }
        Button { text: "gamma" }
    }
}"#;

/// The agreement leg's tree: identical structure, `beta` enabled.
const MIDDLE_STOP_ENABLED_UI: &str = r#"component MiddleStopIsReachableWhenEnabled inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "alpha" }
        ToggleButton { text: "beta" enabled: true }
        Button { text: "gamma" }
    }
}"#;

/// The third leg's tree. `beta`'s `enabled` is **state-bound** on a plain
/// `Button` this time — the third leg needs the dynamic form regardless of
/// the CF-2 workaround the first two legs use, because it disables `beta`
/// *after* it already holds focus. The flip control is a `Box`, not a
/// Button, **deliberately**: `focus::focus_on_click` moves focus to the
/// nearest focusable widget at or above the click target *before*
/// dispatch runs (`window.rs`'s `WM_LBUTTONUP` arm), so a Button flip
/// control would itself steal focus from `beta` the instant it is
/// clicked, defeating this leg's premise ("focus a stop, then disable
/// *that* stop while it still holds focus"). A `Box` has no focusable
/// ancestor, so clicking it cannot move focus at all
/// (`docs/dsl_spec.md` §4.19 "clicking background never clears focus") —
/// the same fact Fixture 4's leg B pins directly.
const DISABLE_WHILE_FOCUSED_UI: &str = r#"component DisableWhileFocusedLosesItAtNextTab inherits Window {
    state middle_enabled: bool = true
    VStack {
        spacing: 0
        padding: 0
        Button { text: "alpha" }
        Button { text: "beta" enabled: middle_enabled }
        Button { text: "gamma" }
        Box { aspect: 4:1 fill: #336699cc clicked => { root.middle_enabled = false; } }
    }
}"#;

/// Three legs. `docs/dsl_spec.md` §4.19 / §4.8: "A Button with
/// `enabled: false` is not focusable — it is skipped by traversal."
///
/// **A literal `enabled: false` on a plain `Button` is silently dropped by
/// the IR loader** (M4-Phase 2 T3 carry-forward CF-2, owned by T8, not yet
/// fixed at this task): `ir_loader::construct_widget`'s `"Button"` arm
/// builds the widget from `text` / `style` only and never reads an
/// `enabled` prop at all, unlike its `"ToggleButton"` sibling arm, which
/// reads `extract_bool_prop(&node.props, "enabled")` as a literal fallback
/// beside its binding check. The difference / agreement legs below
/// therefore author the disabled stop as a `ToggleButton` with the
/// literal, which does land.
///
/// - **Difference leg**: three stops, the middle one (`beta`) disabled.
///   Two Tabs land on the third (`gamma`), not the second.
/// - **Agreement leg**: the identical tree with `beta` enabled. Two Tabs
///   land on `beta` — without this leg, "Tab skips position two" would be
///   indistinguishable from "position two is never reachable at all".
/// - **Third leg**: `beta` is focused first, then disabled through the
///   binding path while still focused, then Tab is pressed once.
///   DD-M4-P2-003's boundary
///   ([log.md §T5](../../process/milestone-4/phase-2/implementation/log.md#t5--per-window-focus-state-and-tab-traversal),
///   "the successor rule applies lazily, at the next traversal"): focus
///   lands on the domain's **first** stop (`alpha`), not on `gamma` (which
///   an eager "find the next enabled stop after beta's old position" rule
///   would have chosen) and not on `beta` itself (which is no longer a
///   stop at all).
#[test]
fn a_disabled_button_is_skipped_by_traversal_and_a_button_disabled_while_focused_loses_it_at_the_next_tab(
) {
    run_on_owning_runtime_thread_or_skip("F3: disabled skip + lazy successor", move || {
        unsafe {
            // Difference leg.
            {
                let ir = lower_ui_to_ir(DISABLED_MIDDLE_STOP_UI);
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(
                    window,
                    CLIENT_W,
                    CLIENT_H,
                    "F3 difference baseline",
                );
                assert_eq!(ffi::__focus_path_for_test(window), None);
                {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let beta = node_at_path(root, &[1]);
                    assert_eq!(
                        beta.__button_enabled_for_test(),
                        Some(false),
                        "fixture stopped discriminating: beta must actually be disabled, or \
                         the difference leg below proves nothing"
                    );
                }
                send_tab(hwnd);
                assert_focused_stop(window, &[0], "alpha", None);
                send_tab(hwnd);
                assert_focused_stop(window, &[2], "gamma", Some(&[0]));
                {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let beta = node_at_path(root, &[1]);
                    assert_eq!(
                        beta.__button_focused_for_test(),
                        Some(false),
                        "the disabled middle stop must never have painted the indicator"
                    );
                }
                ffi::wasamo_window_destroy(window);
            }

            // Agreement leg.
            {
                let ir = lower_ui_to_ir(MIDDLE_STOP_ENABLED_UI);
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(
                    window,
                    CLIENT_W,
                    CLIENT_H,
                    "F3 agreement baseline",
                );
                assert_eq!(ffi::__focus_path_for_test(window), None);
                send_tab(hwnd);
                assert_focused_stop(window, &[0], "alpha", None);
                send_tab(hwnd);
                assert_focused_stop(window, &[1], "beta", Some(&[0]));
                ffi::wasamo_window_destroy(window);
            }

            // Third leg: disable the focused stop dynamically.
            {
                let ir = lower_ui_to_ir(DISABLE_WHILE_FOCUSED_UI);
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(
                    window,
                    CLIENT_W,
                    CLIENT_H,
                    "F3 third-leg baseline",
                );
                assert_eq!(ffi::__focus_path_for_test(window), None);

                send_tab(hwnd);
                assert_focused_stop(window, &[0], "alpha", None);
                send_tab(hwnd);
                assert_focused_stop(window, &[1], "beta", Some(&[0]));
                {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let beta = node_at_path(root, &[1]);
                    assert_eq!(
                        beta.__button_enabled_for_test(),
                        Some(true),
                        "fixture stopped discriminating: beta must start enabled, or \
                         disabling it below proves nothing changed"
                    );
                }

                let (alpha_rect, beta_rect, gamma_rect, flip_rect) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    (
                        root.children[0].__arranged_rect_for_test(),
                        root.children[1].__arranged_rect_for_test(),
                        root.children[2].__arranged_rect_for_test(),
                        root.children[3].__arranged_rect_for_test(),
                    )
                };
                let alpha_rect = alpha_rect.expect("alpha must be laid out");
                let beta_rect = beta_rect.expect("beta must be laid out");
                let gamma_rect = gamma_rect.expect("gamma must be laid out");
                let flip_rect = flip_rect.expect("flip Box must be laid out");
                for (label, rect) in [
                    ("alpha", alpha_rect),
                    ("beta", beta_rect),
                    ("gamma", gamma_rect),
                    ("flip", flip_rect),
                ] {
                    assert!(
                        rect.width > 0.0 && rect.height > 0.0,
                        "fixture stopped discriminating: the {label} rectangle {rect:?} \
                         must be non-degenerate"
                    );
                }
                assert!(
                    alpha_rect.y + alpha_rect.height <= beta_rect.y,
                    "fixture stopped discriminating: alpha and beta must be disjoint"
                );
                assert!(
                    beta_rect.y + beta_rect.height <= gamma_rect.y,
                    "fixture stopped discriminating: beta and gamma must be disjoint"
                );
                assert!(
                    gamma_rect.y + gamma_rect.height <= flip_rect.y,
                    "fixture stopped discriminating: gamma and the flip Box must be disjoint"
                );

                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
                let (flip_cx, flip_cy) = (
                    (flip_rect.x + flip_rect.width / 2.0) * factor,
                    (flip_rect.y + flip_rect.height / 2.0) * factor,
                );
                send_click(hwnd, flip_cx, flip_cy);

                {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let beta = node_at_path(root, &[1]);
                    assert_eq!(
                        beta.__button_enabled_for_test(),
                        Some(false),
                        "the flip Box's handler must have disabled beta synchronously"
                    );
                }
                // Clicking the Box must not have moved focus at all — it has no
                // focusable ancestor (docs/dsl_spec.md §4.19 "clicking background
                // never clears focus"), which is what lets beta still be the
                // record's answer even though it just became disabled.
                assert_focused_stop(window, &[1], "beta", None);

                // The successor rule applies lazily, at this Tab: focus lands on
                // the domain's first stop (alpha), not on gamma.
                send_tab(hwnd);
                assert_focused_stop(window, &[0], "alpha", Some(&[1]));

                ffi::wasamo_window_destroy(window);
            }
        }
    });
}

// ── F4 — click-to-focus: nearest focusable at/above; background is inert ───

const CLICK_FOCUS_UI: &str = r#"component ClickFocusesNearestAndBackgroundChangesNothing inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "clickable" }
        Text { text: "plain text, no focusable ancestor" }
        Box { aspect: 4:1 fill: #33669988 }
    }
}"#;

/// `docs/dsl_spec.md` §4.19: "A click moves focus to the nearest
/// focusable widget at or above the widget it resolved to, and leaves
/// focus unchanged when there is none — clicking background never clears
/// focus."
///
/// **The "at or above" leg is not built here.** It would need a
/// focusable ancestor with a hit-testable, non-focusable descendant
/// underneath it in the *same* subtree — a click on the descendant
/// focusing the ancestor. `WidgetNode::build_layout_tree`
/// (`wasamo-runtime/src/widget.rs`) maps every `Button` / `ToggleButton`
/// node to `LayoutNode::rectangle(...)` and never reads `self.children`
/// for that arm, so a Button-family node's own children never receive an
/// `arranged_rect` and can never be `resolve_topmost`'s target (T3
/// start-gate finding 1, already measured before this task existed).
/// Since only Button-family nodes are `FocusRole::Stop`
/// (`WidgetNode::focus_role`), the only kind of node that could be the
/// "focusable ancestor" this leg needs can never have a hit-testable
/// child to click on instead. This is a stated limit, not an oversight:
/// the three legs below (focus, unchanged-on-non-focusable,
/// unchanged-on-nothing) are this file's complete click-to-focus
/// evidence.
#[test]
fn a_click_focuses_the_nearest_focusable_widget_at_or_above_the_target_and_a_background_click_changes_nothing(
) {
    run_on_owning_runtime_thread_or_skip(
        "F4: click-to-focus and background inertness",
        move || {
            let ir = lower_ui_to_ir(CLICK_FOCUS_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F4 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "M4-Phase 1 F-47: establish the initial focus state by reading it back"
                );

                let (button_rect, text_rect, box_rect) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    (
                        root.children[0].__arranged_rect_for_test(),
                        root.children[1].__arranged_rect_for_test(),
                        root.children[2].__arranged_rect_for_test(),
                    )
                };
                let button_rect = button_rect.expect("Button must have been laid out");
                let text_rect = text_rect.expect("Text must have been laid out");
                let box_rect = box_rect.expect("Box must have been laid out");
                for (label, rect) in [
                    ("Button", button_rect),
                    ("Text", text_rect),
                    ("Box", box_rect),
                ] {
                    assert!(
                        rect.width > 0.0 && rect.height > 0.0,
                        "fixture stopped discriminating: the {label} rectangle {rect:?} must be \
                     non-degenerate"
                    );
                }
                assert!(
                    button_rect.y + button_rect.height <= text_rect.y,
                    "fixture stopped discriminating: Button {button_rect:?} and Text \
                 {text_rect:?} must be disjoint, or a click meant for one could resolve to \
                 the other"
                );
                assert!(
                    text_rect.y + text_rect.height <= box_rect.y,
                    "fixture stopped discriminating: Text {text_rect:?} and Box {box_rect:?} \
                 must be disjoint"
                );

                // Leg A: a click on the Button focuses it.
                let (button_cx, button_cy) = (
                    (button_rect.x + button_rect.width / 2.0) * factor,
                    (button_rect.y + button_rect.height / 2.0) * factor,
                );
                send_click(hwnd, button_cx, button_cy);
                assert_focused_stop(window, &[0], "clickable", None);

                // Leg B: a click on the Text (no focusable ancestor) leaves the
                // previously focused stop unchanged AND still painted — the pair
                // that distinguishes "unchanged" from "cleared".
                let (text_cx, text_cy) = (
                    (text_rect.x + text_rect.width / 2.0) * factor,
                    (text_rect.y + text_rect.height / 2.0) * factor,
                );
                send_click(hwnd, text_cx, text_cy);
                assert_focused_stop(window, &[0], "clickable", None);

                // Leg C: a click below every child, in the empty space the root
                // VStack still covers. `run_layout_as_window_root_at_scale` forces
                // the root's constraints to `Fill`, so this point resolves to the
                // **root container** rather than to nothing — a different case
                // from leg B (which resolves to a `Text` leaf) and the one that
                // matters most, since the root is the last entry of every dispatch
                // chain and is a `Container`, never a stop.
                let (outside_x, outside_y) = (
                    box_rect.x + box_rect.width / 2.0,
                    (box_rect.y + box_rect.height + CLIENT_H as f32) / 2.0,
                );
                assert!(
                    !dip_contains(button_rect, outside_x, outside_y)
                        && !dip_contains(text_rect, outside_x, outside_y)
                        && !dip_contains(box_rect, outside_x, outside_y),
                    "fixture stopped discriminating: ({outside_x}, {outside_y}) must be outside \
                 every child of the root, or this leg is not testing a click that lands on no \
                 widget of its own at all"
                );
                send_click(hwnd, outside_x * factor, outside_y * factor);
                assert_focused_stop(window, &[0], "clickable", None);

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── F5 — Tab is consumed by traversal; an unclaimed key is not ─────────────

const SINGLE_STOP_UI: &str = r#"component TabConsumedWhileUnclaimedKeyFallsThrough inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "only-stop" }
    }
}"#;

const NO_FOCUSABLE_WIDGET_UI: &str = r#"component TabConsumedWithNoFocusableStopAtAll inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Text { text: "just text" }
        Box { aspect: 4:1 fill: #33669988 }
    }
}"#;

/// An arbitrary key traversal does not claim (not `Tab`). Used only for
/// the fallthrough leg; the same convention `focus_core.rs`'s own unit
/// test module uses for its `VK_A`.
const VK_A: u16 = 0x41;

/// Step 0's measurement (this file's module header) rules one assertion
/// shape out and points at another. **Every** candidate key returned
/// `LRESULT(0)` from both a `SendMessageW` through the real `wnd_proc`
/// and a direct `DefWindowProcW` call, so the negative prediction the T5
/// start gate flagged as unmeasured is **confirmed**: the `LRESULT` alone
/// cannot distinguish "traversal declined this key" from "some unrelated
/// path also returned 0".
///
/// **What discriminates is the host key slot.** `window.rs`'s `WM_KEYDOWN`
/// arm reaches `key_down_fn` only after `focus::traverse_on_key` has
/// declined the key — "routing consumes ahead of that slot without
/// installing it" — so a recorder installed there answers the consumption
/// question outright. Each leg below asserts what the recorder saw, and
/// each would fail against the opposite behaviour:
///
/// - **Leg 1, `Tab` with one stop**: the recorder stays empty and focus
///   moves. A traversal that declined `Tab` would put `VK_TAB` in the
///   recorder; one that consumed without moving focus would fail the
///   second half.
/// - **Leg 1, an unclaimed key**: the recorder receives exactly that key
///   and focus does not move. A traversal that over-claimed — consuming
///   anything that is not `Tab` — would leave the recorder empty. The
///   `LRESULT` is additionally compared against a **live** direct
///   `DefWindowProcW` call for the same message rather than a hard-coded
///   `0`; that comparison is a measured equality and, on its own, **not**
///   evidence that the key travelled the fallthrough, which is the
///   coincidence the module header names.
/// - **Leg 2, `Tab` with no focusable widget at all**: the recorder stays
///   empty while focus stays `None`. `docs/dsl_spec.md` §4.19 "`Tab` /
///   `Shift+Tab` — Always the runtime; traversal cannot be overridden"
///   makes no exception for an empty traversal domain, and
///   `focus::traverse_on_key`'s doc comment names this branch explicitly
///   ("Tab over a window with nothing focusable is still Tab's to
///   consume — it just moves focus nowhere"). Without the recorder this
///   leg could not exist at all: consumed and declined both return
///   `LRESULT(0)` and both leave focus `None`, so an `LRESULT` assertion
///   here would have passed against either behaviour.
#[test]
fn tab_is_consumed_by_traversal_while_an_unclaimed_key_is_not() {
    run_on_owning_runtime_thread_or_skip(
        "F5: Tab consumed, unclaimed key falls through",
        move || {
            unsafe {
                // Leg 1: a tree with one stop.
                {
                    let ir = lower_ui_to_ir(SINGLE_STOP_UI);
                    let window = load_window(&ir);
                    let hwnd = (*window).hwnd;
                    normalise_to_reference_baseline(
                        window,
                        CLIENT_W,
                        CLIENT_H,
                        "F5 leg 1 baseline",
                    );

                    assert_eq!(ffi::__focus_path_for_test(window), None);

                    let seen = install_key_slot_recorder(window);
                    assert!(
                        seen.borrow().is_empty(),
                        "fixture stopped discriminating: the recorder must start empty"
                    );

                    let tab_result = send_tab(hwnd);
                    assert_eq!(tab_result.0, 0, "Tab must be consumed: LRESULT(0)");
                    assert_focused_stop(window, &[0], "only-stop", None);
                    assert!(
                        seen.borrow().is_empty(),
                        "routing consumes ahead of the host key slot: a Tab traversal claimed \
                     must never reach key_down_fn, and the recorder saw {:?}",
                        seen.borrow()
                    );

                    let direct_def =
                        DefWindowProcW(hwnd, WM_KEYDOWN, WPARAM(VK_A as usize), LPARAM(0));
                    let key_result = send_key(hwnd, VK_A);
                    assert_eq!(
                        key_result.0, direct_def.0,
                        "an unclaimed key's fallthrough must return the same LRESULT a direct \
                     DefWindowProcW call returns for the same message — measured, not \
                     assumed to be 0"
                    );
                    assert_eq!(
                        seen.borrow().as_slice(),
                        &[VK_A],
                        "a key traversal does not claim must reach the host key slot, which is \
                     what makes 'consumed' and 'declined' distinguishable at all (the LRESULT \
                     is 0 either way — see this file's Step 0 measurement)"
                    );
                    // Still the discriminating half for focus: the unclaimed key
                    // must not have moved it.
                    assert_focused_stop(window, &[0], "only-stop", None);

                    ffi::wasamo_window_destroy(window);
                }

                // Leg 2: a tree with no focusable widget at all.
                {
                    let ir = lower_ui_to_ir(NO_FOCUSABLE_WIDGET_UI);
                    let window = load_window(&ir);
                    let hwnd = (*window).hwnd;
                    normalise_to_reference_baseline(
                        window,
                        CLIENT_W,
                        CLIENT_H,
                        "F5 leg 2 baseline",
                    );

                    assert_eq!(ffi::__focus_path_for_test(window), None);
                    let seen = install_key_slot_recorder(window);
                    let tab_result = send_tab(hwnd);
                    assert_eq!(
                        tab_result.0, 0,
                        "Tab must be consumed even when the domain has no stop at all \
                     (docs/dsl_spec.md §4.19; focus::traverse_on_key's doc comment names \
                     this branch explicitly)"
                    );
                    assert!(
                        seen.borrow().is_empty(),
                        "the assertion that can actually fail here: with nothing focusable, \
                     traversal still claims Tab, so it must not reach the host key slot. The \
                     LRESULT above is 0 whether the key was consumed or declined, and focus \
                     is None either way — the recorder is the only thing that tells the two \
                     apart. It saw {:?}",
                        seen.borrow()
                    );
                    assert_eq!(
                        ffi::__focus_path_for_test(window),
                        None,
                        "with nothing focusable, traversal has consumed the key but moved focus \
                     nowhere"
                    );

                    ffi::wasamo_window_destroy(window);
                }
            }
        },
    );
}

// ── F6 — a focus record surviving removal of its own stop ──────────────────

/// A stop outside a conditional subtree ("toolbar"), and two stops inside
/// it ("inner-a", "closer"); "closer" removes the whole conditional
/// subtree — itself included — from its own `clicked` handler. Shaped
/// after `examples/gallery/gallery.ui`'s lightbox, whose `"x"` Button
/// reaches this same defect: the lightbox is an `if is_lightbox_open`
/// branch, and its Buttons are the highest pre-order focus ids in the
/// tree, so focusing then removing "x" is the shipped case this fixture
/// stands in for with a smaller tree.
const FOCUS_RECORD_SURVIVES_REMOVAL_OF_ITS_OWN_STOP_UI: &str = r#"component FocusRecordSurvivesRemovalOfItsOwnStop inherits Window {
    state open: bool = true
    VStack {
        spacing: 0
        padding: 0
        Button { text: "toolbar" }
        if open {
            VStack {
                spacing: 0
                padding: 0
                Button { text: "inner-a" }
                Button { text: "closer" clicked => { root.open = false; } }
            }
        }
    }
}"#;

/// `docs/architecture.md` §13.3 / DD-M4-P2-003 ("The focused node
/// disappearing"): `WindowFocus` retains a `focus_core::FocusId`, which is
/// the pre-order index of `crate::focus::FocusProjection` — rebuilt fresh
/// on every operation. In this fixture's pre-order, "closer" is id 4
/// (root 0, toolbar 1, the conditional `VStack` 2, inner-a 3, closer 4).
/// `window.rs`'s `WM_LBUTTONUP` arm calls `focus::focus_on_click` — which
/// focuses "closer" against the tree as it stood *before* dispatch —
/// **before** `hit_test_click` runs "closer"'s own handler, whose
/// synchronous drain removes the conditional subtree "closer" lives in.
/// The next projection has only 2 nodes (root, toolbar), so the retained
/// id 4 names nothing in it.
///
/// **This is also the regression fixture for a measured crash.** Before
/// `focus::discard_stale_focus` existed, the `ffi::__focus_path_for_test`
/// read-back immediately after the click panicked:
/// `thread ... panicked at wasamo-runtime\src\focus.rs:63:20: index out of
/// bounds: the len is 2 but the index is 4` — `FocusProjection::path`
/// indexing `self.paths[id]` directly, reached through
/// `focus::focused_path`. A regression that removes the fix therefore
/// makes this test **crash**, not merely fail an assertion.
///
/// Three things are asserted, matching the three call paths
/// `focus::discard_stale_focus`'s doc comment enumerates:
///
/// - The read-back right after the click is `None`, not a stale path —
///   call path 3 (`focused_path`, this is where the probe panicked).
/// - A subsequent `Tab` does not panic and lands on the domain's first
///   surviving stop ("toolbar") — call path 2
///   (`focus_core::FocusTree::tab` -> `stop_index_of` -> `nearest`),
///   reached only because `discard_stale_focus` clears the record before
///   `traverse_on_key` calls into `focus_core`, which this task does not
///   modify.
/// - `assert_focused_stop`'s paired read-back on that landing confirms
///   call path 1's counterpart (`move_focus`) wrote a real, explainable
///   id rather than propagating a stale one.
#[test]
fn a_focus_record_naming_a_removed_stop_is_cleared_rather_than_read_back_stale() {
    run_on_owning_runtime_thread_or_skip(
        "F6: focus record survives removal of its own stop",
        move || {
            let ir = lower_ui_to_ir(FOCUS_RECORD_SURVIVES_REMOVAL_OF_ITS_OWN_STOP_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F6 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "M4-Phase 1 F-47: establish the initial focus state by reading it back"
                );

                // Before the click: the conditional subtree is present, and
                // "closer" is the widget this fixture means to click, not
                // merely a path that happens to resolve to it.
                let (toolbar_rect, inner_a_rect, closer_rect) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        2,
                        "fixture stopped discriminating: `open` starts true, so the \
                         conditional VStack must be present before the click"
                    );
                    let conditional = root.children[1].as_ref();
                    assert_eq!(
                        conditional.child_count(),
                        2,
                        "fixture stopped discriminating: the conditional subtree must \
                         hold both 'inner-a' and 'closer' before the click"
                    );
                    assert_eq!(label_of(root.children[0].as_ref()), "toolbar");
                    assert_eq!(label_of(conditional.children[0].as_ref()), "inner-a");
                    assert_eq!(label_of(conditional.children[1].as_ref()), "closer");
                    (
                        root.children[0].__arranged_rect_for_test(),
                        conditional.children[0].__arranged_rect_for_test(),
                        conditional.children[1].__arranged_rect_for_test(),
                    )
                };
                let toolbar_rect = toolbar_rect.expect("toolbar must be laid out");
                let inner_a_rect = inner_a_rect.expect("inner-a must be laid out");
                let closer_rect = closer_rect.expect("closer must be laid out");
                for (label, rect) in [
                    ("toolbar", toolbar_rect),
                    ("inner-a", inner_a_rect),
                    ("closer", closer_rect),
                ] {
                    assert!(
                        rect.width > 0.0 && rect.height > 0.0,
                        "fixture stopped discriminating: the {label} rectangle {rect:?} must \
                     be non-degenerate"
                    );
                }
                assert!(
                    toolbar_rect.y + toolbar_rect.height <= inner_a_rect.y,
                    "fixture stopped discriminating: toolbar and inner-a must be disjoint"
                );
                assert!(
                    inner_a_rect.y + inner_a_rect.height <= closer_rect.y,
                    "fixture stopped discriminating: inner-a and closer must be disjoint"
                );

                // Click "closer": `focus_on_click` focuses it before
                // `hit_test_click` runs its handler, which removes the
                // subtree "closer" itself lives in — the same-message
                // hazard this fixture pins.
                let (closer_cx, closer_cy) = (
                    (closer_rect.x + closer_rect.width / 2.0) * factor,
                    (closer_rect.y + closer_rect.height / 2.0) * factor,
                );
                send_click(hwnd, closer_cx, closer_cy);

                // After the click: the conditional subtree is gone.
                {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        1,
                        "the handler's own state write ('root.open = false') must have \
                         removed the conditional subtree synchronously, or this fixture is \
                         not exercising a removal at all"
                    );
                }
                // The read-back itself is part of the assertion, and is
                // where the probe that measured this defect panicked
                // before the fix (this fixture's own doc comment).
                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "a focus record naming a removed stop must be cleared, not read back as \
                     a stale path into a tree that no longer has it"
                );

                // Tab once: the lazy successor rule lands on the domain's
                // first surviving stop, not anywhere the removed subtree
                // used to be. This also exercises call path 2
                // (`focus_core::FocusTree::tab`), which panics on a stale
                // id left uncleared, same as the read-back above.
                send_tab(hwnd);
                assert_focused_stop(window, &[0], "toolbar", None);

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}
