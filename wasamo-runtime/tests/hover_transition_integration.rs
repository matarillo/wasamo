//! Mock-free Windows integration evidence for M4-Phase 2 T4 stage 2
//! ([DD-M4-P2-001](../../process/milestone-4/phase-2/decisions/dd-m4-p2-001-event-routing-model.md)
//! §Pointer capture, hover, pressed;
//! [DD-M4-P2-002](../../process/milestone-4/phase-2/decisions/dd-m4-p2-002-hit-testing-and-generic-click.md)):
//! hover / pressed enter-leave transitions over a real widget tree built
//! from real `.ui` source through `wasamoc` -> IR -> `wasamo_load_ui`,
//! driven by real `WM_MOUSEMOVE` / `WM_LBUTTONDOWN` / `WM_LBUTTONUP` /
//! `WM_MOUSELEAVE` messages through the real window procedure
//! (`window.rs`'s four pointer arms), with live widget state read back
//! afterwards. Nothing is mocked.
//!
//! Before this file, **zero** tests in the whole suite exercised hover —
//! measured at the T4 start gate
//! ([log.md §T4](../../process/milestone-4/phase-2/implementation/log.md#t4--hover-and-pressed-behind-the-routing-model),
//! "Nothing in the suite exercises hover"): `rg
//! "update_hover|clear_hover|ButtonState|hovered"` over
//! `wasamo-runtime/tests` returned zero hits. Every fixture below is a
//! genuine red-before / green-after witness for one of the four pointer
//! arms, not a re-statement of something already pinned.
//!
//! # The two read-backs, and why both are asserted together
//!
//! - `WidgetNode::__button_state_for_test() -> Option<&'static str>` —
//!   `"normal"` / `"hovered"` / `"pressed"`, `None` for a non-Button
//!   widget. The **painted** state.
//! - `ffi::__hover_target_for_test(window) -> Option<Vec<usize>>` — the
//!   window's **retained** hover path (`WindowState::hover`, a
//!   `HoverState`).
//!
//! `widget.rs`'s `HoverState` doc comment states the invariant these two
//! are supposed to satisfy: "at most one node in the tree has `state !=
//! ButtonState::Normal`, and it is exactly the node this path names" —
//! implementation-gates trap #3 (parallel / derived data drift). A
//! fixture that only reads the painted state would not notice the
//! retained record drifting out of step with it (e.g. a leave that resets
//! the paint but forgets to clear `hover.target`, or vice versa), so every
//! fixture below asserts both after every step where either could have
//! changed.
//!
//! # What each fixture is for
//!
//! - [`a_button_moves_through_hover_press_release_and_leave_in_one_pointer_sequence`]
//!   (Fixture 1) — the basic enter / press / release / leave cycle on one
//!   Button, firing all four pointer arms in the order a real user
//!   generates them.
//! - [`only_the_topmost_of_two_overlapping_buttons_hovers_and_the_wide_one_still_hovers_where_uncovered`]
//!   (Fixture 2) — **the central claim**:
//!   `docs/architecture.md` §13.2 "computed as enter / leave transitions
//!   against the resolved target rather than by a whole-tree walk". The
//!   pre-T4 whole-tree walk hovered every Button whose rectangle
//!   contained the point; this fixture's difference leg is the assertion
//!   that goes red against that. Its agreement leg (a point inside the
//!   wide Button alone) is paired with the difference leg deliberately —
//!   without it, an implementation that simply never hovered the wide
//!   Button at all would also pass the difference leg.
//! - [`a_disabled_button_paints_nothing_but_still_occludes_and_the_previously_hovered_button_still_leaves`]
//!   (Fixture 3) — `docs/dsl_spec.md` §4.8 "Hover / press visual
//!   transitions are frozen" for a disabled Button: it is still the
//!   resolved target (DD-M4-P2-002 occlusion), but paints nothing and
//!   retains nothing, and — the assertion that shows the leave half runs
//!   independently of what the enter half paints — whatever was
//!   previously hovered still leaves. Its agreement leg (the same
//!   rectangle, enabled) distinguishes "disabled is skipped" from "that
//!   position is never hovered".
//! - [`wm_mouseleave_clears_the_retained_hover_target_and_a_second_leave_is_a_no_op`]
//!   (Fixture 4) — `clear_hover`'s two arms directly: `hover.target ==
//!   Some(_)` (the first `WM_MOUSELEAVE`) and `hover.target == None` (the
//!   second, fired with nothing retained — must not panic or change
//!   anything).
//! - [`hovering_a_non_button_target_paints_nothing_and_still_leaves_the_button_it_partly_covers`]
//!   (Fixture 5) — a plain `Box` (no `ButtonState` at all) becomes the
//!   resolved target and still drives a leave on the Button it partly
//!   covers. This is the shape the gallery's lightbox scrim has (a later
//!   `ZStack` child painting over earlier content —
//!   [log.md §T4 fact 2](../../process/milestone-4/phase-2/implementation/log.md#t4--hover-and-pressed-behind-the-routing-model)
//!   measured the pre-T4 defect in exactly that gallery shape).
//!
//! # Helpers copied from `event_routing_integration.rs`
//!
//! Cargo compiles every `tests/*.rs` file as its own crate, so nothing in
//! that file is importable here. `event_routing_integration.rs`'s own
//! module header states its copying convention ("Copy verbatim what you
//! need... from `hit_resolution_integration.rs`"); this file continues
//! the chain. `lower_ui_to_ir`, `load_window`, `client_extent`,
//! `display_limits`, `window_rect`, `frame_thickness`,
//! `send_dpi_change_to_client`, and `normalise_to_reference_baseline`
//! below are copied verbatim (including their doc comments, which still
//! name `hit_resolution_integration.rs` — the file they were originally
//! written for) from `event_routing_integration.rs`. This file's own
//! additions are the four pointer-message senders (`send_mouse_move`,
//! `send_mouse_down`, `send_mouse_up`, `send_mouse_leave`) and the two
//! pure containment helpers (`dip_contains`, `dip_rect_contains`) — `hit`
//! is a private module (`mod hit;` in `lib.rs`), so `hit::contains` is
//! unreachable from this crate and is mirrored locally instead, with the
//! same left/top-inclusive, right/bottom-exclusive edge convention
//! `hit.rs`'s own doc comment states.
//!
//! # The client stays small (M4-Phase 1 T8 finding)
//!
//! Every fixture normalises to 96 DPI at an explicitly chosen 360x240
//! physical client — the same ceiling `event_routing_integration.rs`'s
//! and `hit_resolution_integration.rs`'s module headers record (a hosted
//! CI desktop failed a larger request twice). No fixture in this file
//! ever changes scale, so 360x240 is also each fixture's only client
//! extent. Every click / move coordinate is derived from
//! `__arranged_rect_for_test()` (the layout-derived DIP rectangle T1/T2
//! store), never from a hand-worked-out constant, and the geometric
//! relationship each fixture depends on (containment, non-containment) is
//! asserted before the point is used — a layout change that broke the
//! assumption then fails the fixture loudly instead of silently testing
//! nothing.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::ffi::c_void;
use std::ptr;

