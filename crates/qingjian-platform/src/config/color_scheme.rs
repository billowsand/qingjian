//! 界面色系：明暗一律跟随系统，每套色系各自提供浅色与深色。

use serde::{Deserialize, Deserializer, Serialize};

/// 候选窗口、状态条与设置程序共用的色系。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    /// 暖奶油底与陶土强调色。
    Cream,

    /// 字在的钴蓝与薄荷品牌色。
    #[default]
    Zizai,

    /// 紫藤与拿铁棕。
    Latte,

    /// 松林与鼠尾草。
    Forest,
}

impl ColorScheme {
    /// 设置界面中的排列顺序。
    pub const ALL: [Self; 4] = [Self::Cream, Self::Zizai, Self::Latte, Self::Forest];

    /// 配置文件里的写法。
    pub const fn key(self) -> &'static str {
        match self {
            Self::Cream => "cream",
            Self::Zizai => "zizai",
            Self::Latte => "latte",
            Self::Forest => "forest",
        }
    }

    /// 界面上的名字。
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cream => "奶油",
            Self::Zizai => "字在蓝",
            Self::Latte => "紫藤拿铁",
            Self::Forest => "森林",
        }
    }
}

impl<'de> Deserialize<'de> for ColorScheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key = String::deserialize(deserializer)?;
        match key.trim().to_ascii_lowercase().as_str() {
            "cream" => Ok(Self::Cream),
            "zizai" | "system" | "light" | "dark" => Ok(Self::Zizai),
            "latte" => Ok(Self::Latte),
            "forest" => Ok(Self::Forest),
            _ => Err(serde::de::Error::unknown_variant(
                &key,
                &["cream", "zizai", "latte", "forest"],
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::ColorScheme;

    #[derive(Deserialize)]
    struct Wrapper {
        value: ColorScheme,
    }

    #[test]
    fn retired_brightness_values_migrate_to_zizai() {
        for old in ["system", "light", "dark"] {
            let parsed: Wrapper = toml::from_str(&format!("value = \"{old}\"")).unwrap();
            assert_eq!(parsed.value, ColorScheme::Zizai);
        }
    }
}
