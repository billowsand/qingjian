//! 各页共用的表单零件：设置卡片（图标 + 标签 + 说明 + 控件）、分组、说明小字、页外壳。
//! 排版取值集中在这里，改一处全局生效；打开文件 / 目录、打包日志的小工具在 [`files`]。

mod files;

use windows_reactor::*;

pub(super) use self::files::{
    export_logs, log_dir, open_in_editor, open_with_explorer, repo_resource,
};

/// 页面大标题字号。
const TITLE_SIZE: f64 = 24.0;

/// 分组标题字号。
const GROUP_SIZE: f64 = 16.0;

/// 设置项标签字号。
const LABEL_SIZE: f64 = 14.0;

/// 说明小字字号，与候选窗口的译文 / 词性同一档（渲染器 `Theme::annotation_font`）。
const NOTE_SIZE: f64 = 12.0;

/// 说明小字的淡化程度。
const NOTE_OPACITY: f64 = 0.65;

/// 卡片圆角，与候选窗口的 `Theme::corner_radius` 一致。
const CARD_RADIUS: f64 = 8.0;

/// 卡片左右内边距。
const CARD_PADDING_X: f64 = 16.0;

/// 卡片上下内边距。
const CARD_PADDING_Y: f64 = 12.0;

/// 卡片里图标、文字、控件三列之间的间距。
const CARD_GAP: f64 = 16.0;

/// 同一组里相邻卡片之间的间距。
const ROW_GAP: f64 = 4.0;

/// 分组之间的间距。
const GROUP_GAP: f64 = 24.0;

/// 控件列的最小宽度，让各行的下拉框 / 开关左边缘对齐。
const CONTROL_WIDTH: f64 = 260.0;

/// 把数量不定的视图排进一个 `StackPanel`：`children` 只收元组与定长数组，动态列表要按位置给键。
fn column(panel: StackPanel, views: Vec<View>) -> View {
    panel.keyed_children(
        views
            .into_iter()
            .enumerate()
            .map(|(index, view)| KeyedView::new(index.to_string(), view)),
    )
}

/// 灰色小字说明，可换行。
pub(super) fn note(text: &str) -> View {
    TextBlock::new()
        .text(text)
        .text_wrapping(TextWrapping::Wrap)
        .font_size(NOTE_SIZE)
        .opacity(NOTE_OPACITY)
        .into()
}

/// 一张设置卡片：跟随系统明暗的底色与描边。
fn card(content: impl Into<View>) -> View {
    Border::new()
        .background(ThemeBrush::CardBackground)
        .border_brush(ThemeBrush::CardStroke)
        .border_thickness(1.0)
        .corner_radius(CARD_RADIUS)
        .padding(Thickness::xy(CARD_PADDING_X, CARD_PADDING_Y))
        .content(content)
}

/// 一整项：卡片里放「图标 + 标签（下接说明）+ 控件」，控件靠右。`hint` 为空则不加说明。
pub(super) fn field(icon: Symbol, label: &str, hint: &str, control: impl Into<View>) -> View {
    let mut text: Vec<View> = Vec::with_capacity(2);
    text.push(
        TextBlock::new()
            .text(label)
            .font_size(LABEL_SIZE)
            .text_wrapping(TextWrapping::Wrap)
            .into(),
    );
    if !hint.is_empty() {
        text.push(note(hint));
    }
    card(
        Grid::new()
            .columns([GridLength::Auto, GridLength::STAR, GridLength::Auto])
            .column_spacing(CARD_GAP)
            .children((
                SymbolIcon::new()
                    .symbol(icon)
                    .grid_column(0)
                    .vertical_alignment(VerticalAlignment::Center),
                column(
                    StackPanel::new()
                        .grid_column(1)
                        .spacing(2.0)
                        .vertical_alignment(VerticalAlignment::Center),
                    text,
                ),
                Border::new()
                    .grid_column(2)
                    .min_width(CONTROL_WIDTH)
                    .vertical_alignment(VerticalAlignment::Center)
                    .horizontal_alignment(HorizontalAlignment::Right)
                    .content(control.into()),
            )),
    )
}

/// 一张只有文字的卡片：没有开关的说明块（词库列表、统计表格这类自带内容的也走它）。
pub(super) fn block(icon: Symbol, title: &str, content: impl Into<View>) -> View {
    card(
        Grid::new()
            .columns([GridLength::Auto, GridLength::STAR])
            .column_spacing(CARD_GAP)
            .children((
                SymbolIcon::new()
                    .symbol(icon)
                    .grid_column(0)
                    .vertical_alignment(VerticalAlignment::Top),
                StackPanel::new().grid_column(1).spacing(8.0).children((
                    TextBlock::new().text(title).font_size(LABEL_SIZE),
                    content.into(),
                )),
            )),
    )
}

/// 一组设置：图标 + 组标题，下面是这一组的卡片。
pub(super) fn group(icon: Symbol, title: &str, rows: impl IntoIterator<Item = View>) -> View {
    let header = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(8.0)
        .children((
            SymbolIcon::new()
                .symbol(icon)
                .vertical_alignment(VerticalAlignment::Center),
            TextBlock::new()
                .text(title)
                .font_size(GROUP_SIZE)
                .font_weight(FontWeight::SEMI_BOLD)
                .vertical_alignment(VerticalAlignment::Center),
        ));
    let cards = column(
        StackPanel::new().spacing(ROW_GAP),
        rows.into_iter().collect(),
    );
    StackPanel::new().spacing(8.0).children((header, cards))
}

/// 在 `(界面名, 配置写法)` 列表里找 `value` 的下标，找不到取 0。
pub(super) fn index_of(options: &[(&str, &str)], value: &str) -> usize {
    options.iter().position(|(_, v)| *v == value).unwrap_or(0)
}

/// 一页外壳：可滚动 + 大标题 + 若干分组。
pub(super) fn page(title: &str, groups: impl IntoIterator<Item = View>) -> View {
    let mut children: Vec<View> = vec![
        TextBlock::new()
            .text(title)
            .font_size(TITLE_SIZE)
            .font_weight(FontWeight::SEMI_BOLD)
            .into(),
    ];
    children.extend(groups);
    children.push(note("●  全部本地处理，不上传任何数据"));
    ScrollViewer::new().content(column(
        StackPanel::new()
            .spacing(GROUP_GAP)
            .margin(Thickness::new(24.0, 16.0, 24.0, 24.0)),
        children,
    ))
}
