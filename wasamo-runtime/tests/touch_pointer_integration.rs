//! Mock-free Windows integration evidence for M4-Phase 2 T11 stage 1
//! ([DD-M4-P2-001](../../process/milestone-4/phase-2/decisions/dd-m4-p2-001-event-routing-model.md)
//! §Touch): a real `WM_POINTER*` sequence, delivered synchronously through
//! the real window procedure (`window.rs`'s claimed-but-inert arm and its
//! `WM_POINTERUP` dispatch arm), against a real widget tree built from real
//! `.ui` source through `wasamoc` -> IR -> `wasamo_load_ui`, with live
//! widget / hover / focus state read back afterwards. Nothing is mocked.
//!
//! # This is the message-level half of T11's evidence, not the injection half
//!
//! Every message below is **posted / sent** (`SendMessageW`) directly to
//! the window procedure — not delivered by a real OS pointer/touch
//! injection API (`InjectTouchInput` / `InjectSyntheticPointerInput`).
//! That is deliberate, and it is a **weaker claim than real injection**:
//! this file shows that `wnd_proc` converts, resolves and dispatches a
//! `WM_POINTER*` message correctly once it arrives, not that a real
//! digitizer contact (or the OS's own injection APIs) actually produces
//! that message and actually gets suppressed from mouse promotion end to
//! end. It needs only the environment this crate's existing Compositor
//! integration suite already runs in (a live Compositor, no interactive
//! desktop requirement) — see
//! [log.md §T11 "Understated 2"](../../process/milestone-4/phase-2/implementation/log.md#t11--touch)
//! for why the two claims are split and where the injection half lives
//! (a separate desktop-tier artifact, stage 2). Nothing here should be read
//! as evidence that OS touch injection reaches this runtime; it is evidence
//! about `wnd_proc` alone.
//!
//! # What each fixture is for
//!
//! - [`a_touch_tap_fires_the_handler_of_the_widget_it_lands_on`] — the
//!   central positive claim: a `WM_POINTERUP` at a widget's centre runs
//!   that widget's `clicked` handler and no other widget's.
//! - [`a_pointer_message_carrying_an_untranslated_client_point_resolves_to_nothing`]
//!   — the leg that makes the fixture above discriminating rather than
//!   incidental. `WM_POINTER*`'s `lParam` is screen-space (measured,
//!   M4-Phase 2 T11 fact 3); this fixture sends the **client** point
//!   directly, unconverted — exactly what a caller upstream of a missing
//!   `ScreenToClient` would produce — at a window deliberately parked away
//!   from the desktop origin, so a correct implementation's
//!   `ScreenToClient` mistranslates it into a coordinate far outside every
//!   widget's rectangle and nothing fires.
//! - [`one_touch_contact_reaches_the_handler_exactly_once`] — fact 2's
//!   delivery order (`ENTER`, `DOWN`, `UP`, `LEAVE`, no `UPDATE` for a
//!   stationary contact) drives the handler exactly once.
//! - [`two_deliveries_of_the_same_contact_are_reachable`] — the agreement
//!   leg for the fixture above: without it, "exactly once" would be
//!   equally satisfied by a counter that cannot count past one. A second,
//!   independent delivery at the same point (a `WM_LBUTTONUP` — what an
//!   *unsuppressed* promotion would additionally produce, fact 4's
//!   unclaimed leg) is shown to move the counter again, so "exactly once"
//!   above is a real count, not a ceiling.
//! - [`the_claimed_pointer_messages_that_act_on_nothing_change_no_state`] —
//!   fires `ENTER` / `DOWN` / `UPDATE` / `LEAVE` directly (no `UP`) and
//!   shows each is a true no-op: no handler counter moves, the retained
//!   hover record is untouched, the retained focus record is untouched.
//!   This **executes** the claimed-but-inert arm — the branch nothing else
//!   in this file fires (implementation-gates trap #4) — but it does not,
//!   and cannot, observe that the arm's `if
//!   claims_pointer_message_without_acting(msg) { return LRESULT(0); }`
//!   check is what produced the no-op rather than, say, `DefWindowProcW`
//!   doing the same by coincidence: see
//!   `claims_pointer_message_without_acting_is_pinned_by_name`'s doc
//!   comment (`window.rs`) for what actually pins the *use* of the
//!   predicate, as opposed to its membership.
//! - [`a_touch_tap_moves_focus_the_way_a_click_does`] — M4-Phase 2 T11's
//!   first explicit decision (closing the T5/T7/T10 focus line): a
//!   `WM_POINTERUP` on a Button moves focus, and it moves it to the same
//!   path a `WM_LBUTTONUP` at the same client point would. Two independent
//!   windows, so "focus was `None` beforehand" is established rather than
//!   inherited from an earlier step in the same fixture.
//! - [`a_touch_tap_leaves_the_hover_record_untouched`] — M4-Phase 2 T11's
//!   second explicit decision: a touch contact writes no hover state. A
//!   real `WM_MOUSEMOVE` sets the retained hover record first (so there is
//!   something to disturb), then a tap on a *different* widget through the
//!   pointer family is shown to leave it exactly where it was; a tap with
//!   nothing hovered beforehand is shown to leave it `None`.
//! - [`a_touch_tap_at_a_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_it`]
//!   — the same central claim as the first fixture, but targeted at
//!   **Button B** rather than Box A and normalised to 120 DPI (scale 1.25)
//!   instead of 96. A missing or doubled division by scale is invisible at
//!   100% and invisible against Box A's full-width, ~72-DIP-tall rectangle
//!   even at 1.25x (both mutations land inside it); Button B is small
//!   enough that both move the resolved point outside its rectangle, which
//!   a precondition assertion and a negative leg (the same undivided point,
//!   sent so it resolves to nothing) prove rather than assume.
//! - [`the_mouse_and_pointer_families_agree_on_the_same_point`] — the same
//!   client point, delivered once as `WM_LBUTTONUP` and once as
//!   `WM_POINTERUP` (in two independent windows), reaches the same
//!   handler.
//! - [`a_non_primary_contact_is_claimed_but_dispatches_nothing`] — the
//!   branch test for the primary-contact dispatch gate
//!   (`window::pointer_message_is_primary`, M4-Phase 2 T11 independent-
//!   review correction C4): a `WM_POINTERUP` with the primary flag clear
//!   runs no handler, moves no focus, and touches no hover record, even
//!   though the message is still claimed (returns `LRESULT(0)`).
//!
//! # Helpers copied from `event_routing_integration.rs`
//!
//! Cargo compiles every `tests/*.rs` file as its own crate, so nothing in
//! that file is importable here. `lower_ui_to_ir`, `load_window`,
//! `client_extent`, `display_limits`, `window_rect`, `frame_thickness`,
//! `send_dpi_change_to_client`, `normalise_to_reference_baseline`,
//! `REFERENCE_DPI`, `CLIENT_W`, `CLIENT_H`, and `read_counter` below are
//! copied with only the diagnostic path string changed (per that file's
//! own stated copying convention, itself continuing the chain from
//! `hit_resolution_integration.rs`) from `event_routing_integration.rs`;
//! see that file's module header for the full reasoning behind each. The
//! one line that differs is `lower_ui_to_ir`'s `path` constant, which
//! reads `"<touch-pointer-integration>"` here instead of
//! `"<event-routing-integration>"` — worth calling out precisely because
//! this file makes "copied" a load-bearing convention, and a silent
//! divergence would defeat the point of stating it. This file's own
//! additions are: the window positioning helper
//! (`move_window_to_known_position`), the DPI-parameterised baseline
//! (`normalise_to_scale_baseline`, generalising the copied
//! reference-DPI-only helper for the non-unit-scale fixture), the
//! `ClientToScreen`-based point mapper (`client_to_screen_physical`), the
//! seven pointer-family senders and two mouse-family senders, and the
//! shared two-widget fixture tree (`TOUCH_DISPATCH_UI`) with its setup
//! helper.
//!
//! # Why `SendMessageW` is enough here
//!
//! Every handler this file's fixture tree carries is an inline `clicked`
//! handler that writes a `state` bound directly into a `Text`'s content.
//! As `event_routing_integration.rs`'s module header explains in full, an
//! inline handler's `Signal::set` drains its reactive Effects
//! **synchronously**, at zero batch depth, so a plain `SendMessageW` (no
//! `PostMessageW` + `run()` pump) is enough for the Effect that writes the
//! `Text.content` to have already run by the time `SendMessageW` returns —
//! `read_counter`'s `__text_content_for_test()` read is then current. No
//! fixture in this file connects a host signal listener (the case that
//! *would* need the queued drain), so no fixture needs the pump.
//!
//! # The window is moved off the desktop origin, on purpose
//!
//! At the desktop origin, a window's screen and client coordinate spaces
//! coincide (screen `(x,y)` == client `(x,y)`), so a `ScreenToClient` call
//! that was deleted entirely would still produce the right answer there.
//! [`move_window_to_known_position`] moves every window in this file to
//! `(`[`WINDOW_X`]`, `[`WINDOW_Y`]`)` — a known, non-origin position,
//! asserted after the move — **before** any baseline normalisation, so the
//! screen-to-client conversion this file exists to pin actually has
//! something to fail against. Every pointer-message point is then derived
//! from the intended **client** point via a real `ClientToScreen` call
//! ([`client_to_screen_physical`]), never hand-computed from the window's
//! position, so a fixture cannot silently encode the wrong offset itself.
//!
//! # The client stays small (M4-Phase 1 T8 finding)
//!
//! Every fixture normalises to a 360x240 physical client — the same
//! ceiling `event_routing_integration.rs`'s and
//! `hit_resolution_integration.rs`'s module headers record (a hosted CI
//! desktop failed a larger request twice).

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::ffi::c_void;
use std::ptr;

