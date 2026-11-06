use std::sync::Arc;
use crate::gateway::gateway_config::GatewayConfig;

pub async fn check_auth(_cfg: Arc<GatewayConfig>, auth_header: Option<&str>) -> bool {
    auth_header.is_some()
}
