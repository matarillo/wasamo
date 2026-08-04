//! Mock-free Windows integration evidence for M4-Phase 1 T8 — the ADR set's
//! verification item (2), driven synthetically.
//!
//! Real windows, real `WM_DPICHANGED` and `WM_LBUTTONUP` messages through the
//! real window procedure, real Composition objects read back. Nothing is
//! mocked and nothing is stubbed.
//!
//! # What each test is for
//!
//! - `a_created_windows_cached_scale_is_the_dpi_the_os_reports` — the cache is
//!   the OS's number and the handler's `wParam` is what moves it.
//! - `dip_layout_is_invariant_while_every_visual_moves_by_the_ratio` — the
//!   positive control, over 125% / 150% / 200% **and one non-standard DPI**.
//! - `a_stale_descendant_scale_still_hit_tests_where_the_widget_is` — the
//!   one-divisor traversal property (T5 finding F-37).
//!
//! # Three stated limits, recorded rather than elided
//!
//! 1. **A synthesised `WM_DPICHANGED` proves the handling path only.** It
//!    never proves that crossing a real monitor boundary delivers the same
//!    message with a usable suggested rectangle. That half is T11's, and
//!    neither alone discharges AC7's third requirement.
//! 2. **The exact-invariance assertion holds because this file preserves the
//!    DIP client extent**, which is the input layout actually receives —
//!    and choosing the rectangle is only the first of three things that
//!    takes. The chosen *physical* client must give an **integer** physical
//!    target at every DPI in the matrix (see [`CLIENT_W`]) — which is not
//!    the same as the DIP extent being recoverable bit-for-bit, and at
//!    100 DPI it is not (see [`factor_is_exact`]); the realised value must
//!    be asserted rather than assumed (see `set_client_extent`); and the
//!    quantity asserted must be **sensitive to** that extent at the
//!    precision the claim needs, which the per-tile geometry is not below
//!    about a DIP and the root Visual is. The OS's own suggested rectangle preserves the
//!    **outer** rectangle instead, and the non-client frame scales by its own
//!    DPI-indexed metrics rather than by `s` — so on the real path the DIP
//!    layout input moves by a DIP or two and invariance is approximate. That
//!    is not a failure; it is a different rectangle.
//! 3. **The messages are still synthesised, so the OS is not what drives
//!    them.** T9 declared Per-Monitor-Aware V2, so `GetDpiForWindow` now
//!    reports the monitor's real DPI and the OS *would* deliver
//!    `WM_DPICHANGED` on a monitor crossing — but this file still sends its
//!    own, because it needs to choose the DPI and the rectangle. Driving four
//!    DPIs including one that is not a Windows scaling is not something a
//!    monitor can be asked to do.
//!
//! # The before-state is established, never inherited (T9 finding F-47)
//!
//! Every constant here is expressed against [`REFERENCE_DPI`], which was a
//! fact about the process until T9 declared awareness and the OS started
//! reporting the monitor's DPI instead of 96. On a 125% machine the created
//! window's 720 physical client became **576 DIP**, the row shape read
//! `(5, 3)` against this file's `(7, 2)`, and the invariance claim was about a
//! rectangle nobody had chosen.
//!
//! The fix is not to compute against whatever DPI the machine happens to
//! report — that would make every exactness argument below conditional on the
//! developer's display settings, and 100 DPI's `factor_is_exact` split would
//! stop meaning what it says. Instead the two tests that measure a scale change
//! **put the window into the before-state they assume**, with one synthesised
//! `WM_DPICHANGED` to [`REFERENCE_DPI`] carrying a rectangle that realises the
//! chosen physical client (see `normalise_to_reference_baseline`). A 96-DPI CI
//! runner and a 120-DPI laptop then run the same arithmetic. Test 1 does not
//! normalise and says why in its own doc comment.
//!
//! # Two stated limits about that baseline, both measured
//!
//! 1. **`normalise_to_reference_baseline`'s scale assertion is vacuous at
//!    96 DPI.** It asserts the committed scale is [`REFERENCE_DPI`] after the
//!    synthesised message — which on a 96-DPI display is the value the window
//!    was created with, so it passes whether or not the handler committed
//!    anything. Measured: with `begin_scale_change` neutered *and* the window
//!    created at 96 DPI, `a_stale_descendant_scale_still_hit_tests_where_the_widget_is`
//!    passes. On the 120-DPI development machine the same mutation fails all
//!    three tests. The normalisation makes the *arithmetic* machine-independent;
//!    it does not make every assertion about it equally sharp everywhere.
//! 2. **The creation-time seeding path has no coverage here on a 96-DPI
//!    runner** — see `a_created_windows_cached_scale_is_the_dpi_the_os_reports`,
//!    where the reason is that no reading distinguishes "seeded from the OS"
//!    from "seeded from the identity" when the monitor is at the reference DPI.
//!
//! What this file deliberately gives up either way is observing the
//! *creation-time* scale on a scaled monitor; that is test 1's job on a scaled
//! machine, and `dpi_awareness_declaration_integration.rs`'s for the level.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;

