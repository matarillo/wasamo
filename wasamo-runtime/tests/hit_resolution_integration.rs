//! Mock-free Windows integration evidence for M4-Phase 2 T2
//! ([DD-M4-P2-002](../../process/milestone-4/phase-2/decisions/dd-m4-p2-002-hit-testing-and-generic-click.md)),
//! covering the two fixtures the task plan names beyond `hit.rs`'s own
//! pure-logic tests: **staleness control** (a click after a property write
//! triggers re-layout — the path where a cache goes stale) and
//! **non-unit-scale resolution** (a click at physical coordinates resolves
//! via the pointer -> DIP conversion, not by coincidence at 100%).
//!
//! Real windows, real `WM_DPICHANGED` / `WM_LBUTTONUP` messages through the
//! real window procedure, real widget state read back. Nothing is mocked.
//!
//! # What each test is for
//!
//! - `a_click_after_a_relayout_triggering_state_write_resolves_at_the_new_rectangle`
//!   — DD-002's named mitigation for T1's retained-rectangle store: a
//!   `clicked` handler writes state, a conditional branch materialises, the
//!   window re-lays-out, and a click must resolve against the **refreshed**
//!   `arranged_rect`, not whatever the target's rectangle used to be.
//! - `a_click_at_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_the_converted_point`
//!   — the conversion Phase 1 measured as cancelling at 100% and no longer
//!   cancelling once T2 switched hit-testing's geometry source
//!   ([preamble.md §What "green" is worth](../../process/milestone-4/phase-2/implementation/preamble.md)):
//!   a wrong pointer->DIP conversion is invisible at scale 1 and visible at
//!   scale 2.
//!
//! # The before-state is established, never inherited (Phase 1 T9 finding F-47)
//!
//! Both fixtures normalise to [`REFERENCE_DPI`] with an explicitly chosen
//! physical client before measuring anything, exactly as
//! `dpi_scale_matrix_integration.rs` does and for the same reason: a
//! constant expressed against whatever DPI the developer's monitor happens
//! to report would make every assertion below conditional on a machine
//! nobody chose. See that file's module header for the full argument; it is
//! not restated here.
//!
//! # The client stays small (M4-Phase 1 T8 finding, corrected further at T9)
//!
//! A hosted CI desktop failed to honour a 1440x960 physical request twice.
//! `dpi_scale_matrix_integration.rs` narrowed to 360x240 baseline / 720x480
//! at 200%; this file narrows further still. Fixture 1 never changes scale,
//! so its physical client is its DIP client: [`FIXTURE1_CLIENT_W`] x
//! [`FIXTURE1_CLIENT_H`] = 360x240. Fixture 2's largest physical request is
//! its post-change one, [`FIXTURE2_DOUBLED_W`] x [`FIXTURE2_DOUBLED_H`] =
//! 480x320 — at or below the stated 480x320 ceiling.
//!
//! # A measured finding: a synthesised `WM_LBUTTONUP` alone does not re-layout
//!
//! Fixture 1 needs a click to trigger a property write, a structural
//! rebuild (the conditional branch materialising) and a **layout pass**
//! that rewrites `arranged_rect`, all before the discriminating clicks that
//! follow. Measured, not assumed: sending the toggle click with plain
//! `SendMessageW` (the pattern every other click in this file and in
//! `dpi_scale_matrix_integration.rs` uses) reaches `wnd_proc`,
//! `hit_test_click` runs the inline handler, and `Signal::set`'s
//! zero-batch-depth drain runs the reactive Effects synchronously — so the
//! conditional's structural rebuild (the new child inserted) is visible the
//! instant `SendMessageW` returns. But the **layout** phase of the
//! three-phase drain (`emit::flush_layout`, which is what actually calls
//! `sync_visuals` and rewrites `arranged_rect`) is a separate mechanism,
//! gated behind `emit::drain_if_outermost`, which `wnd_proc`'s
//! `WM_LBUTTONUP` arm never calls. In production that call happens at
//! `wasamo_run`'s message-loop boundary — the line immediately after every
//! `DispatchMessageW` — not inside `wnd_proc` itself. A window whose click
//! is delivered by a direct `SendMessageW` therefore never crosses that
//! boundary and never gets re-laid-out: the new child materialises with no
//! rectangle and every existing sibling keeps its pre-click one. This was
//! measured directly (a throwaway probe, discarded once understood): after
//! a plain `SendMessageW`'d toggle click, the target Button's rectangle was
//! bit-for-bit identical to its pre-click value even though the tree had
//! grown a third child. [`click_and_drain`] below instead **posts** the
//! click and pumps the real `wasamo_run` loop once (with a synthesised
//! `WM_QUIT` right behind it, so the loop returns instead of blocking) —
//! genuine production code, not a test seam, and the only way available to
//! this file to cross that boundary from outside `wasamo_run`'s own
//! long-running loop.
//!
//! # Fixture 2 cannot fail at the reference DPI (stated per
//! [preamble.md §What "green" is worth](../../process/milestone-4/phase-2/implementation/preamble.md))
//!
//! At scale 1 physical and DIP coordinates are the same numbers, so a click
//! at "the DIP centre, unconverted" and "the DIP centre times the factor"
//! land on the identical pixel — the positive and negative legs would
//! collapse into one assertion that a missing pointer->DIP conversion could
//! not distinguish from a correct one. Fixture 2 exists specifically to run
//! at scale 2, where the two points are 2x apart and a missing conversion
//! resolves to the wrong widget.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;

