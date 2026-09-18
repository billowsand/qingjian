use serde::{Deserialize, Serialize};

use super::modifiers::KeyModifiers;

/// DLL 从 TSF `OnKeyDown` / `OnTestKeyDown` 抓到的一次按键，发给 Server 判定。
///
/// 与 [`Frame`](crate::protocol::Frame) 同理整个结构 `#[serde(default)]`：缺字段退到虚拟键 0
/// （Server 当它没按键、放行），不让整条消息解析失败。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyEvent {
    /// Windows 虚拟键码（`VK_*`）。翻页、方向键、退格、回车等靠它区分。
    pub virtual_key: u32,

    /// 这次按键产生的字符（`ToUnicode` 的结果）；功能键没有字符时为 `None`。
    pub character: Option<char>,

    /// 按下时的修饰键状态。
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub fn new(virtual_key: u32, character: Option<char>, modifiers: KeyModifiers) -> Self {
        Self {
            virtual_key,
            character,
            modifiers,
        }
    }
}
