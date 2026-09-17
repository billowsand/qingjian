//! 「高级」页：打开配置文件 / 数据目录 / 日志目录、详细日志、学习开关、输入日志。

use qingjian_platform::LogLevel;
use windows_reactor::*;

use crate::panel::controls::{field, group, note, page};
use crate::panel::{Message, Settings};

fn files_group(context: &mut ViewContext<Settings>) -> View {
    group(
        Symbol::Folder,
        "文件与目录",
        [
            field(
                Symbol::Document,
                "配置文件",
                "设置改完会自动生效（Server 每秒看一次配置文件）。只有换学习语言要重启 Server。",
                Button::new()
                    .on_click(context.message(Message::OpenConfigFile))
                    .content("在记事本中打开"),
            ),
            field(
                Symbol::OpenLocal,
                "数据目录",
                "词频、用户词、学习数据与统计都在这里。",
                Button::new()
                    .on_click(context.message(Message::OpenDataDir))
                    .content("打开数据目录"),
            ),
            field(
                Symbol::Page2,
                "日志目录",
                "输入法、引擎与设置程序的日志都在这一个目录，按天分文件，保留 7 天。",
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .children((
                        Button::new()
                            .on_click(context.message(Message::OpenLogDir))
                            .content("打开日志目录"),
                        Button::new()
                            .on_click(context.message(Message::ExportLogs))
                            .content("打包日志到桌面"),
                    )),
            ),
        ],
    )
}

fn diagnostics_group(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let g = &settings.config.general;
    group(
        Symbol::Repair,
        "学习与诊断",
        [
            field(
                Symbol::Favorite,
                "学习输入习惯",
                "按你的选择调整候选顺序、记新词与敲错纠正。关掉后不再学，已学的仍参与排序；学习数据在数据目录里。",
                ToggleSwitch::new()
                    .is_on(g.learning)
                    .on_toggled(context.callback(Message::Learning)),
            ),
            field(
                Symbol::Bullets,
                "详细日志",
                "排查问题时临时打开，会记下敲的拼音与上屏文字。",
                ToggleSwitch::new()
                    .is_on(g.log_level == LogLevel::Debug)
                    .on_toggled(context.callback(Message::VerboseLog)),
            ),
            field(
                Symbol::Edit,
                "记录输入日志",
                "每次上屏记一行，只写本机、不上传，用于离线评测与个人模型。",
                ToggleSwitch::new()
                    .is_on(g.input_log)
                    .on_toggled(context.callback(Message::InputLog)),
            ),
            field(
                Symbol::Clear,
                "清空输入日志",
                "",
                Button::new()
                    .on_click(context.message(Message::ClearInputLog))
                    .content("清空输入日志"),
            ),
        ],
    )
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    page(
        "高级",
        "本地数据、日志与诊断工具",
        [
            files_group(context),
            diagnostics_group(settings, context),
            note("改坏了删掉 config.toml 即可回到默认设置。"),
        ],
    )
}
