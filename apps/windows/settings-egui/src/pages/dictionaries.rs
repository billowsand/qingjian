//! 「词库」页：随包领域词库开关，用户导入词库的开关 / 移除 / 导入。
//! 随包开关写 `[dictionaries] domains`（列打开的），用户词库写 `disabled`（列关掉的）。

use std::path::{Path, PathBuf};

use eframe::egui;
use qingjian_core::dictionary::Dictionary;
use qingjian_platform::extra_dictionaries;

use crate::app::Settings;
use crate::files;
use crate::widgets::{LABEL_SIZE, block, hint, note, page};

/// 一本词库读出来的显示信息。
struct DictInfo {
    /// 文件名主干，配置里用它当键。
    stem: String,

    /// 「名称 · N 条 · 随包 / 许可证」，坏文件标出来。
    label: String,

    /// 打不开的文件：只列出来，不给勾。
    broken: bool,
}

pub(crate) fn view(settings: &mut Settings, ui: &mut egui::Ui) {
    page(ui, "词库", "只加载你真正会用到的词", |ui| {
        bundled_block(settings, ui);
        imported_block(settings, ui);
    });
}

fn bundled_block(settings: &mut Settings, ui: &mut egui::Ui) {
    block(ui, "\u{E8F1}", "随包领域词库", |ui| {
        let Some(dir) = files::repo_resource("data/generated/dicts") else {
            note(ui, "没找到随包领域词库目录（安装布局待定）。");
            return;
        };
        let dicts = list(&dir, true);
        if dicts.is_empty() {
            note(ui, "随包领域词库目录是空的。");
            return;
        }
        for info in dicts {
            let mut enabled = settings.config.dictionaries.is_domain_enabled(&info.stem);
            if row(ui, &info, &mut enabled, false).changed() {
                toggle_domain(settings, &info.stem, enabled);
            }
        }
        ui.add_space(4.0);
        note(
            ui,
            "随包的基础词库始终启用，不在这里；这里只管领域词库的开关。改完自动生效。",
        );
    });
}

fn imported_block(settings: &mut Settings, ui: &mut egui::Ui) {
    block(ui, "\u{E8B5}", "导入的词库", |ui| {
        let dicts = list(&user_dir(settings), false);
        if dicts.is_empty() {
            note(
                ui,
                "还没有导入词库。点下面「导入词库」加一本，或把文件放进 %APPDATA%\\Qingjian\\dicts。",
            );
        }
        for info in dicts {
            let mut enabled = settings.config.dictionaries.is_enabled(&info.stem);
            let response = row(ui, &info, &mut enabled, true);
            if response.changed() {
                toggle_user(settings, &info.stem, enabled);
            }
            if response.clicked() {
                remove_user_dict(settings, &info.stem);
                settings.reload();
            }
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button("导入词库…").clicked() {
                import(settings);
                settings.reload();
            }
            note(
                ui,
                "接受字在 TSV、Rime .dict.yaml、.qj；导入即复制进用户词库目录。",
            );
        });
    });
}

/// 一本词库一行：复选框 +（用户词库才有的）「移除」。
///
/// 返回的 `Response`：`changed()` 是勾选变了，`clicked()` 是按了「移除」——一行只可能发生一件。
fn row(ui: &mut egui::Ui, info: &DictInfo, enabled: &mut bool, removable: bool) -> egui::Response {
    ui.horizontal(|ui| {
        let check = ui.add_enabled(
            !info.broken,
            egui::Checkbox::new(enabled, egui::RichText::new(&info.label).size(LABEL_SIZE)),
        );
        let check = hint(
            check,
            if info.broken {
                "文件打不开，检查格式或重新导入"
            } else {
                ""
            },
        );
        if !removable {
            return check;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let remove = ui.button("移除");
            if remove.clicked() { remove } else { check }
        })
        .inner
    })
    .inner
}

/// 目录里的词库，读出显示信息。`builtin` 是随包的那批（标「随包」而不是列许可证）。
fn list(dir: &Path, builtin: bool) -> Vec<DictInfo> {
    extra_dictionaries::list(dir)
        .into_iter()
        .map(|(stem, path)| read_info(&path, stem, builtin))
        .collect()
}

fn read_info(path: &Path, stem: String, builtin: bool) -> DictInfo {
    let Ok(dict) = Dictionary::from_path(path) else {
        return DictInfo {
            label: format!("{stem}（文件损坏）"),
            stem,
            broken: true,
        };
    };
    let metadata = dict.metadata();
    let name = metadata.map_or_else(|| stem.clone(), |m| m.name.clone());
    let license = metadata.map_or_else(String::new, |m| m.license.clone());
    let mut label = format!("{name} · {} 条", dict.len());
    if builtin {
        label.push_str(" · 随包");
    } else if !license.is_empty() {
        label.push_str(&format!(" · {license}"));
    }
    DictInfo {
        stem,
        label,
        broken: false,
    }
}

fn toggle_domain(settings: &mut Settings, stem: &str, on: bool) {
    let mut domains = settings.config.dictionaries.domains.clone();
    if on {
        if !domains.iter().any(|d| d == stem) {
            domains.push(stem.to_owned());
        }
    } else {
        domains.retain(|d| d != stem);
    }
    settings.save_array("dictionaries", "domains", &domains);
}

/// 用户词库缺省启用，`disabled` 列的是关掉的。
fn toggle_user(settings: &mut Settings, stem: &str, on: bool) {
    let mut disabled = settings.config.dictionaries.disabled.clone();
    if on {
        disabled.retain(|d| d != stem);
    } else if !disabled.iter().any(|d| d == stem) {
        disabled.push(stem.to_owned());
    }
    settings.save_array("dictionaries", "disabled", &disabled);
}

/// 用户词库目录 `%APPDATA%\Qingjian\dicts`。
fn user_dir(settings: &Settings) -> PathBuf {
    settings.data_dir().join("dicts")
}

/// 挪进 `dicts\removed`，不真删。
fn remove_user_dict(settings: &Settings, stem: &str) {
    let dir = user_dir(settings);
    let Some((_, path)) = extra_dictionaries::list(&dir)
        .into_iter()
        .find(|(name, _)| name == stem)
    else {
        return;
    };
    let removed = dir.join("removed");
    if let Err(error) = std::fs::create_dir_all(&removed) {
        eprintln!("建 removed 目录失败: {error}");
        return;
    }
    if let Some(file_name) = path.file_name()
        && let Err(error) = std::fs::rename(&path, removed.join(file_name))
    {
        eprintln!("移除词库 {stem} 失败: {error}");
    }
}

/// 文件选择器选一本，复制进用户词库目录。
fn import(settings: &Settings) {
    let Some(source) = rfd::FileDialog::new()
        .add_filter("词库文件", &["tsv", "yaml", "yml", "qj"])
        .add_filter("所有文件", &["*"])
        .set_title("导入词库")
        .pick_file()
    else {
        return;
    };
    let dir = user_dir(settings);
    if let Err(error) = std::fs::create_dir_all(&dir) {
        eprintln!("建用户词库目录失败: {error}");
        return;
    }
    let Some(file_name) = source.file_name() else {
        return;
    };
    if let Err(error) = std::fs::copy(&source, dir.join(file_name)) {
        eprintln!("导入词库失败: {error}");
    }
}
