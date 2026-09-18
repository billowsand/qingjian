//! 设置界面与候选窗口共用字体选择；缺字与图标从系统字体回退，不打包字体文件。

use std::sync::Arc;

use eframe::egui;
use qingjian_render::system_fonts;

/// 与候选窗口 `FontLibrary::system` 相同的缺省界面字体顺序。
const DEFAULT_UI_FAMILIES: [&str; 2] = ["Segoe UI", "Arial"];

/// 与候选窗口 zh-CN 字体库相同的主要中日文字回退。
const SCRIPT_FAMILIES: [&str; 2] = ["Microsoft YaHei", "Yu Gothic"];

/// 图标字体：Win11 是 Segoe Fluent Icons，Win10 只有 Segoe MDL2 Assets（私用区码点大部分通用）。
const ICON_FAMILIES: [&str; 2] = ["Segoe Fluent Icons", "Segoe MDL2 Assets"];

/// 候选窗口所选字体排第一，系统缺省界面字体与中日文字体依次回退，图标字体排最后。
pub(crate) fn install(ctx: &egui::Context, configured: &str) {
    let mut fonts = egui::FontDefinitions::default();
    let mut selected = Vec::new();
    if !configured.trim().is_empty() {
        register(&mut fonts, &mut selected, "candidate", configured.trim());
    }
    register_first(
        &mut fonts,
        &mut selected,
        "candidate-default",
        &DEFAULT_UI_FAMILIES,
    );
    for (index, family) in SCRIPT_FAMILIES.into_iter().enumerate() {
        register(
            &mut fonts,
            &mut selected,
            &format!("script-{index}"),
            family,
        );
    }
    register_first(&mut fonts, &mut selected, "icons", &ICON_FAMILIES);
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        let list = fonts.families.entry(family).or_default();
        for name in selected.iter().rev() {
            list.insert(0, name.clone());
        }
    }
    ctx.set_fonts(fonts);
}

fn register_first(
    fonts: &mut egui::FontDefinitions,
    selected: &mut Vec<String>,
    key: &str,
    families: &[&str],
) {
    for family in families {
        if register(fonts, selected, key, family) {
            return;
        }
    }
    eprintln!("系统里没找到这些字族: {families:?}");
}

fn register(
    fonts: &mut egui::FontDefinitions,
    selected: &mut Vec<String>,
    key: &str,
    family: &str,
) -> bool {
    let Some((file, face_index)) = system_fonts::regular_family_face(family) else {
        return false;
    };
    match std::fs::read(&file) {
        Ok(bytes) => {
            let mut data = egui::FontData::from_owned(bytes);
            data.index = face_index;
            fonts.font_data.insert(key.to_owned(), Arc::new(data));
            selected.push(key.to_owned());
            true
        }
        Err(error) => {
            eprintln!("读字体 {} 失败: {error}", file.display());
            false
        }
    }
}
