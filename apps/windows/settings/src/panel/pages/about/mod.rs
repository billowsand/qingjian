//! 「关于」页：品牌与版本、输入统计、许可证、随包数据的来源与署名（第三方许可要求署名在分发物里可见）、
//! 隐私与反馈。统计原先单独一页，项数太少，并到这里当一块。

mod usage;

use windows_reactor::*;

use crate::panel::controls::{block, note, page};
use crate::panel::{Message, Settings, brand};

pub(crate) const WEBSITE_URL: &str = "https://qingjian.app";

pub(crate) const REPOSITORY_URL: &str = "https://github.com/qingjian-team";

/// 与仓库根 `LICENSE` 一致。
const LICENSE_NOTE: &str = "自由软件，GPL-3.0-or-later 许可证：可以自由使用、修改与再分发，修改后分发须同样开源。官方渠道免费。";

/// 只列随包数据的来源；Windows 壳不带释义表与词汇等级表，所以比 macOS「关于」页少那两条。
const ATTRIBUTIONS: &[(&str, &str)] = &[
    (
        "词库",
        "通用规范汉字表；现代汉语常用词表（liuxilu 校对版）；THUOCL（清华大学自然语言处理实验室，MIT）；读音取自 Unihan（Unicode License v3）。",
    ),
    (
        "语言模型",
        "中文维基百科（CC BY-SA 4.0）与 LCCC（清华大学 CoAI，MIT）语料统计。",
    ),
    ("emoji", "Unicode CLDR annotations（Unicode License v3）。"),
    (
        "英文词表",
        "ESDB / SCOWL（© Kevin Atkinson，按其许可保留版权声明）；CSpell 词典（MIT）。",
    ),
];

const PRIVACY_NOTE: &str = "字在不上传任何数据：拼音、组词、排序与学习都在本机完成。「高级」页的输入日志只写在本机，可以关掉或清空。";

const FEEDBACK_NOTE: &str = "遇到问题点「打包日志到桌面」，把生成的 zip 发给作者即可（含三个进程的日志与配置文件，不含密钥）。缺省日志不含你敲的内容；排查排序问题时作者可能请你在「高级」页临时打开详细日志。";

/// 大号 Logo + 名字 + 一句话 + 版本与构建。
fn identity() -> View {
    let text = StackPanel::new().spacing(4.0).children([
        TextBlock::new()
            .text("字在")
            .font_size(28.0)
            .font_weight(FontWeight::SEMI_BOLD)
            .into(),
        note("字在本机，表达自在。"),
        TextBlock::new()
            .text(concat!("字在 Windows ", env!("QINGJIAN_VERSION")))
            .font_size(14.0)
            .font_weight(FontWeight::SEMI_BOLD)
            .into(),
        // QINGJIAN_BUILD 由 build.rs 从 git 取
        note(&format!(
            "构建 {}",
            option_env!("QINGJIAN_BUILD").unwrap_or("本地构建")
        )),
    ]);
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(16.0)
        .children((
            brand::logo(72.0).vertical_alignment(VerticalAlignment::Top),
            text,
        ))
}

fn links(context: &mut ViewContext<Settings>) -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(12.0)
        .children((
            Button::new()
                .on_click(context.message(Message::OpenWebsite))
                .content("官网"),
            Button::new()
                .on_click(context.message(Message::OpenRepository))
                .content("GitHub"),
            Button::new()
                .on_click(context.message(Message::OpenDataDir))
                .content("打开数据目录"),
            Button::new()
                .on_click(context.message(Message::ExportLogs))
                .content("打包日志到桌面"),
        ))
}

fn credits() -> View {
    let attributions = ATTRIBUTIONS
        .iter()
        .enumerate()
        .map(|(index, (name, text))| {
            KeyedView::new(index.to_string(), note(&format!("{name}：{text}")))
        });
    block(
        Symbol::ProtectedDocument,
        "许可证与数据来源",
        StackPanel::new().spacing(8.0).children((
            note(LICENSE_NOTE),
            StackPanel::new().spacing(6.0).keyed_children(attributions),
        )),
    )
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    page(
        "关于",
        [
            StackPanel::new()
                .spacing(16.0)
                .children((identity(), links(context))),
            usage::view(settings),
            credits(),
            block(
                Symbol::Permissions,
                "隐私",
                StackPanel::new().children([note(PRIVACY_NOTE)]),
            ),
            block(
                Symbol::Message,
                "反馈",
                StackPanel::new().children([note(FEEDBACK_NOTE)]),
            ),
        ],
    )
}
