//! Mock-free Windows integration evidence for M4-Phase 2 T3 stage 2
//! ([DD-M4-P2-001](../../process/milestone-4/phase-2/decisions/dd-m4-p2-001-event-routing-model.md)):
//! click propagation over a real widget tree built from real `.ui` source
//! through `wasamoc` -> IR -> the loader, driven by real `WM_LBUTTONUP`
//! messages through the real window procedure, with live widget state read
//! back afterwards. Nothing is mocked.
//!
//! # What each fixture is for
//!
//! - [`a_click_on_a_widget_without_a_handler_stays_silent_while_a_click_on_a_sibling_with_one_runs_it`]
//!   — the central claim: `clicked` is admitted on **any** widget
//!   (docs/dsl_spec.md §4.19 "Click handling on any widget"), not only the
//!   Button family. Red before T3, green after: `wasamoc check` already
//!   accepted `clicked` on a `Box`, and `ir_loader.rs`'s `build_node`
//!   attaches every handler via `widget.set_inline_handler(...)`
//!   unconditionally on widget kind — but the pre-T3 `hit_test_click`
//!   returned immediately whenever the resolved target's
//!   `button_data_mut()` was `None`. Measured directly against this
//!   stage's own start-of-work `git diff` for `wasamo-runtime/src/widget.rs`:
//!   the very first thing the old body did after resolving the target was
//!   `let Some(btn) = target.button_data_mut() else { return; };`. A
//!   `Box`'s handler could not fire no matter what the checker and loader
//!   agreed to accept.
//! - [`an_ancestor_handler_runs_only_until_a_nested_widget_gets_one_of_its_own`]
//!   — the ancestor walk and DD-M4-P2-001's named consumption control:
//!   "the phase's evidence includes a control where a nested handler is
//!   added and the ancestor is shown to stop firing"
//!   ([dd-m4-p2-001 §Technical risk re-evaluation](../../process/milestone-4/phase-2/decisions/dd-m4-p2-001-event-routing-model.md)).
//! - [`a_disabled_button_suppresses_its_own_handler_without_stopping_propagation`]
//!   — a disabled Button still occludes and still dispatches nothing, but
//!   "having run no handler, it also does not end propagation: the event
//!   continues to its ancestors" (docs/dsl_spec.md §4.19; contract restated
//!   at §4.8). Paired with an enabled Button in the same structural
//!   position that *does* consume — without that agreement leg, an
//!   implementation that simply never reached the ancestor arm at all
//!   would look identical to one where the disabled Button correctly lets
//!   the event through.
//! - [`a_handler_that_removes_its_own_widget_consumes_the_click_and_leaves_no_registration_to_fire`]
//!   — a handler whose state write destroys the very node the walk is
//!   standing on. The walk is shown to have consumed there (the ancestor
//!   never runs), and a host listener connected to that node beforehand is
//!   shown never to fire, because its registration is severed with the
//!   subtree — synchronously, inside the handler's own state write —
//!   before `run_clicked_handlers` reaches the host-listener enqueue step
//!   (docs/dsl_spec.md §4.19 "The ancestor chain is fixed when the event
//!   is dispatched, so a widget removed by a handler's own state write
//!   does not receive the event afterwards").
//! - [`a_host_signal_listener_on_a_non_button_widget_consumes_the_walk_until_disconnected`]
//!   — a host listener connected through the real ABI
//!   (`wasamo_signal_connect`) on a plain `Box` consumes the walk exactly
//!   as an inline handler would, and stops doing so once
//!   `wasamo_signal_disconnect`ed.
//! - [`a_disabled_button_occludes_a_lower_sibling_it_paints_over`] (M4-Phase
//!   2 T8) — the plan's other named `enabled` evidence: a disabled Button
//!   still occludes what is behind it, now that a `Box` beneath it in a
//!   `ZStack` can carry `clicked` too. [`a_disabled_button_suppresses_its_own_handler_without_stopping_propagation`]
//!   above covers the *ancestor* direction (the click continues upward);
//!   this fixture covers occlusion of a *lower sibling*, which only became
//!   observable once the runtime dispatches a `Box`'s `clicked`.
//!
//! # Why an inline handler is observable synchronously and a host listener needs the loop pump
//!
//! An inline DSL handler's state write (`reactive::Signal::set`) drains its
//! reactive Effects **synchronously**, at zero batch depth — see the doc
//! comment on `run_clicked_handlers` in `widget.rs`, next to
//! `hit_test_click`. So a plain [`send_click`] (`SendMessageW`) is enough
//! to observe one: the Effect that writes a bound `Text.content` runs
//! before `SendMessageW` returns, and `__text_content_for_test()` reads
//! that `String` field directly — it is not a layout product, so no
//! re-layout has to happen for the read to be current.
//!
//! A host listener is different. `run_clicked_handlers` routes it through
//! `crate::emit::enqueue_signal`, which only **queues** the callback
//! (abi_spec §6 "emissions triggered by the call are queued and drained
//! after the call returns"); the callback itself fires inside
//! `emit::drain_if_outermost`'s Phase 3, and that function's production
//! call site is the line after `DispatchMessageW` in `wasamo_runtime::run`
//! — nowhere `SendMessageW` reaches. The two fixtures that connect a host
//! listener ([`a_handler_that_removes_its_own_widget_consumes_the_click_and_leaves_no_registration_to_fire`],
//! [`a_host_signal_listener_on_a_non_button_widget_consumes_the_walk_until_disconnected`])
//! therefore use [`click_and_drain`], which posts the click and pumps
//! `run()` once — the same mechanism `hit_resolution_integration.rs`'s "A
//! measured finding" section establishes.
//!
//! # Helpers copied from `hit_resolution_integration.rs`
//!
//! Cargo compiles every `tests/*.rs` file as its own crate, so nothing in
//! that file is importable here. `lower_ui_to_ir`, `load_window`,
//! `client_extent`, `display_limits`, `window_rect`, `frame_thickness`,
//! `send_dpi_change_to_client`, `normalise_to_reference_baseline`,
//! `send_click`, and `click_and_drain` below are copied verbatim (per that
//! file's own stated convention) from `hit_resolution_integration.rs`; see
//! that file's module header for the full reasoning behind each. This
//! file's own additions are [`read_counter`] (the "read counter N"
//! observation technique described above) and
//! [`connect_host_click_counter`] / [`count_host_signal`] (a host
//! `"clicked"` listener connected through the real ABI, used by the two
//! fixtures that need one).
//!
//! # The client stays small (M4-Phase 1 T8 finding)
//!
//! Every fixture normalises to 96 DPI at an explicitly chosen 360x240
//! physical client — the same ceiling `hit_resolution_integration.rs`'s
//! module header records (a hosted CI desktop failed a larger request
//! twice). No fixture in this file ever changes scale, so 360x240 is also
//! each fixture's only client extent.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::cell::Cell;
use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

