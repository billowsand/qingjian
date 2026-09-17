//! 按键怎么作用到 Engine / 高亮上。分流规则与 macOS 壳的 `handle_text` / `handle_command` 对齐。

use qingjian_platform::protocol::KeyEvent;

use super::{Effect, codes, with_prefix};
use crate::dispatch::Router;

impl Router {
    /// 功能键靠键码，其余靠字符。组句中修饰键 + 数字是快捷键；带 Ctrl / Alt / Win 而没配到快捷键的键归应用。
    pub(crate) fn apply_key(&mut self, event: &KeyEvent) -> Effect {
        if self.composing()
            && let Some(digit) = codes::digit_key(event.virtual_key)
            && let Some(effect) = self.apply_digit_shortcut(digit, event.modifiers.chord())
        {
            return effect;
        }
        if event.modifiers.has_command_key() {
            return Effect::Passthrough;
        }
        let Some(c) = event.character.filter(|c| !c.is_control()) else {
            return self.apply_function_key(event);
        };
        // 英文状态（Caps Lock 亮着或持久英文模式）：敲的字母直接进输入框，不组句也不出候选窗。
        let english = event.modifiers.caps || event.modifiers.english_mode;
        if english {
            self.apply_english(c, event)
        } else {
            self.apply_chinese(c, event)
        }
    }

    /// 退格 / Esc / 回车 / Tab / 方向键；没在组句时都交还应用。
    fn apply_function_key(&mut self, event: &KeyEvent) -> Effect {
        if !self.composing() {
            // 回车交给应用：文本流里是一个段落边界（macOS 壳同样记）
            if event.virtual_key == codes::RETURN {
                self.engine.note_passthrough('\n');
            }
            return Effect::Passthrough;
        }
        match event.virtual_key {
            codes::BACK => {
                self.engine.backspace();
                Effect::Changed(None)
            }
            codes::ESCAPE => {
                self.engine.clear();
                Effect::Changed(None)
            }
            codes::RETURN => Effect::Changed(Some(self.engine.take_raw())),
            codes::DOWN => {
                self.move_highlight(1);
                Effect::Navigated
            }
            codes::UP => {
                self.move_highlight(-1);
                Effect::Navigated
            }
            codes::NEXT => {
                self.page(1);
                Effect::Navigated
            }
            codes::PRIOR => {
                self.page(-1);
                Effect::Navigated
            }
            codes::LEFT => {
                self.engine.move_cursor_left();
                Effect::Changed(None)
            }
            codes::RIGHT => {
                self.engine.move_cursor_right();
                Effect::Changed(None)
            }
            codes::HOME => {
                self.engine.move_cursor_home();
                Effect::Changed(None)
            }
            codes::END => {
                self.engine.move_cursor_end();
                Effect::Changed(None)
            }
            _ => Effect::Passthrough,
        }
    }

    /// 中文模式：小写字母进拼音；Shift 大写字母是临时打英文，组句中先把拼音原样上屏；
    /// 辅码开着时组句中的大写字母是辅码键，进缓冲区；
    /// 没在组句时的其他字符走全角标点（与 macOS 壳一致，组句中的标点仍进英文直输段）。
    fn apply_chinese(&mut self, c: char, event: &KeyEvent) -> Effect {
        if c.is_ascii_uppercase() {
            if self.composing() && self.engine.fuma_enabled() {
                // 辅码键：进缓冲区参与过滤（触发判定与反转顺序在 Core）
                self.engine.push(c);
                return Effect::Changed(None);
            }
            let raw = self.composing().then(|| self.engine.take_raw());
            self.engine.note_passthrough(c);
            return with_prefix(raw, Effect::Passthrough, c);
        }
        if c.is_ascii_lowercase() {
            self.engine.push(c);
            return Effect::Changed(None);
        }
        if !self.composing() {
            return self.apply_punctuation(c, event);
        }
        self.apply_printable(c, event)
    }

    /// 当前模式开着全角就让 Core 转（数字后的 `.` 保持半角）；转不了的原样交给应用并告知 Core。
    fn apply_punctuation(&mut self, c: char, event: &KeyEvent) -> Effect {
        let english = event.modifiers.caps || event.modifiers.english_mode;
        if self.full_width_for(english)
            && let Some(text) = self.engine.punctuate(c)
        {
            return Effect::Changed(Some(text.to_owned()));
        }
        self.engine.note_passthrough(c);
        Effect::Passthrough
    }

    /// 英文状态：字母由我们直接插进输入框（大小写按 Shift），不进缓冲区，所以没有候选窗口。
    /// 中文模式敲了一半切过来的，先把拼音原样上屏。其他键按英文模式那份全角设置转，转不了的交给应用。
    fn apply_english(&mut self, c: char, event: &KeyEvent) -> Effect {
        let raw = self.composing().then(|| self.engine.take_raw());
        let effect = if c.is_ascii_alphabetic() {
            self.engine.note_passthrough(c);
            Effect::Changed(Some(c.to_string()))
        } else {
            self.apply_punctuation(c, event)
        };
        with_prefix(raw, effect, c)
    }

    /// 组句中的可打印键：数字选当前页第 N 个，翻页键翻页，空格上屏高亮，其余进英文直输段。
    /// 微软 / 搜狗双拼的 `;` 是 ing 键，末尾有落单声母时进缓冲区。
    fn apply_printable(&mut self, c: char, event: &KeyEvent) -> Effect {
        if c == ';' && self.engine.takes_semicolon() {
            self.engine.push(c);
            return Effect::Changed(None);
        }
        if let Some(digit) = codes::digit(event)
            && self.candidate_count() > 0
        {
            let page_size = self.config.page_size;
            let page = self.highlight / page_size;
            return Effect::Changed(self.commit_index(page * page_size + digit - 1));
        }
        if let Some(step) = codes::page_key(event, self.config.page_keys) {
            self.page(step);
            return Effect::Navigated;
        }
        if c == ' ' {
            return Effect::Changed(Some(self.commit_highlighted()));
        }
        self.engine.push(c);
        Effect::Changed(None)
    }

    /// 上屏高亮候选；没有候选时缓冲原样上屏。
    fn commit_highlighted(&mut self) -> String {
        match self.commit_index(self.highlight) {
            Some(text) => text,
            None => self.engine.take_raw(),
        }
    }

    fn composing(&self) -> bool {
        !self.engine.composition().is_empty()
    }
}
