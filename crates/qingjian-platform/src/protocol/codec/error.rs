//! 帧编解码的错误类型。

use std::io;

/// 编解码错误。
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("io: {0}")]
    Io(#[from] io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("frame too large: {0} bytes")]
    TooLarge(usize),
}

impl CodecError {
    /// 是不是「帧收全了但读不懂」——对端的协议与自己对不上（多半是升级后旧 DLL 还留在没重启的
    /// 应用里）。IO 错误不算：那只是连接断了，重连就好。
    pub fn is_protocol_mismatch(&self) -> bool {
        matches!(self, Self::Json(_))
    }
}