use wasamo_runtime::{ffi, DipRect, WidgetNode};

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

// ── Copied verbatim from `hit_resolution_integration.rs` ───────────────────

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<event-routing-integration>";
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
/// synchronously via `SendMessageW`. Suitable for every click in this file
/// that only needs `hit_test_click`'s synchronous dispatch (an inline
/// handler's state write), not a queued host-listener callback.
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
/// through the production message loop instead of by direct `SendMessageW`
/// — see this file's module header ("Why an inline handler is observable
/// synchronously...") for why the two host-listener fixtures need this
/// rather than [`send_click`]. Posts the click, then a synthesised
/// `WM_QUIT`, then pumps `wasamo_runtime::run()` — the same function
/// `wasamo_run` (abi_spec) wraps — until it returns.
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

// ── This file's own helpers ─────────────────────────────────────────────

/// Read a `Text` widget's rendered content and parse it as the `i32`
/// counter it is bound to (`"\{root.<state>}"`). `__text_content_for_test`
/// reads the `content: String` field directly, which an inline handler's
/// `Signal::set` writes synchronously (see this file's module header) — no
/// layout pass is needed for this read to be current. Panics with `what` in
/// the message if the node is not a `Text` or its content does not parse:
/// either would mean the fixture's binding is broken, not that the counter
/// legitimately holds some value.
fn read_counter(node: &WidgetNode, what: &str) -> i32 {
    let content = node
        .__text_content_for_test()
        .unwrap_or_else(|| panic!("{what}: node must be a Text widget bound to the counter"));
    content
        .parse::<i32>()
        .unwrap_or_else(|e| panic!("{what}: Text content {content:?} did not parse as i32 ({e})"))
}

/// Increments the `Cell<u32>` behind `user_data` by one. The host
/// `"clicked"` listener [`connect_host_click_counter`] installs for the two
/// fixtures that need one. Not `#[no_mangle]`: this pointer is only ever
/// handed to `wasamo_signal_connect` in-process, never looked up by symbol
/// name, so no C-visible export is needed.
///
/// # Safety
/// `user_data` must point to a live `Cell<u32>` for the duration of the
/// call — guaranteed here because [`connect_host_click_counter`] never
/// installs a `destroy_fn` that could free it early, and the caller frees
/// it explicitly only after every click that could reach this function has
/// already run.
unsafe extern "C" fn count_host_signal(
    _sender: *mut ffi::WasamoWidget,
    _args: *const ffi::WasamoValue,
    _arg_count: usize,
    user_data: *mut c_void,
) {
    let counter = user_data as *const Cell<u32>;
    (*counter).set((*counter).get() + 1);
}

/// Connect [`count_host_signal`] to `widget`'s `"clicked"` signal through
/// the **real** ABI entry point (`ffi::wasamo_signal_connect`), not the
/// `__add_signal_registration_for_test` seam other integration tests use —
/// this file's fixtures are evidence for what the production host-listener
/// path does when a click reaches it, not for the registry's storage
/// mechanics alone.
///
/// Returns the raw counter pointer and the connection token. `destroy_fn`
/// is `None`, so nothing frees the counter automatically when the
/// registration is severed (by disconnect, or — F4's point — by the
/// widget's own destruction); it stays valid for the caller to read after
/// any click and must be reclaimed explicitly with `Box::from_raw` once the
/// fixture is done with it.
///
/// # Safety
/// `widget` must be non-null and point to a widget live on the calling
/// (owning) thread.
unsafe fn connect_host_click_counter(widget: *mut ffi::WasamoWidget) -> (*mut Cell<u32>, u64) {
    let counter = Box::into_raw(Box::new(Cell::new(0u32)));
    let mut token: u64 = 0;
    let name = "clicked";
    let status = ffi::wasamo_signal_connect(
        widget,
        name.as_ptr() as *const c_char,
        name.len(),
        Some(count_host_signal),
        counter as *mut c_void,
        None,
        &mut token,
    );
    assert_eq!(
        status,
        ffi::WASAMO_OK,
        "wasamo_signal_connect must succeed for the fixture to mean anything"
    );
    (counter, token)
}

