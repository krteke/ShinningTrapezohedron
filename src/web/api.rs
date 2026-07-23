//! 粗粒度 JSON API 的路由、响应和错误映射。

use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use tracing::{error, warn};

use crate::{
    config::{manager::ConfigError, model::AppConfig},
    status::model::DeviceStatus,
};

use super::AppState;

// 字段路径与 TOML 保持一致，供 WebUI 准确标记生效时机。
const HOT_APPLIED_FIELDS: &[&str] = &["status.sample_interval_seconds"];
const RESTART_REQUIRED_FIELDS: &[&str] = &["web.listen_address", "logging.filter", "logging.ansi"];

#[derive(Serialize)]
struct HealthBody {
    status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigBody {
    config: AppConfig,
    hot_applied_fields: &'static [&'static str],
    restart_required_fields: &'static [&'static str],
}

impl ConfigBody {
    fn new(config: AppConfig) -> Self {
        Self {
            config,
            hot_applied_fields: HOT_APPLIED_FIELDS,
            restart_required_fields: RESTART_REQUIRED_FIELDS,
        }
    }
}

#[derive(Serialize)]
struct ApiErrorBody {
    code: &'static str,
    message: &'static str,
}

struct ApiError {
    status: StatusCode,
    body: ApiErrorBody,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self {
            status,
            body: ApiErrorBody { code, message },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl From<ConfigError> for ApiError {
    fn from(error: ConfigError) -> Self {
        match error {
            ConfigError::Invalid(source) => {
                warn!(%source, "拒绝无效的候选配置");
                Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_config",
                    "候选配置无效",
                )
            }
            ConfigError::Persistence(source) => {
                error!(%source, "持久化候选配置失败");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "config_persistence_failed",
                    "配置保存失败",
                )
            }
            ConfigError::Unavailable => {
                error!("配置管理器不可用");
                Self::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "config_manager_unavailable",
                    "配置管理器暂不可用",
                )
            }
        }
    }
}

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/status", get(device_status))
        .route("/config", get(get_config).put(replace_config))
        .fallback(api_not_found)
        .method_not_allowed_fallback(method_not_allowed)
}

async fn health() -> Json<HealthBody> {
    Json(HealthBody { status: "ok" })
}

async fn device_status(State(state): State<AppState>) -> Json<DeviceStatus> {
    Json(state.status.borrow().clone())
}

async fn get_config(State(state): State<AppState>) -> Json<ConfigBody> {
    Json(ConfigBody::new(state.config.current()))
}

async fn replace_config(
    State(state): State<AppState>,
    payload: Result<Json<AppConfig>, JsonRejection>,
) -> Result<Json<ConfigBody>, ApiError> {
    let Json(candidate) = payload.map_err(|rejection| {
        let status = rejection.status();
        warn!(%rejection, "配置请求体无效");
        ApiError::new(status, "invalid_request", "请求体不是有效的完整配置")
    })?;
    let response = ConfigBody::new(candidate.clone());
    state.config.replace(candidate).await?;
    Ok(Json(response))
}

async fn api_not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "not_found", "接口不存在")
}

async fn method_not_allowed() -> ApiError {
    ApiError::new(
        StatusCode::METHOD_NOT_ALLOWED,
        "method_not_allowed",
        "HTTP 方法不受支持",
    )
}
