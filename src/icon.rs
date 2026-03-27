use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// Embed the PNG at compile time
const ICON_PNG: &[u8] = include_bytes!("../icons8-cup-stickers-32.png");

/// Load the embedded PNG and create an HICON from it.
/// Falls back to a solid coffee-brown square if decoding fails.
pub fn create_icon() -> Result<HICON> {
    match create_icon_from_png() {
        Some(icon) => Ok(icon),
        None => create_placeholder_icon(),
    }
}

fn create_icon_from_png() -> Option<HICON> {
    let img = image::load_from_memory_with_format(ICON_PNG, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();
    let width = rgba.width() as i32;
    let height = rgba.height() as i32;

    // Win32 expects BGRA, not RGBA — swap R and B channels
    // Also, DIB rows are bottom-up by default, but we'll use a top-down DIB (negative height)
    let mut bgra: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for pixel in rgba.pixels() {
        bgra.push(pixel[2]); // B
        bgra.push(pixel[1]); // G
        bgra.push(pixel[0]); // R
        bgra.push(pixel[3]); // A
    }

    unsafe {
        let hdc_screen = GetDC(None);

        // Create a BITMAPINFO for a 32-bit top-down DIB
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // negative = top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0, // BI_RGB
                ..Default::default()
            },
            ..Default::default()
        };

        // Create the color bitmap
        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let color_bmp = CreateDIBSection(
            hdc_screen,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits,
            None,
            0,
        ).ok()?;

        // Copy BGRA data into the bitmap
        std::ptr::copy_nonoverlapping(
            bgra.as_ptr(),
            bits as *mut u8,
            bgra.len(),
        );

        // Create the mask bitmap (all zeros = fully opaque when color has alpha)
        let mask_bmp = CreateBitmap(width, height, 1, 1, None);

        let icon_info = ICONINFO {
            fIcon: BOOL::from(true),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask_bmp,
            hbmColor: color_bmp,
        };

        let icon = CreateIconIndirect(&icon_info).ok();

        let _ = DeleteObject(color_bmp);
        let _ = DeleteObject(mask_bmp);
        ReleaseDC(None, hdc_screen);

        icon
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