/// Local mirror of `hit::contains`'s edge convention — **left/top
/// inclusive, right/bottom exclusive** — copied from
/// `hover_transition_integration.rs` because `hit` is a private module
/// (`mod hit;` in `lib.rs`) and unreachable from this crate. Lets the new
/// M4-Phase 2 T8 fixture below assert exactly the relationship
/// `hit::resolve_topmost` will apply to a probed point, rather than a
/// symmetric approximation of it.
fn dip_contains(rect: DipRect, x: f32, y: f32) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// Whether `outer` fully contains `inner`, edge-inclusive on all four
/// sides (a rectangle contains itself). Copied from
/// `hover_transition_integration.rs`; used below to assert "the disabled
/// Button sits inside the full-bleed Box's rectangle" as a measured fact
/// rather than an assumption.
fn dip_rect_contains(outer: DipRect, inner: DipRect) -> bool {
    outer.x <= inner.x
        && inner.x + inner.width <= outer.x + outer.width
        && outer.y <= inner.y
        && inner.y + inner.height <= outer.y + outer.height
}

// ── F1 — the central claim ──────────────────────────────────────────────

/// A `VStack` holding a `Text` bound to `root.hits` and a `Box` carrying
/// `clicked => { root.hits += 1; }`. `Box.aspect` gives it a non-zero,
/// non-degenerate rectangle (M3-Phase 2 DD-M3-P2-005; without an `aspect`
/// or a child, an empty `Box` with a bounded width and an unbounded height
/// — exactly what a `VStack`'s main axis hands its children — collapses to
/// zero height).
const NON_BUTTON_HANDLER_UI: &str = r#"component NonButtonClickHandler inherits Window {
    state hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.hits}" }
        Box { aspect: 4:1 fill: #336699cc clicked => { root.hits += 1; } }
    }
}"#;

/// **The central claim.** `clicked` fires on a plain `Box`, which is not a
/// Button-family widget and has never had a native `set_clicked` closure
/// slot — see this file's module header for the measured before/after.
///
/// The negative leg (clicking the `Text`) is the agreement check: the
/// `Text` carries no handler and its only ancestor (the `VStack` root) has
/// none either, so nothing should run there regardless of whether the
/// positive leg passes. It is not by itself discriminating pre/post T3 —
/// pre-T3 code never ran a `Text`'s handler either, since a `Text` never
/// had one to run — but it establishes that this fixture's tree does not
/// fire spuriously before the positive leg is asked to prove the opposite
/// case.
#[test]
fn a_click_on_a_widget_without_a_handler_stays_silent_while_a_click_on_a_sibling_with_one_runs_it()
{
    run_on_owning_runtime_thread_or_skip("F1: non-Button widget click dispatch", move || {
        let ir = lower_ui_to_ir(NON_BUTTON_HANDLER_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F1 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (text_rect, box_rect) = {
                let root = (*window).root_widget.as_ref().expect("content root");
                (
                    root.children[0].__arranged_rect_for_test(),
                    root.children[1].__arranged_rect_for_test(),
                )
            };
            let text_rect = text_rect.expect("Text must have been laid out");
            let box_rect = box_rect.expect("Box must have been laid out");
            assert!(
                text_rect.width > 0.0 && text_rect.height > 0.0,
                "fixture stopped discriminating: the Text rectangle {text_rect:?} must be \
                 non-degenerate, or a click at its centre proves nothing"
            );
            assert!(
                box_rect.width > 0.0 && box_rect.height > 0.0,
                "fixture stopped discriminating: the Box rectangle {box_rect:?} must be \
                 non-degenerate, or a click at its centre proves nothing"
            );
            assert!(
                text_rect.y + text_rect.height <= box_rect.y,
                "fixture stopped discriminating: the Text and Box rectangles \
                 ({text_rect:?}, {box_rect:?}) must be disjoint, or a click meant for one \
                 could resolve to the other"
            );

            // Negative / agreement leg: click the Text. Neither it nor its
            // only ancestor (the VStack root) carries a handler.
            let (text_cx, text_cy) = (
                (text_rect.x + text_rect.width / 2.0) * factor,
                (text_rect.y + text_rect.height / 2.0) * factor,
            );
            send_click(hwnd, text_cx, text_cy);
            let after_text_click = {
                let root = (*window).root_widget.as_ref().unwrap();
                read_counter(
                    &root.children[0],
                    "hits after clicking the handler-less Text",
                )
            };

            // Positive leg. Re-read the Box rectangle immediately before
            // clicking it (rule: never click a cached rectangle across a
            // state write that could have re-laid the tree out).
            let box_rect = {
                let root = (*window).root_widget.as_ref().unwrap();
                root.children[1]
                    .__arranged_rect_for_test()
                    .expect("Box must still be laid out")
            };
            let (box_cx, box_cy) = (
                (box_rect.x + box_rect.width / 2.0) * factor,
                (box_rect.y + box_rect.height / 2.0) * factor,
            );
            send_click(hwnd, box_cx, box_cy);
            let after_box_click = {
                let root = (*window).root_widget.as_ref().unwrap();
                read_counter(&root.children[0], "hits after clicking the Box")
            };

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                after_text_click, 0,
                "a click on a widget with no handler, whose only ancestor also has none, \
                 must not run anything"
            );
            assert_eq!(
                after_box_click, 1,
                "a click on a Box carrying an inline `clicked` handler must run it exactly \
                 once, even though Box is not a Button-family widget"
            );
        }
    });
}

// ── F2 — ancestor walk and DD-M4-P2-001's consumption control ──────────────

/// An outer `VStack` holding two counter `Text`s and an inner `VStack`
/// carrying `clicked => { root.ancestor_hits += 1; }`, whose two children
/// are `Box`es: the first with no handler, the second with
/// `clicked => { root.child_hits += 1; }`.
const ANCESTOR_CONSUMPTION_UI: &str = r#"component AncestorConsumption inherits Window {
    state ancestor_hits: i32 = 0
    state child_hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.ancestor_hits}" }
        Text { text: "\{root.child_hits}" }
        VStack {
            spacing: 0
            padding: 0
            clicked => { root.ancestor_hits += 1; }
            Box { aspect: 4:1 fill: #336699cc }
            Box { aspect: 4:1 fill: #996633cc clicked => { root.child_hits += 1; } }
        }
    }
}"#;