use wasamo_runtime::{ffi, DipRect};

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, PostMessageW, PostQuitMessage, SendMessageW,
    SM_CXMAXTRACK, SM_CXSCREEN, SM_CYMAXTRACK, SM_CYSCREEN, WM_DPICHANGED, WM_LBUTTONUP,
};

/// The reference DPI every fixture normalises to before measuring, and the
/// denominator of every scale factor in the runtime. See
/// `dpi_scale_matrix_integration.rs`'s module header for why this is
/// reached deliberately (Phase 1 T9 finding F-47) rather than assumed.
const REFERENCE_DPI: u32 = 96;

/// Fixture 1's physical client. No scale change happens in this fixture, so
/// this is also its only client extent — well under the 480x320 ceiling
/// (M4-Phase 1 T8 finding, restated in this file's module header).
const FIXTURE1_CLIENT_W: i32 = 360;
const FIXTURE1_CLIENT_H: i32 = 240;

/// Fixture 2's baseline (96 DPI) physical client, and its doubled
/// (192 DPI) one. `96 -> 192` is a power-of-two factor, so `doubled / 2 ==
/// baseline` exactly and the DIP client is unchanged across the change —
/// the property the fixture's "horizontally disjoint" and "factor == 2"
/// assertions depend on.
const FIXTURE2_BASELINE_W: i32 = 240;

/// **Derived, not chosen freely** — see [`NON_UNIT_SCALE_UI`]'s doc comment
/// for why the `HStack`'s children are always cross-axis centered. Centering
/// puts each Button's DIP `y` offset at `(FIXTURE2_BASELINE_H - button_h) /
/// 2`. The negative leg below reuses that DIP `y` as an *unconverted*
/// physical `y`, and needs it to still land inside the correct Button's
/// **physical** (`factor`-scaled) row rather than in the empty space above
/// both Buttons: with `physical_top = 2 * offset` and
/// `raw_point = offset + button_h / 2`, that requires
/// `offset <= button_h / 2`, i.e. `FIXTURE2_BASELINE_H <= 2 * button_h`.
/// Measured (a throwaway probe): this fixture's Button is `34.62` DIP tall,
/// so the bound is `FIXTURE2_BASELINE_H <= ~69.2`. `48` keeps a comfortable
/// margin (`offset` ≈ `6.7` against a `17.3` ceiling) rather than sitting on
/// the boundary.
const FIXTURE2_BASELINE_H: i32 = 48;
const FIXTURE2_DOUBLED_W: i32 = 480;
const FIXTURE2_DOUBLED_H: i32 = 96;
const FIXTURE2_SCALED_DPI: u32 = 192;

