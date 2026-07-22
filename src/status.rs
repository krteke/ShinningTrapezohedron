//! 设备状态的数据模型、发布通道与平台采集实现。

pub mod linux;
pub mod model;

use tokio::sync::watch;

use self::model::DeviceStatus;

/// 状态采集器持有的发布端，始终用完整快照替换旧值。
pub type StatusPublisher = watch::Sender<DeviceStatus>;

/// Web 和 OLED 各自持有的订阅端。
pub type StatusSubscriber = watch::Receiver<DeviceStatus>;

/// 创建保存最新完整快照的 watch 通道。
pub fn channel(initial: DeviceStatus) -> (StatusPublisher, StatusSubscriber) {
    watch::channel(initial)
}

#[cfg(test)]
mod tests {
    use super::{channel, model::DeviceStatus};

    #[tokio::test]
    async fn published_snapshot_reaches_subscriber() {
        let (publisher, mut subscriber) = channel(DeviceStatus::default());
        let snapshot = DeviceStatus {
            revision: 1,
            collected_at_unix_ms: Some(42),
            system: None,
        };

        publisher.send_replace(snapshot.clone());
        subscriber.changed().await.unwrap();

        assert_eq!(*subscriber.borrow_and_update(), snapshot);
    }
}
