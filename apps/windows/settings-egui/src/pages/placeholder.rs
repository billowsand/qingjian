//! 未复刻的四页：只画页外壳与一句说明，让导航切换与滚动都能试。

use eframe::egui;

use crate::app::PAGES;
use crate::widgets::page;

pub(crate) fn view(ui: &mut egui::Ui, tag: &str) {
    let title = PAGES
        .iter()
        .find(|(t, _, _)| *t == tag)
        .map(|(_, label, _)| *label)
        .unwrap_or(tag);
    page(ui, title, "spike 只复刻「通用」页，这页是占位", |_| {});
}