/// Fixture 1 — DD-002's staleness control. `expanded` starts `false`
/// (`toggle`, `target` — two children); the toggle's inline handler flips it
/// to `true`, materialising the `Box` between them (`toggle`, `Box`,
/// `target` — three children) and pushing `target` down by the `Box`'s
/// height.
const HIT_STALENESS_UI: &str = r#"component HitStaleness inherits Window {
    state expanded: bool = false
    VStack {
        spacing: 0
        padding: 0
        Button { text: "toggle" clicked => { root.expanded = true; } }
        if expanded {
            Box { aspect: 4:1 fill: #336699cc }
        }
        Button { text: "target" }
    }
}"#;

/// Fixture 2 — non-unit-scale resolution. Two horizontally adjacent Buttons,
/// each independently identifiable by its own `set_clicked` counter.
///
/// **The `HStack` is the window root**, so `run_layout_as_window_root`
/// forces it to `Fill` on both axes (M4-Phase 1 T3 finding F-23). `HStack`'s
/// cross-axis (height) also defaults to `Fill` when it is *not* the root
/// (`WidgetKind::HStack`'s constructor default in `layout.rs` — measured by
/// wrapping it in an outer `VStack` and observing the Buttons' `y` unchanged,
/// a throwaway probe discarded once understood), and the DSL has no authored
/// way to override a container's default size constraint or to top-align an
/// `HStack`'s children (`slot.h-align` / `slot.v-align` are `Grid` / `ZStack`
/// placement keys only — [dsl_spec.md §4.11](../../../docs/dsl_spec.md)).
/// So in every shape this fixture could take, both Buttons end up
/// cross-axis **centered** in whatever height they are given, and
/// [`FIXTURE2_BASELINE_H`]'s size is chosen small enough that this doesn't
/// defeat the negative leg's discriminating-construction check below (see
/// that constant's doc comment for the bound).
const NON_UNIT_SCALE_UI: &str = r#"component NonUnitScaleResolution inherits Window {
    HStack {
        spacing: 0
        padding: 0
        Button { text: "left" }
        Button { text: "right" }
    }
}"#;

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<hit-resolution-integration>";
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
/// rather than derived from a DPI table — see
/// `dpi_scale_matrix_integration.rs::frame_thickness`, copied here
/// unchanged.
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
/// position. Copied unchanged from `dpi_scale_matrix_integration.rs`.
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
/// assumed, for the reasons `dpi_scale_matrix_integration.rs`'s module
/// header states in full.
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
/// synchronously via `SendMessageW` — the same pattern
/// `dpi_scale_matrix_integration.rs::send_click` uses. Suitable for every
/// click in this file that only needs `hit_test_click`'s synchronous
/// dispatch (a `set_clicked` host callback firing), not a structural
/// rebuild or a re-layout.
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
/// — see this file's module header ("A measured finding") for why fixture 1
/// needs this rather than [`send_click`]. Posts the click, then a
/// synthesised `WM_QUIT`, then pumps `wasamo_runtime::run()` — the same
/// function `wasamo_run` (abi_spec) wraps — until it returns.
///
/// `PostQuitMessage`'s quit flag is only consulted once the thread's regular
/// message queue is empty (documented Win32 ordering), so the posted click
/// is always retrieved, dispatched through `wnd_proc`, and drained by
/// `run`'s post-`DispatchMessageW` call **before** the loop sees the quit
/// and returns. Verified directly against this measured ordering with a
/// throwaway probe before this helper was written for real (see the module
/// header).
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