use wasamo_runtime::{ffi, DipRect};

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, SendMessageW, SM_CXMAXTRACK, SM_CXSCREEN,
    SM_CYMAXTRACK, SM_CYSCREEN, WM_DPICHANGED, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
};

/// `WM_MOUSELEAVE` is not exported from `WindowsAndMessaging` in the
/// `windows` crate version this workspace pins — the same fact
/// `window.rs` records next to its own copy of this constant.
const WM_MOUSELEAVE: u32 = 0x02A3;

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

    let path = "<hover-transition-integration>";
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

// ── This file's own helpers ─────────────────────────────────────────────

/// Pack a **physical** client-area pointer position into the `lParam`
/// shape every mouse message below uses — the same packing
/// `event_routing_integration.rs::send_click` and `window::pointer_physical`
/// (its inverse) use, hoisted here because four message senders need it
/// instead of one.
fn pack_pointer(x: f32, y: f32) -> LPARAM {
    let packed = ((y.round() as i32 as u32) << 16) | (x.round() as i32 as u32 & 0xFFFF);
    LPARAM(packed as i32 as isize)
}

/// A real `WM_MOUSEMOVE` at a **physical** client position, delivered
/// synchronously via `SendMessageW`. Reaches `window.rs`'s `WM_MOUSEMOVE`
/// arm, which calls `WidgetNode::update_hover` with `down =
/// state.mouse_down` — `false` unless a preceding [`send_mouse_down`] in
/// the same fixture set it.
unsafe fn send_mouse_move(hwnd: HWND, x: f32, y: f32) {
    SendMessageW(hwnd, WM_MOUSEMOVE, WPARAM(0), pack_pointer(x, y));
}

/// A real `WM_LBUTTONDOWN` at a **physical** client position. Reaches
/// `window.rs`'s `WM_LBUTTONDOWN` arm, which sets `state.mouse_down =
/// true` and calls `update_hover(..., down: true)` — the paint transition
/// to `ButtonState::Pressed`.
unsafe fn send_mouse_down(hwnd: HWND, x: f32, y: f32) {
    SendMessageW(hwnd, WM_LBUTTONDOWN, WPARAM(0), pack_pointer(x, y));
}

