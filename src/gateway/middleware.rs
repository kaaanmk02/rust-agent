use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let expected = std::env::var("GATEWAY_AUTH_TOKEN").ok();
    let provided = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| {
            req.uri()
                .query()
                .and_then(|q| q.split('&').find(|p| p.starts_with("token="))?.strip_prefix("token="))
        });

    match (expected, provided) {
        (Some(expected), Some(provided)) if provided == expected => Ok(next.run(req).await),
        (Some(_), _) => Err(axum::http::StatusCode::UNAUTHORIZED),
        (None, _) => Ok(next.run(req).await),
    }
}
