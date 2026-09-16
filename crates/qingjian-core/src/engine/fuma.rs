//! 辅码激活的判定：从缓冲区里剥出辅码段，供解码归一化与候选过滤用。
//!
//! 键位规则与水杉（MSIME-Engine）一致：末 2 键都是字母、至少一个大写、
//! 且去掉它们之后的前缀全部解成完整双拼音节时，末 2 键是对候选的严格过滤键；
//! 前缀不完整时末尾字母仍按普通拼音处理，全小写时不激活（末键仍是下一音节的声母）。
//! 第一键大写表示两码反转顺序匹配（`Ke` / `KE` 匹配实际辅码 `ek`）。
//!
//! 判定挂在每次 [`Engine::decode`] 上，所以没开辅码、或末 2 键里没有大写时必须一个堆分配都不做。

use std::borrow::Cow;

use crate::shuangpin::Scheme;

use super::Engine;

/// 缓冲区里激活的辅码段。
pub(super) struct FumaInput<'a> {
    /// 剥掉辅码段并小写化后的键串（喂给双拼解码）；本来就全小写时不复制。
    pub base: Cow<'a, str>,

    /// 敲出的两码：已小写、已按反转规则排好。
    pub typed: [char; 2],
}

impl Engine {
    /// 辅码生效所需的双拼方案：要有表、在双拼下、且不在注音模式
    /// （注音走自己的解码，末 2 键不是双拼键，辅码无从算起）。
    fn fuma_scheme(&self) -> Option<Scheme> {
        self.fuma.as_ref()?;
        if self.zhuyin {
            return None;
        }
        self.shuangpin
    }

    /// 缓冲区（或任意键串）里激活的辅码段；没开辅码、不是双拼或条件不满足时为 `None`。
    pub(super) fn fuma_input<'a>(&self, keys: &'a str) -> Option<FumaInput<'a>> {
        let scheme = self.fuma_scheme()?;
        // 末 2 键按字节看：ASCII 字母不可能是多字节字符的一部分，所以字节是字母就等于字符是字母，
        // 也就保证了 `keys.len() - 2` 落在字符边界上
        let [.., first, second] = keys.as_bytes() else {
            return None;
        };
        let (first, second) = (*first as char, *second as char);
        if !(first.is_ascii_alphabetic() && second.is_ascii_alphabetic()) {
            return None;
        }
        if !first.is_ascii_uppercase() && !second.is_ascii_uppercase() {
            return None;
        }
        // 前缀至少要够一个双拼音节（两键），否则不可能解成完整音节
        let base = &keys[..keys.len() - 2];
        if base.len() < 2 {
            return None;
        }
        let base = lowercased(base);
        if !scheme.decode(&base).is_complete() {
            return None;
        }
        let mut typed = [first.to_ascii_lowercase(), second.to_ascii_lowercase()];
        if first.is_ascii_uppercase() {
            typed.swap(0, 1);
        }
        Some(FumaInput { base, typed })
    }

    /// 激活的辅码段占几个字节（0 或 2）。
    pub(super) fn fuma_bytes(&self, keys: &str) -> usize {
        usize::from(self.fuma_input(keys).is_some()) * 2
    }

    /// 辅码激活时 `text` 能不能出候选（严格过滤）；没激活一律放行。
    pub(super) fn fuma_admits(&self, text: &str) -> bool {
        let Some(typed) = self.fuma_input(self.composition.scope()).map(|f| f.typed) else {
            return true;
        };
        self.fuma
            .as_ref()
            .is_some_and(|table| table.matches(text, typed))
    }

    /// 辅码功能是否开着（有表、双拼、非注音）：壳据此决定组句中 Shift + 字母进缓冲区还是临时打英文。
    pub fn fuma_enabled(&self) -> bool {
        self.fuma_scheme().is_some()
    }
}

/// 小写化，本来就没有大写时原样借用（辅码判定挂在每次解码上，不能每次都复制一份键串）。
pub(super) fn lowercased(keys: &str) -> Cow<'_, str> {
    if keys.bytes().any(|b| b.is_ascii_uppercase()) {
        Cow::Owned(keys.to_ascii_lowercase())
    } else {
        Cow::Borrowed(keys)
    }
}
