use qingjian_core::ModeKeys;
use serde::{Deserialize, Serialize};

use super::modifiers::Modifiers;

/// 配置文件 `[shortcut]` 分节：前缀模式键（Core 的 [`ModeKeys`]）加壳层的修饰键组合。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutConfig {
    /// 表达式 / 问字模式键，键名与以前一样直接在分节下（`expression` / `question`）。
    #[serde(flatten)]
    pub mode: ModeKeys,

    /// 数字键配这些修饰键：删掉候选（用户词整个删掉，词库词清掉对它的学习）。
    pub delete_candidate: Modifiers,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            mode: ModeKeys::default(),
            delete_candidate: Modifiers::SHIFT,
        }
    }
}

impl ShortcutConfig {
    /// 删候选的修饰键；为空就退回缺省（组句里不拦空修饰键）。旧的配置文件里若还有别的字段，serde 会忽略。
    pub fn delete_keys(&self) -> Modifiers {
        if self.delete_candidate.is_empty() {
            Self::default().delete_candidate
        } else {
            self.delete_candidate
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_files_without_modifier_keys_still_parse_and_get_defaults() {
        let parsed: ShortcutConfig = toml::from_str("expression = \"i\"\n").unwrap();
        assert_eq!(parsed.mode.expression, 'i');
        assert_eq!(parsed.delete_keys(), Modifiers::SHIFT);
    }

    #[test]
    fn removed_translation_keys_in_old_files_are_ignored() {
        let parsed: ShortcutConfig = toml::from_str(
            "translation = \"ctrl\"\ntranslation_second = \"shift+ctrl\"\ntranslate_selection = \"ctrl+alt+t\"\ndelete_candidate = \"ctrl\"\n",
        )
        .unwrap();
        assert_eq!(
            parsed.delete_keys(),
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            }
        );
    }

    #[test]
    fn empty_delete_keys_falls_back_to_default() {
        let parsed = ShortcutConfig {
            delete_candidate: Modifiers::default(),
            ..ShortcutConfig::default()
        };
        assert_eq!(parsed.delete_keys(), Modifiers::SHIFT);
    }
}
