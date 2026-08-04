//! Mock-free Windows integration evidence for M4-Phase 1 T7.
//!
//! Drives real `WM_DPICHANGED` messages through the real window procedure of a
//! real window and reads the live Composition objects back. Nothing here is
//! mocked and nothing is stubbed: the OS dispatches (or declines to dispatch)
//! the re-entrant `WM_SIZE` itself, which is the whole point — the handler's
//! contract is about what `SetWindowPos` actually does, not about what it is
//! documented to do.
//!
//! **What this binary is and is not.** It fires T7's authored branches: the
//! no-nested-geometry fallback, the null suggested rectangle, and the denial of
//! step 4 when no projection succeeds. The full scale-change positive control —
//! DIP layout results held invariant while every Visual moves by the ratio,
//! across 125% / 150% / 200% and one non-standard DPI — is T8's, and the literal
//! monitor crossing is T11's. The assertions below are deliberately the weakest
//! ones that still discriminate the branch under test.
//!
//! The messages are still synthesised, which is what lets these tests choose
//! the branch under test: the handler reads its new DPI out of `wParam`, so a
//! synthesised message drives the same code the OS drives. What a synthesised
//! message cannot show is that a real monitor crossing delivers a usable
//! suggested rectangle; that half is T11's.
//!
//! **The before-state is established, never inherited** (T9 finding F-47). This
//! file used to name a fixed `CHANGED_DPI` of 120 and derive its expected ratio
//! from 96, both of which were facts about a process that had not declared DPI
//! awareness. Since T9 declared Per-Monitor-Aware V2 the OS reports the
//! monitor's real DPI, and on a 125% machine the constant became the window's
//! *own* DPI — so the change every test here drove was a no-op, and three of
//! the four failed. The target is now derived from the window's committed
//! creation-time DPI, which makes each test drive a real change on a 96-DPI CI
//! runner and on a scaled laptop alike. See [`changed_dpi`].

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;

use wasamo_runtime::ffi;

use windows::core::Interface;
use windows::Win32::Foundation::{LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, SendMessageW, WM_DPICHANGED};
use windows::UI::Composition::{CompositionDrawingSurface, CompositionSurfaceBrush};

/// The ratio every test below expects each Visual to move by.
///
/// Exactly 2 — see [`changed_dpi`]. A power of two is representable at every
/// magnitude, so an assertion that a Visual grew by it is exact rather than
/// approximate. Choosing an awkward factor is T8's job, where the *arithmetic*
/// is what is under test; here the branches are, and an exact factor keeps a
/// branch failure from reading as a rounding one.
const CHANGED_FACTOR: f32 = 2.0;

/// The DPI a test changes `window` to: **twice the DPI the window was created
/// at**, read back from the runtime's own committed value rather than assumed.
///
/// Doubling is what makes this file machine-independent (T9 finding F-47).
/// Naming a constant DPI cannot be: whatever value is picked is some machine's
/// native setting, and on that machine the "change" is a no-op — the ratio
/// assertions read a factor of 1 and the surface-staleness control passes
/// vacuously because nothing needed re-rasterizing. Deriving the target from
/// the before-state guarantees a real change everywhere, and doubling makes the
/// expected ratio [`CHANGED_FACTOR`] on every machine rather than a number the
/// test would have to compute alongside the runtime.
///
/// Read through the T8 scale seam rather than through `GetDpiForWindow`,
/// because what the ratio is taken against is the scale the runtime
/// **committed** at creation — the value the before-state's Visual geometry was
/// actually projected at.
unsafe fn changed_dpi(window: *mut ffi::WasamoWindow) -> u32 {
    ffi::__window_scale_dpi_for_test(window) * 2
}

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<dpi-change-integration>";
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

