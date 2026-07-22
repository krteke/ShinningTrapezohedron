//! 基于 Linux `/proc` 的周期状态采集器。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use procfs::{Current, LoadAverage, Meminfo, Uptime};
use tokio::{
    task::{self, JoinHandle},
    time::{self, MissedTickBehavior},
};
use tracing::warn;

use super::{
    StatusPublisher,
    model::{DeviceStatus, LoadAvg, MemoryStatus, SystemStatus},
};

/// 按配置的时间间隔启动采集任务；返回句柄用于随主服务生命周期保留任务。
pub fn spawn_collector(publisher: StatusPublisher, sample_interval: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = time::interval(sample_interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            match task::spawn_blocking(read_system_status).await {
                Ok(Ok(system)) => publish_system_status(&publisher, system),
                Ok(Err(error)) => {
                    // 失败时保留上一次完整快照，避免发布字段不全的状态。
                    warn!(%error, "读取 /proc 系统状态失败");
                }
                Err(error) => warn!(%error, "系统状态采集任务异常退出"),
            }
        }
    })
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

    use tokio::time::timeout;

    use super::spawn_collector;
    use crate::status::{channel, model::DeviceStatus};

    #[tokio::test]
    async fn collector_publishes_linux_snapshot() {
        let (publisher, mut subscriber) = channel(DeviceStatus::default());
        let task = spawn_collector(publisher, Duration::from_millis(10));

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
}
