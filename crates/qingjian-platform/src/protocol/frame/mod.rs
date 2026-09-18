//! 一次要绘制的组句状态：preedit 行加候选页。

pub mod preedit;

pub use preedit::{PreeditKind, PreeditSegment};

use serde::{Deserialize, Serialize};

use qingjian_core::CandidateList;

use crate::ThemeMode;

/// 没有 `inline_preedit` 字段的帧按「放行内」算（那时的行为）。
fn inline_preedit_default() -> bool {
    true
}

/// Server 告诉 DLL「现在屏幕上该是什么样」：组句的拼音行、候选页、高亮与页码。
/// 空 [`Frame`]（`preedit` 与 `candidates` 都空）表示没有在组句，DLL 收起候选窗口。
///
/// 整个结构 `#[serde(default)]`：升级安装后旧 DLL 还留在没重启的应用里，删掉或改名一个字段
/// 会让它整帧解析失败、每键都失败（0.1.6 前删 `layout` 就是这么炸的）。缺字段一律退到
/// [`Frame::default()`]，旧 DLL 至少还能画出候选。见 [`super::PROTOCOL_VERSION`]。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
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

    /// 坟墓字段：0.1.6 之前的 DLL 要求帧里有 `layout`，缺了整帧解析失败——而它们在升级后还留在
    /// 没重启的应用里（DLL 换不掉），这正是「升级要重启系统」那个毛病。竖排已经去掉了，
    /// 这里恒为横排，只写不读（`skip_deserializing`）。等那批 DLL 淘汰干净（再发一两个版本）就删掉它，
    /// 删的时候记得把 [`PROTOCOL_VERSION`](crate::protocol::PROTOCOL_VERSION) +1。
    /// 构造 [`Frame`] 时不用管它，写 `..Frame::default()` 即可。
    #[serde(rename = "layout", skip_deserializing)]
    pub legacy_layout: &'static str,

    /// 候选窗口外观（跟随系统 / 浅色 / 深色）。`System` 由 DLL 侧按当前系统主题解析。
    pub theme: ThemeMode,

    /// 屏幕提示（删候选后的「已删除…」一句）：画在 preedit 行下方，显示到下一次按键。无则 `None`。
    /// 不参与 [`is_empty`](Self::is_empty)：单有提示不算在组句，否则空组句也会撑开候选窗口。
    pub notice: Option<String>,

    /// 拼音行要不要放进应用里（TSF 组句 / marked text）。DLL 是纯渲染端，
    /// 由 Server 按 `[general] preedit` 随帧下发；为 `false` 时 `preedit` 只画在候选窗口里。
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
            legacy_layout: "horizontal",
            theme: ThemeMode::default(),
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
