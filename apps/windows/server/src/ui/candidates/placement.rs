//! 候选窗口贴在光标上方还是下方，以及贴在哪个坐标。
//!
//! 只按「这一帧放不放得下」决定上下的话，同一行里窗口会上下乱跳：候选条数、释义、页码、提示行
//! 都会让窗口忽高忽低，光标离屏幕底边不远时高一点就放不下、矮一点又放得下。
//! 所以记住上次贴的是哪一边，只要还在同一行、那一边还放得下，就一直贴那边。

use std::cell::Cell;

use windows::Win32::Foundation::{POINT, RECT};

use crate::ui::monitor;

/// 光标行与候选窗之间的间隙（逻辑像素）。
const CARET_GAP: i32 = 2;

/// 上一次的摆放，[`place`] 读写。
#[derive(Clone, Copy)]
pub(super) struct Placement {
    /// 当时光标行的上下沿（屏幕物理像素）。新的锚点与它在竖直方向有重叠就算还在同一行。
    line: (i32, i32),

    /// 当时贴的是光标上方。
    above: bool,
}

/// 记住上次摆放；`None` 是还没摆过。
pub(super) type LastPlacement = Cell<Option<Placement>>;

/// 内容左上角：缺省贴光标下方，放不下贴上方，再放不下贴屏幕内；都夹在光标所在显示器的工作区里。
/// 同一行里沿用上次那一边，并把这次的选择记进 `last`。
pub(super) fn place(last: &LastPlacement, anchor: RECT, content: (i32, i32)) -> (i32, i32) {
    let work = monitor::work_area_near(POINT {
        x: anchor.left,
        y: anchor.top,
    });
    let (position, above) = place_in(work, anchor, content, sticky_side(last, anchor));
    last.set(Some(Placement {
        line: (anchor.top, anchor.bottom),
        above,
    }));
    position
}

/// 上次贴的那一边，锚点换了行就没有。
fn sticky_side(last: &LastPlacement, anchor: RECT) -> Option<bool> {
    last.get()
        .filter(|placement| anchor.top < placement.line.1 && placement.line.0 < anchor.bottom)
        .map(|placement| placement.above)
}

/// 纯算的那一半：返回内容左上角与这次贴的是不是上方。`sticky` 是同一行上次贴的那一边。
fn place_in(
    work: RECT,
    anchor: RECT,
    content: (i32, i32),
    sticky: Option<bool>,
) -> ((i32, i32), bool) {
    let x = anchor
        .left
        .clamp(work.left, (work.right - content.0).max(work.left));
    let below = anchor.bottom + CARET_GAP;
    let above = anchor.top - CARET_GAP - content.1;
    let fits_below = below + content.1 <= work.bottom;
    let fits_above = above >= work.top;
    let use_above = match sticky {
        // 上次那边还放得下就还用它：这一帧候选多了少了都不改边
        Some(true) if fits_above => true,
        Some(false) if fits_below => false,
        _ => !fits_below && fits_above,
    };
    let y = if use_above {
        above
    } else if fits_below {
        below
    } else {
        (work.bottom - content.1).max(work.top)
    };
    ((x, y), use_above)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: RECT = RECT {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1000,
    };

    /// 光标行 `top`–`top + 20`，左边界 100。
    fn caret(top: i32) -> RECT {
        RECT {
            left: 100,
            top,
            right: 108,
            bottom: top + 20,
        }
    }

    #[test]
    fn defaults_to_below_and_flips_up_only_when_it_does_not_fit() {
        let ((x, y), above) = place_in(WORK, caret(400), (300, 200), None);
        assert_eq!((x, y, above), (100, 422, false));
        // 贴着屏幕底边：放不下就贴上方，窗口下沿在光标上方 CARET_GAP 处
        let ((_, y), above) = place_in(WORK, caret(900), (300, 200), None);
        assert_eq!((y, above), (698, true));
    }

    #[test]
    fn keeps_the_side_while_the_caret_stays_on_the_same_line() {
        let last: LastPlacement = Cell::new(Some(Placement {
            line: (900, 920),
            above: true,
        }));
        let anchor = caret(900);
        // 这一帧只剩一条候选，下方放得下了，但同一行仍贴上方
        let ((_, y), above) = place_in(WORK, anchor, (300, 40), sticky_side(&last, anchor));
        assert_eq!((y, above), (858, true));
        // 换到别的行：重新按放不放得下决定
        let anchor = caret(400);
        let ((_, y), above) = place_in(WORK, anchor, (300, 40), sticky_side(&last, anchor));
        assert_eq!((y, above), (422, false));
    }

    #[test]
    fn a_sticky_side_that_no_longer_fits_gives_way() {
        let last: LastPlacement = Cell::new(Some(Placement {
            line: (100, 120),
            above: true,
        }));
        let anchor = caret(100);
        // 上方只剩 100 像素，装不下 200 高的窗口：让给下方
        let ((_, y), above) = place_in(WORK, anchor, (300, 200), sticky_side(&last, anchor));
        assert_eq!((y, above), (122, false));
    }

    #[test]
    fn content_stays_inside_the_work_area() {
        // 窗口比工作区还高：贴在工作区内，不越出屏幕
        let ((_, y), above) = place_in(WORK, caret(500), (300, 2000), None);
        assert_eq!((y, above), (0, false));
        // 太靠右：左移到刚好放得下
        let anchor = RECT {
            left: 1900,
            ..caret(400)
        };
        let ((x, _), _) = place_in(WORK, anchor, (300, 200), None);
        assert_eq!(x, 1620);
    }

    /// 真实工作区大小不定，只核对「记住了这一行」与「同一行不改边」。
    #[test]
    fn place_remembers_the_line_and_keeps_the_side() {
        let last: LastPlacement = Cell::new(None);
        place(&last, caret(400), (300, 200));
        let first = last.get().expect("摆过一次就记下了");
        assert_eq!(first.line, (400, 420));
        // 同一行下一帧窗口矮了很多，仍贴同一边
        place(&last, caret(400), (300, 60));
        assert_eq!(last.get().expect("还记着").above, first.above);
    }
}
