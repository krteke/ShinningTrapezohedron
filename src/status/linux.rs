//! 基于 Linux `/proc` 的周期状态采集器。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use procfs::{Current, LoadAverage, Meminfo, Uptime};
use tokio::{
    task::{self, JoinHandle},
    time::{self, MissedTickBehavior},
};
use tracing::warn;

use crate::config::ConfigSubscriber;

use super::{
    StatusPublisher,
    model::{DeviceStatus, LoadAvg, MemoryStatus, SystemStatus},
};

/// 启动采集任务，并在采样周期变化时重建定时器。
pub fn spawn_collector(publisher: StatusPublisher, mut config: ConfigSubscriber) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut sample_interval = config.borrow_and_update().status.sample_interval();
        let mut interval = configured_interval(sample_interval);
        let mut config_open = true;

        loop {
            tokio::select! {
                _ = interval.tick() => match task::spawn_blocking(read_system_status).await {
                    Ok(Ok(system)) => publish_system_status(&publisher, system),
                    Ok(Err(error)) => {
                        // 失败时保留上一次完整快照，避免发布字段不全的状态。
                        warn!(%error, "读取 /proc 系统状态失败");
                    }
                    Err(error) => warn!(%error, "系统状态采集任务异常退出"),
                },
                changed = config.changed(), if config_open => {
                    if changed.is_ok() {
                        let next_interval = config.borrow_and_update().status.sample_interval();
                        if next_interval != sample_interval {
                            sample_interval = next_interval;
                            interval = configured_interval(sample_interval);
                        }
                    } else {
                        // 配置 actor 异常停止时，采集器继续使用最后一次有效配置。
                        config_open = false;
                        warn!("配置快照通道已关闭，继续使用当前采样周期");
                    }
                }
            }
        }
    })
}

fn configured_interval(sample_interval: Duration) -> time::Interval {
    let mut interval = time::interval(sample_interval);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    interval
}

fn read_system_status() -> procfs::ProcResult<SystemStatus> {
    let uptime = Uptime::current()?;
    let load = LoadAverage::current()?;
    let memory = Meminfo::current()?;
    let available = memory
        .mem_available
        // 老内核缺少 MemAvailable 时，使用 free(1) 常见的保守近似值。
        .unwrap_or_else(|| {
            memory
                .mem_free
                .saturating_add(memory.buffers)
                .saturating_add(memory.cached)
        })
        .min(memory.mem_total);

    Ok(SystemStatus {
        uptime_secs: uptime.uptime_duration().as_secs(),
        load_avg: LoadAvg {
            one_minute: load.one,
            five_minutes: load.five,
            fifteen_minutes: load.fifteen,
        },
        memory: MemoryStatus {
            total_bytes: memory.mem_total,
            available_bytes: available,
            used_bytes: memory.mem_total.saturating_sub(available),
        },
    })
}

fn publish_system_status(publisher: &StatusPublisher, system: SystemStatus) {
    let revision = publisher.borrow().revision.saturating_add(1);
    publisher.send_replace(DeviceStatus {
        revision,
        collected_at_unix_ms: unix_timestamp_ms(),
        system: Some(system),
    });
}

fn unix_timestamp_ms() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::watch;
    use tokio::time::timeout;

    use super::spawn_collector;
    use crate::{
        config::test_config,
        status::{channel, model::DeviceStatus},
    };

    #[tokio::test]
    async fn collector_publishes_linux_snapshot() {
        let (publisher, mut subscriber) = channel(DeviceStatus::default());
        let (_, config) = watch::channel(test_config(60));
        let task = spawn_collector(publisher, config);

        timeout(Duration::from_secs(5), subscriber.changed())
            .await
            .unwrap()
            .unwrap();
        task.abort();

        let snapshot = subscriber.borrow();
        assert_eq!(snapshot.revision, 1);
        assert!(snapshot.collected_at_unix_ms.is_some());
        assert!(snapshot.system.as_ref().unwrap().memory.total_bytes > 0);
    }

    #[tokio::test]
    async fn collector_restarts_interval_after_config_change() {
        let (publisher, mut subscriber) = channel(DeviceStatus::default());
        let (config_tx, config_rx) = watch::channel(test_config(60));
        let task = spawn_collector(publisher, config_rx);

        timeout(Duration::from_secs(5), subscriber.changed())
            .await
            .unwrap()
            .unwrap();
        config_tx.send_replace(test_config(1));
        timeout(Duration::from_secs(1), subscriber.changed())
            .await
            .unwrap()
            .unwrap();
        task.abort();

        assert_eq!(subscriber.borrow().revision, 2);
    }
}
