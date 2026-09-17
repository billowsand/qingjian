//! 左侧导航栏：品牌头（Logo + 名字 + 品牌句）+ 五个分节。
//! WinUI 版这块是 `NavigationView` 一个控件，immediate mode 下要自己拼，也因此能完全照设计稿排。

use eframe::egui;

use crate::app::{PAGES, Settings};
use crate::theme;
use crate::widgets::{BRAND_SIZE, LABEL_SIZE, LOGO_SIZE, NOTE_SIZE};

/// Logo 编进二进制，不依赖随包文件（与 WinUI 版同一张图）。
const LOGO: &[u8] = include_bytes!("../../../../assets/icon/logo.png");

pub(crate) fn view(settings: &mut Settings, ui: &mut egui::Ui) {
    brand(ui);
    ui.add_space(12.0);
    let current = settings.page().to_owned();
    for (tag, label, icon) in PAGES {
        if item(ui, icon, label, current == tag).clicked() {
            settings.select_page(tag);
        }
    }
}

/// 一个导航项：选中的铺一层强调底色。
///
/// 图标与文字是同一段按钮文本，`widget_info` 要重报一次名字——否则 UIA 树里这个 Button 的 Name
/// 连图标那个私用区码点一起念（读屏会念出「方块 通用」）。
fn item(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> egui::Response {
    let text = egui::RichText::new(format!("{icon}  {label}")).size(LABEL_SIZE);
    let button = egui::Button::new(text)
        .frame(true)
        .fill(if selected {
            theme::selected_fill(ui.ctx())
        } else {
            egui::Color32::TRANSPARENT
        })
        .stroke(egui::Stroke::NONE)
        .min_size(egui::vec2(ui.available_width(), 36.0));
    let response = ui.add(button);
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, label)
    });
    response
}

/// 品牌头：Logo + 「字在」+ 「更自在的输入」，层级与安装器、设计稿一致。
fn brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        if let Some(texture) = logo(ui.ctx()) {
            ui.add(egui::Image::new(&texture).fit_to_exact_size(egui::vec2(LOGO_SIZE, LOGO_SIZE)));
        }
        ui.add_space(8.0);
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

/// 解码一次 Logo 存进 egui 的纹理缓存（`png` 解码，不引 egui_extras 的图片加载器）。
fn logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
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
