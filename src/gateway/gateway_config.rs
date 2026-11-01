use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub enabled: bool,
    pub listen: String,
    pub auth_token: Option<String>,
    pub telegram: Option<TelegramConfig>,
    pub whatsapp: Option<WhatsAppConfig>,
    pub discord: Option<DiscordConfig>,
    pub slack: Option<SlackConfig>,
    pub mcp: Option<McpConfig>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen: "0.0.0.0:9090".to_string(),
            auth_token: None,
            telegram: None,
            whatsapp: None,
            discord: None,
            slack: None,
            mcp: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub allowed_user_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    pub enabled: bool,
    pub session_name: String,
    pub endpoint: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub application_id: String,
    pub guild_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub signing_secret: String,
    pub app_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub allowed_tools: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_config_disabled_adapters() {
        let cfg = GatewayConfig::default();
        assert!(cfg.telegram.is_none());
        assert!(cfg.whatsapp.is_none());
        assert!(cfg.discord.is_none());
        assert!(cfg.slack.is_none());
        assert!(cfg.mcp.is_none());
    }
    #[test]
    fn set_config_overrides_channel() {
        let mut cfg = GatewayConfig::default();
        cfg.telegram = Some(TelegramConfig {
            enabled: true,
            bot_token: "123:ABC".into(),
            allowed_user_ids: vec![1, 2],
        });
        assert!(cfg.telegram.as_ref().unwrap().enabled);
        assert_eq!(cfg.telegram.as_ref().unwrap().allowed_user_ids.len(), 2);
    }
}
