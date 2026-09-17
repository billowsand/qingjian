//! 开关控件：egui 没有 WinUI 的 `ToggleSwitch`，自己画一个（官方 demo 的做法）。
//! 关键是 `widget_info` 报成 Checkbox——accesskit 据此在 UIA 树里给出可读的「选中 / 未选中」状态。

use eframe::egui;

/// 开关宽高比：宽是高的两倍，与 Fluent 的 ToggleSwitch 接近。
const ASPECT: f32 = 2.0;

/// 滑块与外框的间距。
const KNOB_INSET: f32 = 2.0;

/// `label` 只给无障碍用：`widget_info` 里不给名字的话，UIA 树里这个 CheckBox 的 Name 是空的，
/// 读屏只念得出「已选中」念不出是哪一项（`labelled_by` 也盖不住它）。
pub(crate) fn toggle(ui: &mut egui::Ui, on: &mut bool, label: &str) -> egui::Response {
    let height = ui.spacing().interact_size.y;
    let size = egui::vec2(height * ASPECT, height);
    let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, label)
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let radius = rect.height() / 2.0;
        let fill = if *on {
            crate::theme::accent(ui.ctx())
        } else {
            visuals.bg_fill
        };
        ui.painter().rect(
            rect,
            radius,
            fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let knob_radius = radius - KNOB_INSET;
        let center_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        ui.painter().circle(
            egui::pos2(center_x, rect.center().y),
            knob_radius,
            egui::Color32::WHITE,
            egui::Stroke::NONE,
        );
    }
    response
}
