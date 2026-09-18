//! 候选窗口一次绘制要用的全部内容，由帧换算而来；渲染器要的帧由 [`RenderData::render_frame`] 再换一次。

use qingjian_platform::ColorScheme;
use qingjian_platform::protocol::{Frame, PreeditKind};
use qingjian_render::{Preedit, PreeditSegment, PreeditStyle, Row};

use super::row;

/// 一次绘制要用的全部内容。
pub(crate) struct RenderData {
    /// 顶部拼音行的各段。
    pub(super) preedit: Vec<(String, PreeditKind)>,

    /// 光标在拼音行里的字符位置。
    pub(super) cursor: usize,

    /// 候选行。
    pub(super) rows: Vec<Row>,

    /// 高亮行下标（页内）。
    pub(super) highlight: usize,

    /// 页码，只有多页时有。
    pub(super) footer: Option<String>,

    /// 辅码「下一键」幽灵提示（` s`，含前导空格），只敲了首码时才有：高亮候选的第二码，
    /// 淡画在拼音行末尾。不放候选旁的标注里，否则候选框高度随标注行出现 / 消失而变。
    pub(super) fuma_hint: Option<String>,

    /// 屏幕提示（删候选后的「已删除…」），画在拼音行下方。
    pub(super) notice: Option<String>,

    /// 候选窗口、状态条与设置程序共用的色系。
    pub(super) color_scheme: ColorScheme,
}

impl RenderData {
    pub(super) fn empty() -> Self {
        Self {
            preedit: Vec::new(),
            cursor: 0,
            rows: Vec::new(),
            highlight: usize::MAX,
            footer: None,
            fuma_hint: None,
            notice: None,
            color_scheme: ColorScheme::default(),
        }
    }

    pub(super) fn set(&mut self, frame: &Frame) {
        self.color_scheme = frame.color_scheme;
        self.preedit = frame
            .preedit
            .iter()
            .map(|segment| (segment.text.clone(), segment.kind))
            .collect();
        self.cursor = frame.cursor;
        self.rows = frame
            .candidates
            .items
            .iter()
            .enumerate()
            .map(|(i, candidate)| row::from_candidate(i, candidate))
            .collect();
        self.highlight = frame.highlight;
        self.footer =
            (frame.page_count > 1).then(|| format!("{}/{}", frame.page + 1, frame.page_count));
        self.notice = frame.notice.clone();
        self.fuma_hint = fuma_hint(frame);
    }

    /// 渲染器要的帧。提示（删了什么词）在渲染器里画在拼音行右侧，与 macOS 一致。
    pub(super) fn render_frame(&self) -> qingjian_render::Frame {
        let preedit = (!self.preedit.is_empty()).then(|| Preedit {
            segments: self
                .preedit
                .iter()
                .map(|(text, kind)| PreeditSegment {
                    text: text.clone(),
                    style: match kind {
                        PreeditKind::Typed => PreeditStyle::Typed,
                        PreeditKind::Rest => PreeditStyle::Rest,
                        PreeditKind::Fuma => PreeditStyle::Fuma,
                        PreeditKind::Corrected => PreeditStyle::Struck,
                    },
                })
                .collect(),
            cursor: self.cursor,
        });
        qingjian_render::Frame {
            preedit,
            rows: self.rows.clone(),
            // 协议里 usize::MAX 表示不高亮。
            highlighted: (self.highlight != usize::MAX).then_some(self.highlight),
            footer: self.footer.clone(),
            sentence: None,
            status: self.notice.clone(),
            fuma_hint: self.fuma_hint.clone(),
        }
    }
}

/// 只敲了辅码首码时，取高亮候选的第二码当「下一键」提示；两码敲完（拼音行里已有辅码段）
/// 或没敲辅码时为 `None`。Candidate::fuma 只在敲了辅码时填，所以取到它就说明正处在首码那档。
fn fuma_hint(frame: &Frame) -> Option<String> {
    if frame
        .preedit
        .iter()
        .any(|segment| segment.kind == PreeditKind::Fuma)
    {
        return None;
    }
    frame
        .candidates
        .items
        .get(frame.highlight)
        .and_then(|candidate| candidate.fuma.as_deref())
        .and_then(|codes| codes.chars().nth(1))
        .map(|code| format!(" {code}"))
}

#[cfg(test)]
mod tests {
    use qingjian_core::{Candidate, CandidateKind, CandidateList};
    use qingjian_platform::protocol::{Frame, PreeditKind, PreeditSegment};

    use super::fuma_hint;

    fn candidate(fuma: Option<&str>) -> Candidate {
        Candidate {
            text: "栏".to_owned(),
            kind: CandidateKind::Chinese,
            syllables: Vec::new(),
            reading: None,
            translation: None,
            fuma: fuma.map(str::to_owned),
        }
    }

    fn frame(kinds: &[PreeditKind], items: Vec<Candidate>, highlight: usize) -> Frame {
        Frame {
            preedit: kinds
                .iter()
                .map(|kind| PreeditSegment {
                    text: "x".to_owned(),
                    kind: *kind,
                })
                .collect(),
            candidates: CandidateList { items },
            highlight,
            ..Frame::default()
        }
    }

    /// 只敲首码：高亮候选的第二码是提示，前导空格由这里带上。
    #[test]
    fn first_code_hints_the_second() {
        let frame = frame(
            &[PreeditKind::Typed],
            vec![candidate(Some("ms")), candidate(Some("cm"))],
            0,
        );
        assert_eq!(fuma_hint(&frame).as_deref(), Some(" s"));
    }

    /// 两码都敲了：辅码段已在拼音行里，不画幽灵提示。
    #[test]
    fn both_codes_draw_no_hint() {
        let frame = frame(
            &[PreeditKind::Typed, PreeditKind::Fuma],
            vec![candidate(Some("ms"))],
            0,
        );
        assert_eq!(fuma_hint(&frame), None);
    }

    /// 没敲辅码（候选没标辅码）或高亮越界：都没有提示。
    #[test]
    fn no_fuma_no_hint() {
        let without_fuma = frame(&[PreeditKind::Typed], vec![candidate(None)], 0);
        assert_eq!(fuma_hint(&without_fuma), None);
        let out_of_range = frame(&[PreeditKind::Typed], vec![candidate(Some("ms"))], 9);
        assert_eq!(fuma_hint(&out_of_range), None);
    }
}
