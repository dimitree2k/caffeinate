use std::sync::Once;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::STATE;

#[link(name = "user32")]
extern "system" {
    fn LockWorkStation() -> BOOL;
}

static BLACKOUT_CLASS_INIT: Once = Once::new();

pub fn activate(parent: HWND) {
    unsafe {
        let instance = GetModuleHandleW(None).expect("GetModuleHandleW");

        BLACKOUT_CLASS_INIT.call_once(|| {
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
        });

        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        let blackout = match CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            w!("CaffeinateBlackout"),
            w!(""),
            WS_POPUP | WS_VISIBLE,
            vx, vy, vw, vh,
            None,
            None,
            Some(instance.into()),
            None,
        ) {
            Ok(hwnd) => hwnd,
            Err(_) => {
                crate::tray::show_balloon(parent, "Caffeinate", "Failed to create blackout window.");
                return;
            }
        };

        STATE.with(|s| {
            s.borrow_mut().blackout_hwnd = Some(blackout);
        });

        let _ = ShowWindow(blackout, SW_SHOW);
        let _ = SetForegroundWindow(blackout);

        LockWorkStation();
    }
}

fn dismiss_blackout(hwnd: HWND) {
    STATE.with(|s| {
        s.borrow_mut().blackout_hwnd = None;
    });
    unsafe { let _ = DestroyWindow(hwnd); }
}

unsafe extern "system" fn blackout_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_KEYDOWN | WM_LBUTTONDOWN => {
            dismiss_blackout(hwnd);
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let brush = HBRUSH(GetStockObject(BLACK_BRUSH).0);
            FillRect(hdc, &ps.rcPaint, brush);
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
