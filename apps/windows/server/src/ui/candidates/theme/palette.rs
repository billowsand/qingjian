//! 浅色 / 深色配色。

use windows::Win32::Foundation::COLORREF;

use super::rgb;

/// 浅色 / 深色各一套。
pub(super) struct Palette {
    pub(super) accent_color: COLORREF,
    pub(super) text_color: COLORREF,
    pub(super) gloss_color: COLORREF,
    pub(super) pos_color: COLORREF,
    pub(super) fresh_color: COLORREF,
    pub(super) index_color: COLORREF,
    pub(super) caret_color: COLORREF,
    pub(super) correction_color: COLORREF,
    pub(super) background: COLORREF,
    pub(super) highlight: COLORREF,
}

impl Palette {
    /// 字在浅色：暖白底、钴蓝强调、薄荷色光标。
    pub(super) fn light() -> Self {
        Self {
            accent_color: rgb(0x31, 0x57, 0xd8),
            text_color: rgb(0x17, 0x20, 0x33),
            gloss_color: rgb(0x66, 0x70, 0x85),
            pos_color: rgb(0x98, 0xa2, 0xb3),
            fresh_color: rgb(0xf5, 0x9e, 0x0b),
            index_color: rgb(0x98, 0xa2, 0xb3),
            caret_color: rgb(0x55, 0xd6, 0xc2),
            correction_color: rgb(0xe4, 0x6d, 0x6d),
            background: rgb(0xf7, 0xf6, 0xf2),
            highlight: rgb(0xe8, 0xee, 0xff),
        }
    }

    /// 字在深色：午夜蓝底，语义色与浅色一致。
    pub(super) fn dark() -> Self {
        Self {
            accent_color: rgb(0x6b, 0x8b, 0xff),
            text_color: rgb(0xf7, 0xf6, 0xf2),
            gloss_color: rgb(0xb8, 0xc0, 0xd0),
            pos_color: rgb(0x7f, 0x8a, 0x9e),
            fresh_color: rgb(0xfb, 0xbf, 0x24),
            index_color: rgb(0x7f, 0x8a, 0x9e),
            caret_color: rgb(0x55, 0xd6, 0xc2),
            correction_color: rgb(0xf3, 0x8b, 0x8b),
            background: rgb(0x17, 0x20, 0x33),
            highlight: rgb(0x25, 0x37, 0x5f),
        }
    }
}
