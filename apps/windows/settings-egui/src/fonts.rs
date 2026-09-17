//! 中文界面字体与图标字体：egui 自带的字体没有汉字，要自己从系统装。
//! 文件路径走渲染器已有的 `system_fonts::family_files`（DirectWrite 按字族名查），不打包字体文件。

use std::sync::Arc;

use eframe::egui;
use qingjian_render::system_fonts;

/// 界面字体候选，按顺序试第一个装着的。
/// 雅黑在 `msyh.ttc` 里，face 0 是 Microsoft YaHei（Win10 的系统 UI 字体），
/// Microsoft YaHei UI 是同一文件的另一个 face，spike 不区分。
const UI_FAMILIES: [&str; 3] = ["Microsoft YaHei", "Microsoft JhengHei", "SimSun"];

/// 图标字体：Win11 是 Segoe Fluent Icons，Win10 只有 Segoe MDL2 Assets（私用区码点大部分通用）。
const ICON_FAMILIES: [&str; 2] = ["Segoe Fluent Icons", "Segoe MDL2 Assets"];

/// 把中文字体排到 Proportional / Monospace 两个家族的**第一位**，图标字体排最后当回退。
pub(crate) fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let chinese = load(&UI_FAMILIES).map(|data| {
        fonts.font_data.insert("ui".to_owned(), Arc::new(data));
        "ui".to_owned()
    });
    let icons = load(&ICON_FAMILIES).map(|data| {
        fonts.font_data.insert("icons".to_owned(), Arc::new(data));
        "icons".to_owned()
    });
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        let list = fonts.families.entry(family).or_default();
        if let Some(name) = chinese.clone() {
            list.insert(0, name);
        }
        if let Some(name) = icons.clone() {
            list.push(name);
        }
    }
    ctx.set_fonts(fonts);
}

/// 按顺序找第一个装着的字族，读它的第一个字体文件。
fn load(families: &[&str]) -> Option<egui::FontData> {
    for family in families {
        let Some(file) = system_fonts::family_files(family).into_iter().next() else {
            continue;
        };
        match std::fs::read(&file) {
            Ok(bytes) => return Some(egui::FontData::from_owned(bytes)),
            Err(error) => eprintln!("读字体 {} 失败: {error}", file.display()),
        }
    }
    eprintln!("系统里没找到这些字族: {families:?}");
    None
}
