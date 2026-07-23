//! 配置命令的串行执行器。

use std::path::{Path, PathBuf};

use anyhow::Error as AnyError;
use thiserror::Error;
use tokio::{
    sync::{mpsc, oneshot, watch},
    task,
};

use super::{ConfigSubscriber, model::AppConfig, save};

// 配置操作频率很低，小型有界队列可以在 WebUI 异常请求时提供背压。
const COMMAND_BUFFER_CAPACITY: usize = 8;

/// 所有配置变更都必须通过的明确命令。
enum ConfigCommand {
    Replace {
        candidate: AppConfig,
        // 单次响应通道把实际持久化结果返回给当前请求。
        result_tx: oneshot::Sender<Result<(), ConfigError>>,
    },
}

/// 供 Web 层映射 HTTP 状态的配置命令错误。
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("候选配置无效: {0}")]
    Invalid(AnyError),
    #[error("配置持久化失败: {0}")]
    Persistence(AnyError),
    #[error("配置管理器已停止")]
    Unavailable,
}

/// 配置 actor 的可克隆句柄，同时提供命令入口和快照订阅。
#[derive(Clone)]
pub struct ConfigManager {
    command_tx: mpsc::Sender<ConfigCommand>,
    subscriber: ConfigSubscriber,
}

impl ConfigManager {
    pub fn subscribe(&self) -> ConfigSubscriber {
        self.subscriber.clone()
    }

    /// 读取最新完整快照。
    pub fn current(&self) -> AppConfig {
        self.subscriber.borrow().clone()
    }

    /// 等待候选配置持久化完成；失败时不会发布新快照。
    pub async fn replace(&self, candidate: AppConfig) -> Result<(), ConfigError> {
        let (result_tx, result_rx) = oneshot::channel();
        self.command_tx
            .send(ConfigCommand::Replace {
                candidate,
                result_tx,
            })
            .await
            .map_err(|_| ConfigError::Unavailable)?;
        result_rx.await.map_err(|_| ConfigError::Unavailable)?
    }
}

pub fn spawn(path: PathBuf, initial: AppConfig) -> ConfigManager {
    let (command_tx, mut command_rx) = mpsc::channel(COMMAND_BUFFER_CAPACITY);
    let (publisher, subscriber) = watch::channel(initial);

    tokio::spawn(async move {
        while let Some(ConfigCommand::Replace {
            candidate,
            result_tx,
        }) = command_rx.recv().await
        {
            let result = persist_candidate(&path, &candidate).await;
            if result.is_ok() {
                publisher.send_replace(candidate);
            }
            let _ = result_tx.send(result);
        }
    });

    ConfigManager {
        command_tx,
        subscriber,
    }
}

async fn persist_candidate(path: &Path, candidate: &AppConfig) -> Result<(), ConfigError> {
    candidate.validate().map_err(ConfigError::Invalid)?;
    let path = path.to_owned();
    let candidate = candidate.clone();
    task::spawn_blocking(move || save(&path, &candidate))
        .await
        .map_err(|error| ConfigError::Persistence(error.into()))?
        .map_err(ConfigError::Persistence)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::spawn;
    use crate::config::{model::AppConfig, save, test_config};

    #[tokio::test]
    async fn replacement_is_persisted_before_publication() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let initial = test_config(2);
        save(&path, &initial).unwrap();
        let manager = spawn(path.clone(), initial);
        let mut subscriber = manager.subscribe();
        let candidate = test_config(5);

        manager.replace(candidate.clone()).await.unwrap();
        subscriber.changed().await.unwrap();

        assert_eq!(*subscriber.borrow_and_update(), candidate);
        assert_eq!(AppConfig::try_load(&path).unwrap(), candidate);
    }

    #[tokio::test]
    async fn failed_persistence_keeps_previous_snapshot() {
        let directory = tempdir().unwrap();
        let initial = test_config(2);
        let manager = spawn(
            directory.path().join("missing/config.toml"),
            initial.clone(),
        );
        let subscriber = manager.subscribe();

        assert!(manager.replace(test_config(5)).await.is_err());
        assert!(!subscriber.has_changed().unwrap());
        assert_eq!(*subscriber.borrow(), initial);
    }
}
