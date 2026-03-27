use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::STATE;

// LockWorkStation via raw FFI — avoids windows-crate feature flag uncertainty
#[link(name = "user32")]
extern "system" {
    fn LockWorkStation() -> BOOL;
}

pub fn activate(_parent: HWND) {
    unsafe {
        let instance = GetModuleHandleW(None).expect("GetModuleHandleW");

        let class_name = w!("CaffeinateBlackout");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(blackout_proc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
            hCursor: LoadCursorW(None, IDC_ARROW).ok(),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        // Virtual screen spans all monitors (origin can be negative)
        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        let blackout = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            w!(""),
            WS_POPUP | WS_VISIBLE,
            vx, vy, vw, vh,
            None,
            None,
            Some(instance.into()),
            None,
        ).expect("CreateWindowExW blackout");

        STATE.with(|s| {
            s.borrow_mut().blackout_hwnd = Some(blackout);
        });

        let _ = ShowWindow(blackout, SW_SHOW);
        let _ = SetForegroundWindow(blackout);

        // Lock the workstation
        LockWorkStation();
    }
}

unsafe extern "system" fn blackout_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_KEYDOWN => {
            STATE.with(|s| {
                s.borrow_mut().blackout_hwnd = None;
            });
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            // Fill black (redundant with hbrBackground, but ensures it on WM_PAINT after unlock)
            let brush = GetStockObject(BLACK_BRUSH);
            FillRect(hdc, &ps.rcPaint, brush);
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
