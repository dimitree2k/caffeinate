use std::cell::Cell;
use std::sync::Once;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, SetFocus};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;

const DLG_WIDTH: i32 = 260;
const DLG_HEIGHT: i32 = 130;
const ID_EDIT: i32 = 301;
const ID_OK: i32 = 302;
const ID_CANCEL: i32 = 303;

thread_local! {
    static DIALOG_RESULT: Cell<Option<u32>> = Cell::new(None);
    static EDIT_HANDLE: Cell<HWND> = Cell::new(HWND::default());
}

static DIALOG_CLASS_INIT: Once = Once::new();

pub fn show_custom_timer_dialog(parent: HWND) -> Option<u32> {
    unsafe {
        DIALOG_RESULT.with(|r| r.set(None));

        let instance = GetWindowLongPtrW(parent, GWL_HINSTANCE);

        DIALOG_CLASS_INIT.call_once(|| {
            let class_name = w!("CaffeinateDialog");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(dialog_proc),
                hInstance: HINSTANCE(instance as _),
                lpszClassName: class_name,
                hbrBackground: GetSysColorBrush(COLOR_3DFACE),
                hCursor: LoadCursorW(HINSTANCE::default(), IDC_ARROW).unwrap_or_default(),
                ..Default::default()
            };
            RegisterClassExW(&wc);
        });

        // Center on screen
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - DLG_WIDTH) / 2;
        let y = (screen_h - DLG_HEIGHT) / 2;

        let dlg = CreateWindowExW(
            WS_EX_DLGMODALFRAME | WS_EX_TOPMOST,
            w!("CaffeinateDialog"),
            w!("Custom Timer"),
            WS_POPUP | WS_CAPTION | WS_SYSMENU,
            x, y, DLG_WIDTH, DLG_HEIGHT,
            parent,
            HMENU::default(),
            HINSTANCE(instance as _),
            None,
        ).expect("CreateWindowExW dialog");

        // Label
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("Minutes:"),
            WS_CHILD | WS_VISIBLE,
            15, 20, 60, 20,
            dlg,
            HMENU::default(),
            HINSTANCE(instance as _),
            None,
        ).expect("label");

        // Edit (number-only via ES_NUMBER = 0x2000)
        let edit = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("EDIT"),
            w!("30"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP
                | WINDOW_STYLE(0x2000), // ES_NUMBER
            80, 18, 80, 24,
            dlg,
            HMENU(ID_EDIT as _),
            HINSTANCE(instance as _),
            None,
        ).expect("edit");
        EDIT_HANDLE.with(|h| h.set(edit));

        // OK button (BS_DEFPUSHBUTTON = 0x0001)
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("OK"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(0x0001),
            50, 60, 70, 28,
            dlg,
            HMENU(ID_OK as _),
            HINSTANCE(instance as _),
            None,
        ).expect("ok button");

        // Cancel button
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("Cancel"),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP,
            135, 60, 70, 28,
            dlg,
            HMENU(ID_CANCEL as _),
            HINSTANCE(instance as _),
            None,
        ).expect("cancel button");

        // Disable parent (modal)
        let _ = EnableWindow(parent, false);
        let _ = ShowWindow(dlg, SW_SHOW);
        let _ = SetFocus(edit);

        // Local message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !IsDialogMessageW(dlg, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            // Check if dialog was closed
            if !IsWindow(dlg).as_bool() {
                break;
            }
        }

        // Re-enable parent
        let _ = EnableWindow(parent, true);
        let _ = SetForegroundWindow(parent);

        DIALOG_RESULT.with(|r| r.get())
    }
}

unsafe extern "system" fn dialog_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as i32;
            match id {
                ID_OK => {
                    let edit = EDIT_HANDLE.with(|h| h.get());
                    let mut buf = [0u16; 16];
                    let len = GetWindowTextW(edit, &mut buf);
                    let text = String::from_utf16_lossy(&buf[..len as usize]);
                    let valid = text.trim().parse::<u32>()
                        .ok()
                        .filter(|&m| m > 0 && m <= 1440);
                    if let Some(mins) = valid {
                        DIALOG_RESULT.with(|r| r.set(Some(mins)));
                        let _ = DestroyWindow(hwnd);
                    } else {
                        MessageBoxW(
                            hwnd,
                            w!("Enter a value between 1 and 1440 minutes."),
                            w!("Invalid Input"),
                            MB_OK | MB_ICONWARNING,
                        );
                    }
                }
                ID_CANCEL => {
                    let _ = DestroyWindow(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
