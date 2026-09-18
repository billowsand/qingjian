use serde::{Deserialize, Serialize};

/// 旧协议里的候选窗口明暗。
///
/// 新设置不再使用这个类型，明暗始终跟随系统；字段继续留在 [`crate::protocol::Frame`] 里，
/// 让升级后仍驻留在应用进程里的旧 DLL 能解析新 Server 发来的帧。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// 跟随系统。
    #[default]
    System,

    /// 始终浅色。
    Light,

    /// 始终深色。
    Dark,
}

impl ThemeMode {
    /// 旧版设置界面的排列顺序。
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    /// 配置文件里的写法。
    pub fn key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// 界面上的名字。
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "跟随系统",
            Self::Light => "浅色",
            Self::Dark => "深色",
        }
    }
}