use wasamo_runtime::ffi;

use windows::core::Interface;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::HiDpi::{
    AreDpiAwarenessContextsEqual, GetDpiForWindow, GetWindowDpiAwarenessContext,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, SendMessageW, SetWindowPos, SM_CXMAXTRACK,
    SM_CXSCREEN, SM_CYMAXTRACK, SM_CYSCREEN, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER,
    WM_DPICHANGED, WM_LBUTTONUP,
};
use windows::UI::Composition::{CompositionDrawingSurface, CompositionSurfaceBrush};

/// The reference DPI: the denominator of every scale factor in the runtime,
/// and the baseline each test below puts its window into before measuring.
///
/// It was also the value an undeclared process was told unconditionally
/// (DD-M4-P1-001) — true of this process until T9 and no longer, which is why
/// it is now reached deliberately rather than assumed (see the module header).
const REFERENCE_DPI: u32 = 96;

/// The DPI matrix, and why it is not four equal probes.
///
/// 120 / 144 / 192 are 125% / 150% / 200%. At 200% the factor is a power of
/// two, so the multiplication is exact and convert-once and convert-twice
/// agree everywhere — it checks magnitude, not the rules (T2 finding F-13).
/// 125% and 150% carry the rule verification.
///
/// **100 is the one that is not a Windows scaling at all**, and it is here
/// because every standard scaling is a multiple of 24, hence has an exactly
/// representable `f32` factor — the property that hid a real arithmetic
/// defect through eleven green T4 tests until a reviewer asked what the
/// documented rule said (T4 independent review finding R-1). A suite that
/// only uses the inputs the product expects cannot find a rule that is wrong
/// outside them.
const DPI_MATRIX: [u32; 4] = [120, 144, 192, 100];

/// The physical client extent every scale-matrix window is normalised to
/// before the change, and the reason the invariance assertion can be an
/// equality at all (finding F-44).
///
/// Preserving the *DIP* client extent across a change to `dpi` means the
/// physical client must become `client × dpi / 96`, which has to be an
/// integer. The window `wasamo_load_ui` creates does not supply one: on a
/// 96-DPI display its client is 784 × 561 physical (T4, measured), and
/// 561 × 1.25 is 701.25 — so no synthesised rectangle preserves the DIP extent
/// from there, at any DPI in the matrix. Since T9 the created client is not
/// even that fixed number, because the window is realised at the monitor's DPI;
/// which is a second reason the extent is chosen here rather than inherited.
/// `96 = 2^5 × 3`, and the matrix contributes denominators 4, 2,
/// 1 and 24, so **a multiple of 24 makes all four targets integers at once**:
/// 360 → 375 / 450 / 540 / 720 and 240 → 250 / 300 / 360 / 480.
///
/// **"Integer target" is not "exact in `f32`", and the two are separate
/// claims** — the confusion T4's finding R-1 was about, arriving one level
/// out. An integer physical target is a fact about rational arithmetic and
/// holds at all four DPIs. Whether the DIP extent is then recoverable
/// bit-for-bit is a fact about `f32` and holds at three of them; see
/// [`factor_is_exact`].
///
/// **Two corrections, both measured rather than reasoned** (mutation M5).
///
/// 1. The consequence first predicted for a non-normalised extent — that the
///    per-tile geometry assertions could not be equalities — is **false of
///    this fixture**. WrapPanel tiles are start-packed at a fixed cross-size,
///    so they do not move when the client extent shifts by a fraction of a
///    DIP: at 785 × 480, where the recovered DIP width is 784.8 rather than
///    785, every per-tile assertion still passed. What the normalisation
///    actually protects is the **root** Visual, which *is* the client
///    rectangle, and the row-break count, which sits on a boundary a fraction
///    of a DIP can cross. The root readback below was added because of that
///    mutation, and with it the same 785 × 480 run fails at `981.0` against an
///    expected `981.25`.
/// 2. The extent must also **fit the monitor at 200%**. Measured: an
///    784 × 561 client asks for 1568 × 1122 at 192 DPI and the window realises
///    1568 × 1014 — the max track size, a harder failure than any rounding
///    one. The former 720 × 480 fixture asked for 1440 × 960, which made the
///    evidence depend on a desktop extent the test does not control. This
///    360 × 240 multiple-of-24 fixture asks for only 720 × 480 at 200% while
///    preserving the same row-count falsifier below. The realised extent is
///    still asserted and now reports the display / maximum-track metrics on
///    failure. **The margin narrowed at T9** and the constraint still holds:
///    the process is Per-Monitor-Aware V2, so the non-client frame the outer
///    rectangle has to carry is the monitor's rather than the 96-DPI one.
const CLIENT_W: i32 = 360;
const CLIENT_H: i32 = 240;

