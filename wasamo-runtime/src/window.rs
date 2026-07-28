use crate::dip_scale::DipScale;
use crate::runtime;
use crate::widget::WidgetNode;
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
            Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, LoadCursorW, PostQuitMessage,
                RegisterClassExW, SetWindowLongPtrW, SetWindowPos, ShowWindow, CS_HREDRAW,
                CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, SWP_NOACTIVATE, SWP_NOMOVE,
                SWP_NOZORDER, SW_SHOW, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN,
                WM_LBUTTONUP, WM_MOUSEMOVE, WM_SIZE, WNDCLASSEXW, WS_EX_NOREDIRECTIONBITMAP,
                WS_OVERLAPPEDWINDOW,
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
    /// Written here and not yet read: the seams that divide and multiply by it
    /// land at T5, and the `WM_DPICHANGED` handler that rewrites it at T7. The
    /// allow is that forward pointer, in the same shape as `dip_scale`'s
    /// module-level one, and goes away with the first seam.
    #[allow(dead_code)]
    pub(crate) scale: DipScale,

    // Event callbacks set by the host before wasamo_run().
    pub resize_fn: Option<Box<dyn FnMut(f32, f32)>>,
    pub key_down_fn: Option<Box<dyn FnMut(u16)>>,
    pub mouse_down_fn: Option<Box<dyn FnMut(i32, i32)>>,
    pub mouse_move_fn: Option<Box<dyn FnMut(i32, i32)>>,
    pub mouse_leave_fn: Option<Box<dyn FnMut()>>,
    pub mouse_up_fn: Option<Box<dyn FnMut(i32, i32)>>,

    // Tracks whether TrackMouseEvent has been called for the current enter/leave cycle.
    tracking_mouse: bool,

    // Owned widget tree installed via `wasamo_window_set_root`. When set,
    // wnd_proc auto-routes WM_SIZE / mouse events to it.
    pub root_widget: Option<Box<WidgetNode>>,
    // Last reported mouse-down state, for hover/press routing through `root_widget`.
    mouse_down: bool,
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
        resize_fn: None,
        key_down_fn: None,
        mouse_down_fn: None,
        mouse_move_fn: None,
        mouse_leave_fn: None,
        mouse_up_fn: None,
        tracking_mouse: false,
        root_widget: None,
        mouse_down: false,
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
/// second code path to keep correct; a guard here would be a branch that no
/// test can fire until the declaration lands.
///
/// **Placed before `WindowState` is boxed and before the `GWLP_USERDATA`
/// pointer is installed.** `SetWindowPos` dispatches window messages
/// *synchronously, before it returns* — the property DD-M4-P1-003 makes
/// load-bearing for `WM_DPICHANGED`. Running the correction here means
/// `wnd_proc` reads a null `GWLP_USERDATA` and hands them to `DefWindowProcW`,
/// so the nested dispatch **cannot reach a half-built `WindowState` at all**.
/// That is a structural guarantee rather than the accident that the arms it
/// would otherwise enter happen to be no-ops today: the `WM_SIZE` arm acquires
/// a division by this window's scale at the conversion seams, and it is the
/// wrong arm to enter with no root widget installed and no emit registration
/// yet made. The `WM_DPICHANGED` handler's ordering obligation is the opposite
/// case and must be derived on its own terms — there the window is fully
/// built and the nested `WM_SIZE` is required to do the re-layout.
///
/// The flags are **not** that handler's either. It applies an OS-suggested
/// rectangle and therefore moves the window on purpose; here the placement is
/// `CW_USEDEFAULT`'s choice and must survive, so `SWP_NOMOVE` is required and
/// the `x` / `y` arguments are ignored.
///
/// A failure leaves the window at the requested numbers — wrong at any scale
/// but 100%, and not a reason to fail window creation (DD-M4-P1-003 §Failure
/// handling: log and survive).
fn realize_dip_window_size(hwnd: HWND, scale: DipScale, width: i32, height: i32) {
    let (physical_width, physical_height) = scale.window_size_to_physical((width, height));
    let _ = unsafe {
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
pub fn set_root(state: &mut WindowState, root: Box<WidgetNode>) -> windows::core::Result<()> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

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
    let mut rect = RECT::default();
    let (cw, ch) = unsafe {
        if GetClientRect(state.hwnd, &mut rect).is_ok() {
            (
                (rect.right - rect.left) as f32,
                (rect.bottom - rect.top) as f32,
            )
        } else {
            (0.0, 0.0)
        }
    };
    state.root_widget = Some(root);
    if let Some(r) = state.root_widget.as_mut() {
        let _ = r.run_layout_as_window_root(cw, ch);
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

    if !state_ptr.is_null() {
        let state = &mut *state_ptr;

        if msg == WM_SIZE {
            let w = (lparam.0 & 0xFFFF) as f32;
            let h = ((lparam.0 >> 16) & 0xFFFF) as f32;
            if let Some(f) = &mut state.resize_fn {
                f(w, h);
            }
            if let Some(root) = state.root_widget.as_mut() {
                let _ = root.run_layout_as_window_root(w, h);
            }
            return LRESULT(0);
        }

        if msg == WM_KEYDOWN {
            let vk = wparam.0 as u16;
            if let Some(f) = &mut state.key_down_fn {
                f(vk);
            }
            return LRESULT(0);
        }

        if msg == WM_MOUSEMOVE {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
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
                let _ = root.update_hover(&runtime::get().compositor, x, y, state.mouse_down);
            }
            return LRESULT(0);
        }

        if msg == WM_MOUSELEAVE {
            state.tracking_mouse = false;
            if let Some(f) = &mut state.mouse_leave_fn {
                f();
            }
            if let Some(root) = state.root_widget.as_mut() {
                let _ = root.clear_hover(&runtime::get().compositor);
            }
            return LRESULT(0);
        }

        if msg == WM_LBUTTONDOWN {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            state.mouse_down = true;
            if let Some(f) = &mut state.mouse_down_fn {
                f(x, y);
            }
            if let Some(root) = state.root_widget.as_mut() {
                let _ = root.update_hover(&runtime::get().compositor, x, y, true);
            }
            return LRESULT(0);
        }

        if msg == WM_LBUTTONUP {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            state.mouse_down = false;
            if let Some(f) = &mut state.mouse_up_fn {
                f(x, y);
            }
            if let Some(root) = state.root_widget.as_mut() {
                root.hit_test_click(x, y);
                let _ = root.update_hover(&runtime::get().compositor, x, y, false);
            }
            return LRESULT(0);
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}
