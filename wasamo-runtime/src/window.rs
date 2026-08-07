use crate::dip_scale::DipScale;
use crate::focus::{self, WindowFocus};
use crate::runtime;
use crate::widget::{HoverState, WidgetNode};
use windows::{
    core::Interface,
    Foundation::Numerics::Vector2,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Dwm::{
            DwmSetWindowAttribute, DWMSBT_MAINWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
            DWMWINDOWATTRIBUTE, DWM_SYSTEMBACKDROP_TYPE,
        },
        UI::{
            HiDpi::GetDpiForWindow,
            Input::KeyboardAndMouse::{
                GetKeyState, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT, VK_SHIFT,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, LoadCursorW, PostQuitMessage,
                RegisterClassExW, SetWindowLongPtrW, SetWindowPos, ShowWindow, CS_HREDRAW,
                CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, SWP_NOACTIVATE, SWP_NOMOVE,
                SWP_NOZORDER, SW_SHOW, WM_DESTROY, WM_DPICHANGED, WM_ERASEBKGND, WM_KEYDOWN,
                WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_SIZE, WNDCLASSEXW,
                WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW,
            },
        },
    },
    UI::Composition::{ContainerVisual, Desktop::DesktopWindowTarget, Visual},
};

// WM_MOUSELEAVE is not exported from WindowsAndMessaging in windows 0.58.
const WM_MOUSELEAVE: u32 = 0x02A3;

pub struct WindowState {
    pub hwnd: HWND,
    pub root: ContainerVisual,
    // Kept alive: dropping DesktopWindowTarget detaches Visual Layer from HWND.
    _target: DesktopWindowTarget,

    /// This window's DIP -> physical conversion factor, seeded from
    /// `GetDpiForWindow` at creation and refreshed by the `WM_DPICHANGED`
    /// handler (DD-M4-P1-003 §Where the scale is held). It is per window and
    /// not per process, which is what lets a second window on a
    /// differently-scaled monitor be an additive change rather than a rebuild.
    ///
    /// A field rather than a value threaded through `create`, so that **no
    /// window can exist without one**: `set_root`'s first layout, the `WM_SIZE`
    /// arm and every conversion seam read it off state they already hold, and
    /// there is no statement order a later edit can invert.
    ///
    /// `pub(crate)` rather than `pub`: `emit::flush_layout` reads it from
    /// another module, but no host does. DD-M4-P1-004 walks every M4 phase and
    /// concludes that no host needs the scale factor, so putting it on a
    /// `pub use`-exported type would ship the surface that decision declines.
    ///
    /// Read by `wnd_proc`'s size and pointer arms, by `set_root`'s first
    /// layout and by `emit::flush_layout`; rewritten by the `WM_DPICHANGED`
    /// handler at T7.
    pub(crate) scale: DipScale,

    /// `Some` for exactly the span of the `SetWindowPos` inside a
    /// `WM_DPICHANGED` handler (M4-Phase 1 T7), carrying what that call's
    /// **re-entrant** `WM_SIZE` reported about DD-M4-P1-003 step 3.
    ///
    /// It exists because `SetWindowPos` dispatches `WM_SIZE` synchronously,
    /// which makes the nested pass the handler's step 3 rather than an ordinary
    /// resize. The `WM_SIZE` arm therefore consults it for two things: it must
    /// **not** refresh text (step 4 owns that, after `SetWindowPos` returns,
    /// and only behind a succeeded projection), and it must report whether its
    /// projection *succeeded* — entry into the message is not the fact the
    /// handler needs.
    ///
    /// Written only by `begin_scale_change` and `handle_dpi_changed`, and
    /// `begin_scale_change` commits `scale` in the same breath: **there is no
    /// path that installs this marker without having already committed the new
    /// scale.** That is the property the primitive buys, and it is narrower than
    /// "the step order cannot be inverted" — a later edit can still move the
    /// call, and measurement says the final state survives it (see
    /// `handle_dpi_changed`).
    pending_scale_change: Option<GeometryProgress>,

    // Event callbacks set by the host before wasamo_run().
    //
    // **Every coordinate below is DIP** (M4-Phase 1; DD-M4-P1-004 fixes DIP as
    // the unit of every outward-facing length, and these are `pub` fields on a
    // `pub use`-exported type). `wnd_proc` divides the message's physical
    // values by this window's scale before invoking them, so a host receives
    // the same numbers its `.ui` dimensions are written in. Recorded rather
    // than inherited: no ABI or Rust-native function installs any of these
    // today, so the unit would otherwise have changed as a silent side effect
    // of moving the conversion seam.
    //
    // The pointer slots are `f32` for the reason `hit_test_click` is: a DIP
    // pointer position is not an integer, and `i32` would have made the
    // declared unit nominal by truncating physical 50 at 150% to DIP 33.
    pub resize_fn: Option<Box<dyn FnMut(f32, f32)>>,
    pub key_down_fn: Option<Box<dyn FnMut(u16)>>,
    pub mouse_down_fn: Option<Box<dyn FnMut(f32, f32)>>,
    pub mouse_move_fn: Option<Box<dyn FnMut(f32, f32)>>,
    pub mouse_leave_fn: Option<Box<dyn FnMut()>>,
    pub mouse_up_fn: Option<Box<dyn FnMut(f32, f32)>>,

    // Tracks whether TrackMouseEvent has been called for the current enter/leave cycle.
    tracking_mouse: bool,

