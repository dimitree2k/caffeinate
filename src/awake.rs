use windows::Win32::System::Power::*;

pub fn enable() {
    unsafe {
        SetThreadExecutionState(
            ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED,
        );
    }
}

pub fn disable() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS);
    }
}
