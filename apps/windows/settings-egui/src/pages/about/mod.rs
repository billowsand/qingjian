//! 「关于」页：品牌与版本、输入统计、许可证、随包数据的来源与署名（第三方许可要求署名在分发物里可见）、
//! 隐私与反馈。

mod usage;

use eframe::egui;

use crate::app::Settings;
use crate::widgets::{LABEL_SIZE, TITLE_SIZE, block, note, page};
use crate::{files, nav};

const WEBSITE_URL: &str = "https://github.com/billowsand/qingjian/releases";

const REPOSITORY_URL: &str = "https://github.com/billowsand/qingjian";

/// 与仓库根 `LICENSE` 一致。
const LICENSE_NOTE: &str = "自由软件，GPL-3.0-or-later 许可证：可以自由使用、修改与再分发，修改后分发须同样开源。官方渠道免费。";

/// 只列随包数据的来源。
const ATTRIBUTIONS: &[(&str, &str)] = &[
    (
        "词库",
        "通用规范汉字表；现代汉语常用词表（liuxilu 校对版）；THUOCL（清华大学自然语言处理实验室，MIT）；读音取自 Unihan（Unicode License v3）。",
    ),
    (
        "语言模型",
        "中文维基百科（CC BY-SA 4.0）与 LCCC（清华大学 CoAI，MIT）语料统计。",
    ),
    ("emoji", "Unicode CLDR annotations（Unicode License v3）。"),
    (
        "英文词表",
        "ESDB / SCOWL（© Kevin Atkinson，按其许可保留版权声明）；CSpell 词典（MIT）。",
    ),
];

const PRIVACY_NOTE: &str = "字在不上传任何数据：拼音、组词、排序与学习都在本机完成。「高级」页的输入日志只写在本机，可以关掉或清空。";

const FEEDBACK_NOTE: &str = "遇到问题点「打包日志到桌面」，把生成的 zip 发给作者即可（含三个进程的日志与配置文件，不含密钥）。缺省日志不含你敲的内容；排查排序问题时作者可能请你在「高级」页临时打开详细日志。";

pub(crate) fn view(settings: &Settings, ui: &mut egui::Ui) {
    page(ui, "关于", "字在本机，表达自在", |ui| {
        identity(ui);
        ui.add_space(10.0);
        links(settings, ui);
        ui.add_space(14.0);
        usage::view(settings, ui);
        credits(ui);
        block(ui, "\u{E72E}", "隐私", |ui| note(ui, PRIVACY_NOTE));
        block(ui, "\u{E8BD}", "反馈", |ui| note(ui, FEEDBACK_NOTE));
    });
}

/// 大号 Logo + 名字 + 一句话 + 版本与构建。
fn identity(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if let Some(texture) = nav::logo(ui.ctx()) {
            ui.add(egui::Image::new(&texture).fit_to_exact_size(egui::vec2(64.0, 64.0)));
        }
        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("字在").size(TITLE_SIZE).strong());
            note(ui, "字在本机，表达自在。");
            ui.label(
                egui::RichText::new("字在 Windows · egui spike")
                    .size(LABEL_SIZE)
                    .strong(),
            );
            // QINGJIAN_BUILD 由 build.rs 从 git 取
            note(
                ui,
                &format!(
                    "构建 {}",
                    option_env!("QINGJIAN_BUILD").unwrap_or("本地构建")
                ),
            );
        });
    });
}

fn links(settings: &Settings, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("下载").clicked() {
            files::open_with_explorer(WEBSITE_URL);
        }
        if ui.button("GitHub").clicked() {
            files::open_with_explorer(REPOSITORY_URL);
        }
        if ui.button("打开数据目录").clicked() {
            files::open_with_explorer(&settings.data_dir().to_string_lossy());
        }
        if ui.button("打包日志到桌面").clicked() {
            files::export_logs();
        }
    });
}

fn credits(ui: &mut egui::Ui) {
    block(ui, "\u{E8A5}", "许可证与数据来源", |ui| {
        note(ui, LICENSE_NOTE);
        ui.add_space(4.0);
        for (name, text) in ATTRIBUTIONS {
            note(ui, &format!("{name}：{text}"));
        }
    });
}
