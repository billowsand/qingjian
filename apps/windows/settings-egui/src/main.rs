//! 设置界面 egui spike 的入口：起窗口、装中文字体、跑 eframe。
//! 目的与要量的四件事见同目录 `README.md`；非 Windows 编成空壳，让工作区能整体编译。
//!
//! 与正式设置程序（`apps/windows/settings`，Windows Reactor / WinUI）并存，互不影响：
//! 两边读写同一个 `config.toml`，可以开着比。

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod fonts;
#[cfg(windows)]
mod gpu;
#[cfg(windows)]
mod nav;
#[cfg(windows)]
mod pages;
#[cfg(windows)]
mod theme;
#[cfg(windows)]
mod widgets;

/// 窗口初始大小，与正式设置程序的观感对齐。
#[cfg(windows)]
const WINDOW_SIZE: [f32; 2] = [980.0, 700.0];

/// 窗口最小大小：再小卡片里的「标签 + 控件」两列就挤了。
#[cfg(windows)]
const MIN_WINDOW_SIZE: [f32; 2] = [760.0, 520.0];

/// 控制台子系统（不加 `windows_subsystem = "windows"`）：spike 要看首帧耗时与 wgpu 选了哪个适配器。
#[cfg(windows)]
fn main() -> eframe::Result<()> {
    let started = std::time::Instant::now();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("字在设置（egui spike）")
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE),
        wgpu_options: gpu::configuration(),
        ..Default::default()
    };
    eframe::run_native(
        "字在设置（egui spike）",
        options,
        Box::new(move |cc| Ok(Box::new(app::Settings::new(cc, started)))),
    )
}

#[cfg(not(windows))]
fn main() {
    eprintln!("qingjian-settings-egui 仅支持 Windows");
}
