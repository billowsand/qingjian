//! 状态条的设置齿轮：八齿外圈加圆孔，只描边不填充，避免字体回退把 U+2699 画成彩色 emoji。

use std::f32::consts::TAU;

use tiny_skia::{BlendMode, PathBuilder, Pixmap};

use crate::canvas::Canvas;
use crate::color::Color;

const TEETH: usize = 8;
const STROKE_RATIO: f32 = 0.09;

/// 画在 `(x, y)` 为左上角、边长 `size` 像素的方块里。
pub(crate) fn draw_gear(canvas: &mut Canvas, x: f32, y: f32, size: f32, color: Color) {
    let side = size.ceil() as u32 + 1;
    let Some(icon) = Pixmap::new(side, side) else {
        return;
    };
    let stroke = (size * STROKE_RATIO).max(1.0);
    let mut layer = Canvas::from_pixmap(icon);
    if let Some(outer) = gear_outline(size, 0.0) {
        layer.fill_path(&outer, color, BlendMode::SourceOver);
    }
    if let Some(inner) = gear_outline(size, stroke) {
        layer.fill_path(&inner, Color::rgb(0, 0, 0), BlendMode::Clear);
    }

    let center = size / 2.0;
    let hole = size * 0.18;
    let mut ring = PathBuilder::new();
    ring.push_circle(center, center, hole + stroke);
    let mut inner = PathBuilder::new();
    inner.push_circle(center, center, hole);
    if let (Some(ring), Some(inner)) = (ring.finish(), inner.finish()) {
        layer.fill_path(&ring, color, BlendMode::SourceOver);
        layer.fill_path(&inner, Color::rgb(0, 0, 0), BlendMode::Clear);
    }
    canvas.blend_pixmap(x.round() as i32, y.round() as i32, &layer.into_pixmap());
}

fn gear_outline(size: f32, inset: f32) -> Option<tiny_skia::Path> {
    let center = size / 2.0;
    let outer = size * 0.48 - inset;
    let inner = size * 0.36 - inset;
    if inner <= 0.0 {
        return None;
    }
    let step = TAU / TEETH as f32;
    let root_half = step * 0.28;
    let tip_half = step * 0.18;
    let mut path = PathBuilder::new();
    for i in 0..TEETH {
        let mid = i as f32 * step;
        for (j, (angle, radius)) in [
            (mid - root_half, inner),
            (mid - tip_half, outer),
            (mid + tip_half, outer),
            (mid + root_half, inner),
        ]
        .into_iter()
        .enumerate()
        {
            let px = center + radius * angle.cos();
            let py = center + radius * angle.sin();
            if i == 0 && j == 0 {
                path.move_to(px, py);
            } else {
                path.line_to(px, py);
            }
        }
    }
    path.close();
    path.finish()
}
