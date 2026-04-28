use crate::runtime;
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
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, LoadCursorW, PostQuitMessage,
                RegisterClassExW, ShowWindow, CS_HREDRAW, CS_VREDRAW,
                CW_USEDEFAULT, IDC_ARROW, SW_SHOW, WM_DESTROY, WM_ERASEBKGND,
                WNDCLASSEXW, WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW,
            },
        },
    },
    UI::Composition::{ContainerVisual, Desktop::DesktopWindowTarget, Visual},
};

pub struct WindowState {
    pub hwnd: HWND,
    pub root: ContainerVisual,
    // Kept alive: dropping DesktopWindowTarget detaches Visual Layer from HWND.
    _target: DesktopWindowTarget,
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
    Ok(Box::new(WindowState { hwnd, root, _target: target }))
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

// Try Win11 22H2+ public API first; fall back to Win11 21H2 private attribute.
// Silently no-ops on Windows 10.
fn apply_mica(hwnd: HWND) {
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
    if ok {
        return;
    }
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

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    // Prevent GDI from painting an opaque background over the DWM backdrop.
    if msg == WM_ERASEBKGND {
        return LRESULT(1);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
