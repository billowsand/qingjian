use serde::{Deserialize, Deserializer, Serialize};

use qingjian_core::MarkedKind;

/// preedit 片段的种类，DLL 按它选样式。是 Core 的 [`MarkedKind`] 的可序列化镜像
/// （协议不直接用 Core 的内部枚举，免得两者耦合）。
///
/// 反序列化是手写的宽容版（见下方 impl）：加新种类不会打死还留在没重启的应用里的旧 DLL。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub enum PreeditKind {
    /// 用户敲的、参与本次候选的拼音（已按音节用 `'` 切开）。
    #[default]
    Typed,

    /// 光标停在中间时，作用域之后剩下的拼音：只显示不参与候选，画淡一点。
    Rest,

    /// 激活的辅码段（末尾那两个键）：不是拼音，画淡一点与拼音区分开。
    Fuma,

    /// 拼写纠错里被改掉的原字母：画删除线。
    Corrected,
}

/// 认不出的种类退到 [`PreeditKind::Typed`]（照常当拼音画），不让整帧解析失败：
/// 新版本加种类时，还留在没重启的应用里的旧 DLL 才不至于每键都失败。
impl<'de> Deserialize<'de> for PreeditKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        Ok(match name.as_str() {
            "Rest" => Self::Rest,
            "Fuma" => Self::Fuma,
            "Corrected" => Self::Corrected,
            _ => Self::Typed,
        })
    }
}

impl From<MarkedKind> for PreeditKind {
    fn from(kind: MarkedKind) -> Self {
        match kind {
            MarkedKind::Typed => Self::Typed,
            MarkedKind::Rest => Self::Rest,
            MarkedKind::Fuma => Self::Fuma,
            MarkedKind::Corrected => Self::Corrected,
        }
    }
}
