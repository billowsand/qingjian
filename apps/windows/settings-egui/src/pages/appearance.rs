//! 「候选窗口」页：主题预览、明暗、排布、渲染引擎、字体、拼音显示位置、悬浮状态条。
//! 预览直接用渲染器的 [`Palette`] 画，和候选窗同一套语义色。

use eframe::egui;
use qingjian_platform::{CandidateRenderer, LayoutMode, PreeditMode, ThemeMode};
use qingjian_render::{Color as RenderColor, Palette, shuangpin_mark};

use crate::app::Settings;
use crate::widgets::{
    CONTROL_WIDTH, LABEL_SIZE, NOTE_SIZE, block, field, group, note, page, toggle,
};

/// 「系统字体」项的下标：列表第 0 项，对应配置里的空串。
const SYSTEM_FONT: usize = 0;

/// 预览卡片的宽度。
const PREVIEW_WIDTH: f32 = 250.0;

pub(crate) fn view(settings: &mut Settings, ui: &mut egui::Ui) {
    page(ui, "候选窗口", "自如展现，输入更高效", |ui| {
        preview_block(settings, ui);
        window_group(settings, ui);
        status_group(settings, ui);
    });
}

fn preview_block(settings: &Settings, ui: &mut egui::Ui) {
    let mode = settings.config.general.theme;
    block(ui, "\u{E890}", "字在 · 钴蓝薄荷", |ui| {
        ui.horizontal(|ui| {
            for (label, palette) in variants(mode) {
                ui.vertical(|ui| {
                    note(ui, label);
                    ui.add_space(4.0);
                    candidate_preview(ui, palette);
                    ui.add_space(6.0);
                    status_preview(ui, settings, palette);
                });
                ui.add_space(12.0);
            }
        });
    });
}

/// 「跟随系统」两张都画，固定明暗各画一张。
fn variants(mode: ThemeMode) -> Vec<(&'static str, Palette)> {
    match mode {
        ThemeMode::System => vec![("浅色", Palette::light()), ("深色", Palette::dark())],
        ThemeMode::Light => vec![("浅色", Palette::light())],
        ThemeMode::Dark => vec![("深色", Palette::dark())],
    }
}

fn color32(color: RenderColor) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

fn text(size: f32, color: RenderColor, value: &str) -> egui::RichText {
    egui::RichText::new(value).size(size).color(color32(color))
}

/// 候选窗预览：拼音行 + 一条选中的候选（序号、词、译文）。
fn candidate_preview(ui: &mut egui::Ui, palette: Palette) {
    egui::Frame::NONE
        .fill(color32(palette.background))
        .stroke(egui::Stroke::new(1.0, color32(palette.highlight)))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::same(9))
        .show(ui, |ui| {
            ui.set_width(PREVIEW_WIDTH);
            ui.horizontal(|ui| {
                ui.label(text(13.0, palette.text, "ni'hao"));
                let (rect, _) = ui.allocate_exact_size(egui::vec2(2.0, 16.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 0.0, color32(palette.caret));
            });
            ui.add_space(5.0);
            egui::Frame::NONE
                .fill(color32(palette.highlight))
                .corner_radius(5.0)
                .inner_margin(egui::Margin::symmetric(7, 4))
                .show(ui, |ui| {
                    ui.set_width(PREVIEW_WIDTH - 18.0);
                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(2.0, 18.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 1.0, color32(palette.accent));
                        ui.label(text(11.0, palette.index, "1"));
                        ui.label(text(16.0, palette.text, "你好"));
                        ui.label(text(12.0, palette.gloss, "int. hello"));
                    });
                });
        });
}

/// 悬浮状态条预览：点阵 · 中 / 英方章（带双拼字）· 标点 · 齿轮。
fn status_preview(ui: &mut egui::Ui, settings: &Settings, palette: Palette) {
    let general = &settings.config.general;
    let full_width = general.full_width_punctuation;
    egui::Frame::NONE
        .fill(color32(palette.background))
        .stroke(egui::Stroke::new(1.0, color32(palette.highlight)))
        .corner_radius(11.0)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                grip(ui, palette);
                separator(ui, palette);
                mode_badge(ui, palette);
                if let Some(mark) = shuangpin_mark(&general.shuangpin) {
                    ui.label(text(15.0, palette.text, mark));
                }
                separator(ui, palette);
                let punctuation = if full_width { "，。" } else { ",." };
                let tone = if full_width {
                    palette.accent
                } else {
                    palette.gloss
                };
                ui.label(text(14.0, tone, punctuation));
                separator(ui, palette);
                ui.label(text(15.0, palette.gloss, "⚙"));
            });
        });
}

