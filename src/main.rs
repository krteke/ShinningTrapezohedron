mod config;
mod status;
mod web;

use std::path::PathBuf;

use anyhow::{Context, Result};
use tokio::{net::TcpListener, signal};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::model::AppConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = required_config_path()?;
    let app_config = AppConfig::try_load(&config_path)?;
    init_tracing(&app_config.logging)?;

    let address = app_config.web.listen_address;
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("无法监听地址 {address}"))?;
    let (status_publisher, status_subscriber) =
        status::channel(status::model::DeviceStatus::default());
    let _status_collector =
        status::linux::spawn_collector(status_publisher, app_config.status.sample_interval());

    info!(%address, "Web 服务已启动");
    axum::serve(listener, web::router(status_subscriber))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Web 服务异常退出")
}

fn required_config_path() -> Result<PathBuf> {
    let mut args = std::env::args();
    let f = args.next();
    args.next().map(PathBuf::from).context(format!(
        "缺少配置文件路径；用法：{} <config.toml>",
        f.unwrap_or("shinning_trapezohedron".to_string())
    ))
}

fn init_tracing(config: &config::model::LoggingConfig) -> Result<()> {
    let filter = EnvFilter::try_new(&config.filter).context("日志过滤规则无效")?;
    tracing_subscriber::fmt()
        .with_ansi(config.ansi)
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
