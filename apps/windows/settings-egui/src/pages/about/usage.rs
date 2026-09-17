//! 「关于」页里的输入统计：今天 / 最近 7 天 / 累计各一行，累计那行折成几本书。
//! 直读 `%APPDATA%\Qingjian` 下的 `usage.tsv`，不经 Server；打开这页时读一次。

use jiff::Zoned;
use qingjian_core::{Usage, UsageSummary, book_scale};
use qingjian_learning::UsageStats;

use crate::app::Settings;

pub(crate) fn summary(settings: &Settings) -> UsageSummary {
    let today = Zoned::now().date();
    UsageStats::open(settings.data_dir().join("usage.tsv")).summary_on(today)
}

/// 一行的右侧：「4,482 字」。
pub(crate) fn hanzi(usage: &Usage) -> String {
    format!("{} 字", group_digits(usage.hanzi))
}

/// 一行的悬停提示：中文词 / 英文词 / 上屏次数。
pub(crate) fn detail(usage: &Usage) -> String {
    format!(
        "中文词 {} · 英文词 {} · 上屏 {} 次",
        group_digits(usage.words),
        group_digits(usage.english_words),
        group_digits(usage.commits)
    )
}

/// 「累计 20.4 万字，约 1.7 本《活着》」。
pub(crate) fn scale_line(summary: &UsageSummary) -> String {
    let hanzi = summary.total.hanzi;
    if hanzi == 0 {
        return "还没有记录，打几个字再来看。".to_owned();
    }
    let (book, ratio) = book_scale(hanzi);
    let since = match &summary.since {
        Some(date) => format!("；自 {date} 起有输入的天数 {}", summary.days),
        None => String::new(),
    };
    format!(
        "累计 {}，约 {} 本《{}》{since}",
        hanzi_text(hanzi),
        format_ratio(ratio),
        book.title,
    )
}

/// 千位分隔。
fn group_digits(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// 万以下整数，万 / 亿以上一位小数。
fn hanzi_text(value: u64) -> String {
    const WAN: f64 = 10_000.0;
    const YI: f64 = 100_000_000.0;
    let value_f = value as f64;
    if value_f >= YI {
        format!("{} 亿字", trim_decimal(value_f / YI))
    } else if value_f >= WAN {
        format!("{} 万字", trim_decimal(value_f / WAN))
    } else {
        format!("{} 字", group_digits(value))
    }
}

/// 一位小数，`.0` 去掉。
fn trim_decimal(value: f64) -> String {
    let text = format!("{value:.1}");
    text.strip_suffix(".0").map_or(text.clone(), str::to_owned)
}

/// 不到 10 倍留一位小数，再多取整。
fn format_ratio(ratio: f64) -> String {
    if ratio >= 10.0 {
        format!("{}", ratio.round() as u64)
    } else {
        trim_decimal(ratio)
    }
}