use wasamo_runtime::{ffi, DipRect, WidgetNode};

use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, SendMessageW, SetWindowPos,
    POINTER_MESSAGE_FLAG_PRIMARY, SM_CXMAXTRACK, SM_CXSCREEN, SM_CYMAXTRACK, SM_CYSCREEN,
    SWP_NOSIZE, SWP_NOZORDER, WM_DPICHANGED, WM_LBUTTONUP, WM_MOUSEMOVE, WM_POINTERDOWN,
    WM_POINTERENTER, WM_POINTERLEAVE, WM_POINTERUP, WM_POINTERUPDATE,
};

/// The reference DPI most fixtures normalise to before measuring. See
/// `hit_resolution_integration.rs`'s module header (Phase 1 T9 finding
/// F-47) for why this is reached deliberately rather than assumed.
const REFERENCE_DPI: u32 = 96;

/// The non-unit-scale DPI the last fixture normalises to instead
/// (`DipScale::from_dpi(120).factor() == 1.25`).
const SCALED_DPI: u32 = 120;

/// The physical client every fixture normalises to (== DIP at
/// [`REFERENCE_DPI`]; DIP-times-1.25 at [`SCALED_DPI`]). See this file's
/// module header, "The client stays small".
const CLIENT_W: i32 = 360;
const CLIENT_H: i32 = 240;

/// A known, non-origin screen position every window in this file is moved
/// to before baseline normalisation. See this file's module header, "The
/// window is moved off the desktop origin, on purpose".
const WINDOW_X: i32 = 300;
const WINDOW_Y: i32 = 300;

/// An arbitrary, fixed `WM_POINTER*` pointer id, packed into the low word
/// of every sender's `wParam` beside the primary flag (see
/// [`pack_pointer_wparam`]). No arm in `window.rs` reads the id itself:
/// the claimed-but-inert arm never inspects `wParam` at all, and the
/// `WM_POINTERUP` arm only reads the **high word**
/// (`window::pointer_message_is_primary`), never the low-word id — so this
/// value carries no meaning here beyond being present, matching a real
/// `WM_POINTER*` message's shape.
const POINTER_ID: u32 = 1;