/// DD-M4-P2-001's named risk control: "adding a handler to a widget
/// silently removes an ancestor's [dispatch]" is mitigated by an authoring-
/// site-visible spelling, and "the phase's evidence includes a control
/// where a nested handler is added and the ancestor is shown to stop
/// firing." The first `Box` (no handler) shows the ancestor walk works at
/// all; the second `Box` (with a handler) shows that same ancestor stop
/// firing once the nested widget has one of its own.
#[test]
fn an_ancestor_handler_runs_only_until_a_nested_widget_gets_one_of_its_own() {
    run_on_owning_runtime_thread_or_skip("F2: ancestor walk and consumption control", move || {
        let ir = lower_ui_to_ir(ANCESTOR_CONSUMPTION_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F2 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (no_handler_rect, handled_rect) = {
                let root = (*window).root_widget.as_ref().expect("content root");
                let inner = &root.children[2];
                (
                    inner.children[0].__arranged_rect_for_test(),
                    inner.children[1].__arranged_rect_for_test(),
                )
            };
            let no_handler_rect = no_handler_rect.expect("first Box must have been laid out");
            let handled_rect = handled_rect.expect("second Box must have been laid out");
            assert!(
                no_handler_rect.width > 0.0 && no_handler_rect.height > 0.0,
                "fixture stopped discriminating: the handler-less Box rectangle \
                 {no_handler_rect:?} must be non-degenerate"
            );
            assert!(
                handled_rect.width > 0.0 && handled_rect.height > 0.0,
                "fixture stopped discriminating: the handled Box rectangle {handled_rect:?} \
                 must be non-degenerate"
            );
            assert!(
                no_handler_rect.y + no_handler_rect.height <= handled_rect.y,
                "fixture stopped discriminating: the two Box rectangles \
                 ({no_handler_rect:?}, {handled_rect:?}) must be disjoint, or a click meant \
                 for one could resolve to the other"
            );

            // Ancestor leg: click the handler-less Box. It runs nothing, so
            // the walk continues to the inner VStack.
            let (no_handler_cx, no_handler_cy) = (
                (no_handler_rect.x + no_handler_rect.width / 2.0) * factor,
                (no_handler_rect.y + no_handler_rect.height / 2.0) * factor,
            );
            send_click(hwnd, no_handler_cx, no_handler_cy);
            let (ancestor_after_first, child_after_first) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    read_counter(&root.children[0], "ancestor_hits after the ancestor leg"),
                    read_counter(&root.children[1], "child_hits after the ancestor leg"),
                )
            };

            // Consumption leg: re-read the handled Box's rectangle
            // immediately before clicking it.
            let handled_rect = {
                let root = (*window).root_widget.as_ref().unwrap();
                root.children[2].children[1]
                    .__arranged_rect_for_test()
                    .expect("second Box must still be laid out")
            };
            let (handled_cx, handled_cy) = (
                (handled_rect.x + handled_rect.width / 2.0) * factor,
                (handled_rect.y + handled_rect.height / 2.0) * factor,
            );
            send_click(hwnd, handled_cx, handled_cy);
            let (ancestor_after_second, child_after_second) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    read_counter(&root.children[0], "ancestor_hits after the consumption leg"),
                    read_counter(&root.children[1], "child_hits after the consumption leg"),
                )
            };

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                ancestor_after_first, 1,
                "a click on a widget with no handler must walk to its ancestor and run the \
                 ancestor's handler exactly once"
            );
            assert_eq!(
                child_after_first, 0,
                "the second Box's handler must not have run from a click on the first Box"
            );
            assert_eq!(
                child_after_second, 1,
                "a click on the second Box must run its own handler exactly once"
            );
            assert_eq!(
                ancestor_after_second, ancestor_after_first,
                "DD-M4-P2-001's consumption control: once the nested Box has its own handler, \
                 the ancestor's handler must stop firing for a click resolved to that Box — \
                 ancestor_hits must stay at {ancestor_after_first}, not increase further"
            );
        }
    });
}

// ── F3 — disabled Button suppresses without consuming ───────────────────────

/// An inner `VStack` carrying `clicked => { root.ancestor_hits += 1; }`
/// with two Button children: one bound `enabled` to a `state` that starts
/// `false`, one enabled carrying its own `clicked` handler.
///
/// **`enabled` is a `state`-bound identifier here, not the `enabled: false`
/// literal §4.8 shows** — measured directly against this file's first
/// draft, which used the literal and got a Button that stayed enabled.
/// `ir_loader.rs`'s `construct_widget` "Button" arm builds the widget from
/// `label` and `style` only; unlike its sibling "ToggleButton" arm (which
/// reads `extract_bool_prop(&node.props, "enabled")` as the literal
/// fallback beside its binding check), it never reads an `enabled` prop at
/// all, so a *literal* `enabled: false` on a plain `Button` is silently
/// dropped and the widget constructs at its `true` default. A
/// `state`-bound `enabled` reaches `PROP_BUTTON_ENABLED` through
/// `build_node`'s `node.bindings` loop instead (`register_bool_binding` ->
/// `widget_write_property_bool`), which is the path
/// `conditional_toggle_integration.rs`'s existing `enabled: enabled_state`
/// fixture already exercises successfully, and which this fixture's own
/// discriminating guard below confirms actually lands. This is an
/// `ir_loader.rs` gap, not a `hit_test_click` one — out of scope for a
/// task that may not touch `src/` — so this fixture routes around it
/// rather than exercising the broken literal path.
const DISABLED_BUTTON_SUPPRESSION_UI: &str = r#"component DisabledButtonSuppression inherits Window {
    state ancestor_hits: i32 = 0
    state disabled_hits: i32 = 0
    state enabled_hits: i32 = 0
    state disabled_button_enabled: bool = false
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.ancestor_hits}" }
        Text { text: "\{root.disabled_hits}" }
        Text { text: "\{root.enabled_hits}" }
        VStack {
            spacing: 0
            padding: 0
            clicked => { root.ancestor_hits += 1; }
            Button { text: "disabled" enabled: disabled_button_enabled clicked => { root.disabled_hits += 1; } }
            Button { text: "enabled" clicked => { root.enabled_hits += 1; } }
        }
    }
}"#;

