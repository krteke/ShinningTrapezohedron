//! 把状态 watch 快照转换为浏览器可订阅的 SSE 事件流。

use axum::{
    Error,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use tokio_stream::{Stream, StreamExt, wrappers::WatchStream};

use crate::status::model::DeviceStatus;

use super::AppState;

enum StreamItem {
    Snapshot(DeviceStatus),
    Shutdown,
}

/// 新连接先收到当前快照，此后仅在采集器发布新快照时产生事件。
pub(super) async fn device_status(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Error>>> {
    let status = WatchStream::new(state.status.clone()).map(StreamItem::Snapshot);
    let shutdown = WatchStream::new(state.shutdown.subscribe())
        .filter_map(|requested| requested.then_some(StreamItem::Shutdown));
    let events = status
        .merge(shutdown)
        .take_while(|item| matches!(item, StreamItem::Snapshot(_)))
        .map(|item| {
            let StreamItem::Snapshot(snapshot) = item else {
                unreachable!("关闭事件不会穿过 take_while")
            };
            let revision = snapshot.revision.to_string();
            Event::default()
                .event("status")
                .id(revision)
                .json_data(snapshot)
        });

    Sse::new(events).keep_alive(KeepAlive::default())
}
