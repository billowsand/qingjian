//! 悬浮状态条：桌面上常驻、可拖动的四格浮窗 `[Logo][中 / A][，。/ ,.][⚙]`，由字在渲染器绘制。
//!
//! 按下鼠标先 `DragDetect`：挪出拖动阈值就交给系统的移动循环（`WM_NCLBUTTONDOWN` + `HTCAPTION`），
//! 结束时 `WM_EXITSIZEMOVE` 报新位置；没挪就是点击，按 x 落进哪格。`WM_MOUSEACTIVATE` 回 `MA_NOACTIVATE` 不抢焦点。
//! 一格的规格在 [`cell`]，摆放与点击在 [`placement`]。

mod placement;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::HiDpi::{GetDpiForSystem, GetDpiForWindow};
use windows::Win32::UI::Input::KeyboardAndMouse::{DragDetect, ReleaseCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, HTCAPTION, HTCLIENT, IDC_HAND,
    LoadCursorW, MA_NOACTIVATE, SW_HIDE, SW_SHOWNA, SendMessageW, ShowWindow, WM_EXITSIZEMOVE,
    WM_LBUTTONDOWN, WM_MOUSEACTIVATE, WM_NCHITTEST, WM_NCLBUTTONDOWN, WNDCLASSEXW, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};
use windows::core::{PCWSTR, Result, w};

use qingjian_render::{StatusCell, shuangpin_mark};

use self::placement::{Placement, StatusAction};
use super::StatusEvents;
use super::candidates::system_prefers_dark;
use super::layered;
use super::monitor;
use super::painter::SharedPainter;
use super::window_class::WindowClass;
use crate::dispatch::StatusView;

const CLASS_NAME: PCWSTR = w!("QingjianStatusBar");
static CLASS: WindowClass = WindowClass::new();

/// 状态条与屏幕边缘的间隙（逻辑像素）。
const EDGE_GAP: i32 = 8;

thread_local! {
    /// 本线程活着的状态条：HWND → 摆放状态。窗口过程按 HWND 查，查不到（已析构）就忽略。
    static PLACEMENTS: RefCell<HashMap<isize, Rc<Placement>>> = RefCell::new(HashMap::new());
}

/// 悬浮状态条窗口。
pub(super) struct StatusBar {
    hwnd: HWND,

    /// 最近一次要显示的内容；还没显示过时为 `None`。
    data: RefCell<Option<StatusView>>,

    /// 上次用的 DPI。
    dpi: Cell<u32>,

    /// 上次解析出的深浅。
    dark: Cell<bool>,

    /// 摆放状态，与窗口过程共享。
    placement: Rc<Placement>,

    /// 字在渲染器。
    painter: SharedPainter,
}

/// 四格从左到右的动作。
const ACTIONS: [StatusAction; 4] = [
    StatusAction::Drag,
    StatusAction::ToggleMode,
    StatusAction::TogglePunctuation,
    StatusAction::OpenSettings,
];

impl StatusBar {
    /// 建一个隐藏的状态条窗口。
    pub(super) fn new(events: StatusEvents, painter: SharedPainter) -> Result<Self> {
        CLASS.ensure(|| WNDCLASSEXW {
            lpfnWndProc: Some(wndproc),
            hInstance: super::module_handle(),
            hCursor: unsafe { LoadCursorW(None, IDC_HAND) }.unwrap_or_default(),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        })?;
        let dpi = unsafe { GetDpiForSystem() }.max(96);
        let dark = system_prefers_dark();
        // NOACTIVATE：显示时不抢应用焦点。
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                CLASS_NAME,
                w!("字在状态条"),
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
        let placement = Rc::new(Placement::new(hwnd, 0, events));
        PLACEMENTS.with(|map| map.borrow_mut().insert(hwnd.0 as isize, placement.clone()));
        Ok(Self {
            hwnd,
            data: RefCell::new(None),
            dpi: Cell::new(dpi),
            dark: Cell::new(dark),
            placement,
            painter,
        })
    }

    /// 显示 / 更新：按记住的位置（首次用 `view.anchor`，都没有就右下角）摆放并重绘。
    pub(super) fn update(&self, view: StatusView) {
        if self.placement.pos.get().is_none() {
            self.placement.pos.set(view.anchor);
        }
        *self.data.borrow_mut() = Some(view);
        self.sync_environment();
        self.render();
    }

