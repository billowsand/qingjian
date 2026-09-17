//! 组句 preedit：把 Server 回的拼音行经 TSF 组句（[`ITfComposition`]）显示在文档光标处（对应 macOS 的内联 marked text）。
//! 只放最朴素的一行拼音；富样式的拼音行在候选窗口里另画。所有写操作都在异步读写编辑会话里做（理由见 `edit_session`）。

mod shared;
mod sink;

use std::mem::ManuallyDrop;
use std::rc::Rc;

use windows::Win32::UI::TextServices::{
    INSERT_TEXT_AT_SELECTION_FLAGS, ITfComposition, ITfCompositionSink, ITfContext,
    ITfContextComposition, ITfInsertAtSelection, ITfRange, TF_AE_END, TF_ANCHOR_END,
    TF_IAS_QUERYONLY, TF_SELECTION, TF_SELECTIONSTYLE,
};
use windows::core::{Interface, Result};

use qingjian_platform::protocol::{Frame, PreeditKind};

pub(crate) use self::shared::Shared;
use self::sink::CompositionSink;
use super::edit::{InputContext, anchor_rect, input_context, selection_start};
use super::service::SharedClient;

/// 内联要显示的拼音行（跳过被纠错划掉的原字母）；空串表示没有组句内容。
/// `[general] preedit` 设成「只在候选窗口」时 Server 会把 `inline_preedit` 关掉，这里一律给空串：
/// 应用里不留 marked text，拼音只在候选窗口顶部显示。
pub(crate) fn preedit_string(frame: &Frame) -> String {
    if !frame.inline_preedit {
        return String::new();
    }
    frame
        .preedit
        .iter()
        .filter(|segment| segment.kind != PreeditKind::Corrected)
        .map(|segment| segment.text.as_str())
        .collect()
}

/// 在编辑会话回调（持写锁 `ec`）里调：先落定 `commit`，再按 `preedit` 起 / 改 / 收组句，最后把光标位置报给 Server。
/// 一段组句起头时顺手判输入框私密不私密（变了就告诉 Server）、把光标前的文字送给 Server（本地整句模型的前文）。
pub(crate) fn apply(
    shared: &Rc<Shared>,
    engine: &SharedClient,
    context: &ITfContext,
    ec: u32,
    commit: Option<&str>,
    preedit: &str,
) -> Result<()> {
    if let Some(text) = commit {
        commit_text(shared, context, ec, text)?;
    }
    // 探输入框要赶在插拼音之前（此时选区还是原来的插入点）。这段组句探过没有看 `probe_input`，
    // 不看有没有组句：行内拼音关掉时压根不起组句，拿它当「起头」会一次都探不到。
    let input = (shared.composing() && shared.probe_input()).then(|| input_context(context, ec));
    if preedit.is_empty() {
        end_composition(shared, ec)?;
    } else {
        update_preedit(shared, context, ec, preedit)?;
    }
    if let Some(InputContext { private, before }) = input {
        report_privacy(engine, private);
        if let Some(before) = before {
            report_surrounding(engine, before);
        }
    }
    report_caret(shared, engine, context, ec);
    Ok(())
}

/// 告诉 Server 输入框私密与否（客户端只在变了时真发）；引擎正被别处借着（罕见）就算了，下段组句再报。
fn report_privacy(engine: &SharedClient, private: bool) {
    if let Ok(mut guard) = engine.try_borrow_mut()
        && let Some(client) = guard.as_mut()
        && let Err(error) = client.set_private(private)
    {
        super::log::log(&format!("报私密状态失败: {error}"));
    }
}

/// 把光标前文送给 Server；引擎正被别处借着（罕见）就算了，Server 退回会话历史。
fn report_surrounding(engine: &SharedClient, before: String) {
    let chars = before.chars().count();
    if let Ok(mut guard) = engine.try_borrow_mut()
        && let Some(client) = guard.as_mut()
    {
        match client.surrounding(before) {
            Ok(()) => super::log::log(&format!("送光标前文 {chars} 字")),
            Err(error) => super::log::log(&format!("送光标前文失败: {error}")),
        }
    }
}

/// 组句进行中才报位置；组句已收 Server 会按空帧 / `Commit` 自行收窗口。
///
/// 锚点优先用组句范围（拼音就在那儿）；行内拼音关掉时压根没有组句，退回当前选区（插入点）——
/// 两者都量不到时 [`anchor_rect`] 再退回鼠标位置。
fn report_caret(shared: &Shared, engine: &SharedClient, context: &ITfContext, ec: u32) {
    if !shared.composing() {
        // 下一段组句在别处，别拿这次的矩形当兜底。
        shared.forget_anchor();
        return;
    }
    let range = match shared.composition() {
        Some(composition) => unsafe { composition.GetRange() }.ok(),
        None => selection_start(context, ec),
    };
    let Some(range) = range else {
        return;
    };
    let rect = anchor_rect(context, ec, &range, shared.last_anchor());
    shared.set_last_anchor(rect);
    // 引擎正被别处借着（罕见）就跳过这拍，Server 保持上次位置。
    if let Ok(mut guard) = engine.try_borrow_mut()
        && let Some(client) = guard.as_mut()
        && let Err(error) = client.position_candidates(rect)
    {
        super::log::log(&format!("上报候选窗口位置失败: {error}"));
    }
}

