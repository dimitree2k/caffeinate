use std::sync::Once;
use std::sync::atomic::{AtomicU64, Ordering};
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

#[link(name = "kernel32")]
extern "system" {
    fn GetTickCount64() -> u64;
}

static BLACKOUT_CLASS_INIT: Once = Once::new();
// Timestamp when the blackout window was created — ignore input for a grace period
static BLACKOUT_CREATED_AT: AtomicU64 = AtomicU64::new(0);
const GRACE_PERIOD_MS: u64 = 1000;

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
                hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0 as _),
                hCursor: LoadCursorW(HINSTANCE::default(), IDC_ARROW).unwrap_or_default(),
                ..Default::default()
            };
            RegisterClassExW(&wc);
        });

        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        BLACKOUT_CREATED_AT.store(GetTickCount64(), Ordering::Relaxed);

        let blackout = match CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            w!("CaffeinateBlackout"),
            w!(""),
            WS_POPUP | WS_VISIBLE,
            vx, vy, vw, vh,
            HWND::default(),
            HMENU::default(),
            instance,
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

        let _ = LockWorkStation();
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
            let elapsed = GetTickCount64() - BLACKOUT_CREATED_AT.load(Ordering::Relaxed);
            if elapsed >= GRACE_PERIOD_MS {
                dismiss_blackout(hwnd);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let brush = HBRUSH(GetStockObject(BLACK_BRUSH).0 as _);
            FillRect(hdc, &ps.rcPaint, brush);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
