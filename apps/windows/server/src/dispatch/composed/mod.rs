//! 组句的展示状态：缓冲变化时重查候选并重建 [`Composed`]，云端词异步并入，高亮 / 翻页，按状态生成给 DLL 的帧。

mod state;

use qingjian_core::{Candidate, CandidateLayout, CandidateList, CloudWord};
use qingjian_platform::protocol::{FUMA_PREEDIT_PROTOCOL, Frame, PreeditKind, PreeditSegment};

pub(super) use self::state::Composed;
use super::Router;

impl Router {
    /// 把聚焦会话那份 DLL 不认识的 preedit 种类降级成它认识的。
    ///
    /// 枚举新加的变体不在 serde「忽略未知字段」的保护范围里：老 DLL 收到不认识的变体名，
    /// **整条消息**反序列化就失败，帧丢了候选窗就不动了。升级安装后没重启的应用里还跑着老 DLL，
    /// 所以按会话报来的协议版本降级。辅码段降成 `Rest`：老 DLL 照样画淡、内联也照样拼上，行为不变。
    fn downgrade_preedit(&self, segments: &mut [PreeditSegment]) {
        if self.focused_protocol() >= FUMA_PREEDIT_PROTOCOL {
            return;
        }
        for segment in segments {
            if segment.kind == PreeditKind::Fuma {
                segment.kind = PreeditKind::Rest;
            }
        }
    }

    /// 缓冲变化后：按 Engine 状态重建 [`Composed`]，发一次云联想请求，归零高亮与整句补全。
    pub(super) fn recompose(&mut self) {
        self.highlight = 0;
        self.navigated = false;
        self.sentence = None;
        if self.engine.composition().is_empty() {
            self.composed = None;
            self.cancel_prediction();
            self.stop_rescoring();
            return;
        }
        self.attach_loaded_model();
        let built = self.engine.query().ok().map(|query| {
            let items = query.candidates.items.clone();
            let mut preedit: Vec<PreeditSegment> =
                query.marked_segments().iter().map(Into::into).collect();
            self.downgrade_preedit(&mut preedit);
            // 光标用 Core 的映射：自动补的 `'` 会让显示串比敲的长。
            (items, preedit, query.marked_cursor())
        });
        self.composed = Some(match built {
            Some((items, preedit, cursor)) => {
                let layout =
                    CandidateLayout::new(items, self.config.page_size, self.config.cloud_slots);
                if self.engine.prediction_enabled() {
                    self.engine.request_prediction(None, layout.local());
                }
                Composed::Candidates {
                    preedit,
                    cursor,
                    layout,
                }
            }
            None => {
                self.cancel_prediction();
                let composition = self.engine.composition();
                let text = composition.text().to_owned();
                let cursor = text[..composition.cursor()].chars().count();
                Composed::Raw { text, cursor }
            }
        });
        self.schedule_rescoring();
    }

    /// 拉一次云联想结果：云端词并进候选布局，整句补全记下。
    pub(super) fn poll_prediction(&mut self) {
        if !self.engine.prediction_enabled() {
            return;
        }
        let Some(prediction) = self.engine.poll_prediction() else {
            return;
        };
        if let Some(Composed::Candidates { layout, .. }) = self.composed.as_mut() {
            let words: Vec<Candidate> = prediction
                .words
                .into_iter()
                .map(CloudWord::into_candidate)
                .collect();
            layout.set_cloud(words);
            self.sentence = prediction.sentence;
        }
    }

    pub(super) fn cancel_prediction(&mut self) {
        if self.engine.prediction_enabled() {
            self.engine.cancel_prediction();
        }
    }

    /// 高亮移动 `delta`，夹在 `[0, 末尾]`，到页边自然换页。
    pub(super) fn move_highlight(&mut self, delta: isize) {
        let count = self.candidate_count();
        if count == 0 {
            self.highlight = 0;
            return;
        }
        let next = (self.highlight as isize + delta).clamp(0, count as isize - 1) as usize;
        self.navigated |= next != self.highlight;
        self.highlight = next;
    }

    /// 整页翻 `step`，高亮落到目标页第一个候选。
    pub(super) fn page(&mut self, step: isize) {
        let count = self.candidate_count();
        if count == 0 {
            self.highlight = 0;
            return;
        }
        let page_size = self.config.page_size;
        let page_count = count.div_ceil(page_size);
        let current = (self.highlight / page_size) as isize;
        let target = (current + step).clamp(0, page_count as isize - 1) as usize;
        if target != current as usize {
            self.navigated = true;
            self.engine.note_page_turn();
        }
        self.highlight = (target * page_size).min(count - 1);
    }

    pub(super) fn candidate_count(&self) -> usize {
        match &self.composed {
            Some(Composed::Candidates { layout, .. }) => layout.len(),
            _ => 0,
        }
    }

    /// 候选布局里第 `index` 个（跨页下标）。
    pub(super) fn layout_candidate(&self, index: usize) -> Option<Candidate> {
        match &self.composed {
            Some(Composed::Candidates { layout, .. }) => layout.candidate(index).cloned(),
            _ => None,
        }
    }

    pub(super) fn commit_index(&mut self, index: usize) -> Option<String> {
        let candidate = self.layout_candidate(index)?;
        Some(self.engine.commit(&candidate))
    }

    /// 按当前状态生成一帧：没在组句给空帧；否则给高亮所在的那一页。
    pub(super) fn current_frame(&self) -> Frame {
        match &self.composed {
            None => Frame::default(),
            Some(Composed::Raw { text, cursor }) => Frame {
                preedit: vec![PreeditSegment {
                    text: text.clone(),
                    kind: PreeditKind::Typed,
                }],
                cursor: *cursor,
                candidates: CandidateList { items: Vec::new() },
                highlight: usize::MAX,
                page: 0,
                page_count: 1,
                layout: self.config.layout,
                theme: self.config.theme,
                sentence: None,
                notice: self.notice.clone(),
                inline_preedit: self.config.preedit.inline(),
            },
            Some(Composed::Candidates {
                preedit,
                cursor,
                layout,
            }) => {
                let page_size = self.config.page_size;
                let highlight = self.highlight.min(layout.len().saturating_sub(1));
                let page = highlight / page_size;
                let items: Vec<Candidate> = layout
                    .page(page)
                    .into_iter()
                    .filter_map(|cell| cell.candidate().cloned())
                    .collect();
                let mut candidates = CandidateList { items };
                self.engine.annotate(&mut candidates);
                Frame {
                    preedit: preedit.clone(),
                    cursor: *cursor,
                    candidates,
                    highlight: highlight - page * page_size,
                    page,
                    page_count: layout.pages().max(1),
                    layout: self.config.layout,
                    theme: self.config.theme,
                    sentence: self.sentence.clone(),
                    notice: self.notice.clone(),
                    inline_preedit: self.config.preedit.inline(),
                }
            }
        }
    }
}
