//! 连 Server：激活 / 获焦时开会话，失败按 [`RECONNECT_INTERVAL`] 退避重试；转发出错就断开、下一键重连。
//! 管道不在（Server 还没起）时顺手把 Server 拉起来，见 [`launch`](crate::client::launch)。

use std::time::{Duration, Instant};

use qingjian_platform::protocol::SessionId;

use super::{
    LAUNCH_INTERVAL, LAUNCH_POLL, LAUNCH_WAIT, RECONNECT_AFTER_LAUNCH, RECONNECT_INTERVAL,
    TextService_Impl,
};
use crate::client::pipe::{PipeStream, connect_default};
use crate::client::{EngineClient, launch, mismatch};
use crate::com::log::log;

impl TextService_Impl {
    /// 连 Server 并开会话（会话 id 用 TSF 的 client id，带上宿主 exe 名）。
    pub(super) fn connect(&self) {
        let session = SessionId(self.client_id.get() as u64);
        let app = crate::com::host_app_name();
        let connected = self
            .open_pipe()
            .map_err(|e| e.to_string())
            .and_then(|stream| EngineClient::open(stream, session, app).map_err(|e| e.to_string()));
        match connected {
            Ok(client) => {
                *self.engine.borrow_mut() = Some(client);
                self.last_connect_failure.set(None);
                log("已连上 Server");
            }
            Err(error) => {
                self.last_connect_failure.set(Some(Instant::now()));
                log(&format!(
                    "连 Server 失败（qingjian-server 没起？）: {error}"
                ));
            }
        }
    }

    /// 开管道；不在就说明 Server 没起（开机时「启动」文件夹要等 Explorer 放行、升级安装后也有一段空窗），
    /// 拉它一把，再轮询到管道出现。拉起本身有节流，别每次退避到点都去拉。
    fn open_pipe(&self) -> std::io::Result<PipeStream> {
        let error = match connect_default() {
            Ok(stream) => return Ok(stream),
            Err(error) => error,
        };
        if error.kind() != std::io::ErrorKind::NotFound || !self.may_launch() {
            return Err(error);
        }
        self.last_launch.set(Some(Instant::now()));
        launch::launch_server();
        self.connect_until(LAUNCH_WAIT)
    }

    /// 反复试到管道出现或超时。Server 从进程起到管道就绪实测约 120 ms，所以这么等一下就能接上，
    /// 用户按下的那一键不至于白按。这一等在应用的 UI 线程上，所以上限要小（等久了 TSF 看门狗会切走输入法）；
    /// 超时也不要紧，接下来几次按键还会按短退避重连（见 [`ensure_connected`](Self::ensure_connected)）。
    fn connect_until(&self, budget: Duration) -> std::io::Result<PipeStream> {
        let deadline = Instant::now() + budget;
        loop {
            std::thread::sleep(LAUNCH_POLL);
            let error = match connect_default() {
                Ok(stream) => return Ok(stream),
                Err(error) => error,
            };
            if Instant::now() >= deadline {
                return Err(error);
            }
        }
    }

    /// 距上次拉 Server 够久了没有。刚拉过就别再拉——它可能正在起。
    fn may_launch(&self) -> bool {
        self.last_launch
            .get()
            .is_none_or(|at| at.elapsed() >= LAUNCH_INTERVAL)
    }

    /// 没连上就重连一次（距上次失败不到退避间隔则跳过）。返回此刻是否连着。
    /// 协议对不上时一律不连：本进程加载的旧 DLL 换不掉，重连只会再失败一次（见 [`mismatch`]）。
    pub(super) fn ensure_connected(&self) -> bool {
        if mismatch::detected() {
            return false;
        }
        if self.engine.borrow().is_some() {
            return true;
        }
        let recently_failed = self
            .last_connect_failure
            .get()
            .is_some_and(|at| at.elapsed() < self.reconnect_interval());
        if recently_failed {
            return false;
        }
        self.connect();
        self.engine.borrow().is_some()
    }

    /// 退避间隔。刚拉起过 Server 就用短的：它正在起来的路上（磁盘忙时比那 120 ms 久得多），
    /// 用常规的两秒会让用户白敲好几键。
    fn reconnect_interval(&self) -> Duration {
        match self.last_launch.get() {
            Some(at) if at.elapsed() < LAUNCH_INTERVAL => RECONNECT_AFTER_LAUNCH,
            _ => RECONNECT_INTERVAL,
        }
    }

    /// 转发失败后断开，下一键重连。
    pub(super) fn disconnect(&self) {
        *self.engine.borrow_mut() = None;
        self.last_connect_failure.set(None);
        self.shared.end_composing();
    }
}
