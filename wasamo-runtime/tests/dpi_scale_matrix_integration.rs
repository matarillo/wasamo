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
//! 2. **The exact-invariance assertion holds because this file chooses the
//!    rectangle.** It preserves the **DIP client extent**, which is the input
//!    layout actually receives. The OS's own suggested rectangle preserves the
//!    **outer** rectangle instead, and the non-client frame scales by its own
//!    DPI-indexed metrics rather than by `s` — so on the real path the DIP
//!    layout input moves by a DIP or two and invariance is approximate. That
//!    is not a failure; it is a different rectangle.
//! 3. **The process has not declared DPI awareness yet** (T9 does), so
//!    `GetDpiForWindow` reports 96 for every window here and the OS does not
//!    drive this message for the per-monitor changes the phase is about. The
//!    creation-time half of test 1 is therefore `96 == 96` until T9; what is
//!    live now is the post-change half, where the cache must equal the DPI
//!    this file sent.

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
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetWindowRect, SendMessageW, SetWindowPos, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOZORDER, WM_DPICHANGED, WM_LBUTTONUP,
};
use windows::UI::Composition::{CompositionDrawingSurface, CompositionSurfaceBrush};

/// The reference DPI, and the value an undeclared process is told
/// unconditionally (DD-M4-P1-001).
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
/// integer. The window `wasamo_load_ui` creates has a client of 784 × 561
/// physical at 96 DPI (T4, measured), and 561 × 1.25 is 701.25 — so no
/// synthesised rectangle preserves the DIP extent from there, at any DPI in
/// the matrix. `96 = 2^5 × 3`, and the matrix contributes denominators 4, 2,
/// 1 and 24, so **a multiple of 24 makes all four exact at once**: 720 → 750 /
/// 900 / 1080 / 1440 and 480 → 500 / 600 / 720 / 960.
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
///    one. 720 × 480 asks for 1440 × 960 and gets it on the development
///    machine; a smaller display needs a smaller multiple of 24, and
///    `set_client_extent`'s assertion is what says so rather than letting the
///    run drift.
const CLIENT_W: i32 = 720;
const CLIENT_H: i32 = 480;

/// Tiles per row, derived from the fixture's authored numbers and
/// [`CLIENT_W`] — **not** from anything read back off a Visual.
///
/// This is the witness that makes the ADR's evidence item (2) two claims
/// rather than one (finding F-45). "The DIP layout results are unchanged" and
/// "the Visuals moved by the ratio" are the same equation while the
/// before-state is `s = 1`, because the only reading of a DIP layout result
/// the runtime offers is a Visual read back and divided. A row count computed
/// from the `.ui` source is independent of both: `88 + 12` per tile in a 720
/// DIP line gives `floor((720 + 12) / 100) = 7`, and an implementation that
/// treated the physical client as logical would lay out into 900 DIP and fit
/// **9** — the signature the plan records for §T10.
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
        item-cross-size: 88
        item-spacing: 12
        line-spacing: 12

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
    assert_eq!(
        client_extent(hwnd),
        (client_w, client_h),
        "{what}: the realised client extent must be the requested one, or every \
         invariance assertion below is about a rectangle nobody chose"
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
/// height with `500.0` read against `499.99997` expected. Note which side is
/// which. The runtime produced exactly 500; the *test's* expectation
/// (`before × factor` = `480 × f32(100/96)`) is the imprecise number, because
/// the DIP extent the runtime actually laid out into is `500 ÷ f32(100/96)` =
/// 480.00003 rather than 480. So the invariance is real at 100 DPI and the
/// naive restatement of it is not exact — a property of `f32`, not of the
/// conversion boundary — which is why the bound is stated for the DPIs that
/// need it instead of being applied everywhere and hiding the three that
/// do not.
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
#[test]
fn a_created_windows_cached_scale_is_the_dpi_the_os_reports() {
    run_on_owning_runtime_thread_or_skip("cached window scale", move || {
        let ir = lower_ui_to_ir(MATRIX_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;

            let os_dpi = GetDpiForWindow(hwnd);
            let cached = ffi::__window_scale_dpi_for_test(window);

            set_client_extent(hwnd, CLIENT_W, CLIENT_H, "creation-time normalisation");
            send_dpi_change_to_client(hwnd, 144, CLIENT_W * 144 / 96, CLIENT_H * 144 / 96);
            let after = ffi::__window_scale_dpi_for_test(window);

            ffi::wasamo_window_destroy(window);

            assert_eq!(
                cached, os_dpi,
                "the window's authoritative scale is seeded from GetDpiForWindow \
                 and from nothing else"
            );
            assert_eq!(
                os_dpi, REFERENCE_DPI,
                "the process has not declared awareness yet (T9 does), so the OS \
                 reports the reference DPI — this assertion is what makes the \
                 equality above a degenerate one until then, and it must be \
                 re-read when T9 lands"
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
/// - **Discrete**: the WrapPanel row assignment, against a count derived from
///   the `.ui` source and the chosen client extent. An implementation that
///   treated the physical client as logical would fit 9 tiles per row here
///   instead of 7.
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

                set_client_extent(hwnd, CLIENT_W, CLIENT_H, "pre-change normalisation");
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

                ffi::wasamo_window_destroy(window);

                assert_eq!(
                    realised_client,
                    (target_w, target_h),
                    "dpi={dpi}: the DIP client extent is preserved only if the \
                     physical one moved by the ratio; the whole invariance claim \
                     is about this input"
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
                     same tiles must sit on the same lines — this is the half a \
                     ratio assertion cannot make"
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

            set_client_extent(hwnd, CLIENT_W, CLIENT_H, "pre-change normalisation");
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
