//! 「本地整句模型」页：随包小模型的开关（`[model] enabled`），全程离线。

use windows_reactor::*;

use crate::panel::controls::{field, note, page};
use crate::panel::{Message, Settings};

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let rows = [
        field(
            "本地整句模型",
            "随包的小模型在本机给整句候选重新排序，全程离线；停键后几十毫秒生效。关掉只用词库统计。",
            ToggleSwitch::new()
                .is_on(settings.config.model.enabled)
                .on_toggled(context.callback(Message::LocalModel)),
        ),
        note(
            "模型文件优先读用户目录 model/ 下的 .qjm，其次安装目录 data/model/；改动即时生效（Server 热加载）。",
        ),
    ];
    page(
        "本地整句模型",
        StackPanel::new().spacing(16.0).children(rows),
    )
}