    // Owned widget tree installed via `wasamo_window_set_root`. When set,
    // wnd_proc auto-routes WM_SIZE / mouse events to it.
    pub root_widget: Option<Box<WidgetNode>>,
    // Last reported mouse-down state, for hover/press routing through `root_widget`.
    mouse_down: bool,
    /// This window's retained hover/press record (M4-Phase 2 T4;
    /// `docs/architecture.md` §13.2). Reset by `set_root` (a previous root's
    /// indices mean nothing in the new tree) and otherwise owned entirely by
    /// `WidgetNode::update_hover` / `WidgetNode::clear_hover`, which the four
    /// pointer arms below call with `&mut self.hover`.
    ///
    /// `pub(crate)` rather than private, matching `scale` above: the
    /// integration-test crate's `lib.rs::ffi::__hover_target_for_test` seam
    /// needs to read it, and no host does — DD-M4-P1-004 keeps state like
    /// this off the `pub use`-exported surface.
    pub(crate) hover: HoverState,
    /// This window's per-window focus record (M4-Phase 2 T5;
    /// `docs/architecture.md` §13.3 "Focus is per window. A `WindowState`
    /// owns one focus record..." — DD-M4-P2-003's L2). One record per
    /// window, no global and no static, which is what leaves M4-Phase 8's
    /// second window an additive change rather than a rebuild — the same
    /// argument `scale` above carries for the DPI factor. Reset by
    /// `set_root` for the same reason `hover` is, and otherwise owned
    /// entirely by `crate::focus`'s operations, which the `WM_KEYDOWN` arm
    /// and the `WM_LBUTTONUP` arm below call with `&mut self.focus`.
    ///
    /// `pub(crate)` rather than private, matching `hover` above: the
    /// integration-test crate's `lib.rs::ffi::__focus_path_for_test` seam
    /// needs to read it, and no host does.
    pub(crate) focus: WindowFocus,
}

// Safety: same single-thread contract as Runtime.
unsafe impl Send for WindowState {}
unsafe impl Sync for WindowState {}

pub fn create(title: &str, width: i32, height: i32) -> windows::core::Result<Box<WindowState>> {
    let hwnd = create_hwnd(title, width, height)?;
    // Read once, from the monitor the OS actually placed the window on, and
    // used twice: to realise the requested DIP size below, and as this
    // window's factor for the rest of its life.
    let scale = DipScale::from_dpi(unsafe { GetDpiForWindow(hwnd) });
    realize_dip_window_size(hwnd, scale, width, height);
    apply_mica(hwnd);
    let compositor = &runtime::get().compositor;
    let target = create_desktop_window_target(compositor, hwnd)?;
    let root = compositor.CreateContainerVisual()?;
    root.cast::<Visual>()?
        .SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
    target.SetRoot(&root.cast::<Visual>()?)?;
    let mut state = Box::new(WindowState {
        hwnd,
        root,
        _target: target,
        scale,
        pending_scale_change: None,
        resize_fn: None,
        key_down_fn: None,
        mouse_down_fn: None,
        mouse_move_fn: None,
        mouse_leave_fn: None,
        mouse_up_fn: None,
        tracking_mouse: false,
        root_widget: None,
        mouse_down: false,
        hover: HoverState::default(),
        focus: WindowFocus::default(),
    });
    // Store a raw pointer to WindowState in GWLP_USERDATA so wnd_proc can reach it.
    // Safety: state is heap-allocated (Box) and will outlive the HWND.
    let ptr = &mut *state as *mut WindowState as isize;
    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr) };
    crate::emit::register_window(&mut *state as *mut WindowState);
    Ok(state)
}

fn create_hwnd(title: &str, width: i32, height: i32) -> windows::core::Result<HWND> {
    let class_name: Vec<u16> = "WasamoWindow\0".encode_utf16().collect();
    let title_w: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
        lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    unsafe { RegisterClassExW(&wc) };

    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_NOREDIRECTIONBITMAP, // required for Visual Layer + DWM backdrop (Mica)
            windows::core::PCWSTR(class_name.as_ptr()),
            windows::core::PCWSTR(title_w.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            width,
            height,
            None,
            None,
            None,
            None,
        )?
    };
    Ok(hwnd)
}

