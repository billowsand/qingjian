//! 字在的品牌标识：Logo 编进 exe（`assets/icon/logo.png`），导航栏顶部与「关于」页共用。

use windows_reactor::*;

/// 版本号由 build.rs 给：`-dev` 版接 git 短哈希。
pub(super) const VERSION: &str = env!("QINGJIAN_VERSION");

/// 编进二进制的应用图标，不依赖随包文件。
const LOGO: &[u8] = include_bytes!("../../../../../assets/icon/logo.png");

/// 指定边长的 Logo。
pub(super) fn logo(size: f64) -> Image {
    Image::new()
        .source_data(EncodedImage::from_static(LOGO))
        .stretch(Stretch::Uniform)
        .width(size)
        .height(size)
}

/// 导航栏顶部的品牌条：保持与安装器、设计稿相同的 Logo + 名字 + 品牌句层级。
pub(super) fn header() -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(12.0)
        .margin(Thickness::new(16.0, 12.0, 16.0, 16.0))
        .children((
            logo(44.0).vertical_alignment(VerticalAlignment::Center),
            StackPanel::new()
                .spacing(1.0)
                .vertical_alignment(VerticalAlignment::Center)
                .children((
                    TextBlock::new()
                        .text("字在")
                        .font_size(20.0)
                        .font_weight(FontWeight::SEMI_BOLD),
                    TextBlock::new()
                        .text("更自在的输入")
                        .font_size(12.0)
                        .opacity(0.62),
                )),
        ))
}
