//! 悬浮状态条（Windows）：几格并排的小条 `[Logo][中 / A][，。/ ,.][⚙]`，每格内容居中、格间一条细线，圆角背景加阴影。

mod cell;
mod rendered;

pub use cell::{StatusCell, shuangpin_mark};
pub use rendered::RenderedStatus;

use super::{Metrics, Rendered, Renderer};
use crate::brand_mark::draw_brand_mark;
use crate::canvas::Canvas;
use crate::error::RenderError;
use crate::gear::draw_gear;
use crate::shadow::Shadow;
use crate::theme::{FontSpec, Theme};

/// 设置入口齿轮的边长（点）。
const GEAR_SIZE: f32 = 15.0;

/// 左侧拖拽 Logo 的边长（点）：设计稿中约占 34 pt 条高的三分之二。
const BRAND_MARK_SIZE: f32 = 23.0;

/// 状态条固定内容高度（点）。
const STATUS_HEIGHT: f32 = 34.0;

/// 中 / A 与双拼单字标记的间距（点）。
const MODE_SCHEME_GAP: f32 = 6.0;

/// 英文态仍保留与设计稿一致的模式格宽度，不让 A 两侧显得拥挤。
const MODE_MIN_CONTENT_WIDTH: f32 = 26.0;

/// 标点在设计稿里比模式文字小一档；最小宽度让半角 `,.` 两侧仍有稳定留白。
const PUNCTUATION_FONT: FontSpec = FontSpec::new(14.0, 17.0);
const PUNCTUATION_MIN_CONTENT_WIDTH: f32 = 25.0;

/// 格间细线的宽度（点）。
const SEPARATOR_WIDTH: f32 = 1.0;

impl Renderer {
    /// 画状态条。每格宽 = 内容宽 + 两侧内边距，高 = 候选词行高 + 内边距；返回位图与各格右边界（供点击命中）。
    pub fn render_status(
        &mut self,
        cells: &[StatusCell],
        theme: &Theme,
        scale: f32,
        shadow: Option<&Shadow>,
    ) -> Result<RenderedStatus, RenderError> {
        let metrics = Metrics { theme, scale };
        let padding = metrics.padding();
        let line_height = metrics.px(theme.text_font.line_height);
        let mut widths: Vec<f32> = cells
            .iter()
            .map(|cell| self.status_cell_width(cell, &metrics) + padding * 2.0)
            .collect();
        let total: f32 = widths.iter().sum();
        let content_width = total.ceil();
        // 取整多出来的零头给最后一格，让最后一格的右边界正好是内容宽
        if let Some(last) = widths.last_mut() {
            *last += content_width - total;
        }
        let content_height = (line_height + padding)
            .max(metrics.px(STATUS_HEIGHT))
            .ceil();
        let margin = shadow.map_or(0.0, |s| metrics.px(s.margin()));
        let width = (content_width + margin * 2.0).ceil();
        let height = (content_height + margin * 2.0).ceil();
        let mut canvas = Canvas::new(width as u32, height as u32)?;
        let radius = metrics.corner_radius();
        if let Some(shadow) = shadow
            && let Some(content) =
                tiny_skia::Rect::from_xywh(margin, margin, content_width, content_height)
        {
            shadow.paint(&mut canvas, content, radius, scale);
        }
        canvas.fill_round_rect(
            margin,
            margin,
            content_width,
            content_height,
            radius,
            theme.colors.background,
        );
        let inset = padding / 2.0;
        let mut x = margin;
        let mut edges = Vec::with_capacity(cells.len());
        for (i, (cell, width)) in cells.iter().zip(&widths).enumerate() {
            if i > 0 {
                canvas.fill_rect(
                    x,
                    margin + inset,
                    metrics.px(SEPARATOR_WIDTH),
                    content_height - inset * 2.0,
                    theme.colors.pos,
                );
            }
            let slot = (x, margin, *width, content_height);
            self.draw_status_cell(&mut canvas, cell, &metrics, slot);
            x += width;
            edges.push(x - margin);
        }
        Ok(RenderedStatus {
            rendered: Rendered {
                pixmap: canvas.into_pixmap(),
                content_x: margin as u32,
                content_y: margin as u32,
                content_width: content_width as u32,
                content_height: content_height as u32,
                scale,
            },
            cell_edges: edges,
        })
    }

    /// 一格内容的宽度（像素，不含内边距）。
    fn status_cell_width(&mut self, cell: &StatusCell, m: &Metrics) -> f32 {
        match cell {
            StatusCell::Logo => m.px(BRAND_MARK_SIZE),
            StatusCell::Mode { text, scheme } => {
                let style = m.text_style();
                let text_width = self.measure(text, &style).width;
                let scheme_width = scheme.as_ref().map_or(0.0, |scheme| {
                    m.px(MODE_SCHEME_GAP) + self.measure(scheme, &style).width
                });
                (text_width + scheme_width).max(m.px(MODE_MIN_CONTENT_WIDTH))
            }
            StatusCell::Text { text, .. } => {
                let style = m.style(PUNCTUATION_FONT, m.theme.colors.text);
                self.measure(text, &style)
                    .width
                    .max(m.px(PUNCTUATION_MIN_CONTENT_WIDTH))
            }
            StatusCell::Gear => m.px(GEAR_SIZE),
        }
    }

