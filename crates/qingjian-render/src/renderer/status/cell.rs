//! 状态条的一格。

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusCell {
    /// 拖动状态条的点阵握柄；单击没有动作。
    Grip,

    /// 当前中 / 英模式：主字画在品牌色方章里；中文态用薄荷点表示本地生效，
    /// 开着双拼时 `scheme` 在方章右边显示单字方案标记。
    Mode {
        text: String,
        scheme: Option<String>,
        local: bool,
    },

    /// 一段文字；`emphasized` 用品牌色（当前模式、生效中的全角标点），否则用译文的灰。
    Text { text: String, emphasized: bool },

    /// 打开设置的「字在」品牌图标。
    Brand,
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
    pub fn mode(text: impl Into<String>, scheme: Option<impl Into<String>>, local: bool) -> Self {
        Self::Mode {
            text: text.into(),
            scheme: scheme.map(Into::into),
            local,
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
