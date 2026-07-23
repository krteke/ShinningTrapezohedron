//! 应用配置文件的加载、持久化与命令入口。

pub mod manager;
pub mod model;

use std::{fs::File, io::Write, path::Path};

use anyhow::{Context, Result};
use tempfile::NamedTempFile;
use tokio::sync::watch;

use self::model::AppConfig;

/// 读取最新完整快照。
pub type ConfigSubscriber = watch::Receiver<AppConfig>;

/// 在目标文件同目录写入临时文件，同步后原子替换旧配置。
fn save(path: &Path, config: &AppConfig) -> Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let content = toml::to_string_pretty(config).context("无法序列化配置")?;
    let mut temporary = NamedTempFile::new_in(parent)
        .with_context(|| format!("无法在 {} 创建临时配置", parent.display()))?;
    temporary
        .write_all(content.as_bytes())
        .context("无法写入临时配置")?;
    temporary.as_file().sync_all().context("无法同步临时配置")?;
    temporary
        .persist(path)
        .with_context(|| format!("无法替换配置文件 {}", path.display()))?;

    if let Err(error) = File::open(parent).and_then(|directory| directory.sync_all()) {
        tracing::warn!(%error, "无法同步配置目录");
    }
    Ok(())
}

#[cfg(test)]
pub fn test_config(interval_seconds: u64) -> AppConfig {
    use std::{net::SocketAddr, num::NonZeroU64};

    use self::model::{LoggingConfig, StatusConfig, WebConfig};

    AppConfig {
        web: WebConfig {
            listen_address: SocketAddr::from(([127, 0, 0, 1], 3000)),
        },
        status: StatusConfig {
            sample_interval_seconds: NonZeroU64::new(interval_seconds).unwrap(),
        },
        logging: LoggingConfig {
            filter: "info".to_owned(),
            ansi: false,
        },
    }
}
