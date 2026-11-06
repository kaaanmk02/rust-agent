use rust_agent::gateway::{
    cli::{Cli, Commands},
    server::serve,
};
use anyhow::Result;
use clap::Parser;

fn init_metrics() {
    rust_agent::metrics::init_metrics();
}

#[tokio::main]
async fn main() -> Result<()> {
    rust_agent::gateway::logging::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { listen, config, .. } => {
            let mut cfg = rust_agent::gateway::GatewayConfig::default();
            if let Some(addr) = listen { cfg.listen = addr; }
            if let Some(path) = config {
                if let Ok(data) = std::fs::read_to_string(path) {
                    if let Ok(partial) = serde_json::from_str::<rust_agent::gateway::GatewayConfig>(&data) { cfg = partial; }
                }
            }
            init_metrics();
            tokio::select! {
                r = serve(cfg) => r?,
                _ = tokio::signal::ctrl_c() => { tracing::info!("shutdown signal"); }
            }
            Ok(())
        }
        Commands::Send { channel, to, text, url, token } => {
            let base = format!("{}/api/chat", url.trim_end_matches('/'));
            let body = serde_json::json!({"channel": channel, "chat_id": to, "text": text});
            let mut req_builder = reqwest::Client::new().post(base).json(&body);
            if let Some(token) = token {
                req_builder = req_builder.query(&[("token", token)]);
            }
            let res = req_builder.send().await?;
            let status = res.status();
            let body = res.text().await?;
            println!("status={} body={}", status, body);
            Ok(())
        }
        Commands::Tools { url, token } => {
            let mut req_builder = reqwest::Client::new().get(format!("{}/api/tools", url.trim_end_matches('/')));
            if let Some(token) = token {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
            }
            let res = req_builder.send().await?;
            let status = res.status();
            let body = res.text().await?;
            println!("status={} body={}", status, body);
            Ok(())
        }
        Commands::ToolsExecute { url, tool, args, token } => {
            let mut body = serde_json::json!({"tool": tool});
            if let Some(args_str) = args {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&args_str) {
                    body["args"] = parsed;
                } else {
                    body["args"] = serde_json::Value::String(args_str);
                }
            }
            let mut req_builder = reqwest::Client::new().post(format!("{}/api/tools/execute", url.trim_end_matches('/'))).json(&body);
            if let Some(token) = token {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
            }
            let res = req_builder.send().await?;
            let status = res.status();
            let body = res.text().await?;
            println!("status={} body={}", status, body);
            Ok(())
        }
        Commands::Config { file, set } => {
            let path = file.unwrap_or_else(|| std::path::PathBuf::from("gateway.json"));
            if !set.is_empty() {
                if let Ok(data) = std::fs::read_to_string(&path) {
                    if let Ok(mut cfg) = serde_json::from_str::<serde_json::Value>(&data) {
                        if let Some(obj) = cfg.as_object_mut() {
                            for kv in set {
                                if let Some((k, v)) = kv.split_once('=') {
                                    obj.insert(k.into(), serde_json::Value::String(v.into()));
                                }
                            }
                        }
                        if let Ok(json) = serde_json::to_string_pretty(&cfg) {
                            let _ = std::fs::write(&path, json);
                            println!("updated config at {:?}", path);
                            return Ok(());
                        }
                    }
                }
                println!("config update failed");
                return Ok(());
            }
            match std::fs::read_to_string(&path) {
                Ok(data) => println!("{}", data),
                Err(_) => println!("no config at {:?}", path),
            }
            Ok(())
        }
        Commands::Chat { url, channel, to, token } => {
            let base = format!("{}/api/chat", url.trim_end_matches('/'));
            let client = reqwest::Client::new();
            println!("terminal chat ready; type a line and press enter, Ctrl+C to exit");
            loop {
                use std::io::{self, Write};
                print!("> ");
                let _ = io::stdout().flush();
                let mut input = String::new();
                let _ = io::stdin().read_line(&mut input);
                let text = input.trim_end_matches(['\n','\r']).to_string();
                if text.is_empty() { continue; }
                if text.eq_ignore_ascii_case("quit") || text.eq_ignore_ascii_case("exit") { break; }
                let body = serde_json::json!({"channel": channel, "chat_id": to, "text": text});
                let mut req_builder = client.post(&base).json(&body);
                if let Some(token) = &token {
                    req_builder = req_builder.query(&[("token", token)]);
                }
                let res = req_builder.send().await?;
                let status = res.status();
                let body = res.text().await?;
                println!("status={} body={}", status, body);
            }
            Ok(())
        }
    }
}
