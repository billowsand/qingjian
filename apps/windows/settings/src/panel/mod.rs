//! 设置窗口根组件：左侧导航 + 右侧当前分节页。每改一项就原地写回 `config.toml`（保留注释）再重读，
//! 界面始终反映文件内容；Server 每秒看 mtime 热加载。
//! 状态在这里，消息在 [`message`]，生命周期在 [`component`]，表单零件在 [`controls`]，
//! Logo 与版本在 [`brand`]，各页在 [`pages`]。

mod brand;
mod component;
mod controls;
mod message;
mod pages;

use std::path::{Path, PathBuf};

use qingjian_platform::Config;
use windows_reactor::*;

pub(crate) use self::message::Message;
use self::pages::{about, advanced, appearance, dictionaries, general};

/// 设置窗口状态。
pub(crate) struct Settings {
    /// 当前配置，每次改动后从盘上重读。
    pub(super) config: Config,

    /// `config.toml` 路径。
    path: PathBuf,

    /// 当前导航分节 tag。
    page: String,

    /// 系统里的字族名（DirectWrite 列举），「字体」下拉列表用。
    families: Vec<String>,
}

impl Settings {
    /// `%APPDATA%\Qingjian\config.toml`；取不到 `APPDATA` 退回工作目录。
    fn config_path() -> PathBuf {
        qingjian_platform::dirs::config_path().unwrap_or_else(|| PathBuf::from("config.toml"))
    }

    /// 数据目录 `%APPDATA%\Qingjian`。
    fn data_dir(&self) -> &Path {
        self.path.parent().unwrap_or_else(|| Path::new("."))
    }

    /// 落盘一个配置值再重读。失败只打印。
    fn save(&mut self, section: &str, key: &str, value: impl Into<toml_edit::Value>) {
        if let Err(error) = Config::set_value(&self.path, section, key, value) {
            crate::log::warn(format!("保存 [{section}] {key} 失败: {error}"));
            return;
        }
        self.reload();
    }

    /// 落盘一个字符串数组再重读。
    fn save_array(&mut self, section: &str, key: &str, values: &[String]) {
        if let Err(error) = Config::set_array(&self.path, section, key, values) {
            crate::log::warn(format!("保存 [{section}] {key} 失败: {error}"));
            return;
        }
        self.reload();
    }

    fn reload(&mut self) {
        if let Ok(config) = Config::load(&self.path) {
            self.config = config;
        }
    }

    fn page_content(&self, context: &mut ViewContext<Self>) -> View {
        match self.page.as_str() {
            "appearance" => appearance::view(self, context),
            "dictionaries" => dictionaries::view(self, context),
            "advanced" => advanced::view(self, context),
            "about" => about::view(self, context),
            _ => general::view(self, context),
        }
    }
}