/// docs/dsl_spec.md §4.8 / §4.19: a disabled Button "suppresses
/// click-handler dispatch" but, "having run no handler, ... also does not
/// end propagation." The difference leg shows both halves of that sentence
/// on the *same* click: `disabled_hits` stays 0 **and** `ancestor_hits`
/// becomes 1. The agreement leg (an *enabled* Button at the equivalent
/// structural position) shows the ancestor **does** stop being reached once
/// something actually dispatches — without it, an implementation that never
/// reached the ancestor arm at all would be indistinguishable from one
/// where the disabled Button is correctly transparent to propagation.
#[test]
fn a_disabled_button_suppresses_its_own_handler_without_stopping_propagation() {
    run_on_owning_runtime_thread_or_skip("F3: disabled Button suppression", move || {
        let ir = lower_ui_to_ir(DISABLED_BUTTON_SUPPRESSION_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F3 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (disabled_rect, enabled_rect) = {
                let root = (*window).root_widget.as_ref().expect("content root");
                let inner = &root.children[3];
                (
                    inner.children[0].__arranged_rect_for_test(),
                    inner.children[1].__arranged_rect_for_test(),
                )
            };
            let disabled_rect = disabled_rect.expect("disabled Button must have been laid out");
            let enabled_rect = enabled_rect.expect("enabled Button must have been laid out");
            assert!(
                disabled_rect.width > 0.0 && disabled_rect.height > 0.0,
                "fixture stopped discriminating: the disabled Button rectangle \
                 {disabled_rect:?} must be non-degenerate"
            );
            assert!(
                enabled_rect.width > 0.0 && enabled_rect.height > 0.0,
                "fixture stopped discriminating: the enabled Button rectangle {enabled_rect:?} \
                 must be non-degenerate"
            );
            assert!(
                disabled_rect.y + disabled_rect.height <= enabled_rect.y,
                "fixture stopped discriminating: the two Button rectangles \
                 ({disabled_rect:?}, {enabled_rect:?}) must be disjoint, or a click meant for \
                 one could resolve to the other"
            );
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[3].children[0].__button_enabled_for_test(),
                    Some(false),
                    "fixture stopped discriminating: the first Button must actually be \
                     disabled, or the difference leg below proves nothing"
                );
            }

            // Difference leg: click the disabled Button. Dispatch is
            // suppressed at the Button, but the walk still reaches its
            // ancestor.
            let (disabled_cx, disabled_cy) = (
                (disabled_rect.x + disabled_rect.width / 2.0) * factor,
                (disabled_rect.y + disabled_rect.height / 2.0) * factor,
            );
            send_click(hwnd, disabled_cx, disabled_cy);
            let (ancestor_after_disabled, disabled_after_disabled) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    read_counter(&root.children[0], "ancestor_hits after the difference leg"),
                    read_counter(&root.children[1], "disabled_hits after the difference leg"),
                )
            };

            // Agreement leg: re-read the enabled Button's rectangle
            // immediately before clicking it.
            let enabled_rect = {
                let root = (*window).root_widget.as_ref().unwrap();
                root.children[3].children[1]
                    .__arranged_rect_for_test()
                    .expect("enabled Button must still be laid out")
            };
            let (enabled_cx, enabled_cy) = (
                (enabled_rect.x + enabled_rect.width / 2.0) * factor,
                (enabled_rect.y + enabled_rect.height / 2.0) * factor,
            );
            send_click(hwnd, enabled_cx, enabled_cy);
            let (ancestor_after_enabled, enabled_after_enabled) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    read_counter(&root.children[0], "ancestor_hits after the agreement leg"),
                    read_counter(&root.children[2], "enabled_hits after the agreement leg"),
                )
            };

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                disabled_after_disabled, 0,
                "a disabled Button must suppress its own `clicked` dispatch \
                 (docs/dsl_spec.md §4.8)"
            );
            assert_eq!(
                ancestor_after_disabled, 1,
                "a disabled Button must not stop propagation: the ancestor's handler must run \
                 exactly once from a click on the disabled Button (docs/dsl_spec.md §4.19)"
            );
            assert_eq!(
                enabled_after_enabled, 1,
                "an enabled Button at the equivalent structural position must run its own \
                 handler exactly once"
            );
            assert_eq!(
                ancestor_after_enabled, ancestor_after_disabled,
                "an enabled Button's handler running must consume the event: ancestor_hits \
                 must stay at {ancestor_after_disabled}, not increase further, once the \
                 enabled Button dispatches"
            );
        }
    });
}

// ── F4 — a handler that removes its own subtree ─────────────────────────────

/// An inner `VStack` carrying `clicked => { root.ancestor_hits += 1; }`
/// containing `if present { Button { ... } }`, whose handler both counts
/// its own run and flips `present` to `false` — removing its own subtree
/// from inside its own dispatch.
const SELF_REMOVING_HANDLER_UI: &str = r#"component SelfRemovingHandler inherits Window {
    state present: bool = true
    state removed_hits: i32 = 0
    state ancestor_hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.removed_hits}" }
        Text { text: "\{root.ancestor_hits}" }
        VStack {
            spacing: 0
            padding: 0
            clicked => { root.ancestor_hits += 1; }
            if present {
                Button { text: "self-destruct" clicked => { root.removed_hits += 1; root.present = false; } }
            }
        }
    }
}"#;

