use windows::Win32::Foundation::{FILETIME, HWND, SYSTEMTIME};
use windows::Win32::System::SystemInformation::GetSystemTimeAsFileTime;
use windows::Win32::System::Time::{FileTimeToSystemTime, SystemTimeToTzSpecificLocalTime};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::{TIMER_ID, STATE};

/// Fixed tooltip text shown while the timer runs ("until HH:MM"), computed
/// once at start - the end timestamp never changes.
fn timer_end_tip(minutes: u32) -> String {
    unsafe {
        let now = GetSystemTimeAsFileTime();
        let ticks = ((now.dwHighDateTime as u64) << 32) | now.dwLowDateTime as u64;
        let end = ticks + minutes as u64 * 60 * 10_000_000;
        let ft = FILETIME {
            dwLowDateTime: end as u32,
            dwHighDateTime: (end >> 32) as u32,
        };
        let mut utc = SYSTEMTIME::default();
        let _ = FileTimeToSystemTime(&ft, &mut utc);
        let mut local = SYSTEMTIME::default();
        let _ = SystemTimeToTzSpecificLocalTime(None, &utc, &mut local);
        format!(
            "Caffeinate \u{2014} timer until {:02}:{:02}",
            local.wHour, local.wMinute
        )
    }
}

pub fn start(hwnd: HWND, minutes: u32) {
    let end_tip = timer_end_tip(minutes);
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        // Cancel existing timer if running
        if state.timer_active {
            unsafe { let _ = KillTimer(hwnd, TIMER_ID); }
        }
        // Enable awake
        state.awake_active = crate::awake::enable();

        // Only mark active if the timer was actually created, otherwise the
        // expiry never fires and the system is kept awake forever.
        let duration_ms = minutes * 60 * 1000;
        unsafe {
            if SetTimer(hwnd, TIMER_ID, duration_ms, None) != 0 {
                state.timer_active = true;
                state.timer_tip = Some(end_tip);
            } else {
                // No timer means no expiry: fall back to fully idle
                crate::awake::disable();
                state.awake_active = false;
            }
        }
    });
}

pub fn stop(hwnd: HWND) {
    unsafe {
        let _ = KillTimer(hwnd, TIMER_ID);
    }
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.timer_active = false;
        state.timer_tip = None;
    });
}

pub fn on_expired(hwnd: HWND) {
    stop(hwnd);
    crate::awake::disable();
    STATE.with(|s| {
        s.borrow_mut().awake_active = false;
    });
    crate::tray::show_balloon(hwnd, "Caffeinate", "Timer expired \u{2014} system can sleep now.");
}
