//! 候选窗口的定位锚点：组句范围在屏幕上的矩形。

use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::UI::TextServices::{ITfContext, ITfRange};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::core::BOOL;

use qingjian_platform::protocol::ScreenRect;

/// `range` 的屏幕矩形。有些应用偶尔量不出来（刚起组句还没排版、自绘输入框给全零），
/// 这时沿用本段组句上次量到的 `previous`——跳到鼠标那儿去会让候选窗忽上忽下；
/// 一次都没量到过才退到鼠标处。
pub(crate) fn anchor_rect(
    context: &ITfContext,
    ec: u32,
    range: &ITfRange,
    previous: Option<ScreenRect>,
) -> ScreenRect {
    match range_rect(context, ec, range) {
        Some(rect) => to_screen(rect),
        None => previous.unwrap_or_else(|| to_screen(mouse_anchor())),
    }
}

fn range_rect(context: &ITfContext, ec: u32, range: &ITfRange) -> Option<RECT> {
    let mut rect = RECT::default();
    let mut clipped = BOOL(0);
    unsafe {
        let view = context.GetActiveView().ok()?;
        view.GetTextExt(ec, range, &mut rect, &mut clipped).ok()?;
    }
    (rect.right > rect.left || rect.bottom > rect.top).then_some(rect)
}

fn mouse_anchor() -> RECT {
    let mut point = POINT::default();
    let _ = unsafe { GetCursorPos(&mut point) };
    RECT {
        left: point.x,
        top: point.y,
        right: point.x,
        bottom: point.y + 16,
    }
}

fn to_screen(rect: RECT) -> ScreenRect {
    ScreenRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}
