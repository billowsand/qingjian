//! 候选窗口：不抢焦点、置顶的分层窗口，跟随光标，画拼音行与候选列表，四周柔和阴影。
//! 字在渲染器出位图后由 Windows 壳贴上。绘制内容在 [`RenderData`]，一行的展示形态在 [`row`]，
//! 贴光标上方还是下方在 [`placement`]。设计语言对齐 macOS 端。

mod placement;
mod render_data;
pub(crate) mod row;

use std::cell::{Cell, RefCell};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::HiDpi::{GetDpiForSystem, GetDpiForWindow};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, IDC_ARROW, LoadCursorW, SW_HIDE, SW_SHOWNA,
    ShowWindow, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP,
};
use windows::core::{PCWSTR, Result, w};

use qingjian_platform::ThemeMode;
use qingjian_platform::protocol::Frame;

use self::placement::{LastPlacement, place};
pub(crate) use self::render_data::RenderData;
use super::layered;
use super::painter::SharedPainter;
use super::window_class::WindowClass;

const CLASS_NAME: PCWSTR = w!("QingjianCandidateWindow");
static CLASS: WindowClass = WindowClass::new();

/// 按外观模式解析深浅；`System` 读系统主题。
pub(super) fn resolve_dark(mode: ThemeMode) -> bool {
    match mode {
        ThemeMode::Light => false,
        ThemeMode::Dark => true,
        ThemeMode::System => system_prefers_dark(),
    }
}

/// `HKCU\...\Themes\Personalize\AppsUseLightTheme` 为 0 是深色；读不到当浅色。
fn system_prefers_dark() -> bool {
    windows_registry::CURRENT_USER
        .open(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|key| key.get_u32("AppsUseLightTheme"))
        .is_ok_and(|value| value == 0)
}

/// 候选窗口。内容经 `UpdateLayeredWindow` 一次贴上，窗口过程只走默认处理。
pub(crate) struct CandidateWindow {
    hwnd: HWND,

    /// 绘制内容。
    data: RefCell<RenderData>,

    /// 上次用的 DPI。
    dpi: Cell<u32>,

    /// 上次解析出的深浅。
    dark: Cell<bool>,

    /// 上次贴在光标的哪一边；同一行里不因窗口高矮改边。
    placement: LastPlacement,

    /// 字在渲染器。
    painter: SharedPainter,
}

impl CandidateWindow {
    /// 建一个隐藏的候选窗口。
    pub(crate) fn new(painter: SharedPainter) -> Result<Self> {
        CLASS.ensure(|| WNDCLASSEXW {
            lpfnWndProc: Some(wndproc),
            hInstance: super::module_handle(),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap_or_default(),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        })?;
        let dpi = unsafe { GetDpiForSystem() }.max(96);
        let dark = resolve_dark(ThemeMode::default());
        let data = RefCell::new(RenderData::empty());
        // NOACTIVATE：显示时不抢应用焦点。
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                CLASS_NAME,
                w!("字在候选"),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(super::module_handle()),
                None,
            )?
        };
        Ok(Self {
            hwnd,
            data,
            dpi: Cell::new(dpi),
            dark: Cell::new(dark),
            placement: Cell::new(None),
            painter,
        })
    }

    /// 刷新内容（不定位、不显示）。
    pub(crate) fn set_content(&self, frame: &Frame) {
        self.data.borrow_mut().set(frame);
    }

    /// 按光标矩形定位并显示：贴光标下方（放不下放上方，同一行里不改边），四周留出阴影。
    pub(crate) fn show(&self, anchor: RECT) {
        self.sync_environment();
        let rendered = {
            let data = self.data.borrow();
            self.painter.borrow_mut().render_frame(
                &data.render_frame(),
                self.dark.get(),
                self.dpi.get(),
            )
        };
        let Some(rendered) = rendered else {
            self.hide();
            return;
        };
        let content = (
            rendered.content_width as i32,
            rendered.content_height as i32,
        );
        if content.0 <= 0 || content.1 <= 0 {
            self.hide();
            return;
        }
        let (content_x, content_y) = place(&self.placement, anchor, content);
        let updated = layered::present(
            self.hwnd,
            &rendered.pixmap,
            (
                content_x - rendered.content_x as i32,
                content_y - rendered.content_y as i32,
            ),
        );
        if updated.is_ok() {
            super::raise_topmost(self.hwnd);
            let _ = unsafe { ShowWindow(self.hwnd, SW_SHOWNA) };
        } else {
            self.hide();
        }
    }

    pub(crate) fn hide(&self) {
        let _ = unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    /// 每次 `show` 前同步 DPI 与深浅模式。
    fn sync_environment(&self) {
        let dpi = match unsafe { GetDpiForWindow(self.hwnd) } {
            0 => self.dpi.get(),
            dpi => dpi,
        };
        let dark = resolve_dark(self.data.borrow().theme_mode);
        self.dpi.set(dpi);
        self.dark.set(dark);
    }
}

impl Drop for CandidateWindow {
    fn drop(&mut self) {
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

/// 分层窗口无需 `WM_PAINT`，全交默认处理。
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
