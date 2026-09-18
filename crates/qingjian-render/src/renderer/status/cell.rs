//! 状态条的一格。

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusCell {
    /// 拖动状态条的品牌 Logo；单击没有动作。
    Logo,

    /// 当前中 / 英模式；开着双拼时 `scheme` 在右边显示单字方案标记。
    Mode {
        text: String,
        scheme: Option<String>,
    },

    /// 一段文字；`emphasized` 用品牌色（生效中的全角标点），否则用译文的灰。
    Text { text: String, emphasized: bool },

    /// 打开设置的齿轮。
    Gear,
}

/// 双拼方案在窄状态条上的单字标记。显示面共用这一份，避免设置预览与实际窗口漂移。
pub fn shuangpin_mark(scheme: &str) -> Option<&'static str> {
    match scheme {
        "xiaohe" => Some("鹤"),
        "ziranma" => Some("自"),
        "microsoft" => Some("微"),
        "sogou" => Some("搜"),
        _ => None,
    }
}

impl StatusCell {
    pub fn mode(text: impl Into<String>, scheme: Option<impl Into<String>>) -> Self {
        Self::Mode {
            text: text.into(),
            scheme: scheme.map(Into::into),
        }
    }

    pub fn text(text: impl Into<String>, emphasized: bool) -> Self {
        Self::Text {
            text: text.into(),
            emphasized,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::shuangpin_mark;

    #[test]
    fn shuangpin_marks_are_short_and_unambiguous() {
        assert_eq!(shuangpin_mark("xiaohe"), Some("鹤"));
        assert_eq!(shuangpin_mark("ziranma"), Some("自"));
        assert_eq!(shuangpin_mark("microsoft"), Some("微"));
        assert_eq!(shuangpin_mark("sogou"), Some("搜"));
        assert_eq!(shuangpin_mark("unknown"), None);
    }
}
