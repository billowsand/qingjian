//! 「通用」页：每页候选数、双拼、注音、标点与中英混输。

use qingjian_platform::MAX_PAGE_SIZE;
use windows_reactor::*;

use crate::panel::controls::{field, index_of, page};
use crate::panel::{Message, Settings};

/// 双拼方案：界面名 + 配置写法（空串为全拼）。
pub(crate) const SHUANGPIN: [(&str, &str); 5] = [
    ("全拼（不启用双拼）", ""),
    ("小鹤双拼", "xiaohe"),
    ("自然码", "ziranma"),
    ("微软双拼", "microsoft"),
    ("搜狗双拼", "sogou"),
];

/// 辅码方案：界面名 + 配置写法（空串为关）。
pub(crate) const FUMA: [(&str, &str); 2] = [("关（不启用辅码）", ""), ("小鹤辅码", "xiaohe")];

fn string_combo(
    options: &'static [(&str, &str)],
    current: &str,
    callback: Callback<Option<usize>>,
) -> ComboBox {
    ComboBox::new()
        .items_source(options.iter().map(|(label, _)| *label))
        .selected_index(index_of(options, current))
        .on_selection_changed(callback)
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let g = &settings.config.general;
    let rows = [
        field(
            "每页候选数",
            "",
            NumberBox::new()
                .minimum(1.0)
                .maximum(MAX_PAGE_SIZE as f64)
                .value(g.page_size as f64)
                .on_value_changed(context.callback(Message::PageSize)),
        ),
        field(
            "双拼",
            "开双拼后两键按方案表拆成声母 + 韵母；微软、搜狗方案的 ; 键是 ing 韵母键。",
            string_combo(
                &SHUANGPIN,
                &g.shuangpin,
                context.callback(Message::Shuangpin),
            ),
        ),
        field(
            "辅码",
            "开双拼后可用：打完双拼再敲两个大写辅码键严格筛选候选（首字第 1 码 + 末字第 1 码，单字取两码），对不上就不出候选；第一码大写表示反转顺序。辅码键不是要打的内容。",
            {
                let combo = string_combo(&FUMA, &g.fuma, context.callback(Message::Fuma));
                let has_shuangpin = !g.shuangpin.trim().is_empty();
                combo.is_enabled(has_shuangpin)
            },
        ),
        field(
            "大千注音",
            "启用大千注音键盘布局；容错按标准大千键位（ㄢㄤ、ㄣㄥ 各自独立）。",
            ToggleSwitch::new()
                .is_on(g.zhuyin)
                .on_toggled(context.callback(Message::Zhuyin)),
        ),
        field(
            "中文模式标点转全角",
            "没在打拼音时敲 , . ? ! 等出「，。？！」，数字后面的点保持半角；悬浮状态条的「，。」格也能切，切的是当前模式那份。",
            ToggleSwitch::new()
                .is_on(g.full_width_punctuation)
                .on_toggled(context.callback(Message::FullWidthPunctuation)),
        ),
        field(
            "英文模式标点转全角",
            "中英各记一份，缺省英文半角。",
            ToggleSwitch::new()
                .is_on(g.english_full_width_punctuation)
                .on_toggled(context.callback(Message::EnglishFullWidthPunctuation)),
        ),
        field(
            "输入拼音时中文候选排在英文词前面",
            "开着时整段输入是英文词时（hello、key）英文词排第二，空格上屏的仍是中文；关着（缺省）拼音不成立的输入英文词排第一。",
            ToggleSwitch::new()
                .is_on(g.chinese_first)
                .on_toggled(context.callback(Message::ChineseFirst)),
        ),
    ];
    page("通用", StackPanel::new().spacing(16.0).children(rows))
}
