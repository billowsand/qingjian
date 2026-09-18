//! Server 与 DLL 之间的传输。线上帧格式在 [`qingjian_platform::protocol`]（两端共用），
//! 这层只提供双工字节流上的消息循环（[`serve`]）与具体传输（命名管道 [`pipe`]）。

#[cfg(windows)]
pub mod pipe;
mod work;

pub use qingjian_platform::protocol::{CodecError, read_message, write_message};
pub use work::Work;

use std::io::{Read, Write};

use qingjian_platform::protocol::{ClientMessage, Incoming, read_incoming};

use crate::dispatch::Router;

/// 在一条已连上的双工流上服务一个客户端：读消息、交给 Router、写回，直到对端在帧边界关闭。
/// 读不懂的消息（比自己新的 DLL 发来的新变体）跳过接着读，与 [`pipe`] 里那条真连接一致。
pub fn serve<S: Read + Write>(stream: &mut S, router: &mut Router) -> Result<(), CodecError> {
    loop {
        match read_incoming::<_, ClientMessage>(&mut *stream)? {
            Incoming::Message(message) => {
                if let Some(response) = router.handle(message) {
                    write_message(&mut *stream, &response)?;
                }
            }
            Incoming::Unknown(reason) => {
                tracing::warn!(reason, "跳过一条读不懂的客户端消息（DLL 比 Server 新？）");
            }
            Incoming::Eof => return Ok(()),
        }
    }
}
