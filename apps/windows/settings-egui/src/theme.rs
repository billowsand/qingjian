//! 配色：直接取候选窗渲染器的 `Palette`，设置界面与候选窗同一套色（WinUI 版只能靠 `ThemeBrush` 跟系统走，
//! 品牌色对不上）。
//!
//! 明暗**自己读注册表**跟随系统，不用 `ThemePreference::System`：实测在 Windows 上把系统切成深色后
//! winit 没把主题变化报过来，窗口一直停在浅色。读的是 Server 判断候选窗深浅的同一个键，两边不会打架。

use eframe::egui;
use qingjian_platform::ColorScheme;
use qingjian_render::{Color, Palette};

use crate::widgets::{CARD_PADDING_X, CARD_PADDING_Y, CARD_RADIUS, NOTE_OPACITY};

/// 控件高度基准；行高由 `widgets::list` 定，这里只管控件自己多高。
const INTERACT_HEIGHT: f32 = 20.0;

/// 下拉框 / 按钮的圆角。
const CONTROL_RADIUS: u8 = 5;

/// 装两套 Visuals（浅色 / 深色）与更紧的间距，再按当前系统明暗选一套。
pub(crate) fn install(ctx: &egui::Context, scheme: ColorScheme) {
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(6.0, 3.0);
        style.spacing.button_padding = egui::vec2(10.0, 3.0);
        style.spacing.interact_size.y = INTERACT_HEIGHT;
        style.spacing.combo_height = 320.0;
    });
    apply_visuals(ctx, scheme);
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
pub(crate) fn follow_system(
    ctx: &egui::Context,
    dark: &mut bool,
    applied_scheme: &mut ColorScheme,
    scheme: ColorScheme,
) {
    ctx.request_repaint_after(std::time::Duration::from_millis(800));
    let now = system_prefers_dark();
    let scheme_changed = scheme != *applied_scheme;
    if scheme_changed {
        apply_visuals(ctx, scheme);
        *applied_scheme = scheme;
    }
    if now != *dark || scheme_changed {
        *dark = now;
        ctx.set_theme(preference(now));
        // 自绘标题栏也从当前 egui Theme 取色，和正文在同一帧切换。
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(if now {
            egui::SystemTheme::Dark
        } else {
            egui::SystemTheme::Light
        }));
    }
}

fn apply_visuals(ctx: &egui::Context, scheme: ColorScheme) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::new("color-scheme"), scheme));
    ctx.set_visuals_of(
        egui::Theme::Light,
        visuals(egui::Visuals::light(), false, scheme),
    );
    ctx.set_visuals_of(
        egui::Theme::Dark,
        visuals(egui::Visuals::dark(), true, scheme),
    );
}

fn preference(dark: bool) -> egui::ThemePreference {
    if dark {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    }
}

pub(crate) fn palette_for(scheme: ColorScheme, dark: bool) -> Palette {
    match (scheme, dark) {
        (ColorScheme::Cream, false) => Palette::cream_light(),
        (ColorScheme::Cream, true) => Palette::cream_dark(),
        (ColorScheme::Zizai, false) => Palette::zizai_light(),
        (ColorScheme::Zizai, true) => Palette::zizai_dark(),
        (ColorScheme::Latte, false) => Palette::latte_light(),
        (ColorScheme::Latte, true) => Palette::latte_dark(),
        (ColorScheme::Forest, false) => Palette::forest_light(),
        (ColorScheme::Forest, true) => Palette::forest_dark(),
    }
}

fn current_scheme(ctx: &egui::Context) -> ColorScheme {
    ctx.data(|data| {
        data.get_temp::<ColorScheme>(egui::Id::new("color-scheme"))
            .unwrap_or_default()
    })
}

pub(crate) fn current_palette(ctx: &egui::Context) -> Palette {
    palette_for(current_scheme(ctx), dark_mode(ctx))
}

