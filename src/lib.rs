// Standard library imports
use std::mem::transmute;

// Windows core
use windows::core::*;

// Windows API imports
use windows::Win32::{
    Foundation::{BOOL, *},
    System::{Diagnostics::ToolHelp::*, SystemServices::*, Threading::*},
    UI::WindowsAndMessaging::*,
};

static mut PREV_WNDPROC: WNDPROC = None;

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn DllMain(_: HINSTANCE, call_reason: u32, _: *mut ()) {
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        _ => {}
    }
}

fn attach() {
    unsafe {
        let pid = GetCurrentProcessId();
        let mut handle = Default::default();
        let mut data = (handle, pid);

        let _ = EnumWindows(
            Some(find_window_by_pid),
            LPARAM(&mut data as *mut (HWND, u32) as _),
        );

        handle = data.0;

        let _ = EnumChildWindows(
            Some(handle),
            Some(find_child_window),
            LPARAM(&mut handle as *mut HWND as _),
        );

        let original_proc = GetWindowLongPtrW(handle, GWLP_WNDPROC);
        PREV_WNDPROC = transmute::<_, WNDPROC>(original_proc);

        SetWindowLongPtrW(handle, GWLP_WNDPROC, wnd_proc as _);
    }
}

unsafe extern "system" fn find_child_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut class_name: [u16; 256] = [0; 256];
    GetClassNameW(hwnd, &mut class_name);
    let window_class = String::from_utf16_lossy(&class_name);
    let handle = lparam.0 as *mut HWND;

    if window_class.contains("Chrome_WidgetWin_1") {
        *handle = hwnd;
        return BOOL(0);
    }
    BOOL(1)
}

unsafe extern "system" fn find_window_by_pid(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = lparam.0 as *mut (HWND, u32);
    let target_pid = (*data).1;
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));

    if pid == target_pid {
        (*data).0 = hwnd;
        return BOOL(0);
    }
    BOOL(1)
}

#[no_mangle]
extern "system" fn wnd_proc(
    window: HWND,
    message: u32,
    mut wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if message == WM_MOUSEMOVE || message == WM_LBUTTONDOWN {
            wparam = WPARAM(wparam.0 & !MK_LBUTTON.0 as usize);
        }
        CallWindowProcW(PREV_WNDPROC, window, message, wparam, lparam)
    }
}