/// Does `rect` contain `(x, y)`, under the same half-open convention
/// `hit::resolve_topmost` uses (left/top inclusive, right/bottom
/// exclusive — see `wasamo-runtime/src/hit.rs::contains`)?
///
/// `hit` is a private module, so this is a local copy for constructing this
/// file's own discriminating assertions ("is this probe point actually
/// inside / outside the rectangle I claim it is"), not a second test of the
/// production predicate — `hit.rs`'s own unit tests own that, including the
/// DD-V-029 boundary-condition obligation.
fn contains(rect: DipRect, x: f32, y: f32) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// `rect`, scaled by `factor` on every field — DIP -> physical at a window
/// whose scale is `factor`.
fn scale_rect(rect: DipRect, factor: f32) -> DipRect {
    DipRect {
        x: rect.x * factor,
        y: rect.y * factor,
        width: rect.width * factor,
        height: rect.height * factor,
    }
}

/// **Staleness control** (DD-002's named mitigation). A click resolves
/// correctly *after* a property write triggers re-layout — the path where a
/// cache goes stale.
///
/// The two discriminating legs are the point: a click at the target's *new*
/// rectangle centre must fire (the store was refreshed), and a click at its
/// *old* rectangle centre must not (a stale cache would still have fired
/// there). Both legs share one `set_clicked` counter installed on the
/// target **before** the toggle click, which stays valid across the
/// conditional's structural rebuild — `insert_structural_child` is a plain
/// `Vec::insert` on the parent's children (M4-Phase 2 `ir_loader.rs`
/// `mutate_conditional_subtree`), so the target's own `WidgetNode` is never
/// rebuilt, only shifted from index 1 to index 2; its installed closure
/// survives the shift.
#[test]
fn a_click_after_a_relayout_triggering_state_write_resolves_at_the_new_rectangle() {
    run_on_owning_runtime_thread_or_skip("hit staleness control", move || {
        let ir = lower_ui_to_ir(HIT_STALENESS_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(
                window,
                FIXTURE1_CLIENT_W,
                FIXTURE1_CLIENT_H,
                "hit staleness pre-change baseline",
            );

            let clicks = Rc::new(Cell::new(0u32));
            let initial_children;
            let before_rect;
            let toggle_rect;
            {
                let root = (*window).root_widget.as_mut().expect("content root");
                initial_children = root.children.len();
                let counter = Rc::clone(&clicks);
                root.children[1].set_clicked(move || counter.set(counter.get() + 1));
                before_rect = root.children[1].__arranged_rect_for_test();
                toggle_rect = root.children[0].__arranged_rect_for_test();
            }

            // Drive the state write through the production path: a real
            // WM_LBUTTONUP at the toggle Button's rectangle centre. At the
            // 96 DPI baseline the scale factor is 1, so physical and DIP
            // coincide -- but the factor is read back from the scale the
            // runtime actually committed rather than written as the
            // constant ratio `REFERENCE_DPI / REFERENCE_DPI`, which would
            // be 1 by construction and would therefore say nothing about
            // the window this fixture is running against.
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
            let (toggle_cx, toggle_cy) = toggle_rect
                .map(|r| {
                    (
                        (r.x + r.width / 2.0) * factor,
                        (r.y + r.height / 2.0) * factor,
                    )
                })
                .unwrap_or((-1.0, -1.0));
            click_and_drain(hwnd, toggle_cx, toggle_cy);

            let materialised_children;
            let after_rect;
            {
                let root = (*window).root_widget.as_ref().expect("content root");
                materialised_children = root.children.len();
                after_rect = root
                    .children
                    .get(2)
                    .and_then(|c| c.__arranged_rect_for_test());
            }

            // Two discriminating legs, both real WM_LBUTTONUPs through
            // wnd_proc at physical coordinates (== DIP at this baseline).
            let before_rect_for_click = before_rect.unwrap_or(DipRect {
                x: -1.0,
                y: -1.0,
                width: 0.0,
                height: 0.0,
            });
            let after_rect_for_click = after_rect.unwrap_or(DipRect {
                x: -1.0,
                y: -1.0,
                width: 0.0,
                height: 0.0,
            });
            let old_cx = (before_rect_for_click.x + before_rect_for_click.width / 2.0) * factor;
            let old_cy = (before_rect_for_click.y + before_rect_for_click.height / 2.0) * factor;
            let new_cx = (after_rect_for_click.x + after_rect_for_click.width / 2.0) * factor;
            let new_cy = (after_rect_for_click.y + after_rect_for_click.height / 2.0) * factor;

            let before_new_leg = clicks.get();
            send_click(hwnd, new_cx, new_cy);
            let after_new_leg = clicks.get();

            send_click(hwnd, old_cx, old_cy);
            let after_old_leg = clicks.get();

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                initial_children, 2,
                "expanded starts false: only toggle and target must be present before the click"
            );
            assert_eq!(
                materialised_children, 3,
                "the conditional Box must have materialised after the state write, or the \
                 rest of this fixture is vacuous"
            );
            let before_rect = before_rect.expect("target must have been laid out initially");
            let after_rect =
                after_rect.expect("target must have been re-laid-out after the state write");
            assert_ne!(
                after_rect.y, before_rect.y,
                "the property write must have moved the target Button (the Box now sits above \
                 it), or the two legs below cannot discriminate a refreshed rectangle store \
                 from a stale one"
            );
            assert!(
                !contains(after_rect, old_cx, old_cy),
                "fixture stopped discriminating: the old rectangle's centre ({old_cx}, {old_cy}) \
                 must fall outside the new rectangle {after_rect:?}, or a stale-cache read and a \
                 fresh one would resolve the same widget by coincidence rather than by the store \
                 actually being refreshed"
            );
            assert_eq!(
                after_new_leg - before_new_leg,
                1,
                "a click at the target's new (post-relayout) rectangle centre must resolve to \
                 it exactly once: the store was refreshed"
            );
            assert_eq!(
                after_old_leg - after_new_leg,
                0,
                "a click at the target's old (pre-relayout) rectangle centre must not resolve \
                 to it: a stale cache would still fire here"
            );
        }
    });
}

