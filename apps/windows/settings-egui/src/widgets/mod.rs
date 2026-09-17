//! 各页共用的表单零件：页外壳、设置列表卡片、悬停提示、小字。
//!
//! 版式的三条规矩（与 WinUI 版最大的不同）：
//! 1. **一页一张列表卡**，行与行之间只有一条淡分隔线；页内不再分「输入方案 / 按键 / 标点」这类小节——
//!    分节已经在左侧导航里了，右边直接列选项。
//! 2. **说明不常驻**：挂成整行的悬停提示，标签后一个淡色 ⓘ。
//! 3. **一行三列**：图标列定宽、标签、控件右对齐定宽；三者垂直居中（细节见 [`list`]）。

mod list;
mod toggle;

use eframe::egui;

pub(crate) use self::list::List;
pub(crate) use self::toggle::toggle;
use crate::theme;

/// 左侧导航栏宽度。
pub(crate) const NAV_WIDTH: f32 = 168.0;

/// 品牌名字号。
pub(crate) const BRAND_SIZE: f32 = 16.0;

/// 导航栏顶部 Logo 边长。
pub(crate) const LOGO_SIZE: f32 = 30.0;

/// 页面标题字号。
pub(crate) const TITLE_SIZE: f32 = 20.0;

/// 正文与标签字号。
pub(crate) const LABEL_SIZE: f32 = 13.5;

/// 说明小字字号。
pub(crate) const NOTE_SIZE: f32 = 12.0;

/// 说明小字的淡化程度。
pub(crate) const NOTE_OPACITY: f32 = 0.72;

/// 卡片圆角，与候选窗口的 `Theme::corner_radius` 一致。
pub(crate) const CARD_RADIUS: f32 = 8.0;

/// 卡片左右内边距。
pub(crate) const CARD_PADDING_X: f32 = 14.0;

/// 卡片上下内边距：行高本身够大，卡片只留一点点。
pub(crate) const CARD_PADDING_Y: f32 = 5.0;

/// 图标列宽（含与标签之间的间距）。
pub(crate) const ICON_COLUMN: f32 = 28.0;

/// 控件列宽度：下拉框都这么宽，各行左边缘就对齐了。窄窗口下够放「全拼（不启用双拼）」。
pub(crate) const CONTROL_WIDTH: f32 = 184.0;

/// 悬停提示的最大宽度，再宽一行字就读不动了。
const TIP_WIDTH: f32 = 320.0;

/// 「有说明可悬停」的记号（Segoe Fluent Icons 的 info）。
const INFO_MARK: &str = "\u{E946}";

/// 灰色小字，可换行。
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

/// 一张列表卡片，里面用 [`List::row`] 一行行往下放。
pub(crate) fn list(ui: &mut egui::Ui, rows: impl FnOnce(&mut List)) {
    card(ui, true, rows);
}

/// 不要图标列的列表卡片：词库那种每行图标都一样的列表，留着图标只是噪音。
pub(crate) fn plain_list(ui: &mut egui::Ui, rows: impl FnOnce(&mut List)) {
    card(ui, false, rows);
}

fn card(ui: &mut egui::Ui, icons: bool, rows: impl FnOnce(&mut List)) {
    theme::card_frame(ui.ctx()).show(ui, |ui| {
        ui.vertical(|ui| {
            let mut list = List::new(ui, icons);
            rows(&mut list);
        });
    });
    ui.add_space(10.0);
}

/// 卡片上方的小标题，需要把一页分成两张卡时用（「词库」页的随包 / 导入）。
pub(crate) fn caption(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(NOTE_SIZE)
            .color(theme::note_color(ui.ctx())),
    );
    ui.add_space(4.0);
}

/// 一页外壳：标题 + 可滚动的内容。
pub(crate) fn page(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    ui.label(egui::RichText::new(title).size(TITLE_SIZE).strong());
    ui.add_space(12.0);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width() - 4.0);
            body(ui);
        });
}
