use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::{TIMER_ID, STATE};

pub fn start(hwnd: HWND, minutes: u32) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        // Cancel existing timer if running
        if state.timer_active {
            unsafe { let _ = KillTimer(hwnd, TIMER_ID); }
            state.timer_active = false;
        }
        // Enable awake
        state.awake_active = crate::awake::enable();

        // Only mark active if the timer was actually created, otherwise the
        // expiry never fires and the system is kept awake forever.
        let duration_ms = minutes * 60 * 1000;
        unsafe {
            if SetTimer(hwnd, TIMER_ID, duration_ms, None) != 0 {
                state.timer_active = true;
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
        s.borrow_mut().timer_active = false;
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
