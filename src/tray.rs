use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::WM_TRAY_CALLBACK;

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
