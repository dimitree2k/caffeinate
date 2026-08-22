use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// Embedded .ico files at compile time (16x16 + 32x32, 32-bit RGBA)
const ICON_ACTIVE_ICO: &[u8] = include_bytes!("../caffeinate.ico");
const ICON_IDLE_ICO: &[u8] = include_bytes!("../caffeinate_idle.ico");

/// Load the embedded active (orange) .ico file and create an HICON.
pub fn create_active_icon() -> Result<HICON> {
    create_icon_from_ico(ICON_ACTIVE_ICO).or_else(|_| create_placeholder_icon(0x00374E6F)) // #6F4E37 (coffee brown)
}

/// Load the embedded idle (gray) .ico file and create an HICON.
pub fn create_idle_icon() -> Result<HICON> {
    create_icon_from_ico(ICON_IDLE_ICO).or_else(|_| create_placeholder_icon(0x00808080)) // #808080 (gray)
}

fn create_icon_from_ico(ico_bytes: &[u8]) -> Result<HICON> {
    unsafe {
        // ICO header is 6 bytes.
        if ico_bytes.len() < 6 {
            return Err(Error::empty());
        }

        // Count of images in the ICO file.
        let count = u16::from_le_bytes([ico_bytes[4], ico_bytes[5]]) as usize;
        if count == 0 {
            return Err(Error::empty());
        }

        // Query the ideal system tray (small icon) size.
        let target_cx = GetSystemMetrics(SM_CXSMICON);
        let target_cy = GetSystemMetrics(SM_CYSMICON);
        let target_cx = if target_cx <= 0 { 16 } else { target_cx };
        let target_cy = if target_cy <= 0 { 16 } else { target_cy };

        // Traverse the icon directory to find the best match for our target size.
        // Scoring: exact match first, then downscaling (smallest size above
        // target), then upscaling (largest size below target).
        let mut best_index = 0;
        let mut best_score = (u32::MAX, u32::MAX);

        for i in 0..count {
            let entry_offset = 6 + i * 16;
            if entry_offset + 16 > ico_bytes.len() {
                break;
            }
            let w = ico_bytes[entry_offset] as i32;
            let w = if w == 0 { 256 } else { w };

            let score = match w - target_cx {
                0 => (0, 0),
                diff if diff > 0 => (0, diff as u32),
                diff => (1, (-diff) as u32),
            };

            if score < best_score {
                best_score = score;
                best_index = i;
                if score == (0, 0) {
                    break; // Exact match found
                }
            }
        }

        let entry_offset = 6 + best_index * 16;
        if entry_offset + 16 > ico_bytes.len() {
            return Err(Error::empty());
        }

        let data_size = u32::from_le_bytes([
            ico_bytes[entry_offset + 8],
            ico_bytes[entry_offset + 9],
            ico_bytes[entry_offset + 10],
            ico_bytes[entry_offset + 11],
        ]);
        let data_offset = u32::from_le_bytes([
            ico_bytes[entry_offset + 12],
            ico_bytes[entry_offset + 13],
            ico_bytes[entry_offset + 14],
            ico_bytes[entry_offset + 15],
        ]) as usize;

        if data_offset + data_size as usize > ico_bytes.len() {
            return Err(Error::empty());
        }

        let icon_data = &ico_bytes[data_offset..data_offset + data_size as usize];

        let icon = CreateIconFromResourceEx(
            icon_data,
            true, // fIcon
            0x00030000, // version (required: 0x00030000)
            target_cx,
            target_cy,
            LR_DEFAULTCOLOR,
        )?;

        Ok(icon)
    }
}

/// Fallback: create a solid color icon matching the system tray icon size.
fn create_placeholder_icon(color_hex: u32) -> Result<HICON> {
    unsafe {
        let target_cx = GetSystemMetrics(SM_CXSMICON);
        let target_cy = GetSystemMetrics(SM_CYSMICON);
        let target_cx = if target_cx <= 0 { 16 } else { target_cx };
        let target_cy = if target_cy <= 0 { 16 } else { target_cy };

        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let bmp_color = CreateCompatibleBitmap(hdc_screen, target_cx, target_cy);
        let old = SelectObject(hdc_mem, bmp_color);

        let brush = CreateSolidBrush(COLORREF(color_hex));
        let rect = RECT { left: 0, top: 0, right: target_cx, bottom: target_cy };
        FillRect(hdc_mem, &rect, brush);

        SelectObject(hdc_mem, old);
        let _ = DeleteObject(brush);

        let bmp_mask = CreateBitmap(target_cx, target_cy, 1, 1, None);

        let icon_info = ICONINFO {
            fIcon: BOOL::from(true),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: bmp_mask,
            hbmColor: bmp_color,
        };

        // CreateIconIndirect copies the bitmaps, so delete them regardless of outcome
        let result = CreateIconIndirect(&icon_info);
        let _ = DeleteObject(bmp_color);
        let _ = DeleteObject(bmp_mask);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);

        result
    }
}
