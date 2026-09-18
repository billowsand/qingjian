//! 「候选窗口」页：明暗、每页候选数、字体、拼音显示、悬浮状态条。
//! 候选窗长什么样打字时就看见了，这里不再摆一份预览图。

use eframe::egui;
use qingjian_platform::{MAX_PAGE_SIZE, PreeditMode, ThemeMode};

use crate::app::Settings;
use crate::fonts;
use crate::widgets::{CONTROL_WIDTH, LABEL_SIZE, list, page, toggle};

/// 「系统字体」项的下标：列表第 0 项，对应配置里的空串。
const SYSTEM_FONT: usize = 0;

pub(crate) fn view(settings: &mut Settings, ui: &mut egui::Ui) {
    page(ui, "候选窗口", |ui| {
        list(ui, |list| {
            let theme = settings.config.general.theme;
            list.row(
                "\u{E793}",
                "明暗",
                "跟随系统会在 Windows 切换浅色或深色后自动换成相应的字在主题。",
                |ui| {
                    let (response, picked) =
                        mode_combo(ui, "theme", &ThemeMode::ALL, theme, ThemeMode::label);
                    if let Some(mode) = picked {
                        settings.save("general", "theme", mode.key());
                    }
                    response
                },
            );
            let page_size = settings.config.general.page_size;
            list.row("\u{EA37}", "每页候选数", "", |ui| {
                let (response, picked) = page_size_combo(ui, page_size);
                if let Some(size) = picked {
                    settings.save("general", "page_size", size as i64);
                }
                response
            });
            let (options, selected) = font_options(settings);
            list.row(
                "\u{E8D2}",
                "字体",
                "候选窗口与设置界面共用此字体；没安装时自动回到系统字体。",
                |ui| {
                    let (response, picked) = font_combo(ui, &options, selected);
                    match picked {
                        Some(SYSTEM_FONT) => {
                            settings.save("general", "font", "");
                            fonts::install(ui.ctx(), &settings.config.general.font);
                        }
                        Some(index) => {
                            let family = options[index].clone();
                            settings.save("general", "font", family);
                            fonts::install(ui.ctx(), &settings.config.general.font);
                        }
                        None => {}
                    }
                    response
                },
            );
            let preedit = settings.config.general.preedit;
            list.row(
                "\u{E7B3}",
                "拼音显示",
                "「只在候选窗口」时正在敲的拼音不显示在应用里，终端或行内拼音不正常的应用可以选它。",
                |ui| {
                    let (response, picked) =
                        mode_combo(ui, "preedit", &PreeditMode::ALL, preedit, PreeditMode::label);
                    if let Some(mode) = picked {
                        settings.save("general", "preedit", mode.key());
                    }
                    response
                },
            );
            let mut enabled = settings.config.status_bar.enabled;
            list.row(
                "\u{E890}",
                "悬浮状态条",
                "桌面上常驻、可拖动的小条：点模式方章切换中 / 英，点「，。」切全角 / 半角标点，点齿轮打开设置。只在当前输入法是字在时显示。",
                |ui| {
                    let response = toggle(ui, &mut enabled, "悬浮状态条");
                    if response.changed() {
                        settings.save("status_bar", "enabled", enabled);
                    }
                    response
                },
            );
        });
    });
}

/// 每页候选数下拉：1–9。
fn page_size_combo(ui: &mut egui::Ui, current: usize) -> (egui::Response, Option<usize>) {
    let mut picked = None;
    let response = egui::ComboBox::from_id_salt("page-size")
        .width(CONTROL_WIDTH)
        .selected_text(egui::RichText::new(format!("{current} 个")).size(LABEL_SIZE))
        .show_ui(ui, |ui| {
            for size in 1..=MAX_PAGE_SIZE {
                if ui
                    .selectable_label(size == current, format!("{size} 个"))
                    .clicked()
                    && size != current
                {
                    picked = Some(size);
                }
            }
        })
        .response;
    (response, picked)
}

/// 枚举下拉：按 `label()` 列项，选中 `current`；返回控件 `Response` 与新选的枚举值。
fn mode_combo<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id: &str,
    all: &'static [T],
    current: T,
    label: fn(T) -> &'static str,
) -> (egui::Response, Option<T>) {
    let selected = all.iter().position(|mode| *mode == current).unwrap_or(0);
    let mut picked = None;
    let response = egui::ComboBox::from_id_salt(id)
        .width(CONTROL_WIDTH)
        .selected_text(egui::RichText::new(label(all[selected])).size(LABEL_SIZE))
        .show_ui(ui, |ui| {
            for (index, mode) in all.iter().enumerate() {
                if ui
                    .selectable_label(index == selected, label(*mode))
                    .clicked()
                    && index != selected
                {
                    picked = Some(*mode);
                }
            }
        })
        .response;
    (response, picked)
}

/// 字体下拉：第 0 项是「系统字体」，其余是系统里装的字族。
/// 配置里写了、但系统里没装的字族补在最后并选中它，免得下拉显示的和配置里存的对不上。
fn font_options(settings: &Settings) -> (Vec<String>, usize) {
    let configured = settings.config.general.font.trim();
    let mut options: Vec<String> = Vec::with_capacity(settings.families.len() + 1);
    options.push("系统字体".to_owned());
    options.extend(settings.families.iter().cloned());
    let mut selected = settings
        .families
        .iter()
        .position(|family| family.eq_ignore_ascii_case(configured))
        .map(|index| index + 1)
        .unwrap_or(SYSTEM_FONT);
    if selected == SYSTEM_FONT && !configured.is_empty() {
        options.push(configured.to_owned());
        selected = options.len() - 1;
    }
    (options, selected)
}

/// 字族下拉：项数上千，给它一个固定高度的滚动区，别把窗口撑满。
fn font_combo(
    ui: &mut egui::Ui,
    options: &[String],
    selected: usize,
) -> (egui::Response, Option<usize>) {
    let mut picked = None;
    let response = egui::ComboBox::from_id_salt("font")
        .width(CONTROL_WIDTH)
        .height(300.0)
        .selected_text(egui::RichText::new(&options[selected]).size(LABEL_SIZE))
        .show_ui(ui, |ui| {
            for (index, family) in options.iter().enumerate() {
                if ui.selectable_label(index == selected, family).clicked() && index != selected {
                    picked = Some(index);
                }
            }
        })
        .response;
    (response, picked)
}
