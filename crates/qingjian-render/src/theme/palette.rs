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
    /// 字在蓝浅色（保留旧名字给已有调用）。
    pub const fn light() -> Self {
        Self::zizai_light()
    }

    /// 字在蓝深色（保留旧名字给已有调用）。
    pub const fn dark() -> Self {
        Self::zizai_dark()
    }

    pub const fn cream_light() -> Self {
        Self {
            accent: Color::rgb(197, 99, 66),
            text: Color::rgb(45, 41, 37),
            gloss: Color::rgb(117, 106, 97),
            pos: Color::rgb(160, 145, 133),
            fresh: Color::rgb(201, 123, 53),
            index: Color::rgb(160, 145, 133),
            cloud: Color::rgb(65, 151, 183),
            caret: Color::rgb(79, 184, 166),
            correction: Color::rgb(199, 91, 83),
            background: Color::rgb(247, 241, 232),
            highlight: Color::rgb(242, 222, 209),
        }
    }

    pub const fn cream_dark() -> Self {
        Self {
            accent: Color::rgb(229, 138, 102),
            text: Color::rgb(247, 238, 227),
            gloss: Color::rgb(200, 185, 170),
            pos: Color::rgb(145, 131, 118),
            fresh: Color::rgb(231, 169, 83),
            index: Color::rgb(145, 131, 118),
            cloud: Color::rgb(93, 190, 220),
            caret: Color::rgb(103, 205, 187),
            correction: Color::rgb(235, 128, 119),
            background: Color::rgb(38, 31, 27),
            highlight: Color::rgb(75, 50, 40),
        }
    }

    pub const fn zizai_light() -> Self {
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

    pub const fn zizai_dark() -> Self {
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

    pub const fn latte_light() -> Self {
        Self {
            accent: Color::rgb(123, 90, 166),
            text: Color::rgb(48, 39, 56),
            gloss: Color::rgb(120, 107, 129),
            pos: Color::rgb(164, 151, 173),
            fresh: Color::rgb(201, 123, 53),
            index: Color::rgb(164, 151, 173),
            cloud: Color::rgb(94, 158, 199),
            caret: Color::rgb(196, 154, 108),
            correction: Color::rgb(202, 101, 113),
            background: Color::rgb(247, 243, 249),
            highlight: Color::rgb(233, 223, 240),
        }
    }

    pub const fn latte_dark() -> Self {
        Self {
            accent: Color::rgb(181, 150, 211),
            text: Color::rgb(244, 236, 246),
            gloss: Color::rgb(198, 183, 204),
            pos: Color::rgb(143, 129, 151),
            fresh: Color::rgb(230, 166, 86),
            index: Color::rgb(143, 129, 151),
            cloud: Color::rgb(104, 184, 220),
            caret: Color::rgb(210, 174, 131),
            correction: Color::rgb(235, 133, 145),
            background: Color::rgb(33, 26, 38),
            highlight: Color::rgb(59, 45, 71),
        }
    }

    pub const fn forest_light() -> Self {
        Self {
            accent: Color::rgb(62, 124, 85),
            text: Color::rgb(36, 49, 40),
            gloss: Color::rgb(100, 114, 104),
            pos: Color::rgb(146, 158, 149),
            fresh: Color::rgb(193, 131, 50),
            index: Color::rgb(146, 158, 149),
            cloud: Color::rgb(57, 143, 157),
            caret: Color::rgb(149, 168, 79),
            correction: Color::rgb(199, 94, 91),
            background: Color::rgb(240, 244, 237),
            highlight: Color::rgb(221, 234, 223),
        }
    }

    pub const fn forest_dark() -> Self {
        Self {
            accent: Color::rgb(114, 179, 135),
            text: Color::rgb(237, 244, 234),
            gloss: Color::rgb(182, 195, 184),
            pos: Color::rgb(124, 140, 128),
            fresh: Color::rgb(224, 171, 79),
            index: Color::rgb(124, 140, 128),
            cloud: Color::rgb(88, 183, 196),
            caret: Color::rgb(176, 196, 94),
            correction: Color::rgb(234, 132, 128),
            background: Color::rgb(23, 35, 27),
            highlight: Color::rgb(41, 66, 51),
        }
    }
}
