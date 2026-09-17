//! 「候选窗口」页：外观、排布、渲染引擎、字体、拼音显示位置、悬浮状态条。

use qingjian_platform::{CandidateRenderer, LayoutMode, PreeditMode, ThemeMode};
use qingjian_render::{Color as RenderColor, Palette};
use windows_reactor::*;

use crate::panel::controls::{field, note, page};
use crate::panel::{Message, Settings};

/// 「系统字体」项的下标：列表第 0 项，对应配置里的空串。
const SYSTEM_FONT: usize = 0;

/// 枚举下拉：按 `label()` 列项，选中 `current`（找不到取 0）。
fn mode_combo<T: PartialEq + Copy>(
    all: &'static [T],
    current: T,
    label: fn(T) -> &'static str,
    callback: Callback<Option<usize>>,
) -> ComboBox {
    ComboBox::new()
        .items_source(all.iter().map(|mode| label(*mode)))
        .selected_index(all.iter().position(|mode| *mode == current).unwrap_or(0))
        .on_selection_changed(callback)
}

/// 字体下拉：第 0 项是「系统字体」，其余是系统里装的字族（DirectWrite 列举，已按名字排序）。
/// 配置里写了、但系统里没装的字族补在最后并选中它，免得下拉框显示的和配置里存的对不上。
/// 返回（列表，选中项）。
fn font_combo(settings: &Settings) -> (Vec<String>, usize) {
    let configured = settings.config.general.font.trim();
    let mut options: Vec<String> = Vec::with_capacity(settings.families.len() + 1);
    options.push("系统字体（默认）".to_owned());
    options.extend(settings.families.iter().cloned());
    let mut selected = settings
        .families
        .iter()
        .position(|family| family.eq_ignore_ascii_case(configured))
        .map(|index| index + 1)
        .unwrap_or(SYSTEM_FONT);
    if selected == SYSTEM_FONT && !configured.is_empty() {
        options.push(configured.to_owned());
        selected = options.len() - 1;
    }
    (options, selected)
}

fn ui_color(color: RenderColor) -> windows_reactor::Color {
    windows_reactor::Color::argb(color.a, color.r, color.g, color.b)
}

fn preview_text(text: &str, size: f64, color: RenderColor) -> TextBlock {
    TextBlock::new()
        .text(text)
        .font_size(size)
        .foreground(ui_color(color))
}

/// 设置项上方的品牌主题预览。这里直接读取渲染器的 [`Palette`]，避免设置页与候选窗各维护一套颜色。
fn preview_card(label: &str, palette: Palette) -> View {
    let preedit = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(2.0)
        .children((
            preview_text("ni'hao", 13.0, palette.text),
            Border::new()
                .width(2.0)
                .height(18.0)
                .background(ui_color(palette.caret)),
        ));
    let selected = Border::new()
        .background(ui_color(palette.highlight))
        .corner_radius(5.0)
        .padding(Thickness::xy(7.0, 4.0))
        .content(
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(8.0)
                .children((
                    Border::new()
                        .width(2.0)
                        .height(20.0)
                        .corner_radius(1.0)
                        .background(ui_color(palette.accent)),
                    preview_text("1", 11.0, palette.index),
                    preview_text("你好", 16.0, palette.text),
                    preview_text("int. hello", 12.0, palette.gloss),
                )),
        );
    let cloud = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(8.0)
        .children((
            preview_text("2", 11.0, palette.index),
            preview_text("☁ 你好吗", 16.0, palette.cloud),
            preview_text("hello?", 12.0, palette.gloss),
        ));
    StackPanel::new().spacing(6.0).children((
        preview_text(label, 12.0, palette.gloss),
        Border::new()
            .min_width(270.0)
            .background(ui_color(palette.background))
            .border_brush(ui_color(palette.highlight))
            .border_thickness(1.0)
            .corner_radius(8.0)
            .padding(10.0)
            .content(
                StackPanel::new()
                    .spacing(7.0)
                    .children((preedit, selected, cloud)),
            ),
    ))
}

fn theme_preview(mode: ThemeMode) -> View {
    let cards = match mode {
        ThemeMode::System => StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(12.0)
            .children((
                preview_card("浅色", Palette::light()),
                preview_card("深色", Palette::dark()),
            )),
        ThemeMode::Light => StackPanel::new().children([preview_card("浅色", Palette::light())]),
        ThemeMode::Dark => StackPanel::new().children([preview_card("深色", Palette::dark())]),
    };
    StackPanel::new().spacing(6.0).children((
        TextBlock::new()
            .text("字在 · 钴蓝薄荷")
            .font_size(16.0)
            .font_weight(FontWeight::SEMI_BOLD),
        note("预览与候选窗、输入光标和云候选共用同一套语义色；“跟随系统”同时展示两种外观。"),
        cards,
    ))
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let g = &settings.config.general;
    let (font_options, font_selected) = font_combo(settings);
    let rows = [
        field(
            "明暗模式",
            "跟随系统会在 Windows 切换浅色或深色后自动使用相应的字在主题。",
            mode_combo(
                &ThemeMode::ALL,
                g.theme,
                ThemeMode::label,
                context.callback(Message::Theme),
            ),
        ),
        field(
            "排布",
            "",
            mode_combo(
                &LayoutMode::ALL,
                g.layout,
                LayoutMode::label,
                context.callback(Message::Layout),
            ),
        ),
        field(
            "渲染引擎",
            "字在渲染器使用统一的品牌主题，并让候选窗口在各平台保持一致。",
            mode_combo(
                &CandidateRenderer::ALL,
                g.renderer,
                CandidateRenderer::label,
                context.callback(Message::Renderer),
            ),
        ),
        field(
            "字体",
            "只对字在渲染器生效；下拉列表是系统里装的字体，选「系统字体」用默认；配置里的字体没装时自动回到系统字体。",
            ComboBox::new()
                .width(260.0)
                .items_source(font_options)
                .selected_index(font_selected)
                .on_selection_changed(context.callback(Message::Font)),
        ),
        field(
            "拼音显示",
            "「只在候选窗口」时正在敲的拼音不显示在应用里，终端或行内拼音不正常的应用可以选它。",
            mode_combo(
                &PreeditMode::ALL,
                g.preedit,
                PreeditMode::label,
                context.callback(Message::Preedit),
            ),
        ),
        field(
            "悬浮状态条",
            "桌面上常驻、可拖动的小条：从左侧点阵处拖动，点「中 / A」切换模式（开着双拼时还显示方案名），点「，。」切全角 / 半角标点，点齿轮打开设置。只在当前输入法是字在时显示，拖到哪下次还在哪。",
            ToggleSwitch::new()
                .is_on(settings.config.status_bar.enabled)
                .on_toggled(context.callback(Message::StatusBar)),
        ),
    ];
    page(
        "候选窗口",
        StackPanel::new().spacing(20.0).children((
            theme_preview(g.theme),
            StackPanel::new().spacing(16.0).children(rows),
        )),
    )
}
