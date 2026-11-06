use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct McpClient {
    pub endpoint: String,
    pub client: Client,
    pub allowed_tools: Vec<String>,
}

impl McpClient {
    pub fn new(endpoint: impl Into<String>, allowed_tools: Vec<String>) -> Self {
        Self { endpoint: endpoint.into(), client: Client::new(), allowed_tools }
    }

    pub async fn call(&self, tool: &str, args: Value) -> Result<Value> {
        if !self.allowed_tools.is_empty() && !self.allowed_tools.contains(&tool.to_string()) {
            anyhow::bail!("tool not allowed: {}", tool);
        }
        let body = serde_json::json!({"tool": tool, "arguments": args});
        let resp = self.client.post(&self.endpoint).json(&body).send().await?;
        let v: Value = resp.json().await?;
        Ok(v)
    }
}
