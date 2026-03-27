#![windows_subsystem = "windows"]

mod awake;
mod blackout;
mod dialog;
mod icon;
mod timer;
mod tray;

use std::cell::RefCell;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const WM_TRAY_CALLBACK: u32 = WM_USER + 1;
pub const TIMER_ID: usize = 1;

pub const CMD_KEEP_AWAKE: u16 = 100;
pub const CMD_TIMER_15: u16 = 101;
pub const CMD_TIMER_30: u16 = 102;
pub const CMD_TIMER_60: u16 = 103;
pub const CMD_TIMER_120: u16 = 104;
pub const CMD_TIMER_CUSTOM: u16 = 105;
pub const CMD_BLACKOUT: u16 = 106;
pub const CMD_QUIT: u16 = 107;

pub struct AppState {
    pub hwnd: HWND,
    pub awake_active: bool,
    pub timer_active: bool,
    pub blackout_hwnd: Option<HWND>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hwnd: HWND::default(),
            awake_active: false,
            timer_active: false,
            blackout_hwnd: None,
        }
    }
}

thread_local! {
    pub static STATE: RefCell<AppState> = RefCell::new(AppState::default());
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = w!("CaffeinateClass");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Caffeinate"),
            WINDOW_STYLE::default(),
            0, 0, 0, 0,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        STATE.with(|s| s.borrow_mut().hwnd = hwnd);

        let icon_handle = icon::create_placeholder_icon()?;
        tray::add_tray_icon(hwnd, icon_handle)?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            tray::remove_tray_icon(hwnd);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
