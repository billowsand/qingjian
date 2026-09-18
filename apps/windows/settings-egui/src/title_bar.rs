//! 与当前色系一体的无边框标题栏：拖动、双击最大化以及三个窗口按钮都由 egui 绘制。

use eframe::egui;

use crate::{nav, theme};

const HEIGHT: f32 = 36.0;
const BUTTON_WIDTH: f32 = 44.0;
const ICON_SIZE: f32 = 10.0;

pub(crate) fn view(ctx: &egui::Context) {
    egui::TopBottomPanel::top("title-bar")
        .exact_height(HEIGHT)
        .frame(theme::title_bar_frame(ctx))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let controls_left = rect.right() - BUTTON_WIDTH * 3.0;
            let drag_rect =
                egui::Rect::from_min_max(rect.min, egui::pos2(controls_left, rect.bottom()));
            let drag = ui.interact(
                drag_rect,
                ui.id().with("drag"),
                egui::Sense::click_and_drag(),
            );
            let maximized = ctx.input(|input| input.viewport().maximized.unwrap_or(false));
            if drag.double_clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            } else if drag.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            paint_brand(ui, drag_rect);
            window_button(ui, 2, "最小化", WindowAction::Minimize, maximized);
            window_button(ui, 1, "最大化", WindowAction::Maximize, maximized);
            window_button(ui, 0, "关闭", WindowAction::Close, maximized);

            ui.painter().hline(
                rect.x_range(),
                rect.bottom() - 0.5,
                egui::Stroke::new(1.0, theme::separator_color(ctx)),
            );
        });
}

fn paint_brand(ui: &egui::Ui, rect: egui::Rect) {
    let left = rect.left() + 2.0;
    let center_y = rect.center().y;
    if let Some(texture) = nav::logo(ui.ctx()) {
        let logo =
            egui::Rect::from_center_size(egui::pos2(left + 10.0, center_y), egui::vec2(20.0, 20.0));
        ui.painter().image(
            texture.id(),
            logo,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
    ui.painter().text(
        egui::pos2(left + 28.0, center_y),
        egui::Align2::LEFT_CENTER,
        "字在设置",
        egui::FontId::proportional(13.0),
        ui.visuals().text_color(),
    );
}

#[derive(Clone, Copy)]
enum WindowAction {
    Minimize,
    Maximize,
    Close,
}

fn window_button(
    ui: &mut egui::Ui,
    from_right: usize,
    label: &'static str,
    action: WindowAction,
    maximized: bool,
) {
    let panel = ui.max_rect();
    let right = panel.right() - BUTTON_WIDTH * from_right as f32;
    let rect = egui::Rect::from_min_max(
        egui::pos2(right - BUTTON_WIDTH, panel.top()),
        egui::pos2(right, panel.bottom()),
    );
    let response = ui.interact(rect, ui.id().with(label), egui::Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    if response.hovered() {
        let fill = if matches!(action, WindowAction::Close) {
            egui::Color32::from_rgb(196, 43, 28)
        } else {
            theme::selected_fill(ui.ctx())
        };
        ui.painter().rect_filled(rect, 0.0, fill);
    }
    let color = if response.hovered() && matches!(action, WindowAction::Close) {
        egui::Color32::WHITE
    } else {
        ui.visuals().text_color()
    };
    paint_window_icon(ui.painter(), rect.center(), action, maximized, color);
    if response.clicked() {
        match action {
            WindowAction::Minimize => {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            WindowAction::Maximize => {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }
            WindowAction::Close => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }
}

fn paint_window_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    action: WindowAction,
    maximized: bool,
    color: egui::Color32,
) {
    let stroke = egui::Stroke::new(1.2, color);
    let half = ICON_SIZE / 2.0;
    match action {
        WindowAction::Minimize => {
            painter.hline(
                (center.x - half)..=(center.x + half),
                center.y + half - 1.0,
                stroke,
            );
        }
        WindowAction::Maximize if maximized => {
            let back = egui::Rect::from_min_size(
                center + egui::vec2(-half + 2.0, -half),
                egui::vec2(ICON_SIZE - 2.0, ICON_SIZE - 2.0),
            );
            let front = back.translate(egui::vec2(-2.0, 2.0));
            painter.rect_stroke(back, 0.0, stroke, egui::StrokeKind::Inside);
            painter.rect_filled(front, 0.0, theme::title_bar_frame(painter.ctx()).fill);
            painter.rect_stroke(front, 0.0, stroke, egui::StrokeKind::Inside);
        }
        WindowAction::Maximize => {
            painter.rect_stroke(
                egui::Rect::from_center_size(center, egui::vec2(ICON_SIZE, ICON_SIZE)),
                0.0,
                stroke,
                egui::StrokeKind::Inside,
            );
        }
        WindowAction::Close => {
            painter.line_segment(
                [
                    center + egui::vec2(-half, -half),
                    center + egui::vec2(half, half),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(half, -half),
                    center + egui::vec2(-half, half),
                ],
                stroke,
            );
        }
    }
}
