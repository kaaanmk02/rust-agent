use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

pub async fn auth_middleware(req: Request<axum::body::Body>, next: Next) -> Response {
    let skip = req.uri().path().starts_with("/health") || req.uri().path().starts_with("/ready") || req.uri().path().starts_with("/version");
    if !skip {
        let has_header = req.headers().get("authorization").is_some();
        let has_query = req.uri().query().map(|q| q.contains("token=")).unwrap_or(false);
        if !has_header && !has_query {
            warn!(uri=%req.uri(), "missing auth");
            return Response::builder().status(401).body(axum::body::Body::empty()).unwrap();
        }
    }
    next.run(req).await
}
