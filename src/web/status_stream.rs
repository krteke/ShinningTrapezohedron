//! 把状态 watch 快照转换为浏览器可订阅的 SSE 事件流。

use axum::{
    Error,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use tokio_stream::{Stream, StreamExt, wrappers::WatchStream};

use super::AppState;

/// 新连接先收到当前快照，此后仅在采集器发布新快照时产生事件。
pub(super) async fn device_status(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Error>>> {
    let events = WatchStream::new(state.status.clone()).map(|snapshot| {
        let revision = snapshot.revision.to_string();
        Event::default()
            .event("status")
            .id(revision)
            .json_data(snapshot)
    });

    Sse::new(events).keep_alive(KeepAlive::default())
}
