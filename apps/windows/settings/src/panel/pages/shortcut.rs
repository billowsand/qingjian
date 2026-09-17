//! 「快捷键」页：翻页键、删候选的修饰键。

use qingjian_platform::Modifiers;
use windows_reactor::*;

use crate::panel::controls::{field, index_of, page};
use crate::panel::{Message, Settings};

/// 翻页键对：界面名 + 配置写法。
pub(crate) const PAGE_KEYS: [(&str, &str); 3] = [
    ("方括号 [ ]", "[]"),
    ("逗号句号 , .", ",."),
    ("减号等号 - =", "-="),
];

/// 修饰键预设：界面名 + 配置写法。
pub(crate) const MODIFIERS: [(&str, &str); 6] = [
    ("Ctrl", "ctrl"),
    ("Alt", "alt"),
    ("Shift", "shift"),
    ("Ctrl + Shift", "shift+ctrl"),
    ("Ctrl + Alt", "ctrl+alt"),
    ("Alt + Shift", "shift+alt"),
];

/// 按解析后相等找当前项，不依赖字符串写法。
fn modifier_combo(current: Modifiers, callback: Callback<Option<usize>>) -> ComboBox {
    let selected = MODIFIERS
        .iter()
        .position(|(_, value)| value.parse::<Modifiers>().ok() == Some(current))
        .unwrap_or(0);
    ComboBox::new()
        .items_source(MODIFIERS.iter().map(|(label, _)| *label))
        .selected_index(selected)
        .on_selection_changed(callback)
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let rows = [
        field(
            "翻页键",
            "选「, .」或「- =」时组句中敲对应符号是翻页，不再是上屏加标点。",
            ComboBox::new()
                .items_source(PAGE_KEYS.iter().map(|(label, _)| *label))
                .selected_index(index_of(&PAGE_KEYS, &settings.config.general.page_keys))
                .on_selection_changed(context.callback(Message::PageKeys)),
        ),
        field(
            "删除候选",
            "按住修饰键再按候选序号：自己造的词整删；词库里的词清掉学习记录，回到原排序。",
            modifier_combo(
                settings.config.shortcut.delete_candidate,
                context.callback(Message::DeleteCandidate),
            ),
        ),
    ];
    page("快捷键", StackPanel::new().spacing(16.0).children(rows))
}