/// A real `WM_LBUTTONUP` at a **physical** client position. Reaches
/// `window.rs`'s `WM_LBUTTONUP` arm, which runs `hit_test_click` first
/// (T3's click dispatch — a no-op unless the target carries a handler),
/// then `update_hover(..., down: false)` at the **same** point.
unsafe fn send_mouse_up(hwnd: HWND, x: f32, y: f32) {
    SendMessageW(hwnd, WM_LBUTTONUP, WPARAM(0), pack_pointer(x, y));
}

/// A real `WM_MOUSELEAVE`, carrying no pointer position — `window.rs`'s
/// arm resets `state.tracking_mouse` and calls `WidgetNode::clear_hover`
/// unconditionally on whatever is currently retained.
unsafe fn send_mouse_leave(hwnd: HWND) {
    SendMessageW(hwnd, WM_MOUSELEAVE, WPARAM(0), LPARAM(0));
}

/// Local mirror of `hit::contains`'s edge convention — **left/top
/// inclusive, right/bottom exclusive** — copied because `hit` is a
/// private module (`mod hit;` in `lib.rs`) and unreachable from this
/// crate. Lets a fixture assert exactly the relationship
/// `hit::resolve_topmost` will apply to the same point, rather than a
/// symmetric approximation of it.
fn dip_contains(rect: DipRect, x: f32, y: f32) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// Whether `outer` fully contains `inner`, edge-inclusive on all four
/// sides (a rectangle contains itself). Used by Fixture 2 to assert "the
/// narrow one sits inside the wide one's rectangle" as a measured fact
/// rather than an assumption.
fn dip_rect_contains(outer: DipRect, inner: DipRect) -> bool {
    outer.x <= inner.x
        && inner.x + inner.width <= outer.x + outer.width
        && outer.y <= inner.y
        && inner.y + inner.height <= outer.y + outer.height
}

// ── F1 — enter / press / release / leave ────────────────────────────────

const HOVER_PRESS_RELEASE_LEAVE_UI: &str = r#"component HoverPressReleaseLeave inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "target" }
    }
}"#;

/// The basic pointer sequence on one Button, firing all four of
/// `window.rs`'s hover-related arms in the order a real user generates
/// them: `WM_MOUSEMOVE` onto it, `WM_LBUTTONDOWN`, `WM_LBUTTONUP` (release
/// still inside — `docs/architecture.md` §13.2 "so the widget that paints
/// pressed is the widget a release would dispatch to"), `WM_MOUSEMOVE` to
/// a point outside every widget.
#[test]
fn a_button_moves_through_hover_press_release_and_leave_in_one_pointer_sequence() {
    run_on_owning_runtime_thread_or_skip("F1: enter / press / release / leave", move || {
        let ir = lower_ui_to_ir(HOVER_PRESS_RELEASE_LEAVE_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F1 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let button_rect = {
                let root = (*window).root_widget.as_ref().expect("content root");
                root.children[0].__arranged_rect_for_test()
            }
            .expect("Button must have been laid out");
            assert!(
                button_rect.width > 0.0 && button_rect.height > 0.0,
                "fixture stopped discriminating: the Button rectangle {button_rect:?} must be \
                 non-degenerate, or a pointer at its centre proves nothing"
            );
            assert!(
                button_rect.y + button_rect.height < CLIENT_H as f32,
                "fixture stopped discriminating: the Button rectangle {button_rect:?} must \
                 leave room below it inside the {CLIENT_H} DIP client, or the 'move outside \
                 every widget' step below has nowhere to land"
            );

            let (cx, cy) = (
                (button_rect.x + button_rect.width / 2.0) * factor,
                (button_rect.y + button_rect.height / 2.0) * factor,
            );

            // Step 1: WM_MOUSEMOVE onto the Button.
            send_mouse_move(hwnd, cx, cy);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("hovered"),
                    "docs/architecture.md §13.2: entering the resolved target's rectangle \
                     must paint it Hovered"
                );
            }
            assert_eq!(
                ffi::__hover_target_for_test(window),
                Some(vec![0]),
                "the retained hover path must name the entered Button (implementation-gates \
                 trap #3: the record and the painted state are written together)"
            );

            // Step 2: WM_LBUTTONDOWN on the same point.
            send_mouse_down(hwnd, cx, cy);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("pressed"),
                    "docs/architecture.md §13.2: a pointer-down over the resolved target must \
                     paint it Pressed"
                );
            }
            assert_eq!(ffi::__hover_target_for_test(window), Some(vec![0]));

            // Step 3: WM_LBUTTONUP at the same point — the release is still inside.
            send_mouse_up(hwnd, cx, cy);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("hovered"),
                    "docs/architecture.md §13.2: 'so the widget that paints pressed is the \
                     widget a release would dispatch to' — releasing inside the same \
                     rectangle must fall back to Hovered, not Normal"
                );
            }
            assert_eq!(ffi::__hover_target_for_test(window), Some(vec![0]));

            // Step 4: WM_MOUSEMOVE to a point outside every widget.
            let (ox, oy) = (
                button_rect.x + button_rect.width / 2.0,
                (button_rect.y + button_rect.height + CLIENT_H as f32) / 2.0,
            );
            assert!(
                !dip_contains(button_rect, ox, oy),
                "fixture stopped discriminating: the 'outside' point ({ox}, {oy}) must not be \
                 inside the Button rectangle {button_rect:?}, or this step re-hovers instead \
                 of leaving"
            );
            send_mouse_move(hwnd, ox * factor, oy * factor);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("normal"),
                    "docs/architecture.md §13.2: leaving the resolved target must reset it to \
                     Normal"
                );
            }
            assert_eq!(
                ffi::__hover_target_for_test(window),
                None,
                "with the pointer over no widget, the retained hover path must be cleared"
            );

            ffi::wasamo_window_destroy(window);
        }
    });
}

