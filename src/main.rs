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
use windows::Win32::System::Threading::{CreateMutexW, OpenMutexW, MUTEX_ALL_ACCESS};
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
    pub icon_handle: Option<HICON>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hwnd: HWND::default(),
            awake_active: false,
            timer_active: false,
            blackout_hwnd: None,
            icon_handle: None,
        }
    }
}

thread_local! {
    pub static STATE: RefCell<AppState> = RefCell::new(AppState::default());
}

fn main() -> Result<()> {
    unsafe {
        // Single-instance guard
        if OpenMutexW(MUTEX_ALL_ACCESS, false, w!("CaffeinateAppMutex")).is_ok() {
            return Ok(()); // Another instance is already running
        }
        let _mutex = CreateMutexW(None, false, w!("CaffeinateAppMutex"))?;
        std::mem::forget(_mutex); // Keep alive until process exits

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

        let icon_handle = icon::create_icon()?;
        tray::add_tray_icon(hwnd, icon_handle)?;
        STATE.with(|s| s.borrow_mut().icon_handle = Some(icon_handle));

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
        WM_TRAY_CALLBACK => {
            let event = (lparam.0 & 0xFFFF) as u32;
            if event == WM_RBUTTONUP {
                tray::show_context_menu(hwnd);
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let cmd = (wparam.0 & 0xFFFF) as u16;
            handle_command(hwnd, cmd);
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_ID {
                timer::on_expired(hwnd);
                update_tray_status(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            timer::stop(hwnd);
            awake::disable();
            tray::remove_tray_icon(hwnd);
            STATE.with(|s| {
                if let Some(icon) = s.borrow().icon_handle {
                    let _ = DestroyIcon(icon);
                }
            });
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn handle_command(hwnd: HWND, cmd: u16) {
    match cmd {
        CMD_KEEP_AWAKE => {
            let (was_active, timer_was_active) = STATE.with(|s| {
                let state = s.borrow();
                (state.awake_active, state.timer_active)
            });
            if was_active {
                awake::disable();
                if timer_was_active {
                    timer::stop(hwnd);
                }
                STATE.with(|s| {
                    let mut state = s.borrow_mut();
                    state.awake_active = false;
                    state.timer_active = false;
                });
            } else {
                if timer_was_active {
                    timer::stop(hwnd);
                }
                if awake::enable() {
                    STATE.with(|s| {
                        let mut state = s.borrow_mut();
                        state.awake_active = true;
                        state.timer_active = false;
                    });
                } else {
                    tray::show_balloon(hwnd, "Caffeinate", "Failed to set keep-awake state.");
                }
            }
        }
        CMD_TIMER_15 => timer::start(hwnd, 15),
        CMD_TIMER_30 => timer::start(hwnd, 30),
        CMD_TIMER_60 => timer::start(hwnd, 60),
        CMD_TIMER_120 => timer::start(hwnd, 120),
        CMD_TIMER_CUSTOM => {
            if let Some(minutes) = dialog::show_custom_timer_dialog(hwnd) {
                timer::start(hwnd, minutes);
            }
        }
        CMD_BLACKOUT => {
            // Only one blackout at a time
            let already_active = STATE.with(|s| s.borrow().blackout_hwnd.is_some());
            if !already_active {
                blackout::activate(hwnd);
            }
        }
        CMD_QUIT => unsafe {
            let _ = DestroyWindow(hwnd);
        },
        _ => {}
    }
    update_tray_status(hwnd);
}

fn update_tray_status(hwnd: HWND) {
    STATE.with(|s| {
        let state = s.borrow();
        let tip = if state.timer_active {
            "Caffeinate \u{2014} timer active"
        } else if state.awake_active {
            "Caffeinate \u{2014} keeping awake"
        } else {
            "Caffeinate \u{2014} idle"
        };
        tray::update_tooltip(hwnd, tip);
    });
}
