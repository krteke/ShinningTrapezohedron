//! 可供运行时和 WebUI 共用的配置模型。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr, num::NonZeroU64, path::Path, time::Duration};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub web: WebConfig,
    pub status: StatusConfig,
    pub logging: LoggingConfig,
}

impl AppConfig {
    /// 从指定 TOML 文件加载完整配置。
    pub fn try_load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("无法读取配置文件 {}", path.display()))?;
        parse(&content).with_context(|| format!("配置文件 {} 格式错误", path.display()))
    }
}

fn parse(content: &str) -> Result<AppConfig, toml::de::Error> {
    toml::from_str(content)
}

/// Web 服务监听配置。
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WebConfig {
    pub listen_address: SocketAddr,
}

/// 周期状态采集配置。
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatusConfig {
    /// 非零类型确保错误配置不会导致无间隔的采集循环。
    pub sample_interval_seconds: NonZeroU64,
}

impl StatusConfig {
    pub fn sample_interval(&self) -> Duration {
        Duration::from_secs(self.sample_interval_seconds.get())
    }
}

/// tracing 日志输出配置。
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    /// tracing 指令，例如 `info` 或 `warn,tower_http=info`。
    pub filter: String,
    /// 是否输出 ANSI 终端控制字符，写入 systemd journal 时通常关闭。
    pub ansi: bool,
}

#[cfg(test)]
mod tests {
    use crate::config::model::parse;

    const VALID_CONFIG: &str = r#"
[web]
listen_address = "127.0.0.1:3000"

[status]
sample_interval_seconds = 2

[logging]
filter = "info"
ansi = false
"#;

    #[test]
    fn config_can_round_trip_through_toml() {
        let config = parse(VALID_CONFIG).unwrap();
        let encoded = toml::to_string_pretty(&config).unwrap();

        assert_eq!(parse(&encoded).unwrap(), config);
    }

    #[test]
    fn zero_sample_interval_is_rejected() {
        let invalid =
            VALID_CONFIG.replace("sample_interval_seconds = 2", "sample_interval_seconds = 0");
        assert!(parse(&invalid).is_err());
    }

    #[test]
    fn unknown_field_is_rejected() {
        let invalid = VALID_CONFIG.replace("ansi = false", "ansi = false\nunknown = true");
        assert!(parse(&invalid).is_err());
    }
}