/// A window whose content tree lays out cleanly, with one `Text` node as the
/// witness: its DIP extent is `Fixed` from `measure`, so its Visual size is the
/// projection target's factor and nothing else — independent of the client
/// extent the change happens to produce.
const CLEAN_UI: &str = r#"
component Probe inherits Window {
    title: "T7 probe"

    VStack {
        Text { text: "projection witness" }
    }
}
"#;

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

/// The witness Text node's live Visual extent and the pixel extent of the
/// surface actually installed on it. The two are separate facts by design since
/// T6 — geometry projection and last-rasterized DPI have their own markers.
/// Three tests read both. The failed-projection test intentionally reads only
/// the surface: no geometry pass succeeds there, so it has no projected Visual
/// extent to treat as a witness.
unsafe fn witness(window: *mut ffi::WasamoWindow) -> ((f32, f32), (i32, i32)) {
    let root = (*window)
        .root_widget
        .as_ref()
        .expect("wasamo_load_ui installs a content root");
    let text = &root.children[0];
    let size = text.visual.Size().expect("witness Visual size");
    let brush: CompositionSurfaceBrush = text
        .visual
        .Brush()
        .expect("witness brush")
        .cast()
        .expect("witness uses a surface brush");
    let surface: CompositionDrawingSurface = brush
        .Surface()
        .expect("witness surface")
        .cast()
        .expect("witness uses a drawing surface");
    let pixels = surface.SizeInt32().expect("witness surface pixel size");
    ((size.X, size.Y), (pixels.Width, pixels.Height))
}

unsafe fn window_rect(window: *mut ffi::WasamoWindow) -> RECT {
    let mut rect = RECT::default();
    GetWindowRect((*window).hwnd, &mut rect).expect("GetWindowRect");
    rect
}

/// Send a real `WM_DPICHANGED`. `suggested` is the `lParam` the OS would supply
/// as a `RECT*`; `None` is the null case.
unsafe fn send_dpi_change(window: *mut ffi::WasamoWindow, dpi: u32, suggested: Option<&RECT>) {
    let wparam = WPARAM(((dpi as usize) << 16) | dpi as usize);
    let lparam = LPARAM(match suggested {
        Some(r) => r as *const RECT as isize,
        None => 0,
    });
    let result = SendMessageW((*window).hwnd, WM_DPICHANGED, wparam, lparam);
    assert_eq!(
        result.0, 0,
        "DD-M4-P1-003 step 5: the handler returns LRESULT(0) whatever happened"
    );
}

/// Install a counter on `resize_fn` and hand back the shared cell.
///
/// This is the only **public** observation of whether a re-entrant `WM_SIZE`
/// ran, and it is one because T7 decided to synthesise no host callback of its
/// own: `resize_fn` fires from the `WM_SIZE` arm and from nowhere else, so a
/// fallback — which happens precisely when no `WM_SIZE` was dispatched — leaves
/// it untouched.
unsafe fn count_resizes(window: *mut ffi::WasamoWindow) -> Rc<Cell<u32>> {
    let calls = Rc::new(Cell::new(0u32));
    let sink = Rc::clone(&calls);
    (*window).resize_fn = Some(Box::new(move |_w, _h| {
        sink.set(sink.get() + 1);
    }));
    calls
}

fn assert_close(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "{what}: expected {expected}, read {actual}"
    );
}

