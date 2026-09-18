//! 候选窗口的输出端。

use qingjian_platform::protocol::{Frame, ScreenRect};

/// Router 只产出帧，画交给它；Windows 上由 UI 线程实现。
pub trait CandidateSink: Send {
    /// 把候选窗口摆到 `rect`（组句范围的屏幕矩形）下方并按 `frame` 重绘。
    fn show(&self, frame: Frame, rect: ScreenRect);

    fn hide(&self);

    /// 换候选窗口字体：装上时与配置热加载后调，只在设置变了时调。
    fn set_font(&self, font: String);
}

/// 不画候选窗口的空实现。
pub struct NoopSink;

impl CandidateSink for NoopSink {
    fn show(&self, _frame: Frame, _rect: ScreenRect) {}

    fn hide(&self) {}

    fn set_font(&self, _font: String) {}
}
