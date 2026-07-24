//! 在操作系统信号、Web 服务和长连接之间发布统一关闭通知。

use tokio::sync::watch;

/// 可克隆的关闭令牌；内部发送端保证订阅通道在进程退出前保持有效。
#[derive(Clone)]
pub struct ShutdownToken {
    publisher: watch::Sender<bool>,
}

impl ShutdownToken {
    pub fn new() -> Self {
        Self {
            publisher: watch::Sender::new(false),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.publisher.subscribe()
    }

    pub fn request(&self) {
        self.publisher.send_replace(true);
    }
}