/// Tiles per row, predicted from the fixture's authored numbers and
/// [`CLIENT_W`]: `44 + 6` per tile in a 360 DIP line gives
/// `floor((360 + 6) / 50) = 7`. An implementation that treated the physical
/// client as logical would lay out into 450 DIP at 120 DPI and fit **9** — the
/// signature the plan records for §T10, and it is measured here rather than
/// predicted (mutation M1 with the root assertion shadowed reads `(9, 2)`).
///
/// **What this witness is, corrected at the T8 round-2 review (finding
/// MINOR 3).** It was introduced as the fact that makes the ADR's evidence
/// item (2) two claims rather than one (F-45), and described as "the half a
/// ratio assertion cannot make". **That is false of the landed fixture, and
/// the reason is arithmetic rather than incidental:** `row_shape` partitions
/// tiles by equal `Y`, and a partition by equality is invariant under
/// multiplication by a positive factor. So once `assert_scaled` has fixed
/// every tile at `before × factor` and `row_shape(before)` is pinned to this
/// constant, `row_shape(after)` **follows**; no state exists where the second
/// fires and the first two do not. The claim withdrawn is that it adds
/// discriminating power.
///
/// **What it does carry**, and why it stays:
///
/// - `row_shape(before)` against this constant is genuinely independent — it
///   checks the 96-DPI baseline is the fixture the `.ui` describes, which
///   nothing read off the post-change tree can establish. It says nothing
///   about the scale change.
/// - `row_shape(after)` is redundant against the conjunction above and is
///   kept for **legibility**: a failure names the phase's known 9-vs-7
///   signature instead of presenting a pile of `f32` mismatches. That is a
///   real property of an evidence artifact and it is not evidence.
///
/// F-45's *problem* stands — the two halves of evidence item (2) are one
/// equation at `s = 1`. What separates them is on the **input** side: the
/// realised physical client asserted against a target this file computed, and
/// the root Visual asserted against the constants above. See
/// `dip_layout_is_invariant_while_every_visual_moves_by_the_ratio`.
///
/// **The 9-vs-7 signature degenerates at 100 DPI** (finding MINOR 5). A
/// physical-as-logical implementation lays out into 375 DIP there, and
/// `floor((375 + 6) / 50)` is **7** — the same count a correct
/// implementation gives. The discrete witness discriminates at 120 / 144 /
/// 192 and is blind at 100; only the root and ratio assertions catch M1
/// there.
const TILES_PER_ROW: usize = 7;
const TILE_COUNT: usize = 12;
const ROWS: usize = 2;

