use qingjian_core::ShuangpinScheme;
use qingjian_platform::protocol::KeyModifiers;
use qingjian_platform::{Config, PreeditMode, ThemeMode};

/// Router 要用的配置项，与 macOS 壳的 `Host` 字段对齐。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouterConfig {
    /// 每页候选数（`[general] page_size`）。
    pub page_size: usize,

    /// 候选窗口外观（`[general] theme`）。
    pub theme: ThemeMode,

    /// 组句拼音显示在行内还是候选窗口（`[general] preedit`）。
    pub preedit: PreeditMode,

    /// 候选窗口字体的字族名（`[general] font`），空为系统字体。
    pub font: String,

    /// 翻页键对（`[general] page_keys`，上一页 / 下一页）。
    pub page_keys: (char, char),

    /// 中文模式下不在组句时的标点转全角（`[general] full_width_punctuation`）；状态条可切。
    pub full_width: bool,

    /// 英文模式的那一份（`[general] english_full_width_punctuation`）。
    pub english_full_width: bool,

    /// 删候选的修饰键（`[shortcut] delete_candidate`）。
    pub delete_keys: KeyModifiers,

    /// 悬浮状态条开关（`[status_bar] enabled`）。
    pub status_enabled: bool,

    /// 状态条记住的位置（`[status_bar] x` / `y`，内容左上角物理像素）。
    pub status_pos: Option<(i32, i32)>,

    /// 双拼方案（`[general] shuangpin`）；全拼为 `None`。
    pub shuangpin: Option<ShuangpinScheme>,
}

impl From<&Config> for RouterConfig {
    fn from(config: &Config) -> Self {
        Self {
            page_size: config.general.page_size(),
            theme: config.general.theme,
            preedit: config.general.preedit,
            font: config.general.font.trim().to_owned(),
            page_keys: config.general.page_keys(),
            full_width: config.general.full_width_punctuation,
            english_full_width: config.general.english_full_width_punctuation,
            delete_keys: config.shortcut.delete_keys().into(),
            status_enabled: config.status_bar.enabled,
            status_pos: config.status_bar.x.zip(config.status_bar.y),
            shuangpin: config.general.shuangpin(),
        }
    }
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self::from(&Config::default())
    }
}
