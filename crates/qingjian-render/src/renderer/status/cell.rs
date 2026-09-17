//! 状态条的一格。

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusCell {
    /// 拖动状态条的点阵握柄；单击没有动作。
    Grip,

    /// 一段文字；`emphasized` 用品牌色（当前模式、生效中的全角标点），否则用译文的灰。
    Text { text: String, emphasized: bool },

    /// 打开设置的齿轮。
    Gear,
}

impl StatusCell {
    pub fn text(text: impl Into<String>, emphasized: bool) -> Self {
        Self::Text {
            text: text.into(),
            emphasized,
        }
    }
}