/// Resize the freshly-created window from the DIP size the caller asked for to
/// its physical equivalent.
///
/// `wasamo_window_create`'s `width` / `height` are DIP of the **outer window
/// rectangle** (DD-M4-P1-004), but `CreateWindowExW` interprets them as
/// physical pixels once the process is DPI-aware — and the monitor, hence the
/// DPI, is not knowable until the window exists, because placement is
/// `CW_USEDEFAULT`. So the window is created at the requested numbers and
/// corrected here (DD-M4-P1-003 §Initial scale acquisition, option I1).
///
/// **This belongs to `window::create`, not to `wasamo_window_create`.**
/// `create` has three callers — the ABI entry point, `wasamo_load_ui` (which
/// creates its own 800 x 600 DIP window and never goes through that entry
/// point), and `lib.rs::window_create`. A correction one level up would leave
/// every `.ui`-loaded window — that is, all three example hosts — at the wrong
/// physical size.
///
/// **Unconditional, with no `scale != 1` guard.** DD-M4-P1-001's tolerance of
/// a failed awareness declaration rests on the conversion machinery having no
/// second code path to keep correct; a guard here would be a branch that could
/// not be fired by any test until the declaration landed at T9, on the path
/// every host takes. It is reachable now, and the reason not to add it is
/// unchanged: it would be the second code path the tolerance argument denies.
///
/// **Placed before `WindowState` is boxed and before the `GWLP_USERDATA`
/// pointer is installed.** `SetWindowPos` dispatches window messages
/// *synchronously, before it returns* — the property DD-M4-P1-003 makes
/// load-bearing for `WM_DPICHANGED`. Measured here, on a window whose size the
/// correction actually changes: `WM_WINDOWPOSCHANGING`, `WM_GETMINMAXINFO`,
/// `WM_NCCALCSIZE`, `WM_WINDOWPOSCHANGED`, **`WM_SIZE`**, then `WM_GETICON`.
/// Running the correction here means `wnd_proc` reads a null `GWLP_USERDATA`
/// for every one of them and hands them to `DefWindowProcW`, so the nested
/// dispatch **cannot reach a half-built `WindowState` at all**. That is a
/// structural guarantee rather than the accident that the arms it would
/// otherwise enter happen to be no-ops today: the `WM_SIZE` arm acquires a
/// division by this window's scale at the conversion seams, and it is the wrong
/// arm to enter with no root widget installed and no emit registration yet
/// made. **The claim is exactly that, and not "a null pointer makes `wnd_proc`
/// inert"**: `WM_DESTROY` and `WM_ERASEBKGND` are handled *above* the null
/// check, and the first calls `PostQuitMessage` (T4 independent review finding
/// R-5). Neither appears in the measured set, so this correction is safe — but
/// the safety of the two arms above the check rests on the message set, not on
/// the pointer. Note that at a scale of 1 the size does not change and **no
/// `WM_SIZE` is dispatched at all**, which is why this placement was
/// unverifiable while the process was still undeclared, and why it was decided
/// structurally rather than by observing that nothing went wrong. T9's
/// declaration makes the correction change the size on any scaled monitor, so
/// the nested dispatch is now reachable — but **no test observes it**:
/// `resize_fn` can only be installed after `create` returns, so the
/// creation-time nested pass has no witness by construction. The structural
/// argument is still what carries this, not a measurement. The
/// `WM_DPICHANGED` handler's ordering obligation is the opposite case and must
/// be derived on its own terms: there the window is fully built and the nested
/// `WM_SIZE` is *required* to run the re-layout.
///
/// The flags are **not** that handler's either. It applies an OS-suggested
/// rectangle and therefore moves the window on purpose; here the placement is
/// `CW_USEDEFAULT`'s choice and must survive, so `SWP_NOMOVE` is required and
/// the `x` / `y` arguments are ignored.
///
/// A failure leaves the `CreateWindowExW` rectangle uncorrected — which is not
/// a reason to fail window creation, but is a reason to say so. What that
/// rectangle then *means* depends on the process's effective awareness and is
/// deliberately not asserted here: an aware process gets a window that is too
/// small by the scale factor, while an unaware one has its logical rectangle
/// stretched by DWM and looks approximately right. DD-M4-P1-003 §Failure
/// handling is "log
/// **and** survive", and the
/// runtime's diagnostic channel is the `wasamo:`-prefixed `eprintln!` the
/// handler, IR loader and reactive engine already use; swallowing this would
/// leave a mis-sized window on a scaled monitor with nothing to attribute it to.
fn realize_dip_window_size(hwnd: HWND, scale: DipScale, width: i32, height: i32) {
    let (physical_width, physical_height) = scale.window_size_to_physical((width, height));
    let result = unsafe {
        SetWindowPos(
            hwnd,
            None,
            0,
            0,
            physical_width,
            physical_height,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
    };
    if let Err(e) = result {
        eprintln!(
            "wasamo: could not realise the requested {width}x{height} DIP window size as \
             {physical_width}x{physical_height} physical (scale {}): {e}. The original \
             CreateWindowExW rectangle remains uncorrected.",
            scale.factor()
        );
    }
}

fn create_desktop_window_target(
    compositor: &windows::UI::Composition::Compositor,
    hwnd: HWND,
) -> windows::core::Result<DesktopWindowTarget> {
    use windows::Win32::System::WinRT::Composition::ICompositorDesktopInterop;
    let interop: ICompositorDesktopInterop = compositor.cast()?;
    unsafe { interop.CreateDesktopWindowTarget(hwnd, false) }
}

pub fn show(state: &WindowState) {
    unsafe {
        let _ = ShowWindow(state.hwnd, SW_SHOW);
    };
}

/// Install `root` as the window's content tree, taking ownership of the
/// subtree. A previously-installed root is detached and dropped after
/// disconnecting any registry entries it held. Performs an initial
/// layout pass against the window's current client size.
pub fn set_root(state: &mut WindowState, mut root: Box<WidgetNode>) -> windows::core::Result<()> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    // Prepare the new content tree before detaching the old one. A failure
    // preserves the installed root. The incoming Box is still consumed by
    // this call and is dropped on failure; the experimental C ABI's
    // ownership-on-error contract is tracked separately rather than described
    // here as caller-recoverable.
    let runtime = crate::runtime::get();
    root.refresh_text_surfaces_recursive(&runtime.compositor, &runtime.text_renderer, state.scale)?;

    if let Some(prev) = state.root_widget.take() {
        prev.for_each_ptr(&mut |p| {
            crate::registry::remove_for_widget(p as *mut crate::abi::WasamoWidget);
        });
        // Detach the previous root visual from the container.
        let prev_visual: Visual = prev.visual.cast()?;
        let _ = state.root.Children()?.Remove(&prev_visual);
        drop(prev);
    }

    let child_visual: Visual = root.visual.cast()?;
    state.root.Children()?.InsertAtTop(&child_visual)?;

    // Initial layout against current client size.
    //
    // DD-M4-P1-002 audit row 2, the inbound seam. `GetClientRect` reports
    // physical pixels and the layout engine works in DIP, so the extent is
    // divided by this window's scale before it reaches `run_layout_as_window_root`
    // — through `pair_to_dip` rather than a hand-written division, so the one
    // place the conversion rule lives stays one place (T2 finding F-15).
    let mut rect = RECT::default();
    let (cw, ch) = unsafe {
        if GetClientRect(state.hwnd, &mut rect).is_ok() {
            state.scale.pair_to_dip((
                (rect.right - rect.left) as f32,
                (rect.bottom - rect.top) as f32,
            ))
        } else {
            (0.0, 0.0)
        }
    };
    state.root_widget = Some(root);
    // The previous root (if any) is already dropped above; its indices name
    // nothing in the new tree, so a stale retained path must not survive
    // the swap (M4-Phase 2 T4). The focus record carries the identical
    // hazard (M4-Phase 2 T5) and is reset here for the same reason.
    state.hover = HoverState::default();
    state.focus = WindowFocus::default();
    if let Some(r) = state.root_widget.as_mut() {
        let _ = r.run_layout_as_window_root_at_scale(cw, ch, state.scale);
    }
    Ok(())
}

// Try Win11 22H2+ public API first; fall back to Win11 21H2 private attribute.
// Silently no-ops on Windows 10.
fn apply_mica(hwnd: HWND) {
    // Must be set before (or alongside) the backdrop type so DWM renders the
    // correct Mica tone. Not setting this causes Windows to default to the
    // light-mode Mica surface even when the system is in dark mode.
    apply_dark_mode(hwnd);

    let backdrop = DWMSBT_MAINWINDOW;
    let ok = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop as *const DWM_SYSTEMBACKDROP_TYPE as *const _,
            std::mem::size_of::<DWM_SYSTEMBACKDROP_TYPE>() as u32,
        )
        .is_ok()
    };
    if !ok {
        // Win11 21H2 (Build 22000–22522): private DWMWA_MICA_EFFECT attribute.
        const DWMWA_MICA_EFFECT: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(1029);
        let enabled: u32 = 1;
        let _ = unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_MICA_EFFECT,
                &enabled as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            )
        };
    }
}