// ── Copied verbatim from `event_routing_integration.rs` ────────────────────

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<touch-pointer-integration>";
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

/// Put a freshly created window into the before-state most fixtures below
/// assume: cached scale [`REFERENCE_DPI`], physical (and therefore DIP)
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

// ── This file's own helpers ─────────────────────────────────────────────

/// Move `hwnd` to a known, non-origin screen position and assert the move
/// landed. Must run **before** [`normalise_to_reference_baseline`] /
/// [`normalise_to_scale_baseline`] — see this file's module header, "The
/// window is moved off the desktop origin, on purpose": at the desktop
/// origin, screen and client coordinates coincide, and this file's central
/// claim (a `ScreenToClient` translation the pointer family needs and the
/// mouse family does not) has nothing to fail against there.
unsafe fn move_window_to_known_position(hwnd: HWND, x: i32, y: i32, what: &str) {
    SetWindowPos(hwnd, None, x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER).expect("SetWindowPos");
    let outer = window_rect(hwnd);
    assert_eq!(
        (outer.left, outer.top),
        (x, y),
        "{what}: the window must actually have moved to ({x},{y}) before baseline \
         normalisation, or every screen-to-client assertion below is about an offset that \
         happens to be zero"
    );
}

/// Generalises the copied [`normalise_to_reference_baseline`] to an
/// arbitrary target `dpi`, for
/// [`a_touch_tap_at_a_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_it`],
/// the one fixture in this file that normalises away from
/// [`REFERENCE_DPI`].
unsafe fn normalise_to_scale_baseline(
    window: *mut ffi::WasamoWindow,
    dpi: u32,
    client_w: i32,
    client_h: i32,
    what: &str,
) {
    let hwnd = (*window).hwnd;
    send_dpi_change_to_client(hwnd, dpi, client_w, client_h);
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
        dpi,
        "{what}: the baseline scale must be {dpi}, or the factors below are taken against \
         the wrong scale"
    );
}

/// Map `hwnd`'s **client**-area physical point to its current **screen**
/// point via the real `ClientToScreen` — every pointer-message point in
/// this file is derived this way rather than hand-computed from the
/// window's position (this file's module header, "The window is moved off
/// the desktop origin, on purpose").
unsafe fn client_to_screen_physical(hwnd: HWND, x: f32, y: f32) -> (f32, f32) {
    let mut point = POINT {
        x: x.round() as i32,
        y: y.round() as i32,
    };
    let ok = ClientToScreen(hwnd, &mut point);
    assert!(
        ok.as_bool(),
        "ClientToScreen must succeed for a fixture point to mean anything"
    );
    (point.x as f32, point.y as f32)
}

/// Pack a **physical** pointer position into the `lParam` shape every
/// message below uses — the same packing `event_routing_integration.rs`'s
/// `send_click` and `hover_transition_integration.rs`'s `pack_pointer` use
/// (itself the inverse of `window::pointer_physical`).
fn pack_pointer(x: f32, y: f32) -> LPARAM {
    let packed = ((y.round() as i32 as u32) << 16) | (x.round() as i32 as u32 & 0xFFFF);
    LPARAM(packed as i32 as isize)
}

/// Pack a `WM_POINTER*` message's `wParam`: the pointer id in the low
/// word, the `POINTER_MESSAGE_FLAG_PRIMARY` bit (`0x2000`) in the high
/// word when `primary` is set — the same shape
/// `window::pointer_message_is_primary` reads (M4-Phase 2 T11
/// independent-review correction C4). Every sender below except
/// [`send_pointer_up_non_primary`] marks its (single) contact primary,
/// matching what a real single-contact tap was measured to carry (see
/// `pointer_message_is_primary`'s doc comment for the measurement).
fn pack_pointer_wparam(pointer_id: u32, primary: bool) -> WPARAM {
    let flags: u32 = if primary {
        POINTER_MESSAGE_FLAG_PRIMARY
    } else {
        0
    };
    WPARAM(((flags << 16) | (pointer_id & 0xFFFF)) as usize)
}

/// A real `WM_POINTERENTER` at a **physical** client position, translated
/// to screen space via [`client_to_screen_physical`] before it is packed —
/// `WM_POINTER*`'s `lParam` is screen-space (fact 3), unlike the mouse
/// family's client-space `lParam`. Delivered synchronously via
/// `SendMessageW`, reaching `window.rs`'s claimed-but-inert arm.
unsafe fn send_pointer_enter(hwnd: HWND, cx: f32, cy: f32) {
    let (sx, sy) = client_to_screen_physical(hwnd, cx, cy);
    SendMessageW(
        hwnd,
        WM_POINTERENTER,
        pack_pointer_wparam(POINTER_ID, true),
        pack_pointer(sx, sy),
    );
}

/// A real `WM_POINTERDOWN`, screen-translated exactly as
/// [`send_pointer_enter`] is. Reaches `window.rs`'s claimed-but-inert arm.
unsafe fn send_pointer_down(hwnd: HWND, cx: f32, cy: f32) {
    let (sx, sy) = client_to_screen_physical(hwnd, cx, cy);
    SendMessageW(
        hwnd,
        WM_POINTERDOWN,
        pack_pointer_wparam(POINTER_ID, true),
        pack_pointer(sx, sy),
    );
}

/// A real `WM_POINTERUPDATE`, screen-translated exactly as
/// [`send_pointer_enter`] is. Reaches `window.rs`'s claimed-but-inert arm.
/// A stationary tap never produces this member on its own (fact 2); this
/// file sends it explicitly anyway, to fire that arm's branch directly
/// rather than relying on incidental coverage.
unsafe fn send_pointer_update(hwnd: HWND, cx: f32, cy: f32) {
    let (sx, sy) = client_to_screen_physical(hwnd, cx, cy);
    SendMessageW(
        hwnd,
        WM_POINTERUPDATE,
        pack_pointer_wparam(POINTER_ID, true),
        pack_pointer(sx, sy),
    );
}

