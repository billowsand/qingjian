//! 「候选窗口」页：四套同步色系、每页候选数、字体、拼音显示、悬浮状态条。

use eframe::egui;
use qingjian_platform::{ColorScheme, MAX_PAGE_SIZE, PreeditMode};
use qingjian_render::{Color as RenderColor, Palette};

use crate::app::Settings;
use crate::fonts;
use crate::theme;
use crate::widgets::{CONTROL_WIDTH, LABEL_SIZE, list, note, page, toggle};

/// 「系统字体」项的下标：列表第 0 项，对应配置里的空串。
const SYSTEM_FONT: usize = 0;

pub(crate) fn view(settings: &mut Settings, ui: &mut egui::Ui) {
    page(ui, "候选窗口", |ui| {
        theme_picker(settings, ui);
        ui.add_space(10.0);
        list(ui, |list| {
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

fn theme_picker(settings: &mut Settings, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("配色").size(LABEL_SIZE).strong());
        note(ui, "明暗跟随 Windows");
    });
    ui.add_space(4.0);

    let current = settings.config.general.theme;
    let mut picked = None;
    for row in ColorScheme::ALL.chunks(2) {
        ui.columns(2, |columns| {
            for (column, scheme) in columns.iter_mut().zip(row.iter().copied()) {
                if theme_card(column, scheme, scheme == current).clicked() {
                    picked = Some(scheme);
                }
            }
        });
        ui.add_space(6.0);
    }
    if let Some(scheme) = picked {
        settings.save("general", "theme", scheme.key());
    }
}

fn theme_card(ui: &mut egui::Ui, scheme: ColorScheme, selected: bool) -> egui::Response {
    let base = theme::card_frame(ui.ctx());
    let stroke = if selected {
        egui::Stroke::new(1.5, theme::accent(ui.ctx()))
    } else {
        base.stroke
    };
    let shown = egui::Frame::NONE
        .fill(base.fill)
        .stroke(stroke)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(9, 7))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(scheme.label())
                        .size(LABEL_SIZE)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    radio_mark(ui, selected);
                });
            });
            ui.add_space(3.0);
            preview_strip(ui, "浅", theme::palette_for(scheme, false));
            ui.add_space(3.0);
            preview_strip(ui, "深", theme::palette_for(scheme, true));
        });
    let response = ui.interact(
        shown.response.rect,
        ui.id().with(("theme-card", scheme.key())),
        egui::Sense::click(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::RadioButton,
            ui.is_enabled(),
            selected,
            scheme.label(),
        )
    });
    response
}

fn radio_mark(ui: &mut egui::Ui, selected: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(15.0, 15.0), egui::Sense::hover());
    let color = if selected {
        theme::accent(ui.ctx())
    } else {
        theme::icon_color(ui.ctx())
    };
    ui.painter()
        .circle_stroke(rect.center(), 6.0, egui::Stroke::new(1.3, color));
    if selected {
        ui.painter().circle_filled(rect.center(), 3.0, color);
    }
}

/// 两条预览都由当前 egui 字体现场绘制；换字体后这里与真实候选窗一起更新。
fn preview_strip(ui: &mut egui::Ui, label: &str, palette: Palette) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 25.0), egui::Sense::hover());
    let label_width = 18.0;
    let panel = egui::Rect::from_min_max(
        egui::pos2(rect.left() + label_width, rect.top()),
        rect.right_bottom(),
    );
    let painter = ui.painter();
    painter.text(
        egui::pos2(rect.left(), rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(10.5),
        color32(palette.gloss),
    );
    painter.rect_filled(panel, 5.0, color32(palette.background));
    painter.rect_stroke(
        panel,
        5.0,
        egui::Stroke::new(1.0, color32(palette.pos).gamma_multiply(0.38)),
        egui::StrokeKind::Inside,
    );
    let preedit_x = panel.left() + 8.0;
    painter.text(
        egui::pos2(preedit_x, panel.center().y),
        egui::Align2::LEFT_CENTER,
        "ni'hao",
        egui::FontId::proportional(10.5),
        color32(palette.text),
    );
    let caret_x = preedit_x + 36.0;
    painter.vline(
        caret_x,
        (panel.top() + 5.0)..=(panel.bottom() - 5.0),
        egui::Stroke::new(1.5, color32(palette.caret)),
    );
    let candidate = egui::Rect::from_min_size(
        egui::pos2(caret_x + 7.0, panel.top() + 3.0),
        egui::vec2((panel.width() - 61.0).min(57.0), panel.height() - 6.0),
    );
    painter.rect_filled(candidate, 4.0, color32(palette.highlight));
    painter.rect_filled(
        egui::Rect::from_min_size(candidate.min, egui::vec2(2.0, candidate.height())),
        1.0,
        color32(palette.accent),
    );
    painter.text(
        candidate.center(),
        egui::Align2::CENTER_CENTER,
        "1 你好",
        egui::FontId::proportional(11.0),
        color32(palette.text),
    );
}

fn color32(color: RenderColor) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
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
