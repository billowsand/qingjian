//! 「字在」浅色 / 深色配色；颜色角色与数值见 `docs/design/brand.md`。

use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// 品牌强调色：当前模式、主要选中态。
    pub accent: Color,

    /// 候选词。
    pub text: Color,

    /// 译文。
    pub gloss: Color,

    /// 词性，比译文更浅。
    pub pos: Color,

    /// 生词译文：比普通译文醒目，看熟了就回到译文色。
    pub fresh: Color,

    /// 序号。
    pub index: Color,

    /// 云联想的云朵与文字：比译文醒目一点，但不抢候选词。
    pub cloud: Color,

    /// preedit 输入光标：只表示正在输入 / 本地生效。
    pub caret: Color,

    /// 拼写纠正里被替换的原输入。
    pub correction: Color,

    /// 窗口背景。
    pub background: Color,

    /// 当前候选的高亮底色。
    pub highlight: Color,
}

impl Palette {
    pub const fn light() -> Self {
        Self {
            accent: Color::rgb(49, 87, 216),
            text: Color::rgb(23, 32, 51),
            gloss: Color::rgb(102, 112, 133),
            pos: Color::rgb(152, 162, 179),
            fresh: Color::rgb(245, 158, 11),
            index: Color::rgb(152, 162, 179),
            cloud: Color::rgb(50, 181, 233),
            caret: Color::rgb(85, 214, 194),
            correction: Color::rgb(228, 109, 109),
            background: Color::rgb(247, 246, 242),
            highlight: Color::rgb(232, 238, 255),
        }
    }

    pub const fn dark() -> Self {
        Self {
            accent: Color::rgb(107, 139, 255),
            text: Color::rgb(247, 246, 242),
            gloss: Color::rgb(184, 192, 208),
            pos: Color::rgb(127, 138, 158),
            fresh: Color::rgb(251, 191, 36),
            index: Color::rgb(127, 138, 158),
            cloud: Color::rgb(56, 189, 248),
            caret: Color::rgb(85, 214, 194),
            correction: Color::rgb(243, 139, 139),
            background: Color::rgb(23, 32, 51),
            highlight: Color::rgb(37, 55, 95),
        }
    }
}
