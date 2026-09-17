//! 左侧导航栏：品牌头（Logo + 名字 + 品牌句）+ 五个分节。
//! WinUI 版这块是 `NavigationView` 一个控件，immediate mode 下要自己拼，也因此能完全照设计稿排。

use eframe::egui;

use crate::app::{PAGES, Settings};
use crate::theme;
use crate::widgets::{BRAND_SIZE, LABEL_SIZE, LOGO_SIZE, NOTE_SIZE};

/// Logo 编进二进制，不依赖随包文件（与 WinUI 版同一张图）。
const LOGO: &[u8] = include_bytes!("../../../../assets/icon/logo.png");

/// 导航项行高。
const ITEM_HEIGHT: f32 = 34.0;

/// 图标中心距行左边缘。
const ICON_INSET: f32 = 16.0;

/// 文字左边缘距行左边缘。
const LABEL_INSET: f32 = 30.0;

pub(crate) fn view(settings: &mut Settings, ui: &mut egui::Ui) {
    brand(ui);
    ui.add_space(14.0);
    let current = settings.page().to_owned();
    for (tag, label, icon) in PAGES {
        if item(ui, icon, label, current == tag).clicked() {
            settings.select_page(tag);
        }
    }
}

/// 一个导航项：整行可点，选中的铺一层强调底色、文字用强调色。
///
/// 图标由 painter 按字形外框居中画（与列表行同一套做法，不跟字体基线走），文字单独 `label`——
/// 这样图标和文字在视觉上真正对齐，`widget_info` 里报的名字也不带那个私用区码点。
fn item(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, ITEM_HEIGHT), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, label)
    });
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 6.0, theme::selected_fill(ui.ctx()));
    } else if response.hovered() {
        painter.rect_filled(rect, 6.0, theme::hover_fill(ui.ctx()));
    }
    let tone = if selected {
        theme::accent(ui.ctx())
    } else {
        ui.style().visuals.text_color()
    };
    painter.text(
        egui::pos2(rect.left() + ICON_INSET, rect.center().y + 1.0),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(16.0),
        if selected {
            tone
        } else {
            theme::icon_color(ui.ctx())
        },
    );
    painter.text(
        egui::pos2(rect.left() + LABEL_INSET, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(LABEL_SIZE),
        tone,
    );
    response
}

/// 品牌头：Logo + 「字在」+ 「更自在的输入」，层级与安装器、设计稿一致。
fn brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_space(2.0);
        if let Some(texture) = logo(ui.ctx()) {
            ui.add(egui::Image::new(&texture).fit_to_exact_size(egui::vec2(LOGO_SIZE, LOGO_SIZE)));
        }
        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("字在").size(BRAND_SIZE).strong());
            ui.label(
                egui::RichText::new("更自在的输入")
                    .size(NOTE_SIZE)
                    .color(theme::note_color(ui.ctx())),
            );
        });
    });
}

/// 解码一次 Logo 存进 egui 的纹理缓存（`png` 解码，不引 egui_extras 的图片加载器）；「关于」页也用它。
pub(crate) fn logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::new("brand-logo");
    if let Some(handle) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
        return Some(handle);
    }
    let decoder = png::Decoder::new(std::io::Cursor::new(LOGO));
    let mut reader = decoder.read_info().ok()?;
    let mut buffer = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buffer).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [info.width as usize, info.height as usize],
        &buffer[..info.buffer_size()],
    );
    let handle = ctx.load_texture("brand-logo", image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|data| data.insert_temp(id, handle.clone()));
    Some(handle)
}