/// Clicking the Button runs its handler, whose second statement
/// (`root.present = false`) drives the conditional's structural rebuild
/// **synchronously, while the node is still being dispatched** — the node
/// the walk is standing on is destroyed inside its own handler.
///
/// A host listener is connected to the Button *before* the click, through
/// the real ABI. `docs/dsl_spec.md §4.19`: "a widget removed by a
/// handler's own state write does not receive the event afterwards" — this
/// fixture shows the mechanism, not just the outcome: the Button's
/// registrations are severed by `widget_destroy` (called from
/// `mutate_conditional_subtree`'s removal arm) *before*
/// `run_clicked_handlers` reaches its own `enqueue_signal` step, so the
/// enqueue finds no tokens and the drained callback never fires. The
/// process not crashing despite dispatching through a node it destroys
/// mid-dispatch is asserted indirectly by every assertion after the click
/// succeeding at all, and directly by the `Healthy` runtime-health check.
#[test]
fn a_handler_that_removes_its_own_widget_consumes_the_click_and_leaves_no_registration_to_fire() {
    run_on_owning_runtime_thread_or_skip("F4: handler removes its own subtree", move || {
        let ir = lower_ui_to_ir(SELF_REMOVING_HANDLER_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F4 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (button_ptr, button_rect, children_before) = {
                let root = (*window).root_widget.as_mut().expect("content root");
                let inner = root.children[2].as_mut();
                assert_eq!(
                    inner.child_count(),
                    1,
                    "fixture stopped discriminating: `present` starts true, so the inner \
                     VStack must have exactly one child (the Button) before the click"
                );
                let button: &mut WidgetNode = inner.children[0].as_mut();
                let ptr = button as *mut WidgetNode;
                let rect = button.__arranged_rect_for_test();
                (ptr, rect, inner.child_count())
            };
            let button_rect = button_rect.expect("Button must have been laid out");
            assert!(
                button_rect.width > 0.0 && button_rect.height > 0.0,
                "fixture stopped discriminating: the Button rectangle {button_rect:?} must be \
                 non-degenerate, or a click at its centre proves nothing"
            );

            let (host_counter, _token) =
                connect_host_click_counter(button_ptr as *mut ffi::WasamoWidget);

            let (cx, cy) = (
                (button_rect.x + button_rect.width / 2.0) * factor,
                (button_rect.y + button_rect.height / 2.0) * factor,
            );
            // The host listener only fires from the drain's Phase 3 (see
            // this file's module header), so this fixture must pump the
            // real message loop once even though the structural removal
            // itself already happens synchronously inside `hit_test_click`.
            click_and_drain(hwnd, cx, cy);

            let (removed_hits, ancestor_hits, children_after) = {
                let root = (*window).root_widget.as_ref().unwrap();
                let inner = &root.children[2];
                (
                    read_counter(&root.children[0], "removed_hits after the click"),
                    read_counter(&root.children[1], "ancestor_hits after the click"),
                    inner.child_count(),
                )
            };
            let health = ffi::__runtime_health_for_test();
            let host_fired = (*host_counter).get();

            ffi::wasamo_window_destroy(window);
            drop(Box::from_raw(host_counter));

            assert_eq!(
                removed_hits, 1,
                "the Button's own handler must have run exactly once before it removed itself"
            );
            assert_eq!(
                ancestor_hits, 0,
                "the walk must have consumed at the removed Button, so its ancestor's handler \
                 must never run"
            );
            assert!(
                children_after < children_before,
                "the Button's subtree must actually have been removed (child count \
                 {children_before} -> {children_after}), or this fixture is not testing \
                 self-removal at all"
            );
            assert_eq!(
                health, "Healthy",
                "dispatching through a node that destroys itself mid-dispatch must not leave \
                 the runtime Diverged"
            );
            assert_eq!(
                host_fired, 0,
                "a host listener connected to a widget that removes itself from inside its own \
                 handler must never fire: its registration is severed with the subtree before \
                 the enqueue step runs (docs/dsl_spec.md §4.19)"
            );
        }
    });
}

// ── F5 — a host-registered listener on a non-Button widget consumes the walk ─

/// An inner `VStack` carrying `clicked => { root.ancestor_hits += 1; }`
/// whose single child is a `Box` with no inline handler at all — only a
/// host listener, connected before the click.
const HOST_LISTENER_CONSUMPTION_UI: &str = r#"component HostListenerConsumption inherits Window {
    state ancestor_hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.ancestor_hits}" }
        VStack {
            spacing: 0
            padding: 0
            clicked => { root.ancestor_hits += 1; }
            Box { aspect: 4:1 fill: #336699cc }
        }
    }
}"#;

