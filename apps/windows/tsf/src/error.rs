use qingjian_platform::protocol::CodecError;

/// 与 Server 通信时的错误。
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// 帧编解码 / 底层 IO 出错。
    #[error("codec: {0}")]
    Codec(#[from] CodecError),

    /// 等应答时对端在帧边界关闭了连接。
    #[error("server closed the connection")]
    Closed,

    /// 收到了与当前请求不匹配的消息。
    #[error("unexpected server message: {0}")]
    Unexpected(&'static str),
}

impl ClientError {
    /// 是不是「说的话对面听不懂」——本 DLL 与 Server 的协议对不上（升级安装后本进程还加载着旧 DLL）。
    /// 重连没有用：DLL 换不掉，得重启这个应用。连接断了 / Server 没起不算，那些重连就好。
    pub fn is_protocol_mismatch(&self) -> bool {
        match self {
            Self::Codec(error) => error.is_protocol_mismatch(),
            Self::Unexpected(_) => true,
            Self::Closed => false,
        }
    }
}
