mod api;

use std::borrow::Cow;

use axum::{
    Router,
    body::Body,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::get,
};
use rust_embed::{Embed, EmbeddedFile};
use tower_http::trace::TraceLayer;

use crate::{config::manager::ConfigManager, status::StatusSubscriber};

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/frontend/build/"]
struct FrontendAssets;

#[derive(Clone)]
struct AppState {
    status: StatusSubscriber,
    config: ConfigManager,
}

pub fn router(status: StatusSubscriber, config: ConfigManager) -> Router {
    Router::new()
        .nest("/api", api::router())
        .fallback_service(get(frontend))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { status, config })
}

async fn frontend(uri: Uri) -> Response {
    let requested_path = uri.path().trim_start_matches('/');
    let path = if requested_path.is_empty() {
        "index.html"
    } else {
        requested_path
    };

    if let Some(file) = FrontendAssets::get(path) {
        return asset_response(path, file);
    }

    // SvelteKit 的静态适配器会把 /settings/status 生成为 settings/status.html。
    let page_path = format!("{}.html", path.trim_end_matches('/'));
    if !path.contains('.')
        && let Some(file) = FrontendAssets::get(&page_path)
    {
        return asset_response(&page_path, file);
    }

    if !path.starts_with("_app/")
        && !path.contains('.')
        && let Some(index) = FrontendAssets::get("index.html")
    {
        return asset_response("index.html", index);
    }

    StatusCode::NOT_FOUND.into_response()
}

fn asset_response(path: &str, file: EmbeddedFile) -> Response {
    let cache_control = if path.ends_with(".html") {
        "no-cache"
    } else if path.starts_with("_app/immutable/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    };
    let response = Response::builder()
        .header(header::CONTENT_TYPE, file.metadata.mimetype())
        .header(header::CACHE_CONTROL, cache_control);
    let body = match file.data {
        Cow::Borrowed(bytes) => Body::from(bytes),
        Cow::Owned(bytes) => Body::from(bytes),
    };

    response.body(body).expect("无法构造静态资源响应")
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode, header},
        response::Response,
    };
    use serde_json::{Value, json};
    use tempfile::{TempDir, tempdir};
    use tower::ServiceExt;

    use super::router;
    use crate::{
        config::{manager, model::AppConfig, test_config},
        status::{channel, model::DeviceStatus},
    };

    fn test_router() -> (axum::Router, TempDir) {
        test_router_with_status(DeviceStatus::default())
    }

    fn test_router_with_status(status: DeviceStatus) -> (axum::Router, TempDir) {
        let directory = tempdir().unwrap();
        let config = manager::spawn(directory.path().join("config.toml"), test_config(2));
        let (_, subscriber) = channel(status);
        (router(subscriber, config), directory)
    }

    async fn request(uri: &str) -> Response {
        let (app, _directory) = test_router();
        app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    fn replace_request(config: &AppConfig) -> Request<Body> {
        Request::builder()
            .method(Method::PUT)
            .uri("/api/config")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(config).unwrap()))
            .unwrap()
    }

    async fn json_body(response: Response) -> Value {
        let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let response = request("/api/health").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(json_body(response).await, json!({ "status": "ok" }));
    }

    #[tokio::test]
    async fn status_returns_current_snapshot() {
        let snapshot = DeviceStatus {
            revision: 7,
            collected_at_unix_ms: Some(1234),
            system: None,
        };
        let (app, _directory) = test_router_with_status(snapshot);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await,
            json!({ "revision": 7, "collectedAtUnixMs": 1234, "system": null })
        );
    }

    #[tokio::test]
    async fn config_returns_snapshot_and_apply_modes() {
        let response = request("/api/config").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await,
            json!({
                "config": {
                    "web": { "listen_address": "127.0.0.1:3000" },
                    "status": { "sample_interval_seconds": 2 },
                    "logging": { "filter": "info", "ansi": false }
                },
                "hotAppliedFields": ["status.sample_interval_seconds"],
                "restartRequiredFields": [
                    "web.listen_address", "logging.filter", "logging.ansi"
                ]
            })
        );
    }

    #[tokio::test]
    async fn config_replace_persists_candidate() {
        let (app, directory) = test_router();
        let candidate = test_config(5);
        let response = app.oneshot(replace_request(&candidate)).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await["config"]["status"]["sample_interval_seconds"],
            5
        );
        assert_eq!(
            AppConfig::try_load(&directory.path().join("config.toml")).unwrap(),
            candidate
        );
    }

    #[tokio::test]
    async fn invalid_config_returns_json_error_without_persisting() {
        let (app, directory) = test_router();
        let mut candidate = test_config(5);
        candidate.logging.filter = "info[".to_owned();
        let response = app.oneshot(replace_request(&candidate)).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            json_body(response).await,
            json!({ "code": "invalid_config", "message": "候选配置无效" })
        );
        assert!(!directory.path().join("config.toml").exists());
    }

    #[tokio::test]
    async fn persistence_failure_returns_json_error() {
        let directory = tempdir().unwrap();
        let config = manager::spawn(directory.path().join("missing/config.toml"), test_config(2));
        let (_, status) = channel(DeviceStatus::default());
        let app = router(status, config);
        let response = app.oneshot(replace_request(&test_config(5))).await.unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            json_body(response).await,
            json!({
                "code": "config_persistence_failed",
                "message": "配置保存失败"
            })
        );
    }

    #[tokio::test]
    async fn malformed_config_body_returns_json_error() {
        let (app, _directory) = test_router();
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/api/config")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{"))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            json_body(response).await,
            json!({
                "code": "invalid_request",
                "message": "请求体不是有效的完整配置"
            })
        );
    }

    #[tokio::test]
    async fn root_returns_embedded_html() {
        let response = request("/").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html");
        let body = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("<!doctype html>"));
    }

    #[tokio::test]
    async fn prerendered_path_returns_matching_html() {
        let response = request("/settings/status").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
        let body = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("<title>状态采样</title>"));
    }

    #[tokio::test]
    async fn unknown_spa_path_returns_index() {
        let response = request("/network/settings").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
        let body = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("/_app/immutable/"));
    }

    #[tokio::test]
    async fn unknown_api_returns_json_404() {
        let response = request("/api/unknown").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            json_body(response).await,
            json!({ "code": "not_found", "message": "接口不存在" })
        );
    }

    #[tokio::test]
    async fn unsupported_api_method_returns_json_405() {
        let (app, _directory) = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(
            json_body(response).await,
            json!({
                "code": "method_not_allowed",
                "message": "HTTP 方法不受支持"
            })
        );
    }

    #[tokio::test]
    async fn missing_asset_returns_404() {
        let response = request("/_app/missing.js").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
