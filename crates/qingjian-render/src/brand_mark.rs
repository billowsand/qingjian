//! 悬浮状态条左侧的「字在」品牌 Logo，同时也是拖拽握柄。

use tiny_skia::{LineCap, LineJoin, PathBuilder, Stroke};

use crate::canvas::Canvas;
use crate::theme::Theme;

/// 在给定方形区域绘制当前主题的品牌 Logo：外框取强调色，圆点与中心竖线取输入色。
pub(crate) fn draw_brand_mark(canvas: &mut Canvas, x: f32, y: f32, size: f32, theme: &Theme) {
    let point = |px: f32, py: f32| (x + px * size, y + py * size);
    let stroke = rounded_stroke(size * 0.085);
    let frame = theme.colors.accent;
    let signal = theme.colors.caret;

    let mut top = PathBuilder::new();
    let (start_x, start_y) = point(0.16, 0.34);
    top.move_to(start_x, start_y);
    let (c1x, c1y) = point(0.34, 0.25);
    let (c2x, c2y) = point(0.65, 0.25);
    let (end_x, end_y) = point(0.83, 0.34);
    top.cubic_to(c1x, c1y, c2x, c2y, end_x, end_y);
    if let Some(path) = top.finish() {
        canvas.stroke_path(&path, frame, &stroke);
    }

    let mut lower = PathBuilder::new();
    let (left_x, left_y) = point(0.17, 0.48);
    lower.move_to(left_x, left_y);
    let (_, down_y) = point(0.17, 0.70);
    lower.line_to(left_x, down_y);
    let (c1x, c1y) = point(0.17, 0.79);
    let (c2x, c2y) = point(0.22, 0.83);
    let (turn_x, turn_y) = point(0.31, 0.83);
    lower.cubic_to(c1x, c1y, c2x, c2y, turn_x, turn_y);
    let (lower_end_x, lower_end_y) = point(0.65, 0.83);
    lower.line_to(lower_end_x, lower_end_y);
    if let Some(path) = lower.finish() {
        canvas.stroke_path(&path, frame, &stroke);
    }

    let mut hook = PathBuilder::new();
    let (hook_x, hook_y) = point(0.76, 0.83);
    hook.move_to(hook_x, hook_y);
    let (c1x, c1y) = point(0.82, 0.83);
    let (c2x, c2y) = point(0.84, 0.77);
    let (end_x, end_y) = point(0.84, 0.67);
    hook.cubic_to(c1x, c1y, c2x, c2y, end_x, end_y);
    if let Some(path) = hook.finish() {
        canvas.stroke_path(&path, frame, &stroke);
    }

    let dot = size * 0.055;
    let (dot_x, dot_y) = point(0.28, 0.16);
    canvas.fill_round_rect(dot_x - dot, dot_y - dot, dot * 2.0, dot * 2.0, dot, signal);

    let stem = (size * 0.075).max(1.0);
    let (stem_x, stem_top) = point(0.50, 0.48);
    let (_, stem_bottom) = point(0.50, 0.70);
    canvas.fill_round_rect(
        stem_x - stem / 2.0,
        stem_top,
        stem,
        stem_bottom - stem_top,
        stem / 2.0,
        signal,
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