// Read the system apps theme and tell DWM to render the matching Mica tone.
// DWMWA_USE_IMMERSIVE_DARK_MODE controls whether DWM draws the dark or light
// variant of the non-client area and backdrop material.
fn apply_dark_mode(hwnd: HWND) {
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Graphics::Dwm::DWMWA_USE_IMMERSIVE_DARK_MODE;
    use windows::UI::ViewManagement::{UIColorType, UISettings};

    let dark: BOOL = UISettings::new()
        .and_then(|s| s.GetColorValue(UIColorType::Background))
        .map(|c| c.R < 128) // near-black background → dark mode
        .unwrap_or(false)
        .into();

    let _ = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark as *const BOOL as *const _,
            std::mem::size_of::<BOOL>() as u32,
        )
    };
}

// ── WM_DPICHANGED (DD-M4-P1-003) ─────────────────────────────────────────────

/// Whether DD-M4-P1-003 **step 3** — a whole-tree geometry projection at the
/// newly committed scale — has succeeded yet.
///
/// The reason this is a value rather than a `bool` at the call site is that the
/// question is asked twice about two different attempts (the re-entrant
/// `WM_SIZE`, then the fallback) and answered once for step 4. The two
/// unsuccessful states are distinguished for the diagnostic only: both lead to
/// the fallback, and the final message reports which one preceded it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GeometryProgress {
    /// No projection has run. What a `SetWindowPos` that dispatched no
    /// `WM_SIZE` leaves behind — which includes the case where it *succeeded*
    /// without changing the size, and the case where it failed.
    NotProjected,
    /// A projection ran and failed. A further attempt is still made, because a
    /// different client extent may resolve.
    Failed,
    /// The whole tree is projected and its per-node geometry cache is
    /// committed at the new scale.
    Projected,
}

// The OS-bound calls stay in the unsafe handler helpers. Immediately after a
// call, its `windows::core::Error` is rendered to an owned summary; everything
// below that boundary is ordinary Rust data and can be unit-tested without a
// Win32/WinRT FFI dependency. The pure functions own both the failure decision
// and dispatch to a caller-supplied diagnostic sink, so trap #4 tests the
// branch that emits rather than only a string builder beside it.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RectangleSnapshot {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

fn os_error_summary(result: &windows::core::Result<()>) -> Option<String> {
    result.as_ref().err().map(ToString::to_string)
}

fn emit_runtime_diagnostic(diagnostic: &str) {
    eprintln!("{diagnostic}");
}

/// Complete the post-`SetWindowPos` decision. Step 3's verdict does not depend
/// on it: the fallback covers a window that kept its previous rectangle.
fn finish_rectangle_application(
    failure: Option<&str>,
    r: RectangleSnapshot,
    mut emit: impl FnMut(&str),
) {
    if let Some(e) = failure {
        let diagnostic = format!(
            "wasamo: could not apply the OS-suggested window rectangle {}x{} at ({}, {}) for a \
             DPI change: {e}. The window keeps its previous rectangle; the widget tree is \
             still re-projected at the new scale.",
            r.right - r.left,
            r.bottom - r.top,
            r.left,
            r.top
        );
        emit(&diagnostic);
    }
}

/// Return the client extent the fallback should project, or emit the failed
/// read's diagnostic and return its verdict.
///
/// The subtraction is here rather than at the call site so that the one piece
/// of arithmetic on this path is covered beside the failure arm.
fn client_extent_or_failure(
    failure: Option<&str>,
    rect: RectangleSnapshot,
    dpi: u32,
    mut emit: impl FnMut(&str),
) -> Result<(f32, f32), GeometryProgress> {
    if let Some(e) = failure {
        let diagnostic = format!(
            "wasamo: could not read the client rectangle to re-project the widget tree \
             at {dpi} DPI: {e}. No geometry pass ran for this change."
        );
        emit(&diagnostic);
        return Err(GeometryProgress::Failed);
    }
    Ok((
        (rect.right - rect.left) as f32,
        (rect.bottom - rect.top) as f32,
    ))
}

/// Complete the post-step-4 decision and emit the failure consequence.
fn finish_text_refresh(failure: Option<&str>, dpi: u32, mut emit: impl FnMut(&str)) {
    if let Some(e) = failure {
        let diagnostic = format!(
            "wasamo: could not re-rasterize every text surface at {dpi} DPI: {e}. The geometry \
             is at the new scale; the surfaces that failed stay at their previous resolution \
             and are retried on the next pass."
        );
        emit(&diagnostic);
    }
}

impl GeometryProgress {
    /// The invariant step 4 hangs off: **text surfaces may be replaced and
    /// raster markers advanced only once a whole-tree projection has
    /// succeeded.**
    ///
    /// A `raster_scale` that moves past a geometry pass which did not happen
    /// claims a convergence the Visual tree does not have — the failure the T6
    /// independent review rejected in its shared-cache form, which reaches this
    /// handler through the message loop instead. Asked twice: once to decide
    /// whether the fallback is required, once to decide whether step 4 may run.
    fn whole_tree_projected(self) -> bool {
        matches!(self, Self::Projected)
    }
}

