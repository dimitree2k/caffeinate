use windows::Win32::System::Power::*;

/// Returns true if the execution state was set successfully.
pub fn enable() -> bool {
    unsafe {
        let result = SetThreadExecutionState(
            ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED,
        );
        result.0 != 0
    }
}

pub fn disable() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS);
    }
}
