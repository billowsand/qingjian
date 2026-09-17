//! 「词库」页：随包领域词库一张卡，导入的词库一张卡。一本一行，名称 + 条数在左，开关（用户词库还有「移除」）在右。
//! 随包开关写 `[dictionaries] domains`（列打开的），用户词库写 `disabled`（列关掉的）。

use std::path::{Path, PathBuf};

use eframe::egui;
use qingjian_core::dictionary::Dictionary;
use qingjian_platform::extra_dictionaries;

use crate::app::Settings;
use crate::files;
use crate::widgets::{List, caption, note, page, plain_list, toggle};

/// 一本词库读出来的显示信息。
struct DictInfo {
    /// 文件名主干，配置里用它当键。
    stem: String,

    /// 「名称 · N 条」。
    label: String,

    /// 许可证，挂在悬停提示里。
    license: String,

    /// 打不开的文件：只列出来，开关灰着。
    broken: bool,
}

/// 一行上发生的事。
enum Action {
    None,

    /// 勾选变了，值是新状态。
    Toggled(bool),

    /// 按了「移除」。
    Remove,
}

pub(crate) fn view(settings: &mut Settings, ui: &mut egui::Ui) {
    page(ui, "词库", |ui| {
        caption(ui, "随包领域词库 · 基础词库始终启用，不在这里");
        bundled(settings, ui);
        caption(ui, "导入的词库");
        imported(settings, ui);
    });
}

fn bundled(settings: &mut Settings, ui: &mut egui::Ui) {
    let Some(dir) = files::repo_resource("data/generated/dicts") else {
        note(ui, "没找到随包领域词库目录。");
        return;
    };
    let dicts = read_dir(&dir);
    if dicts.is_empty() {
        note(ui, "随包领域词库目录是空的。");
        return;
    }
    plain_list(ui, |list| {
        for info in dicts {
            let enabled = settings.config.dictionaries.is_domain_enabled(&info.stem);
            if let Action::Toggled(on) = row(list, &info, enabled, false) {
                toggle_domain(settings, &info.stem, on);
            }
        }
    });
}

fn imported(settings: &mut Settings, ui: &mut egui::Ui) {
    let dicts = read_dir(&user_dir(settings));
    plain_list(ui, |list| {
        for info in &dicts {
            let enabled = settings.config.dictionaries.is_enabled(&info.stem);
            match row(list, info, enabled, true) {
                Action::Toggled(on) => toggle_user(settings, &info.stem, on),
                Action::Remove => {
                    remove_user_dict(settings, &info.stem);
                    settings.reload();
                }
                Action::None => {}
            }
        }
        list.custom(|ui| {
            ui.horizontal(|ui| {
                if ui.button("导入词库…").clicked() {
                    import(settings);
                    settings.reload();
                }
                if dicts.is_empty() {
                    note(ui, "接受字在 TSV、Rime .dict.yaml、.qj");
                }
            });
        });
    });
}

/// 一本词库一行：名称 + 条数在左，开关（`removable` 时还有「移除」）在右；许可证与坏文件提示挂悬停。
fn row(list: &mut List, info: &DictInfo, enabled: bool, removable: bool) -> Action {
    let tip = if info.broken {
        "文件打不开，检查格式或重新导入"
    } else {
        &info.license
    };
    let mut action = Action::None;
    let broken = info.broken;
    let mut on = enabled;
    list.row("", &info.label, tip, |ui| {
        let mut response = ui
            .add_enabled_ui(!broken, |ui| toggle(ui, &mut on, &info.label))
            .inner;
        if response.changed() {
            action = Action::Toggled(on);
        }
        if removable {
            let remove = ui.button("移除");
            if remove.clicked() {
                action = Action::Remove;
            }
            response |= remove;
        }
        response
    });
    action
}

/// 目录里的词库，读出显示信息。
fn read_dir(dir: &Path) -> Vec<DictInfo> {
    extra_dictionaries::list(dir)
        .into_iter()
        .map(|(stem, path)| read_info(&path, stem))
        .collect()
}

fn read_info(path: &Path, stem: String) -> DictInfo {
    let Ok(dict) = Dictionary::from_path(path) else {
        return DictInfo {
            label: format!("{stem}（文件损坏）"),
            stem,
            license: String::new(),
            broken: true,
        };
    };
    let metadata = dict.metadata();
    let name = metadata.map_or_else(|| stem.clone(), |m| m.name.clone());
    let license = metadata.map_or_else(String::new, |m| m.license.clone());
    // 随包词库的名称都带「青简领域词库：」这类前缀，列表里只留后半截。
    let short = name.rsplit('：').next().unwrap_or(&name).trim().to_owned();
    DictInfo {
        label: format!("{short} · {} 条", dict.len()),
        stem,
        license,
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