/// A WrapPanel root, so the window-root `Fill` override gives it exactly the
/// client extent and the row arithmetic above is about a number this file
/// controls. Each tile carries a `Text`, which supplies the raster-side
/// witness — a different shape of fact from the geometry the ratio assertion
/// reads (T7 finding F-42).
const MATRIX_UI: &str = r#"
component ScaleMatrix inherits Window {
    title: "T8 scale matrix"

    WrapPanel {
        item-cross-size: 44
        item-spacing: 6
        line-spacing: 6

        Box { aspect: 1:1 fill: #4f6272 Text { text: "t00" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t01" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t02" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t03" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t04" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t05" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t06" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t07" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t08" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t09" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t10" } }
        Box { aspect: 1:1 fill: #4f6272 Text { text: "t11" } }
    }
}
"#;

/// A header above a Button, so the Button's parent-relative offset is
/// non-zero — which is what makes the divisor choice observable at all.
const MIXED_SCALE_UI: &str = r#"
component MixedScale inherits Window {
    title: "T8 mixed scale"

    VStack {
        Text { text: "header above the click target" }
        Button { text: "click target" }
    }
}
"#;

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<dpi-scale-matrix-integration>";
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

/// The runner geometry that constrains a real `SetWindowPos` request. These
/// values are diagnostic evidence only: the fixture remains fixed so its
/// arithmetic and mutation signatures do not become environment-dependent.
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

/// The non-client frame, as `outer − client` on each axis.
///
/// **Measured rather than predicted.** Below T9 the process is unaware, so
/// `WM_NCCALCSIZE` should use the 96-DPI metrics whatever a synthesised
/// message claims the DPI is — but that is a prediction, and the realised
/// client is asserted after every rectangle this file applies rather than
/// derived from this number and trusted.
unsafe fn frame_thickness(hwnd: HWND) -> (i32, i32) {
    let outer = window_rect(hwnd);
    let (cw, ch) = client_extent(hwnd);
    (
        (outer.right - outer.left) - cw,
        (outer.bottom - outer.top) - ch,
    )
}

/// Apply an outer rectangle that realises `(client_w, client_h)` physical
/// client pixels, and assert that it did.
unsafe fn set_client_extent(hwnd: HWND, client_w: i32, client_h: i32, what: &str) {
    let (frame_w, frame_h) = frame_thickness(hwnd);
    SetWindowPos(
        hwnd,
        None,
        0,
        0,
        client_w + frame_w,
        client_h + frame_h,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
    )
    .expect("SetWindowPos");
    let limits = display_limits();
    assert_eq!(
        client_extent(hwnd),
        (client_w, client_h),
        "{what}: the realised client extent must be the requested one, or every \
         invariance assertion below is about a rectangle nobody chose; \
         requested_client=({client_w},{client_h}), \
         display=(screen {}x{}, max_track {}x{})",
        limits.0,
        limits.1,
        limits.2,
        limits.3,
    );
}

/// Put a freshly created window into the before-state the assertions below
/// assume: cached scale [`REFERENCE_DPI`], physical client
/// `(CLIENT_W, CLIENT_H)`, and therefore a **DIP** client of the same two
/// numbers.
///
/// One synthesised `WM_DPICHANGED` does both halves, because the handler
/// commits `HIWORD(wParam)` and applies the suggested rectangle in one pass.
/// Both halves are then asserted rather than assumed: the realised client,
/// because a rectangle the display cannot honour is silently not the one
/// applied, and the cached DPI, because everything downstream is expressed as
/// a ratio against it.
///
/// **Why the baseline is reached rather than inherited** (T9 finding F-47):
/// see the module header. The window is created at whatever DPI the monitor
/// reports — 96 on a CI runner, 120 on the development laptop — and nothing
/// below would be machine-independent if that value were allowed through.
///
/// **The scale assertion below is vacuous on a 96-DPI display** — see the
/// module header's stated limit 1. It is kept because it is load-bearing
/// everywhere else and because a silent baseline is worse than a
/// conditionally-sharp one, not because it fires on every machine.
///
/// Note that the non-client frame does **not** move when this runs. The
/// process is Per-Monitor-Aware V2 and the window is still on its real
/// monitor, so the frame keeps that monitor's DPI-indexed metrics whatever
/// this message claims the DPI is. That is why `frame_thickness` is measured
/// live on every call instead of being derived from a DPI.
unsafe fn normalise_to_reference_baseline(window: *mut ffi::WasamoWindow, what: &str) {
    let hwnd = (*window).hwnd;
    send_dpi_change_to_client(hwnd, REFERENCE_DPI, CLIENT_W, CLIENT_H);
    let limits = display_limits();
    assert_eq!(
        client_extent(hwnd),
        (CLIENT_W, CLIENT_H),
        "{what}: the realised client extent must be the requested one, or every \
         invariance assertion below is about a rectangle nobody chose; \
         requested_client=({CLIENT_W},{CLIENT_H}), \
         display=(screen {}x{}, max_track {}x{})",
        limits.0,
        limits.1,
        limits.2,
        limits.3,
    );
    assert_eq!(
        ffi::__window_scale_dpi_for_test(window),
        REFERENCE_DPI,
        "{what}: the baseline scale must be the reference one, or the ratios \
         below are taken against the developer's monitor"
    );
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

/// One node's live Visual rectangle, in physical pixels.
fn visual_rect(node: &wasamo_runtime::WidgetNode) -> (f32, f32, f32, f32) {
    let offset = node.visual.Offset().expect("Visual offset");
    let size = node.visual.Size().expect("Visual size");
    (offset.X, offset.Y, size.X, size.Y)
}

/// The pixel extent of the drawing surface actually installed on a node.
fn surface_pixels(node: &wasamo_runtime::WidgetNode) -> (i32, i32) {
    let brush: CompositionSurfaceBrush = node
        .visual
        .Brush()
        .expect("brush")
        .cast()
        .expect("a text node uses a surface brush");
    let surface: CompositionDrawingSurface = brush
        .Surface()
        .expect("surface")
        .cast()
        .expect("a text node uses a drawing surface");
    let size = surface.SizeInt32().expect("surface pixel size");
    (size.Width, size.Height)
}

/// `(tiles on the first line, number of distinct lines)`, computed by grouping
/// the tiles' Visual `Y` offsets.
///
/// Tiles arranged on the same WrapPanel line share one arranged `Y`, so the
/// grouping is exact rather than a clustering.
fn row_shape(tiles: &[(f32, f32, f32, f32)]) -> (usize, usize) {
    let mut lines: Vec<f32> = Vec::new();
    for (_, y, _, _) in tiles {
        if !lines.contains(y) {
            lines.push(*y);
        }
    }
    let first = lines
        .iter()
        .copied()
        .fold(f32::INFINITY, |a, b| if b < a { b } else { a });
    let on_first = tiles.iter().filter(|(_, y, _, _)| *y == first).count();
    (on_first, lines.len())
}

/// The DPIs whose factor `f32` holds exactly, so the DIP client extent this
/// file chooses survives the inbound division bit-for-bit and the geometry
/// assertion can be an equality.
///
/// `dpi / 96` is dyadic — and therefore exact at these magnitudes — exactly
/// when 3 divides the DPI. At 100 DPI it is not, and the split is **measured
/// rather than defensive**: forcing this function to `true` fails on the root
/// height. The runtime produces the exact integer target; the *test's*
/// expectation (`CLIENT_H × f32(100/96)`) is the imprecise number, because the
/// DIP extent the runtime actually laid out into is the exact integer target
/// divided by that `f32` factor. So the invariance is real at 100 DPI and the
/// naive restatement of it is not exact — a property of `f32`, not of the
/// conversion boundary — which is why the bound is stated for the DPIs that
/// need it instead of being applied everywhere and hiding the three that do
/// not.
fn factor_is_exact(dpi: u32) -> bool {
    dpi % 3 == 0
}

fn assert_scaled(actual: f32, before: f32, factor: f32, exact: bool, what: &str) {
    let expected = before * factor;
    if exact {
        assert_eq!(
            actual, expected,
            "{what}: an exactly-representable factor over a preserved DIP extent \
             must reproduce the geometry bit-for-bit"
        );
    } else {
        let tolerance = 1.0e-3_f32.max(expected.abs() * 8.0 * f32::EPSILON);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{what}: expected {expected}, read {actual}"
        );
    }
}

/// The cached scale is the OS's number, and the handler's `wParam` is the only
/// thing that moves it.
///
/// **The creation-time half is non-degenerate only on a scaled monitor, and
/// this test cannot make it otherwise** (findings F-47 and, for the
/// qualification, the T9 independent review's major 1).
///
/// The assertion used to be paired with `os_dpi == 96`, whose stated job was to
/// record that the equality was `96 == 96` in an unaware process and to fail
/// loudly when T9 landed. It did fail, as designed. Its replacement is not the
/// same assertion with a different number — no number works, because 96 is the
/// *correct* answer on a 100% monitor and `os_dpi != 96` would redden a correct
/// build on a 96-DPI CI runner.
///
/// **What the awareness precondition does and does not buy.** It is what makes
/// `cached == os_dpi` a statement about **per-monitor** DPI rather than about
/// the constant an unaware process is handed; without it the equality is true
/// of a window on any monitor at any scale. It does **not** restore
/// discrimination on a 96-DPI display, and the first version of this comment
/// claimed it did. Measured: with `WindowState`'s seed replaced by
/// `DipScale::IDENTITY` — i.e. a runtime that ignores `GetDpiForWindow`
/// entirely — this test still passes whenever the monitor is at 96 DPI, because
/// `IDENTITY.dpi()` *is* 96 and the awareness assertion is independently true.
/// **So the seeding path has no CI coverage from this test**, and none is
/// available here: on a 96-DPI monitor "seeded from the OS" and "seeded from
/// the identity" are the same number, and no second reading distinguishes them.
/// The development machine at 120 DPI is what makes this test live; CI is not.
/// Recorded as a stated limit in the module header rather than papered over.
///
/// The full artifact for the level itself is
/// `dpi_awareness_declaration_integration.rs`; the precondition below is what
/// this test needs in order to mean what it says.
#[test]
fn a_created_windows_cached_scale_is_the_dpi_the_os_reports() {
    run_on_owning_runtime_thread_or_skip("cached window scale", move || {
        let ir = lower_ui_to_ir(MATRIX_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;

            let os_dpi = GetDpiForWindow(hwnd);
            let cached = ffi::__window_scale_dpi_for_test(window);
            let declared_per_monitor_v2 = AreDpiAwarenessContextsEqual(
                GetWindowDpiAwarenessContext(hwnd),
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            )
            .as_bool();

            set_client_extent(hwnd, CLIENT_W, CLIENT_H, "creation-time normalisation");
            send_dpi_change_to_client(hwnd, 144, CLIENT_W * 144 / 96, CLIENT_H * 144 / 96);
            let after = ffi::__window_scale_dpi_for_test(window);

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                cached, os_dpi,
                "the window's authoritative scale is seeded from GetDpiForWindow \
                 and from nothing else"
            );
            assert!(
                declared_per_monitor_v2,
                "the equality above is only a claim about per-monitor DPI while \
                 Per-Monitor-Aware V2 is the level in force; under an unaware \
                 process GetDpiForWindow answers 96 unconditionally and the \
                 assertion holds for a window on any monitor at any scale"
            );
            assert_eq!(
                after, 144,
                "step 1 of DD-M4-P1-003 commits HIWORD(wParam), so the cache is \
                 the DPI the message carried"
            );
        }
    });
}

/// **The positive control.** The DIP layout results are unchanged while every
/// Visual offset and size has moved by the scale ratio.
///
/// The two halves are asserted by two different shapes of fact, because with
/// the before-state at `s = 1` a ratio assertion and a "DIP unchanged"
/// assertion are one equation (finding F-45):
///
/// - **Input-side**: `realised_client` equals a target computed from
///   [`CLIENT_W`]. This is the one assertion here that no ratio assertion can
///   stand in for — it is read from `GetClientRect`, not off the Visual tree,
///   and nothing else touches it. The root-Visual assertions sit beside it and
///   are **partly implied**: at the three exact DPIs `before_root × factor`
///   pins the size components, so what stays independent there is the *offset*
///   components — no `assert_scaled` covers `.0` / `.1` — and, at 100 DPI, an
///   exact tuple where the ratio assertion only carries a tolerance.
///
///   **What this does and does not do, corrected at the T8 round-3 review.**
///   It does not supply a second independent *reading of the output*, which is
///   what F-45 says the runtime does not offer. It guarantees the experiment
///   held the input it claims to have held. That is the honest role, and the
///   earlier wording — "the one that actually separates them", plus "nothing
///   read off the post-change tree can stand in for them" — overstated it in
///   two ways at once: `after_root` **is** read off the post-change tree, so
///   the sentence disqualified one of its own two members.
/// - **Discrete**: the WrapPanel row assignment, against a count derived from
///   the `.ui` source. Independent for the **before** state; redundant for
///   the after state and kept for legibility — see [`TILES_PER_ROW`], which
///   records why the original "the half a ratio assertion cannot make" was
///   withdrawn.
/// - **Continuous**: every tile's offset and size equals its 96-DPI value
///   times the factor.
/// - **Integer, from the raster path**: each tile's text surface is the `ceil`
///   of its Visual extent, so geometry and rasterization agree at every DPI in
///   the matrix rather than only at the ones the product expects.
#[test]
fn dip_layout_is_invariant_while_every_visual_moves_by_the_ratio() {
    run_on_owning_runtime_thread_or_skip("DIP invariance across the scale matrix", move || {
        let ir = lower_ui_to_ir(MATRIX_UI);
        for dpi in DPI_MATRIX {
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                let factor = dpi as f32 / REFERENCE_DPI as f32;
                let exact = factor_is_exact(dpi);

                normalise_to_reference_baseline(window, "pre-change normalisation");
                let before_root = read_root_rect(window);
                let before = read_tiles(window);
                let before_surface = read_first_label_surface(window);

                let target_w = CLIENT_W * dpi as i32 / REFERENCE_DPI as i32;
                let target_h = CLIENT_H * dpi as i32 / REFERENCE_DPI as i32;
                send_dpi_change_to_client(hwnd, dpi, target_w, target_h);

                let realised_client = client_extent(hwnd);
                let cached = ffi::__window_scale_dpi_for_test(window);
                let after_root = read_root_rect(window);
                let after = read_tiles(window);
                let after_surface = read_first_label_surface(window);
                let after_label = read_first_label_rect(window);
                let limits = display_limits();

                ffi::wasamo_window_destroy(window);

                assert_eq!(
                    realised_client,
                    (target_w, target_h),
                    "dpi={dpi}: the DIP client extent is preserved only if the \
                     physical one moved by the ratio; the whole invariance claim \
                     is about this input; requested_client=({target_w},{target_h}), \
                     display=(screen {}x{}, max_track {}x{})",
                    limits.0,
                    limits.1,
                    limits.2,
                    limits.3,
                );
                assert_eq!(cached, dpi, "dpi={dpi}: the scale is committed");

                // The one readback that **does** depend on the client extent's
                // low bits, and therefore the one that makes the normalisation
                // load-bearing rather than merely tidy (finding F-44, corrected
                // by mutation M5): the window-root override forces the root
                // node to `Fill`, so its Visual is the client rectangle. The
                // tiles below are start-packed at a fixed cross-size and do
                // not move when the extent shifts by a fraction of a DIP —
                // measured, not assumed.
                assert_eq!(
                    before_root,
                    (0.0, 0.0, CLIENT_W as f32, CLIENT_H as f32),
                    "dpi={dpi}: at 96 DPI the root Visual is the chosen physical \
                     client, so the layout input is the number this file picked \
                     and not one the OS chose"
                );
                assert_eq!(
                    after_root,
                    (0.0, 0.0, target_w as f32, target_h as f32),
                    "dpi={dpi}: the root fills the new physical client exactly, so \
                     the DIP extent layout received is unchanged"
                );

                // The witness is not degenerate: a "scaled by k" assertion is
                // satisfied by zero (T7 finding F-42).
                assert_eq!(before.len(), TILE_COUNT, "dpi={dpi}");
                for (i, (_, _, w, h)) in before.iter().enumerate() {
                    assert!(*w > 0.0 && *h > 0.0, "dpi={dpi}: tile {i} is degenerate");
                }

                // Discrete: the row assignment, against the source-derived count.
                assert_eq!(
                    row_shape(&before),
                    (TILES_PER_ROW, ROWS),
                    "dpi={dpi}: the 96-DPI baseline must already be the row shape \
                     the fixture's authored numbers predict"
                );
                assert_eq!(
                    row_shape(&after),
                    (TILES_PER_ROW, ROWS),
                    "dpi={dpi}: layout receives the same DIP client extent, so the \
                     same tiles must sit on the same lines. Redundant against the \
                     ratio assertion below and kept so a failure names the 9-vs-7 \
                     signature rather than a pile of f32 mismatches — see \
                     TILES_PER_ROW"
                );

                // Continuous: every Visual moved by the ratio.
                assert_scaled(
                    after_root.2,
                    before_root.2,
                    factor,
                    exact,
                    &format!("dpi={dpi} root width"),
                );
                assert_scaled(
                    after_root.3,
                    before_root.3,
                    factor,
                    exact,
                    &format!("dpi={dpi} root height"),
                );
                for (i, (b, a)) in before.iter().zip(after.iter()).enumerate() {
                    assert_scaled(a.0, b.0, factor, exact, &format!("dpi={dpi} tile {i} x"));
                    assert_scaled(a.1, b.1, factor, exact, &format!("dpi={dpi} tile {i} y"));
                    assert_scaled(a.2, b.2, factor, exact, &format!("dpi={dpi} tile {i} w"));
                    assert_scaled(a.3, b.3, factor, exact, &format!("dpi={dpi} tile {i} h"));
                }

                // Integer, from the raster path.
                assert_ne!(
                    before_surface, after_surface,
                    "dpi={dpi}: the control requires the 96-DPI surface to be \
                     observably stale at the new DPI"
                );
                assert_eq!(
                    after_surface,
                    (
                        after_label.2.ceil().max(1.0) as i32,
                        after_label.3.ceil().max(1.0) as i32
                    ),
                    "dpi={dpi}: the surface is allocated at ceil(dip x s) over the \
                     Visual's exact physical extent"
                );
            }
        }
    });
}