// ── F2 — overlap: only the topmost widget reacts (the central claim) ───────

const OVERLAP_TOPMOST_ONLY_UI: &str = r#"component OverlapTopmostOnly inherits Window {
    ZStack {
        Button { text: "a considerably wider button label" }
        Button { text: "x" }
    }
}"#;

/// **The central claim**, paired difference + agreement legs in one test
/// (per the task brief's pairing requirement). Both Buttons land centred
/// by ZStack's default per-child alignment (`docs/dsl_spec.md` §4.13 "an
/// omitted ZStack alignment [defaults] to center"), so the narrow one
/// (later in document order — topmost, per `docs/dsl_spec.md` §4.13
/// "children overlap and paint back-to-front in document order") sits
/// inside the wide one's rectangle. Geometry is measured and asserted,
/// not assumed.
///
/// - **Difference leg:** a point inside both rectangles must hover only
///   the narrow (topmost) Button; the wide one underneath must stay
///   `"normal"`. `docs/architecture.md` §13.2's "against the resolved
///   target rather than by a whole-tree walk" is exactly the rule this
///   goes red against under the pre-T4 walk, which hovered both.
/// - **Agreement leg:** a point inside the wide Button but outside the
///   narrow one must hover the wide Button. Without this leg, an
///   implementation that simply never hovered the wide Button at all
///   would also pass the difference leg.
#[test]
fn only_the_topmost_of_two_overlapping_buttons_hovers_and_the_wide_one_still_hovers_where_uncovered(
) {
    run_on_owning_runtime_thread_or_skip("F2: overlap, topmost only", move || {
        let ir = lower_ui_to_ir(OVERLAP_TOPMOST_ONLY_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F2 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (wide_rect, narrow_rect) = {
                let root = (*window).root_widget.as_ref().expect("content root");
                (
                    root.children[0].__arranged_rect_for_test(),
                    root.children[1].__arranged_rect_for_test(),
                )
            };
            let wide_rect = wide_rect.expect("wide Button must have been laid out");
            let narrow_rect = narrow_rect.expect("narrow Button must have been laid out");
            assert!(
                wide_rect.width > 0.0 && wide_rect.height > 0.0,
                "fixture stopped discriminating: the wide Button rectangle {wide_rect:?} \
                 must be non-degenerate"
            );
            assert!(
                narrow_rect.width > 0.0 && narrow_rect.height > 0.0,
                "fixture stopped discriminating: the narrow Button rectangle {narrow_rect:?} \
                 must be non-degenerate"
            );
            assert!(
                wide_rect.width - narrow_rect.width > 20.0,
                "fixture stopped discriminating: the wide Button must be meaningfully wider \
                 than the narrow one (wide {wide_rect:?}, narrow {narrow_rect:?}), or the \
                 agreement-leg point below has no room to land inside the wide rectangle alone"
            );
            assert!(
                dip_rect_contains(wide_rect, narrow_rect),
                "geometry this fixture depends on (docs/dsl_spec.md §4.13's default centre \
                 alignment): the narrow rectangle {narrow_rect:?} must sit entirely inside \
                 the wide one {wide_rect:?}"
            );

            // A point inside both rectangles: the narrow Button's own centre, which the
            // full containment just asserted guarantees is also inside the wide one.
            let (both_x, both_y) = (
                narrow_rect.x + narrow_rect.width / 2.0,
                narrow_rect.y + narrow_rect.height / 2.0,
            );
            assert!(
                dip_contains(narrow_rect, both_x, both_y)
                    && dip_contains(wide_rect, both_x, both_y),
                "fixture stopped discriminating: ({both_x}, {both_y}) must be inside both \
                 rectangles"
            );

            // A point inside the wide rectangle but outside the narrow one: midway
            // between the wide rectangle's left edge and the narrow rectangle's left
            // edge, a gap the full-containment fact above guarantees is non-empty.
            let (wide_only_x, wide_only_y) = (
                (wide_rect.x + narrow_rect.x) / 2.0,
                wide_rect.y + wide_rect.height / 2.0,
            );
            assert!(
                dip_contains(wide_rect, wide_only_x, wide_only_y)
                    && !dip_contains(narrow_rect, wide_only_x, wide_only_y),
                "fixture stopped discriminating: ({wide_only_x}, {wide_only_y}) must be inside \
                 the wide rectangle {wide_rect:?} and outside the narrow one {narrow_rect:?}, \
                 or the agreement leg below is not testing what it claims to"
            );

            // Difference leg.
            send_mouse_move(hwnd, both_x * factor, both_y * factor);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[1].__button_state_for_test(),
                    Some("hovered"),
                    "DD-M4-P2-002 target selection (topmost-first): the later (topmost- \
                     painted) Button must be the one that paints hover where both rectangles \
                     contain the point"
                );
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("normal"),
                    "docs/architecture.md §13.2: hover is computed against the single \
                     resolved target, not a whole-tree walk — the wide Button underneath must \
                     NOT paint hover just because its own rectangle also contains the point"
                );
            }
            assert_eq!(ffi::__hover_target_for_test(window), Some(vec![1]));

            // Agreement leg.
            send_mouse_move(hwnd, wide_only_x * factor, wide_only_y * factor);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("hovered"),
                    "the wide Button must hover where it alone contains the point — without \
                     this leg, never hovering the wide Button at all would also have passed \
                     the difference leg above"
                );
                assert_eq!(
                    root.children[1].__button_state_for_test(),
                    Some("normal"),
                    "the narrow Button must have left once the resolved target moved off it"
                );
            }
            assert_eq!(ffi::__hover_target_for_test(window), Some(vec![0]));

            ffi::wasamo_window_destroy(window);
        }
    });
}

