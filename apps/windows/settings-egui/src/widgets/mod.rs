//! 各页共用的表单零件：设置卡片（图标 + 标签 + 控件）、内容卡片、分组、页外壳。
//!
//! 与 WinUI 版最大的不同：**说明不常驻**。每项只占一行，说明挂成整行的悬停提示，
//! 标签后跟一个淡色 ⓘ 提示「这里有说明」——原来一屏放得下四五项，现在放得下十来项。

mod toggle;

use eframe::egui;

pub(crate) use self::toggle::toggle;
use crate::theme;

/// 左侧导航栏宽度。
pub(crate) const NAV_WIDTH: f32 = 200.0;

/// 品牌名字号。
pub(crate) const BRAND_SIZE: f32 = 19.0;

/// 导航栏顶部 Logo 边长。
pub(crate) const LOGO_SIZE: f32 = 40.0;

/// 页面大标题字号。
pub(crate) const TITLE_SIZE: f32 = 22.0;

/// 分组标题字号。
const GROUP_SIZE: f32 = 15.0;

/// 设置项标签字号。
pub(crate) const LABEL_SIZE: f32 = 14.0;

/// 说明小字字号，与候选窗口的译文 / 词性同一档。
pub(crate) const NOTE_SIZE: f32 = 12.0;

/// 说明小字的淡化程度。
pub(crate) const NOTE_OPACITY: f32 = 0.65;

/// 卡片圆角，与候选窗口的 `Theme::corner_radius` 一致。
pub(crate) const CARD_RADIUS: f32 = 8.0;

/// 卡片左右内边距。
pub(crate) const CARD_PADDING_X: f32 = 14.0;

/// 卡片上下内边距：一行高的设置卡片，比 WinUI 版的 12 紧一半还多。
pub(crate) const CARD_PADDING_Y: f32 = 5.0;

/// 卡片里图标、文字、控件三列之间的间距。
const CARD_GAP: f32 = 12.0;

/// 同一组里相邻卡片之间的间距。
const ROW_GAP: f32 = 3.0;

/// 分组之间的间距。
const GROUP_GAP: f32 = 18.0;

/// 控件列宽度，让各行的下拉框 / 开关左边缘对齐。
pub(crate) const CONTROL_WIDTH: f32 = 240.0;

/// 悬停提示的最大宽度，再宽一行字就读不动了。
const TIP_WIDTH: f32 = 360.0;

/// 「有说明可悬停」的记号（Segoe Fluent Icons 的 info）。
const INFO_MARK: &str = "\u{E946}";

/// 灰色小字说明，可换行。
pub(crate) fn note(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(NOTE_SIZE)
            .color(theme::note_color(ui.ctx())),
    );
}

/// 给一个 `Response` 挂上限宽的悬停提示；`hint` 为空就不挂。
pub(crate) fn hint(response: egui::Response, hint: &str) -> egui::Response {
    if hint.is_empty() {
        return response;
    }
    response.on_hover_ui(|ui| {
        ui.set_max_width(TIP_WIDTH);
        ui.label(egui::RichText::new(hint).size(NOTE_SIZE));
    })
}

/// 一整项：一行卡片放「图标 + 标签 + 控件」，控件靠右；说明进悬停提示。
///
/// 控件闭包要把自己的 `Response` 交回来：immediate mode 下控件本身没有名字，
/// 这里用 `labelled_by` 把它和左边的标签系在一起，读屏才念得出「双拼 组合框 小鹤双拼」。
pub(crate) fn field(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    tip: &str,
    control: impl FnOnce(&mut egui::Ui) -> egui::Response,
) {
    let card = theme::card_frame(ui.ctx()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            ui.label(egui::RichText::new(icon).size(LABEL_SIZE + 2.0));
            ui.add_space(CARD_GAP - ui.spacing().item_spacing.x);
            let text_width = (ui.available_width() - CONTROL_WIDTH - CARD_GAP).max(120.0);
            let label_id = ui
                .allocate_ui_with_layout(
                    egui::vec2(text_width, 0.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        let response = ui.label(egui::RichText::new(label).size(LABEL_SIZE));
                        if !tip.is_empty() {
                            ui.label(
                                egui::RichText::new(INFO_MARK)
                                    .size(NOTE_SIZE)
                                    .color(theme::note_color(ui.ctx()).gamma_multiply(0.7)),
                            );
                        }
                        response.id
                    },
                )
                .inner;
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.0),
                egui::Layout::right_to_left(egui::Align::Center),
                control,
            )
            .inner
            .labelled_by(label_id);
        });
    });
    hint(card.response, tip);
    ui.add_space(ROW_GAP);
}

/// 一张只有内容的卡片：标题 + 自绘内容（词库列表、统计表格、许可证这类自带内容的走它）。
pub(crate) fn block(ui: &mut egui::Ui, icon: &str, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    theme::card_frame(ui.ctx()).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(LABEL_SIZE + 2.0));
                ui.add_space(CARD_GAP - ui.spacing().item_spacing.x);
                ui.label(egui::RichText::new(title).size(LABEL_SIZE));
            });
            ui.add_space(6.0);
            body(ui);
        });
    });
    ui.add_space(ROW_GAP);
}

/// 一组设置：图标 + 组标题，下面是这一组的卡片。
pub(crate) fn group(ui: &mut egui::Ui, icon: &str, title: &str, rows: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).size(GROUP_SIZE));
        ui.label(egui::RichText::new(title).size(GROUP_SIZE).strong());
    });
    ui.add_space(6.0);
    rows(ui);
    ui.add_space(GROUP_GAP - ROW_GAP);
}

/// 一页外壳：可滚动 + 标题 / 一句话说明 + 页内容 + 页脚。
pub(crate) fn page(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    groups: impl FnOnce(&mut egui::Ui),
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(title).size(TITLE_SIZE).strong());
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(subtitle)
                    .size(NOTE_SIZE)
                    .color(theme::note_color(ui.ctx())),
            );
        });
        ui.add_space(14.0);
        groups(ui);
        footer(ui);
    });
}

/// 每页底部的本地隐私状态与品牌落款；薄荷点只表达「数据留在本机」。
fn footer(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 4.0, egui::Color32::from_rgb(85, 214, 194));
        note(ui, "数据只留在本机");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new("字在 · 更自在的输入")
                    .size(NOTE_SIZE)
                    .color(theme::note_color(ui.ctx()).gamma_multiply(0.8)),
            );
        });
    });
}