/// A real `WM_POINTERUP`, screen-translated exactly as
/// [`send_pointer_enter`] is, with `POINTER_MESSAGE_FLAG_PRIMARY` set.
/// Reaches `window.rs`'s `WM_POINTERUP` dispatch arm: `focus_on_click`
/// then `hit_test_click`, no `update_hover`.
unsafe fn send_pointer_up(hwnd: HWND, cx: f32, cy: f32) {
    let (sx, sy) = client_to_screen_physical(hwnd, cx, cy);
    SendMessageW(
        hwnd,
        WM_POINTERUP,
        pack_pointer_wparam(POINTER_ID, true),
        pack_pointer(sx, sy),
    );
}

/// A real `WM_POINTERUP`, screen-translated exactly as [`send_pointer_up`]
/// is, but with `POINTER_MESSAGE_FLAG_PRIMARY` **clear** — what a second,
/// simultaneous, non-primary contact's release looks like. `window.rs`'s
/// `WM_POINTERUP` arm still claims this message (returns `LRESULT(0)`) but
/// does not dispatch it, per `pointer_message_is_primary`'s stated limit.
/// See
/// [`a_non_primary_contact_is_claimed_but_dispatches_nothing`].
unsafe fn send_pointer_up_non_primary(hwnd: HWND, cx: f32, cy: f32) {
    let (sx, sy) = client_to_screen_physical(hwnd, cx, cy);
    SendMessageW(
        hwnd,
        WM_POINTERUP,
        pack_pointer_wparam(POINTER_ID, false),
        pack_pointer(sx, sy),
    );
}

/// A real `WM_POINTERLEAVE`, screen-translated exactly as
/// [`send_pointer_enter`] is. Reaches `window.rs`'s claimed-but-inert arm.
unsafe fn send_pointer_leave(hwnd: HWND, cx: f32, cy: f32) {
    let (sx, sy) = client_to_screen_physical(hwnd, cx, cy);
    SendMessageW(
        hwnd,
        WM_POINTERLEAVE,
        pack_pointer_wparam(POINTER_ID, true),
        pack_pointer(sx, sy),
    );
}

/// A `WM_POINTERUP` whose `lParam` is the **client** point packed
/// directly — deliberately **not** translated through `ClientToScreen`.
/// This is the wrong space for a real pointer message (fact 3): it is
/// exactly what a caller upstream of a missing `ScreenToClient` would
/// produce, or equivalently what `window.rs`'s
/// `pointer_message_to_client_dip` would be asked to convert if that
/// translation step were deleted. `wParam` still carries the primary flag
/// (this fixture is about the wrong coordinate space, not about which
/// contact) — see
/// [`a_pointer_message_carrying_an_untranslated_client_point_resolves_to_nothing`].
unsafe fn send_pointer_up_untranslated(hwnd: HWND, cx: f32, cy: f32) {
    SendMessageW(
        hwnd,
        WM_POINTERUP,
        pack_pointer_wparam(POINTER_ID, true),
        pack_pointer(cx, cy),
    );
}

/// A real `WM_LBUTTONUP` at a **physical client** position — the mouse
/// family's `lParam` is already client-space, so this is packed directly,
/// with no `ClientToScreen` step. Delivered synchronously via
/// `SendMessageW`.
unsafe fn send_mouse_up(hwnd: HWND, cx: f32, cy: f32) {
    SendMessageW(hwnd, WM_LBUTTONUP, WPARAM(0), pack_pointer(cx, cy));
}

/// A real `WM_MOUSEMOVE` at a **physical client** position. Used only by
/// [`a_touch_tap_leaves_the_hover_record_untouched`], to put a hover record
/// in place for a touch tap to (not) disturb.
unsafe fn send_mouse_move(hwnd: HWND, cx: f32, cy: f32) {
    SendMessageW(hwnd, WM_MOUSEMOVE, WPARAM(0), pack_pointer(cx, cy));
}

/// The physical client centre of a DIP rectangle at `factor` (`dpi /
/// 96`), matching `event_routing_integration.rs`'s and
/// `hover_transition_integration.rs`'s own inline computation.
fn physical_centre(rect: DipRect, factor: f32) -> (f32, f32) {
    (
        (rect.x + rect.width / 2.0) * factor,
        (rect.y + rect.height / 2.0) * factor,
    )
}

/// Whether `(x, y)` — already in DIP units, inclusive of both edges —
/// falls inside `rect`. Used only by
/// [`a_touch_tap_at_a_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_it`]
/// to prove that fixture's own discriminating power against a missing or
/// doubled scale division, rather than assume it.
fn dip_rect_contains(rect: DipRect, x: f32, y: f32) -> bool {
    x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
}

// ── The shared fixture tree ─────────────────────────────────────────────

/// A `VStack` holding two counter `Text`s and two sibling widgets, each
/// carrying `clicked` and writing a *different* `i32` state: a `Box`
/// (`children[2]`, the non-Button leg — `aspect: 4:1` gives it a
/// non-zero, non-degenerate rectangle, M3-Phase 2 DD-M3-P2-005, the same
/// technique `event_routing_integration.rs`'s `NON_BUTTON_HANDLER_UI`
/// uses) and a `Button` (`children[3]`, the focus leg — M4-Phase 2 T11's
/// focus decision is only observable through a widget focus actually
/// lands on).
const TOUCH_DISPATCH_UI: &str = r#"component TouchDispatch inherits Window {
    state a_hits: i32 = 0
    state b_hits: i32 = 0
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.a_hits}" }
        Text { text: "\{root.b_hits}" }
        Box { aspect: 4:1 fill: #336699cc clicked => { root.a_hits += 1; } }
        Button { text: "B" clicked => { root.b_hits += 1; } }
    }
}"#;