/// A queued host listener counts as a handler that ran, so it consumes the
/// walk exactly as an inline handler would — host signal callbacks fire in
/// the drain's Phase 3, not synchronously, so this fixture uses
/// [`click_and_drain`] for both legs. The agreement leg (after
/// `wasamo_signal_disconnect`) shows the same click at the same rectangle
/// reaches the ancestor once the listener is gone; without it, "the
/// ancestor did not fire" would be equally produced by a click that simply
/// resolved to nothing.
#[test]
fn a_host_signal_listener_on_a_non_button_widget_consumes_the_walk_until_disconnected() {
    run_on_owning_runtime_thread_or_skip("F5: host listener consumes the walk", move || {
        let ir = lower_ui_to_ir(HOST_LISTENER_CONSUMPTION_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F5 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (box_ptr, box_rect) = {
                let root = (*window).root_widget.as_mut().expect("content root");
                let inner = root.children[1].as_mut();
                let box_node: &mut WidgetNode = inner.children[0].as_mut();
                let ptr = box_node as *mut WidgetNode;
                let rect = box_node.__arranged_rect_for_test();
                (ptr, rect)
            };
            let box_rect = box_rect.expect("Box must have been laid out");
            assert!(
                box_rect.width > 0.0 && box_rect.height > 0.0,
                "fixture stopped discriminating: the Box rectangle {box_rect:?} must be \
                 non-degenerate, or a click at its centre proves nothing"
            );

            let (host_counter, token) =
                connect_host_click_counter(box_ptr as *mut ffi::WasamoWidget);

            // Consumption leg: the host listener is connected, so the click
            // must consume at the Box.
            let (cx, cy) = (
                (box_rect.x + box_rect.width / 2.0) * factor,
                (box_rect.y + box_rect.height / 2.0) * factor,
            );
            click_and_drain(hwnd, cx, cy);
            let (host_fired_connected, ancestor_after_connected) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    (*host_counter).get(),
                    read_counter(
                        &root.children[0],
                        "ancestor_hits with the listener connected",
                    ),
                )
            };

            // Agreement leg: disconnect, then click again. Re-read the
            // Box's rectangle immediately before this click too, even
            // though nothing in this fixture should have moved it.
            let disconnect_status = ffi::wasamo_signal_disconnect(token);
            let box_rect_again = {
                let root = (*window).root_widget.as_ref().unwrap();
                root.children[1].children[0]
                    .__arranged_rect_for_test()
                    .expect("Box must still be laid out")
            };
            let (cx2, cy2) = (
                (box_rect_again.x + box_rect_again.width / 2.0) * factor,
                (box_rect_again.y + box_rect_again.height / 2.0) * factor,
            );
            click_and_drain(hwnd, cx2, cy2);
            let (host_fired_disconnected, ancestor_after_disconnected) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    (*host_counter).get(),
                    read_counter(
                        &root.children[0],
                        "ancestor_hits after disconnecting the listener",
                    ),
                )
            };

            ffi::wasamo_window_destroy(window);
            drop(Box::from_raw(host_counter));

            assert_eq!(
                disconnect_status,
                ffi::WASAMO_OK,
                "wasamo_signal_disconnect must succeed for the agreement leg to mean anything"
            );
            assert_eq!(
                host_fired_connected, 1,
                "a connected host listener on the Box must fire exactly once from a click on it"
            );
            assert_eq!(
                ancestor_after_connected, 0,
                "a queued host listener counts as a handler that ran, so it must consume the \
                 walk: the ancestor's handler must not run while the listener is connected"
            );
            assert_eq!(
                host_fired_disconnected, host_fired_connected,
                "a disconnected host listener must not fire again"
            );
            assert_eq!(
                ancestor_after_disconnected,
                ancestor_after_connected + 1,
                "the same click at the same coordinates must reach the ancestor once the \
                 listener is disconnected, or 'the ancestor did not fire' in the first leg \
                 could equally have been produced by a click that resolved to nothing"
            );
        }
    });
}

// ── F6 — a disabled Button occludes a lower sibling (M4-Phase 2 T8) ────────

/// A full-bleed `Box { clicked => ... }` sits **underneath** a disabled
/// `Button`. `slot.h-align: stretch` / `slot.v-align: stretch` are required
/// on the Box, not incidental: a `Box`'s default size constraint is
/// `Shrink`/`Shrink` (`widget.rs`'s `WidgetNode::box_` doc comment), and
/// `ZStack`'s own default per-child alignment is `center`
/// (`docs/dsl_spec.md` §4.13) — a Shrink, center-aligned Box with no
/// `aspect` and no child has no way to resolve its own size at all
/// (`layout::arrange_zstack` measures a non-stretchy child against
/// unbounded bounds to learn its natural size, which a childless
/// non-aspect Box cannot supply — `LayoutError::BoxNoExtent`,
/// `docs/architecture.md`'s DD-M3-P2-005). `stretch` on both axes is the
/// same technique `hover_transition_integration.rs`'s F5 fixture uses for
/// its own scrim-shaped Box, here on both axes instead of one so the Box
/// becomes genuinely full-bleed rather than partial. The disabled Button,
/// by contrast, keeps ZStack's plain default (no `slot.*`), landing at its
/// own intrinsic size, centred inside the Box's rectangle.
///
/// `enabled: false` is authored as a literal, not a `state`-bound
/// identifier: the literal-drop defect the older
/// `DISABLED_BUTTON_SUPPRESSION_UI` fixture above had to route around
/// (CF-2) is already fixed on this branch (see this crate's
/// `ir_loader.rs` "CF-2 disposition" comment on the `"Button"` arm of
/// `construct_widget`), so the literal now lands.
const DISABLED_BUTTON_OCCLUDES_A_LOWER_SIBLING_UI: &str = r#"component DisabledButtonOccludesALowerSibling inherits Window {
    state hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.hits}" }
        ZStack {
            Box {
                slot.h-align: stretch
                slot.v-align: stretch
                fill: #336699cc
                clicked => { root.hits += 1; }
            }
            Button { text: "gate" enabled: false }
        }
    }
}"#;

