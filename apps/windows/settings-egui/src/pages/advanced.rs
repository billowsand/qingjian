//! 「高级」页：打开配置文件 / 数据目录 / 日志目录、详细日志、学习开关、输入日志。

use eframe::egui;
use qingjian_platform::LogLevel;

use crate::app::Settings;
use crate::files;
use crate::widgets::{field, group, note, page, toggle};

pub(crate) fn view(settings: &mut Settings, ui: &mut egui::Ui) {
    page(ui, "高级", "本地数据、日志与诊断工具", |ui| {
        files_group(settings, ui);
        diagnostics_group(settings, ui);
        note(ui, "改坏了删掉 config.toml 即可回到默认设置。");
    });
}

fn files_group(settings: &Settings, ui: &mut egui::Ui) {
    group(ui, "\u{E8B7}", "文件与目录", |ui| {
        field(
            ui,
            "\u{E8A5}",
            "配置文件",
            "设置改完会自动生效（Server 每秒看一次配置文件）。只有换学习语言要重启 Server。",
            |ui| {
                let response = ui.button("在记事本中打开");
                if response.clicked() {
                    files::open_in_editor(settings.config_file());
                }
                response
            },
        );
        field(
            ui,
            "\u{E838}",
            "数据目录",
            "词频、用户词、学习数据与统计都在这里。",
            |ui| {
                let response = ui.button("打开数据目录");
                if response.clicked() {
                    files::open_with_explorer(&settings.data_dir().to_string_lossy());
                }
                response
            },
        );
        field(
            ui,
            "\u{E7C3}",
            "日志目录",
            "输入法、引擎与设置程序的日志都在这一个目录，按天分文件，保留 7 天。",
            |ui| {
                let export = ui.button("打包日志到桌面");
                if export.clicked() {
                    files::export_logs();
                }
                let open = ui.button("打开日志目录");
                if open.clicked()
                    && let Some(logs) = files::log_dir()
                {
                    files::open_with_explorer(&logs.to_string_lossy());
                }
                open | export
            },
        );
    });
}

fn diagnostics_group(settings: &mut Settings, ui: &mut egui::Ui) {
    group(ui, "\u{E90F}", "学习与诊断", |ui| {
        let mut learning = settings.config.general.learning;
        field(
            ui,
            "\u{E734}",
            "学习输入习惯",
            "按你的选择调整候选顺序、记新词与敲错纠正。关掉后不再学，已学的仍参与排序；学习数据在数据目录里。",
            |ui| {
                let response = toggle(ui, &mut learning, "学习输入习惯");
                if response.changed() {
                    settings.save("general", "learning", learning);
                }
                response
            },
        );
        let mut verbose = settings.config.general.log_level == LogLevel::Debug;
        field(
            ui,
            "\u{E8FD}",
            "详细日志",
            "排查问题时临时打开，会记下敲的拼音与上屏文字。",
            |ui| {
                let response = toggle(ui, &mut verbose, "详细日志");
                if response.changed() {
                    let level = if verbose {
                        LogLevel::Debug
                    } else {
                        LogLevel::Info
                    };
                    settings.save("general", "log_level", level.key());
                }
                response
            },
        );
        let mut input_log = settings.config.general.input_log;
        field(
            ui,
            "\u{E70F}",
            "记录输入日志",
            "每次上屏记一行，只写本机、不上传，用于离线评测与个人模型。",
            |ui| {
                let response = toggle(ui, &mut input_log, "记录输入日志");
                if response.changed() {
                    settings.save("general", "input_log", input_log);
                }
                response
            },
        );
        field(ui, "\u{E894}", "清空输入日志", "", |ui| {
            let response = ui.button("清空输入日志");
            if response.clicked() {
                let log = settings.data_dir().join("input-log.jsonl");
                if let Err(error) = std::fs::remove_file(&log)
                    && error.kind() != std::io::ErrorKind::NotFound
                {
                    eprintln!("清空输入日志失败: {error}");
                }
            }
            response
        });
    });
}
