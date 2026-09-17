//! spike 的窗口状态：当前配置、`config.toml` 路径、当前分节、系统字族表，以及启动计时。
//! 落盘逻辑与正式设置程序一样——改一项就原地写回（保留注释）再重读，界面始终反映文件内容。
//! 每帧怎么画在 [`update`]。

mod update;

use std::path::{Path, PathBuf};
use std::time::Instant;

use qingjian_platform::Config;

use crate::{fonts, theme};

/// 左侧导航的分节：tag + 界面名 + 图标码点。
pub(crate) const PAGES: [(&str, &str, &str); 5] = [
    ("general", "通用", "\u{E713}"),
    ("appearance", "候选窗口", "\u{E890}"),
    ("dictionaries", "词库", "\u{E8F1}"),
    ("advanced", "高级", "\u{E90F}"),
    ("about", "关于", "\u{E897}"),
];

pub(crate) struct Settings {
    /// 当前配置，每次改动后从盘上重读。
    pub(crate) config: Config,

    /// `config.toml` 路径。
    path: PathBuf,

    /// 当前导航分节 tag。
    page: String,

    /// 系统里的字族名（DirectWrite 列举），「字体」下拉用；开窗时列一次。
    pub(crate) families: Vec<String>,

    /// 上次解析出的系统明暗，变了换一套 Visuals。
    pub(crate) dark: bool,

    /// 进程启动的时刻，首帧画完时报一次耗时。
    started: Instant,

    /// 首帧是否已经报过耗时。
    reported: bool,
}

impl Settings {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>, started: Instant) -> Self {
        fonts::install(&cc.egui_ctx);
        theme::install(&cc.egui_ctx);
        let path = Self::config_path();
        let config = Config::load(&path).unwrap_or_default();
        Self {
            config,
            path,
            page: "general".to_owned(),
            families: qingjian_render::system_fonts::families(),
            dark: theme::system_prefers_dark(),
            started,
            reported: false,
        }
    }

    /// `%APPDATA%\Qingjian\config.toml`；取不到 `APPDATA` 退回工作目录。
    fn config_path() -> PathBuf {
        qingjian_platform::dirs::config_path().unwrap_or_else(|| PathBuf::from("config.toml"))
    }

    /// 配置文件本身，「高级」页要打开它。
    pub(crate) fn config_file(&self) -> &Path {
        &self.path
    }

    /// 数据目录 `%APPDATA%\Qingjian`。
    pub(crate) fn data_dir(&self) -> &Path {
        self.path.parent().unwrap_or_else(|| Path::new("."))
    }

    /// 落盘一个配置值再重读。spike 里失败只打印。
    pub(crate) fn save(&mut self, section: &str, key: &str, value: impl Into<toml_edit::Value>) {
        if let Err(error) = Config::set_value(&self.path, section, key, value) {
            eprintln!("保存 [{section}] {key} 失败: {error}");
            return;
        }
        self.reload();
    }

    /// 落盘一个字符串数组再重读。
    pub(crate) fn save_array(&mut self, section: &str, key: &str, values: &[String]) {
        if let Err(error) = Config::set_array(&self.path, section, key, values) {
            eprintln!("保存 [{section}] {key} 失败: {error}");
            return;
        }
        self.reload();
    }

    pub(crate) fn reload(&mut self) {
        if let Ok(config) = Config::load(&self.path) {
            self.config = config;
        }
    }
}