/// Tile Visual rectangles, in document order.
unsafe fn read_tiles(window: *mut ffi::WasamoWindow) -> Vec<(f32, f32, f32, f32)> {
    let root = (*window)
        .root_widget
        .as_ref()
        .expect("wasamo_load_ui installs a content root");
    root.children.iter().map(|c| visual_rect(c)).collect()
}

/// The content root's own Visual rectangle — the client rectangle, because
/// `run_layout_as_window_root_at_scale` forces the root node to `Fill`.
unsafe fn read_root_rect(window: *mut ffi::WasamoWindow) -> (f32, f32, f32, f32) {
    visual_rect((*window).root_widget.as_ref().expect("content root"))
}

unsafe fn read_first_label_rect(window: *mut ffi::WasamoWindow) -> (f32, f32, f32, f32) {
    let root = (*window).root_widget.as_ref().expect("content root");
    visual_rect(&root.children[0].children[0])
}

unsafe fn read_first_label_surface(window: *mut ffi::WasamoWindow) -> (i32, i32) {
    let root = (*window).root_widget.as_ref().expect("content root");
    surface_pixels(&root.children[0].children[0])
}

/// **The one-divisor traversal property** (T5 finding F-37).
///
/// A hit-test traversal divides every `visual_rect` readback by the
/// *traversal root's* scale, not by each node's own, so a descendant whose
/// cached geometry scale is stale still resolves to the rectangle it is
/// actually composited at. Per-node division would place it at
/// `physical ÷ its own scale`, which is the composited position multiplied by
/// the window's factor — a different rectangle.
///
/// The state is reached through a seam because no legitimate path leaves one:
/// `commit_scale_recursive` writes the whole subtree, and the incremental
/// attach paths F-32 enumerated leave a fresh node with no geometry to
/// hit-test at all. The stale value is the **constructor identity**, which is
/// the value those paths really would leave behind.
///
/// The click is driven as a real `WM_LBUTTONUP` so the pointer crosses the
/// inbound seam in `wnd_proc` — divided by the *window's* scale — rather than
/// being handed to `hit_test_click` already in DIP. That is the mixture the
/// property is about.
#[test]
fn a_stale_descendant_scale_still_hit_tests_where_the_widget_is() {
    run_on_owning_runtime_thread_or_skip("mixed-scale hit test", move || {
        let ir = lower_ui_to_ir(MIXED_SCALE_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            let dpi = 120_u32;
            let factor = dpi as f32 / REFERENCE_DPI as f32;

            normalise_to_reference_baseline(window, "pre-change normalisation");
            send_dpi_change_to_client(
                hwnd,
                dpi,
                CLIENT_W * dpi as i32 / REFERENCE_DPI as i32,
                CLIENT_H * dpi as i32 / REFERENCE_DPI as i32,
            );

            let clicks = Rc::new(Cell::new(0u32));
            let (px, py, pw, ph) = {
                let root = (*window).root_widget.as_mut().expect("content root");
                let counter = Rc::clone(&clicks);
                root.children[1].set_clicked(move || counter.set(counter.get() + 1));
                visual_rect(&root.children[1])
            };

            // The click point, in the physical client pixels a pointer message
            // carries: horizontally central, one DIP below the button's top
            // edge. The vertical choice is what makes the test discriminate —
            // see the assertion on `stale_top_dip` below.
            let click_x = px + pw / 2.0;
            let click_y = py + factor;
            let click_dip_y = click_y / factor;

            // What per-node division would compute for this node: its physical
            // rectangle divided by the *constructor identity* rather than by
            // the window's scale, i.e. the composited position multiplied by
            // the factor.
            let stale_top_dip = py;

            let control_before = clicks.get();
            send_click(hwnd, click_x, click_y);
            let control_after = clicks.get();

            (*window)
                .root_widget
                .as_mut()
                .expect("content root")
                .children[1]
                .__set_geometry_scale_dpi_for_test(REFERENCE_DPI);
            send_click(hwnd, click_x, click_y);
            let stale_after = clicks.get();

            let stale_dpi_readback = {
                let root = (*window).root_widget.as_ref().expect("content root");
                (visual_rect(&root.children[1]), root.children.len())
            };

            ffi::wasamo_window_destroy(window);

            assert!(
                pw > 0.0 && ph > 0.0,
                "the click target must have been laid out: read {pw}x{ph}"
            );
            assert_eq!(stale_dpi_readback.1, 2, "header + button");
            assert_eq!(
                stale_dpi_readback.0,
                (px, py, pw, ph),
                "poking a node's cached scale must not move its Visual — the \
                 seam writes the derived copy and nothing else"
            );
            assert!(
                click_dip_y < stale_top_dip,
                "the fixture has stopped discriminating: the click at {click_dip_y} \
                 DIP must fall inside the button's real DIP rectangle and outside \
                 the {stale_top_dip}-DIP top edge per-node division would compute. \
                 A header tall enough to push the button down is what buys this."
            );
            assert_eq!(
                control_after - control_before,
                1,
                "control: with no stale cache the click point resolves to the \
                 button, so the negative case below is about the divisor and not \
                 about the coordinates"
            );
            assert_eq!(
                stale_after - control_after,
                1,
                "a descendant left at the constructor identity must still hit-test \
                 where it is composited — the traversal divides by the root's scale"
            );
        }
    });
}

/// A real `WM_LBUTTONUP` at a **physical** client position, which is what the
/// message stream carries.
unsafe fn send_click(hwnd: HWND, x: f32, y: f32) {
    let packed = ((y.round() as i32 as u32) << 16) | (x.round() as i32 as u32 & 0xFFFF);
    SendMessageW(
        hwnd,
        WM_LBUTTONUP,
        WPARAM(0),
        LPARAM(packed as i32 as isize),
    );
}