/// 有组句就把组句范围替换成 `text` 再结束组句，否则在选区插入。
fn commit_text(shared: &Shared, context: &ITfContext, ec: u32, text: &str) -> Result<()> {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    match shared.composition() {
        Some(composition) => {
            let range = unsafe { composition.GetRange()? };
            unsafe { range.SetText(ec, 0, &utf16)? };
            move_selection_to_end(context, ec, &range)?;
            unsafe { composition.EndComposition(ec)? };
            shared.set_composition(None);
        }
        None => {
            let insert: ITfInsertAtSelection = context.cast()?;
            // 标志不能用 NOQUERY：它不回传 range，windows-rs 会把 NULL 当失败。
            let range = unsafe {
                insert.InsertTextAtSelection(ec, INSERT_TEXT_AT_SELECTION_FLAGS(0), &utf16)?
            };
            // 不移光标的话下一次插入又落在原处，字会从右往左堆。
            move_selection_to_end(context, ec, &range)?;
        }
    }
    Ok(())
}

fn update_preedit(shared: &Rc<Shared>, context: &ITfContext, ec: u32, preedit: &str) -> Result<()> {
    let composition = match shared.composition() {
        Some(composition) => composition,
        None => start_composition(shared, context, ec)?,
    };
    let utf16: Vec<u16> = preedit.encode_utf16().collect();
    let range = unsafe { composition.GetRange()? };
    unsafe { range.SetText(ec, 0, &utf16)? };
    super::display_attribute::mark(context, ec, &range);
    move_selection_to_end(context, ec, &range)
}

/// 在当前选区处起一个空组句；组句 sink 交给框架持有。
fn start_composition(shared: &Rc<Shared>, context: &ITfContext, ec: u32) -> Result<ITfComposition> {
    let insert: ITfInsertAtSelection = context.cast()?;
    let range = unsafe { insert.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])? };
    let context_composition: ITfContextComposition = context.cast()?;
    let sink: ITfCompositionSink = CompositionSink::new(shared.clone()).into();
    let composition = unsafe { context_composition.StartComposition(ec, &range, &sink)? };
    shared.set_composition(Some(composition.clone()));
    Ok(composition)
}

/// 清空组句文本再结束，避免残留拼音。
fn end_composition(shared: &Shared, ec: u32) -> Result<()> {
    if let Some(composition) = shared.take_composition() {
        let range = unsafe { composition.GetRange()? };
        unsafe { range.SetText(ec, 0, &[])? };
        unsafe { composition.EndComposition(ec)? };
    }
    Ok(())
}

fn move_selection_to_end(context: &ITfContext, ec: u32, range: &ITfRange) -> Result<()> {
    let end = unsafe { range.Clone()? };
    unsafe { end.Collapse(ec, TF_ANCHOR_END)? };
    let selection = TF_SELECTION {
        range: ManuallyDrop::new(Some(end)),
        style: TF_SELECTIONSTYLE {
            ase: TF_AE_END,
            fInterimChar: false.into(),
        },
    };
    // SetSelection 不接管 range 的所有权，之后手动释放。
    let result = unsafe { context.SetSelection(ec, std::slice::from_ref(&selection)) };
    drop(ManuallyDrop::into_inner(selection.range));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use qingjian_platform::protocol::PreeditSegment;

    fn frame_with(inline_preedit: bool) -> Frame {
        Frame {
            preedit: vec![
                PreeditSegment {
                    text: "ni".to_owned(),
                    kind: PreeditKind::Corrected,
                },
                PreeditSegment {
                    text: "ni'hao".to_owned(),
                    kind: PreeditKind::Typed,
                },
            ],
            inline_preedit,
            ..Frame::default()
        }
    }

    #[test]
    fn inline_preedit_skips_the_corrected_letters() {
        assert_eq!(preedit_string(&frame_with(true)), "ni'hao");
    }

    /// 「只在候选窗口」：应用里一个字都不留，拼音分段仍在帧里给候选窗口画。
    #[test]
    fn preedit_stays_out_of_the_document_when_inline_is_off() {
        assert_eq!(preedit_string(&frame_with(false)), "");
    }
}
