use std::collections::HashMap;
use axum::{
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    RequestPartsExt, Router,
};

fn home_router() -> Router {
    Router::new().route("/{version}/foo", get(handler))
}

async fn handler(version: Version) -> Html<String> {
    Html(format!("received request with version {version:?}"))
}

#[derive(Debug)]
enum Version {
    V1,
    V2,
    V3,
}

impl<S> FromRequestParts<S> for Version
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let params: Path<HashMap<String, String>> =
            parts.extract().await.map_err(IntoResponse::into_response)?;

        let version = params
            .get("version")
            .ok_or_else(|| (StatusCode::NOT_FOUND, "version param missing").into_response())?;

        match version.as_str() {
            "v1" => Ok(Version::V1),
            "v2" => Ok(Version::V2),
            "v3" => Ok(Version::V3),
            _ => Err((StatusCode::NOT_FOUND, "unknown version").into_response()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, http::StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_v1() {
        let response = home_router()
            .oneshot(
                Request::builder()
                    .uri("/v1/foo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(html, "received request with version V1");
    }

    #[tokio::test]
    async fn test_v4() {
        let response = home_router()
            .oneshot(
                Request::builder()
                    .uri("/v4/foo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(html, "unknown version");
    }
}