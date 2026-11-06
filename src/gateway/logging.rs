use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
pub fn init() {
    let fmt = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_line_number(true);
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt)
        .init();
}
