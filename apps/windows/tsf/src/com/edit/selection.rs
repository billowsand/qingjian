//! 当前选区（插入点）：起组句时读光标前文要它，行内拼音关掉、没有组句范围可锚时候选窗口的定位也要它。

use std::mem::ManuallyDrop;

use windows::Win32::UI::TextServices::{
    ITfContext, ITfRange, TF_ANCHOR_START, TF_DEFAULT_SELECTION, TF_SELECTION,
};

/// 选区折成起点（插入点）；没有选区或读不到时为 `None`。
pub(crate) fn selection_start(context: &ITfContext, ec: u32) -> Option<ITfRange> {
    let mut selection = [TF_SELECTION::default()];
    let mut fetched = 0u32;
    unsafe {
        context
            .GetSelection(ec, TF_DEFAULT_SELECTION, &mut selection, &mut fetched)
            .ok()?;
    }
    if fetched == 0 {
        return None;
    }
    // GetSelection 移交 range 的所有权（ManuallyDrop），取出后由这里释放。
    let range = unsafe { ManuallyDrop::take(&mut selection[0].range) }?;
    unsafe { range.Collapse(ec, TF_ANCHOR_START) }.ok()?;
    Some(range)
}