// ── F3 — a disabled Button paints nothing but still occludes ───────────────

/// `enabled` is bound to `gate_enabled` (a `state` initialised `false`),
/// not authored as the literal `enabled: false` on the plain `Button` —
/// see the fixture's own discriminating-guard assertion for why (M4-Phase
/// 2 T3 carry-forward CF-2).
const DISABLED_BUTTON_OCCLUDES_AND_STILL_LEAVES_UI: &str = r#"component DisabledButtonOccludesAndStillLeaves inherits Window {
    state gate_enabled: bool = false
    VStack {
        spacing: 0
        padding: 0
        Button { text: "trigger" }
        Button { text: "gate" enabled: gate_enabled }
        Button { text: "flip" clicked => { root.gate_enabled = true; } }
    }
}"#;

/// `docs/dsl_spec.md` §4.8: "Hover / press visual transitions are frozen"
/// for a disabled Button — it is still the resolved target
/// (DD-M4-P2-002's occlusion), but paints nothing and retains nothing.
/// The difference leg's third assertion is the one that matters most: the
/// **previously hovered** `trigger` Button must still leave, even though
/// the new target painted nothing — showing `update_hover`'s leave half
/// runs unconditionally of what the enter half paints. The agreement leg
/// (the same rectangle, later enabled via a `clicked` handler) then shows
/// hovering does work there, distinguishing "disabled is skipped" from
/// "that position is never hovered".
#[test]
fn a_disabled_button_paints_nothing_but_still_occludes_and_the_previously_hovered_button_still_leaves(
) {
    run_on_owning_runtime_thread_or_skip("F3: disabled Button occludes, still leaves", move || {
        let ir = lower_ui_to_ir(DISABLED_BUTTON_OCCLUDES_AND_STILL_LEAVES_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F3 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (trigger_rect, gate_rect, flip_rect) = {
                let root = (*window).root_widget.as_ref().expect("content root");
                (
                    root.children[0].__arranged_rect_for_test(),
                    root.children[1].__arranged_rect_for_test(),
                    root.children[2].__arranged_rect_for_test(),
                )
            };
            let trigger_rect = trigger_rect.expect("trigger Button must have been laid out");
            let gate_rect = gate_rect.expect("gate Button must have been laid out");
            let flip_rect = flip_rect.expect("flip Button must have been laid out");
            for (label, rect) in [
                ("trigger", trigger_rect),
                ("gate", gate_rect),
                ("flip", flip_rect),
            ] {
                assert!(
                    rect.width > 0.0 && rect.height > 0.0,
                    "fixture stopped discriminating: the {label} Button rectangle {rect:?} \
                     must be non-degenerate"
                );
            }
            assert!(
                trigger_rect.y + trigger_rect.height <= gate_rect.y,
                "fixture stopped discriminating: trigger {trigger_rect:?} and gate \
                 {gate_rect:?} must be disjoint, or a pointer meant for one could resolve to \
                 the other"
            );
            assert!(
                gate_rect.y + gate_rect.height <= flip_rect.y,
                "fixture stopped discriminating: gate {gate_rect:?} and flip {flip_rect:?} \
                 must be disjoint"
            );
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[1].__button_enabled_for_test(),
                    Some(false),
                    "fixture stopped discriminating: the gate Button must actually be \
                     disabled, or the difference leg below proves nothing. Bound to \
                     `gate_enabled` rather than authored as the literal `enabled: false` \
                     because a literal on a plain Button is silently dropped by the IR loader \
                     as of this task (M4-Phase 2 T3 carry-forward CF-2, owned by T8): the \
                     'Button' arm of `ir_loader::construct_widget` builds the widget from \
                     `label` / `style` only and never reads an `enabled` prop, unlike its \
                     'ToggleButton' sibling arm. A `state`-bound `enabled` reaches \
                     `PROP_BUTTON_ENABLED` through the binding path (`build_node`'s \
                     `node.bindings` loop) instead, which does land"
                );
            }

            let (trigger_cx, trigger_cy) = (
                (trigger_rect.x + trigger_rect.width / 2.0) * factor,
                (trigger_rect.y + trigger_rect.height / 2.0) * factor,
            );
            let (gate_cx, gate_cy) = (
                (gate_rect.x + gate_rect.width / 2.0) * factor,
                (gate_rect.y + gate_rect.height / 2.0) * factor,
            );
            let (flip_cx, flip_cy) = (
                (flip_rect.x + flip_rect.width / 2.0) * factor,
                (flip_rect.y + flip_rect.height / 2.0) * factor,
            );

            // Step 1: hover the enabled trigger Button.
            send_mouse_move(hwnd, trigger_cx, trigger_cy);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(root.children[0].__button_state_for_test(), Some("hovered"));
            }
            assert_eq!(ffi::__hover_target_for_test(window), Some(vec![0]));

            // Step 2 (difference leg): move onto the disabled gate Button.
            send_mouse_move(hwnd, gate_cx, gate_cy);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[1].__button_state_for_test(),
                    Some("normal"),
                    "docs/dsl_spec.md §4.8: a disabled Button must paint nothing on entry"
                );
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("normal"),
                    "the trigger Button must still leave (paint Normal) even though the new \
                     target painted nothing"
                );
            }
            assert_eq!(
                ffi::__hover_target_for_test(window),
                None,
                "a disabled Button is resolved and occludes, but nothing is retained for it \
                 (widget.rs `update_hover`: 'the paint target: resolved only when it names an \
                 enabled Button-family node')"
            );

            // Step 3: click flip to enable the gate Button. The inline handler's state
            // write drains synchronously (event_routing_integration.rs's module header
            // documents the mechanism), so the bound `enabled` property is already
            // updated before SendMessageW returns.
            send_mouse_up(hwnd, flip_cx, flip_cy);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[1].__button_enabled_for_test(),
                    Some(true),
                    "the flip Button's handler must have flipped gate_enabled synchronously"
                );
                // Documented side effect, not the property under test: WM_LBUTTONUP's
                // `update_hover` runs at the same point right after `hit_test_click`
                // (window.rs), so releasing on flip also hovers flip itself.
                assert_eq!(
                    root.children[2].__button_state_for_test(),
                    Some("hovered"),
                    "release-still-inside also hovers the flip Button itself (same rule \
                     Fixture 1 pins) — recorded so the next step's reset is not a surprise"
                );
            }

            // Step 4: move to a point outside every widget, to reset the transient hover
            // step 3's release left before re-approaching gate.
            let (outside_x, outside_y) = {
                let root = (*window).root_widget.as_ref().unwrap();
                let flip_rect = root.children[2]
                    .__arranged_rect_for_test()
                    .expect("flip Button must still be laid out");
                assert!(
                    flip_rect.y + flip_rect.height < CLIENT_H as f32,
                    "fixture stopped discriminating: no room below flip {flip_rect:?} inside \
                     the {CLIENT_H} DIP client"
                );
                (
                    flip_rect.x + flip_rect.width / 2.0,
                    (flip_rect.y + flip_rect.height + CLIENT_H as f32) / 2.0,
                )
            };
            send_mouse_move(hwnd, outside_x * factor, outside_y * factor);
            assert_eq!(ffi::__hover_target_for_test(window), None);

            // Agreement leg: the same structural position, now enabled, does hover.
            // Re-read the gate rectangle immediately before moving onto it (rule: never
            // reuse a rectangle cached across a state write), even though
            // PROP_BUTTON_ENABLED is not size-affecting and it should not have moved.
            let gate_rect = {
                let root = (*window).root_widget.as_ref().unwrap();
                root.children[1]
                    .__arranged_rect_for_test()
                    .expect("gate Button must still be laid out")
            };
            let (gate_cx, gate_cy) = (
                (gate_rect.x + gate_rect.width / 2.0) * factor,
                (gate_rect.y + gate_rect.height / 2.0) * factor,
            );
            send_mouse_move(hwnd, gate_cx, gate_cy);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[1].__button_state_for_test(),
                    Some("hovered"),
                    "agreement leg: the same rectangle, now enabled, must hover — otherwise \
                     step 2's 'paints nothing' could equally mean this position is never \
                     hovered at all"
                );
            }
            assert_eq!(ffi::__hover_target_for_test(window), Some(vec![1]));

            ffi::wasamo_window_destroy(window);
        }
    });
}