/// The nested path, **and two controls**.
///
/// The first is what gives the next test's negative assertion its meaning: it
/// establishes that `resize_fn` fires at all, so "`resize_fn` was not called" is
/// a fact about that run rather than about the slot being permanently inert.
///
/// The second is the `SWP_NOMOVE` hazard. The suggested rectangle here **moves
/// as well as resizes**, and the window's realised rectangle is asserted against
/// it — because the flag set is the one thing on this path that
/// `window::realize_dip_window_size` gets right for its own purposes and wrong
/// for this one, and the handoff records that inheriting it "pins the window on
/// every monitor crossing while every test stays green". A rectangle that only
/// changed the size would have left that true.
#[test]
fn a_size_changing_suggested_rectangle_projects_through_the_nested_wm_size() {
    run_on_owning_runtime_thread_or_skip("DPI change with a size-changing rectangle", move || {
        let ir = lower_ui_to_ir(CLEAN_UI);
        unsafe {
            let window = load_window(&ir);
            let (before_size, before_pixels) = witness(window);
            let resizes = count_resizes(window);

            let current = window_rect(window);
            let suggested = RECT {
                left: current.left + 40,
                top: current.top + 30,
                right: current.right + 160,
                bottom: current.bottom + 120,
            };
            send_dpi_change(window, changed_dpi(window), Some(&suggested));

            let (after_size, after_pixels) = witness(window);
            let realised = window_rect(window);
            let resize_calls = resizes.get();
            ffi::wasamo_window_destroy(window);

            assert!(
                resize_calls >= 1,
                "a suggested rectangle that changes the size must dispatch a \
                 re-entrant WM_SIZE — without this the next test's negative \
                 assertion would be vacuous"
            );
            assert_eq!(
                (realised.left, realised.top, realised.right, realised.bottom),
                (
                    suggested.left,
                    suggested.top,
                    suggested.right,
                    suggested.bottom
                ),
                "the OS suggestion is a new position *and* size; inheriting \
                 SWP_NOMOVE from the creation-time correction would pin the window"
            );
            assert_close(
                after_size.0,
                before_size.0 * CHANGED_FACTOR,
                "witness Visual width after the nested projection",
            );
            assert_close(
                after_size.1,
                before_size.1 * CHANGED_FACTOR,
                "witness Visual height after the nested projection",
            );
            assert_eq!(
                (after_pixels.0, after_pixels.1),
                (after_size.0.ceil() as i32, after_size.1.ceil() as i32),
                "step 4 must re-rasterize at the new device resolution"
            );
            assert_ne!(
                before_pixels, after_pixels,
                "the control requires the creation-DPI surface to be observably \
                 stale at the doubled DPI"
            );
        }
    });
}

/// **The trap-#4 artifact.** A suggested rectangle equal to the window's current
/// one makes `SetWindowPos` succeed without changing the size, and a
/// `SetWindowPos` that changes no size dispatches no `WM_SIZE` at all (measured
/// at T4). The DIP client extent has still changed, because the same physical
/// extent divided by a new scale is a new DIP extent — so the fallback is the
/// only thing that can have projected the tree, and `resize_fn` not firing is
/// what says so.
#[test]
fn an_unchanged_suggested_rectangle_projects_through_the_fallback() {
    run_on_owning_runtime_thread_or_skip("DPI change with no nested WM_SIZE", move || {
        let ir = lower_ui_to_ir(CLEAN_UI);
        unsafe {
            let window = load_window(&ir);
            let (before_size, before_pixels) = witness(window);
            let resizes = count_resizes(window);

            let unchanged = window_rect(window);
            send_dpi_change(window, changed_dpi(window), Some(&unchanged));

            let (after_size, after_pixels) = witness(window);
            let resize_calls = resizes.get();
            ffi::wasamo_window_destroy(window);

            assert_eq!(
                resize_calls, 0,
                "an unchanged rectangle must dispatch no re-entrant WM_SIZE; if it \
                 did, this test would be exercising the nested path instead of the \
                 fallback"
            );
            assert_close(
                after_size.0,
                before_size.0 * CHANGED_FACTOR,
                "witness Visual width after the fallback projection",
            );
            assert_close(
                after_size.1,
                before_size.1 * CHANGED_FACTOR,
                "witness Visual height after the fallback projection",
            );
            assert_eq!(
                (after_pixels.0, after_pixels.1),
                (after_size.0.ceil() as i32, after_size.1.ceil() as i32),
                "step 4 runs after a successful fallback, not only after a nested pass"
            );
            assert_ne!(before_pixels, after_pixels);
        }
    });
}

