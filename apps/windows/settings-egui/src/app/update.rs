//! 每帧：左侧导航 + 右侧当前分节页；首帧画完报一次启动耗时与 wgpu 选中的适配器（spike 要量的两项）。

use eframe::egui;

use super::Settings;
use crate::pages;
use crate::widgets::NAV_WIDTH;

impl eframe::App for Settings {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        crate::theme::follow_system(ctx, &mut self.dark);

        egui::SidePanel::left("nav")
            .exact_width(NAV_WIDTH)
            .resizable(false)
            .frame(crate::theme::nav_frame(ctx))
            .show(ctx, |ui| crate::nav::view(self, ui));

        egui::CentralPanel::default()
            .frame(crate::theme::page_frame(ctx))
            .show(ctx, |ui| match self.page() {
                "appearance" => pages::appearance::view(self, ui),
                "dictionaries" => pages::dictionaries::view(self, ui),
                "advanced" => pages::advanced::view(self, ui),
                "about" => pages::about::view(self, ui),
                _ => pages::general::view(self, ui),
            });

        self.report_first_frame(frame);
    }
}

impl Settings {
    /// 当前分节 tag。
    pub(crate) fn page(&self) -> &str {
        &self.page
    }

    pub(crate) fn select_page(&mut self, tag: &str) {
        self.page = tag.to_owned();
    }

    /// 首帧一次性输出：进程启动到第一帧画完的墙钟时间，以及 wgpu 实际选中的适配器
    /// （无独显 / RDP 下应当看到 "Microsoft Basic Render Driver"，即 D3D12 的 WARP 软件适配器）。
    fn report_first_frame(&mut self, frame: &eframe::Frame) {
        if self.reported {
            return;
        }
        self.reported = true;
        println!(
            "首帧耗时 {:.0} ms",
            self.started.elapsed().as_secs_f64() * 1000.0
        );
        if let Some(state) = frame.wgpu_render_state() {
            let info = state.adapter.get_info();
            println!(
                "wgpu 适配器 {} / {:?} / {:?}",
                info.name, info.device_type, info.backend
            );
        }
    }
}
