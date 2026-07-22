use serde::Serialize;
use tokio::sync::watch;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub revision: u64,
    pub collected_at_unix_ms: Option<u64>,
}

pub type StatusPublisher = watch::Sender<DeviceStatus>;
pub type StatusSubscriber = watch::Receiver<DeviceStatus>;

pub fn channel(initial: DeviceStatus) -> (StatusPublisher, StatusSubscriber) {
    watch::channel(initial)
}

#[cfg(test)]
mod tests {
    use super::{DeviceStatus, channel};

    #[tokio::test]
    async fn published_snapshot_reaches_subscriber() {
        let (publisher, mut subscriber) = channel(DeviceStatus::default());
        let snapshot = DeviceStatus {
            revision: 1,
            collected_at_unix_ms: Some(42),
        };

        publisher.send_replace(snapshot.clone());
        subscriber.changed().await.unwrap();

        assert_eq!(*subscriber.borrow_and_update(), snapshot);
    }
}
