use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Create a 16x16 solid coffee-brown (#6F4E37) icon at runtime via GDI.
pub fn create_placeholder_icon() -> Result<HICON> {
    unsafe {
        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let bmp_color = CreateCompatibleBitmap(hdc_screen, 16, 16);
        let old = SelectObject(hdc_mem, bmp_color);

        // #6F4E37 in BGR COLORREF = 0x00374E6F
        let brush = CreateSolidBrush(COLORREF(0x00374E6F));
        let rect = RECT { left: 0, top: 0, right: 16, bottom: 16 };
        FillRect(hdc_mem, &rect, brush);

        SelectObject(hdc_mem, old);
        let _ = DeleteObject(brush);

        // Mask bitmap — all zeros = fully opaque
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
