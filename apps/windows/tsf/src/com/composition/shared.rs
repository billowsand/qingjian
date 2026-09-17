use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows::Win32::UI::TextServices::{ITfComposition, ITfContext};

use qingjian_platform::protocol::ScreenRect;

use crate::com::service::SharedClient;

/// `TextService`、编辑会话、组句 sink、轮询定时器之间共享的组句状态（STA 单线程，`Rc` 传递）。
pub(crate) struct Shared {
    /// 当前活动的组句，跨按键存活。
    composition: RefCell<Option<ITfComposition>>,

    /// Server 上次回的帧非空；决定 `OnTestKeyDown` 要不要吃功能键。
    composing: Cell<bool>,

    /// 本段组句已经探过输入框（私密判定 + 光标前文）；一段只探一次，`composing` 落下就清。
    probed: Cell<bool>,

    /// 最近一次收键的文档上下文；失焦 / 停用回调不带上下文，落定拼音要用它。
    last_context: RefCell<Option<ITfContext>>,

    /// 本段组句上次量到的光标矩形，量不出来时沿用（见 [`anchor_rect`](crate::com::edit::anchor_rect)）。
    last_anchor: Cell<Option<ScreenRect>>,

    /// 组句被应用强行终止过：拼音已成普通文本，但 Server 的缓冲还在，下次说话前先让它清空。
    server_stale: Cell<bool>,

    /// 本线程当前有键盘焦点（`OnSetFocus`）；轮询定时器只在前台时问状态条的切模式请求。
    foreground: Cell<bool>,

    /// 与 `TextService` 共用的引擎客户端；DLL 侧结束组句时要通知 Server 收候选窗口（它无从知晓）。
    client: SharedClient,
}

impl Shared {
    pub(crate) fn new(client: SharedClient) -> Rc<Self> {
        Rc::new(Self {
            composition: RefCell::new(None),
            composing: Cell::new(false),
            probed: Cell::new(false),
            last_context: RefCell::new(None),
            last_anchor: Cell::new(None),
            server_stale: Cell::new(false),
            foreground: Cell::new(false),
            client,
        })
    }

    pub(crate) fn foreground(&self) -> bool {
        self.foreground.get()
    }

    pub(crate) fn set_foreground(&self, value: bool) {
        self.foreground.set(value);
    }

    pub(crate) fn last_context(&self) -> Option<ITfContext> {
        self.last_context.borrow().clone()
    }

    pub(crate) fn set_last_context(&self, context: Option<ITfContext>) {
        *self.last_context.borrow_mut() = context;
    }

    pub(super) fn last_anchor(&self) -> Option<ScreenRect> {
        self.last_anchor.get()
    }

    pub(super) fn set_last_anchor(&self, rect: ScreenRect) {
        self.last_anchor.set(Some(rect));
    }

    /// 忘掉上次量到的光标矩形：这段组句收了，下一段在别处，要重新量。
    pub(super) fn forget_anchor(&self) {
        self.last_anchor.set(None);
    }

    pub(crate) fn take_server_stale(&self) -> bool {
        self.server_stale.replace(false)
    }

    pub(crate) fn composing(&self) -> bool {
        self.composing.get()
    }

    /// 组句停了就把「探过输入框」一并清掉：下一段可能在别的输入框里。
    pub(crate) fn set_composing(&self, value: bool) {
        if !value {
            self.probed.set(false);
        }
        self.composing.set(value);
    }

    /// 这段组句该不该探一次输入框；探过返回 `false`，同段不重复。
    pub(super) fn probe_input(&self) -> bool {
        !self.probed.replace(true)
    }

    /// 通知 Server 收起候选窗口。引擎正被别处借着或没连上时静默跳过。
    pub(crate) fn hide_candidates(&self) {
        if let Ok(mut guard) = self.client.try_borrow_mut()
            && let Some(client) = guard.as_mut()
        {
            let _ = client.hide_candidates();
        }
    }

    pub(crate) fn has_composition(&self) -> bool {
        self.composition.borrow().is_some()
    }

    /// clone 出来再用，别把 `borrow()` 挂在 match 上（分支里再 `borrow_mut` 会 panic）。
    pub(super) fn composition(&self) -> Option<ITfComposition> {
        self.composition.borrow().clone()
    }

    /// 组句收了就忘掉光标矩形（见 [`forget_anchor`](Self::forget_anchor)）。
    pub(super) fn set_composition(&self, composition: Option<ITfComposition>) {
        if composition.is_none() {
            self.forget_anchor();
        }
        *self.composition.borrow_mut() = composition;
    }

    pub(super) fn take_composition(&self) -> Option<ITfComposition> {
        self.forget_anchor();
        self.composition.borrow_mut().take()
    }

    /// 组句结束（应用终止组句 / 断线 / 失焦上屏）：不再当作在组句，并让 Server 收候选窗口。
    pub(crate) fn end_composing(&self) {
        self.set_composing(false);
        self.hide_candidates();
    }

    /// 清掉一切本地组句状态，不碰文档。
    pub(crate) fn reset(&self) {
        self.set_composition(None);
        self.set_last_context(None);
        self.end_composing();
    }

    /// 组句被应用强行终止：本地清掉，并记下 Server 的缓冲还没清。
    pub(super) fn terminated(&self) {
        self.reset();
        self.server_stale.set(true);
    }
}
