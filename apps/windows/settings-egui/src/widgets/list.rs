//! 设置列表：一页一张卡片，里面一行一项，行与行之间一条淡分隔线（从文字列起，不顶到图标下面）。
//!
//! 排版规则集中在这里，每一行都遵守：
//! - 行高固定 [`ROW_HEIGHT`]，图标、标签、控件三者**垂直居中对齐**——
//!   图标是私用区字形，`ascent` / `descent` 与正文字体不一样，靠布局居中会偏上，所以图标由 painter
//!   按 `Align2::CENTER_CENTER` 画在行的正中，与字形度量无关。
//! - 控件列右对齐、宽度固定 [`CONTROL_WIDTH`]，各行左边缘自然对齐。
//! - 说明不常驻，挂成整行的悬停提示，标签后跟一个淡色 ⓘ。

use eframe::egui;

use super::{CONTROL_WIDTH, ICON_COLUMN, INFO_MARK, LABEL_SIZE, NOTE_SIZE, hint};
use crate::fonts;
use crate::theme;

/// 一行的高度：放得下一个下拉框，又不至于像 WinUI 版那样一项占两行。
const ROW_HEIGHT: f32 = 38.0;

/// 图标字号。
const ICON_SIZE: f32 = 16.0;

/// 图标往下挪这么一点：Segoe Fluent 的字形视觉重心比汉字高，两边都「居中」时看着仍差一线。
const ICON_NUDGE: f32 = 1.0;

/// 卡片内的行构建器。
pub(crate) struct List<'a> {
    ui: &'a mut egui::Ui,

    /// 图标列宽；0 表示这张卡不要图标列（词库那种每行图标都一样的列表）。
    /// 分隔线也从这里开始，与标签左边缘对齐。
    icon_column: f32,

    /// 第一行上面不画分隔线。
    first: bool,
}

impl<'a> List<'a> {
    pub(super) fn new(ui: &'a mut egui::Ui, icons: bool) -> Self {
        Self {
            ui,
            icon_column: if icons { ICON_COLUMN } else { 0.0 },
            first: true,
        }
    }

    /// 一项：图标 + 标签（+ ⓘ）+ 右侧控件；`tip` 为空就没有悬停提示。
    ///
    /// 控件闭包交回自己的 `Response`，这里用 `labelled_by` 把它系到标签上，读屏才念得出名字。
    pub(crate) fn row(
        &mut self,
        icon: &str,
        label: &str,
        tip: &str,
        control: impl FnOnce(&mut egui::Ui) -> egui::Response,
    ) {
        self.separator();
        let width = self.ui.available_width();
        let icon_column = self.icon_column;
        let response = self
            .ui
            .allocate_ui_with_layout(
                egui::vec2(width, ROW_HEIGHT),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                    if icon_column > 0.0 {
                        paint_icon(ui, icon, icon_column);
                    }
                    // 提示挂在标签与 ⓘ 这两个 widget 上：整行的 response 会被行内的控件挡住，
                    // 鼠标停在行上不一定算 hover，挂在标签上才稳。
                    let label_response = ui.label(egui::RichText::new(label).size(LABEL_SIZE));
                    let label_id = label_response.id;
                    hint(label_response, tip);
                    if !tip.is_empty() {
                        let mark = ui.label(
                            egui::RichText::new(INFO_MARK)
                                .font(fonts::icon_font(NOTE_SIZE))
                                .color(theme::note_color(ui.ctx()).gamma_multiply(0.7)),
                        );
                        hint(mark, tip);
                    }
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width().max(CONTROL_WIDTH), ROW_HEIGHT),
                        egui::Layout::right_to_left(egui::Align::Center),
                        control,
                    )
                    .inner
                    .labelled_by(label_id);
                },
            )
            .response;
        hint(response, tip);
    }

    /// 一行自定义内容（词库列表、统计表这种自带排版的），照样吃行距与分隔线。
    pub(crate) fn custom(&mut self, body: impl FnOnce(&mut egui::Ui)) {
        self.separator();
        let width = self.ui.available_width();
        self.ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.add_space(6.0);
                body(ui);
                ui.add_space(6.0);
            },
        );
    }

    fn separator(&mut self) {
        if self.first {
            self.first = false;
            return;
        }
        let width = self.ui.available_width();
        let (rect, _) = self
            .ui
            .allocate_exact_size(egui::vec2(width, 1.0), egui::Sense::hover());
        self.ui.painter().hline(
            (rect.left() + self.icon_column)..=rect.right(),
            rect.center().y,
            egui::Stroke::new(1.0, theme::separator_color(self.ui.ctx())),
        );
    }
}

/// 图标画在固定宽的一列里、按字形外框居中——不跟着字体的基线走。
fn paint_icon(ui: &mut egui::Ui, icon: &str, column: f32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(column - ui.spacing().item_spacing.x, ROW_HEIGHT),
        egui::Sense::hover(),
    );
    if icon.is_empty() {
        return;
    }
    ui.painter().text(
        rect.center() + egui::vec2(0.0, ICON_NUDGE),
        egui::Align2::CENTER_CENTER,
        icon,
        fonts::icon_font(ICON_SIZE),
        theme::icon_color(ui.ctx()),
    );
}
