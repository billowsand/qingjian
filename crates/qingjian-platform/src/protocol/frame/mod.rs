//! 一次要绘制的组句状态：preedit 行加候选页。

pub mod preedit;

pub use preedit::{PreeditKind, PreeditSegment};

use serde::{Deserialize, Serialize};

use qingjian_core::CandidateList;

use crate::{LayoutMode, ThemeMode};

/// 老 Server 发来的帧没有 `inline_preedit` 字段，按「放行内」算（那时的行为）。
fn inline_preedit_default() -> bool {
    true
}

/// Server 告诉 DLL「现在屏幕上该是什么样」：组句的拼音行、候选页、高亮与页码。
/// 空 [`Frame`]（`preedit` 与 `candidates` 都空）表示没有在组句，DLL 收起候选窗口。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// 组句拼音行的分段，按顺序拼成整行。
    pub preedit: Vec<PreeditSegment>,

    /// 光标在拼音行里的位置，按 `preedit` 拼接后的字符（`char`）数算。
    pub cursor: usize,

    /// 当前页的候选（已排好序、不带译文由后续 [`super::ServerMessage::Update`] 补）。
    pub candidates: CandidateList,

    /// 当前页里高亮的候选下标（页内，从 0 起）。
    pub highlight: usize,

    /// 当前页码（从 0 起）。
    pub page: usize,

    /// 总页数；翻页键是否可用看它。
    pub page_count: usize,

    /// 候选排布（竖排 / 横排）。DLL 是纯渲染端，布局由 Server 按 `[general] layout` 配置随帧下发。
    pub layout: LayoutMode,

    /// 候选窗口外观（跟随系统 / 浅色 / 深色）。`System` 由 DLL 侧按当前系统主题解析。
    pub theme: ThemeMode,

    /// 整句补全（云联想给的整段拼音的整句结果）：画在 preedit 行右侧，按 Tab 上屏。无则 `None`。
    pub sentence: Option<String>,

    /// 屏幕提示（删候选后的「已删除…」一句）：画在 preedit 行下方，显示到下一次按键。无则 `None`。
    /// 不参与 [`is_empty`](Self::is_empty)：单有提示不算在组句，否则空组句也会撑开候选窗口。
    #[serde(default)]
    pub notice: Option<String>,

    /// 拼音行要不要放进应用里（TSF 组句 / marked text）。DLL 是纯渲染端，
    /// 由 Server 按 `[general] preedit` 随帧下发；为 `false` 时 `preedit` 只画在候选窗口里。
    #[serde(default = "inline_preedit_default")]
    pub inline_preedit: bool,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            preedit: Vec::new(),
            cursor: 0,
            candidates: CandidateList::default(),
            highlight: 0,
            page: 0,
            page_count: 0,
            layout: LayoutMode::default(),
            theme: ThemeMode::default(),
            sentence: None,
            notice: None,
            inline_preedit: inline_preedit_default(),
        }
    }
}

impl Frame {
    /// 没有在组句：DLL 据此收起候选窗口。提示不算数（见 [`notice`](Self::notice)）。
    pub fn is_empty(&self) -> bool {
        self.preedit.is_empty() && self.candidates.items.is_empty()
    }
}
