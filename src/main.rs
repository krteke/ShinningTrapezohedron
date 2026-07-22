mod status;
mod web;

use anyhow::{Context, Result};
use tokio::{net::TcpListener, signal};
use tracing::info;
use tracing_subscriber::EnvFilter;

const DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let address =
        std::env::var("SHINNING_LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN_ADDRESS.to_owned());
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("无法监听地址 {address}"))?;
    let (status_publisher, status_subscriber) =
        status::channel(status::model::DeviceStatus::default());
    let _status_collector = status::linux::spawn_collector(status_publisher);

    info!(%address, "Web 服务已启动");
    axum::serve(listener, web::router(status_subscriber))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Web 服务异常退出")
}

fn init_tracing() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(|error| anyhow::anyhow!("初始化日志系统失败: {error}"))
}

async fn shutdown_signal() {
    tokio::select! {
        _ = signal::ctrl_c() => {}
        _ = terminate_signal() => {}
    }
    info!("收到退出信号，正在停止 Web 服务");
}

#[cfg(unix)]
async fn terminate_signal() {
    let mut signal =
        signal::unix::signal(signal::unix::SignalKind::terminate()).expect("无法监听 SIGTERM");
    signal.recv().await;
}

#[cfg(not(unix))]
async fn terminate_signal() {
    std::future::pending::<()>().await;
}
