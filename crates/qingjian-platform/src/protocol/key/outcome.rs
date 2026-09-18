use serde::{Deserialize, Deserializer, Serialize};

/// Server 对一次按键的处置：DLL 据此决定 TSF 里 `OnKeyDown` 返回吃掉还是放行。
///
/// 反序列化是手写的宽容版（见下方 impl）：加新处置不会打死还留在没重启的应用里的旧 DLL。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub enum KeyOutcome {
    /// 输入法消费了这次按键（进了组句缓冲或触发了上屏 / 翻页等）；DLL 吃掉，应用收不到。
    Consumed,

    /// 输入法不处理（如没在组句时的普通字符、快捷键）；DLL 放行给应用。
    #[default]
    Passthrough,
}

/// 认不出的处置退到 [`KeyOutcome::Passthrough`]：宁可把键还给应用（用户至少打得出字母），
/// 也不让整条应答解析失败。
impl<'de> Deserialize<'de> for KeyOutcome {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        Ok(match name.as_str() {
            "Consumed" => Self::Consumed,
            _ => Self::Passthrough,
        })
    }
}