// ── F4 — WM_MOUSELEAVE clears through the same path ─────────────────────────

const MOUSELEAVE_CLEARS_UI: &str = r#"component MouseLeaveClears inherits Window {
    VStack {
        spacing: 0
        padding: 0
        Button { text: "target" }
    }
}"#;

/// `clear_hover`'s two arms, fired directly. The first `WM_MOUSELEAVE`
/// fires the `hover.target == Some(_)` arm; the second, sent with nothing
/// retained, fires the `hover.target == None` arm and must be a
/// side-effect-free no-op (no panic, no change) rather than merely
/// unreachable code.
#[test]
fn wm_mouseleave_clears_the_retained_hover_target_and_a_second_leave_is_a_no_op() {
    run_on_owning_runtime_thread_or_skip("F4: WM_MOUSELEAVE clears", move || {
        let ir = lower_ui_to_ir(MOUSELEAVE_CLEARS_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F4 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let button_rect = {
                let root = (*window).root_widget.as_ref().expect("content root");
                root.children[0].__arranged_rect_for_test()
            }
            .expect("Button must have been laid out");
            assert!(
                button_rect.width > 0.0 && button_rect.height > 0.0,
                "fixture stopped discriminating: the Button rectangle {button_rect:?} must be \
                 non-degenerate"
            );

            let (cx, cy) = (
                (button_rect.x + button_rect.width / 2.0) * factor,
                (button_rect.y + button_rect.height / 2.0) * factor,
            );
            send_mouse_move(hwnd, cx, cy);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(root.children[0].__button_state_for_test(), Some("hovered"));
            }
            assert_eq!(ffi::__hover_target_for_test(window), Some(vec![0]));

            // First WM_MOUSELEAVE: fires `clear_hover`'s `hover.target == Some(_)` arm.
            send_mouse_leave(hwnd);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("normal"),
                    "docs/architecture.md §13.2: leaving the client area must reset the \
                     entered Button to Normal"
                );
            }
            assert_eq!(ffi::__hover_target_for_test(window), None);

            // Second WM_MOUSELEAVE with nothing retained: fires `clear_hover`'s
            // `hover.target == None` arm directly. Must be a no-op.
            send_mouse_leave(hwnd);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("normal"),
                    "a second WM_MOUSELEAVE with nothing retained must not panic or change \
                     anything"
                );
            }
            assert_eq!(ffi::__hover_target_for_test(window), None);

            ffi::wasamo_window_destroy(window);
        }
    });
}

