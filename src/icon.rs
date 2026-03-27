use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// Pre-converted .ico embedded at compile time (16x16 + 32x32, 32-bit RGBA)
const ICON_ICO: &[u8] = include_bytes!("../caffeinate.ico");

/// Load the embedded .ico file and create an HICON.
/// Falls back to a solid coffee-brown square if loading fails.
pub fn create_icon() -> Result<HICON> {
    create_icon_from_ico().or_else(|_| create_placeholder_icon())
}

fn create_icon_from_ico() -> Result<HICON> {
    unsafe {
        // ICO files contain a directory header; skip to the first image entry.
        // ICO header: 3x u16 (6 bytes), then each entry is 16 bytes.
        // Entry offset to image data is at bytes 18..22 (u32 LE).
        // Entry image size is at bytes 14..18 (u32 LE).
        if ICON_ICO.len() < 22 {
            return Err(Error::empty());
        }

        let data_size = u32::from_le_bytes([
            ICON_ICO[14], ICON_ICO[15], ICON_ICO[16], ICON_ICO[17],
        ]);
        let data_offset = u32::from_le_bytes([
            ICON_ICO[18], ICON_ICO[19], ICON_ICO[20], ICON_ICO[21],
        ]) as usize;

        if data_offset + data_size as usize > ICON_ICO.len() {
            return Err(Error::empty());
        }

        let icon_data = &ICON_ICO[data_offset..data_offset + data_size as usize];

        let icon = CreateIconFromResourceEx(
            icon_data,
            true, // fIcon
            0x00030000, // version (required: 0x00030000)
            16, 16,
            LR_DEFAULTCOLOR,
        )?;

        Ok(icon)
    }
}

/// Fallback: create a 16x16 solid coffee-brown (#6F4E37) icon.
fn create_placeholder_icon() -> Result<HICON> {
    unsafe {
        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let bmp_color = CreateCompatibleBitmap(hdc_screen, 16, 16);
        let old = SelectObject(hdc_mem, bmp_color);

        let brush = CreateSolidBrush(COLORREF(0x00374E6F));
        let rect = RECT { left: 0, top: 0, right: 16, bottom: 16 };
        FillRect(hdc_mem, &rect, brush);

        SelectObject(hdc_mem, old);
        let _ = DeleteObject(brush);

        let bmp_mask = CreateBitmap(16, 16, 1, 1, None);

        let icon_info = ICONINFO {
            fIcon: BOOL::from(true),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: bmp_mask,
            hbmColor: bmp_color,
        };

        let icon = CreateIconIndirect(&icon_info)?;

        let _ = DeleteObject(bmp_color);
        let _ = DeleteObject(bmp_mask);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);

        Ok(icon)
    }
}
