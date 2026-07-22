//! Web、OLED 和状态采集器共同使用的只读快照模型。

use serde::Serialize;

/// 设备状态快照。
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    /// 每次成功采集后递增，用于识别是否收到新快照。
    pub revision: u64,
    /// Unix 时间戳；系统时钟无效时为 `None`。
    pub collected_at_unix_ms: Option<u64>,
    /// 首次成功读取 `/proc` 前为 `None`。
    pub system: Option<SystemStatus>,
}

/// 从 Linux 系统接口读取的基础运行状态。
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub uptime_secs: u64,
    pub load_avg: LoadAvg,
    pub memory: MemoryStatus,
}

/// 系统平均负载。
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadAvg {
    pub one_minute: f32,
    pub five_minutes: f32,
    pub fifteen_minutes: f32,
}

/// 统一使用字节表示的物理内存状态。
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStatus {
    pub total_bytes: u64,
    /// 内核估算的可供新程序使用的内存，不等同于空闲内存。
    pub available_bytes: u64,
    pub used_bytes: u64,
}
