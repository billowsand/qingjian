//! 「协议对不上」的全局闸。
//!
//! 升级安装时新的 DLL 是按版本并排装的，已经开着的应用仍加载着旧 DLL、换不掉（见安装脚本的「升级」一节）。
//! 协议不兼容时旧 DLL 再怎么重连都读不懂 Server 的话，从前的做法是每一键「失败 → 断开 → 重连 → 再失败」，
//! 输入法在这个应用里既打不出字又吞掉按键，只能重启系统。
//!
//! 所以一旦发现对不上就合上这道闸：本进程之后所有键整键放行（退回直接打英文），不再连 Server。
//! 闸是进程级的——协议合不合得上是「这份 DLL 对这个 Server」的事，与线程、与哪个文本服务实例无关；
//! 也不会再打开：DLL 在本进程里换不掉，用户重启这个应用才会加载新的。

use core::sync::atomic::{AtomicBool, Ordering};

use crate::com::log::log;

static MISMATCHED: AtomicBool = AtomicBool::new(false);

/// 记下协议对不上，并记一条日志（只记第一次，免得每键刷屏）。
pub fn mark(reason: &str) {
    if MISMATCHED.swap(true, Ordering::SeqCst) {
        return;
    }
    log(&format!(
        "与 Server 的协议对不上（{reason}）：本应用加载的是升级前的旧 DLL，\
         之后所有键放行给应用，重启这个应用即可恢复"
    ));
}

/// 闸合上了没有。合上后不再连 Server、不再吃键。
pub fn detected() -> bool {
    MISMATCHED.load(Ordering::SeqCst)
}