fn color32(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

fn visuals(base: egui::Visuals, dark: bool, scheme: ColorScheme) -> egui::Visuals {
    let colors = palette_for(scheme, dark);
    let mut visuals = base;
    visuals.panel_fill = color32(colors.background);
    visuals.window_fill = color32(colors.background);
    visuals.extreme_bg_color = color32(colors.background);
    visuals.override_text_color = Some(color32(colors.text));
    visuals.selection.bg_fill = color32(colors.highlight);
    visuals.selection.stroke = egui::Stroke::new(1.0, color32(colors.accent));
    visuals.hyperlink_color = color32(colors.accent);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(CARD_RADIUS as u8);
    // 下拉框与按钮：白卡上再放一层更浅的底 + 1 px 描边，像 Fluent 的控件，而不是 egui 缺省的灰块。
    let control = color32(colors.background).gamma_multiply(if dark { 1.22 } else { 1.035 });
    let border = color32(colors.pos).gamma_multiply(if dark { 0.72 } else { 0.58 });
    let border_strong = color32(colors.accent).gamma_multiply(if dark { 0.9 } else { 0.78 });
    for (widget, fill, stroke) in [
        (&mut visuals.widgets.inactive, control, border),
        (
            &mut visuals.widgets.hovered,
            control.gamma_multiply(if dark { 1.18 } else { 0.97 }),
            border_strong,
        ),
        (
            &mut visuals.widgets.active,
            control.gamma_multiply(if dark { 1.3 } else { 0.94 }),
            border_strong,
        ),
    ] {
        widget.corner_radius = egui::CornerRadius::same(CONTROL_RADIUS);
        widget.weak_bg_fill = fill;
        widget.bg_fill = fill;
        widget.bg_stroke = egui::Stroke::new(1.0, stroke);
    }
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(CONTROL_RADIUS);
    visuals.widgets.open.weak_bg_fill = control;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, color32(colors.accent));
    visuals
}

fn dark_mode(ctx: &egui::Context) -> bool {
    ctx.style().visuals.dark_mode
}

/// 说明小字的颜色（WinUI 版用的是正文色 + 0.65 不透明度）。
pub(crate) fn note_color(ctx: &egui::Context) -> egui::Color32 {
    let colors = current_palette(ctx);
    color32(colors.gloss).gamma_multiply(NOTE_OPACITY)
}

/// 导航选中项的底色。
pub(crate) fn selected_fill(ctx: &egui::Context) -> egui::Color32 {
    let colors = current_palette(ctx);
    color32(colors.highlight)
}

/// 导航项鼠标悬停时的底色。
pub(crate) fn hover_fill(ctx: &egui::Context) -> egui::Color32 {
    selected_fill(ctx).gamma_multiply(0.5)
}

/// 图标列的颜色：比正文淡一档，让标签自己突出来。
pub(crate) fn icon_color(ctx: &egui::Context) -> egui::Color32 {
    color32(current_palette(ctx).gloss)
}

/// 行与行之间那条淡线。
pub(crate) fn separator_color(ctx: &egui::Context) -> egui::Color32 {
    color32(current_palette(ctx).pos).gamma_multiply(if dark_mode(ctx) { 0.52 } else { 0.34 })
}

/// 强调色（品牌蓝），开关打开时用。
pub(crate) fn accent(ctx: &egui::Context) -> egui::Color32 {
    color32(current_palette(ctx).accent)
}

/// 左侧导航栏的底：比正文区略深一档，和 WinUI 的 NavigationView 分区一致。
pub(crate) fn nav_frame(ctx: &egui::Context) -> egui::Frame {
    let dark = dark_mode(ctx);
    let fill = if dark {
        color32(current_palette(ctx).background).gamma_multiply(0.82)
    } else {
        color32(current_palette(ctx).background).gamma_multiply(0.97)
    };
    egui::Frame::NONE
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(10, 14))
}

/// 正文区的底。
pub(crate) fn page_frame(ctx: &egui::Context) -> egui::Frame {
    egui::Frame::NONE
        .fill(color32(current_palette(ctx).background))
        .inner_margin(egui::Margin::symmetric(20, 16))
}

/// 一张设置卡片的外框：底色 + 1 px 描边 + 圆角，取值与 WinUI 版 `controls/mod.rs` 相同。
pub(crate) fn card_frame(ctx: &egui::Context) -> egui::Frame {
    let dark = dark_mode(ctx);
    let colors = current_palette(ctx);
    let fill = color32(colors.background).gamma_multiply(if dark { 1.18 } else { 1.04 });
    let stroke = color32(colors.pos).gamma_multiply(if dark { 0.58 } else { 0.42 });
    egui::Frame::NONE
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(CARD_RADIUS)
        .inner_margin(egui::Margin::symmetric(
            CARD_PADDING_X as i8,
            CARD_PADDING_Y as i8,
        ))
}

/// 自绘标题栏与导航栏用同一层底色。
pub(crate) fn title_bar_frame(ctx: &egui::Context) -> egui::Frame {
    egui::Frame::NONE
        .fill(nav_frame(ctx).fill)
        .inner_margin(egui::Margin::symmetric(10, 0))
}