/// DD-M4-P1-003's `WM_DPICHANGED` propagation, in the ADR's fixed order.
///
/// **Called from above `wnd_proc`'s `&mut *state_ptr` block, deliberately.**
/// `SetWindowPos` dispatches `WM_SIZE` before it returns, so this is the first
/// place in the runtime where `wnd_proc` re-enters with `GWLP_USERDATA`
/// installed — T4's creation-time correction dispatches its nested messages
/// while the pointer is still null. The nested frame needs its own
/// `&mut WindowState` to lay out; what must not exist is an outer one alive at
/// the same time, so every access here is short-lived and none spans step 2.
/// Handling this message inside that block would have been sound only by the
/// accident that the aliasing does not currently miscompile.
///
/// **Why step 1 precedes step 2.** The nested `WM_SIZE` divides the client
/// extent by `WindowState::scale` and projects the tree at it, so a scale
/// committed after `SetWindowPos` would leave that pass laying out and
/// projecting with the previous factor — one visibly wrong frame, and the
/// phase's single most likely ordering defect. `begin_scale_change` writes the
/// scale and installs the marker together, and the marker is what tells the
/// nested pass it is step 3, so an edit that moved the call after
/// `SetWindowPos` would leave that pass unrecognised, hence unreported, hence
/// re-done by the fallback at the committed scale.
///
/// **What that is worth is measured, and it is less than "the order cannot be
/// got wrong".** Inverting the two calls leaves all four integration tests
/// green, because the *final* state converges either way. What the inversion
/// demonstrably still produces is a **geometry write at the stale factor and a
/// text refresh at the stale DPI**, both overwritten a moment later by the
/// fallback. Whether that intermediate state is ever *presented* was not
/// measured and is not claimed: the whole handler runs inside one message, and
/// Composition commits on the dispatcher tick, so the compositor may never see
/// it. DD-M4-P1-003 predicts "visibly wrong for one frame at best"; this task
/// establishes the wrong intermediate state, not the frame. So the structure
/// turns the defect from **persistent** into **transient** — no more than that.
///
/// It was hard to observe at all until the process declared awareness: the OS
/// does not deliver this message for the per-monitor changes this phase is
/// about until then, and at a scale of 1 `SetWindowPos` dispatches no `WM_SIZE`
/// at all (T4, measured). Not that an undeclared process was categorically
/// unreachable — Microsoft documents delivery to unaware and system-aware
/// top-level windows in some cases. T9's declaration makes the OS a real
/// driver of this path, but it does not turn the transient window into
/// something a test can catch, so the ordering stays argued structurally
/// rather than from "nothing currently goes wrong".
///
/// Failure handling is **log and survive** throughout, per DD-M4-P1-003: no
/// path here tears down the window, none reports a new error to `wasamo_run`,
/// and the runtime is never put into `Diverged` — that state is for
/// reactive-engine divergence, and nothing in a scale change enters the
/// reactive drain. The caller returns `LRESULT(0)` regardless (step 5).
///
/// `WM_GETDPISCALEDSIZE` is not handled this phase. V2 delivers it *before* the
/// change so a process can propose its own new size, and Wasamo has no reason
/// to override a suggestion that already preserves logical size. Forward
/// exposure, not an omission.
unsafe fn handle_dpi_changed(
    hwnd: HWND,
    state_ptr: *mut WindowState,
    wparam: WPARAM,
    lparam: LPARAM,
) {
    // Step 1. `HIWORD(wParam)` is the new Y-axis DPI; Windows reports the same
    // value in both halves, and DD-M4-P1-003 names this one.
    let new_scale = DipScale::from_dpi(((wparam.0 >> 16) & 0xFFFF) as u32);
    begin_scale_change(state_ptr, new_scale);

    // Step 2.
    apply_suggested_rectangle(hwnd, lparam);

    // Step 3. Close the observation window and read what the re-entrant pass
    // reported. `None` cannot occur — step 1 installed the marker and nothing
    // else takes it — and is read as "nothing was reported" rather than
    // asserted, because a panic unwinding out of a window procedure is not a
    // failure mode worth introducing to diagnose an unreachable one.
    let nested = (*state_ptr)
        .pending_scale_change
        .take()
        .unwrap_or(GeometryProgress::NotProjected);
    let progress = if nested.whole_tree_projected() {
        nested
    } else {
        project_current_client_extent(state_ptr)
    };

    // Step 4.
    if progress.whole_tree_projected() {
        refresh_text_at_new_scale(state_ptr);
    } else {
        eprintln!(
            "wasamo: no geometry pass succeeded for the change to {} DPI (step 3: \
             {nested:?}, then the fallback also failed), so the text surfaces are \
             left at their previous resolution and every geometry cache and raster \
             marker stays stale. The Visual tree may be partially updated, because \
             the projection writes Visuals as it walks and commits caches only \
             after the whole walk succeeds. The next resize or size-affecting \
             property write retries the whole tree.",
            new_scale.dpi()
        );
    }
}

/// Step 1: commit the new scale **and** open the nested-pass observation, in one
/// place.
///
/// The two writes are here together rather than at the call site so that no
/// path can install the marker without having committed the scale — the marker
/// is the nested pass's only signal that it is step 3, and a marker installed
/// against a stale scale is the ordering defect this function exists to make
/// unconstructible.
unsafe fn begin_scale_change(state_ptr: *mut WindowState, new_scale: DipScale) {
    let state = &mut *state_ptr;
    state.scale = new_scale;
    state.pending_scale_change = Some(GeometryProgress::NotProjected);
}

/// Step 2: apply the OS-suggested **physical** rectangle.
///
/// Applying it rather than ignoring it is what preserves the window's *logical*
/// size across the change, which is what the DIP contract means; ignoring it
/// would give a window that grows and shrinks as it crosses monitors.
///
/// **The flags are not `window::realize_dip_window_size`'s, and that helper is
/// not reusable here** (T4 finding F-30). It converts a DIP *size* and must
/// leave `CW_USEDEFAULT`'s placement alone, so it passes `SWP_NOMOVE`. This
/// rectangle's whole content is a new position *and* size, so `SWP_NOMOVE`
/// would pin the window and defeat the suggestion on every monitor crossing —
/// while every test stayed green. The two sites sit either side of the same
/// mistake.
///
/// `lParam` is a raw `RECT*` supplied by the message, and `wnd_proc` is
/// reachable by `SendMessageW` from any process — so a null is a reachable
/// input, not a hypothetical one, and dereferencing it is the one failure in
/// this handler that would not survive. Skipping step 2 leaves the scale
/// already committed and the fallback carrying the change, so a malformed
/// message still converges.
unsafe fn apply_suggested_rectangle(hwnd: HWND, lparam: LPARAM) {
    let suggested = lparam.0 as *const windows::Win32::Foundation::RECT;
    if suggested.is_null() {
        eprintln!(
            "wasamo: WM_DPICHANGED carried a null suggested rectangle, so the window \
             rectangle is left unchanged. The new scale is already committed and the \
             widget tree is re-projected from the current client rectangle instead."
        );
        return;
    }
    let r = *suggested;
    let result = SetWindowPos(
        hwnd,
        None,
        r.left,
        r.top,
        r.right - r.left,
        r.bottom - r.top,
        SWP_NOZORDER | SWP_NOACTIVATE,
    );
    let failure = os_error_summary(&result);
    finish_rectangle_application(
        failure.as_deref(),
        RectangleSnapshot {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        },
        emit_runtime_diagnostic,
    );
}

