//! 「关于」页：品牌与版本、输入统计、四个入口、许可证。
//! 第三方许可要求署名在分发物里可见——放在可折叠的「数据来源」里，点开即见。

mod usage;

use eframe::egui;

use crate::app::Settings;
use crate::widgets::{BRAND_SIZE, LABEL_SIZE, NOTE_SIZE, list, note, page};
use crate::{files, nav, theme};

const WEBSITE_URL: &str = "https://github.com/billowsand/qingjian/releases";

const REPOSITORY_URL: &str = "https://github.com/billowsand/qingjian";

/// 随包数据的来源与署名，折叠着。
const ATTRIBUTIONS: &[(&str, &str)] = &[
    (
        "词库",
        "通用规范汉字表；现代汉语常用词表（liuxilu 校对版）；THUOCL（清华 NLP，MIT）；读音取自 Unihan（Unicode License v3）",
    ),
    (
        "语言模型",
        "中文维基百科（CC BY-SA 4.0）与 LCCC（清华 CoAI，MIT）",
    ),
    ("emoji", "Unicode CLDR annotations（Unicode License v3）"),
    (
        "英文词表",
        "ESDB / SCOWL（© Kevin Atkinson）；CSpell 词典（MIT）",
    ),
];

pub(crate) fn view(settings: &Settings, ui: &mut egui::Ui) {
    page(ui, "关于", |ui| {
        identity(ui);
        ui.add_space(14.0);
        stats(settings, ui);
        actions(settings, ui);
        ui.add_space(6.0);
        note(ui, "GPL-3.0-or-later 自由软件 · 数据只留在本机，不上传");
        ui.add_space(6.0);
        attributions(ui);
    });
}

/// Logo + 名字 + 一句话 + 版本。
fn identity(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if let Some(texture) = nav::logo(ui.ctx()) {
            ui.add(egui::Image::new(&texture).fit_to_exact_size(egui::vec2(48.0, 48.0)));
        }
        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("字在").size(BRAND_SIZE).strong());
            note(ui, "字在本机，表达自在");
            // QINGJIAN_BUILD 由 build.rs 从 git 取
            // 只显示「短哈希 (日期)」那半截，分支名对用户没有意义。
            let build = option_env!("QINGJIAN_BUILD")
                .and_then(|build| build.split_once('@').map(|(_, rest)| rest))
                .unwrap_or("本地构建");
            ui.label(
                egui::RichText::new(format!("egui spike · {build}"))
                    .size(NOTE_SIZE)
                    .color(theme::note_color(ui.ctx()).gamma_multiply(0.8)),
            );
        });
    });
}

/// 三行输入量，右侧是汉字数，悬停给中英文词与上屏次数。
fn stats(settings: &Settings, ui: &mut egui::Ui) {
    let summary = usage::summary(settings);
    list(ui, |list| {
        for (icon, label, usage) in [
            ("\u{E787}", "今天", &summary.today),
            ("\u{E81C}", "最近 7 天", &summary.week),
            ("\u{E8EF}", "累计", &summary.total),
        ] {
            list.row(icon, label, &usage::detail(usage), |ui| {
                ui.label(egui::RichText::new(usage::hanzi(usage)).size(LABEL_SIZE))
            });
        }
    });
    note(ui, &usage::scale_line(&summary));
    ui.add_space(10.0);
}

fn actions(settings: &Settings, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("下载").clicked() {
            files::open_with_explorer(WEBSITE_URL);
        }
        if ui.button("GitHub").clicked() {
            files::open_with_explorer(REPOSITORY_URL);
        }
        if ui.button("数据目录").clicked() {
            files::open_with_explorer(&settings.data_dir().to_string_lossy());
        }
        let export = ui.button("打包日志");
        if export.clicked() {
            files::export_logs();
        }
        export.on_hover_text("日志与配置打成 zip 放到桌面，发给作者即可；不含你敲的内容。");
    });
}

fn attributions(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new(
        egui::RichText::new("数据来源与署名")
            .size(NOTE_SIZE)
            .color(theme::note_color(ui.ctx())),
    )
    .id_salt("attributions")
    .show(ui, |ui| {
        for (name, text) in ATTRIBUTIONS {
            note(ui, &format!("{name}：{text}"));
        }
    });
}