/// Read Box A's and Button B's DIP rectangles back off a loaded, baselined
/// `window`, asserted non-degenerate and disjoint so a tap at either
/// centre cannot resolve to the other. Shared by both setup helpers below
/// — the only difference between them is which normalisation call put the
/// window into its before-state.
unsafe fn widget_rects(window: *mut ffi::WasamoWindow, what: &str) -> (DipRect, DipRect) {
    let (a_rect, b_rect) = {
        let root = (*window).root_widget.as_ref().expect("content root");
        (
            root.children[2].__arranged_rect_for_test(),
            root.children[3].__arranged_rect_for_test(),
        )
    };
    let a_rect = a_rect.expect("Box A must have been laid out");
    let b_rect = b_rect.expect("Button B must have been laid out");
    assert!(
        a_rect.width > 0.0 && a_rect.height > 0.0,
        "{what}: fixture stopped discriminating: Box A rectangle {a_rect:?} must be \
         non-degenerate, or a tap at its centre proves nothing"
    );
    assert!(
        b_rect.width > 0.0 && b_rect.height > 0.0,
        "{what}: fixture stopped discriminating: Button B rectangle {b_rect:?} must be \
         non-degenerate, or a tap at its centre proves nothing"
    );
    assert!(
        a_rect.y + a_rect.height <= b_rect.y,
        "{what}: fixture stopped discriminating: Box A and Button B rectangles \
         ({a_rect:?}, {b_rect:?}) must be disjoint, or a tap meant for one could resolve to \
         the other"
    );
    (a_rect, b_rect)
}

/// Load [`TOUCH_DISPATCH_UI`], move the window to [`WINDOW_X`] /
/// [`WINDOW_Y`], normalise to [`REFERENCE_DPI`], and return the window
/// together with Box A's and Button B's DIP rectangles. What every fixture
/// except the non-unit-scale one uses.
unsafe fn setup_touch_dispatch_window(what: &str) -> (*mut ffi::WasamoWindow, DipRect, DipRect) {
    let ir = lower_ui_to_ir(TOUCH_DISPATCH_UI);
    let window = load_window(&ir);
    let hwnd = (*window).hwnd;
    move_window_to_known_position(hwnd, WINDOW_X, WINDOW_Y, what);
    normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, what);
    let (a_rect, b_rect) = widget_rects(window, what);
    (window, a_rect, b_rect)
}

/// Same as [`setup_touch_dispatch_window`], normalised to `dpi` instead of
/// [`REFERENCE_DPI`]. Used only by
/// [`a_touch_tap_at_a_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_it`].
unsafe fn setup_touch_dispatch_window_at_dpi(
    dpi: u32,
    what: &str,
) -> (*mut ffi::WasamoWindow, DipRect, DipRect) {
    let ir = lower_ui_to_ir(TOUCH_DISPATCH_UI);
    let window = load_window(&ir);
    let hwnd = (*window).hwnd;
    move_window_to_known_position(hwnd, WINDOW_X, WINDOW_Y, what);
    normalise_to_scale_baseline(window, dpi, CLIENT_W, CLIENT_H, what);
    let (a_rect, b_rect) = widget_rects(window, what);
    (window, a_rect, b_rect)
}

/// Read both counters back from `window`'s root.
unsafe fn read_hits(window: *mut ffi::WasamoWindow) -> (i32, i32) {
    let root = (*window).root_widget.as_ref().unwrap();
    (
        read_counter(&root.children[0], "a_hits"),
        read_counter(&root.children[1], "b_hits"),
    )
}

// ── F1 — the central positive claim ─────────────────────────────────────

/// A `WM_POINTERUP` at Box A's centre runs A's handler and not B's.
#[test]
fn a_touch_tap_fires_the_handler_of_the_widget_it_lands_on() {
    run_on_owning_runtime_thread_or_skip(
        "F1: touch tap fires the widget it lands on",
        move || unsafe {
            let (window, a_rect, _b_rect) = setup_touch_dispatch_window("F1 setup");
            let hwnd = (*window).hwnd;
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (ax, ay) = physical_centre(a_rect, factor);
            send_pointer_up(hwnd, ax, ay);
            let (a_hits, b_hits) = read_hits(window);

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                a_hits, 1,
                "a WM_POINTERUP at Box A's centre must run A's handler exactly once"
            );
            assert_eq!(
                b_hits, 0,
                "a WM_POINTERUP at Box A's centre must not run B's handler"
            );
        },
    );
}

// ── F2 — the leg that makes F1 discriminating ───────────────────────────

/// Sending `WM_POINTERUP` with the raw **client** point (no
/// `ClientToScreen`) must resolve to nothing: a correct
/// `pointer_message_to_client_dip` runs the untranslated client point
/// through `ScreenToClient` as if it were a screen point, which — because
/// the window sits at `(`[`WINDOW_X`]`, `[`WINDOW_Y`]`)`, far outside the
/// small client rectangle — lands well outside every widget's rectangle.
#[test]
fn a_pointer_message_carrying_an_untranslated_client_point_resolves_to_nothing() {
    run_on_owning_runtime_thread_or_skip(
        "F2: untranslated client point resolves to nothing",
        move || unsafe {
            let (window, a_rect, _b_rect) = setup_touch_dispatch_window("F2 setup");
            let hwnd = (*window).hwnd;
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (ax, ay) = physical_centre(a_rect, factor);
            assert!(
                ax < CLIENT_W as f32 && ay < CLIENT_H as f32,
                "F2 setup: fixture stopped discriminating: Box A's centre ({ax},{ay}) must sit \
                 inside the {CLIENT_W}x{CLIENT_H} client, or sending it unconverted proves \
                 nothing about ScreenToClient"
            );
            send_pointer_up_untranslated(hwnd, ax, ay);
            let (a_hits, b_hits) = read_hits(window);

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                a_hits, 0,
                "an untranslated client point sent as WM_POINTERUP's lParam must not resolve \
                 to Box A: at window position ({WINDOW_X},{WINDOW_Y}), ScreenToClient \
                 mistranslates it far outside every widget's rectangle. If this assertion \
                 fails, the ScreenToClient translation is not actually running."
            );
            assert_eq!(
                b_hits, 0,
                "an untranslated client point must not resolve to Button B either"
            );
        },
    );
}

