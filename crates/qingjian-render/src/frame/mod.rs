//! 一帧要画的全部内容：顶部拼音行、候选行、高亮、页脚、右侧整句补全。只是展示形态，不含排序或查词。

mod preedit;
mod row;
mod tone;

pub use preedit::{Preedit, PreeditSegment, PreeditStyle};
pub use row::Row;
pub use tone::Tone;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frame {
    /// 顶部拼音行；配置成只在行内显示时为 `None`。
    pub preedit: Option<Preedit>,

    /// 候选行。
    pub rows: Vec<Row>,

    /// 高亮行；`None` 不高亮。
    pub highlighted: Option<usize>,

    /// 右下角页码。
    pub footer: Option<String>,

    /// 拼音行右侧的整句补全（云联想），组句时才有。
    pub sentence: Option<String>,

    /// 拼音行右侧的一句临时状态（删了什么词），有它时不画整句补全。
    pub status: Option<String>,

    /// 拼音行末尾的辅码「下一键」幽灵提示（` s`，含前导空格），只敲了首码时才有；画淡。
    /// 放拼音行而不放候选旁，是因为候选框高度会随候选行的标注行出现 / 消失而变。
    pub fuma_hint: Option<String>,
}

impl Frame {
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.preedit.is_none() && self.trailing().is_none()
    }

    /// 顶部要不要画一行（拼音或右侧文字任一存在）。
    pub fn has_top_line(&self) -> bool {
        self.preedit.is_some() || self.trailing().is_some()
    }

    /// 拼音行右侧画什么：状态优先，其次整句补全；`bool` 是要不要带云朵。
    pub fn trailing(&self) -> Option<(&str, bool)> {
        self.status
            .as_deref()
            .map(|s| (s, false))
            .or_else(|| self.sentence.as_deref().map(|s| (s, true)))
    }
}