/// 左端的拖动点阵（3 行 2 列）：盲文点阵字符雅黑与 Segoe Fluent 都没有，会显示成方框，所以自己画。
fn grip(ui: &mut egui::Ui, palette: Palette) {
    const RADIUS: f32 = 1.6;
    const STEP: f32 = 5.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 20.0), egui::Sense::hover());
    let painter = ui.painter();
    let color = color32(palette.pos);
    let origin = rect.center() - egui::vec2(STEP / 2.0, STEP);
    for row in 0..3 {
        for column in 0..2 {
            let center = origin + egui::vec2(column as f32 * STEP, row as f32 * STEP);
            painter.circle_filled(center, RADIUS, color);
        }
    }
}

fn separator(ui: &mut egui::Ui, palette: Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 20.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, 0.0, color32(palette.pos).gamma_multiply(0.45));
}

/// 钴蓝模式方章：中文态右上角的薄荷点表示输入与学习都在本地生效。
fn mode_badge(ui: &mut egui::Ui, palette: Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 8.0, color32(palette.accent));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "中",
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );
    painter.circle_filled(
        egui::pos2(rect.right() - 6.0, rect.top() + 6.0),
        3.5,
        color32(palette.caret),
    );
}

fn window_group(settings: &mut Settings, ui: &mut egui::Ui) {
    group(ui, "\u{E890}", "候选窗口", |ui| {
        let theme = settings.config.general.theme;
        field(
            ui,
            "\u{E793}",
            "明暗模式",
            "跟随系统会在 Windows 切换浅色或深色后自动使用相应的字在主题。",
            |ui| {
                let (response, picked) =
                    mode_combo(ui, "theme", &ThemeMode::ALL, theme, ThemeMode::label);
                if let Some(mode) = picked {
                    settings.save("general", "theme", mode.key());
                }
                response
            },
        );
        let layout = settings.config.general.layout;
        field(ui, "\u{EA37}", "排布", "", |ui| {
            let (response, picked) =
                mode_combo(ui, "layout", &LayoutMode::ALL, layout, LayoutMode::label);
            if let Some(mode) = picked {
                settings.save("general", "layout", mode.key());
            }
            response
        });
        let renderer = settings.config.general.renderer;
        field(
            ui,
            "\u{EB9F}",
            "渲染引擎",
            "字在渲染器使用统一的品牌主题，并让候选窗口在各平台保持一致。",
            |ui| {
                let (response, picked) = mode_combo(
                    ui,
                    "renderer",
                    &CandidateRenderer::ALL,
                    renderer,
                    CandidateRenderer::label,
                );
                if let Some(mode) = picked {
                    settings.save("general", "renderer", mode.key());
                }
                response
            },
        );
        let (options, selected) = font_options(settings);
        field(
            ui,
            "\u{E8D2}",
            "字体",
            "只对字在渲染器生效；下拉列表是系统里装的字体，选「系统字体」用默认；配置里的字体没装时自动回到系统字体。设置界面本身用 Windows 的界面字体，不跟这里走。",
            |ui| {
                let (response, picked) = font_combo(ui, &options, selected);
                match picked {
                    Some(SYSTEM_FONT) => settings.save("general", "font", ""),
                    Some(index) => {
                        let family = options[index].clone();
                        settings.save("general", "font", family);
                    }
                    None => {}
                }
                response
            },
        );
        let preedit = settings.config.general.preedit;
        field(
            ui,
            "\u{E7B3}",
            "拼音显示",
            "「只在候选窗口」时正在敲的拼音不显示在应用里，终端或行内拼音不正常的应用可以选它。",
            |ui| {
                let (response, picked) = mode_combo(
                    ui,
                    "preedit",
                    &PreeditMode::ALL,
                    preedit,
                    PreeditMode::label,
                );
                if let Some(mode) = picked {
                    settings.save("general", "preedit", mode.key());
                }
                response
            },
        );
    });
}

fn status_group(settings: &mut Settings, ui: &mut egui::Ui) {
    group(ui, "\u{E784}", "悬浮状态条", |ui| {
        let mut enabled = settings.config.status_bar.enabled;
        field(
            ui,
            "\u{E713}",
            "显示悬浮状态条",
            "桌面上常驻、可拖动的小条：从左侧点阵处拖动，点模式方章切换中 / 英，点「，。」切全角 / 半角标点，点齿轮打开设置。只在当前输入法是字在时显示，拖到哪下次还在哪。",
            |ui| {
                let response = toggle(ui, &mut enabled, "显示悬浮状态条");
                if response.changed() {
                    settings.save("status_bar", "enabled", enabled);
                }
                response
            },
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(
                "双拼方案在窄状态条上收成单字：小鹤「鹤」、自然码「自」、微软「微」、搜狗「搜」。",
            )
            .size(NOTE_SIZE)
            .color(crate::theme::note_color(ui.ctx())),
        );
    });
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
    options.push("系统字体（默认）".to_owned());
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
        .height(320.0)
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