    /// 在 `slot = (x, y, 宽, 高)` 的格子里居中画一格。
    fn draw_status_cell(
        &mut self,
        canvas: &mut Canvas,
        cell: &StatusCell,
        m: &Metrics,
        slot: (f32, f32, f32, f32),
    ) {
        let (x, y, width, height) = slot;
        match cell {
            StatusCell::Logo => {
                let size = m.px(BRAND_MARK_SIZE);
                draw_brand_mark(
                    canvas,
                    x + (width - size) / 2.0,
                    y + (height - size) / 2.0,
                    size,
                    m.theme,
                );
            }
            StatusCell::Mode { text, scheme } => {
                let style = m.text_style();
                let gap = m.px(MODE_SCHEME_GAP);
                let text_size = self.measure(text, &style);
                let scheme_width = scheme
                    .as_ref()
                    .map_or(0.0, |scheme| self.measure(scheme, &style).width);
                let content_width =
                    text_size.width + scheme.as_ref().map_or(0.0, |_| gap + scheme_width);
                let left = x + (width - content_width) / 2.0;
                let top = y + (height - text_size.height) / 2.0;
                self.draw_text(canvas, text, &style, left, top);

                if let Some(scheme) = scheme {
                    let size = self.measure(scheme, &style);
                    self.draw_text(
                        canvas,
                        scheme,
                        &style,
                        left + text_size.width + gap,
                        y + (height - size.height) / 2.0,
                    );
                }
            }
            StatusCell::Text { text, emphasized } => {
                let color = if *emphasized {
                    m.theme.colors.accent
                } else {
                    m.theme.colors.gloss
                };
                let style = m.style(PUNCTUATION_FONT, color);
                let size = self.measure(text, &style);
                let left = x + (width - size.width) / 2.0;
                let top = y + (height - size.height) / 2.0;
                self.draw_text(canvas, text, &style, left, top);
            }
            StatusCell::Gear => {
                let size = m.px(GEAR_SIZE);
                draw_gear(
                    canvas,
                    x + (width - size) / 2.0,
                    y + (height - size) / 2.0,
                    size,
                    m.theme.colors.gloss,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StatusCell;
    use crate::fonts::FontLibrary;
    use crate::renderer::Renderer;
    use crate::shadow::Shadow;
    use crate::theme::Theme;

    #[test]
    fn cells_have_increasing_edges_ending_at_content_width() {
        // 没有系统字体的环境（CI 容器）跳过
        let Ok(library) = FontLibrary::system("zh-CN") else {
            return;
        };
        let mut renderer = Renderer::new(library);
        let cells = [
            StatusCell::Logo,
            StatusCell::mode("中", Some("鹤")),
            StatusCell::text(",.", false),
            StatusCell::Gear,
        ];
        let out = renderer
            .render_status(&cells, &Theme::light(), 2.0, Some(&Shadow::mac_panel()))
            .unwrap();
        assert_eq!(out.cell_edges.len(), 4);
        assert!(out.cell_edges.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            out.cell_edges.last().map(|edge| edge.round() as u32),
            Some(out.rendered.content_width)
        );
        assert!(out.rendered.pixmap.width() > out.rendered.content_width);
        assert!(out.rendered.content_x > 0);
    }

    #[test]
    fn logo_mode_and_gear_render_in_every_theme_and_appearance() {
        let Ok(library) = FontLibrary::system("zh-CN") else {
            return;
        };
        let mut renderer = Renderer::new(library);
        for theme in [
            Theme::cream(false),
            Theme::cream(true),
            Theme::zizai(false),
            Theme::zizai(true),
            Theme::latte(false),
            Theme::latte(true),
            Theme::forest(false),
            Theme::forest(true),
        ] {
            let out = renderer
                .render_status(
                    &[
                        StatusCell::Logo,
                        StatusCell::mode("中", None::<&str>),
                        StatusCell::Gear,
                    ],
                    &theme,
                    2.0,
                    None,
                )
                .unwrap();
            assert_eq!(out.cell_edges.len(), 3);
            assert!(out.rendered.content_width > 0);
            assert!(out.rendered.content_height > 0);
        }
    }

    #[test]
    fn approved_english_layout_keeps_the_designed_spacing() {
        let Ok(library) = FontLibrary::system("zh-CN") else {
            return;
        };
        let mut renderer = Renderer::new(library);
        let out = renderer
            .render_status(
                &[
                    StatusCell::Logo,
                    StatusCell::mode("A", None::<&str>),
                    StatusCell::text(",.", false),
                    StatusCell::Gear,
                ],
                &Theme::zizai(false),
                1.0,
                None,
            )
            .unwrap();
        let edges: Vec<u32> = out
            .cell_edges
            .iter()
            .map(|edge| edge.round() as u32)
            .collect();
        assert_eq!(edges, [39, 81, 122, 153]);
        assert_eq!(out.rendered.content_height, 34);
    }
}
