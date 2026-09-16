//! preedit 片段的画法。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreeditStyle {
    /// 敲的拼音：正常深浅。
    Typed,

    /// 光标后剩下的拼音：淡一点。
    Rest,

    /// 激活的辅码段：淡一点，与拼音区分开（它不是要打的内容，只是这一次筛候选用的键）。
    Fuma,

    /// 被纠错改掉的字母：淡且带删除线。
    Struck,
}
