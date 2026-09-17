//! 各页共用的表单零件：设置卡片（图标 + 标签 + 说明 + 控件）、分组、说明小字、页外壳。
//! 排版取值一律照抄 WinUI 版 `settings/src/panel/controls/mod.rs`，好并排比观感；开关在 [`toggle`]。

mod toggle;

use eframe::egui;

pub(crate) use self::toggle::toggle;
use crate::theme;

/// 左侧导航栏宽度。
pub(crate) const NAV_WIDTH: f32 = 220.0;

/// 品牌名字号。
pub(crate) const BRAND_SIZE: f32 = 20.0;

/// 导航栏顶部 Logo 边长。
pub(crate) const LOGO_SIZE: f32 = 44.0;

/// 页面大标题字号。
const TITLE_SIZE: f32 = 28.0;

/// 分组标题字号。
const GROUP_SIZE: f32 = 16.0;

/// 设置项标签字号。
pub(crate) const LABEL_SIZE: f32 = 14.0;

/// 说明小字字号，与候选窗口的译文 / 词性同一档。
pub(crate) const NOTE_SIZE: f32 = 12.0;

/// 说明小字的淡化程度。
pub(crate) const NOTE_OPACITY: f32 = 0.65;

/// 卡片圆角，与候选窗口的 `Theme::corner_radius` 一致。
pub(crate) const CARD_RADIUS: f32 = 8.0;

/// 卡片左右内边距。
pub(crate) const CARD_PADDING_X: f32 = 16.0;

/// 卡片上下内边距。
pub(crate) const CARD_PADDING_Y: f32 = 12.0;

/// 卡片里图标、文字、控件三列之间的间距。
const CARD_GAP: f32 = 16.0;

/// 同一组里相邻卡片之间的间距。
const ROW_GAP: f32 = 4.0;

/// 分组之间的间距。
const GROUP_GAP: f32 = 24.0;

/// 控件列宽度，让各行的下拉框 / 开关左边缘对齐。
pub(crate) const CONTROL_WIDTH: f32 = 260.0;

/// 文字列的最小宽度：窗口再窄也不把说明挤成一列字。
const MIN_TEXT_WIDTH: f32 = 180.0;

/// 灰色小字说明，可换行。
pub(crate) fn note(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(NOTE_SIZE)
            .color(theme::note_color(ui.ctx())),
    );
}

/// 一整项：卡片里放「图标 + 标签（下接说明）+ 控件」，控件靠右。`hint` 为空则不加说明。
///
/// 控件闭包要把自己的 `Response` 交回来：immediate mode 下控件本身没有名字，
/// 这里用 `labelled_by` 把它和左边的标签系在一起，读屏才念得出「双拼 组合框 小鹤双拼」
/// （不系的话 UIA 树里 ComboBox / CheckBox 的 Name 是空的）。
pub(crate) fn field(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    hint: &str,
    control: impl FnOnce(&mut egui::Ui) -> egui::Response,
) {
    theme::card_frame(ui.ctx()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.label(egui::RichText::new(icon).size(LABEL_SIZE + 2.0));
            ui.add_space(CARD_GAP - ui.spacing().item_spacing.x);
            let text_width = (ui.available_width() - CONTROL_WIDTH - CARD_GAP).max(MIN_TEXT_WIDTH);
            let label_id = ui
                .allocate_ui_with_layout(
                    egui::vec2(text_width, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        let response = ui.label(egui::RichText::new(label).size(LABEL_SIZE));
                        if !hint.is_empty() {
                            note(ui, hint);
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
    ui.add_space(ROW_GAP);
}

/// 一组设置：图标 + 组标题，下面是这一组的卡片。
pub(crate) fn group(ui: &mut egui::Ui, icon: &str, title: &str, rows: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).size(GROUP_SIZE));
        ui.label(egui::RichText::new(title).size(GROUP_SIZE).strong());
    });
    ui.add_space(8.0);
    rows(ui);
    ui.add_space(GROUP_GAP - ROW_GAP);
}

/// 一页外壳：可滚动 + 标题 / 一句话说明 + 若干分组 + 页脚。
pub(crate) fn page(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    groups: impl FnOnce(&mut egui::Ui),
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(title).size(TITLE_SIZE).strong());
        ui.add_space(4.0);
        note(ui, subtitle);
        ui.add_space(GROUP_GAP);
        groups(ui);
        footer(ui);
    });
}

/// 每页底部的本地隐私状态与品牌落款；薄荷点只表达「数据留在本机」。
fn footer(ui: &mut egui::Ui) {
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
