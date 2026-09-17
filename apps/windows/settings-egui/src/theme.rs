//! 配色：直接取候选窗渲染器的 `Palette`，设置界面与候选窗同一套色（WinUI 版只能靠 `ThemeBrush` 跟系统走，
//! 品牌色对不上）。明暗跟随系统（egui 的 `ThemePreference::System`，winit 报系统主题）。

use eframe::egui;
use qingjian_render::{Color, Palette};

use crate::widgets::{CARD_PADDING_X, CARD_PADDING_Y, CARD_RADIUS, NOTE_OPACITY};

/// 装两套 Visuals（浅色 / 深色），让 egui 按系统主题挑。
pub(crate) fn install(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::System);
    ctx.set_visuals_of(egui::Theme::Light, visuals(egui::Visuals::light(), false));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(egui::Visuals::dark(), true));
}

fn palette(dark: bool) -> Palette {
    if dark {
        Palette::dark()
    } else {
        Palette::light()
    }
}

fn color32(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

fn visuals(base: egui::Visuals, dark: bool) -> egui::Visuals {
    let colors = palette(dark);
    let mut visuals = base;
    visuals.panel_fill = color32(colors.background);
    visuals.window_fill = color32(colors.background);
    visuals.extreme_bg_color = color32(colors.background);
    visuals.override_text_color = Some(color32(colors.text));
    visuals.selection.bg_fill = color32(colors.highlight);
    visuals.selection.stroke = egui::Stroke::new(1.0, color32(colors.accent));
    visuals.hyperlink_color = color32(colors.accent);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(CARD_RADIUS as u8);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
    visuals
}

fn dark_mode(ctx: &egui::Context) -> bool {
    ctx.style().visuals.dark_mode
}

/// 说明小字的颜色（WinUI 版用的是正文色 + 0.65 不透明度）。
pub(crate) fn note_color(ctx: &egui::Context) -> egui::Color32 {
    let colors = palette(dark_mode(ctx));
    color32(colors.gloss).gamma_multiply(NOTE_OPACITY)
}

/// 导航选中项的底色。
pub(crate) fn selected_fill(ctx: &egui::Context) -> egui::Color32 {
    let colors = palette(dark_mode(ctx));
    color32(colors.highlight)
}

/// 强调色（品牌蓝），开关打开时用。
pub(crate) fn accent(ctx: &egui::Context) -> egui::Color32 {
    color32(palette(dark_mode(ctx)).accent)
}

/// 左侧导航栏的底：比正文区略深一档，和 WinUI 的 NavigationView 分区一致。
pub(crate) fn nav_frame(ctx: &egui::Context) -> egui::Frame {
    let dark = dark_mode(ctx);
    let fill = if dark {
        color32(palette(true).background).gamma_multiply(0.82)
    } else {
        color32(palette(false).background).gamma_multiply(0.97)
    };
    egui::Frame::NONE
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(12, 16))
}

/// 正文区的底。
pub(crate) fn page_frame(ctx: &egui::Context) -> egui::Frame {
    egui::Frame::NONE
        .fill(color32(palette(dark_mode(ctx)).background))
        .inner_margin(egui::Margin::symmetric(28, 20))
}

/// 一张设置卡片的外框：底色 + 1 px 描边 + 圆角，取值与 WinUI 版 `controls/mod.rs` 相同。
pub(crate) fn card_frame(ctx: &egui::Context) -> egui::Frame {
    let dark = dark_mode(ctx);
    let (fill, stroke) = if dark {
        (
            egui::Color32::from_rgb(38, 41, 48),
            egui::Color32::from_rgb(58, 62, 70),
        )
    } else {
        (
            egui::Color32::from_rgb(255, 255, 255),
            egui::Color32::from_rgb(228, 226, 219),
        )
    };
    egui::Frame::NONE
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(CARD_RADIUS)
        .inner_margin(egui::Margin::symmetric(
            CARD_PADDING_X as i8,
            CARD_PADDING_Y as i8,
        ))
}
