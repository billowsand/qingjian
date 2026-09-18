//! 把字在渲染器画好的整张位图贴到 Windows 分层窗口上。

mod canvas;

use qingjian_render::Pixmap;
use windows::Win32::Foundation::{COLORREF, E_INVALIDARG, HWND, POINT, SIZE};
use windows::Win32::Graphics::Gdi::{AC_SRC_ALPHA, AC_SRC_OVER, BLENDFUNCTION, HDC};
use windows::Win32::UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow};
use windows::core::{Error, Result};

use self::canvas::Canvas;

/// 把渲染器出的预乘 RGBA 位图（已含阴影边）贴到分层窗口上，`win_pos` 是位图左上角的屏幕坐标。
pub(super) fn present(hwnd: HWND, pixmap: &Pixmap, win_pos: (i32, i32)) -> Result<()> {
    let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
    if w <= 0 || h <= 0 {
        return Err(Error::from(E_INVALIDARG));
    }
    let mut canvas = Canvas::new(w, h)?;
    // tiny-skia 是 RGBA，DIB 是 BGRA；都是预乘，只换通道顺序。
    for (dst, src) in canvas.pixels().chunks_exact_mut(4).zip(pixmap.pixels()) {
        dst[0] = src.blue();
        dst[1] = src.green();
        dst[2] = src.red();
        dst[3] = src.alpha();
    }
    update(hwnd, canvas.dc(), win_pos, (w, h))
}

/// `UpdateLayeredWindow`：整张位图按预乘 alpha 贴上并挪到 `win_pos`。
fn update(hwnd: HWND, hdc: HDC, win_pos: (i32, i32), win_size: (i32, i32)) -> Result<()> {
    let (w, h) = win_size;
    let dst = POINT {
        x: win_pos.0,
        y: win_pos.1,
    };
    let size = SIZE { cx: w, cy: h };
    let src = POINT { x: 0, y: 0 };
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };
    unsafe {
        UpdateLayeredWindow(
            hwnd,
            None,
            Some(&dst),
            Some(&size),
            Some(hdc),
            Some(&src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        )
    }
}