// ── F5 — a non-Button target paints nothing and still causes a leave ───────

const NON_BUTTON_TARGET_STILL_CAUSES_LEAVE_UI: &str = r#"component NonButtonTargetStillCausesLeave inherits Window {
    ZStack {
        Button { text: "under" }
        Box {
            slot.h-align: stretch
            slot.v-align: end
            aspect: 3:1
            fill: #33669988
        }
    }
}"#;

/// A plain `Box` (later in document order — painted on top, per
/// `docs/dsl_spec.md` §4.13) partially covers a Button: the Box stretches
/// to the full window width and is anchored to the bottom, so it covers
/// roughly the lower half of the vertically-centred Button and leaves the
/// upper half reachable. This is the shape the gallery's lightbox scrim
/// has — a later `ZStack` child (the scrim) painting over earlier content
/// ([log.md §T4 fact 2](../../process/milestone-4/phase-2/implementation/log.md#t4--hover-and-pressed-behind-the-routing-model)
/// measured the pre-T4 defect in exactly that gallery shape: the toolbar
/// hovered through the open lightbox's scrim).
///
/// `Box` has no `ButtonState` at all (`__button_state_for_test` returns
/// `None` for it), so this fixture's only read-back for the Box itself is
/// that hovering it retains nothing; what it actually asserts is that the
/// Button underneath does not hover **at a point both rectangles contain**.
/// The paired legs are the covered point (Button stays `"normal"`, nothing
/// retained) and the uncovered point (Button hovers) — the same rectangle,
/// two points, so the fixture separates "the topmost target owns hover"
/// from "this Button never hovers".
#[test]
fn hovering_a_non_button_target_paints_nothing_and_still_leaves_the_button_it_partly_covers() {
    run_on_owning_runtime_thread_or_skip("F5: non-Button target still causes a leave", move || {
        let ir = lower_ui_to_ir(NON_BUTTON_TARGET_STILL_CAUSES_LEAVE_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F5 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

            let (button_rect, box_rect) = {
                let root = (*window).root_widget.as_ref().expect("content root");
                (
                    root.children[0].__arranged_rect_for_test(),
                    root.children[1].__arranged_rect_for_test(),
                )
            };
            let button_rect = button_rect.expect("Button must have been laid out");
            let box_rect = box_rect.expect("Box must have been laid out");
            assert!(
                button_rect.width > 0.0 && button_rect.height > 0.0,
                "fixture stopped discriminating: the Button rectangle {button_rect:?} must be \
                 non-degenerate"
            );
            assert!(
                box_rect.width > 0.0 && box_rect.height > 0.0,
                "fixture stopped discriminating: the Box rectangle {box_rect:?} must be \
                 non-degenerate"
            );
            assert!(
                button_rect.y < box_rect.y,
                "geometry this fixture depends on: the Button, centred by ZStack's default \
                 alignment, must extend above the Box's top edge {box_rect:?} — this is the \
                 uncovered part of the Button the fixture hovers first"
            );
            assert!(
                button_rect.y + button_rect.height > box_rect.y,
                "geometry this fixture depends on: the Button must also extend into the \
                 Box's rectangle {box_rect:?} — this is what 'a plain Box that covers it' \
                 means"
            );

            // A point over the uncovered part of the Button: above the Box's top edge
            // (outside the Box by construction) and below the Button's own top edge (the
            // overlap fact just asserted).
            let (button_only_x, button_only_y) = (
                button_rect.x + button_rect.width / 2.0,
                (button_rect.y + box_rect.y) / 2.0,
            );
            assert!(
                dip_contains(button_rect, button_only_x, button_only_y)
                    && !dip_contains(box_rect, button_only_x, button_only_y),
                "fixture stopped discriminating: ({button_only_x}, {button_only_y}) must be \
                 inside the Button {button_rect:?} and outside the Box {box_rect:?}"
            );

            // The discriminating point: inside the Box **and** inside the Button — the
            // Button's covered lower part. A point inside the Box but outside the Button
            // would prove nothing here, because the pre-T4 whole-tree walk would also
            // have left the Button there simply for being outside its rectangle. Only a
            // point both rectangles contain separates "the topmost target owns hover"
            // from "the pointer is off the Button".
            let (covered_x, covered_y) = (
                button_rect.x + button_rect.width / 2.0,
                (box_rect.y + button_rect.y + button_rect.height) / 2.0,
            );
            assert!(
                dip_contains(box_rect, covered_x, covered_y)
                    && dip_contains(button_rect, covered_x, covered_y),
                "fixture stopped discriminating: ({covered_x}, {covered_y}) must be inside \
                 BOTH the Box {box_rect:?} and the Button {button_rect:?} — a point outside \
                 the Button would make the leave below trivially true under the pre-T4 walk \
                 as well"
            );

            // Hover the uncovered part of the Button first.
            send_mouse_move(hwnd, button_only_x * factor, button_only_y * factor);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(root.children[0].__button_state_for_test(), Some("hovered"));
            }
            assert_eq!(ffi::__hover_target_for_test(window), Some(vec![0]));

            // Move to the covered point, which both rectangles contain.
            send_mouse_move(hwnd, covered_x * factor, covered_y * factor);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children[0].__button_state_for_test(),
                    Some("normal"),
                    "docs/architecture.md §13.2 / widget.rs `update_hover`: the Box is the \
                     topmost widget containing this point, and a non-Button resolved target \
                     is never an 'enabled Button-family node', so the paint target is None — \
                     the Button underneath must NOT hover even though its own rectangle also \
                     contains the point (the pre-T4 whole-tree walk hovered it, which is the \
                     gallery's hover-through-the-scrim defect)"
                );
            }
            assert_eq!(
                ffi::__hover_target_for_test(window),
                None,
                "nothing is retained for a non-Button resolved target"
            );

            ffi::wasamo_window_destroy(window);
        }
    });
}
