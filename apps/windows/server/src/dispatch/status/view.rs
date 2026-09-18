use qingjian_platform::ColorScheme;

/// 状态条一次要显示的内容。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusView {
    /// 英文模式（`false` 中文）。
    pub english: bool,

    /// 开着双拼时的配置键（`xiaohe` / `ziranma` / `microsoft` / `sogou`）；
    /// UI 把它收成单字方案标记。
    pub scheme: Option<String>,

    /// 当前模式的全角标点开着（中英各记一份配置）；关着时格子显示 `,.` 画成灰的。
    pub full_width: bool,

    /// 与候选窗口、设置程序共用的色系。
    pub color_scheme: ColorScheme,

    /// 配置里记住的内容左上角物理像素；`None` 首次按屏幕右下角摆。
    pub anchor: Option<(i32, i32)>,
}