/// **The plan's other named `enabled` evidence**: "a disabled Button still
/// occludes what is behind it" (`docs/dsl_spec.md` §4.19), now exercised
/// against a *lower sibling* rather than an ancestor.
/// [`a_disabled_button_suppresses_its_own_handler_without_stopping_propagation`]
/// above already shows the click continuing *upward* past a disabled
/// Button; that says nothing about what happens *underneath* one, which
/// only became a meaningful question once a plain `Box` could carry
/// `clicked` and the runtime actually dispatches it (T3). Two legs, both
/// required:
///
/// - **Occlusion leg:** a click inside the disabled Button's rectangle
///   must leave the Box's counter at `0`. `hit::resolve_topmost` resolves
///   the Button (topmost-painted, DD-M4-P2-002), which is on the dispatch
///   chain but suppressed (§4.8) — its ancestor is the `ZStack`, not the
///   Box (a *sibling*, never visited), so the Box's own `clicked` handler
///   is never reached at all, regardless of the Button's `enabled` state.
/// - **Agreement leg:** a click on the same Box, outside the Button's
///   rectangle, must increment the counter. Without this leg, "the counter
///   stayed zero" in the first leg would be equally produced by a Box that
///   never receives clicks at all (e.g. a hit-test or dispatch-chain
///   regression that always resolves to the Button's ancestor), not by
///   genuine occlusion.
#[test]
fn a_disabled_button_occludes_a_lower_sibling_it_paints_over() {
    run_on_owning_runtime_thread_or_skip(
        "F6: disabled Button occludes a lower sibling",
        move || {
            let ir = lower_ui_to_ir(DISABLED_BUTTON_OCCLUDES_A_LOWER_SIBLING_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F6 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                let (box_rect, button_rect) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    let zstack = &root.children[1];
                    assert_eq!(
                        zstack.__kind_name_for_test(),
                        "ZStack",
                        "fixture stopped discriminating: the second child must actually be the \
                     ZStack"
                    );
                    assert_eq!(
                        zstack.children[1].__button_enabled_for_test(),
                        Some(false),
                        "fixture stopped discriminating: the Button must actually be disabled, or \
                     the occlusion leg below proves nothing about `enabled`"
                    );
                    (
                        zstack.children[0].__arranged_rect_for_test(),
                        zstack.children[1].__arranged_rect_for_test(),
                    )
                };
                let box_rect = box_rect.expect("Box must have been laid out");
                let button_rect = button_rect.expect("Button must have been laid out");
                assert!(
                    box_rect.width > 0.0 && box_rect.height > 0.0,
                    "fixture stopped discriminating: the Box rectangle {box_rect:?} must be \
                 non-degenerate"
                );
                assert!(
                    button_rect.width > 0.0 && button_rect.height > 0.0,
                    "fixture stopped discriminating: the Button rectangle {button_rect:?} must be \
                 non-degenerate"
                );
                assert!(
                box_rect.width - button_rect.width > 20.0,
                "fixture stopped discriminating: the full-bleed Box must be meaningfully wider \
                 than the Button (box {box_rect:?}, button {button_rect:?}), or the agreement- \
                 leg point below has no room to land inside the Box alone"
            );
                assert!(
                    dip_rect_contains(box_rect, button_rect),
                    "geometry this fixture depends on (docs/dsl_spec.md §4.13's default centre \
                 alignment over a Fill/Fill Box): the Button rectangle {button_rect:?} must sit \
                 entirely inside the Box's {box_rect:?}"
                );

                // Occlusion leg: the Button's own centre, which the full
                // containment above guarantees is also inside the Box.
                let (gate_x, gate_y) = (
                    button_rect.x + button_rect.width / 2.0,
                    button_rect.y + button_rect.height / 2.0,
                );
                assert!(
                    dip_contains(button_rect, gate_x, gate_y)
                        && dip_contains(box_rect, gate_x, gate_y),
                    "fixture stopped discriminating: ({gate_x}, {gate_y}) must be inside both \
                 rectangles"
                );
                send_click(hwnd, gate_x * factor, gate_y * factor);
                let hits_after_gate = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    read_counter(&root.children[0], "hits after clicking the disabled Button")
                };

                // Agreement leg: midway between the Box's left edge and the
                // Button's left edge — inside the Box, outside the Button — the
                // same technique `hover_transition_integration.rs`'s overlap
                // fixture uses. Re-read both rectangles immediately before
                // this click (rule: never click a cached rectangle across a
                // state write that could have re-laid the tree out), even
                // though nothing in this fixture should have moved either.
                let (box_rect, button_rect) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let zstack = &root.children[1];
                    (
                        zstack.children[0]
                            .__arranged_rect_for_test()
                            .expect("Box must still be laid out"),
                        zstack.children[1]
                            .__arranged_rect_for_test()
                            .expect("Button must still be laid out"),
                    )
                };
                let (box_only_x, box_only_y) = (
                    (box_rect.x + button_rect.x) / 2.0,
                    box_rect.y + box_rect.height / 2.0,
                );
                assert!(
                    dip_contains(box_rect, box_only_x, box_only_y)
                        && !dip_contains(button_rect, box_only_x, box_only_y),
                    "fixture stopped discriminating: ({box_only_x}, {box_only_y}) must be inside \
                 the Box {box_rect:?} and outside the Button {button_rect:?}, or the agreement \
                 leg below is not testing what it claims to"
                );
                send_click(hwnd, box_only_x * factor, box_only_y * factor);
                let hits_after_box = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    read_counter(
                        &root.children[0],
                        "hits after clicking the Box outside the Button",
                    )
                };

                ffi::wasamo_window_destroy(window);

                assert_eq!(
                    hits_after_gate, 0,
                    "a click inside the disabled Button's rectangle must leave the Box's counter \
                 at 0: the disabled Button occludes the Box beneath it, and the Box — a \
                 sibling, not an ancestor of the Button — is never reached by the walk at all"
                );
                assert_eq!(
                    hits_after_box,
                    hits_after_gate + 1,
                    "a click on the same Box outside the Button's rectangle must increment the \
                 counter, or 'the counter stayed zero' above would be equally explained by a \
                 Box that never receives clicks at all"
                );
            }
        },
    );
}
