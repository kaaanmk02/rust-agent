use axum::{
    middleware::Next,
    response::Response,
};
use std::time::Duration;

pub async fn request_timeout(duration: Duration, req: axum::http::Request<axum::body::Body>, next: Next) -> Response {
    match tokio::time::timeout(duration, next.run(req)).await {
        Ok(response) => response,
        Err(_) => Response::builder()
            .status(axum::http::StatusCode::GATEWAY_TIMEOUT)
            .body(axum::body::Body::from("request timeout"))
            .unwrap(),
    }
}
