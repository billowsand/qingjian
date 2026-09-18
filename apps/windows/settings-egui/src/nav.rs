//! 左侧导航栏：品牌头（Logo + 名字 + 品牌句）+ 五个分节。
//! WinUI 版这块是 `NavigationView` 一个控件，immediate mode 下要自己拼，也因此能完全照设计稿排。

use eframe::egui;

use crate::app::{PAGES, Settings};
use crate::fonts;
use crate::theme;
use crate::widgets::{BRAND_SIZE, LABEL_SIZE, LOGO_SIZE, NOTE_SIZE};

/// Logo 的原始遮罩编进二进制；显示时按当前色系与系统明暗实时换色。
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
        fonts::icon_font(16.0),
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

/// 以主图标为几何遮罩生成当前主题的 Logo，并按「色系 × 明暗」缓存；「关于」页也用它。
pub(crate) fn logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let scheme = theme::current_scheme(ctx);
    let dark = theme::dark_mode(ctx);
    let id = egui::Id::new(("brand-logo", scheme.key(), dark));
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
    let size = [info.width as usize, info.height as usize];
    let pixels = themed_logo_pixels(
        &buffer[..info.buffer_size()],
        size[0],
        size[1],
        theme::current_palette(ctx),
        dark,
    );
    let image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    let handle = ctx.load_texture(
        format!(
            "brand-logo-{}-{}",
            scheme.key(),
            if dark { "dark" } else { "light" }
        ),
        image,
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|data| data.insert_temp(id, handle.clone()));
    Some(handle)
}

fn themed_logo_pixels(
    source: &[u8],
    width: usize,
    height: usize,
    palette: qingjian_render::Palette,
    dark: bool,
) -> Vec<u8> {
    let mut output = Vec::with_capacity(source.len());
    let warm_white = [247, 246, 242];
    let mint = [85, 214, 194];
    let blue_top = [49, 87, 216];
    let blue_bottom = [36, 79, 219];
    for (index, pixel) in source.chunks_exact(4).enumerate() {
        let x = index % width;
        let y = index / width;
        let source_rgb = [pixel[0], pixel[1], pixel[2]];
        let semantic = [warm_white, mint, blue_top, blue_bottom]
            .into_iter()
            .min_by_key(|candidate| color_distance(source_rgb, *candidate))
            .unwrap_or(blue_top);
        let color = if semantic == warm_white {
            warm_white
        } else if semantic == mint {
            [palette.caret.r, palette.caret.g, palette.caret.b]
        } else if dark && border_pixel(x, y, width, height) {
            [palette.accent.r, palette.accent.g, palette.accent.b]
        } else {
            let top = if dark {
                mix_rgb(palette.highlight, palette.accent, 0.16)
            } else {
                [palette.accent.r, palette.accent.g, palette.accent.b]
            };
            let bottom = if dark {
                [
                    palette.highlight.r,
                    palette.highlight.g,
                    palette.highlight.b,
                ]
            } else {
                mix_rgb(palette.accent, palette.background, 0.12)
            };
            mix_array(top, bottom, y as f32 / height.max(1) as f32)
        };
        output.extend_from_slice(&[color[0], color[1], color[2], pixel[3]]);
    }
    output
}

fn color_distance(left: [u8; 3], right: [u8; 3]) -> u32 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| {
            let delta = i32::from(left) - i32::from(right);
            (delta * delta) as u32
        })
        .sum()
}

fn mix_rgb(left: qingjian_render::Color, right: qingjian_render::Color, amount: f32) -> [u8; 3] {
    mix_array(
        [left.r, left.g, left.b],
        [right.r, right.g, right.b],
        amount,
    )
}

fn mix_array(left: [u8; 3], right: [u8; 3], amount: f32) -> [u8; 3] {
    let amount = amount.clamp(0.0, 1.0);
    std::array::from_fn(|index| {
        (f32::from(left[index]) * (1.0 - amount) + f32::from(right[index]) * amount).round() as u8
    })
}

fn border_pixel(x: usize, y: usize, width: usize, height: usize) -> bool {
    let scale_x = width as f32 / 1024.0;
    let scale_y = height as f32 / 1024.0;
    let x = x as f32 / scale_x;
    let y = y as f32 / scale_y;
    rounded_rect_contains(x, y, 64.0, 64.0, 960.0, 960.0, 208.0)
        && !rounded_rect_contains(x, y, 120.0, 120.0, 904.0, 904.0, 152.0)
}

fn rounded_rect_contains(
    x: f32,
    y: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    radius: f32,
) -> bool {
    if x < left || x > right || y < top || y > bottom {
        return false;
    }
    let nearest_x = x.clamp(left + radius, right - radius);
    let nearest_y = y.clamp(top + radius, bottom - radius);
    let dx = x - nearest_x;
    let dy = y - nearest_y;
    dx * dx + dy * dy <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::{border_pixel, color_distance, mix_array};

    #[test]
    fn dark_logo_border_follows_the_original_rounded_square() {
        assert!(border_pixel(72, 512, 1024, 1024));
        assert!(border_pixel(112, 512, 1024, 1024));
        assert!(!border_pixel(140, 512, 1024, 1024));
        assert!(!border_pixel(10, 10, 1024, 1024));
    }

    #[test]
    fn source_colors_map_to_their_semantic_roles() {
        assert_eq!(color_distance([247, 246, 242], [247, 246, 242]), 0);
        assert_eq!(mix_array([0, 100, 200], [100, 200, 0], 0.5), [50, 150, 100]);
    }
}
