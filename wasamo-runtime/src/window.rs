use crate::runtime;
use crate::widget::WidgetNode;
use windows::{
    core::Interface,
    Foundation::Numerics::Vector2,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Dwm::{
            DwmSetWindowAttribute,
            DWMWA_SYSTEMBACKDROP_TYPE,
            DWMWINDOWATTRIBUTE, DWM_SYSTEMBACKDROP_TYPE, DWMSBT_MAINWINDOW,
        },
        UI::{
            Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, LoadCursorW,
                PostQuitMessage, RegisterClassExW, SetWindowLongPtrW, ShowWindow,
                CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW,
                SW_SHOW, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN,
                WM_LBUTTONUP, WM_MOUSEMOVE, WM_SIZE, WNDCLASSEXW,
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

    // Event callbacks set by the host before wasamo_run().
    pub resize_fn:      Option<Box<dyn FnMut(f32, f32)>>,
    pub key_down_fn:    Option<Box<dyn FnMut(u16)>>,
    pub mouse_down_fn:  Option<Box<dyn FnMut(i32, i32)>>,
    pub mouse_move_fn:  Option<Box<dyn FnMut(i32, i32)>>,
    pub mouse_leave_fn: Option<Box<dyn FnMut()>>,
    pub mouse_up_fn:    Option<Box<dyn FnMut(i32, i32)>>,

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
    apply_mica(hwnd);
    let compositor = &runtime::get().compositor;
    let target = create_desktop_window_target(compositor, hwnd)?;
    let root = compositor.CreateContainerVisual()?;
    root.cast::<Visual>()?.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
    target.SetRoot(&root.cast::<Visual>()?)?;
    let mut state = Box::new(WindowState {
        hwnd,
        root,
        _target: target,
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
            WS_EX_NOREDIRECTIONBITMAP,  // required for Visual Layer + DWM backdrop (Mica)
            windows::core::PCWSTR(class_name.as_ptr()),
            windows::core::PCWSTR(title_w.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT, CW_USEDEFAULT,
            width, height,
            None, None, None, None,
        )?
    };
    Ok(hwnd)
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
    unsafe { let _ = ShowWindow(state.hwnd, SW_SHOW); };
}

/// Install `root` as the window's content tree, taking ownership of the
/// subtree. A previously-installed root is detached and dropped after
/// disconnecting any registry entries it held. Performs an initial
/// layout pass against the window's current client size.
pub fn set_root(
    state: &mut WindowState,
    root: Box<WidgetNode>,
) -> windows::core::Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
    use windows::Win32::Foundation::RECT;

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
            ((rect.right - rect.left) as f32, (rect.bottom - rect.top) as f32)
        } else {
            (0.0, 0.0)
        }
    };
    state.root_widget = Some(root);
    if let Some(r) = state.root_widget.as_mut() {
        let _ = r.run_layout(cw, ch);
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
                let _ = root.run_layout(w, h);
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
                let _ = root.update_hover(
                    &runtime::get().compositor,
                    x, y, state.mouse_down,
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
                let _ = root.update_hover(
                    &runtime::get().compositor,
                    x, y, true,
                );
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
                let _ = root.update_hover(
                    &runtime::get().compositor,
                    x, y, false,
                );
            }
            return LRESULT(0);
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}