/// Step 3, fallback: project the whole tree from the **current** client
/// rectangle at the committed scale.
///
/// Required whenever `SetWindowPos` produced no successful re-entrant
/// projection — which is not only the failure case. A `SetWindowPos` that
/// succeeds *without changing the size* dispatches no `WM_SIZE` at all (T4,
/// measured), and the DIP client extent has still changed, because the same
/// physical extent divided by a new scale is a new DIP extent. Without this,
/// the authoritative scale would move while every derived geometry cache stayed
/// behind it.
///
/// The extent is read physical and converted through the **committed** scale,
/// and the same scale is passed as the projection target, so the divisor and
/// the multiplier are one value rather than two that happen to agree.
unsafe fn project_current_client_extent(state_ptr: *mut WindowState) -> GeometryProgress {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    let state = &mut *state_ptr;
    let target = state.scale;
    let hwnd = state.hwnd;

    let mut rect = RECT::default();
    let read = GetClientRect(hwnd, &mut rect);
    let failure = os_error_summary(&read);
    let physical = match client_extent_or_failure(
        failure.as_deref(),
        RectangleSnapshot {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        },
        target.dpi(),
        emit_runtime_diagnostic,
    ) {
        Ok(extent) => extent,
        Err(verdict) => return verdict,
    };
    let (w, h) = target.pair_to_dip(physical);

    match state.root_widget.as_mut() {
        // A window with no content tree has nothing to project, so the change
        // has converged. Reported as success rather than as a failure with no
        // cause, and it lets step 4 no-op instead of logging.
        None => GeometryProgress::Projected,
        Some(root) => match root.run_layout_as_window_root_at_scale(w, h, target) {
            Ok(()) => GeometryProgress::Projected,
            Err(e) => {
                eprintln!(
                    "wasamo: could not re-project the widget tree at {} DPI from the \
                     current {w}x{h} DIP client extent: {e}.",
                    target.dpi()
                );
                GeometryProgress::Failed
            }
        },
    }
}

/// Step 4: re-rasterize the text surfaces whose installed DPI is no longer the
/// window's.
///
/// Reached only behind [`GeometryProgress::whole_tree_projected`]. Safe to run
/// after step 3 rather than before it because `measure` is DIP and
/// scale-invariant, so replacing a surface cannot change any node's
/// `SizeConstraint::Fixed` pair and cannot invalidate the layout step 3 just
/// computed. That is a property of today's text stack, not a free ordering: an
/// explicitly-hinted or snapped `measure` would make this a correctness
/// constraint to re-derive (DD-M4-P1-003 §Forward-compat exposure 6).
///
/// A failure here leaves the failed node's surface at the old resolution —
/// visibly blurry, honest about it, and retryable, because the raster marker
/// advances per node only after its brush is installed and geometry has already
/// converged at the new scale.
unsafe fn refresh_text_at_new_scale(state_ptr: *mut WindowState) {
    let state = &mut *state_ptr;
    let target = state.scale;
    let Some(root) = state.root_widget.as_mut() else {
        return;
    };
    let runtime = runtime::get();
    let refreshed =
        root.refresh_text_surfaces_recursive(&runtime.compositor, &runtime.text_renderer, target);
    let failure = os_error_summary(&refreshed);
    finish_text_refresh(failure.as_deref(), target.dpi(), emit_runtime_diagnostic);
}