/// The null-`lParam` guard. `wnd_proc` is reachable by `SendMessageW` from any
/// process, so this is a reachable input rather than a hypothetical one. Step 2
/// is skipped, the scale is already committed, and the fallback carries the
/// change — so a malformed message still converges instead of leaving the
/// authoritative scale ahead of every projection.
#[test]
fn a_null_suggested_rectangle_survives_and_still_projects() {
    run_on_owning_runtime_thread_or_skip("DPI change with a null rectangle", move || {
        let ir = lower_ui_to_ir(CLEAN_UI);
        unsafe {
            let window = load_window(&ir);
            let (before_size, _) = witness(window);
            let resizes = count_resizes(window);
            let before_rect = window_rect(window);

            send_dpi_change(window, changed_dpi(window), None);

            let (after_size, after_pixels) = witness(window);
            let after_rect = window_rect(window);
            let resize_calls = resizes.get();
            ffi::wasamo_window_destroy(window);

            assert_eq!(
                resize_calls, 0,
                "step 2 is skipped, so no WM_SIZE is dispatched"
            );
            assert_eq!(
                (
                    before_rect.left,
                    before_rect.top,
                    before_rect.right,
                    before_rect.bottom
                ),
                (
                    after_rect.left,
                    after_rect.top,
                    after_rect.right,
                    after_rect.bottom
                ),
                "with no rectangle to apply the window keeps its own"
            );
            assert_close(
                after_size.0,
                before_size.0 * CHANGED_FACTOR,
                "witness Visual width after the guard's fallback",
            );
            assert_eq!(
                (after_pixels.0, after_pixels.1),
                (after_size.0.ceil() as i32, after_size.1.ceil() as i32),
                "the guard degrades to the fallback, which permits step 4"
            );
        }
    });
}

/// A tree whose layout cannot resolve, so **both** the nested projection and the
/// fallback fail and step 4 must be denied.
///
/// The failure is deterministic rather than injected: `measure_vstack` passes an
/// infinite child height and `measure_hstack` an infinite child width, so a
/// childless `Box` nested in both is measured against unbounded space on both
/// axes and returns `LayoutError::BoxNoExtent` (DD-M3-P2-005). It is reached
/// through the `.ui` path because `WidgetNode::box_` is crate-private.
///
/// This is discriminating against the *shipped* alternative rather than against
/// a hypothetical one: the `WM_SIZE` arm has refreshed text unconditionally
/// since T6, so a T7 that did not gate step 4 would advance the witness's raster
/// marker to the new DPI while no geometry pass had succeeded — exactly the
/// convergence claim the T6 independent review rejected.
#[test]
fn two_failed_projections_leave_the_text_stale_without_diverging() {
    run_on_owning_runtime_thread_or_skip("DPI change with unresolvable layout", move || {
        let ir = lower_ui_to_ir(
            r#"
component Unresolvable inherits Window {
    title: "T7 unresolvable"

    VStack {
        Text { text: "stale raster witness" }
        HStack {
            Box { }
        }
    }
}
"#,
        );
        unsafe {
            let window = load_window(&ir);
            let (_, before_pixels) = witness(window);

            let current = window_rect(window);
            let grown = RECT {
                left: current.left,
                top: current.top,
                right: current.right + 120,
                bottom: current.bottom + 90,
            };
            send_dpi_change(window, changed_dpi(window), Some(&grown));

            let (_, after_pixels) = witness(window);
            let health = ffi::__runtime_health_for_test();
            ffi::wasamo_window_destroy(window);

            assert_eq!(
                after_pixels, before_pixels,
                "with no successful geometry pass, step 4 must not advance any raster \
                 marker — a surface at the new DPI would claim a convergence the \
                 Visual tree does not have"
            );
            assert_eq!(
                health, "Healthy",
                "DD-M4-P1-003 §Failure handling: a failed geometry call is neither \
                 reactive nor unrecoverable, so the runtime is not put into Diverged"
            );
        }
    });
}