/// **Non-unit-scale resolution.** A click at physical coordinates resolves
/// to the widget whose DIP rectangle contains the *converted* point, at a
/// scale where the conversion is load-bearing (see this file's module
/// header, "Fixture 2 cannot fail at the reference DPI").
///
/// The positive leg multiplies the second Button's DIP centre by the
/// factor before clicking — the correct conversion. The negative leg clicks
/// the *unconverted* DIP centre, i.e. what a pointer-handling path that
/// forgot the DIP conversion would effectively compare against
/// `arranged_rect`; because the factor is 2, that point is constructed to
/// land inside the *first* Button's physical rectangle instead, so a
/// missing conversion is observable as resolving to the wrong widget rather
/// than as no resolution at all.
#[test]
fn a_click_at_non_unit_scale_resolves_the_widget_whose_dip_rectangle_contains_the_converted_point()
{
    run_on_owning_runtime_thread_or_skip("non-unit-scale hit resolution", move || {
        let ir = lower_ui_to_ir(NON_UNIT_SCALE_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(
                window,
                FIXTURE2_BASELINE_W,
                FIXTURE2_BASELINE_H,
                "non-unit-scale pre-change baseline",
            );

            send_dpi_change_to_client(
                hwnd,
                FIXTURE2_SCALED_DPI,
                FIXTURE2_DOUBLED_W,
                FIXTURE2_DOUBLED_H,
            );
            let realised_client = client_extent(hwnd);
            let committed_scale = ffi::__window_scale_dpi_for_test(window);
            let limits = display_limits();

            let clicks_left = Rc::new(Cell::new(0u32));
            let clicks_right = Rc::new(Cell::new(0u32));
            let left_rect;
            let right_rect;
            {
                let root = (*window).root_widget.as_mut().expect("content root");
                let c_left = Rc::clone(&clicks_left);
                root.children[0].set_clicked(move || c_left.set(c_left.get() + 1));
                let c_right = Rc::clone(&clicks_right);
                root.children[1].set_clicked(move || c_right.set(c_right.get() + 1));
                left_rect = root.children[0].__arranged_rect_for_test();
                right_rect = root.children[1].__arranged_rect_for_test();
            }
            let fallback = DipRect {
                x: -1.0,
                y: -1.0,
                width: 0.0,
                height: 0.0,
            };
            let right_rect_for_click = right_rect.unwrap_or(fallback);

            // Derived from the scale the runtime **committed**, not from
            // `FIXTURE2_SCALED_DPI`: written as the constant ratio the
            // `assert_ne!(factor, 1.0)` below would be 2 by construction and
            // could never fire, which is the tautological-assertion shape
            // M4-Phase 2 T1's retrospective flagged. Read back, it fails
            // when the change did not commit — the state in which every
            // conversion claim below would be about a scale nobody reached.
            let factor = committed_scale as f32 / REFERENCE_DPI as f32;
            let right_dip_cx = right_rect_for_click.x + right_rect_for_click.width / 2.0;
            let right_dip_cy = right_rect_for_click.y + right_rect_for_click.height / 2.0;

            // Positive leg: the converted point.
            let positive_x = right_dip_cx * factor;
            let positive_y = right_dip_cy * factor;
            let before_positive = (clicks_left.get(), clicks_right.get());
            send_click(hwnd, positive_x, positive_y);
            let after_positive = (clicks_left.get(), clicks_right.get());

            // Negative leg: the un-multiplied DIP centre, used as-is as a
            // physical coordinate -- what a missing pointer -> DIP
            // conversion would effectively compare.
            send_click(hwnd, right_dip_cx, right_dip_cy);
            let after_negative = (clicks_left.get(), clicks_right.get());

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                realised_client,
                (FIXTURE2_DOUBLED_W, FIXTURE2_DOUBLED_H),
                "the realised client extent must be the requested one, or every assertion \
                 below is about a rectangle nobody chose; \
                 requested_client=({FIXTURE2_DOUBLED_W},{FIXTURE2_DOUBLED_H}), \
                 display=(screen {}x{}, max_track {}x{})",
                limits.0,
                limits.1,
                limits.2,
                limits.3,
            );
            assert_eq!(
                committed_scale, FIXTURE2_SCALED_DPI,
                "the scale must be committed to the requested DPI"
            );
            assert_ne!(
                factor, 1.0,
                "the whole point of this fixture is a scale where the conversion is load-\
                 bearing; factor == 1 would make the two legs below identical"
            );
            let left_rect = left_rect.expect("left Button must have been laid out");
            let right_rect = right_rect.expect("right Button must have been laid out");
            assert!(
                left_rect.x + left_rect.width <= right_rect.x,
                "the two Buttons must be horizontally disjoint, with left strictly before \
                 right in document order: left={left_rect:?}, right={right_rect:?}"
            );
            let left_physical_rect = scale_rect(left_rect, factor);
            assert!(
                contains(left_physical_rect, right_dip_cx, right_dip_cy),
                "fixture stopped discriminating: the right Button's unconverted DIP centre \
                 ({right_dip_cx}, {right_dip_cy}) must fall inside the left Button's physical \
                 rectangle {left_physical_rect:?}, or the negative leg below cannot show a \
                 missing conversion resolving to the wrong widget"
            );
            assert_eq!(
                after_positive.1 - before_positive.1,
                1,
                "positive leg: a click at the converted point must resolve to the right \
                 Button exactly once"
            );
            assert_eq!(
                after_positive.0 - before_positive.0,
                0,
                "positive leg: the left Button must not fire"
            );
            assert_eq!(
                after_negative.0 - after_positive.0,
                1,
                "negative leg: a click at the unconverted DIP centre must resolve to the left \
                 Button -- a missing pointer -> DIP conversion resolves to the wrong widget \
                 rather than to none at all"
            );
            assert_eq!(
                after_negative.1 - after_positive.1,
                0,
                "negative leg: the right Button must not fire a second time"
            );
        }
    });
}
