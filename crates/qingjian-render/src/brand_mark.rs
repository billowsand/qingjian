//! 悬浮状态条里的「字在」品牌标记：与主 Logo 共用几何语义，但按小尺寸重新绘制。

use tiny_skia::{LineCap, LineJoin, PathBuilder, Stroke};

use crate::canvas::Canvas;
use crate::color::Color;
use crate::theme::Theme;

const WARM_WHITE: Color = Color::rgb(247, 246, 242);

/// 在给定方形区域绘制当前主题对应的「字在」标记。
pub(crate) fn draw_brand_mark(canvas: &mut Canvas, x: f32, y: f32, size: f32, theme: &Theme) {
    let dark = relative_luminance(theme.colors.background) < 0.34;
    let inset = if dark { size * 0.055 } else { 0.0 };
    let body = if dark {
        theme.colors.highlight
    } else {
        theme.colors.accent
    };
    if dark {
        canvas.fill_round_rect(x, y, size, size, size * 0.225, theme.colors.accent);
    }
    canvas.fill_round_rect(
        x + inset,
        y + inset,
        size - inset * 2.0,
        size - inset * 2.0,
        size * 0.19,
        body,
    );

    let point = |px: f32, py: f32| (x + px * size, y + py * size);
    let mut top = PathBuilder::new();
    let (start_x, start_y) = point(0.205, 0.381);
    top.move_to(start_x, start_y);
    let (c1x, c1y) = point(0.371, 0.308);
    let (c2x, c2y) = point(0.635, 0.308);
    let (end_x, end_y) = point(0.781, 0.381);
    top.cubic_to(c1x, c1y, c2x, c2y, end_x, end_y);
    if let Some(path) = top.finish() {
        canvas.stroke_path(&path, WARM_WHITE, &rounded_stroke(size * 0.07));
    }

    let mut lower = PathBuilder::new();
    let (left_x, left_y) = point(0.205, 0.488);
    lower.move_to(left_x, left_y);
    let (_, down_y) = point(0.205, 0.664);
    lower.line_to(left_x, down_y);
    let (c1x, c1y) = point(0.205, 0.718);
    let (c2x, c2y) = point(0.229, 0.742);
    let (turn_x, turn_y) = point(0.283, 0.742);
    lower.cubic_to(c1x, c1y, c2x, c2y, turn_x, turn_y);
    let (lower_end_x, lower_end_y) = point(0.586, 0.742);
    lower.line_to(lower_end_x, lower_end_y);
    if let Some(path) = lower.finish() {
        canvas.stroke_path(&path, WARM_WHITE, &rounded_stroke(size * 0.07));
    }

    let mut hook = PathBuilder::new();
    let (hook_x, hook_y) = point(0.703, 0.742);
    hook.move_to(hook_x, hook_y);
    let (c1x, c1y) = point(0.757, 0.742);
    let (c2x, c2y) = point(0.791, 0.703);
    let (hook_end_x, hook_end_y) = point(0.791, 0.635);
    hook.cubic_to(c1x, c1y, c2x, c2y, hook_end_x, hook_end_y);
    if let Some(path) = hook.finish() {
        canvas.stroke_path(&path, WARM_WHITE, &rounded_stroke(size * 0.07));
    }

    let dot = size * 0.043;
    let (dot_x, dot_y) = point(0.293, 0.249);
    canvas.fill_round_rect(
        dot_x - dot,
        dot_y - dot,
        dot * 2.0,
        dot * 2.0,
        dot,
        theme.colors.caret,
    );
    let caret_width = (size * 0.049).max(1.0);
    let (caret_x, caret_top) = point(0.488, 0.459);
    let (_, caret_bottom) = point(0.488, 0.635);
    canvas.fill_round_rect(
        caret_x - caret_width / 2.0,
        caret_top,
        caret_width,
        caret_bottom - caret_top,
        caret_width / 2.0,
        theme.colors.caret,
    );
}

fn rounded_stroke(width: f32) -> Stroke {
    Stroke {
        width: width.max(1.0),
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    }
}

fn relative_luminance(color: Color) -> f32 {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

#[cfg(test)]
mod tests {
    use super::relative_luminance;
    use crate::color::Color;

    #[test]
    fn distinguishes_light_and_dark_surfaces() {
        assert!(relative_luminance(Color::rgb(247, 241, 232)) > 0.34);
        assert!(relative_luminance(Color::rgb(23, 35, 27)) < 0.34);
    }
}
