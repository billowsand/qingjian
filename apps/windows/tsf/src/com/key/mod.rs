//! 按键相关：TSF 虚拟键码到协议 [`KeyEvent`](qingjian_platform::protocol::KeyEvent) 的翻译（[`event`]）、
//! 单击 Shift 切中英的判定（[`shift`]）。

pub(crate) mod event;
mod layout;
mod shift;

pub(crate) use self::shift::ShiftTap;