    pub(super) fn hide(&self) {
        let _ = unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    /// 同步 DPI 与深浅模式。
    fn sync_environment(&self) {
        let dpi = match unsafe { GetDpiForWindow(self.hwnd) } {
            0 => self.dpi.get(),
            dpi => dpi,
        };
        let dark = system_prefers_dark();
        self.dpi.set(dpi);
        self.dark.set(dark);
    }

    /// 渲染器要的四格：拖拽 Logo、普通模式文字、标点（生效时品牌色，否则灰）、设置齿轮。
    fn status_cells(view: &StatusView) -> Vec<StatusCell> {
        let scheme = if view.english {
            None
        } else {
            view.scheme.as_deref().and_then(shuangpin_mark)
        };
        vec![
            StatusCell::Logo,
            StatusCell::mode(if view.english { "A" } else { "中" }, scheme),
            StatusCell::text(if view.full_width { "，。" } else { ",." }, view.full_width),
            StatusCell::Gear,
        ]
    }

    /// 画好贴上并显示；顺带记下各格边界给点击用。
    fn render(&self) {
        let rendered = {
            let data = self.data.borrow();
            let mut painter = self.painter.borrow_mut();
            match data.as_ref() {
                Some(view) => painter.render_status(
                    &Self::status_cells(view),
                    view.color_scheme,
                    self.dark.get(),
                    self.dpi.get(),
                ),
                None => None,
            }
        };
        let Some(rendered) = rendered else {
            self.hide();
            return;
        };
        let bitmap = &rendered.rendered;
        let content = (bitmap.content_width as i32, bitmap.content_height as i32);
        if content.0 <= 0 || content.1 <= 0 {
            self.hide();
            return;
        }
        let margin = bitmap.content_x as i32;
        self.placement.margin.set(margin);
        *self.placement.cells.borrow_mut() = rendered
            .cell_edges
            .iter()
            .zip(ACTIONS)
            .map(|(edge, action)| (edge.round() as i32, action))
            .collect();
        let anchor = self.anchor(content, margin);
        let updated = layered::present(
            self.hwnd,
            &bitmap.pixmap,
            (anchor.0 - margin, anchor.1 - margin),
        );
        if updated.is_ok() {
            super::raise_topmost(self.hwnd);
            let _ = unsafe { ShowWindow(self.hwnd, SW_SHOWNA) };
        } else {
            self.hide();
        }
    }

    /// 内容左上角：记住的位置，没有就右下角，再夹进工作区；顺带记下。
    fn anchor(&self, content: (i32, i32), margin: i32) -> (i32, i32) {
        let anchor = self
            .placement
            .pos
            .get()
            .unwrap_or_else(|| default_anchor(content, margin));
        let anchor = clamp_anchor(anchor, content, margin);
        self.placement.pos.set(Some(anchor));
        anchor
    }
}

impl Drop for StatusBar {
    fn drop(&mut self) {
        PLACEMENTS.with(|map| map.borrow_mut().remove(&(self.hwnd.0 as isize)));
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

/// 首次出现的位置：主显示器工作区右下角，留出边距与阴影。
fn default_anchor(content: (i32, i32), margin: i32) -> (i32, i32) {
    let work = monitor::primary_work_area();
    let gap = ((EDGE_GAP * margin) / 16).max(EDGE_GAP);
    (
        work.right - margin - gap - content.0,
        work.bottom - margin - gap - content.1,
    )
}

/// 把内容左上角夹进所在显示器的工作区，使整块内容可见。
fn clamp_anchor(anchor: (i32, i32), content: (i32, i32), margin: i32) -> (i32, i32) {
    let work = monitor::work_area_near(POINT {
        x: anchor.0,
        y: anchor.1,
    });
    let x = anchor.0.clamp(
        work.left + margin,
        (work.right - margin - content.0).max(work.left + margin),
    );
    let y = anchor.1.clamp(
        work.top + margin,
        (work.bottom - margin - content.1).max(work.top + margin),
    );
    (x, y)
}

fn placement_of(hwnd: HWND) -> Option<Rc<Placement>> {
    // clone 出来放开借用，再调回调。
    PLACEMENTS.with(|map| map.borrow().get(&(hwnd.0 as isize)).cloned())
}

/// 按下：拖动交给系统移动循环，没拖就是点击；点击不激活；拖动结束报位置。
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_NCHITTEST => LRESULT(HTCLIENT as isize),
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_LBUTTONDOWN => {
            if let Some(placement) = placement_of(hwnd) {
                // lparam 低 16 位是客户区 x（有符号）。只有左侧 Logo 能拖，其他格直接执行点击。
                let client_x = (lparam.0 & 0xFFFF) as i16 as i32;
                if placement.action_at(client_x) == Some(StatusAction::Drag) {
                    let mut point = POINT::default();
                    let _ = unsafe { GetCursorPos(&mut point) };
                    if unsafe { DragDetect(hwnd, point) }.as_bool() {
                        let _ = unsafe { ReleaseCapture() };
                        unsafe {
                            SendMessageW(
                                hwnd,
                                WM_NCLBUTTONDOWN,
                                Some(WPARAM(HTCAPTION as usize)),
                                Some(LPARAM(0)),
                            )
                        };
                    }
                } else {
                    placement.on_click(client_x);
                }
            }
            LRESULT(0)
        }
        WM_EXITSIZEMOVE => {
            if let Some(placement) = placement_of(hwnd) {
                placement.on_moved();
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(test)]
mod tests {
    use qingjian_platform::ColorScheme;
    use qingjian_render::StatusCell;

    use super::{StatusBar, StatusView};

    fn view(english: bool, scheme: Option<&str>) -> StatusView {
        StatusView {
            english,
            scheme: scheme.map(str::to_owned),
            full_width: false,
            color_scheme: ColorScheme::Zizai,
            anchor: None,
        }
    }

    #[test]
    fn shuangpin_uses_compact_scheme_marks() {
        let xiaohe = view(false, Some("xiaohe"));
        assert_eq!(
            StatusBar::status_cells(&xiaohe)[1],
            StatusCell::mode("中", Some("鹤"))
        );

        let english = view(true, Some("xiaohe"));
        assert_eq!(
            StatusBar::status_cells(&english)[1],
            StatusCell::mode("A", None::<&str>)
        );
    }
}