// ── F3 / F4 — single-delivery count, and its agreement leg ─────────────

/// `ENTER`, `DOWN`, `UP`, `LEAVE` for one stationary contact (fact 2's
/// order, no `UPDATE`) must run the handler exactly once.
#[test]
fn one_touch_contact_reaches_the_handler_exactly_once() {
    run_on_owning_runtime_thread_or_skip(
        "F3: one contact reaches the handler once",
        move || unsafe {
            let (window, a_rect, _b_rect) = setup_touch_dispatch_window("F3 setup");
            let hwnd = (*window).hwnd;
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
            let (ax, ay) = physical_centre(a_rect, factor);

            send_pointer_enter(hwnd, ax, ay);
            send_pointer_down(hwnd, ax, ay);
            send_pointer_up(hwnd, ax, ay);
            send_pointer_leave(hwnd, ax, ay);
            let (a_hits, _b_hits) = read_hits(window);

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                a_hits, 1,
                "one contact's ENTER/DOWN/UP/LEAVE sequence must run the handler exactly once"
            );
        },
    );
}

/// The agreement leg for
/// [`one_touch_contact_reaches_the_handler_exactly_once`]: without a way to
/// move the counter past 1, "exactly once" would be indistinguishable from
/// a counter with a ceiling. A second, independent delivery at the same
/// point — a `WM_LBUTTONUP`, what an *unsuppressed* promotion would
/// additionally produce (fact 4's unclaimed leg) — moves it again.
#[test]
fn two_deliveries_of_the_same_contact_are_reachable() {
    run_on_owning_runtime_thread_or_skip(
        "F4: two deliveries both reach the handler",
        move || unsafe {
            let (window, a_rect, _b_rect) = setup_touch_dispatch_window("F4 setup");
            let hwnd = (*window).hwnd;
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
            let (ax, ay) = physical_centre(a_rect, factor);

            send_pointer_up(hwnd, ax, ay);
            let (after_pointer, _) = read_hits(window);

            send_mouse_up(hwnd, ax, ay);
            let (after_both, _) = read_hits(window);

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                after_pointer, 1,
                "the WM_POINTERUP delivery must have run the handler once"
            );
            assert_eq!(
                after_both, 2,
                "a second, independent delivery (WM_LBUTTONUP) at the same point must run the \
                 handler again, reaching 2 — this is the counter proving 'exactly once' above \
                 was a real count and not a ceiling"
            );
        },
    );
}

// ── F5 — the claimed-but-inert arm fires and changes nothing ────────────

/// `ENTER`, `DOWN`, `UPDATE`, `LEAVE` (no `UP`), sent individually, must
/// each be a true no-op: no counter moves, the retained hover record is
/// untouched, the retained focus record is untouched.
#[test]
fn the_claimed_pointer_messages_that_act_on_nothing_change_no_state() {
    run_on_owning_runtime_thread_or_skip(
        "F5: claimed-but-inert arm changes nothing",
        move || unsafe {
            let (window, a_rect, _b_rect) = setup_touch_dispatch_window("F5 setup");
            let hwnd = (*window).hwnd;
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
            let (ax, ay) = physical_centre(a_rect, factor);

            let hover_before = ffi::__hover_target_for_test(window);
            let focus_before = ffi::__focus_path_for_test(window);
            assert_eq!(
                hover_before, None,
                "F5 setup: fixture stopped discriminating: a freshly baselined window must \
                 start with no hover"
            );
            assert_eq!(
                focus_before, None,
                "F5 setup: fixture stopped discriminating: a freshly baselined window must \
                 start with no focus"
            );

            let assert_untouched = |window: *mut ffi::WasamoWindow, step: &str| {
                let (a_hits, b_hits) = read_hits(window);
                assert_eq!(a_hits, 0, "{step}: A's counter must not move");
                assert_eq!(b_hits, 0, "{step}: B's counter must not move");
                assert_eq!(
                    ffi::__hover_target_for_test(window),
                    hover_before,
                    "{step}: the retained hover record must be unchanged"
                );
                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    focus_before,
                    "{step}: the retained focus record must be unchanged"
                );
            };

            send_pointer_enter(hwnd, ax, ay);
            assert_untouched(window, "after WM_POINTERENTER");
            send_pointer_down(hwnd, ax, ay);
            assert_untouched(window, "after WM_POINTERDOWN");
            send_pointer_update(hwnd, ax, ay);
            assert_untouched(window, "after WM_POINTERUPDATE");
            send_pointer_leave(hwnd, ax, ay);
            assert_untouched(window, "after WM_POINTERLEAVE");

            ffi::wasamo_window_destroy(window);
        },
    );
}

// ── F6 — touch moves focus, exactly as a click does ─────────────────────

