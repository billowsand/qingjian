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

/// 导航栏顶部的品牌条：Logo + 「字在设置」+ 版本号。
pub(super) fn header() -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(10.0)
        .margin(Thickness::new(16.0, 8.0, 16.0, 8.0))
        .children((
            logo(28.0).vertical_alignment(VerticalAlignment::Center),
            TextBlock::new()
                .text("字在设置")
                .font_size(18.0)
                .font_weight(FontWeight::SEMI_BOLD)
                .vertical_alignment(VerticalAlignment::Center),
            TextBlock::new()
                .text(VERSION)
                .font_size(12.0)
                .opacity(0.6)
                .vertical_alignment(VerticalAlignment::Bottom),
        ))
}