/// The **physical** client-area pointer position packed into a mouse message's
/// `lParam`, sign-extended per axis.
///
/// Hoisted at M4-Phase 1 T5 because the conversion seam would otherwise have
/// divided this expression at three sites that each wrote it out again — the
/// same rule-in-two-places shape T2 finding F-14 records. The signed
/// extraction is deliberate and is *not* shared with `WM_SIZE`: a pointer
/// coordinate is negative when the cursor leaves the client area to the left or
/// above, while a client extent is unsigned.
fn pointer_physical(lparam: LPARAM) -> (f32, f32) {
    (
        (lparam.0 & 0xFFFF) as i16 as f32,
        ((lparam.0 >> 16) & 0xFFFF) as i16 as f32,
    )
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Retrieve the WindowState pointer stored at creation time.
    // Zero means the window hasn't been fully initialized yet (early WM_CREATE etc.).
    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowState;

    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return LRESULT(0);
    }

    // Prevent GDI from painting an opaque background over the DWM backdrop.
    if msg == WM_ERASEBKGND {
        return LRESULT(1);
    }

    // Handled **above** the `&mut *state_ptr` block below, not inside it: its
    // `SetWindowPos` re-enters this procedure synchronously, and an outer
    // `&mut WindowState` alive across that call would alias the nested frame's.
    // See `handle_dpi_changed`. With no state installed there is no scale to
    // update and nothing to project, so the message falls through to
    // `DefWindowProcW`.
    if msg == WM_DPICHANGED && !state_ptr.is_null() {
        handle_dpi_changed(hwnd, state_ptr, wparam, lparam);
        return LRESULT(0);
    }

    if !state_ptr.is_null() {
        let state = &mut *state_ptr;
        // This window's conversion factor, copied out once so the arms below
        // can hold a `&mut` borrow of a callback slot at the same time.
        // DD-M4-P1-002 audit rows 1 and 3 both divide by it: the client extent
        // and the pointer stream arrive in the client area's physical pixels
        // and everything downstream of this procedure — layout, hit-testing,
        // hover, and the host callbacks — is DIP.
        let scale = state.scale;

        if msg == WM_SIZE {
            let (w, h) = scale.pair_to_dip((
                (lparam.0 & 0xFFFF) as f32,
                ((lparam.0 >> 16) & 0xFFFF) as f32,
            ));
            if let Some(f) = &mut state.resize_fn {
                f(w, h);
            }
            // Is this DD-M4-P1-003 step 3 — the pass re-entered from inside the
            // `WM_DPICHANGED` handler's `SetWindowPos` — or an ordinary resize?
            // The marker's presence decides two things, and its value is how
            // this pass reports back.
            let scale_change = state.pending_scale_change.is_some();
            let mut progress = GeometryProgress::Projected;
            if let Some(root) = state.root_widget.as_mut() {
                progress = match root.run_layout_as_window_root_at_scale(w, h, scale) {
                    Ok(()) => GeometryProgress::Projected,
                    Err(e) => {
                        // Logged only on the scale-change path: the ordinary
                        // resize has swallowed this Result since M3-Phase 2 and
                        // this task is not the place to start reporting it, but
                        // a scale change that fails to project is the one thing
                        // the handler's final diagnostic is about.
                        if scale_change {
                            eprintln!(
                                "wasamo: the re-entrant WM_SIZE of a DPI change could not \
                                 project the widget tree at {} DPI over a {w}x{h} DIP client \
                                 extent: {e}.",
                                scale.dpi()
                            );
                        }
                        GeometryProgress::Failed
                    }
                };
                // Under a scale change the target has moved, so refreshing here
                // would advance every raster marker to the new DPI whether or
                // not the projection that should accompany them succeeded. Step
                // 4 owns it, after `SetWindowPos` returns and behind a succeeded
                // projection. The ordinary-resize call is pre-existing and
                // deliberately outside T7's convergence argument: it may retry
                // a failed rasterization or prepare a newly attached node, and
                // it remains unconditional even if that resize's geometry pass
                // failed. Only the scale-change path claims that raster
                // freshness follows a successful target-scale projection.
                if !scale_change {
                    let runtime = crate::runtime::get();
                    let _ = root.refresh_text_surfaces_recursive(
                        &runtime.compositor,
                        &runtime.text_renderer,
                        scale,
                    );
                }
            }
            if scale_change {
                // Reports the projection's **outcome**, not that this arm ran:
                // entry into `WM_SIZE` is not the fact step 4 needs.
                state.pending_scale_change = Some(progress);
            }
            return LRESULT(0);
        }

        if msg == WM_KEYDOWN {
            let vk = wparam.0 as u16;
            // The state `Tab` / `Shift+Tab` need. `GetKeyState` reports the
            // key state as of this message's position in the input queue,
            // and the high bit of the returned `i16`, read as `u16`, is set
            // while the key is held (Win32 `GetKeyState` contract).
            let shift_down = (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;
            // With no root widget installed there is no tree to traverse,
            // so `Tab` is not claimed and falls through with every other
            // key. That is a narrower state than "a tree with no focus
            // stop", which `traverse_on_key` does claim: a window between
            // `create` and `set_root` has no content at all, and there is
            // nothing for traversal to be the recipient of.
            if let Some(root) = state.root_widget.as_mut() {
                if focus::traverse_on_key(
                    root,
                    &runtime::get().compositor,
                    &mut state.focus,
                    vk,
                    shift_down,
                ) {
                    // Traversal consumed the key.
                    return LRESULT(0);
                }
            }
            // Routing consumes ahead of the host key slot without
            // installing it: the first installer would fix the callback
            // unit as shipped API (constraints.md §2), and this phase must
            // not be that installer.
            if let Some(f) = &mut state.key_down_fn {
                f(vk);
            }
            // No `return` here: a key nothing consumed is not swallowed
            // (docs/architecture.md §13.2) and must continue to
            // `DefWindowProcW` at the bottom of this procedure. Every `if`
            // below tests a different `msg` value than `WM_KEYDOWN`, so
            // control falls straight through to that call.
            //
            // `DefWindowProcW` can dispatch further messages into this same
            // procedure synchronously, and a nested frame will derive its
            // own `&mut WindowState` from `GWLP_USERDATA` — sound here
            // because nothing uses the outer `state` after the fallthrough
            // leaves this arm, which is a different argument from
            // `WM_DPICHANGED`'s: that handler is hoisted above the
            // `&mut *state_ptr` borrow because it *does* use `state`
            // afterwards.
        }

        if msg == WM_MOUSEMOVE {
            let (x, y) = scale.pair_to_dip(pointer_physical(lparam));
            if !state.tracking_mouse {
                // Request WM_MOUSELEAVE when the cursor leaves the client area.
                let mut tme = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                let _ = TrackMouseEvent(&mut tme);
                state.tracking_mouse = true;
            }
            if let Some(f) = &mut state.mouse_move_fn {
                f(x, y);
            }
            if let Some(root) = state.root_widget.as_mut() {
                let _ = root.update_hover(
                    &runtime::get().compositor,
                    &mut state.hover,
                    x,
                    y,
                    state.mouse_down,
                );
            }
            return LRESULT(0);
        }

        if msg == WM_MOUSELEAVE {
            state.tracking_mouse = false;
            if let Some(f) = &mut state.mouse_leave_fn {
                f();
            }
            if let Some(root) = state.root_widget.as_mut() {
                let _ = root.clear_hover(&runtime::get().compositor, &mut state.hover);
            }
            return LRESULT(0);
        }

        if msg == WM_LBUTTONDOWN {
            let (x, y) = scale.pair_to_dip(pointer_physical(lparam));
            state.mouse_down = true;
            if let Some(f) = &mut state.mouse_down_fn {
                f(x, y);
            }
            if let Some(root) = state.root_widget.as_mut() {
                let _ = root.update_hover(&runtime::get().compositor, &mut state.hover, x, y, true);
            }
            return LRESULT(0);
        }

        if msg == WM_LBUTTONUP {
            let (x, y) = scale.pair_to_dip(pointer_physical(lparam));
            state.mouse_down = false;
            if let Some(f) = &mut state.mouse_up_fn {
                f(x, y);
            }
            if let Some(root) = state.root_widget.as_mut() {
                // Deliberate order (T3's carry-forward to T4, extended by
                // T5). Focus moves first: a click focuses the widget the
                // user actually touched, and a handler's synchronous
                // rebuild can invalidate the resolved path, so the focus
                // write must precede dispatch (M4-Phase 2 T5,
                // DD-M4-P2-003). `focus_on_click` and `hit_test_click`
                // below both resolve the same point against the same,
                // still-unmutated tree — neither has run any handler yet —
                // so the two resolutions cannot disagree even though each
                // walks the tree separately.
                //
                // `hit_test_click` runs next, before `update_hover`, for
                // the reason T4 recorded: the release must dispatch
                // against the tree the user actually saw. Its handler can
                // rebuild or remove subtrees synchronously (see
                // `run_clicked_handlers`'s safety note); a newly
                // materialised node has no `arranged_rect` yet (T1's
                // store, refreshed only by the next layout pass), so it is
                // not a hit candidate and the `update_hover` immediately
                // below can resolve against a tree that has already moved
                // out from under it. That staleness is not corrected here
                // — it self-corrects on the next real pointer message,
                // after the message-loop boundary's drain
                // (`emit::drain_if_outermost`) has re-laid-out whatever the
                // handler rebuilt.
                let _ =
                    focus::focus_on_click(root, &runtime::get().compositor, &mut state.focus, x, y);
                root.hit_test_click(x, y);
                let _ =
                    root.update_hover(&runtime::get().compositor, &mut state.hover, x, y, false);
            }
            return LRESULT(0);
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

#[cfg(test)]
mod tests {
    use super::{
        client_extent_or_failure, finish_rectangle_application, finish_text_refresh,
        GeometryProgress, RectangleSnapshot,
    };

    const SUGGESTED: RectangleSnapshot = RectangleSnapshot {
        left: 100,
        top: 60,
        right: 1100,
        bottom: 810,
    };

    #[test]
    fn a_failed_rectangle_application_reports_the_rectangle_and_the_consequence() {
        let mut diagnostics = Vec::new();
        finish_rectangle_application(None, SUGGESTED, |diagnostic| {
            diagnostics.push(diagnostic.to_owned())
        });
        assert!(diagnostics.is_empty());

        finish_rectangle_application(Some("simulated device failure"), SUGGESTED, |diagnostic| {
            diagnostics.push(diagnostic.to_owned())
        });
        assert_eq!(diagnostics.len(), 1, "the failure must reach the sink once");
        let diagnostic = &diagnostics[0];
        assert!(diagnostic.contains("1000x750"), "{diagnostic}");
        assert!(diagnostic.contains("(100, 60)"), "{diagnostic}");
        assert!(
            diagnostic.contains("simulated device failure"),
            "the OS-bound error summary must survive the pure boundary: {diagnostic}"
        );
        assert!(
            diagnostic.contains("keeps its previous rectangle"),
            "an operator needs the window's resulting state, not just the failure: {diagnostic}"
        );
        assert!(
            diagnostic.contains("still re-projected"),
            "and needs to know the widget tree does converge anyway: {diagnostic}"
        );
    }

    #[test]
    fn a_failed_client_rect_read_yields_no_projection_and_says_so() {
        let mut diagnostics = Vec::new();
        let verdict = client_extent_or_failure(
            Some("simulated device failure"),
            SUGGESTED,
            120,
            |diagnostic| diagnostics.push(diagnostic.to_owned()),
        )
        .expect_err("a failed read cannot produce an extent");
        assert_eq!(
            verdict,
            GeometryProgress::Failed,
            "a client extent that could not be read means no geometry pass ran, which must \
             not permit step 4"
        );
        assert!(!verdict.whole_tree_projected());
        assert_eq!(diagnostics.len(), 1, "the failure must reach the sink once");
        let diagnostic = &diagnostics[0];
        assert!(diagnostic.contains("120 DPI"), "{diagnostic}");
        assert!(diagnostic.contains("No geometry pass ran"), "{diagnostic}");
        assert!(
            diagnostic.contains("simulated device failure"),
            "{diagnostic}"
        );
    }

    /// The success arm carries the fallback's only arithmetic, so it is pinned
    /// beside the failure arm rather than left to the integration tests.
    #[test]
    fn a_successful_client_rect_read_yields_the_physical_extent() {
        let mut diagnostics = Vec::new();
        let extent = client_extent_or_failure(None, SUGGESTED, 120, |diagnostic| {
            diagnostics.push(diagnostic.to_owned())
        })
        .expect("a successful read yields an extent");
        assert_eq!(extent, (1000.0, 750.0));
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn a_failed_text_refresh_reports_that_geometry_already_converged() {
        let mut diagnostics = Vec::new();
        finish_text_refresh(None, 120, |diagnostic| {
            diagnostics.push(diagnostic.to_owned())
        });
        assert!(diagnostics.is_empty());

        finish_text_refresh(Some("simulated device failure"), 120, |diagnostic| {
            diagnostics.push(diagnostic.to_owned())
        });
        assert_eq!(diagnostics.len(), 1, "the failure must reach the sink once");
        let diagnostic = &diagnostics[0];
        assert!(diagnostic.contains("120 DPI"), "{diagnostic}");
        assert!(
            diagnostic.contains("simulated device failure"),
            "{diagnostic}"
        );
        assert!(
            diagnostic.contains("geometry is at the new scale"),
            "the whole point of separating the markers is that this failure does not roll \
             geometry back, and the diagnostic has to say so: {diagnostic}"
        );
        assert!(
            diagnostic.contains("retried on the next pass"),
            "a stale surface is retryable, not permanent: {diagnostic}"
        );
    }

    /// The step-4 permission rule, pinned as pure logic.
    ///
    /// This exists because two of the three states are not producible from the
    /// OS on demand. `NotProjected` is reachable from a test (a `SetWindowPos`
    /// that changes no size dispatches no `WM_SIZE`), but a *failed* projection
    /// needs a layout error and a *succeeded* one needs a live Compositor — so
    /// the integration binary covers the paths it can reach and the rule itself
    /// is fired here over its whole input space. The proposition is the one the
    /// T6 independent review established: only a succeeded whole-tree
    /// projection permits a raster marker to advance.
    #[test]
    fn only_a_succeeded_projection_permits_step_four() {
        assert!(GeometryProgress::Projected.whole_tree_projected());
        assert!(!GeometryProgress::Failed.whole_tree_projected());
        assert!(!GeometryProgress::NotProjected.whole_tree_projected());
    }

    /// A `SetWindowPos` that succeeds without changing the size and one that
    /// fails outright leave the handler in the *same* state, and the fallback
    /// must fire for both. They are distinct variants only so the final
    /// diagnostic can say which happened, so the test that matters is that
    /// neither is mistaken for progress.
    #[test]
    fn neither_unsuccessful_state_is_progress() {
        for state in [GeometryProgress::NotProjected, GeometryProgress::Failed] {
            assert!(
                !state.whole_tree_projected(),
                "{state:?} must send the handler into the fallback"
            );
        }
        assert_ne!(GeometryProgress::NotProjected, GeometryProgress::Failed);
    }
}
