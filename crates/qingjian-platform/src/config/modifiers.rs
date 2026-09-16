use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// 一组修饰键。配置里写成 `ctrl+alt` 这样的串（顺序随意，`win` / `cmd` 也认，
/// 老的 macOS 写法 `option` / `command` 也照收，分别当 Alt 与 Win）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Modifiers {
    /// Alt
    pub alt: bool,

    /// Shift
    pub shift: bool,

    /// Ctrl
    pub ctrl: bool,

    /// Win（⊞）
    pub win: bool,
}

impl Modifiers {
    pub const SHIFT: Self = Self {
        alt: false,
        shift: true,
        ctrl: false,
        win: false,
    };

    pub fn is_empty(&self) -> bool {
        !(self.alt || self.shift || self.ctrl || self.win)
    }

    /// 配置文件里的写法，固定顺序 ctrl+shift+alt+win。
    pub fn key(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("ctrl");
        }
        if self.shift {
            parts.push("shift");
        }
        if self.alt {
            parts.push("alt");
        }
        if self.win {
            parts.push("win");
        }
        parts.join("+")
    }
}

impl FromStr for Modifiers {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut out = Self::default();
        for token in text.split(['+', ' ']).filter(|t| !t.is_empty()) {
            match token.to_ascii_lowercase().as_str() {
                "alt" | "option" | "⌥" => out.alt = true,
                "shift" | "⇧" => out.shift = true,
                "control" | "ctrl" | "⌃" => out.ctrl = true,
                "win" | "command" | "cmd" | "⌘" => out.win = true,
                other => return Err(format!("unknown modifier: {other}")),
            }
        }
        if out.is_empty() {
            return Err("no modifier given".to_owned());
        }
        Ok(out)
    }
}

impl TryFrom<String> for Modifiers {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl From<Modifiers> for String {
    fn from(modifiers: Modifiers) -> Self {
        modifiers.key()
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aliases_in_any_order_and_prints_canonically() {
        let both: Modifiers = "alt+shift".parse().unwrap();
        assert_eq!(
            both,
            Modifiers {
                alt: true,
                shift: true,
                ctrl: false,
                win: false,
            }
        );
        assert_eq!(both.key(), "shift+alt");
        assert_eq!("ctrl+alt".parse::<Modifiers>().unwrap().key(), "ctrl+alt");
        assert_eq!("option".parse::<Modifiers>().unwrap().key(), "alt");
        assert_eq!(
            "shift+command".parse::<Modifiers>().unwrap().key(),
            "shift+win"
        );
        assert!("".parse::<Modifiers>().is_err());
        assert!("hyper".parse::<Modifiers>().is_err());
        for combo in ["ctrl", "shift+alt", "ctrl+shift+win"] {
            assert_eq!(combo.parse::<Modifiers>().unwrap().key(), combo);
        }
    }
}
