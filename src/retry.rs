use std::time::Duration;
use backoff::ExponentialBackoff;

pub async fn with_retry<F, T, E>(mut f: F) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Display,
{
    let backoff = ExponentialBackoff::default();
    backoff::future::retry(backoff, || f()).await
}

pub fn retry_backoff_default() -> ExponentialBackoff {
    ExponentialBackoff::default()
}
