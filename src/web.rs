use std::borrow::Cow;

use axum::{
    Json, Router,
    body::Body,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::get,
};
use rust_embed::{Embed, EmbeddedFile};
use serde::Serialize;
use tower_http::trace::TraceLayer;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/frontend/build/"]
struct FrontendAssets;

#[derive(Serialize)]
struct HealthBody {
    status: &'static str,
}

#[derive(Serialize)]
struct ApiErrorBody {
    code: &'static str,
    message: &'static str,
}

pub(crate) fn router() -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .fallback(api_not_found);

    Router::new()
        .nest("/api", api)
        .fallback_service(get(frontend))
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<HealthBody> {
    Json(HealthBody { status: "ok" })
}

async fn api_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody {
            code: "not_found",
            message: "接口不存在",
        }),
    )
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

    if !path.starts_with("_app/") && !path.contains('.') {
        if let Some(index) = FrontendAssets::get("index.html") {
            return asset_response("index.html", index);
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

fn asset_response(path: &str, file: EmbeddedFile) -> Response {
    let cache_control = if path == "index.html" {
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
        http::{Request, StatusCode, header},
        response::Response,
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::router;

    async fn request(uri: &str) -> Response {
        router()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
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
    async fn root_returns_embedded_html() {
        let response = request("/").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html");
        let body = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("<!doctype html>"));
    }

    #[tokio::test]
    async fn spa_path_returns_index() {
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
    async fn missing_asset_returns_404() {
        let response = request("/_app/missing.js").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
