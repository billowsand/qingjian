//! [`read_incoming`](super::read_incoming) 读到的一条消息。

/// 读一帧的结果。比 [`read_message`](super::read_message) 多出 [`Unknown`](Self::Unknown) 这一档：
/// 帧已经完整读走了，只是内容读不懂（对端比自己新，加了不认识的消息变体），连接还能接着用。
///
/// 变体新增不在「缺字段退默认值」的保护范围里（见 [`PROTOCOL_VERSION`](crate::protocol::PROTOCOL_VERSION)），
/// 这一档就是给那种情况兜底的：收的一方跳过这条消息，而不是把连接断掉、让对面每键重连。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Incoming<T> {
    /// 读到一条认得的消息。
    Message(T),

    /// 读到一条读不懂的消息；里面是解析错误的文字，记日志用。
    Unknown(String),

    /// 对端在帧边界干净关闭了连接。
    Eof,
}