/// A `WM_POINTERUP` on Button B moves focus to the same path a
/// `WM_LBUTTONUP` at the same client point does. Two independent windows,
/// so "nothing focused beforehand" is established by reading it back in
/// each rather than inherited from an earlier step.
#[test]
fn a_touch_tap_moves_focus_the_way_a_click_does() {
    run_on_owning_runtime_thread_or_skip(
        "F6: touch tap moves focus like a click",
        move || unsafe {
            let (pointer_window, _a_rect, b_rect) =
                setup_touch_dispatch_window("F6 pointer-window setup");
            let pointer_hwnd = (*pointer_window).hwnd;
            let pointer_factor =
                ffi::__window_scale_dpi_for_test(pointer_window) as f32 / REFERENCE_DPI as f32;
            assert_eq!(
                ffi::__focus_path_for_test(pointer_window),
                None,
                "F6 pointer-window setup: fixture stopped discriminating: a freshly baselined \
                 window must start with no focus, or 'focus moved' below could be inherited \
                 rather than caused"
            );
            let (bx, by) = physical_centre(b_rect, pointer_factor);
            send_pointer_up(pointer_hwnd, bx, by);
            let focus_after_pointer = ffi::__focus_path_for_test(pointer_window);
            ffi::wasamo_window_destroy(pointer_window);

            let (mouse_window, _a_rect2, b_rect2) =
                setup_touch_dispatch_window("F6 mouse-window setup");
            let mouse_hwnd = (*mouse_window).hwnd;
            let mouse_factor =
                ffi::__window_scale_dpi_for_test(mouse_window) as f32 / REFERENCE_DPI as f32;
            assert_eq!(
                ffi::__focus_path_for_test(mouse_window),
                None,
                "F6 mouse-window setup: fixture stopped discriminating: a freshly baselined \
                 window must start with no focus"
            );
            let (bx2, by2) = physical_centre(b_rect2, mouse_factor);
            send_mouse_up(mouse_hwnd, bx2, by2);
            let focus_after_mouse = ffi::__focus_path_for_test(mouse_window);
            ffi::wasamo_window_destroy(mouse_window);

            assert!(
                focus_after_pointer.is_some(),
                "a WM_POINTERUP on Button B must move focus, not leave it None"
            );
            assert_eq!(
                focus_after_pointer, focus_after_mouse,
                "a WM_POINTERUP and a WM_LBUTTONUP at the same client point on the same tree \
                 must move focus to the same path"
            );
        },
    );
}

// ── F7 — touch writes no hover state ─────────────────────────────────────

/// A tap with nothing hovered beforehand leaves the hover record `None`.
/// A real `WM_MOUSEMOVE` over Button B then sets a hover record; a
/// subsequent tap on a *different* widget (Box A) through the pointer
/// family must leave that record exactly where it was.
#[test]
fn a_touch_tap_leaves_the_hover_record_untouched() {
    run_on_owning_runtime_thread_or_skip("F7: touch tap leaves hover untouched", move || unsafe {
        let (window, a_rect, b_rect) = setup_touch_dispatch_window("F7 setup");
        let hwnd = (*window).hwnd;
        let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

        assert_eq!(
            ffi::__hover_target_for_test(window),
            None,
            "F7 setup: fixture stopped discriminating: a freshly baselined window must start \
             with no hover"
        );

        // Leg 1: tap Box A with nothing hovered beforehand. Hover stays None.
        let (ax, ay) = physical_centre(a_rect, factor);
        send_pointer_up(hwnd, ax, ay);
        assert_eq!(
            ffi::__hover_target_for_test(window),
            None,
            "a touch tap with nothing hovered beforehand must leave the hover record None"
        );

        // Leg 2: a real WM_MOUSEMOVE over Button B sets a hover record.
        let (bx, by) = physical_centre(b_rect, factor);
        send_mouse_move(hwnd, bx, by);
        let hover_after_move = ffi::__hover_target_for_test(window);
        assert_eq!(
            hover_after_move,
            Some(vec![3]),
            "F7: fixture stopped discriminating: a WM_MOUSEMOVE over Button B must set the \
             hover record to Button B's path, or the disturbance leg below proves nothing"
        );

        // Tap Box A (a different widget) through the pointer family. The
        // hover record must be exactly what WM_MOUSEMOVE left it as.
        send_pointer_up(hwnd, ax, ay);
        let hover_after_tap = ffi::__hover_target_for_test(window);

        ffi::wasamo_window_destroy(window);

        assert_eq!(
            hover_after_tap, hover_after_move,
            "a touch tap on a different widget must leave a pre-existing hover record \
             untouched, not clear it and not move it to the tapped widget"
        );
    });
}

// ── F8 — non-unit scale ──────────────────────────────────────────────────

/// The same central claim as
/// [`a_touch_tap_fires_the_handler_of_the_widget_it_lands_on`], normalised
/// to [`SCALED_DPI`] (scale 1.25) instead of [`REFERENCE_DPI`] — but
/// targeted at **Button B**, not Box A. Box A spans the full client width
/// and ~72 DIP of height (`VStack{spacing:0 padding:0 Text Text
/// Box{aspect:4:1} Button}`), so both a missing division (`pair_to_dip`
/// dropped) and a doubled division (`pair_to_dip` applied twice) still
/// land inside Box A at this window's size — the fixture would stay green
/// under either mutation. Button B is small enough that both mutations
/// move the resolved point outside its rectangle instead; the precondition
/// assertion below proves that rather than assumes it, and the negative
/// leg proves the consequence rather than the geometry alone.
#[test]
fn a_touch_tap_at_a_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_it() {
    run_on_owning_runtime_thread_or_skip("F8: touch tap at 1.25x scale", move || unsafe {
        let (window, _a_rect, b_rect) = setup_touch_dispatch_window_at_dpi(SCALED_DPI, "F8 setup");
        let hwnd = (*window).hwnd;
        let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
        assert!(
            (factor - 1.25).abs() < 0.001,
            "F8 setup: fixture stopped discriminating: the scale factor must be 1.25 at \
             {SCALED_DPI} DPI, got {factor}"
        );

        // The physical point that a correct implementation divides by
        // `factor` down to Button B's DIP centre.
        let (bx, by) = physical_centre(b_rect, factor);

        // Precondition: (bx, by) read directly as a DIP point — with no
        // division applied — is what a missing division would use as the
        // resolved coordinate, and what a doubled division would move even
        // further from. It must land outside Button B's rectangle, or this
        // fixture cannot tell a missing/doubled division apart from a
        // correct one. If this assertion ever fails, the fixture has
        // stopped discriminating and must be re-shaped (Button B moved,
        // resized, or the scale changed).
        assert!(
            !dip_rect_contains(b_rect, bx, by),
            "F8 setup: fixture stopped discriminating: the undivided physical point \
             ({bx}, {by}) must land outside Button B's DIP rectangle {b_rect:?}, or a \
             missing or doubled division by scale would resolve to the same widget as \
             a correct implementation and this fixture could not catch it"
        );

        send_pointer_up(hwnd, bx, by);
        let (a_hits, b_hits) = read_hits(window);
        assert_eq!(
            b_hits, 1,
            "at a 1.25x scale, a WM_POINTERUP at Button B's centre must run B's handler \
             exactly once — a missing or doubled division by scale would resolve \
             elsewhere or nowhere"
        );
        assert_eq!(
            a_hits, 0,
            "at a 1.25x scale, a WM_POINTERUP at Button B's centre must not run A's \
             handler"
        );

        // Negative leg: a wire client point of (bx * factor, by * factor)
        // is exactly what a *correct* division maps back down to (bx, by)
        // — the undivided physical point the precondition above just
        // proved sits outside Button B. This is the leg that makes the
        // precondition load-bearing: a real message that resolves to that
        // point must fire nothing, not merely look like it should.
        send_pointer_up(hwnd, bx * factor, by * factor);
        let (a_hits_after, b_hits_after) = read_hits(window);

        ffi::wasamo_window_destroy(window);

        assert_eq!(
            (a_hits_after, b_hits_after),
            (a_hits, b_hits),
            "a WM_POINTERUP whose wire point divides down to the undivided physical \
             point ({bx}, {by}) must resolve to nothing: neither counter may move \
             past its post-positive-leg value"
        );
    });
}

