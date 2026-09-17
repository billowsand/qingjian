//! 「关于」页里的输入统计：输入量（今天 / 7 天 / 累计）、折成几本书。
//! 直读 `%APPDATA%\Qingjian` 下的 `usage.tsv`，不经 Server；打开这页时读一次。

use eframe::egui;
use jiff::Zoned;
use qingjian_core::{Usage, UsageSummary, book_scale};
use qingjian_learning::UsageStats;

use crate::app::Settings;
use crate::widgets::{LABEL_SIZE, block, note};

const COLUMNS: [&str; 4] = ["汉字", "中文词", "英文词", "上屏次数"];

pub(crate) fn view(settings: &Settings, ui: &mut egui::Ui) {
    let today = Zoned::now().date();
    let usage = UsageStats::open(settings.data_dir().join("usage.tsv")).summary_on(today);
    block(ui, "\u{E8EF}", "输入统计", |ui| {
        egui::Grid::new("usage-table")
            .num_columns(COLUMNS.len() + 1)
            .spacing([18.0, 4.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("").size(LABEL_SIZE));
                for column in COLUMNS {
                    ui.label(egui::RichText::new(column).size(LABEL_SIZE).strong());
                }
                ui.end_row();
                row(ui, "今天", &usage.today);
                row(ui, "最近 7 天", &usage.week);
                row(ui, "累计", &usage.total);
            });
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(scale_line(usage.total.hanzi))
                .size(LABEL_SIZE)
                .strong(),
        );
        ui.add_space(2.0);
        note(ui, &since_line(&usage));
        note(
            ui,
            "数的是上屏的文字：选一个词算一个中文词，整句按词切开数；英文候选、回车原样上屏的英文词算英文词。只在这台电脑上数，与输入日志无关。",
        );
    });
}

fn row(ui: &mut egui::Ui, label: &str, usage: &Usage) {
    ui.label(egui::RichText::new(label).size(LABEL_SIZE));
    for value in [usage.hanzi, usage.words, usage.english_words, usage.commits] {
        ui.label(egui::RichText::new(group_digits(value)).size(LABEL_SIZE));
    }
    ui.end_row();
}

fn since_line(summary: &UsageSummary) -> String {
    match &summary.since {
        Some(date) => format!("自 {date} 起，有输入的天数 {}。", summary.days),
        None => "还没有记录，打几个字再来看。".to_owned(),
    }
}

/// 「累计输入 20.4 万字，约等于 1.7 本《活着》（约 12 万字）。」
fn scale_line(hanzi: u64) -> String {
    if hanzi == 0 {
        return "累计输入 0 字。".to_owned();
    }
    let (book, ratio) = book_scale(hanzi);
    format!(
        "累计输入 {}，约等于 {} 本《{}》（约 {}）。",
        hanzi_text(hanzi),
        format_ratio(ratio),
        book.title,
        hanzi_text(book.hanzi),
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
