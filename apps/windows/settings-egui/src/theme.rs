//! 配色：直接取候选窗渲染器的 `Palette`，设置界面与候选窗同一套色（WinUI 版只能靠 `ThemeBrush` 跟系统走，
//! 品牌色对不上）。
//!
//! 明暗**自己读注册表**跟随系统，不用 `ThemePreference::System`：实测在 Windows 上把系统切成深色后
//! winit 没把主题变化报过来，窗口一直停在浅色。读的是 Server 判断候选窗深浅的同一个键，两边不会打架。

use eframe::egui;
use qingjian_render::{Color, Palette};

use crate::widgets::{CARD_PADDING_X, CARD_PADDING_Y, CARD_RADIUS, NOTE_OPACITY};

/// 控件高度基准。加上 `button_padding` 与卡片内边距，一行设置正好 34 px（WinUI 版两行文字要 76）。
const INTERACT_HEIGHT: f32 = 18.0;

/// 装两套 Visuals（浅色 / 深色）与更紧的间距，再按当前系统明暗选一套。
pub(crate) fn install(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(6.0, 3.0);
        style.spacing.button_padding = egui::vec2(8.0, 2.0);
        style.spacing.interact_size.y = INTERACT_HEIGHT;
        style.spacing.combo_height = 320.0;
    });
    ctx.set_visuals_of(egui::Theme::Light, visuals(egui::Visuals::light(), false));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(egui::Visuals::dark(), true));
    ctx.set_theme(preference(system_prefers_dark()));
}

/// `HKCU\...\Themes\Personalize\AppsUseLightTheme` 为 0 是深色；读不到当浅色。
/// 与 `server/src/ui/candidates/mod.rs::system_prefers_dark` 同一个判断。
pub(crate) fn system_prefers_dark() -> bool {
    windows_registry::CURRENT_USER
        .open(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|key| key.get_u32("AppsUseLightTheme"))
        .is_ok_and(|value| value == 0)
}

/// 系统明暗变了就换一套 Visuals（在 Windows 的设置里切主题，窗口开着也跟）。
///
/// egui 只在有输入时重绘，所以这里顺手预约下一次醒来——不然鼠标不动就看不到主题切过来。
pub(crate) fn follow_system(ctx: &egui::Context, dark: &mut bool) {
    ctx.request_repaint_after(std::time::Duration::from_millis(800));
    let now = system_prefers_dark();
    if now != *dark {
        *dark = now;
        ctx.set_theme(preference(now));
        // 标题栏是系统画的，跟着一起换（不换就出现深色标题栏 + 浅色正文）。
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(if now {
            egui::SystemTheme::Dark
        } else {
            egui::SystemTheme::Light
        }));
    }
}

fn preference(dark: bool) -> egui::ThemePreference {
    if dark {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    }
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