// ── F9 — the two families agree ─────────────────────────────────────────

/// The same client point on Box A, delivered once as `WM_LBUTTONUP` and
/// once as `WM_POINTERUP` (two independent windows), reaches the same
/// handler.
#[test]
fn the_mouse_and_pointer_families_agree_on_the_same_point() {
    run_on_owning_runtime_thread_or_skip("F9: mouse and pointer families agree", move || unsafe {
        let (mouse_window, a_rect, _b_rect) = setup_touch_dispatch_window("F9 mouse-window setup");
        let mouse_hwnd = (*mouse_window).hwnd;
        let mouse_factor =
            ffi::__window_scale_dpi_for_test(mouse_window) as f32 / REFERENCE_DPI as f32;
        let (mx, my) = physical_centre(a_rect, mouse_factor);
        send_mouse_up(mouse_hwnd, mx, my);
        let (mouse_a_hits, mouse_b_hits) = read_hits(mouse_window);
        ffi::wasamo_window_destroy(mouse_window);

        let (pointer_window, a_rect2, _b_rect2) =
            setup_touch_dispatch_window("F9 pointer-window setup");
        let pointer_hwnd = (*pointer_window).hwnd;
        let pointer_factor =
            ffi::__window_scale_dpi_for_test(pointer_window) as f32 / REFERENCE_DPI as f32;
        let (px, py) = physical_centre(a_rect2, pointer_factor);
        send_pointer_up(pointer_hwnd, px, py);
        let (pointer_a_hits, pointer_b_hits) = read_hits(pointer_window);
        ffi::wasamo_window_destroy(pointer_window);

        assert_eq!(
            mouse_a_hits, 1,
            "F9: fixture stopped discriminating: WM_LBUTTONUP at Box A's centre must run A's \
             handler, or the agreement claim below is vacuous"
        );
        assert_eq!(
            mouse_a_hits, pointer_a_hits,
            "the same client point on Box A must reach the same handler whether delivered as \
             WM_LBUTTONUP or WM_POINTERUP"
        );
        assert_eq!(
            mouse_b_hits, pointer_b_hits,
            "neither family should have touched Button B's counter"
        );
    });
}

// ── F10 — a non-primary contact is claimed but dispatches nothing ───────

/// A `WM_POINTERUP` with `POINTER_MESSAGE_FLAG_PRIMARY` clear — what a
/// second, simultaneous contact's release looks like — at Button B's
/// centre must run no handler, move no focus, and touch no hover record.
/// `window.rs`'s `WM_POINTERUP` arm still claims the message (the
/// unconditional `return LRESULT(0)` does not depend on this flag); this
/// fixture is the branch test for the *dispatch* gate
/// `pointer_message_is_primary` adds (implementation-gates trap #4), the
/// counterpart to
/// [`the_claimed_pointer_messages_that_act_on_nothing_change_no_state`] for
/// the claimed-but-inert arm above.
#[test]
fn a_non_primary_contact_is_claimed_but_dispatches_nothing() {
    run_on_owning_runtime_thread_or_skip(
        "F10: non-primary contact is claimed but inert",
        move || unsafe {
            let (window, _a_rect, b_rect) = setup_touch_dispatch_window("F10 setup");
            let hwnd = (*window).hwnd;
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
            let (bx, by) = physical_centre(b_rect, factor);

            let hover_before = ffi::__hover_target_for_test(window);
            let focus_before = ffi::__focus_path_for_test(window);
            assert_eq!(
                focus_before, None,
                "F10 setup: fixture stopped discriminating: a freshly baselined window \
                 must start with no focus, or 'focus did not move' below could be \
                 inherited rather than caused"
            );

            send_pointer_up_non_primary(hwnd, bx, by);
            let (a_hits, b_hits) = read_hits(window);
            let hover_after = ffi::__hover_target_for_test(window);
            let focus_after = ffi::__focus_path_for_test(window);

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                a_hits, 0,
                "a non-primary WM_POINTERUP at Button B's centre must not run A's handler"
            );
            assert_eq!(
                b_hits, 0,
                "a non-primary WM_POINTERUP at Button B's centre must run no handler"
            );
            assert_eq!(
                focus_after, focus_before,
                "a non-primary WM_POINTERUP must not move focus"
            );
            assert_eq!(
                hover_after, hover_before,
                "a non-primary WM_POINTERUP must not touch the hover record"
            );
        },
    );
}
