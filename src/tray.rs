use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::{WM_TRAY_CALLBACK, CMD_KEEP_AWAKE, CMD_TIMER_15, CMD_TIMER_30, CMD_TIMER_60,
            CMD_TIMER_120, CMD_TIMER_CUSTOM, CMD_BLACKOUT, CMD_QUIT, STATE};

const TRAY_ICON_ID: u32 = 1;

fn wstr_copy(dest: &mut [u16], src: &str) {
    let wide: Vec<u16> = src.encode_utf16().collect();
    let len = wide.len().min(dest.len() - 1);
    dest[..len].copy_from_slice(&wide[..len]);
    dest[len] = 0;
}

pub fn add_tray_icon(hwnd: HWND, icon: HICON) -> Result<()> {
    unsafe {
        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = TRAY_ICON_ID;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAY_CALLBACK;
        nid.hIcon = icon;
        wstr_copy(&mut nid.szTip, "Caffeinate");

        Shell_NotifyIconW(NIM_ADD, &nid)?;
        Ok(())
    }
}

pub fn remove_tray_icon(hwnd: HWND) {
    unsafe {
        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = TRAY_ICON_ID;
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}

pub fn show_context_menu(hwnd: HWND) {
    unsafe {
        let menu = CreatePopupMenu().expect("CreatePopupMenu");
        let timer_sub = CreatePopupMenu().expect("CreatePopupMenu");

        // Check mark for Keep Awake
        let awake_flags = STATE.with(|s| {
            if s.borrow().awake_active { MF_STRING | MF_CHECKED } else { MF_STRING }
        });

        let _ = AppendMenuW(menu, awake_flags, CMD_KEEP_AWAKE as usize, w!("Keep Awake"));

        // Timer submenu
        let _ = AppendMenuW(timer_sub, MF_STRING, CMD_TIMER_15 as usize, w!("15 minutes"));
        let _ = AppendMenuW(timer_sub, MF_STRING, CMD_TIMER_30 as usize, w!("30 minutes"));
        let _ = AppendMenuW(timer_sub, MF_STRING, CMD_TIMER_60 as usize, w!("1 hour"));
        let _ = AppendMenuW(timer_sub, MF_STRING, CMD_TIMER_120 as usize, w!("2 hours"));
        let _ = AppendMenuW(timer_sub, MF_SEPARATOR, 0, PCWSTR::null());
        let _ = AppendMenuW(timer_sub, MF_STRING, CMD_TIMER_CUSTOM as usize, w!("Custom..."));

        let _ = AppendMenuW(menu, MF_POPUP, timer_sub.0 as usize, w!("Timer"));

        let _ = AppendMenuW(menu, MF_STRING, CMD_BLACKOUT as usize, w!("Black Out Screen"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
        let _ = AppendMenuW(menu, MF_STRING, CMD_QUIT as usize, w!("Quit"));

        // Position at cursor
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);

        // Required for menu to dismiss properly
        SetForegroundWindow(hwnd);
        TrackPopupMenuEx(
            menu,
            TPM_LEFTALIGN | TPM_BOTTOMALIGN,
            pt.x, pt.y, hwnd, None,
        );
        let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
    }
}

pub fn show_balloon(hwnd: HWND, title: &str, message: &str) {
    unsafe {
        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = TRAY_ICON_ID;
        nid.uFlags = NIF_INFO;
        nid.dwInfoFlags = NIIF_INFO;
        wstr_copy(&mut nid.szInfoTitle, title);
        wstr_copy(&mut nid.szInfo, message);
        let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}
